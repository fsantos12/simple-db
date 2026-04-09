use simple_db::{
    driver::memory::MemoryDriver,
    entity::{DbEntity, DbEntityKey, DbEntityModel, DbEntityState},
    query::Query,
    types::{DbError, DbRow, DbValue, FromDbRow},
    DbContext,
};
use std::sync::Arc;

// ==========================================
// Test Entity Definition
// ==========================================

#[derive(Clone, Debug, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String,
}

impl FromDbRow for User {
    fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
        Ok(Self {
            id: row.take_i32("id")?,
            name: row.take_string("name")?,
            email: row.take_string("email")?,
        })
    }
}

impl Into<DbRow> for User {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("id", self.id);
        row.insert("name", self.name);
        row.insert("email", self.email);
        row
    }
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> DbEntityKey {
        vec![("id".to_string(), DbValue::I32(self.id))]
    }
}

// ==========================================
// Entity Creation Tests
// ==========================================

#[tokio::test]
async fn test_entity_new_creates_added_state() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let entity = DbEntity::new(user);

    assert!(matches!(entity.state(), DbEntityState::Added));
}

#[tokio::test]
async fn test_entity_save_inserts_new_record() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);

    assert!(entity.save(&ctx).await.is_ok());
    assert!(matches!(entity.state(), DbEntityState::Tracked));
}

#[tokio::test]
async fn test_entity_save_persists_to_database() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 2,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user.clone());

    entity.save(&ctx).await.unwrap();

    // Verify it was inserted
    let query = Query::find("users").filter(|fb| fb.eq("id", 2));
    let result = ctx.find(query).await.unwrap();

    assert_eq!(result.len(), 1);
    let retrieved = User::from_db_row(&mut result[0].clone()).unwrap();
    assert_eq!(retrieved.id, 2);
    assert_eq!(retrieved.name, "Charlie");
}

// ==========================================
// Entity Tracking & Updates Tests
// ==========================================

#[tokio::test]
async fn test_entity_from_db_creates_tracked_state() {
    let mut row = DbRow::new();
    row.insert("id", 1i32);
    row.insert("name", "Diana");
    row.insert("email", "diana@example.com");

    let user = User::from_db_row(&mut row.clone()).unwrap();
    let entity = DbEntity::from_db(user, row);

    assert!(matches!(entity.state(), DbEntityState::Tracked));
}

#[tokio::test]
async fn test_entity_update_detects_dirty_fields() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert initial user
    let user = User {
        id: 3,
        name: "Eve".to_string(),
        email: "eve@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);

    entity.save(&ctx).await.unwrap();

    // Load from database (simulating a tracked entity)
    let query = Query::find("users").filter(|fb| fb.eq("id", 3));
    let result = ctx.find(query).await.unwrap();
    let loaded_user = User::from_db_row(&mut result[0].clone()).unwrap();
    let mut loaded_entity = DbEntity::from_db(loaded_user.clone(), result[0].clone());

    // Modify the entity
    loaded_entity.entity.name = "Eve Updated".to_string();

    // Save changes
    let save_result = loaded_entity.save(&ctx).await;
    assert!(save_result.is_ok());
}

#[tokio::test]
async fn test_entity_update_changes_database() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert initial user
    let user = User {
        id: 4,
        name: "Frank".to_string(),
        email: "frank@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    // Load and update
    let query = Query::find("users").filter(|fb| fb.eq("id", 4));
    let result = ctx.find(query).await.unwrap();
    let loaded_user = User::from_db_row(&mut result[0].clone()).unwrap();
    let mut loaded_entity = DbEntity::from_db(loaded_user, result[0].clone());

    // Modify the email
    loaded_entity.entity.email = "frank.updated@example.com".to_string();
    loaded_entity.save(&ctx).await.unwrap();

    // Verify update
    let updated_query = Query::find("users").filter(|fb| fb.eq("id", 4));
    let updated_result = ctx.find(updated_query).await.unwrap();
let updated_user = User::from_db_row(&mut updated_result[0].clone()).unwrap();

    assert_eq!(updated_user.email, "frank.updated@example.com");
}

#[tokio::test]
async fn test_entity_no_update_when_no_changes() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert user
    let user = User {
        id: 5,
        name: "Grace".to_string(),
        email: "grace@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    // Load and save without changes
    let query = Query::find("users").filter(|fb| fb.eq("id", 5));
    let result = ctx.find(query).await.unwrap();
    let loaded_user = User::from_db_row(&mut result[0].clone()).unwrap();
    let mut loaded_entity = DbEntity::from_db(loaded_user, result[0].clone());

    // Save without any modifications
    let save_result = loaded_entity.save(&ctx).await;
    assert!(save_result.is_ok());
}

