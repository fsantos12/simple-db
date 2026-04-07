use simple_db::{
    driver::memory::MemoryDriver,
    types::{DbRow, DbError, FromDbRow},
    query::Query,
    DbContext,
};
use std::sync::Arc;

#[tokio::test]
async fn test_complex_filter_scenarios() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // Test null checking
    let mut row_with_null = DbRow::new();
    row_with_null.insert("id", 1i32);
    row_with_null.insert("name", "Alice");
    row_with_null.insert("email", None::<String>);

    let query = Query::find("users")
        .filter(|fb| fb.is_null("email"));

    assert_eq!(query.filters.len(), 1);
}

#[tokio::test]
async fn test_string_filtering() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // Insert test data
    let mut row1 = DbRow::new();
    row1.insert("id", 1i32);
    row1.insert("username", "alice_smith");

    let mut row2 = DbRow::new();
    row2.insert("id", 2i32);
    row2.insert("username", "bob_johnson");

    // Test startswith filter
    let query = Query::find("users")
        .filter(|fb| fb.starts_with("username", "alice"));

    assert_eq!(query.filters.len(), 1);
}

#[tokio::test]
async fn test_range_filtering() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // Test between filter
    let query = Query::find("users")
        .filter(|fb| fb.between("age", 18, 65));

    assert_eq!(query.filters.len(), 1);
}

#[tokio::test]
async fn test_set_membership_filtering() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    // Test in filter
    let query = Query::find("users")
        .filter(|fb| fb.is_in("status", vec!["active", "pending"]));

    assert_eq!(query.filters.len(), 1);

    // Test not_in filter
    let query = Query::find("users")
        .filter(|fb| fb.not_in("status", vec!["deleted", "archived"]));

    assert_eq!(query.filters.len(), 1);
}

#[tokio::test]
async fn test_numeric_comparisons() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver.clone());

    let query = Query::find("products")
        .filter(|fb| fb.lt("price", 100.0))
        .filter(|fb| fb.lte("price", 100.0))
        .filter(|fb| fb.gt("stock", 0))
        .filter(|fb| fb.gte("stock", 10));

    assert_eq!(query.filters.len(), 4);
}

#[tokio::test]
async fn test_combined_filters_with_and_or() {
    let query = Query::find("users")
        .filter(|fb| fb.and(|inner| {
            inner
                .eq("status", "active")
                .gte("age", 18)
        }))
        .filter(|fb| fb.or(|inner| {
            inner
                .eq("role", "admin")
                .eq("role", "moderator")
        }));

    assert_eq!(query.filters.len(), 2);
}

#[tokio::test]
async fn test_ordering_combinations() {
    let query = Query::find("orders")
        .order_by(|sb| sb.asc("created_at").desc("total_price").asc("customer_name"));

    assert_eq!(query.sorts.len(), 3);
}

#[tokio::test]
async fn test_limit_offset_edge_cases() {
    let query1 = Query::find("users").limit(0);
    assert_eq!(query1.limit, Some(0));

    let query2 = Query::find("users").limit(1000000);
    assert_eq!(query2.limit, Some(1000000));

    let query3 = Query::find("users").offset(0);
    assert_eq!(query3.offset, Some(0));
}

#[tokio::test]
async fn test_row_type_conversions() {
    let mut row = DbRow::new();
    row.insert("i32_val", 42i32);
    row.insert("i64_val", 9223372036854775807i64);
    row.insert("f64_val", 3.14159f64);
    row.insert("string_val", "hello");
    row.insert("bool_val", true);

    assert!(row.get_i32("i32_val").is_ok());
    assert!(row.get_i64("i64_val").is_ok());
    assert!(row.get_f64("f64_val").is_ok());
    assert!(row.get_string("string_val").is_ok());
    assert!(row.get_bool("bool_val").is_ok());
}

#[tokio::test]
async fn test_row_null_handling() {
    let mut row = DbRow::new();
    row.insert::<&str, _>("nullable_int", None::<i32>);
    row.insert::<&str, _>("nullable_string", None::<String>);

    let int_result = row.get_i32("nullable_int");
    assert!(int_result.is_err());

    let string_result = row.get_string("nullable_string");
    assert!(string_result.is_err());
}

#[tokio::test]
async fn test_row_type_mismatch_errors() {
    let mut row = DbRow::new();
    row.insert("name", "Alice");

    // Try to get as i32 when it's a string
    let result = row.get_i32("name");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_row_missing_field() {
    let row = DbRow::new();

    let result = row.get_i32("nonexistent");
    assert!(result.is_err());

    let result = row.get_string("nonexistent");
    assert!(result.is_err());
}
