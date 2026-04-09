use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_db::{
    driver::memory::MemoryDriver,
    entity::DbEntityModel,
    query::Query,
    types::{DbError, DbRow, DbValue, FromDbRow},
    DbContext,
};

criterion_group!(
    benches,
    bench_find_entities_readonly_100k,
    bench_find_entities_tracked_100k,
    bench_find_rows_eq_i32_100k,
    bench_find_rows_gte_i32_100k,
    bench_find_rows_contains_string_100k,
    bench_find_rows_regex_string_100k,
    bench_dirty_field_detection_1k,
    bench_query_builder_10k
);
criterion_main!(benches);

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
    active: bool,
}

impl FromDbRow for User {
    fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,
            name: row.take_string("name")?,
            email: row.take_string("email")?,
            active: row.take_bool("active")?,
        })
    }
}

impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);
        row.insert("name", self.name);
        row.insert("email", self.email);
        row.insert("active", self.active);
        row
    }
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> Vec<(String, DbValue)> {
        vec![("id".to_string(), DbValue::I32(self.id))]
    }
}

fn seed_users(ctx: &DbContext, n: usize) -> impl std::future::Future<Output = Result<(), DbError>> + '_ {
    async move {
        let mut rows = Vec::with_capacity(n);
        for i in 0..n {
            let user = User {
                id: i as i32,
                name: format!("User {i}"),
                email: format!("user{i}@example.com"),
                active: (i % 2) == 0,
            };
            rows.push(user.into());
        }

        ctx.insert(Query::insert("users").values(rows)).await?;
        Ok(())
    }
}

fn bench_find_entities_readonly_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    let query = Query::find("users").filter(|f| f.eq("active", true));

    c.bench_function("find_entities_readonly_100k_rows", |b| {
        b.to_async(&rt).iter(|| async {
            let users = ctx
                .find_entities_readonly::<User>(black_box(query.clone()))
                .await
                .unwrap();
            black_box(users.len());
        })
    });
}

fn bench_find_entities_tracked_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    let query = Query::find("users").filter(|f| f.eq("active", true));

    c.bench_function("find_entities_tracked_100k_rows", |b| {
        b.to_async(&rt).iter(|| async {
            let users = ctx
                .find_entities::<User>(black_box(query.clone()))
                .await
                .unwrap();
            black_box(users.len());
        })
    });
}

fn bench_dirty_field_detection_1k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("dirty_field_detection_1k_entities_modify_10pct", |b| {
        b.to_async(&rt).iter(|| async {
            // Fresh DB each iteration so we're measuring the same work.
            let driver = Arc::new(MemoryDriver::new());
            let ctx = DbContext::new(driver);
            seed_users(&ctx, 1_000).await.unwrap();

            let query = Query::find("users").order_by(|s| s.asc("id"));
            let mut entities = ctx.find_entities::<User>(query).await.unwrap();

            // Modify ~10% of the entities then save them.
            for (idx, entity) in entities.iter_mut().enumerate() {
                if idx % 10 == 0 {
                    entity.entity.email = format!("updated{idx}@example.com");
                    entity.save(&ctx).await.unwrap();
                }
            }

            black_box(entities.len());
        })
    });
}

fn bench_find_rows_eq_i32_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    let query = Query::find("users").filter(|f| f.eq("id", 99_999i32));

    c.bench_function("find_rows_eq_i32_100k", |b| {
        b.to_async(&rt).iter(|| async {
            let rows = ctx.find(black_box(query.clone())).await.unwrap();
            black_box(rows.len());
        })
    });
}

fn bench_find_rows_gte_i32_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    let query = Query::find("users").filter(|f| f.gte("id", 50_000i32));

    c.bench_function("find_rows_gte_i32_100k", |b| {
        b.to_async(&rt).iter(|| async {
            let rows = ctx.find(black_box(query.clone())).await.unwrap();
            black_box(rows.len());
        })
    });
}

fn bench_find_rows_contains_string_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    let query = Query::find("users").filter(|f| f.contains("email", "@example.com"));

    c.bench_function("find_rows_contains_string_100k", |b| {
        b.to_async(&rt).iter(|| async {
            let rows = ctx.find(black_box(query.clone())).await.unwrap();
            black_box(rows.len());
        })
    });
}

fn bench_find_rows_regex_string_100k(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    rt.block_on(seed_users(&ctx, 100_000)).unwrap();

    // Note: regex compilation is inside filter evaluation (current design),
    // so this benchmarks both compilation + match cost.
    let query = Query::find("users").filter(|f| f.regex("email", r"^user\\d+@example\\.com$"));

    c.bench_function("find_rows_regex_string_100k", |b| {
        b.to_async(&rt).iter(|| async {
            let rows = ctx.find(black_box(query.clone())).await.unwrap();
            black_box(rows.len());
        })
    });
}

fn bench_query_builder_10k(c: &mut Criterion) {
    c.bench_function("build_query_10k", |b| {
        b.iter(|| {
            for _ in 0..10_000 {
                black_box(
                    Query::find("users")
                        .filter(|f| f.eq("status", "active").gte("age", 18))
                        .order_by(|s| s.asc("created_at"))
                        .limit(10),
                );
            }
        })
    });
}

