use std::sync::Arc;

use simple_db::{DbContext, PostgresDriver}; 
use simple_db_test_lib::run_test_cases;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    // --- DATABASE SETUP ---
    let database_url = "postgres://postgres:postgres@localhost:5432/simple_db_test";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::query(
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
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(PostgresDriver::new(pool)));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
