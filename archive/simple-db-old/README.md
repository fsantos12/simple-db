# simple-db

A lightweight, type-safe ORM for Rust with automatic query building, entity tracking, and procedural macros.

## What is simple-db?

simple-db is an **Object-Relational Mapping** (ORM) library that makes it easy to:
- Define database entities (tables) as Rust structs
- Query and manipulate data with a fluent API
- Track changes to entities automatically
- Use in-memory or pluggable database drivers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
simple-db = { path = "./simple-db" }
simple-db-macro = { path = "./simple-db-macro" }
tokio = { version = "1.51", features = ["full"] }
```

## Quick Start

### 1. Define an Entity (Struct)

Use the `#[db_entity]` attribute macro to automatically implement database functionality:

```rust
use simple_db_macro::db_entity;

#[db_entity(collection = "users")]
pub struct User {
    #[primary_key]
    pub id: i32,
    pub name: String,
    pub email: String,
}
```

That's it! The macro automatically:
- Adds `#[derive(Clone, Debug)]`
- Implements database serialization/deserialization
- Implements entity tracking for changes

### 2. Create and Save Data

```rust
use simple_db::{DbContext, driver::memory::MemoryDriver, entity::DbEntity};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Create a new user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Wrap in DbEntity and save
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();
}
```

### 3. Query Data

```rust
use simple_db::query::Query;

// Find users by ID
let query = Query::find("users")
    .filter(|fb| fb.eq("id", 1));
let results = ctx.find(query).await.unwrap();

// Read-only queries (no tracking overhead)
let users = ctx.find_entities_readonly::<User>(query).await.unwrap();

// Tracked queries (for updates/deletes)
let mut tracked = ctx.find_entities::<User>(query).await.unwrap();
```

### 4. Update Data

```rust
// Load existing user
let query = Query::find("users").filter(|fb| fb.eq("id", 1));
let mut entities = ctx.find_entities::<User>(query).await.unwrap();

if let Some(mut entity) = entities.into_iter().next() {
    // Modify
    entity.entity.email = "alice.new@example.com".to_string();
    
    // Save changes (only changed fields are updated)
    entity.save(&ctx).await.unwrap();
}
```

### 5. Delete Data

```rust
// Load and delete
let query = Query::find("users").filter(|fb| fb.eq("id", 1));
let mut entities = ctx.find_entities::<User>(query).await.unwrap();

if let Some(entity) = entities.into_iter().next() {
    entity.delete(&ctx).await.unwrap();
}
```

## Entity Macro

The `#[db_entity]` attribute macro simplifies entity definition:

```rust
#[db_entity(collection = "users")]
pub struct User {
    #[primary_key]        // Mark as primary key
    pub id: i32,
    pub name: String,
    pub email: String,
}
```

**Supported Field Types:**
- Primitive: `i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool, char`
- Temporal: `chrono::NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
- Special: `String`, `Decimal`, `Uuid`, `Vec<u8>`, `serde_json::Value`

**Composite Keys:**
```rust
#[db_entity(collection = "user_roles")]
pub struct UserRole {
    #[primary_key]
    pub user_id: i32,
    #[primary_key]
    pub role_id: i32,
    pub name: String,
}
```

## Entity Lifecycle

Entities have four states:

1. **Added** - New entity, not yet in database
2. **Tracked** - Loaded from database, changes are tracked
3. **Deleted** - Entity removed from database
4. (No **Detached** needed - just don't use `DbEntity` wrapper for read-only queries)

```rust
let mut entity = DbEntity::new(user);           // State: Added
entity.save(&ctx).await.unwrap();              // State: Tracked (after save)
entity.delete(&ctx).await.unwrap();            // State: Deleted (after delete)
```

## Querying

### Basic Find

```rust
let query = Query::find("users");
let rows = ctx.find(query).await.unwrap();
```

### Filtering

```rust
let query = Query::find("users")
    .filter(|fb| fb.eq("id", 1))                // Equals
    .filter(|fb| fb.gte("age", 18));            // Greater than or equal
