use std::sync::Arc;

use simple_db::{DbContext, MysqlDriver};
use simple_db_test_lib::run_test_cases;

#[tokio::main]
async fn main() {
    let driver = MysqlDriver::connect("mysql://root:root@localhost:3306/simple_db_test")
        .await
        .expect("Failed to connect to MySQL");

    driver
        .execute_raw(
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
        .await
        .expect("Failed to create users table");

    let db_context = DbContext::new(Arc::new(driver));

    // --- RUN TEST CASES ---
    run_test_cases(&db_context).await;
}
