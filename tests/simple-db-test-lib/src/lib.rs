use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use simple_db::DbContext;
use simple_db::{filter, project, sort};
use simple_db::query::Query;
use simple_db::types::{DbRow, DbRowExt, DbValue};

mod orm_tests;
use orm_tests::get_orm_test_cases;

mod benchmarks;
use benchmarks::get_benchmark_cases;

// ─────────────────────────────────────────────────────────────────────────────
// Harness
// ─────────────────────────────────────────────────────────────────────────────
type TestFn = fn(context: &DbContext) -> BoxFuture<'static, bool>;

pub async fn run_test_cases(context: &DbContext) {
    println!();
    println!("══════════════════════════════════════════════════════════════");
    println!("  simple-db test suite");
    println!("══════════════════════════════════════════════════════════════");
    println!();

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<&'static str> = Vec::new();

    for (name, f) in get_test_cases() {
        if f(context).await {
            passed += 1;
        } else {
            failed += 1;
            failures.push(name);
        }
    }

    println!("══════════════════════════════════════════════════════════════");
    println!("  {passed} passed  │  {failed} failed");
    if !failures.is_empty() {
        println!("  Failed:");
        for name in &failures {
            println!("    ✗ {name}");
        }
    }
    println!("══════════════════════════════════════════════════════════════");
    println!();

    // Run benchmarks
    println!("══════════════════════════════════════════════════════════════");
    println!("  Performance Benchmarks");
    println!("══════════════════════════════════════════════════════════════");
    println!();

    for (_name, f) in get_benchmark_cases() {
        let _ = f(context).await;
    }

    println!("══════════════════════════════════════════════════════════════");
}

fn get_test_cases() -> Vec<(&'static str, TestFn)> {
    let mut cases: Vec<(&'static str, TestFn)> = vec![
        // ── Insert ──────────────────────────────────────────────────────────
        ("Insert · Single row",                                single_insert_test as TestFn),
        ("Insert · Bulk 1000 rows",                            bulk_insert_throughput_test as TestFn),
        // ── Find ────────────────────────────────────────────────────────────
        ("Find · All rows",                                    find_all_test as TestFn),
        ("Find · Eq filter",                                   find_eq_filter_test as TestFn),
        ("Find · Range  (gt / lt / between)",                  find_range_filter_test as TestFn),
        ("Find · String (contains / starts_with / ends_with)", find_string_filter_test as TestFn),
        ("Find · In / Not-In",                                 find_in_filter_test as TestFn),
        ("Find · OR grouping",                                 find_or_filter_test as TestFn),
        ("Find · NOT grouping",                                find_not_filter_test as TestFn),
        ("Find · Null checks",                                 find_null_filter_test as TestFn),
        ("Find · Pagination",                                  find_pagination_test as TestFn),
        ("Find · Sorting  (asc / desc)",                       find_sorting_test as TestFn),
        ("Find · Aggregations",                                find_aggregations_test as TestFn),
        ("Find · Read throughput  1 000 rows",                 find_throughput_test as TestFn),
        // ── Update ──────────────────────────────────────────────────────────
        ("Update · Single field",                              update_single_field_test as TestFn),
        ("Update · Multiple fields",                           update_multiple_fields_test as TestFn),
        ("Update · Bulk",                                      update_bulk_test as TestFn),
        // ── Delete ──────────────────────────────────────────────────────────
        ("Delete · With filter",                               delete_with_filter_test as TestFn),
        ("Delete · All rows",                                  delete_all_test as TestFn),
        // ── Transactions ────────────────────────────────────────────────────
        ("Transaction · Commit",                               transaction_commit_test as TestFn),
        ("Transaction · Rollback",                             transaction_rollback_test as TestFn),
    ];

    // ── ORM ──────────────────────────────────────────────────────────────────
    let orm_cases: Vec<(&'static str, TestFn)> = get_orm_test_cases()
        .into_iter()
        .map(|(name, f)| (name, f as TestFn))
        .collect();
    cases.extend(orm_cases);

    cases
}

// ─────────────────────────────────────────────────────────────────────────────
// Output helpers — each test owns its printing
// ─────────────────────────────────────────────────────────────────────────────

fn header(name: &str) {
    println!("┌─ {name}");
}

fn detail(line: &str) {
    println!("│  {line}");
}

fn footer(passed: bool, elapsed: Duration) {
    let icon = if passed { "✅ Passed" } else { "❌ Failed" };
    let ms = elapsed.as_secs_f64() * 1000.0;
    println!("└─ {icon}  │  {ms:.3} ms");
    println!();
}

fn check(ok: bool) -> &'static str {
    if ok { "✓" } else { "✗" }
}