```

### Read-Only vs Tracked

```rust
// Read-only: No tracking overhead, no state management
let users = ctx.find_entities_readonly::<User>(query).await?;
for user in users {
    println!("{}", user.name);
}

// Tracked: For modifications
let mut entities = ctx.find_entities::<User>(query).await?;
for mut entity in entities {
    entity.entity.name = "Updated".to_string();
    entity.save(&ctx).await?;
}
```

## Project Structure

```
simple-db/
├── Cargo.toml (workspace root)
│
├── simple-db/                    # Main ORM library
│   ├── src/
│   │   ├── lib.rs (entry point)
│   │   ├── context.rs (DbContext - main API)
│   │   ├── entity/ (entity state management)
│   │   ├── query/ (query builder)
│   │   ├── driver/ (database drivers)
│   │   └── types/ (value types)
│   └── tests/
│       ├── integration_crud.rs
│       ├── integration_entity.rs
│       ├── integration_filters.rs
│       └── integration_macro.rs
│
├── simple-db-macro/              # Procedural macros
│   └── src/lib.rs (db_entity macro)
│
└── examples/
    └── db_entity_macro_example.rs
```

## Testing

Run all tests:

```bash
cargo test --lib --tests --all
```

Run specific test file:

```bash
cargo test --test integration_macro
```

Current test status: **118 tests passing ✅**

## How It Works

### Change Tracking

When you load an entity with `.from_db()`, the ORM keeps a "snapshot" of the original data:

```
Original: User { id: 1, name: "Alice", email: "alice@example.com" }
↓
Load into DbEntity
↓
Modified: User { id: 1, name: "Alice", email: "alice.new@example.com" }
↓
Save detects: email changed
↓
Only updates email column in database (efficient!)
```

### Ownership & `take_*` Methods

When deserializing from `DbRow`, the macro uses `take_*` methods which **move** values:

```rust
// ✅ Efficient - no cloning
let user = row.take_string("name")?;  // String is moved out

// ❌ Less efficient - creates clone
let user = row.get_string("name")?.clone();
```

This is intentional and good for performance!

## Common Patterns

### Create and Save

```rust
let user = User { id: 1, name: "Bob".to_string(), email: "bob@example.com".to_string() };
let mut entity = DbEntity::new(user);
entity.save(&ctx).await?;
```

### Load, Update, Save

```rust
let query = Query::find("users").filter(|fb| fb.eq("id", 1));
let mut entities = ctx.find_entities::<User>(query).await?;

if let Some(mut entity) = entities.into_iter().next() {
    entity.entity.email = "updated@example.com".to_string();
    entity.save(&ctx).await?;
}
```

### Delete

```rust
let query = Query::find("users").filter(|fb| fb.eq("id", 1));
let mut entities = ctx.find_entities::<User>(query).await?;

if let Some(entity) = entities.into_iter().next() {
    entity.delete(&ctx).await?;
}
```

### Bulk Query

```rust
let query = Query::find("users")
    .filter(|fb| fb.gte("age", 18));
let adult_users = ctx.find_entities_readonly::<User>(query).await?;
```

## Features

✅ Type-safe query builder  
✅ Automatic entity serialization/deserialization  
✅ Change tracking and dirty field detection  
✅ Composite key support  
✅ Read-only query optimization  
✅ Both attribute and derive macros  
✅ Async/await with Tokio  
✅ In-memory and pluggable drivers  
✅ Comprehensive test suite (118 tests)

## Architecture

```
DbContext
    ↓
Query Builder (Find, Insert, Update, Delete)
    ↓
Driver (Memory/Custom)
    ↓
DbRow ↔ DbEntity<T> ↔ Your Struct
```

## Next Steps

1. **Try the macro** - Define your first entity with `#[db_entity]`
2. **Load data** - Use `find_entities` or `find_entities_readonly`
3. **Modify** - Change entity fields and call `save()`
4. **Delete** - Use `delete()` to remove records

## License

MIT

---

**Happy coding!** 🚀
