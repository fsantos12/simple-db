//! Full correctness test suite shared by all driver integration tests.
//!
//! Each test function accepts a freshly provisioned [`DbContext`] and returns a
//! [`TestResult`]. Tests are independent: the harness creates a new empty database
//! for every test case.
//!
//! The suite covers:
//! - Connectivity (`test_ping`)
//! - INSERT (single, multiple rows, bulk)
//! - SELECT with projections, filters, sorts, aggregations, pagination
//! - UPDATE with and without filters
//! - DELETE with and without filters
//! - Transaction commit and rollback

use simple_db::DbContext;
use simple_db::query::Query;
use simple_db::types::DbValue;

pub type TestResult = Result<(), String>;

// ─── Schema seed ─────────────────────────────────────────────────────────────
//
// Five users inserted by most tests.  Schema (created by the harness):
//
//   users (id PK AUTO, name TEXT NOT NULL, email TEXT NOT NULL,
//          age INTEGER, active BOOLEAN NOT NULL DEFAULT true,
//          balance REAL, bio TEXT)
//
// Data summary:
//   Alice   25  active  balance=150.0  bio="Loves coding"
//   Bob     30  inactive  balance=NULL  bio="Drinks coffee"
//   Charlie 25  active  balance=200.0  bio=NULL
//   Dave    35  inactive  balance=100.0  bio="Likes pizza"
//   Eve     NULL  active  balance=50.0  bio="Likes coffee"

async fn seed_users(db: &DbContext) {
    db.insert(
        Query::insert("users").bulk_insert(vec![
            vec![
                ("name", DbValue::from("Alice")),
                ("email", DbValue::from("alice@example.com")),
                ("age", DbValue::from(25i32)),
                ("active", DbValue::from(true)),
                ("balance", DbValue::from(150.0f64)),
                ("bio", DbValue::from("Loves coding")),
            ],
            vec![
                ("name", DbValue::from("Bob")),
                ("email", DbValue::from("bob@example.com")),
                ("age", DbValue::from(30i32)),
                ("active", DbValue::from(false)),
                ("balance", DbValue::from_null()),
                ("bio", DbValue::from("Drinks coffee")),
            ],
            vec![
                ("name", DbValue::from("Charlie")),
                ("email", DbValue::from("charlie@example.com")),
                ("age", DbValue::from(25i32)),
                ("active", DbValue::from(true)),
                ("balance", DbValue::from(200.0f64)),
                ("bio", DbValue::from_null()),
            ],
            vec![
                ("name", DbValue::from("Dave")),
                ("email", DbValue::from("dave@example.com")),
                ("age", DbValue::from(35i32)),
                ("active", DbValue::from(false)),
                ("balance", DbValue::from(100.0f64)),
                ("bio", DbValue::from("Likes pizza")),
            ],
            vec![
                ("name", DbValue::from("Eve")),
                ("email", DbValue::from("eve@example.com")),
                ("age", DbValue::from_null()),
                ("active", DbValue::from(true)),
                ("balance", DbValue::from(50.0f64)),
                ("bio", DbValue::from("Likes coffee")),
            ],
        ]),
    )
    .await
    .expect("seed_users failed");
}

/// Collects all rows from a cursor into a vec of rows.
async fn collect_rows(
    cursor: &mut Box<dyn simple_db::types::DbCursor>,
) -> Vec<Box<dyn simple_db::types::DbRow>> {
    let mut rows = Vec::new();
    while let Ok(Some(row)) = cursor.next().await {
        rows.push(row);
    }
    rows
}

// ─── Connectivity ────────────────────────────────────────────────────────────

pub async fn test_ping(db: &DbContext) -> TestResult {
    db.ping().await.map_err(|e| e.to_string())
}

// ─── Insert ──────────────────────────────────────────────────────────────────

pub async fn test_insert_single(db: &DbContext) -> TestResult {
    let affected = db
        .insert(Query::insert("users").insert([
            ("name", DbValue::from("Alice")),
            ("email", DbValue::from("alice@example.com")),
            ("age", DbValue::from(30i32)),
            ("active", DbValue::from(true)),
        ]))
        .await
        .map_err(|e| e.to_string())?;

    if affected != 1 {
        return Err(format!("Expected 1 row affected, got {affected}"));
    }

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let row = cursor.next().await.map_err(|e| e.to_string())?;
    if row.is_none() {
        return Err("No row found after insert".into());
    }
    Ok(())
}