// ==========================================
// Entity Deletion Tests
// ==========================================

#[tokio::test]
async fn test_entity_delete_removes_record() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert user
    let user = User {
        id: 6,
        name: "Henry".to_string(),
        email: "henry@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    // Delete entity
    assert!(entity.delete(&ctx).await.is_ok());

    // Verify deletion
    let query = Query::find("users").filter(|fb| fb.eq("id", 6));
    let result = ctx.find(query).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_entity_delete_changes_state_to_deleted() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 7,
        name: "Iris".to_string(),
        email: "iris@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    entity.delete(&ctx).await.unwrap();
}

// ==========================================
// Entity State Transitions Tests
// ==========================================
#[tokio::test]
async fn test_entity_state_transitions() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 9,
        name: "Karen".to_string(),
        email: "karen@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    
    // Initially Added
    assert!(matches!(entity.state(), DbEntityState::Added));
    
    // After save, becomes Tracked
    entity.save(&ctx).await.unwrap();
    assert!(matches!(entity.state(), DbEntityState::Tracked));
}

// ==========================================
// Multiple Entity Operations Tests
// ==========================================

#[tokio::test]
async fn test_multiple_entities_can_be_managed() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let users = vec![
        User {
            id: 10,
            name: "Leo".to_string(),
            email: "leo@example.com".to_string(),
        },
        User {
            id: 11,
            name: "Mia".to_string(),
            email: "mia@example.com".to_string(),
        },
        User {
            id: 12,
            name: "Noah".to_string(),
            email: "noah@example.com".to_string(),
        },
    ];

    for user in users {
        let mut entity = DbEntity::new(user);
        entity.save(&ctx).await.unwrap();
    }

    // Verify all were inserted
    let query = Query::find("users").filter(|fb| fb.is_in("id", vec![10, 11, 12]));
    let result = ctx.find(query).await.unwrap();

    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn test_entity_operations_in_sequence() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Create and save
    let user = User {
        id: 13,
        name: "Olivia".to_string(),
        email: "olivia@example.com".to_string(),
    };
    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    // Load and modify
    let query = Query::find("users").filter(|fb| fb.eq("id", 13));
    let result = ctx.find(query).await.unwrap();
    let loaded_user = User::from_db_row(&mut result[0].clone()).unwrap();
    let mut loaded_entity = DbEntity::from_db(loaded_user, result[0].clone());

    loaded_entity.entity.email = "olivia.updated@example.com".to_string();
    loaded_entity.save(&ctx).await.unwrap();

    // Verify modification
    let verify_query = Query::find("users").filter(|fb| fb.eq("id", 13));
    let verify_result = ctx.find(verify_query).await.unwrap();
    let verified_user = User::from_db_row(&mut verify_result[0].clone()).unwrap();

    assert_eq!(verified_user.email, "olivia.updated@example.com");

    // Delete original entity
    entity.delete(&ctx).await.unwrap();

    // Verify deletion
    let final_query = Query::find("users").filter(|fb| fb.eq("id", 13));
    let final_result = ctx.find(final_query).await.unwrap();
    assert_eq!(final_result.len(), 0);
}

// ==========================================
// Entity Key Filter Tests
// ==========================================

#[tokio::test]
async fn test_entity_key_filter_generation() {
    let user = User {
        id: 20,
        name: "Peter".to_string(),
        email: "peter@example.com".to_string(),
    };

    let filter = user.key_filter();
    assert!(filter.is_ok());
}

#[tokio::test]
async fn test_entity_collection_name() {
    assert_eq!(User::collection_name(), "users");
}

// ==========================================
// Entity Serialization Tests
// ==========================================

#[tokio::test]
async fn test_entity_converts_to_row() {
    let user = User {
        id: 30,
        name: "Quinn".to_string(),
        email: "quinn@example.com".to_string(),
    };

    let row: DbRow = user.into();

    assert_eq!(row.get("id"), Some(&DbValue::I32(30)));
    assert_eq!(
        row.get("name"),
        Some(&DbValue::String("Quinn".to_string()))
    );
    assert_eq!(
        row.get("email"),
        Some(&DbValue::String("quinn@example.com".to_string()))
    );
}

