use std::sync::Arc;

use async_trait::async_trait;
use simple_db::{DbContext, PostgresDriver};
use simple_db_test_lib::{
    harness::TestHarness,
    runner::{RunnerConfig, TestRunner},
};
use sqlx::postgres::PgPoolOptions;

/// Connection URL from `POSTGRES_URL`, `DATABASE_URL`, or a local default.
///
/// Example:
///   POSTGRES_URL="postgres://user:pass@localhost:5432/testdb" cargo run --bin simple-db-postgres-test
fn connection_url() -> String {
    std::env::var("POSTGRES_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/simple_db_test".into())
}

/// Test harness for the PostgreSQL driver.
///
/// Reads the connection URL from the environment and drops/recreates the `users`
/// table on each [`create_context`] call so every test starts from a clean state.
struct PostgresHarness {
    url: String,
}

impl PostgresHarness {
    fn new() -> Self {
        Self { url: connection_url() }
    }
}

#[async_trait]
impl TestHarness for PostgresHarness {
    fn driver_name(&self) -> &str {
        "PostgreSQL"
    }

    async fn create_context(&self) -> DbContext {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.url)
            .await
            .expect("Failed to connect to PostgreSQL — set POSTGRES_URL env var");

        sqlx::query("DROP TABLE IF EXISTS users CASCADE")
            .execute(&pool)
            .await
            .expect("Failed to drop users table");

        sqlx::query(
            "CREATE TABLE users (
                id      BIGSERIAL        PRIMARY KEY,
                name    TEXT             NOT NULL,
                email   TEXT             NOT NULL,
                age     INTEGER,
                active  BOOLEAN          NOT NULL DEFAULT TRUE,
                balance DOUBLE PRECISION,
                bio     TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

        DbContext::new(Arc::new(PostgresDriver::new(pool)))
    }
}

#[tokio::main]
async fn main() {
    let config = RunnerConfig::from_args();
    let runner = TestRunner::new(PostgresHarness::new(), config);
    let report = runner.run().await;
    report.print();
}