// Flexible integer reader — widens any signed integer DbValue to i64.
// Also maps bool→0/1 because MySQL decodes TINYINT(1) columns as DbValue::from_bool.
fn flex_i64(v: DbValue) -> i64 {
    v.as_i64()
        .or_else(|| v.as_i32().map(i64::from))
        .or_else(|| v.as_i16().map(i64::from))
        .or_else(|| v.as_i8().map(i64::from))
        .or_else(|| v.as_bool().map(|b| b as i64))
        .expect("expected an integer or boolean DbValue")
}

fn row_int(row: &dyn DbRow, name: &str) -> i64 {
    flex_i64(row.get_by_name(name).unwrap())
}

fn idx_int(row: &dyn DbRow, idx: usize) -> i64 {
    flex_i64(row.get_by_index(idx).unwrap())
}

// Reads a floating-point value from a row by index.
// Accepts f64 (SQLite/MySQL), f32, and Decimal (Postgres NUMERIC from AVG/SUM).
fn idx_f64(row: &dyn DbRow, idx: usize) -> f64 {
    let v = row.get_by_index(idx).unwrap();
    v.as_f64()
        .or_else(|| v.as_f32().map(f64::from))
        .or_else(|| v.as_decimal().map(|d| d.to_string().parse().unwrap()))
        .expect("expected a float or decimal DbValue")
}

// ─────────────────────────────────────────────────────────────────────────────
// DB helpers
// ─────────────────────────────────────────────────────────────────────────────

async fn truncate(ctx: &DbContext) {
    ctx.delete(Query::delete("users")).await.unwrap();
}

async fn count_rows(ctx: &DbContext) -> usize {
    let mut cursor = ctx.find(Query::find("users")).await.unwrap();
    let mut n = 0usize;
    while cursor.next().await.unwrap().is_some() {
        n += 1;
    }
    n
}

// Five deterministic sample users:
//   Alice  (25, active=1, balance= 100, bio="Alice bio")
//   Bob    (35, active=1, balance= 500, bio="Bob bio")
//   Charlie(20, active=0, balance= 250, bio=NULL)
//   Diana  (40, active=1, balance=1000, bio="Diana bio")
//   Eve    (30, active=0, balance= 750, bio=NULL)
fn sample_users() -> Vec<Vec<(&'static str, DbValue)>> {
    vec![
        vec![
            ("name", "Alice".into()), ("email", "alice@example.com".into()),
            ("age", 25i64.into()), ("active", 1i64.into()), ("balance", 100.0f64.into()),
            ("bio", "Alice bio".into()),
        ],
        vec![
            ("name", "Bob".into()), ("email", "bob@example.com".into()),
            ("age", 35i64.into()), ("active", 1i64.into()), ("balance", 500.0f64.into()),
            ("bio", "Bob bio".into()),
        ],
        vec![
            ("name", "Charlie".into()), ("email", "charlie@example.com".into()),
            ("age", 20i64.into()), ("active", 0i64.into()), ("balance", 250.0f64.into()),
            ("bio", DbValue::from_null()),
        ],
        vec![
            ("name", "Diana".into()), ("email", "diana@example.com".into()),
            ("age", 40i64.into()), ("active", 1i64.into()), ("balance", 1000.0f64.into()),
            ("bio", "Diana bio".into()),
        ],
        vec![
            ("name", "Eve".into()), ("email", "eve@example.com".into()),
            ("age", 30i64.into()), ("active", 0i64.into()), ("balance", 750.0f64.into()),
            ("bio", DbValue::from_null()),
        ],
    ]
}

