use std::time::{Duration, Instant};

use simple_db::DbContext;
use simple_db::query::Query;
use simple_db::types::DbValue;

// ─── Result type ─────────────────────────────────────────────────────────────

/// Statistics for a single named benchmark operation.
pub struct BenchResult {
    /// Benchmark name shown in the report.
    pub name: String,
    /// Number of timed iterations.
    pub runs: u32,
    /// Sum of all iteration durations.
    pub total: Duration,
    /// Fastest single iteration.
    pub min: Duration,
    /// Slowest single iteration.
    pub max: Duration,
    /// Mean duration across all iterations.
    pub avg: Duration,
    /// 50th-percentile (median) duration.
    pub p50: Duration,
    /// 95th-percentile duration.
    pub p95: Duration,
    /// 99th-percentile duration.
    pub p99: Duration,
}

impl BenchResult {
    /// Computes statistics from a list of individual iteration durations.
    pub fn from_durations(name: impl Into<String>, mut durations: Vec<Duration>) -> Self {
        if durations.is_empty() {
            return Self::zero(name);
        }
        durations.sort_unstable();
        let runs = durations.len() as u32;
        let total: Duration = durations.iter().sum();
        let min = durations[0];
        let max = *durations.last().unwrap();
        let avg = total / runs;
        let p50 = percentile(&durations, 50);
        let p95 = percentile(&durations, 95);
        let p99 = percentile(&durations, 99);
        Self { name: name.into(), runs, total, min, max, avg, p50, p95, p99 }
    }

    fn zero(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            runs: 0,
            total: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            avg: Duration::ZERO,
            p50: Duration::ZERO,
            p95: Duration::ZERO,
            p99: Duration::ZERO,
        }
    }
}

/// Returns the value at the given percentile of a **sorted** slice of durations.
fn percentile(sorted: &[Duration], p: usize) -> Duration {
    let idx = ((sorted.len() * p).saturating_sub(1) / 100).min(sorted.len() - 1);
    sorted[idx]
}

// ─── Benchmark functions ──────────────────────────────────────────────────────
// Each function receives a freshly provisioned DbContext (empty schema) and
// the number of runs to execute.  Setup/teardown that is NOT the target
// operation is done outside the timed window.

