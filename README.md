# Prism-DB

Prism-DB is a modular database toolkit for Rust, built around a clean query-builder layer, pluggable SQL drivers, and a shared core API. It is designed to keep application code consistent across SQLite, PostgreSQL, and MySQL while leaning on Rust's type system and SQLx for safe, predictable database access.

## What lives where

- `prism-db-core`: shared query builders, driver traits, and core types
- `prism-db-orm`: entity tracking and persistence helpers
- `prism-db-macros`: derive macros for entity support
- `prism-db-sqlite`, `prism-db-postgres`, `prism-db-mysql`: SQLx-backed drivers
- `prism-db`: public facade that re-exports the pieces most applications need

## Install

Enable only the pieces you need in `Cargo.toml`:

```toml
[dependencies]
prism-db = { version = "0.1", features = ["sqlite", "orm"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Driver features are optional:

- `sqlite`
- `postgres`
- `mysql`
- `orm`

## Builders and Queries

Prism-DB exposes fluent builders for projections, filters, sorts, and groups. The `Query` helper gives you a simple entry point for composing CRUD statements.

### Build a `SELECT`

```rust
use prism_db::Query;

let query = Query::find("users")
    .with_projection_builder(|b| b.field("id").field("name").count_all())
    .with_filter_builder(|b| b.eq("status", "active").gte("score", 10))
    .with_sort_builder(|b| b.desc("created_at"))
    .with_group_builder(|b| b.field("organization_id"))
    .limit(25)
    .offset(0);
```

### Build filters directly

```rust
use prism_db::query::FilterBuilder;

let filters = FilterBuilder::new()
    .is_not_null("email")
    .contains("name", "alice")
    .between("age", 18, 30)
    .build();
```

### Query shapes

```rust
use prism_db::Query;

let select_users = Query::find("users");
let insert_user = Query::insert("users");
let update_user = Query::update("users");
let delete_user = Query::delete("users");
```

## Drivers

Each driver implements the shared executor and transaction traits, so the same query-building code works across the supported SQL backends.

### SQLite

```rust
use prism_db::{Query, SqliteDriver};

let driver = SqliteDriver::connect("sqlite::memory:").await?;

let query = Query::find("users").limit(10);
let mut cursor = driver.find(query).await?;
```

### PostgreSQL

```rust
use prism_db::{PostgresDriver, Query};

let driver = PostgresDriver::connect("postgres://localhost/prism_db").await?;
let rows = driver.find(Query::find("users").limit(10)).await?;
```

### MySQL

```rust
use prism_db::{MySqlDriver, Query};

let driver = MySqlDriver::connect("mysql://localhost/prism_db").await?;
let affected = driver.insert(Query::insert("users")).await?;
```

### Transactions

Drivers also expose transaction support through the shared driver trait, so you can keep atomic work inside the same abstraction boundary.

## Roadmap

- Expand SQL feature coverage and query ergonomics
- Strengthen ORM helpers and entity persistence flows
- Improve documentation and examples for production use
- Add more end-to-end driver coverage in tests

## Why Prism-DB

If you want a small, explicit database layer instead of a large framework, Prism-DB gives you:

- a shared query-building model
- driver-specific SQLx implementations
- optional ORM helpers
- a facade crate that keeps imports simple

## License

MIT