async fn insert_sample_users(ctx: &DbContext) {
    let query = sample_users()
        .into_iter()
        .fold(Query::insert("users"), |q, row| q.insert(row));
    ctx.insert(query).await.unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// Insert tests
// ─────────────────────────────────────────────────────────────────────────────

fn single_insert_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        header("Insert · Single row");

        let row: Vec<(&str, DbValue)> = vec![
            ("name",    "Alice".into()),
            ("email",   "alice@example.com".into()),
            ("age",     25i64.into()),
            ("active",  1i64.into()),
            ("balance", 100.0f64.into()),
        ];

        let t = Instant::now();
        let affected = ctx.insert(Query::insert("users").insert(row)).await.unwrap();
        let db_count = count_rows(&ctx).await;

        let mut cursor = ctx.find(
            Query::find("users").project(project!(field("name"), field("age"), field("balance")))
        ).await.unwrap();
        let row = cursor.next().await.unwrap().unwrap();
        let name: String = row.get_by_name_as("name").unwrap();
        let age          = row_int(&*row, "age");
        let bal:  f64    = row.get_by_name_as("balance").unwrap();
        let elapsed = t.elapsed();

        let count_ok  = affected == 1 && db_count == 1;
        let values_ok = name == "Alice" && age == 25 && (bal - 100.0).abs() < 0.001;

        detail(&format!("affected={affected}  db_count={db_count}  {}", check(count_ok)));
        detail(&format!(
            "name=\"{name}\"  age={age}  balance={bal:.1}  {}",
            check(values_ok)
        ));
        detail(&format!("latency  {:.3} ms/op", elapsed.as_secs_f64() * 1000.0));

        let ok = count_ok && values_ok;
        footer(ok, elapsed);
        ok
    })
}

fn bulk_insert_throughput_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        header("Insert · Bulk 1000 rows");

        const N: usize = 1000;
        let rows: Vec<Vec<(&str, DbValue)>> = (0..N)
            .map(|i| vec![
                ("name",    format!("User {i}").into()),
                ("email",   format!("user{i}@example.com").into()),
                ("age",     (20 + (i % 50) as i64).into()),
                ("active",  (if i % 3 == 0 { 0i64 } else { 1i64 }).into()),
                ("balance", ((i as f64) * 10.5).into()),
            ])
            .collect();

        let query = rows.into_iter().fold(Query::insert("users"), |q, row| q.insert(row));

        let t = Instant::now();
        let affected = ctx.insert(query).await.unwrap();
        let elapsed = t.elapsed();

        let ops_sec    = affected as f64 / elapsed.as_secs_f64();
        let ms_per_row = elapsed.as_secs_f64() * 1000.0 / affected as f64;
        let ok         = affected == N as u64;

        detail(&format!("rows inserted  {affected}/{N}  {}", check(ok)));
        detail(&format!("throughput     {ops_sec:.0} ops/sec"));
        detail(&format!("latency        {ms_per_row:.4} ms/row"));

        footer(ok, elapsed);
        ok
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Find tests
// ─────────────────────────────────────────────────────────────────────────────

