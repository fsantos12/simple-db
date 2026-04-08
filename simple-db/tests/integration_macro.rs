// Integration test for DbEntity macro with various field types

use simple_db::{
    DbContext,
    driver::memory::MemoryDriver,
    entity::DbEntity,
    query::Query,
};
use simple_db_macro::DbEntity;  // Import the derive macro
use std::sync::Arc;

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "simple_users")]
pub struct SimpleUser {
    #[db_entity(primary_key)]
    pub id: i32,
    pub name: String,
}

#[tokio::test]
async fn test_macro_basic_user() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = SimpleUser {
        id: 1,
        name: "Alice".to_string(),
    };

    let mut entity = DbEntity::new(user);
    assert!(entity.save(&ctx).await.is_ok());

    // Try to load back
    let query = Query::find("simple_users").filter(|fb| fb.eq("id", 1));
    let results = ctx.find(query).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "composite_keys")]
pub struct CompositeKeyEntity {
    #[db_entity(primary_key)]
    pub key1: i32,
    #[db_entity(primary_key)]
    pub key2: i32,
    pub data: String,
}

#[tokio::test]
async fn test_macro_composite_key() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let entity_data = CompositeKeyEntity {
        key1: 1,
        key2: 2,
        data: "test".to_string(),
    };

    let mut entity = DbEntity::new(entity_data);
    assert!(entity.save(&ctx).await.is_ok());
}

#[derive(DbEntity, Clone, Debug)]
#[db_entity(collection = "typed_entities")]
pub struct TypedEntity {
    #[db_entity(primary_key)]
    pub id: i64,
    pub flag: bool,
    pub value: f64,
}

#[tokio::test]
async fn test_macro_various_types() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let entity_data = TypedEntity {
        id: 100,
        flag: true,
        value: 3.14,
    };

    let mut entity = DbEntity::new(entity_data);
    assert!(entity.save(&ctx).await.is_ok());
}
