use std::sync::Arc;

use simple_db::{DbContext, PostgresDriver};
use simple_db_test_lib::run_test_cases;

#[tokio::main]
async fn main() {
    let driver = PostgresDriver::connect("postgres://postgres:postgres@localhost:5432/simple_db_test")
        .await
        .expect("Failed to connect to PostgreSQL");

    driver
        .execute_raw(
            "CREATE TABLE IF NOT EXISTS users (
                id      SERIAL PRIMARY KEY,
                name    TEXT    NOT NULL,
                email   TEXT    NOT NULL,
                age     INTEGER,
                active  INTEGER NOT NULL DEFAULT 1,
                balance DOUBLE PRECISION,
                bio     TEXT
            )",
        )
        .await
        .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(driver));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
