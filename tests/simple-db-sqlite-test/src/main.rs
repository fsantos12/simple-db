use std::sync::Arc;

use simple_db::{DbContext, SqliteDriver};
use simple_db_test_lib::run_test_cases;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
    // --- DATABASE SETUP ---
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://:memory:")
        .await
        .expect("Failed to open in-memory SQLite database");

    sqlx::query(
        "CREATE TABLE users (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            name    TEXT    NOT NULL,
            email   TEXT    NOT NULL,
            age     INTEGER,
            active  INTEGER NOT NULL DEFAULT 1,
            balance REAL,
            bio     TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(SqliteDriver::new(pool)));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