fn find_all_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Find · All rows");

        let t = Instant::now();
        let n = count_rows(&ctx).await;
        let elapsed = t.elapsed();

        let ok = n == 5;
        detail(&format!("rows found  {n}/5  {}", check(ok)));
        detail(&format!("latency     {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

fn find_eq_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Find · Eq filter");

        let t = Instant::now();
        let mut cursor = ctx.find(
            Query::find("users")
                .project(project!(field("name"), field("age")))
                .filter(filter!(eq("name", "Alice")))
        ).await.unwrap();

        let mut count = 0usize;
        let mut name_ok = false;
        let mut age_ok  = false;
        while let Some(row) = cursor.next().await.unwrap() {
            let name: String = row.get_by_name_as("name").unwrap();
            let age          = row_int(&*row, "age");
            if name == "Alice" { name_ok = true; }
            if age  == 25      { age_ok  = true; }
            count += 1;
        }
        let elapsed = t.elapsed();

        let ok = count == 1 && name_ok && age_ok;
        detail(&format!("filter  name = \"Alice\"  →  {count}/1 row  {}", check(count == 1)));
        detail(&format!("values  name {}  age=25 {}", check(name_ok), check(age_ok)));

        footer(ok, elapsed);
        ok
    })
}

fn find_range_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // ages: Alice=25, Bob=35, Charlie=20, Diana=40, Eve=30
        header("Find · Range  (gt / lt / between)");

        let t = Instant::now();

        let mut c = ctx.find(Query::find("users").filter(filter!(gt("age", 28i64)))).await.unwrap();
        let mut gt = 0usize;
        while c.next().await.unwrap().is_some() { gt += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(lt("age", 30i64)))).await.unwrap();
        let mut lt = 0usize;
        while c.next().await.unwrap().is_some() { lt += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(lte("age", 25i64)))).await.unwrap();
        let mut lte = 0usize;
        while c.next().await.unwrap().is_some() { lte += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(between("age", 25i64, 35i64)))).await.unwrap();
        let mut between = 0usize;
        while c.next().await.unwrap().is_some() { between += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(not_between("age", 25i64, 35i64)))).await.unwrap();
        let mut not_between = 0usize;
        while c.next().await.unwrap().is_some() { not_between += 1; }

        let elapsed = t.elapsed();

        let ok = gt == 3 && lt == 2 && lte == 2 && between == 3 && not_between == 2;
        detail(&format!("age > 28          →  {gt}/3         {}", check(gt == 3)));
        detail(&format!("age < 30          →  {lt}/2         {}", check(lt == 2)));
        detail(&format!("age <= 25         →  {lte}/2        {}", check(lte == 2)));
        detail(&format!("age BETWEEN 25,35 →  {between}/3   {}", check(between == 3)));
        detail(&format!("age NOT BETWEEN   →  {not_between}/2   {}", check(not_between == 2)));

        footer(ok, elapsed);
        ok
    })
}

fn find_string_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // names: Alice, Bob, Charlie, Diana, Eve
        header("Find · String  (contains / starts_with / ends_with)");

        let t = Instant::now();

        let mut c = ctx.find(Query::find("users").filter(filter!(contains("name", "li")))).await.unwrap();
        let mut contains = 0usize;
        while c.next().await.unwrap().is_some() { contains += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(not_contains("name", "li")))).await.unwrap();
        let mut not_contains = 0usize;
        while c.next().await.unwrap().is_some() { not_contains += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(starts_with("name", "A")))).await.unwrap();
        let mut starts = 0usize;
        while c.next().await.unwrap().is_some() { starts += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(ends_with("name", "e")))).await.unwrap();
        let mut ends = 0usize;
        while c.next().await.unwrap().is_some() { ends += 1; }

        let elapsed = t.elapsed();

        let ok = contains == 2 && not_contains == 3 && starts == 1 && ends == 3;
        detail(&format!("LIKE '%li%'   →  {contains}/2      {}", check(contains == 2)));
        detail(&format!("NOT LIKE '%li%' →  {not_contains}/3  {}", check(not_contains == 3)));
        detail(&format!("LIKE 'A%'     →  {starts}/1      {}", check(starts == 1)));
        detail(&format!("LIKE '%e'     →  {ends}/3      {}", check(ends == 3)));

        footer(ok, elapsed);
        ok
    })
}