pub async fn test_insert_multiple_rows(db: &DbContext) -> TestResult {
    for i in 0..3i32 {
        db.insert(Query::insert("users").insert([
            ("name", DbValue::from(format!("User {i}"))),
            ("email", DbValue::from(format!("user{i}@example.com"))),
            ("age", DbValue::from(20i32 + i)),
            ("active", DbValue::from(true)),
        ]))
        .await
        .map_err(|e| e.to_string())?;
    }

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 3 {
        return Err(format!("Expected 3 rows, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_insert_bulk(db: &DbContext) -> TestResult {
    let rows: Vec<Vec<(&str, DbValue)>> = (0..5i32)
        .map(|i| {
            vec![
                ("name", DbValue::from(format!("User {i}"))),
                ("email", DbValue::from(format!("user{i}@example.com"))),
                ("age", DbValue::from(20i32 + i)),
                ("active", DbValue::from(true)),
            ]
        })
        .collect();

    let affected = db
        .insert(Query::insert("users").bulk_insert(rows))
        .await
        .map_err(|e| e.to_string())?;

    if affected != 5 {
        return Err(format!("Expected 5 rows affected from bulk_insert, got {affected}"));
    }
    Ok(())
}

// ─── Find – basic ────────────────────────────────────────────────────────────

pub async fn test_find_all(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 5 {
        return Err(format!("Expected 5 rows, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_with_projection(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(
            Query::find("users").project(|b| b.field("name").field("email")),
        )
        .await
        .map_err(|e| e.to_string())?;

    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 5 {
        return Err(format!("Expected 5 rows, got {}", rows.len()));
    }
    // Each row should have exactly 2 columns (name, email)
    for row in &rows {
        if row.len() != 2 {
            return Err(format!("Expected 2 columns in projected row, got {}", row.len()));
        }
    }
    Ok(())
}

pub async fn test_find_with_limit(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users").limit(3))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 3 {
        return Err(format!("Expected 3 rows with limit=3, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_with_offset(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users").offset(3))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("Expected 2 rows with offset=3 (5 total), got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_with_sort_asc(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users").order_by(|b| b.asc("name")))
        .await
        .map_err(|e| e.to_string())?;

    let rows = collect_rows(&mut cursor).await;
    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.get_by_name("name").and_then(|v| v.as_string().map(|s| s.to_string())))
        .collect();

    let expected = vec!["Alice", "Bob", "Charlie", "Dave", "Eve"];
    if names != expected {
        return Err(format!("Expected ASC order {:?}, got {:?}", expected, names));
    }
    Ok(())
}

pub async fn test_find_with_sort_desc(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users").order_by(|b| b.desc("name")))
        .await
        .map_err(|e| e.to_string())?;

    let rows = collect_rows(&mut cursor).await;
    let names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.get_by_name("name").and_then(|v| v.as_string().map(|s| s.to_string())))
        .collect();

    let expected = vec!["Eve", "Dave", "Charlie", "Bob", "Alice"];
    if names != expected {
        return Err(format!("Expected DESC order {:?}, got {:?}", expected, names));
    }
    Ok(())
}

// ─── Find – filters ──────────────────────────────────────────────────────────

pub async fn test_find_filter_eq(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // name = 'Alice' → 1 row
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.eq("name", "Alice")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 1 {
        return Err(format!("eq filter: expected 1, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_neq(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age != 25 → Bob(30), Dave(35) = 2 rows (Eve's null age excluded)
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.neq("age", 25i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("neq filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_gt(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age > 25 → Bob(30), Dave(35) = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.gt("age", 25i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("gt filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_lt(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age < 30 → Alice(25), Charlie(25) = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.lt("age", 30i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("lt filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_gte(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age >= 30 → Bob(30), Dave(35) = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.gte("age", 30i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("gte filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_lte(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age <= 25 → Alice(25), Charlie(25) = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.lte("age", 25i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("lte filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_between(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age BETWEEN 25 AND 32 → Alice(25), Bob(30), Charlie(25) = 3 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.between("age", 25i32, 32i32)))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 3 {
        return Err(format!("between filter: expected 3, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_is_in(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age IN (25, 35) → Alice(25), Charlie(25), Dave(35) = 3 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.is_in("age", vec![25i32, 35i32])))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 3 {
        return Err(format!("is_in filter: expected 3, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_not_in(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age NOT IN (25, 30) → Dave(35) = 1 row (Eve's null excluded)
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.not_in("age", vec![25i32, 30i32])))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 1 {
        return Err(format!("not_in filter: expected 1, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_contains(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // name LIKE '%li%' → Alice, Charlie = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.contains("name", "li")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("contains filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_starts_with(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // name LIKE 'A%' → Alice = 1 row
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.starts_with("name", "A")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 1 {
        return Err(format!("starts_with filter: expected 1, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_ends_with(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // name LIKE '%e' → Alice, Charlie, Dave, Eve = 4 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.ends_with("name", "e")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 4 {
        return Err(format!("ends_with filter: expected 4, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_is_null(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age IS NULL → Eve = 1 row
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.is_null("age")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 1 {
        return Err(format!("is_null filter: expected 1, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_is_not_null(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // age IS NOT NULL → Alice, Bob, Charlie, Dave = 4 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.is_not_null("age")))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 4 {
        return Err(format!("is_not_null filter: expected 4, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_or(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // name='Alice' OR name='Bob' → 2 rows
    let mut cursor = db
        .find(
            Query::find("users")
                .filter(|b| b.or(|o| o.eq("name", "Alice").eq("name", "Bob"))),
        )
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("or filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_find_filter_not(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // NOT (active = true) → Bob, Dave = 2 rows
    let mut cursor = db
        .find(Query::find("users").filter(|b| b.not(|n| n.eq("active", true))))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("not filter: expected 2, got {}", rows.len()));
    }
    Ok(())
}

// ─── Find – aggregations ─────────────────────────────────────────────────────

pub async fn test_find_count_all(db: &DbContext) -> TestResult {
    seed_users(db).await;
    let mut cursor = db
        .find(Query::find("users").project(|b| b.count_all()))
        .await
        .map_err(|e| e.to_string())?;

    let row = cursor.next().await.map_err(|e| e.to_string())?.ok_or("no row returned")?;
    let count = row
        .get_by_index(0)
        .and_then(|v| v.as_i64())
        .ok_or("count_all: expected i64 at index 0")?;

    if count != 5 {
        return Err(format!("count_all: expected 5, got {count}"));
    }
    Ok(())
}

pub async fn test_find_sum(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // SUM(age): 25+30+25+35 = 115 (Eve's null is ignored by SQL SUM)
    let mut cursor = db
        .find(Query::find("users").project(|b| b.sum("age")))
        .await
        .map_err(|e| e.to_string())?;

    let row = cursor.next().await.map_err(|e| e.to_string())?.ok_or("no row returned")?;
    let val = row.get_by_index(0).ok_or("sum: no value at index 0")?;

    let sum = val
        .as_i64()
        .or_else(|| val.as_f64().map(|f| f as i64))
        .ok_or("sum: could not read numeric value")?;

    if sum != 115 {
        return Err(format!("sum(age): expected 115, got {sum}"));
    }
    Ok(())
}

pub async fn test_find_avg(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // AVG(age): (25+30+25+35)/4 = 28.75
    let mut cursor = db
        .find(Query::find("users").project(|b| b.avg("age")))
        .await
        .map_err(|e| e.to_string())?;

    let row = cursor.next().await.map_err(|e| e.to_string())?.ok_or("no row returned")?;
    let val = row.get_by_index(0).ok_or("avg: no value at index 0")?;

    let avg = val
        .as_f64()
        .or_else(|| val.as_i64().map(|i| i as f64))
        .ok_or("avg: could not read numeric value")?;

    if (avg - 28.75).abs() > 0.5 {
        return Err(format!("avg(age): expected ~28.75, got {avg}"));
    }
    Ok(())
}

pub async fn test_find_min_max(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // MIN(age)=25, MAX(age)=35  (Eve's null is ignored)
    let mut cursor = db
        .find(Query::find("users").project(|b| b.min("age").max("age")))
        .await
        .map_err(|e| e.to_string())?;

    let row = cursor.next().await.map_err(|e| e.to_string())?.ok_or("no row returned")?;

    let min_val = row.get_by_index(0).ok_or("min_max: no value at index 0")?;
    let max_val = row.get_by_index(1).ok_or("min_max: no value at index 1")?;

    let min = min_val.as_i64().or_else(|| min_val.as_f64().map(|f| f as i64))
        .ok_or("min: could not read numeric value")?;
    let max = max_val.as_i64().or_else(|| max_val.as_f64().map(|f| f as i64))
        .ok_or("max: could not read numeric value")?;

    if min != 25 {
        return Err(format!("min(age): expected 25, got {min}"));
    }
    if max != 35 {
        return Err(format!("max(age): expected 35, got {max}"));
    }
    Ok(())
}

pub async fn test_find_group_by(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // GROUP BY active, COUNT(*):  active=false→2, active=true→3
    let mut cursor = db
        .find(
            Query::find("users")
                .project(|b| b.count_all())
                .group_by(|b| b.field("active")),
        )
        .await
        .map_err(|e| e.to_string())?;

    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 2 {
        return Err(format!("group_by: expected 2 groups, got {}", rows.len()));
    }

    let mut counts: Vec<i64> = rows
        .iter()
        .filter_map(|r| r.get_by_index(0).and_then(|v| v.as_i64()))
        .collect();
    counts.sort_unstable();

    if counts != vec![2, 3] {
        return Err(format!("group_by counts: expected [2, 3], got {counts:?}"));
    }
    Ok(())
}

// ─── Update ──────────────────────────────────────────────────────────────────

pub async fn test_update_with_filter(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // Update age=99 where name='Alice'
    let affected = db
        .update(
            Query::update("users")
                .set("age", 99i32)
                .filter(|b| b.eq("name", "Alice")),
        )
        .await
        .map_err(|e| e.to_string())?;

    if affected != 1 {
        return Err(format!("update_with_filter: expected 1 affected, got {affected}"));
    }

    let mut cursor = db
        .find(Query::find("users").filter(|b| b.eq("name", "Alice")))
        .await
        .map_err(|e| e.to_string())?;
    let row = cursor.next().await.map_err(|e| e.to_string())?.ok_or("no row after update")?;
    let age = row.get_by_name("age").and_then(|v| v.as_i64()).ok_or("age column missing")?;
    if age != 99 {
        return Err(format!("update_with_filter: expected age=99, got {age}"));
    }
    Ok(())
}

pub async fn test_update_all(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // UPDATE users SET balance = 0.0  (no filter → all rows)
    let affected = db
        .update(Query::update("users").set("balance", 0.0f64))
        .await
        .map_err(|e| e.to_string())?;

    if affected != 5 {
        return Err(format!("update_all: expected 5 affected, got {affected}"));
    }

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    while let Ok(Some(row)) = cursor.next().await {
        let bal = row.get_by_name("balance").and_then(|v| v.as_f64()).unwrap_or(-1.0);
        if (bal - 0.0).abs() > 0.001 {
            return Err(format!("update_all: expected balance=0.0, got {bal}"));
        }
    }
    Ok(())
}

// ─── Delete ──────────────────────────────────────────────────────────────────

pub async fn test_delete_with_filter(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // Delete Bob
    let affected = db
        .delete(Query::delete("users").filter(|b| b.eq("name", "Bob")))
        .await
        .map_err(|e| e.to_string())?;

    if affected != 1 {
        return Err(format!("delete_with_filter: expected 1 affected, got {affected}"));
    }

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 4 {
        return Err(format!("delete_with_filter: expected 4 remaining, got {}", rows.len()));
    }
    Ok(())
}

pub async fn test_delete_all(db: &DbContext) -> TestResult {
    seed_users(db).await;
    // DELETE FROM users (no filter)
    let affected = db
        .delete(Query::delete("users"))
        .await
        .map_err(|e| e.to_string())?;

    if affected != 5 {
        return Err(format!("delete_all: expected 5 affected, got {affected}"));
    }

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let row = cursor.next().await.map_err(|e| e.to_string())?;
    if row.is_some() {
        return Err("delete_all: table should be empty".into());
    }
    Ok(())
}

// ─── Transactions ────────────────────────────────────────────────────────────

pub async fn test_transaction_commit(db: &DbContext) -> TestResult {
    let tx = db.begin().await.map_err(|e| e.to_string())?;

    tx.insert(Query::insert("users").insert([
        ("name", DbValue::from("TxUser")),
        ("email", DbValue::from("txuser@example.com")),
        ("age", DbValue::from(25i32)),
        ("active", DbValue::from(true)),
    ]))
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    let mut cursor = db
        .find(Query::find("users").filter(|b| b.eq("name", "TxUser")))
        .await
        .map_err(|e| e.to_string())?;
    let row = cursor.next().await.map_err(|e| e.to_string())?;
    if row.is_none() {
        return Err("transaction_commit: row not persisted after commit".into());
    }
    Ok(())
}

pub async fn test_transaction_rollback(db: &DbContext) -> TestResult {
    // Insert baseline first
    db.insert(Query::insert("users").insert([
        ("name", DbValue::from("Baseline")),
        ("email", DbValue::from("baseline@example.com")),
        ("age", DbValue::from(25i32)),
        ("active", DbValue::from(true)),
    ]))
    .await
    .map_err(|e| e.to_string())?;

    let tx = db.begin().await.map_err(|e| e.to_string())?;

    tx.insert(Query::insert("users").insert([
        ("name", DbValue::from("Rollback")),
        ("email", DbValue::from("rollback@example.com")),
        ("age", DbValue::from(30i32)),
        ("active", DbValue::from(true)),
    ]))
    .await
    .map_err(|e| e.to_string())?;

    tx.rollback().await.map_err(|e| e.to_string())?;

    let mut cursor = db
        .find(Query::find("users"))
        .await
        .map_err(|e| e.to_string())?;
    let rows = collect_rows(&mut cursor).await;
    if rows.len() != 1 {
        return Err(format!("transaction_rollback: expected 1 row after rollback, got {}", rows.len()));
    }
    Ok(())
}
