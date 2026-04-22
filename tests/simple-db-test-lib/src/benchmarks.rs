use std::time::Instant;
use futures::future::BoxFuture;
use simple_db::{DbContext, DbEntity, DbEntityTrait};
use simple_db::{filter, project};
use simple_db::driver::DbExecutor;
use simple_db::types::DbValue;
use simple_db::query::Query;

use crate::orm_tests::UserEntity;

// ─────────────────────────────────────────────────────────────────────────────
// Output helpers
// ─────────────────────────────────────────────────────────────────────────────

fn bench_header(name: &str) {
    println!("┌─ {name}");
}

fn bench_detail(line: &str) {
    println!("│  {line}");
}

fn bench_footer(elapsed: std::time::Duration) {
    let ms = elapsed.as_secs_f64() * 1000.0;
    println!("└─ ⏱  {ms:.3} ms");
    println!();
}

fn fmt_ops_sec(count: usize, elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs_f64();
    if secs > 0.0 {
        let ops_per_sec = (count as f64) / secs;
        format!("{:.0} ops/sec", ops_per_sec)
    } else {
        "N/A".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Query Benchmarks
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark INSERT performance: bulk insert 1000 rows
fn bench_insert_bulk(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let _ = ctx.delete(Query::delete("users")).await;

        let rows: Vec<Vec<(&str, DbValue)>> = (0..1000).map(|i| vec![
            ("name",  DbValue::from_string(format!("User{}", i))),
            ("email", DbValue::from_string(format!("user{}@example.com", i))),
            ("age",   DbValue::from_i32(20 + (i as i32 % 50))),
        ]).collect();

        let insert_start = Instant::now();
        let _ = ctx.insert(Query::insert("users").bulk_insert(rows)).await;
        let insert_time = insert_start.elapsed();

        bench_header("Query Benchmark · INSERT 1000 rows (bulk)");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(1000, insert_time)));
        bench_detail(&format!("latency      {:.3} ms total", insert_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark SELECT performance: read 1000 rows
fn bench_select_all(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let read_start = Instant::now();
        let mut cursor = ctx.find(Query::find("users")).await.unwrap();
        let mut count = 0usize;
        while let Ok(Some(_row)) = cursor.next().await {
            count += 1;
        }
        let read_time = read_start.elapsed();

        bench_header("Query Benchmark · SELECT all 1000 rows");
        bench_detail(&format!("rows read    {}/1000  ✓", count));
        bench_detail(&format!("throughput   {}", fmt_ops_sec(count, read_time)));
        bench_detail(&format!("latency      {:.3} ms total", read_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        count == 1000
    })
}

/// Benchmark SELECT with filter: find by age range
fn bench_select_with_filter(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let filter_start = Instant::now();
        let mut cursor = ctx.find(
            Query::find("users").filter(filter!(gte("age", 30), lt("age", 40)))
        ).await.unwrap();
        let mut count = 0usize;
        while let Ok(Some(_row)) = cursor.next().await {
            count += 1;
        }
        let filter_time = filter_start.elapsed();

        bench_header("Query Benchmark · SELECT with filter (30 <= age < 40)");
        bench_detail(&format!("rows matched {}", count));
        bench_detail(&format!("throughput   {}", fmt_ops_sec(count, filter_time)));
        bench_detail(&format!("latency      {:.3} ms", filter_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        count > 0
    })
}

/// Benchmark UPDATE performance: update 100 rows
fn bench_update_bulk(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let names: Vec<DbValue> = (0..100)
            .map(|i| DbValue::from_string(format!("User{}", i)))
            .collect();
        let update_start = Instant::now();
        let _ = ctx.update(
            Query::update("users")
                .set("age", DbValue::from_i32(50))
                .filter(filter!(is_in("name", names)))
        ).await;
        let update_time = update_start.elapsed();

        bench_header("Query Benchmark · UPDATE 100 rows (bulk)");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(100, update_time)));
        bench_detail(&format!("latency      {:.3} ms total", update_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark DELETE performance: delete 100 rows
fn bench_delete_bulk(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let names: Vec<DbValue> = (0..100)
            .map(|i| DbValue::from_string(format!("User{}", i)))
            .collect();
        let delete_start = Instant::now();
        let _ = ctx.delete(
            Query::delete("users").filter(filter!(is_in("name", names)))
        ).await;
        let delete_time = delete_start.elapsed();

        bench_header("Query Benchmark · DELETE 100 rows (bulk)");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(100, delete_time)));
        bench_detail(&format!("latency      {:.3} ms total", delete_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark aggregation: COUNT, AVG, SUM, MIN, MAX
fn bench_aggregations(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        bench_header("Query Benchmark · Aggregations (COUNT, AVG, SUM, MIN, MAX)");

        let agg_start = Instant::now();

        let mut cursor = ctx.find(Query::find("users").project(project!(count_all()))).await.unwrap();
        if let Ok(Some(_row)) = cursor.next().await {
            let count_time = agg_start.elapsed();
            bench_detail(&format!("COUNT(*)     latency {:.3} ms", count_time.as_secs_f64() * 1000.0));
        }

        let avg_start = Instant::now();
        let mut cursor = ctx.find(Query::find("users").project(project!(avg("age")))).await.unwrap();
        if let Ok(Some(_row)) = cursor.next().await {
            let avg_time = avg_start.elapsed();
            bench_detail(&format!("AVG(age)     latency {:.3} ms", avg_time.as_secs_f64() * 1000.0));
        }

        let sum_start = Instant::now();
        let mut cursor = ctx.find(Query::find("users").project(project!(sum("age")))).await.unwrap();
        if let Ok(Some(_row)) = cursor.next().await {
            let sum_time = sum_start.elapsed();
            bench_detail(&format!("SUM(age)     latency {:.3} ms", sum_time.as_secs_f64() * 1000.0));
        }

        bench_footer(t.elapsed());
        true
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// ORM Benchmarks
// ─────────────────────────────────────────────────────────────────────────────

/// Benchmark ORM INSERT: save 100 new entities
fn bench_orm_insert(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let _ = ctx.delete(Query::delete("users")).await;

        let insert_start = Instant::now();
        let tx = ctx.begin().await.unwrap();
        for i in 0..100 {
            let user = UserEntity::new(
                20000 + i as i64,
                format!("OrmUser{}", i),
                format!("ormuser{}@example.com", i),
            );
            let mut entity = DbEntity::new(user);
            let _ = entity.save(tx.as_ref() as &dyn DbExecutor).await;
        }
        tx.commit().await.unwrap();
        let insert_time = insert_start.elapsed();

        bench_header("ORM Benchmark · INSERT 100 new entities");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(100, insert_time)));
        bench_detail(&format!("latency      {:.3} ms total", insert_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark ORM UPDATE: load and update 50 entities
fn bench_orm_update(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let ids: Vec<DbValue> = (0..50).map(|i| DbValue::from_i64(20000 + i)).collect();
        let mut entities = UserEntity::find(&ctx, filter!(is_in("id", ids))).await.unwrap();
        for (i, entity) in entities.iter_mut().enumerate() {
            entity.get_mut().set_email(format!("updated{}@example.com", i));
        }

        let update_start = Instant::now();
        let tx = ctx.begin().await.unwrap();
        for entity in entities.iter_mut() {
            let _ = entity.save(tx.as_ref() as &dyn DbExecutor).await;
        }
        tx.commit().await.unwrap();
        let update_time = update_start.elapsed();

        bench_header("ORM Benchmark · UPDATE 50 tracked entities");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(50, update_time)));
        bench_detail(&format!("latency      {:.3} ms total", update_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark ORM DELETE: load and delete 50 entities
fn bench_orm_delete(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let ids: Vec<DbValue> = (50..100).map(|i| DbValue::from_i64(20000 + i)).collect();
        let mut entities = UserEntity::find(&ctx, filter!(is_in("id", ids))).await.unwrap();

        let delete_start = Instant::now();
        let tx = ctx.begin().await.unwrap();
        for entity in entities.iter_mut() {
            let _ = entity.delete(tx.as_ref() as &dyn DbExecutor).await;
        }
        tx.commit().await.unwrap();
        let delete_time = delete_start.elapsed();

        bench_header("ORM Benchmark · DELETE 50 tracked entities");
        bench_detail(&format!("throughput   {}", fmt_ops_sec(50, delete_time)));
        bench_detail(&format!("latency      {:.3} ms total", delete_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        true
    })
}

/// Benchmark ORM FIND with tracking: load 100 entities as tracked
fn bench_orm_find_tracked(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let _ = ctx.delete(Query::delete("users")).await;
        let seed_rows: Vec<Vec<(&str, DbValue)>> = (0..100).map(|i| vec![
            ("name",  DbValue::from_string(format!("FindUser{}", i))),
            ("email", DbValue::from_string(format!("finduser{}@example.com", i))),
        ]).collect();
        let _ = ctx.insert(Query::insert("users").bulk_insert(seed_rows)).await;

        let find_start = Instant::now();
        let entities = UserEntity::find(&ctx, filter!(gte("id", 0))).await.unwrap();
        let find_time = find_start.elapsed();
        let count = entities.len();

        bench_header("ORM Benchmark · FIND 100 entities as tracked");
        bench_detail(&format!("rows loaded  {}/100  ✓", count));
        bench_detail(&format!("throughput   {}", fmt_ops_sec(count, find_time)));
        bench_detail(&format!("latency      {:.3} ms total", find_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        count == 100
    })
}

/// Benchmark ORM FIND_READONLY: load 100 entities as detached
fn bench_orm_find_readonly(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();

        let find_start = Instant::now();
        let entities = UserEntity::find_readonly(&ctx, filter!(gte("id", 0))).await.unwrap();
        let find_time = find_start.elapsed();
        let count = entities.len();

        bench_header("ORM Benchmark · FIND_READONLY 100 entities as detached");
        bench_detail(&format!("rows loaded  {}/100  ✓", count));
        bench_detail(&format!("throughput   {}", fmt_ops_sec(count, find_time)));
        bench_detail(&format!("latency      {:.3} ms total", find_time.as_secs_f64() * 1000.0));
        bench_footer(t.elapsed());
        count == 100
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark Registry
// ─────────────────────────────────────────────────────────────────────────────

pub fn get_benchmark_cases() -> Vec<(&'static str, fn(&DbContext) -> BoxFuture<'static, bool>)> {
    vec![
        ("Query · INSERT 1000 rows",                 bench_insert_bulk as fn(&DbContext) -> BoxFuture<'static, bool>),
        ("Query · SELECT all 1000 rows",             bench_select_all),
        ("Query · SELECT with filter",               bench_select_with_filter),
        ("Query · UPDATE 100 rows",                  bench_update_bulk),
        ("Query · DELETE 100 rows",                  bench_delete_bulk),
        ("Query · Aggregations",                     bench_aggregations),
        ("ORM · INSERT 100 new entities",            bench_orm_insert),
        ("ORM · UPDATE 50 tracked entities",         bench_orm_update),
        ("ORM · DELETE 50 tracked entities",         bench_orm_delete),
        ("ORM · FIND 100 entities (tracked)",        bench_orm_find_tracked),
        ("ORM · FIND_READONLY 100 entities (detached)", bench_orm_find_readonly),
    ]
}