fn find_in_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Find · In / Not-In");

        let t = Instant::now();

        let mut c = ctx.find(
            Query::find("users").filter(filter!(is_in("name", vec!["Alice", "Bob"])))
        ).await.unwrap();
        let mut in_count = 0usize;
        while c.next().await.unwrap().is_some() { in_count += 1; }

        let mut c = ctx.find(
            Query::find("users").filter(filter!(not_in("name", vec!["Alice", "Bob"])))
        ).await.unwrap();
        let mut not_in_count = 0usize;
        while c.next().await.unwrap().is_some() { not_in_count += 1; }

        let elapsed = t.elapsed();

        let ok = in_count == 2 && not_in_count == 3;
        detail(&format!("IN  (Alice, Bob)      →  {in_count}/2  {}", check(in_count == 2)));
        detail(&format!("NOT IN (Alice, Bob)   →  {not_in_count}/3  {}", check(not_in_count == 3)));

        footer(ok, elapsed);
        ok
    })
}

fn find_or_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // OR(active=0, balance>800) → Charlie, Eve, Diana  = 3
        header("Find · OR grouping");

        let t = Instant::now();
        let mut c = ctx.find(
            Query::find("users")
                .filter(filter!(or(filter!(eq("active", 0i64), gt("balance", 800.0f64)))))
        ).await.unwrap();
        let mut count = 0usize;
        while c.next().await.unwrap().is_some() { count += 1; }
        let elapsed = t.elapsed();

        let ok = count == 3;
        detail("filter  (active = 0) OR (balance > 800)");
        detail(&format!("expected  Charlie + Eve + Diana  =  3 rows"));
        detail(&format!("got       {count}/3  {}", check(ok)));

        footer(ok, elapsed);
        ok
    })
}

fn find_not_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // NOT(active=1) → Charlie, Eve  = 2
        header("Find · NOT grouping");

        let t = Instant::now();
        let mut c = ctx.find(
            Query::find("users")
                .filter(filter!(not(filter!(eq("active", 1i64)))))
        ).await.unwrap();
        let mut count = 0usize;
        while c.next().await.unwrap().is_some() { count += 1; }
        let elapsed = t.elapsed();

        let ok = count == 2;
        detail("filter  NOT (active = 1)");
        detail(&format!("expected  Charlie + Eve  =  2 rows"));
        detail(&format!("got       {count}/2  {}", check(ok)));

        footer(ok, elapsed);
        ok
    })
}

fn find_null_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // bio = NULL for Charlie, Eve (2)  │  NOT NULL for Alice, Bob, Diana (3)
        header("Find · Null checks  (is_null / is_not_null)");

        let t = Instant::now();

        let mut c = ctx.find(Query::find("users").filter(filter!(is_null("bio")))).await.unwrap();
        let mut null_n = 0usize;
        while c.next().await.unwrap().is_some() { null_n += 1; }

        let mut c = ctx.find(Query::find("users").filter(filter!(is_not_null("bio")))).await.unwrap();
        let mut not_null_n = 0usize;
        while c.next().await.unwrap().is_some() { not_null_n += 1; }

        let elapsed = t.elapsed();

        let ok = null_n == 2 && not_null_n == 3;
        detail(&format!("bio IS NULL      →  {null_n}/2  {}", check(null_n == 2)));
        detail(&format!("bio IS NOT NULL  →  {not_null_n}/3  {}", check(not_null_n == 3)));

        footer(ok, elapsed);
        ok
    })
}

fn find_pagination_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // 5 rows  →  page_size=2, 3 pages  (last page has 1 row)
        header("Find · Pagination  (limit / offset)");

        let t = Instant::now();

        let mut c = ctx.find(Query::find("users").order_by(sort!(asc("id"))).limit(2).offset(0)).await.unwrap();
        let mut page1 = 0usize;
        while c.next().await.unwrap().is_some() { page1 += 1; }

        let mut c = ctx.find(Query::find("users").order_by(sort!(asc("id"))).limit(2).offset(2)).await.unwrap();
        let mut page2 = 0usize;
        while c.next().await.unwrap().is_some() { page2 += 1; }

        let mut c = ctx.find(Query::find("users").order_by(sort!(asc("id"))).limit(2).offset(4)).await.unwrap();
        let mut page3 = 0usize;
        while c.next().await.unwrap().is_some() { page3 += 1; }

        let elapsed = t.elapsed();

        let ok = page1 == 2 && page2 == 2 && page3 == 1;
        detail("page_size=2  over 5 rows  →  3 pages");
        detail(&format!("page 1  (offset 0)  →  {page1}/2  {}", check(page1 == 2)));
        detail(&format!("page 2  (offset 2)  →  {page2}/2  {}", check(page2 == 2)));
        detail(&format!("page 3  (offset 4)  →  {page3}/1  {}", check(page3 == 1)));

        footer(ok, elapsed);
        ok
    })
}

