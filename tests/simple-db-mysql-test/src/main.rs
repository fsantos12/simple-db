use std::sync::Arc;

use simple_db::{DbContext, MysqlDriver}; 
use simple_db_test_lib::run_test_cases;
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() {
    // --- DATABASE SETUP ---
    let database_url = "mysql://root:root@localhost:3306/simple_db_test";

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to MySQL");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id      INT PRIMARY KEY AUTO_INCREMENT,
            name    VARCHAR(255) NOT NULL,
            email   VARCHAR(255) NOT NULL,
            age     INT,
            active  TINYINT(1)   NOT NULL DEFAULT 1,
            balance DOUBLE,
            bio     TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(MysqlDriver::new(pool)));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
