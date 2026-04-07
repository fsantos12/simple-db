use simple_db::{
    driver::memory::MemoryDriver,
    query::Query,
    types::DbRow,
    DbContext,
};
use std::sync::Arc;

// ==========================================
// Null Check Filters
// ==========================================

#[tokio::test]
async fn test_filter_is_null_matches_null_values() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    for i in 1..=3 {
        let mut row = DbRow::new();
        row.insert("id", i);
        if i <= 2 {
            row.insert("email", None::<String>);
        } else {
            row.insert("email", "user@example.com");
        }
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.is_null("email"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_is_not_null_matches_non_null_values() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    for i in 1..=3 {
        let mut row = DbRow::new();
        row.insert("id", i);
        if i == 1 {
            row.insert("phone", None::<String>);
        } else {
            row.insert("phone", format!("555-000{}", i));
        }
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.is_not_null("phone"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

// ==========================================
// Comparison Filters  
// ==========================================

#[tokio::test]
async fn test_filter_not_equals_excludes_matching_values() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let statuses = vec!["active", "pending", "active"];
    for (i, status) in statuses.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("status", *status);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.neq("status", "active"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_filter_less_than_or_equal_includes_boundary() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let ages = vec![20, 30, 40];
    for (i, age) in ages.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("age", *age);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.lte("age", 30));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_greater_than_or_equal_includes_boundary() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let scores = vec![60, 75, 85];
    for (i, score) in scores.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("score", *score);
        rows.push(row);
    }

    let insert_query = Query::insert("results").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("results")
        .filter(|fb| fb.gte("score", 75));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

// ==========================================
// Pattern Matching Filters
// ==========================================

#[tokio::test]
async fn test_filter_starts_with_matches_beginning() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let names = vec!["alice_smith", "alice_jones", "bob_smith"];
    for (i, name) in names.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("name", *name);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.starts_with("name", "alice"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_ends_with_matches_ending() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let emails = vec!["user@example.com", "admin@test.org", "dev@example.com"];
    for (i, email) in emails.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("email", *email);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.ends_with("email", ".com"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_contains_matches_substring() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let descriptions = vec!["urgent bug fix", "feature implementation", "bug in auth"];
    for (i, desc) in descriptions.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("description", *desc);
        rows.push(row);
    }

    let insert_query = Query::insert("tasks").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("tasks")
        .filter(|fb| fb.contains("description", "bug"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_not_contains_excludes_substring() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let labels = vec!["deprecated API", "active feature", "deprecated code"];
    for (i, label) in labels.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("label", *label);
        rows.push(row);
    }

    let insert_query = Query::insert("items").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("items")
        .filter(|fb| fb.not_contains("label", "deprecated"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 1);
}

// ==========================================
// Range Filters
// ==========================================

#[tokio::test]
async fn test_filter_between_includes_boundaries() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let ages = vec![10, 25, 40, 70];
    for (i, age) in ages.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("age", *age);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.between("age", 18, 65));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_filter_not_between_excludes_range() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let prices = vec![5.0, 50.0, 150.0];
    for (i, price) in prices.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("price", *price);
        rows.push(row);
    }

    let insert_query = Query::insert("products").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("products")
        .filter(|fb| fb.not_between("price", 20.0, 100.0));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}

// ==========================================
// Set Membership Filters
// ==========================================

#[tokio::test]
async fn test_filter_is_in_matches_any_value_in_list() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let statuses = vec!["pending", "active", "deleted", "active"];
    for (i, status) in statuses.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("status", *status);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.is_in("status", vec!["active", "pending"]));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_filter_not_in_excludes_values_in_list() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let countries = vec!["US", "CA", "FR", "US"];
    for (i, country) in countries.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("country", *country);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.not_in("country", vec!["US", "CA"]));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 1);
}

// ==========================================
// Combination Filters
// ==========================================

#[tokio::test]
async fn test_filter_multiple_conditions_with_and_requires_all_match() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let data = vec![(1i32, 20, "active"), (2i32, 30, "active"), (3i32, 25, "inactive")];
    for (i, (age, status)) in data.iter().map(|(_, a, s)| (a, s)).enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("age", *age);
        row.insert("status", *status);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.gte("age", 25))
        .filter(|fb| fb.eq("status", "active"));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_filter_with_logical_or_matches_either_condition() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let mut rows = vec![];
    let roles = vec!["user", "admin", "moderator", "user"];
    for (i, role) in roles.iter().enumerate() {
        let mut row = DbRow::new();
        row.insert("id", (i + 1) as i32);
        row.insert("role", *role);
        rows.push(row);
    }

    let insert_query = Query::insert("users").values(rows);
    ctx.insert(insert_query).await.unwrap();

    let find_query = Query::find("users")
        .filter(|fb| fb.or(|inner| {
            inner
                .eq("role", "admin")
                .eq("role", "moderator")
        }));
    let result = ctx.find(find_query).await.unwrap();

    assert_eq!(result.len(), 2);
}