fn find_sorting_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // ages: Alice=25, Bob=35, Charlie=20, Diana=40, Eve=30
        header("Find · Sorting  (asc / desc)");

        let t = Instant::now();

        let mut c = ctx.find(
            Query::find("users").project(project!(field("age"))).order_by(sort!(asc("age")))
        ).await.unwrap();
        let mut asc: Vec<i64> = Vec::new();
        while let Some(row) = c.next().await.unwrap() {
            asc.push(row_int(&*row, "age"));
        }

        let mut c = ctx.find(
            Query::find("users").project(project!(field("age"))).order_by(sort!(desc("age")))
        ).await.unwrap();
        let mut desc: Vec<i64> = Vec::new();
        while let Some(row) = c.next().await.unwrap() {
            desc.push(row_int(&*row, "age"));
        }

        let elapsed = t.elapsed();

        let asc_ok  = asc  == [20, 25, 30, 35, 40];
        let desc_ok = desc == [40, 35, 30, 25, 20];
        let ok = asc_ok && desc_ok;

        detail(&format!("ASC   →  {asc:?}  {}", check(asc_ok)));
        detail(&format!("DESC  →  {desc:?}  {}", check(desc_ok)));

        footer(ok, elapsed);
        ok
    })
}

fn find_aggregations_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // ages: 25,35,20,40,30   →  avg=30  min=20  max=40
        // balances: 100,500,250,1000,750  →  sum=2600
        header("Find · Aggregations  (count / avg / sum / min / max)");

        let t = Instant::now();
        let mut cursor = ctx.find(
            Query::find("users").project(project!(
                count_all(),    // idx 0 → i64
                avg("age"),     // idx 1 → f64
                sum("balance"), // idx 2 → f64
                min("age"),     // idx 3 → i64
                max("age")      // idx 4 → i64
            ))
        ).await.unwrap();
        let row = cursor.next().await.unwrap().unwrap();
        let elapsed = t.elapsed();

        let count:   i64 = row.get_by_index_as(0).unwrap_or(-1);
        let avg_age: f64 = idx_f64(&*row, 1);
        let sum_bal: f64 = idx_f64(&*row, 2);
        let min_age      = idx_int(&*row, 3);
        let max_age      = idx_int(&*row, 4);

        let count_ok = count   == 5;
        let avg_ok   = (avg_age - 30.0).abs()    < 0.01;
        let sum_ok   = (sum_bal - 2600.0).abs()  < 0.01;
        let min_ok   = min_age == 20;
        let max_ok   = max_age == 40;
        let ok = count_ok && avg_ok && sum_ok && min_ok && max_ok;

        detail(&format!("count(*)       =  {count}       {}",   check(count_ok)));
        detail(&format!("avg(age)       =  {avg_age:.1}    {}",   check(avg_ok)));
        detail(&format!("sum(balance)   =  {sum_bal:.1}  {}",  check(sum_ok)));
        detail(&format!("min(age)       =  {min_age}       {}",   check(min_ok)));
        detail(&format!("max(age)       =  {max_age}       {}",   check(max_ok)));

        footer(ok, elapsed);
        ok
    })
}

