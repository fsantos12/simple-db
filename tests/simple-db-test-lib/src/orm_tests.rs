use std::time::Instant;

use futures::future::BoxFuture;
use simple_db::{DbContext, DbEntity, DbEntityTrait};
use simple_db::filter;
use simple_db::types::{DbValue, DbRow};
use simple_db::query::Query;

// ─────────────────────────────────────────────────────────────────────────────
// ORM Entity for Testing
// ─────────────────────────────────────────────────────────────────────────────

/// Test entity mapping to the users table (id, name, email fields only)
#[derive(Debug, Clone, PartialEq)]
pub struct UserEntity {
    id: i64,
    name: String,
    email: String,
}

impl UserEntity {
    pub fn new(id: i64, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn set_email(&mut self, email: String) {
        self.email = email;
    }
}

impl DbEntityTrait for UserEntity {
    fn collection_name() -> &'static str {
        "users"
    }

    fn primary_key(&self) -> Vec<(&'static str, DbValue)> {
        vec![("id", DbValue::from_i64(self.id))]
    }

    fn to_db(&self) -> Vec<(&'static str, DbValue)> {
        vec![
            ("id", DbValue::from_i64(self.id)),
            ("name", DbValue::from_string(self.name.clone())),
            ("email", DbValue::from_string(self.email.clone())),
        ]
    }

    fn from_db(row: &dyn DbRow) -> Self {
        UserEntity {
            id: row.get_by_name("id")
                .and_then(|v| v.as_i64().or_else(|| v.as_i32().map(|i| i as i64)))
                .unwrap_or(0),
            name: row.get_by_name("name")
                .and_then(|v| v.as_string().map(|s| s.to_string()))
                .unwrap_or_default(),
            email: row.get_by_name("email")
                .and_then(|v| v.as_string().map(|s| s.to_string()))
                .unwrap_or_default(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Output helpers
// ─────────────────────────────────────────────────────────────────────────────

fn header(name: &str) {
    println!("┌─ {name}");
}

fn detail(line: &str) {
    println!("│  {line}");
}

fn footer(passed: bool, elapsed: std::time::Duration) {
    let icon = if passed { "✅ Passed" } else { "❌ Failed" };
    let ms = elapsed.as_secs_f64() * 1000.0;
    println!("└─ {icon}  │  {ms:.3} ms");
    println!();
}

fn check(ok: bool) -> &'static str {
    if ok { "✓" } else { "✗" }
}

// ─────────────────────────────────────────────────────────────────────────────
// DB helpers
// ─────────────────────────────────────────────────────────────────────────────

async fn truncate(ctx: &DbContext) {
    ctx.delete(Query::delete("users")).await.unwrap();
}

async fn truncate_and_reset(ctx: &DbContext) {
    truncate(ctx).await;
    let _ = ctx.delete(Query::delete("users")).await;
}

// ─────────────────────────────────────────────────────────────────────────────
// ORM Integration Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Save new untracked entity (INSERT)
fn orm_save_new(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();
        truncate_and_reset(&ctx).await;
        header("ORM · Save new entity (INSERT)");

        let user = UserEntity::new(10000, "Alice".to_string(), "alice@example.com".to_string());
        let mut entity = DbEntity::new(user.clone());

        let was_untracked = entity.is_untracked();

        let save_result = entity.save(&ctx).await;
        let save_ok = save_result.is_ok();

        let is_tracked = entity.is_tracked();

        let rows = UserEntity::find(&ctx, filter!(eq("id", 10000))).await.unwrap();
        let row_found = !rows.is_empty();

        let row_matches = if let Some(loaded) = rows.get(0) {
            let loaded_user = loaded.get();
            let id_match = loaded_user.id == user.id;
            let name_match = loaded_user.name == user.name;
            let email_match = loaded_user.email == user.email;

            if !id_match || !name_match || !email_match {
                detail(&format!(
                    "DEBUG - Expected: id={}, name='{}', email='{}'",
                    user.id, user.name, user.email
                ));
                detail(&format!(
                    "DEBUG - Got:      id={}, name='{}', email='{}'",
                    loaded_user.id, loaded_user.name, loaded_user.email
                ));
            }

            id_match && name_match && email_match
        } else {
            false
        };

        let ok = was_untracked && save_ok && is_tracked && row_found && row_matches;
        detail(&format!(
            "Untracked before: {} | Saved: {} | Tracked after: {} | Row found: {} | Matches: {}",
            check(was_untracked), check(save_ok), check(is_tracked), check(row_found), check(row_matches)
        ));

        footer(ok, t.elapsed());
        ok
    })
}

/// Save update to tracked entity (UPDATE with partial fields)
fn orm_save_update(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();
        truncate(&ctx).await;
        header("ORM · Save update to tracked entity (UPDATE)");

        ctx.insert(Query::insert("users").insert(vec![
            ("id", DbValue::from_i64(10001)),
            ("name", DbValue::from_string("Bob".to_string())),
            ("email", DbValue::from_string("bob@example.com".to_string())),
        ])).await.unwrap();

        let mut entities = UserEntity::find(&ctx, filter!(eq("id", 10001))).await.unwrap();
        let loaded = entities.len() == 1;

        if let Some(entity) = entities.get_mut(0) {
            let was_tracked = entity.is_tracked();

            entity.get_mut().email = "bob.new@example.com".to_string();

            let save_ok = entity.save(&ctx).await.is_ok();

            let updated = UserEntity::find(&ctx, filter!(eq("id", 10001))).await.unwrap();

            let name_unchanged = updated.get(0).map(|e| e.get().name() == "Bob").unwrap_or(false);
            let email_changed = updated.get(0).map(|e| e.get().email() == "bob.new@example.com").unwrap_or(false);

            let ok = loaded && was_tracked && save_ok && name_unchanged && email_changed;
            detail(&format!(
                "Loaded: {} | Was tracked: {} | Saved: {} | Name unchanged: {} | Email changed: {}",
                check(loaded), check(was_tracked), check(save_ok), check(name_unchanged), check(email_changed)
            ));

            footer(ok, t.elapsed());
            ok
        } else {
            footer(false, t.elapsed());
            false
        }
    })
}

