use std::sync::Arc;

use async_trait::async_trait;
use simple_db::{DbContext, SqliteDriver};
use simple_db_test_lib::{
    harness::TestHarness,
    runner::{RunnerConfig, TestRunner},
};
use sqlx::sqlite::SqlitePoolOptions;

/// Test harness for the SQLite driver using an in-memory database.
///
/// Each call to [`create_context`] creates a fresh in-memory SQLite database
/// with the `users` table so every test starts from a clean state.
struct SqliteHarness;

#[async_trait]
impl TestHarness for SqliteHarness {
    fn driver_name(&self) -> &str {
        "SQLite (in-memory)"
    }

    async fn create_context(&self) -> DbContext {
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

        DbContext::new(Arc::new(SqliteDriver::new(pool)))
    }
}

#[tokio::main]
async fn main() {
    let config = RunnerConfig::from_args();
    let runner = TestRunner::new(SqliteHarness, config);
    let report = runner.run().await;
    report.print();
}