fn find_throughput_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        header("Find · Read throughput  1 000 rows");

        const N: usize = 1000;
        let rows: Vec<Vec<(&str, DbValue)>> = (0..N)
            .map(|i| vec![
                ("name",    format!("User {i}").into()),
                ("email",   format!("u{i}@test.com").into()),
                ("age",     (20 + (i % 50) as i64).into()),
                ("active",  1i64.into()),
                ("balance", (i as f64 * 10.0).into()),
            ])
            .collect();
        ctx.insert(rows.into_iter().fold(Query::insert("users"), |q, r| q.insert(r)))
            .await.unwrap();

        let t = Instant::now();
        let mut cursor = ctx.find(Query::find("users")).await.unwrap();
        let mut read = 0usize;
        while cursor.next().await.unwrap().is_some() { read += 1; }
        let elapsed = t.elapsed();

        let rows_sec = read as f64 / elapsed.as_secs_f64();
        let ok = read == N;

        detail(&format!("rows read    {read}/{N}  {}", check(ok)));
        detail(&format!("throughput   {rows_sec:.0} rows/sec"));
        detail(&format!("latency      {:.3} ms total", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Update tests
// ─────────────────────────────────────────────────────────────────────────────

fn update_single_field_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Update · Single field");

        let t = Instant::now();
        let affected = ctx.update(
            Query::update("users")
                .set("email", "newalice@example.com")
                .filter(filter!(eq("name", "Alice")))
        ).await.unwrap();

        let new_email = ctx.find(
            Query::find("users")
                .project(project!(field("email")))
                .filter(filter!(eq("name", "Alice")))
        ).await.unwrap()
            .next().await.unwrap()
            .map(|r| r.get_by_name_as::<String>("email").unwrap_or_default())
            .unwrap_or_default();
        let elapsed = t.elapsed();

        let affected_ok = affected == 1;
        let value_ok    = new_email == "newalice@example.com";
        let ok = affected_ok && value_ok;

        detail(&format!("SET email WHERE name = \"Alice\""));
        detail(&format!("affected  {affected}/1  {}", check(affected_ok)));
        detail(&format!("new value \"{}\"  {}", new_email, check(value_ok)));
        detail(&format!("latency   {:.3} ms/op", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

fn update_multiple_fields_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Update · Multiple fields");

        let t = Instant::now();
        let affected = ctx.update(
            Query::update("users")
                .set("name",    "Alice Updated")
                .set("balance", 9999.0f64)
                .set("active",  0i64)
                .filter(filter!(eq("name", "Alice")))
        ).await.unwrap();

        let fields_ok = ctx.find(
            Query::find("users")
                .project(project!(field("name"), field("balance"), field("active")))
                .filter(filter!(eq("name", "Alice Updated")))
        ).await.unwrap()
            .next().await.unwrap()
            .map(|r| {
                let name:    String = r.get_by_name_as("name").unwrap();
                let balance: f64    = r.get_by_name_as("balance").unwrap();
                let active           = row_int(&*r, "active");
                name == "Alice Updated" && (balance - 9999.0).abs() < 0.01 && active == 0
            })
            .unwrap_or(false);
        let elapsed = t.elapsed();

        let ok = affected == 1 && fields_ok;
        detail("SET name, balance, active  WHERE name = \"Alice\"");
        detail(&format!("affected    {affected}/1  {}", check(affected == 1)));
        detail(&format!("all values  {}", check(fields_ok)));
        detail(&format!("latency     {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

fn update_bulk_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // active=1: Alice, Bob, Diana  →  3 rows  →  set active=0
        header("Update · Bulk  (all matching rows)");

        let t = Instant::now();
        let affected = ctx.update(
            Query::update("users").set("active", 0i64).filter(filter!(eq("active", 1i64)))
        ).await.unwrap();

        let mut c = ctx.find(Query::find("users").filter(filter!(eq("active", 0i64)))).await.unwrap();
        let mut inactive = 0usize;
        while c.next().await.unwrap().is_some() { inactive += 1; }
        let elapsed = t.elapsed();

        let affected_ok = affected == 3;
        let inactive_ok = inactive == 5;
        let ok = affected_ok && inactive_ok;
        let ops_sec = if elapsed.as_secs_f64() > 0.0 {
            affected as f64 / elapsed.as_secs_f64()
        } else {
            f64::INFINITY
        };

        detail("SET active=0  WHERE active=1");
        detail(&format!("rows updated    {affected}/3  {}", check(affected_ok)));
        detail(&format!("now inactive    {inactive}/5  {}", check(inactive_ok)));
        detail(&format!("throughput      {ops_sec:.0} ops/sec"));

        footer(ok, elapsed);
        ok
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Delete tests
// ─────────────────────────────────────────────────────────────────────────────

fn delete_with_filter_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        // age < 30: Alice(25), Charlie(20)  →  2 deleted, 3 remain
        header("Delete · With filter");

        let t = Instant::now();
        let affected = ctx.delete(
            Query::delete("users").filter(filter!(lt("age", 30i64)))
        ).await.unwrap();
        let remaining = count_rows(&ctx).await;
        let elapsed = t.elapsed();

        let deleted_ok   = affected  == 2;
        let remaining_ok = remaining == 3;
        let ok = deleted_ok && remaining_ok;

        detail("DELETE WHERE age < 30  (Alice=25, Charlie=20)");
        detail(&format!("rows deleted   {affected}/2   {}", check(deleted_ok)));
        detail(&format!("rows remaining {remaining}/3  {}", check(remaining_ok)));
        detail(&format!("latency        {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

fn delete_all_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        insert_sample_users(&ctx).await;
        header("Delete · All rows  (no filter)");

        let t = Instant::now();
        let affected = ctx.delete(Query::delete("users")).await.unwrap();
        let remaining = count_rows(&ctx).await;
        let elapsed = t.elapsed();

        let deleted_ok   = affected  == 5;
        let remaining_ok = remaining == 0;
        let ok = deleted_ok && remaining_ok;

        detail("DELETE  (no WHERE clause)");
        detail(&format!("rows deleted   {affected}/5  {}", check(deleted_ok)));
        detail(&format!("rows remaining {remaining}/0  {}", check(remaining_ok)));
        detail(&format!("latency        {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Transaction tests
// ─────────────────────────────────────────────────────────────────────────────

fn transaction_commit_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        header("Transaction · Commit");

        let t = Instant::now();
        let tx = ctx.begin().await.unwrap();
        let row: Vec<(&str, DbValue)> = vec![
            ("name",   "TxUser".into()),
            ("email",  "tx@example.com".into()),
            ("age",    42i64.into()),
            ("active", 1i64.into()),
        ];
        tx.insert(Query::insert("users").insert(row)).await.unwrap();
        tx.commit().await.unwrap();

        let count = count_rows(&ctx).await;
        let elapsed = t.elapsed();

        let ok = count == 1;
        detail("BEGIN  →  INSERT 1 row  →  COMMIT");
        detail(&format!("post-commit row count  {count}/1  {}", check(ok)));
        detail(&format!("latency                {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}

fn transaction_rollback_test(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        truncate(&ctx).await;
        header("Transaction · Rollback");

        let t = Instant::now();
        let tx = ctx.begin().await.unwrap();
        let row: Vec<(&str, DbValue)> = vec![
            ("name",   "TxUser".into()),
            ("email",  "tx@example.com".into()),
            ("age",    42i64.into()),
            ("active", 1i64.into()),
        ];
        tx.insert(Query::insert("users").insert(row)).await.unwrap();
        tx.rollback().await.unwrap();

        let count = count_rows(&ctx).await;
        let elapsed = t.elapsed();

        let ok = count == 0;
        detail("BEGIN  →  INSERT 1 row  →  ROLLBACK");
        detail(&format!("post-rollback row count  {count}/0  {}", check(ok)));
        detail(&format!("latency                  {:.3} ms", elapsed.as_secs_f64() * 1000.0));

        footer(ok, elapsed);
        ok
    })
}
