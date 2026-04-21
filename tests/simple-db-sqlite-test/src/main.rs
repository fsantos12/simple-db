use std::sync::Arc;

use simple_db::{DbContext, SqliteDriver};
use simple_db_test_lib::run_test_cases;

#[tokio::main]
async fn main() {
    let driver = SqliteDriver::connect("sqlite://:memory:")
        .await
        .expect("Failed to open in-memory SQLite database");

    driver
        .execute_raw(
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
        .await
        .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(driver));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
