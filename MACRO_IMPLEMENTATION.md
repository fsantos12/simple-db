# DbEntity Derive Macro - Implementation Guide

## ✅ Complete Implementation

Your macro now supports:

1. **Field-level `#[db_entity(primary_key)]` attributes**
   - Mark individual fields as primary keys
   - Support for composite keys (multiple primary keys)

2. **Proper type inference for ALL `DbRow` methods**
   - Primitive types: `i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool, char`
   - Temporal types: `NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>`
   - Boxed types: `String, Decimal, Uuid, Vec<u8>, serde_json::Value`

3. **Simplified attribute syntax**
   - Struct: `#[db_entity(collection = "table_name")]` (only collection name)
   - Fields: `#[db_entity(primary_key)]` (only marker)

## New Syntax

```rust
#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "users")]
pub struct User {
    #[db_entity(primary_key)]  // ← Marks as PK
    pub id: i32,
    pub username: String,
    pub email: String,
}

// Composite key:
#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "user_roles")]
pub struct UserRole {
    #[db_entity(primary_key)]
    pub user_id: i32,
    #[db_entity(primary_key)]
    pub role_id: i32,
    pub assigned_at: String,
}
```

## About Ownership & take_* Methods

### Why is ownership loss GOOD? ✅

When you use `take_string("field")` or `take_decimal("field")`, the value is **removed** from the DbRow HashMap and **moved** into your struct. This is excellent for several reasons:

#### 1. **Safety - Prevents Accidental Double-Use**
```rust
// ❌ Impossible with take_*:
let name1 = row.take_string("name")?;
let name2 = row.take_string("name")?;  // ERROR: Field already taken!

// ✅ With get_string (gives reference):
let name1 = row.get_string("name")?;
let name2 = row.get_string("name")?;   // OK but you need to clone one
```

#### 2. **Performance - Zero-Copy for Large Types**
```rust
// DbValue::String(Box<String>) - uses Box to store efficiently
// 
// take_string() unboxes and MOVES:
let name = row.take_string("name")?;  // No clone, just Box unwrap
//
// get_string() gives &String reference:
let name = row.get_string("name")?;   // Reference to Box<String>
let owned = name.clone();              // Must clone if you want to own it
```

#### 3. **Idiomatic Rust - Consumes Input**
```rust
impl FromDbRow for User {
    fn from_db_row(mut row: DbRow) -> Result<Self, DbError> {
        // We take ownership of row, so using take_* is natural
        // The row will be dropped after FromDbRow returns anyway
        Ok(Self {
            id: row.take_i32("id")?,      // Move value out
            username: row.take_string("username")?,  // Move String out
            email: row.take_string("email")?,        // Move String out
        })
        // row dropped here - but all fields already extracted!
    }
}
```

#### 4. **Serialization Mirror - Symmetry**
```rust
// Deserialization (FromDbRow) uses take_*:
impl FromDbRow for User {
    fn from_db_row(mut row: DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,           // Extract values
            name: row.take_string("name")?,    // owned values
        })
    }
}

// Serialization (Into<DbRow>) uses insert:
impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);           // Insert values
        row.insert("name", self.name);       // consumed values (moved)
        row
    }
}
```

### Memory Layout Comparison

```
┌─── DbRow ───────────────────────────────┐
│ HashMap { "id": I32(1), "name": ... }   │
└─────────────────────────────────────────┘

Operation 1: get_string("name")
  - Returns &Box<String>
  - Row still owns the Box
  - Clone needed for ownership

Operation 2: take_string("name")
  - Unboxes and moves String
  - Row no longer holds it
  - Zero copies!
```

### Real-World Impact: Parsing CSV

```rust
// Loading 1000 users from CSV with take_*:
// - 1000 String fields: Zero clones
// - 1000 i32 ids: Copied inline (cheap)
// - 1000 Decimal prices: Unboxed (no clones)
// Total allocations: Just the Strings you own

// With get_* and clone:
// - 1000 String fields: 1000 clones!
// - Memory bandwidth wasted
```

## Type Mapping Reference

| Rust Type | DbRow Deserialize | DbRow Serialize | Notes |
|-----------|-------------------|-----------------|-------|
| `i32` | `take_i32()` | `insert("field", val)` | Copied inline |
| `i64` | `take_i64()` | `insert("field", val)` | Copied inline |
| `f64` | `take_f64()` | `insert("field", val)` | Copied inline |
| `bool` | `take_bool()` | `insert("field", val)` | Copied inline |
| `String` | `take_string()` | `insert("field", val)` | Unboxed & moved |
| `Decimal` | `take_decimal()` | `insert("field", val)` | Unboxed & moved |
| `Uuid` | `take_uuid()` | `insert("field", val)` | Unboxed & moved |
| `Vec<u8>` | `take_bytes()` | `insert("field", val)` | Unboxed & moved |
| `DateTime<Utc>` | `take_timestamptz()` | `insert("field", val)` | Copied inline |
| `NaiveDate` | `take_date()` | `insert("field", val)` | Copied inline |

## Generated Code Example

For this struct:
```rust
#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "users")]
pub struct User {
    #[db_entity(primary_key)]
    pub id: i32,
    pub name: String,
    pub price: Decimal,
}
```

The macro generates:
```rust
impl FromDbRow for User {
    fn from_db_row(mut row: DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,         // Primitive: copied
            name: row.take_string("name")?,  // Boxed: unboxed & moved
            price: row.take_decimal("price")?,  // Boxed: unboxed & moved
        })
    }
}

impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);
        row.insert("name", self.name);     // String moved into row
        row.insert("price", self.price);   // Decimal moved into row
        row
    }
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> DbEntityKey {
        vec![
            ("id".to_string(), DbValue::from(self.id.clone())),
        ]
    }
}
```

## Performance Characteristics

### Time Complexity
- Deserialization: **O(n)** where n = number of fields (one HashMap lookup per field)
- Serialization: **O(n)** same reason

### Space Complexity
- No temporary allocations during conversion
- Large types (String, Decimal) use existing allocations - zero copies
- Small types (primitives) are copied inline

### Typical Scenario: Loading 10,000 users

**With take_* (our macro):**
```
Allocations: ~10,000 (one String per user)
Copies: 10,000 i32 IDs (each 4 bytes) = 40KB
Memory bandwidth: Minimal
```

**With get_* + clone:**
```
Allocations: ~20,000 (DB string + cloned string)
Copies: All strings duplicated!
Memory bandwidth: Heavy traffic
```

---

## Testing Your Macro

Run the example:
```bash
cargo run --example macro_usage_example
```

The macro generation is transparent - you just use the standard FromDbRow/Into<DbRow>/DbEntityModel traits that the macro implements! ✅
