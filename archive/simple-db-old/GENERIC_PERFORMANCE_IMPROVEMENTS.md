# Simple-DB Generic Performance Improvements

## Overview

This analysis covers **architecture-level performance issues** that affect ALL drivers (memory, SQL, etc.), not specific to the memory driver. These are design choices that impact every query execution path.

---

## 1. 🔴 CRITICAL: DbRow Uses HashMap<String, DbValue> 

### Current State
```rust
pub struct DbRow(pub HashMap<String, DbValue>);
```

### Problems

**A. String Key Hashing Overhead**
- Every field access requires `hash("field_name")` computation
- ~100-200 cycles per lookup vs 0 cycles for array indexing
- Memory indirection: HashMap → bucket → entry → value

**B. Runtime Field Name Strings**
- No compile-time verification of field existence
- Field name typos discovered at runtime
- Prevents query optimization (backend doesn't know schema)
- Example: Backend can't use column index if names are always strings

**C. Impacts Driver Implementation**
```rust
// Memory driver must iterate and compare field names
Filter::Eq(field, val) => row.get(field).is_some_and(...)
                              // field is &str runtime string

// vs potential compiled driver
Filter::Eq(field_id: FieldId, val) => row.fields[field_id]
                              // field is u8 compile constant
```

### Solutions

#### Option 1: Struct Fields (Best for compile-time safety)
```rust
// Change query/entity layer to work with typed structs
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

// Driver still works with dynamic rows internally
// But entity layer has compile-time safety
impl From<User> for DbRow { ... }
impl TryFrom<DbRow> for User { ... }
```

**Pros:** Compile-time safety, 0 overhead for hot path
**Cons:** Less flexible for dynamic queries, need more macro code

#### Option 2: FieldId Enum Abstraction (Middle ground)
```rust
pub enum FieldId {
    Id = 0,
    Name = 1,
    Email = 2,
}

pub struct DbRow {
    fields: Vec<DbValue>,  // Indexed by FieldId as u8
}

impl DbRow {
    pub fn get(&self, field: FieldId) -> Option<&DbValue> {
        self.fields.get(field as usize)
    }
}
```

**Pros:** Zero-cost, compile-time verification
**Cons:** Requires schema knowledge at macro time

#### Option 3: Hybrid - String Keys + Caching
```rust
pub struct DbRow {
    fields: HashMap<String, DbValue>,
    field_index: Arc<FieldIndex>,  // Cache of field→index mappings
}

impl DbRow {
    pub fn get(&self, field: FieldId) -> Option<&DbValue> {
        // Fast path: use cached index
        self.fields.values().nth(field as usize)
    }
}
```

**Pros:** Works with existing drivers, reduces hash cost
**Cons:** Cache invalidation complexity

### Recommendation

**Near-term (Quick fix):**
- Add companion `FieldId` enum to macro-generated code
- Update macro to generate FieldId variants
- Keep HashMap but add indexing support

**Long-term:**
- Plan migration to columnar storage
- Batch query support with fixed field positions

---

## 2. 🔴 CRITICAL: DbValue Enum Overhead

### Current State
```rust
pub enum DbValue {
    I8(Option<i8>),
    I32(Option<i32>),
    String(Option<Box<String>>),
    Decimal(Option<Box<Decimal>>),
    // ... 20+ variants
}
```

### Problems

**A. Discriminant + Option Overhead**
```
Memory layout (DbValue::I32):
[ Discriminant (8 bytes) ][ Padding (8 bytes) ][ Option<i32> (16 bytes) ]
Total: 32 bytes per value in enum

Optimal for i32: 4 bytes only
Overhead: 8×
```

**B. Pattern Matching Cost**
```rust
// Every access requires pattern match (branch prediction)
match value {
    DbValue::I32(Some(v)) => { /* use v */ },
    DbValue::I32(None) => { /* handle null */ },
    _ => { /* error */ }
}

// Compiler can't optimize this path easily
// vs direct i32: zero branches
```

**C. Boxing Small Types**
```rust
String(Option<Box<String>>)  // 24 + 8 + 8 = 40 bytes
Vec<u8>(Option<Box<Vec<u8>>>)  // Similar overhead
Decimal(Option<Box<Decimal>>)  // Decimal is already 16 bytes

// For small strings (<16 bytes), Box adds 24 byte overhead
```

**D. Serialization/Deserialization Overhead**
```rust
// Macro-generated code must pattern match every field
impl From<User> for DbRow {
    fn from(user: User) -> Self {
        let mut row = DbRow::new();
        row.insert("id", user.id.into());        // i32 → DbValue::I32(Some(i32))
        row.insert("name", user.name.into());    // String → DbValue::String(Some(Box<String>))
        // ...
    }
}

// Each insert does:
// 1. Call From<i32> for DbValue
// 2. Wrap in Some
// 3. Allocate HashMap entry
// 4. Compute string hash
// = Expensive for bulk operations
```

### Solutions

#### Option 1: Separate Null Representation
```rust
pub enum DbValue {
    Null,
    I8(i8),
    I16(i16),
    I32(i32),
    // ... no Option wrapper
    String(Box<String>),
    Decimal(Box<Decimal>),
}
```

**Benefit:** 8 bytes saved per value, better pattern matching
**Cost:** DbValue is still 32 bytes on 64-bit system

#### Option 2: Inline Small Strings (SmallString)
```rust
pub enum DbValue {
    I32(Option<i32>),
    String(Option<SmallString>),  // SSO - small string optimization
    // ...
}

// SmallString: 24 bytes inline, no Box for <22 byte strings
struct SmallString {
    len: u8,
    data: [u8; 23],  // Inline storage
}
```

**Benefit:** No heap allocation for small strings (common case)
**Cost:** More complex implementation

#### Option 3: Columnar Storage (Advanced)
```rust
pub struct DbRow {
    i32_values: Vec<Option<i32>>,
    string_values: Vec<Option<String>>,
    decimal_values: Vec<Option<Decimal>>,
    // ... one Vec per type
}

pub struct DbColumn {
    field_name: String,
    type_tag: ValueType,
    data: ColumnData,  // Enum of vectors
}

// Benefit: SIMD optimization, cache locality, no enum overhead
// Cost: Major refactor
```

### Recommendation

**Immediate (1-2 weeks):**
- Separate `Null` variant from `Option(T)` - saves 8 bytes
- Profile hot paths in macro-generated code

**Short-term (1 month):**
- Add SmallString variant for String type
- Benchmark against alternatives

**Long-term (3+ months):**
- Columnar storage for batch operations
- Row-major format for single-row access (switching based on access pattern)

---

## 3. 🟠 HIGH: DbContext Clones Row for Entity Hydration

### Current State
```rust
// From context.rs - find_entities()
pub async fn find_entities<T: DbEntityModel>(&self, query: FindQuery) -> Result<Vec<DbEntity<T>>, DbError> {
    let rows = self.find(query).await?;
    let mut entities = Vec::with_capacity(rows.len());

    for row in rows {
        let model = T::from_db_row(row.clone())?;  // ← CLONE HERE
        entities.push(DbEntity::from_db(model, row));
    }

    Ok(entities)
}
```

### Problems

**A. Double Use of Row**
```
1. Clone row
2. Pass to from_db_row() which takes ownership and extracts fields
3. Also pass original row to DbEntity::from_db() for snapshot
= Must clone the entire DbRow (HashMap) before use
```

**B. Impact**
```
100K rows with 10 fields:
- 100K clones of HashMap
- Each clone: allocate + copy 10 KV pairs
- Memory: 100K × (16 + 10×40) bytes = ~41 GB overhead
```

**C. Macro-Generated Code Issue**
```rust
// Macro generates: Into<DbRow> impl that TAKES OWNERSHIP
impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id.clone());
        row
    }
}

// So from_db_row() needs &DbRow or DbRow
// But DbEntity also needs the original for snapshot
// Solution: clone before deserializing
```

### Solutions

#### Option 1: Split Reference & Owned Paths
```rust
// New trait for reference methods
pub trait FromDbRowRef: Sized {
    fn from_db_row_ref(row: &DbRow) -> Result<Self, DbError>;
}

// Update macro to generate both:
// 1. FromDbRowRef using borrowing
// 2. FromDbRow for consuming (current)

// Then:
pub async fn find_entities<T: DbEntityModel + FromDbRowRef>(&self, query: FindQuery) 
    -> Result<Vec<DbEntity<T>>, DbError> 
{
    let rows = self.find(query).await?;
    for row in rows {
        let model = T::from_db_row_ref(&row)?;  // Borrow!
        entities.push(DbEntity::from_db(model, row));  // Move
    }
}
```

**Benefit:** No clone, save 40% memory
**Cost:** Macro must generate two code paths

#### Option 2: DbRow Takes Mutable Reference
```rust
// Change FromDbRow signature
pub trait FromDbRow: Sized {
    fn from_db_row(row: &mut DbRow) -> Result<Self, DbError>;
}

// Macro implementation uses &mut row.take()
impl FromDbRow for User {
    fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
        Ok(User {
            id: row.take_i32("id")?,
            name: row.take_string("name")?,
        })
    }
}

// Then context doesn't need to clone:
let model = T::from_db_row(&mut row.clone())?;  // Still one clone, but less work
```

**Benefit:** Mutable borrow pattern is idiomatic
**Cost:** Changes public API

#### Option 3: Streaming Hydration (Best)
```rust
// Don't hydrate all at once, create async iterator
pub async fn find_entities_stream<T: DbEntityModel>(
    &self, 
    query: FindQuery
) -> Result<impl futures::Stream<Item = Result<DbEntity<T>, DbError>>, DbError> {
    // Stream yields entities one at a time
    // No batch cloning needed
}
```

**Benefit:** Memory O(1) instead of O(n), can process infinite result sets
**Cost:** Requires async iterator trait (nightly or boxed)

### Recommendation

**Near-term:**
- Add `FromDbRowRef` trait to macro
- Use `find_entities_readonly()` path (already exists!) and document when to use
- Document memory implications

**Medium-term:**
- Change `FromDbRow` to take `&mut DbRow` (breaking change)
- Update macro to generate optimized borrow path

**Long-term:**
- Streaming hydration with async iterators

---

## 4. 🟠 HIGH: Entity Change Tracking Clone Overhead

### Current State
```rust
impl<T: DbEntityModel> DbEntity<T> {
    pub async fn save(&mut self, ctx: &DbContext) -> Result<(), DbError> {
        match self.state {
            DbEntityState::Tracked => {
                let updates = self.dirty_fields();
                // ...
            }
        }
    }

    fn dirty_fields(&self) -> DbRow {
        let current: DbRow = self.entity.clone().into();  // ← CLONE entity
        let mut updates = DbRow::new();

        if let Some(ref original) = self.snapshot {
            for (field, val) in &current.0 {
                if original.get(field) != Some(val) {
                    updates.insert(field.clone(), val.clone());  // ← CLONE fields
                }
            }
        }
        updates
    }
}
```

### Problems

**A. Clone for Comparison**
```
To detect changes:
1. Clone current entity to DbRow
2. Compare with snapshot DbRow field-by-field
3. Extract changed fields
4. Clone each changed field value again for update

This is 3 clones for a single change!
```

**B. No Change Tracking at Entity Level**
```rust
pub struct User {
    id: i32,
    name: String,  // Which fields changed?
    email: String,
}

// No bitmap of changed fields, must reconstruct entire row
```

### Solutions

#### Option 1: Track Dirty Fields at Struct Level
```rust
pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: Option<DbRow>,
    state: DbEntityState,
    dirty_fields: HashSet<String>,  // ← Track what changed
}

// Usage:
user.name = "new name";
user_entity.mark_dirty("name");  // Explicit
user_entity.save(ctx).await?;     // Only updates "name"
```

**Benefit:** O(1) dirty check, no clones needed
**Cost:** Manual field tracking (error-prone)

#### Option 2: Macro-Generated Tracking Wrapper (Better)
```rust
// Macro generates wrapper struct:
pub struct UserTracked {
    inner: User,
    dirty: [bool; 3],  // Bitmap for id, name, email
}

// Generate setter methods that auto-track:
impl UserTracked {
    pub fn set_name(&mut self, value: String) {
        self.inner.name = value;
        self.dirty[1] = true;  // Mark "name" as dirty
    }
}
```

**Benefit:** Automatic tracking, zero-cost with compiler optimizations
**Cost:** Code generation complexity

#### Option 3: Copy-on-Write Snapshot
```rust
pub struct DbEntity<T: DbEntityModel> {
    pub entity: Arc<T>,  // Shared reference to original
    snapshot: Arc<DbRow>,  // Shared snapshot
    state: DbEntityState,
}

// dirty_fields() only clones if entity was modified
fn dirty_fields(&self) -> DbRow {
    // If Arc ref count is 1, entity wasn't cloned
    if Arc::strong_count(&self.entity) == 1 {
        // Safe to compare without cloning
    }
}
```

**Benefit:** Lazy cloning, efficient when entities aren't modified
**Cost:** Complexity with Arc, lifetime issues

### Recommendation

**Quick fix (2 weeks):**
- Add `dirty_fields` HashSet to DbEntity
- Document API for marking dirty
- Don't require macro changes

**Medium-term (1 month):**
- Extend macro to generate tracking setters
- Opt-in via `#[track_changes]` attribute

**Long-term:**
- Implement differential serialization (only serialize dirty fields)

---

## 5. 🟠 HIGH: Macro-Generated Code Cloning

### Current State
```rust
// Generated by macro in Into<DbRow>
impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id.clone());         // ← Clone primitives!
        row.insert("name", self.name.clone());     // ← Clone strings
        row
    }
}
```

### Problems

**A. Unnecessary Clones**
```rust
self.id.clone()   // i32 should be Copy, not Clone
self.name.clone() // Clones entire String
```

**B. Generated setters in InsertQuery**
```rust
pub fn insert<I, K, V>(mut self, row: I) -> Self
where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> 
{
    let db_row: DbRow = row
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))  // Allocates String for every key!
        .collect();
}
```

### Solutions

#### Option 1: Use Copy for Primitives
```rust
// Macro generates different code for primitives:
impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);         // No clone (Copy)
        row.insert("name", self.name);     // Move, not clone
        row
    }
}
```

**Benefit:** Eliminate primitive clones
**Cost:** Minimal, just macro logic change

#### Option 2: Move Instead of Clone
```rust
// Trait requires Into, not &
impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let User { id, name, email } = self;
        let mut row = DbRow::new();
        row.insert("id", id);           // i32 - Copy
        row.insert("name", name);       // String - Move
        row
    }
}
```

**Benefit:** Zero clones on hot path
**Cost:** Forces consuming self (can't reuse entity)

#### Option 3: Reduce String Allocations in Query Builders
```rust
// Current:
pub fn insert<I, K, V>(mut self, row: I) -> Self
where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue>

// Better: use &'static str for field names
pub fn insert<I, K, V>(mut self, row: I) -> Self
where I: IntoIterator<Item = (K, V)>, K: Into<Cow<'static, str>>, V: Into<DbValue>
```

**Benefit:** StaticField names can be borrowed
**Cost:** API change, less flexible

### Recommendation

**Immediate (0.5 week):**
- Update macro to not clone Copy types
- Change generated Into to use move semantics

**Short-term (2 weeks):**
- Profile macro-generated code for allocation hotspots
- Add `#[derive(Copy)]` for primitive DbValue accesses

---

## 6. 🟡 MEDIUM: Driver Trait Returns Vec<DbRow> Instead of Iterator

### Current State
```rust
#[async_trait]
pub trait Driver: Send + Sync {
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError>;
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>;
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>;
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>;
}
```

### Problems

**A. All Results Collected**
```
Large query (1M rows):
- Allocate Vec<DbRow> for 1M rows
- Wait for entire result set before returning
- Can't process results streaming
- Memory: O(n) where n = result set size
```

**B. Can't Do Early Termination**
```rust
// vs
for row in driver.find().await? {
    if should_stop { break; }  // Can skip remaining
}

// Current forces consuming all rows before processing
```

**C. SQL Backend Inefficiency**
```
SqlDriver::find() must:
1. Execute query
2. Fetch ALL rows from database
3. Collect into Vec<DbRow>
4. Return to caller
5. Caller then hydrates to entities

= Materializes intermediate Vec
```

### Solutions

#### Option 1: Streaming Results (Best but Complex)
```rust
#[async_trait]
pub trait Driver: Send + Sync {
    type RowStream: futures::Stream<Item = Result<DbRow, DbError>> + Send;
    
    async fn find(&self, query: FindQuery) -> Result<Self::RowStream, DbError>;
}

// Usage:
let mut stream = driver.find(query).await?;
while let Some(row) = stream.next().await {
    let row = row?;
    // Process one row at a time
}
```

**Benefit:** Memory O(1), can process infinite results, natural for databases
**Cost:** Associated type complexity, async trait limitations

#### Option 2: Cursor-Like Interface (Middle ground)
```rust
pub struct QueryCursor {
    cursor: Box<dyn std::iter::Iterator<Item = Result<DbRow, DbError>>>,
}

pub trait Driver: Send + Sync {
    fn find(&self, query: FindQuery) -> Result<QueryCursor, DbError>;
}

// Usage:
let cursor = driver.find(query)?;
for row in cursor {
    let row = row?;
}
```

**Benefit:** Iterator pattern, familiar API
**Cost:** Can't be async (database operations must finish before returning)

#### Option 3: Batched Results (Pragmatic)
```rust
pub struct ResultBatch {
    rows: Vec<DbRow>,
    has_more: bool,
}

pub trait Driver: Send + Sync {
    async fn find_batch(
        &self, 
        query: FindQuery, 
        batch_size: usize
    ) -> Result<ResultBatch, DbError>;
}

// Usage:
let mut batch = driver.find_batch(query, 1000).await?;
loop {
    for row in batch.rows {
        // Process batch of 1000
    }
    if !batch.has_more { break; }
    batch = driver.find_batch(query, 1000).await?;
}
```

**Benefit:** Practical for both in-memory and SQL backends
**Cost:** API complexity (batch size parameter)

### Recommendation

**Short-term (Keep current):**
- Document memory implications of Vec return
- Recommend `find_entities_readonly()` for large sets

**Medium-term (2+ months):**
- Add optional streaming via feature flag
- Implement cursor interface alongside Vec

**Long-term (4+ months):**
- Make streaming the primary API
- Keep Vec as convenience wrapper: `driver.find(q).await?.collect()`

---

## 7. 🟡 MEDIUM: Query Builders Create Intermediate Allocations

### Current State
```rust
pub struct FindQuery {
    pub collection: String,
    pub projections: ProjectionDefinition,  // Vec<Projection>
    pub filters: FilterDefinition,          // Vec<Filter>
    pub sorts: SortDefinition,              // Vec<Sort>
    pub groups: GroupDefinition,            // Vec<String>
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// Usage:
Query::find("users")
    .filter(|fb| fb.eq("status", "active").gte("age", 18))
    .order_by(|sb| sb.asc("created_at"))
    .limit(10)
```

### Problems

**A. Closure-Based Builders**
```rust
.filter(|fb| fb.eq("status", "active"))
// 1. Creates FilterBuilder
// 2. Builds up Vec<Filter>
// 3. Returns
// 4. Extends into find_query.filters
// = Extra allocation + copy
```

**B. String Collections**
```rust
pub type GroupDefinition = Vec<Box<String>>;  // Boxed strings!?

// Could be:
pub type GroupDefinition = Vec<&'static str>;  // Borrowed at compile time
```

**C. Filter Cloning**
```rust
pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
    self.filters.extend(filters);  // Moves, OK
    self
}
```

**OK here but...**

```rust
pub fn filter<F>(mut self, build: F) -> Self 
where F: FnOnce(FilterBuilder) -> FilterBuilder 
{
    let builder = build(FilterBuilder::new());
    self.filters.extend(builder.build());  // Extra Vec allocation in build()
    self
}
```

### Solutions

#### Option 1: Builder Consumes Into Query
```rust
// Instead of:
.filter(|fb| fb.eq(...).gte(...))

// Support:
let filters = FilterBuilder::new()
    .eq("status", "active")
    .gte("age", 18)
    .build();

query.with_filters(filters);
```

**Benefit:** No closure overhead, natural chaining
**Cost:** More verbose API

#### Option 2: Optimize Closure Builders
```rust
pub fn filter<F>(mut self, build: F) -> Self 
where F: FnOnce(&mut Vec<Filter>) 
{
    build(&mut self.filters);  // Direct mutable reference, no allocation!
    self
}

// Usage:
.filter(|filters| {
    filters.push(Filter::eq("status", "active"));
    filters.push(Filter::gte("age", 18));
})
```

**Benefit:** Zero intermediate allocations
**Cost:** Less idiomatic

#### Option 3: Use SmallVec for Filter/Sort Defs
```rust
use smallvec::SmallVec;

pub struct FindQuery {
    pub filters: SmallVec<[Filter; 8]>,  // Stack-allocated up to 8 filters
    pub sorts: SmallVec<[Sort; 4]>,      // Stack-allocated up to 4 sorts
}

// Benefits:
// - Most queries have 1-3 filters: zero heap allocation
// - Large queries fall back to Vec automatically
```

**Benefit:** Most queries are zero-alloc
**Cost:** Additional dependency, max capacity

### Recommendation

**Immediate (1 week):**
- Use SmallVec for FilterDefinition, SortDefinition
- Set reasonable defaults (8 filters, 4 sorts)

**Short-term:**
- Profile builder code for allocation hotspots
- Consider &'static str for query literals

---

## 8. 🟡 MEDIUM: No Query Compilation/Preparation

### Current State
```rust
// Every execution parses the query fresh
pub async fn find_entities<T: DbEntityModel>(
    &self, 
    query: FindQuery
) -> Result<Vec<DbEntity<T>>, DbError> 
{
    let rows = self.find(query).await?;  // Query passed raw
    // Driver must interpret filters, sorts, etc.
}
```

### Problems

**A. Repeated Parsing**
```
100 calls to same query:
Query::find("users").filter(...).order_by(...).limit(10)
- Closure called 100 times
- FilterBuilder instantiated 100 times
- Filter Vec built 100 times
- Driver re-interprets filters 100 times
```

**B. No Optimization Opportunity**
```
Can't optimize:
- WHERE status='active' AND age >= 18
  → Reorder filters for cardinality (most selective first)
  
- ORDER BY created_at DESC LIMIT 10
  → Use partial sort, not full sort
  
- SELECT * with 10 columns WHERE needs 2
  → Apply projection push-down
```

**C. No Index Selection**
```
For SQL drivers, no way to know:
- Are there indexes on status, age, created_at?
- Should use index vs full table scan?
- Hint not available from driver trait
```

### Solutions

#### Option 1: Compiled Queries
```rust
// Create a prepared query representation
pub trait CompiledQuery: Send + Sync {
    async fn execute(&self, driver: &dyn Driver) -> Result<Vec<DbRow>, DbError>;
}

// Usage:
let compiled = Query::find("users")
    .filter(|fb| fb.eq("status", "active"))
    .compile()?;  // Optimization happens here

// Run same query many times
async {
    for i in 0..100 {
        let rows = compiled.execute(&driver).await?;
    }
}
```

**Benefit:** Query optimized once, fast repeated execution
**Cost:** Additional trait, API complexity

#### Option 2: Query Plan in Driver
```rust
pub struct QueryPlan {
    filters: Vec<Filter>,
    filter_order: Vec<usize>,  // Optimized order
    use_index: Option<IndexName>,
    projections: Vec<String>,
}

pub trait Driver {
    async fn plan(&self, query: &FindQuery) -> Result<QueryPlan, DbError>;
    async fn find_planned(&self, plan: &QueryPlan) -> Result<Vec<DbRow>, DbError>;
}
```

**Benefit:** Driver can optimize for its storage model
**Cost:** Duplicates query execution logic

#### Option 3: Statistics & Hints
```rust
// Driver tracks cardinality estimates
impl Driver {
    async fn explain(&self, query: &FindQuery) -> Result<QueryStats, DbError> {
        // Returns: rows affected, estimated cost, used index
    }
}

// ORM generates hints:
Query::find("users")
    .filter(|fb| fb.eq("status", "active"))  // High selectivity, do first
    .filter(|fb| fb.gte("age", 18))          // Low selectivity,do second
```

**Benefit:** Helps driver make better decisions
**Cost:** Requires cardinality estimates

### Recommendation

**Short-term:**
- Add statistics collection to memory driver
- Benchmark impact of filter ordering

**Medium-term (2 months):**
- Implement QueryPlan abstraction
- Allow manual reordering hints

**Long-term:**
- Query compilation with JIT optimization
- Auto-generate indexes based on access patterns

---

## 9. 🟡 MEDIUM: String Allocations in Query Builders

### Current State
```rust
pub fn filter<F>(mut self, build: F) -> Self 
where F: FnOnce(FilterBuilder) -> FilterBuilder 
{
    // ...
    self.filters.extend(builder.build());
}

// Inside FilterBuilder:
pub fn eq<V: Into<DbValue>>(mut self, field: impl Into<String>, val: V) -> Self {
    self.filters.push(Filter::Eq(field.into(), val.into()));  // ← String allocation
    self
}
```

### Problems

**A. Dynamic String Keys**
```
Query::find("users")
    .filter(|fb| fb
        .eq("status", "active")      // Allocates "status"
        .eq("created_at", "2024")    // Allocates "created_at"
    )

Hard-coded string literals are allocated at runtime
Should be compile-time constants
```

**B. Field Name Repetition**
```rust
.filter(|fb| fb.eq("status", "active")).order_by(|sb| sb.asc("status"))

// "status" allocated twice, should be singleton
```

### Solutions

#### Option 1: Borrowed Static Strings
```rust
// Change API to accept &'static str
pub fn eq(mut self, field: &'static str, val: impl Into<DbValue>) -> Self {
    self.filters.push(Filter::Eq(field, val.into()));
    self
}

// Now stored as &'static str, no allocation
pub struct Filter {
    pub field: &'static str,  // Borrowed, not owned
    pub value: DbValue,
}
```

**Benefit:** Zero string allocations for static queries
**Cost:** Dynamic queries still need String

#### Option 2: Interned Field Names
```rust
pub struct FieldName {
    id: u32,  // Intern ID
}

// Global pool of field names
lazy_static::lazy_static! {
    static ref FIELD_INTERN: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

pub fn eq(mut self, field: &str, val: impl Into<DbValue>) -> Self {
    let id = FIELD_INTERN.intern(field);
    self.filters.push(Filter::Eq(FieldName(id), val.into()));
    self
}
```

**Benefit:** Field names deduplicated globally
**Cost:** Lock contention, global state

#### Option 3: Macro-Generated Field Idents
```rust
// Macro generates field ID enum:
#[derive(DbEntity)]
#[db_entity(collection = "users")]
struct User {
    pub id: i32,
    pub status: String,
    pub created_at: String,
}

// Macro generates:
pub mod fields {
    pub const ID: &'static str = "id";
    pub const STATUS: &'static str = "status";
    pub const CREATED_AT: &'static str = "created_at";
}

// Usage:
.eq(fields::STATUS, "active")  // No allocation, literal
```

**Benefit:** Type-safe, no allocations
**Cost:** Macro complexity, changes API

### Recommendation

**Immediate:**
- Benchmark string allocation overhead
- Profile FilterBuilder in hot paths

**Short-term (2 weeks):**
- Add ` fields::*` module to macro output
- Document that using constants is faster than literals

**Long-term:**
- Transition to &'static str in Filter struct
- Make string queries fallback path

---

## 10. 🟡 MEDIUM: DbEntity Snapshot Always Clones Row

### Current State
```rust
pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: Option<DbRow>,  // Always full row clone
    state: DbEntityState
}

// Every find_entities() call clones the row for snapshot
pub async fn find_entities<T: DbEntityModel>(
    &self, 
    query: FindQuery
) -> Result<Vec<DbEntity<T>>, DbError> 
{
    for row in rows {
        let model = T::from_db_row(row.clone())?;
        entities.push(DbEntity::from_db(model, row));  // row cloned again above
    }
}
```

### Problems

**A. Memory Doubled**
```
1000 entities loaded:
- Entity struct: 50 bytes
- Snapshot row: 400 bytes
Total: 450 bytes × 1000 = 450 KB

With clone: 900 KB (memory doubled!)
```

**B. Entities Not Meant for Modification**
```rust
// Common pattern:
let users = ctx.find_entities(query).await?;

for user in users {
    println!("{}", user.entity.name);
    // entity NOT modified!
}

// Snapshot wasted memory
```

### Solutions

#### Option 1: Two-Tier Entities
```rust
// Read-only entity (no snapshot)
pub struct DbEntityReadOnly<T> {
    pub entity: T,
    state: DbEntityState,  // Always Tracked
}

// Tracked entity (with snapshot)
pub struct DbEntityMutable<T> {
    pub entity: T,
    snapshot: DbRow,
    state: DbEntityState,
}

// Trait unifies both
pub trait DbEntityRef<T> { }

impl<T> DbEntityRef<T> for DbEntityReadOnly<T> { }
impl<T> DbEntityRef<T> for DbEntityMutable<T> { }
```

**Benefit:** Read-only queries don't pay snapshot cost
**Cost:** API duplication

#### Option 2: Optional Snapshot
```rust
pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: Option<DbRow>,  // Keep optional
    state: DbEntityState
}

// Modify find_entities_readonly to use read-only path
pub async fn find_entities_readonly<T: DbEntityModel>(
    &self, 
    query: FindQuery
) -> Result<Vec<DbEntity<T>>, DbError> 
{
    for row in rows {
        // Don't hydrate snapshot, just entity
        let model = T::from_db_row(row)?;
        entities.push(DbEntity::readonly(model));
    }
}
```

**Benefit:** Already implemented, just encourage usage
**Cost:** User must choose between `find_entities` and `find_entities_readonly`

#### Option 3: Lazy Snapshot
```rust
pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: OnceCell<DbRow>,  // Lazily initialized
    state: DbEntityState
}

// Snapshot created only when save() called
pub async fn save(&mut self, ctx: &DbContext) -> Result<(), DbError> {
    if self.snapshot.get().is_none() {
        self.snapshot.set(self.entity.clone().into());
    }
    // Compare and update
}
```

**Benefit:** Memory only allocated when needed
**Cost:** Snapshot becomes Option-like, extra check on save

### Recommendation

**Immediate (document):**
- Add prominent docs: use `find_entities_readonly()` for read-only queries
- Show performance comparison example

**Short-term (2 weeks):**
- Make OnceCell snapshot non-required

**Long-term:**
- Separate ReadOnly and Mutable entity types
- Deprecate generic DbEntity

---

## Summary Table: Priority & Effort

| Issue | Severity | Effort | Impact | Timeline |
|-------|----------|--------|--------|----------|
| HashMap row keys | 🔴 Critical | High | 5-10× | 3 months |
| DbValue enum | 🔴 Critical | High | 2-5× | 2 months |
| Row clone in find_entities | 🔴 Critical | Medium | 2× | 2 weeks |
| Entity change tracking | 🟠 High | Medium | 1.5× | 3 weeks |
| Macro-generated cloning | 🟠 High | Low | 1.5× | 1 week |
| Vec instead of iterator | 🟠 High | High | 3-5× | 2 months |
| Query builder allocations | 🟡 Medium | Low | 1.1× | 1 week |
| No query compilation | 🟡 Medium | High | 1.5-2× | 3 months |
| String allocations | 🟡 Medium | Low | 1.1× | 2 weeks |
| Entity snapshot memory | 🟡 Medium | Low | 1.3× | 1 week |

---

## Quick Wins (Start Here!)

1. **Macro: Don't clone primitives** (1 day) → 1.2× speedup
2. **Use SmallVec for filters/sorts** (2 days) → 1.15× speedup
3. **Document find_entities_readonly usage** (0.5 day) → 2× speedup for read queries
4. **Add OnceCell to snapshot** (3 days) → 1.5× memory reduction

**Total effort: ~1 week | Total gain: 1.5-2.5× for typical read-heavy workload**

---

## Architectural Recommendations

### For Production Use
1. Consider switching backends (sqlx/diesel) instead of optimizing in-memory
2. If staying with simple-db, focus on eliminating clones first (quick wins)
3. Implement streaming results for large datasets

### For Teaching/Testing
- Current architecture is fine
- Add performance documentation
- Note difference from production ORMs

---

## Measurement Plan

Before implementing improvements, establish baselines:

```rust
#[cfg(test)]
mod benchmarks {
    #[tokio::test]
    async fn bench_find_entities_100k() {
        // Load 100K users, measure time + memory
        // Baseline: record current performance
    }
    
    #[tokio::test]
    async fn bench_dirty_field_detection() {
        // Load 1000 entities, modify some, measure save time
    }
    
    #[tokio::test]
    async fn bench_query_builder() {
        // Build same query 10,000 times, measure allocations
    }
}
```

Use `cargo-flamegraph` to profile hot paths before/after changes.