pub async fn bench_insert_single(db: &DbContext, runs: u32) -> BenchResult {
    let mut durations = Vec::with_capacity(runs as usize);
    for i in 0..runs {
        let start = Instant::now();
        let _ = db
            .insert(
                Query::insert("users").insert([
                    ("name", DbValue::from(format!("User {i}"))),
                    ("email", DbValue::from(format!("user{i}@bench.com"))),
                    ("age", DbValue::from(20i32 + (i % 50) as i32)),
                    ("active", DbValue::from(true)),
                ]),
            )
            .await;
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("insert_single", durations)
}

pub async fn bench_insert_bulk_10(db: &DbContext, runs: u32) -> BenchResult {
    let mut durations = Vec::with_capacity(runs as usize);
    for run in 0..runs {
        let rows: Vec<Vec<(&str, DbValue)>> = (0..10i32)
            .map(|i| {
                vec![
                    ("name", DbValue::from(format!("Bulk {run}_{i}"))),
                    ("email", DbValue::from(format!("bulk{run}{i}@bench.com"))),
                    ("age", DbValue::from(20i32 + i)),
                    ("active", DbValue::from(i % 2 == 0)),
                ]
            })
            .collect();
        let start = Instant::now();
        let _ = db.insert(Query::insert("users").bulk_insert(rows)).await;
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("insert_bulk_10", durations)
}

pub async fn bench_find_all(db: &DbContext, runs: u32) -> BenchResult {
    // Pre-populate 100 rows (not measured)
    let rows: Vec<Vec<(&str, DbValue)>> = (0..100i32)
        .map(|i| {
            vec![
                ("name", DbValue::from(format!("User {i}"))),
                ("email", DbValue::from(format!("user{i}@bench.com"))),
                ("age", DbValue::from(20i32 + i % 60)),
                ("active", DbValue::from(i % 3 != 0)),
            ]
        })
        .collect();
    db.insert(Query::insert("users").bulk_insert(rows))
        .await
        .expect("bench_find_all: seed failed");

    let mut durations = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let start = Instant::now();
        let mut cursor = db.find(Query::find("users")).await.unwrap();
        while let Ok(Some(_)) = cursor.next().await {}
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("find_all_100_rows", durations)
}

pub async fn bench_find_with_filter(db: &DbContext, runs: u32) -> BenchResult {
    let rows: Vec<Vec<(&str, DbValue)>> = (0..100i32)
        .map(|i| {
            vec![
                ("name", DbValue::from(format!("User {i}"))),
                ("email", DbValue::from(format!("user{i}@bench.com"))),
                ("age", DbValue::from(20i32 + i % 60)),
                ("active", DbValue::from(i % 3 != 0)),
            ]
        })
        .collect();
    db.insert(Query::insert("users").bulk_insert(rows))
        .await
        .expect("bench_find_with_filter: seed failed");

    let mut durations = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let start = Instant::now();
        let mut cursor = db
            .find(Query::find("users").filter(|b| b.gte("age", 40i32)))
            .await
            .unwrap();
        while let Ok(Some(_)) = cursor.next().await {}
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("find_filter_gte", durations)
}

pub async fn bench_update_with_filter(db: &DbContext, runs: u32) -> BenchResult {
    // Pre-populate 20 rows
    let rows: Vec<Vec<(&str, DbValue)>> = (0..20i32)
        .map(|i| {
            vec![
                ("name", DbValue::from(format!("User {i}"))),
                ("email", DbValue::from(format!("user{i}@bench.com"))),
                ("age", DbValue::from(20i32 + i)),
                ("active", DbValue::from(true)),
            ]
        })
        .collect();
    db.insert(Query::insert("users").bulk_insert(rows))
        .await
        .expect("bench_update: seed failed");

    let mut durations = Vec::with_capacity(runs as usize);
    for i in 0..runs {
        let start = Instant::now();
        let _ = db
            .update(
                Query::update("users")
                    .set("active", i % 2 == 0)
                    .filter(|b| b.lte("age", 30i32)),
            )
            .await;
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("update_with_filter", durations)
}

pub async fn bench_delete_with_filter(db: &DbContext, runs: u32) -> BenchResult {
    let mut durations = Vec::with_capacity(runs as usize);
    for i in 0..runs {
        // Setup (not measured): insert a distinct row
        let name = format!("del_{i}");
        db.insert(
            Query::insert("users").insert([
                ("name", DbValue::from(name.clone())),
                ("email", DbValue::from(format!("del{i}@bench.com"))),
                ("age", DbValue::from(25i32)),
                ("active", DbValue::from(true)),
            ]),
        )
        .await
        .expect("bench_delete: insert failed");

        // Measured: delete by name
        let start = Instant::now();
        let _ = db
            .delete(Query::delete("users").filter(|b| b.eq("name", name)))
            .await;
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("delete_with_filter", durations)
}

pub async fn bench_transaction_commit(db: &DbContext, runs: u32) -> BenchResult {
    let mut durations = Vec::with_capacity(runs as usize);
    for i in 0..runs {
        let start = Instant::now();
        let tx = db.begin().await.expect("begin tx failed");
        tx.insert(
            Query::insert("users").insert([
                ("name", DbValue::from(format!("TxUser {i}"))),
                ("email", DbValue::from(format!("tx{i}@bench.com"))),
                ("age", DbValue::from(25i32)),
                ("active", DbValue::from(true)),
            ]),
        )
        .await
        .expect("tx insert failed");
        tx.commit().await.expect("commit failed");
        durations.push(start.elapsed());
    }
    BenchResult::from_durations("transaction_commit", durations)
}
