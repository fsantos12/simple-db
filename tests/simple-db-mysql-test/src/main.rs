use std::sync::Arc;

use async_trait::async_trait;
use simple_db::{DbContext, MysqlDriver};
use simple_db_test_lib::{
    harness::TestHarness,
    runner::{RunnerConfig, TestRunner},
};
use sqlx::mysql::MySqlPoolOptions;

/// Connection URL from `MYSQL_URL`, `DATABASE_URL`, or a local default.
///
/// Example:
///   MYSQL_URL="mysql://root:pass@localhost:3306/simple_db_test" cargo run --bin simple-db-mysql-test
fn connection_url() -> String {
    std::env::var("MYSQL_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "mysql://root:root@localhost:3306/simple_db_test".into())
}

/// Test harness for the MySQL driver.
///
/// Reads the connection URL from the environment and drops/recreates the `users`
/// table on each [`create_context`] call so every test starts from a clean state.
struct MysqlHarness {
    url: String,
}

impl MysqlHarness {
    fn new() -> Self {
        Self { url: connection_url() }
    }
}

#[async_trait]
impl TestHarness for MysqlHarness {
    fn driver_name(&self) -> &str {
        "MySQL"
    }

    async fn create_context(&self) -> DbContext {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&self.url)
            .await
            .expect("Failed to connect to MySQL — set MYSQL_URL env var");

        sqlx::query("DROP TABLE IF EXISTS users")
            .execute(&pool)
            .await
            .expect("Failed to drop users table");

        sqlx::query(
            "CREATE TABLE users (
                id      BIGINT       AUTO_INCREMENT PRIMARY KEY,
                name    TEXT         NOT NULL,
                email   TEXT         NOT NULL,
                age     INT,
                active  TINYINT(1)   NOT NULL DEFAULT 1,
                balance DOUBLE,
                bio     TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

        DbContext::new(Arc::new(MysqlDriver::new(pool)))
    }
}

#[tokio::main]
async fn main() {
    let config = RunnerConfig::from_args();
    let runner = TestRunner::new(MysqlHarness::new(), config);
    let report = runner.run().await;
    report.print();
}
