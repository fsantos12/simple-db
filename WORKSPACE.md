# simple-db Workspace

A Rust workspace containing the simple-db ORM library and procedural macros for automatic entity implementation.

## Project Structure

```
simple-db/
├── Cargo.toml (Workspace root)
├── simple-db/                           # Main ORM library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── context.rs
│   │   ├── entity/
│   │   ├── driver/
│   │   ├── query/
│   │   └── types/
│   └── tests/
│       ├── integration_crud.rs
│       ├── integration_entity.rs
│       └── integration_filters.rs
└── simple-db-macro/                     # Procedural macros crate
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

## Crates

### simple-db
The main ORM library providing:
- Type-safe query builder (Find, Insert, Update, Delete)
- Entity state management (Added, Tracked, Deleted)
- Change tracking and dirty field detection
- Memory driver with pluggable architecture
- Async/await support with Tokio

### simple-db-macro
Procedural macros for automatic entity implementation:
- `#[derive(DbEntity)]` - Automatically implements `DbEntityModel`, `FromDbRow`, and `Into<DbRow>`

## Usage

### Manual Implementation (Current)
```rust
use simple_db::{
    entity::{DbEntity, DbEntityModel},
    types::{DbRow, DbValue, FromDbRow},
    DbContext,
};

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
}

impl FromDbRow for User {
    fn from_db_row(mut row: DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,
            name: row.take_string("name")?,
        })
    }
}

impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);
        row.insert("name", self.name);
        row
    }
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> DbEntityKey {
        vec![("id".to_string(), DbValue::I32(Some(self.id)))]
    }
}
```

### Macro Implementation (Future)
```rust
use simple_db::{
    entity::DbEntity,
    DbContext,
    DbEntity as _,  // Import the derive macro
};

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "users", key = "id")]
struct User {
    id: i32,
    name: String,
}
```

## Building

```bash
# Build entire workspace
cargo build

# Build specific crate
cargo build -p simple-db
cargo build -p simple-db-macro

# Run tests
cargo test

# Run tests for specific crate
cargo test -p simple-db
```

## Next Steps for Macro Development

1. **Type Inference**: Enhance the macro to automatically infer `take_i32()`, `take_string()`, etc. based on field types
2. **Attribute Options**: Support optional attributes for field-level customization
3. **Composite Keys**: Handle multi-field primary keys
4. **Relationships**: Add support for foreign keys and relationships
5. **Validation**: Add attribute-based field validation

## Testing

All 115 tests pass:
- 64 unit tests in simple-db
- 51 integration tests across multiple files
- 1 doc test (ignored)

```bash
cargo test --all
```

---

**Version**: 0.1.0  
**Edition**: 2021  
**License**: MIT