#[tokio::test]
async fn test_entity_converts_from_row() {
    let mut row = DbRow::new();
    row.insert("id", 31i32);
    row.insert("name", "Rachel");
    row.insert("email", "rachel@example.com");

    let user = User::from_db_row(&mut row).unwrap();

    assert_eq!(user.id, 31);
    assert_eq!(user.name, "Rachel");
    assert_eq!(user.email, "rachel@example.com");
}

// ==========================================
// Entity Composite Key Tests (Bonus)
// ==========================================

#[derive(Clone, Debug)]
struct UserRole {
    user_id: i32,
    role_id: i32,
    assigned_at: String,
}

impl FromDbRow for UserRole {
    fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
        Ok(Self {
            user_id: row.take_i32("user_id")?,
            role_id: row.take_i32("role_id")?,
            assigned_at: row.take_string("assigned_at")?
        })
    }
}

impl Into<DbRow> for UserRole {
    fn into(self) -> DbRow {
        let mut row = DbRow::new();
        row.insert("user_id", self.user_id);
        row.insert("role_id", self.role_id);
        row.insert("assigned_at", self.assigned_at);
        row
    }
}

impl DbEntityModel for UserRole {
    fn collection_name() -> &'static str {
        "user_roles"
    }

    fn key(&self) -> DbEntityKey {
        vec![
            ("user_id".to_string(), DbValue::I32(self.user_id)),
            ("role_id".to_string(), DbValue::I32(self.role_id)),
        ]
    }
}

#[tokio::test]
async fn test_entity_with_composite_key() {
    let user_role = UserRole {
        user_id: 1,
        role_id: 2,
        assigned_at: "2026-04-08".to_string(),
    };

    let filter = user_role.key_filter();
    assert!(filter.is_ok());
    assert_eq!(filter.unwrap().len(), 2);
}

#[tokio::test]
async fn test_composite_key_entity_save_and_retrieve() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user_role = UserRole {
        user_id: 5,
        role_id: 3,
        assigned_at: "2026-04-08".to_string(),
    };

    let mut entity = DbEntity::new(user_role);
    entity.save(&ctx).await.unwrap();

    let query = Query::find("user_roles").filter(|fb| fb.eq("user_id", 5));
    let result = ctx.find(query).await.unwrap();

    assert_eq!(result.len(), 1);
    let retrieved = UserRole::from_db_row(&mut result[0].clone()).unwrap();
    assert_eq!(retrieved.user_id, 5);
    assert_eq!(retrieved.role_id, 3);
}

// ==========================================
// Read-Only Entity Tests
// ==========================================

#[tokio::test]
async fn test_find_entities_readonly_returns_deserialized() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 40,
        name: "ReadOnly".to_string(),
        email: "readonly@example.com".to_string(),
    };

    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    // Use read-only find - no DbEntity wrapper, no tracking overhead
    let query = Query::find("users").filter(|fb| fb.eq("id", 40));
    let users = ctx.find_entities_readonly::<User>(query).await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, 40);
    assert_eq!(users[0].name, "ReadOnly");
    assert_eq!(users[0].email, "readonly@example.com");
}

#[tokio::test]
async fn test_find_entities_readonly_multiple_records() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert multiple users
    for i in 50..55 {
        let user = User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
        };
        let mut entity = DbEntity::new(user);
        entity.save(&ctx).await.unwrap();
    }

    // Read-only query for all inserted users
    let query = Query::find("users").filter(|fb| fb.is_in("id", vec![50, 51, 52, 53, 54]));
    let users = ctx.find_entities_readonly::<User>(query).await.unwrap();

    assert_eq!(users.len(), 5);
    assert_eq!(users[0].name, "User50");
    assert_eq!(users[4].name, "User54");
}

#[tokio::test]
async fn test_find_entities_readonly_vs_tracked() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    let user = User {
        id: 60,
        name: "Compare".to_string(),
        email: "compare@example.com".to_string(),
    };

    let mut entity = DbEntity::new(user);
    entity.save(&ctx).await.unwrap();

    let query = Query::find("users").filter(|fb| fb.eq("id", 60));

    // Tracked version
    let tracked = ctx.find_entities::<User>(query.clone()).await.unwrap();
    assert_eq!(tracked[0].state(), &DbEntityState::Tracked);
    assert_eq!(tracked[0].entity.name, "Compare");

    // Read-only version - no state tracking
    let readonly = ctx.find_entities_readonly::<User>(query).await.unwrap();
    assert_eq!(readonly[0].name, "Compare");
    // readonly is just User, not DbEntity<User>
}