/// Delete tracked entity
fn orm_delete(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();
        truncate(&ctx).await;
        header("ORM · Delete tracked entity");

        ctx.insert(Query::insert("users").insert(vec![
            ("id", DbValue::from_i64(10002)),
            ("name", DbValue::from_string("Charlie")),
            ("email", DbValue::from_string("charlie@example.com")),
        ])).await.unwrap();

        let mut entities = UserEntity::find(&ctx, filter!(eq("id", 10002))).await.unwrap();
        let loaded = entities.len() == 1;

        if let Some(entity) = entities.get_mut(0) {
            let was_tracked = entity.is_tracked();

            let delete_ok = entity.delete(&ctx).await.is_ok();

            let is_detached = entity.is_detached();

            let remaining = UserEntity::find(&ctx, filter!(eq("id", 10002))).await.unwrap();
            let row_deleted = remaining.is_empty();

            let ok = loaded && was_tracked && delete_ok && is_detached && row_deleted;
            detail(&format!(
                "Loaded: {} | Was tracked: {} | Deleted: {} | Detached: {} | Row gone: {}",
                check(loaded), check(was_tracked), check(delete_ok), check(is_detached), check(row_deleted)
            ));

            footer(ok, t.elapsed());
            ok
        } else {
            footer(false, t.elapsed());
            false
        }
    })
}

/// Load entity as tracked
fn orm_load_tracked(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();
        truncate(&ctx).await;
        header("ORM · Load as tracked entity");

        ctx.insert(Query::insert("users").insert(vec![
            ("id", DbValue::from_i64(10003)),
            ("name", DbValue::from_string("Diana")),
            ("email", DbValue::from_string("diana@example.com")),
        ])).await.unwrap();

        let entities = UserEntity::find(&ctx, filter!(eq("id", 10003))).await.unwrap();
        let has_results = !entities.is_empty();

        if let Some(entity) = entities.get(0) {
            let is_tracked = entity.is_tracked();
            let not_untracked = !entity.is_untracked();
            let not_detached = !entity.is_detached();
            let name_matches = entity.get().name() == "Diana";
            let email_matches = entity.get().email() == "diana@example.com";

            let ok = has_results && is_tracked && not_untracked && not_detached && name_matches && email_matches;
            detail(&format!(
                "Found: {} | Tracked: {} | Name: {} | Email: {}",
                check(has_results), check(is_tracked), check(name_matches), check(email_matches)
            ));

            footer(ok, t.elapsed());
            ok
        } else {
            footer(false, t.elapsed());
            false
        }
    })
}

/// Load entity as read-only (detached)
fn orm_load_readonly(context: &DbContext) -> BoxFuture<'static, bool> {
    let ctx = context.clone();
    Box::pin(async move {
        let t = Instant::now();
        truncate(&ctx).await;
        header("ORM · Load as read-only/detached entity");

        ctx.insert(Query::insert("users").insert(vec![
            ("id", DbValue::from_i64(10004)),
            ("name", DbValue::from_string("Eve")),
            ("email", DbValue::from_string("eve@example.com")),
        ])).await.unwrap();

        let entities = UserEntity::find_readonly(&ctx, filter!(eq("id", 10004))).await.unwrap();
        let has_results = !entities.is_empty();

        if let Some(entity) = entities.get(0) {
            let is_detached = entity.is_detached();
            let not_tracked = !entity.is_tracked();
            let not_untracked = !entity.is_untracked();
            let name_matches = entity.get().name() == "Eve";
            let email_matches = entity.get().email() == "eve@example.com";

            let ok = has_results && is_detached && not_tracked && not_untracked && name_matches && email_matches;
            detail(&format!(
                "Found: {} | Detached: {} | Name: {} | Email: {}",
                check(has_results), check(is_detached), check(name_matches), check(email_matches)
            ));

            footer(ok, t.elapsed());
            ok
        } else {
            footer(false, t.elapsed());
            false
        }
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Registry
// ─────────────────────────────────────────────────────────────────────────────

pub fn get_orm_test_cases() -> Vec<(&'static str, fn(&DbContext) -> BoxFuture<'static, bool>)> {
    vec![
        ("ORM · Save new entity (INSERT)",           orm_save_new),
        ("ORM · Save update to entity (UPDATE)",     orm_save_update),
        ("ORM · Delete tracked entity",              orm_delete),
        ("ORM · Load as tracked entity",             orm_load_tracked),
        ("ORM · Load as read-only/detached entity",  orm_load_readonly),
    ]
}
