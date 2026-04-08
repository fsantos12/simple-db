// Example: Using the DbEntity macro with field-level primary_key attributes

use simple_db::{
    DbContext,
    driver::memory::MemoryDriver,
    entity::DbEntity,
    query::Query,
};
use rust_decimal::Decimal;
use uuid::Uuid;
use std::sync::Arc;

// ==========================================
// Simple Entity - Single Primary Key
// ==========================================

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "users")]
pub struct User {
    #[db_entity(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
}

// ==========================================
// Entity with Composite Primary Key
// ==========================================

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "user_roles")]
pub struct UserRole {
    #[db_entity(primary_key)]
    pub user_id: i32,
    #[db_entity(primary_key)]
    pub role_id: i32,
    pub assigned_at: String,
}

// ==========================================
// Entity with Various Field Types
// ==========================================

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "products")]
pub struct Product {
    #[db_entity(primary_key)]
    pub id: i32,
    pub name: String,
    pub price: Decimal,          // take_decimal() - proper type!
    pub stock: i64,              // take_i64()
    pub available: bool,         // take_bool()
    pub product_uuid: Uuid,      // take_uuid() - proper type!
}

// ==========================================
// ABOUT OWNERSHIP: Why take_* is good
// ==========================================
//
// When DbRow deserializes data to your entity, the `take_*` methods:
//
// 1. **Consume the field from the HashMap**: The value is MOVED, not cloned
// 2. **Prevents duplicate usage**: Once taken, it's gone - no accidental reuse
// 3. **Zero-copy efficiency**: For large types (String, Decimal, Vec<u8>, Uuid),
//    values are unboxed and moved directly without cloning
// 4. **Requires &mut**: The macro generates code with `mut row` - this is intentional
//    because we're modifying the HashMap (removing fields)
//
// Example comparison:
//
//   // ❌ What you might do manually (inefficient):
//   let name = row.get_string("name")?.clone();  // Clone!
//   let id = row.get_i32("id")?;                 // Copy (no clone)
//
//   // ✅ What the macro generates (efficient):
//   let name = row.take_string("name")?;  // Moved (no clone!)
//   let id = row.take_i32("id")?;         // Moved (small, stored inline)
//
// This is NOT a problem because:
// - You consume the row once during deserialization
// - Each field is taken exactly once
// - The row is discarded after FromDbRow completes
//
// Real-world impact: Large records deserialize without allocations!

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Create and save a user
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await?;

    // Create a product with Decimal price (now properly handled!)
    let product = Product {
        id: 100,
        name: "Laptop".to_string(),
        price: Decimal::new(99999, 2),  // $999.99
        stock: 5,
        available: true,
        product_uuid: Uuid::nil(),
    };

    let mut product_entity = DbEntity::new(product);
    product_entity.save(&ctx).await?;

    // Read-only query (no entity tracking overhead)
    let query = Query::find("users").filter(|fb| fb.eq("id", 1));
    let users = ctx.find_entities_readonly::<User>(query).await?;

    if let Some(user) = users.first() {
        println!("Found user: {} ({})", user.username, user.email);
    }

    // Tracked query (for updates)
    let query = Query::find("products").filter(|fb| fb.eq("id", 100));
    let mut products = ctx.find_entities::<Product>(query).await?;

    if let Some(mut entity) = products.into_iter().next() {
        entity.entity.price = Decimal::new(49999, 2); // $499.99
        entity.save(&ctx).await?;
        println!("Price updated!");
    }

    Ok(())
}
