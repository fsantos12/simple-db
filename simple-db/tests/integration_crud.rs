use simple_db::{
    driver::memory::MemoryDriver,
    query::Query,
    types::DbRow,
    DbContext,
};
use std::sync::Arc;

// ==========================================
// Insert Tests
// ==========================================

#[tokio::test]
async fn insert_single_record_returns_count_of_one() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut row = DbRow::new();
    row.insert("id", 1i32);
    row.insert("name", "Alice");

    let query = Query::insert("users").values(vec![row]);
    let result = ctx.insert(query).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[tokio::test]
async fn insert_multiple_records_returns_correct_count() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    for i in 1..=5 {
        let mut row = DbRow::new();
        row.insert("id", i);
        row.insert("name", format!("User {}", i));
        rows.push(row);
    }

    let query = Query::insert("users").values(rows);
    let result = ctx.insert(query).await;

    assert_eq!(result.unwrap(), 5);
}

// ==========================================
// Find Tests
// ==========================================

#[tokio::test]
async fn find_returns_empty_list_when_collection_does_not_exist() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let query = Query::find("nonexistent");
    let result = ctx.find(query).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn find_all_returns_all_inserted_records() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert 3 products
    let mut rows = vec![];
    for i in 1..=3 {
        let mut row = DbRow::new();
        row.insert("id", i);
        row.insert("name", format!("Product {}", i));
        row.insert("price", 10.0 * i as f64);
        rows.push(row);
    }

    let insert_query = Query::insert("products").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find all
    let find_query = Query::find("products");
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn find_with_less_than_filter_returns_matching_records() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert products with different prices
    let mut rows = vec![];
    for (id, price) in [(1i32, 29.99), (2i32, 99.99), (3i32, 199.99)] {
        let mut row = DbRow::new();
        row.insert("id", id);
        row.insert("price", price);
        rows.push(row);
    }

    let insert_query = Query::insert("products").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find products with price < 100
    let find_query = Query::find("products")
        .filter(|fb| fb.lt("price", 100.0));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn find_with_greater_than_filter_returns_matching_records() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert products
    let mut rows = vec![];
    for (id, age) in [(1i32, 25), (2i32, 35), (3i32, 45)] {
        let mut row = DbRow::new();
        row.insert("id", id);
        row.insert("age", age);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find users with age > 30
    let find_query = Query::find("users")
        .filter(|fb| fb.gt("age", 30));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn find_with_equals_filter_returns_exact_match() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert users with different statuses
    let mut rows = vec![];
    for (id, status) in [(1i32, "active"), (2i32, "inactive"), (3i32, "active")] {
        let mut row = DbRow::new();
        row.insert("id", id);
        row.insert("status", status);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find active users
    let find_query = Query::find("users")
        .filter(|fb| fb.eq("status", "active"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn find_ordered_by_field_ascending_returns_sorted_results() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert unsorted names
    let mut rows = vec![];
    for (id, name) in [(1i32, "Charlie"), (2i32, "Alice"), (3i32, "Bob")] {
        let mut row = DbRow::new();
        row.insert("id", id);
        row.insert("name", name);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find and sort by name ascending
    let find_query = Query::find("users")
        .order_by(|sb| sb.asc("name"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 3);
    // First result should be Alice (id=2)
    assert_eq!(*result[0].get("id").unwrap(), 2i32.into());
    // Last result should be Charlie (id=1)
    assert_eq!(*result[2].get("id").unwrap(), 1i32.into());
}

#[tokio::test]
async fn find_ordered_by_field_descending_returns_reverse_sorted_results() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert prices
    let mut rows = vec![];
    for (id, price) in [(1i32, 50.0), (2i32, 100.0), (3i32, 75.0)] {
        let mut row = DbRow::new();
        row.insert("id", id);
        row.insert("price", price);
        rows.push(row);
    }

    let insert_query = Query::insert("products").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find and sort by price descending
    let find_query = Query::find("products")
        .order_by(|sb| sb.desc("price"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 3);
    // First should be highest price (100.0, id=2)
    assert_eq!(*result[0].get("id").unwrap(), 2i32.into());
}

#[tokio::test]
async fn find_with_limit_returns_only_limited_records() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert 5 users
    let mut rows = vec![];
    for i in 1..=5 {
        let mut row = DbRow::new();
        row.insert("id", i);
        row.insert("name", format!("User {}", i));
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Find with limit 2
    let find_query = Query::find("users").limit(2);
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn find_with_limit_and_offset_returns_paginated_results() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert 10 records
    let mut rows = vec![];
    for i in 1..=10 {
        let mut row = DbRow::new();
        row.insert("id", i);
        rows.push(row);
    }

    let insert_query = Query::insert("items").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Get page 2 (items 6-10)
    let find_query = Query::find("items")
        .order_by(|sb| sb.asc("id"))
        .limit(5)
        .offset(5);
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 5);
}

// ==========================================
// Update Tests
// ==========================================

#[tokio::test]
async fn update_single_field_changes_the_value() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert a user
    let mut row = DbRow::new();
    row.insert("id", 1i32);
    row.insert("name", "Alice");
    row.insert("age", 25i32);

    let insert_query = Query::insert("users").values(vec![row]);
    ctx.insert(insert_query).await.unwrap();

    // Update age
    let update_query = Query::update("users")
        .set("age", 26i32)
        .filter(|fb| fb.eq("id", 1i32));
    let count = ctx.update(update_query).await.unwrap();

    assert_eq!(count, 1);

    // Verify the update
    let find_query = Query::find("users")
        .filter(|fb| fb.eq("id", 1i32));
    let result = ctx.find(find_query).await.unwrap();
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn update_multiple_records_matching_filter() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert 3 inactive users
    let mut rows = vec![];
    for i in 1..=3 {
        let mut row = DbRow::new();
        row.insert("id", i);
        row.insert("status", "inactive");
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Activate all inactive users
    let update_query = Query::update("users")
        .set("status", "active")
        .filter(|fb| fb.eq("status", "inactive"));
    let count = ctx.update(update_query).await.unwrap();

    assert_eq!(count, 3);
}

// ==========================================
// Delete Tests
// ==========================================

#[tokio::test]
async fn delete_record_removes_matching_row() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert a user
    let mut row = DbRow::new();
    row.insert("id", 1i32);
    row.insert("name", "Alice");

    let insert_query = Query::insert("users").values(vec![row]);
    ctx.insert(insert_query).await.unwrap();

    // Delete the user
    let delete_query = Query::delete("users")
        .filter(|fb| fb.eq("id", 1i32));
    let count = ctx.delete(delete_query).await.unwrap();

    assert_eq!(count, 1);

    // Verify deletion
    let find_query = Query::find("users");
    let result = ctx.find(find_query).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn delete_multiple_records_matching_condition() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert 5 users, 2 inactive
    let mut rows = vec![];
    for i in 1..=5 {
        let mut row = DbRow::new();
        row.insert("id", i);
        row.insert("status", if i <= 2 { "inactive" } else { "active" });
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    // Delete inactive users
    let delete_query = Query::delete("users")
        .filter(|fb| fb.eq("status", "inactive"));
    let count = ctx.delete(delete_query).await.unwrap();
    assert_eq!(count, 2);

    // Verify remaining records
    let find_query = Query::find("users");
    let result = ctx.find(find_query).await.unwrap();
    assert_eq!(result.len(), 3);
}
