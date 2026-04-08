// Example of using the DbEntity derive macro (once fully implemented)
// This file demonstrates the intended API for the procedural macro

#![allow(dead_code)]

use simple_db::{
    DbContext,
    driver::memory::MemoryDriver,
    entity::{DbEntity, DbEntityModel},
    query::Query,
};
use std::sync::Arc;

// ==========================================
// Simple Entity with Single Primary Key
// ==========================================

// NOTE: The macro will automatically implement:
// - FromDbRow: Deserialize from DbRow
// - Into<DbRow>: Serialize to DbRow  
// - DbEntityModel: Provide collection_name() and key() methods
//
// Instead of manually implementing all three traits!

/*
#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "users", key = "id")]
struct User {
    id: i32,
    name: String,
    email: String,
}

// ==========================================
// Entity with Composite Primary Key
// ==========================================

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "user_roles", key = "user_id,role_id")]
struct UserRole {
    user_id: i32,
    role_id: i32,
    assigned_at: String,
}

// ==========================================
// Entity with Various Field Types
// ==========================================

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "products", key = "id")]
struct Product {
    id: i32,
    name: String,
    price: f64,
    in_stock: bool,
    quantity: i64,
}

// ==========================================
// Usage Example
// ==========================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Create a new user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Wrap in DbEntity and save
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await?;

    // Load users with read-only query
    let query = Query::find("users").filter(|fb| fb.eq("id", 1));
    let users = ctx.find_entities_readonly::<User>(query).await?;

    println!("Found {} user(s)", users.len());

    // Load users with tracking for updates
    let query = Query::find("users").filter(|fb| fb.eq("id", 1));
    let tracked_users = ctx.find_entities::<User>(query).await?;

    if let Some(tracked) = tracked_users.first() {
        println!("Tracked user: {}", tracked.entity.name);
    }

    Ok(())
}
*/

// For now, you can still manually implement the traits as shown in integration_entity.rs
// The macro will handle this automatically once ready!
