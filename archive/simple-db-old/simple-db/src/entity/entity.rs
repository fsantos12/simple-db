//! Entity model definitions and change tracking.
//!
//! Provides the ORM layer with entity state management (Added, Tracked, Deleted, Detached),
//! change detection, and persistence operations for converting between Rust types and
//! database rows.

use crate::{DbContext, query::{Query, filters::{FilterBuilder, FilterDefinition}}, types::{DbError, DbRow, DbValue, FromDbRow}};
use once_cell::sync::OnceCell;

// ==========================================
// Entity State Management
// ==========================================

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DbEntityState {
    /// Entity is new and not yet in the DB.
    Added,
    /// Entity is known to the DB and tracked for changes.
    Tracked,
    /// Entity has been marked for removal or deleted.
    Deleted
}

pub type DbEntityKey = Vec<(String, DbValue)>;
// ==========================================
// Entity Model Contract
// ==========================================

pub trait DbEntityModel: FromDbRow + Into<DbRow> + Send + Sync + Clone + 'static {
    fn collection_name() -> &'static str;
    fn key(&self) -> DbEntityKey;

    /// Generates a database filter safely based on the key fields. Throws an error if the key is empty to prevent mass operations.
    fn key_filter(&self) -> Result<FilterDefinition, DbError> {
        let key_pairs = self.key();
        if key_pairs.is_empty() {
            return Err(DbError::MappingError(format!(
                "Entity '{}' provided an empty key. Operations aborted to prevent accidental mass-deletion/update.",
                Self::collection_name()
            )));
        }

        let mut filter = FilterBuilder::new();
        for (field, value) in key_pairs {
            filter = filter.eq(field, value);
        }

        Ok(filter.build())
    }
}

// ==========================================
// Tracked Entity & Persistence
// ==========================================
pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: OnceCell<DbRow>,
    state: DbEntityState
}

impl<T: DbEntityModel> DbEntity<T> {
    pub fn new(entity: T) -> Self {
        Self {
            entity,
            snapshot: OnceCell::new(),
            state: DbEntityState::Added
        }
    }

    /// Wraps data loaded from the database, marking the entity as tracked.
    pub fn from_db(entity: T, row: DbRow) -> Self {
        let snapshot = OnceCell::new();
        let _ = snapshot.set(row);
        Self {
            entity,
            snapshot,
            state: DbEntityState::Tracked,
        }
    }

    fn dirty_fields(&self) -> DbRow {
        let current: DbRow = self.entity.clone().into();
        let mut updates = DbRow::new();

        if let Some(original) = self.snapshot.get() {
            for (field, val) in &current.0 {
                if original.get(field)!= Some(val) {
                    updates.insert(field.clone(), val.clone());
                }
            }
        }

        updates
    }

    /// Persists changes to the database.
    pub async fn save(&mut self, ctx: &DbContext) -> Result<(), DbError> {
        match self.state {
            DbEntityState::Added => {
                let row: DbRow = self.entity.clone().into();
                let q = Query::insert(T::collection_name()).insert(row.clone());
                ctx.insert(q).await?;
                let _ = self.snapshot.set(row);
                self.state = DbEntityState::Tracked;
            }
            DbEntityState::Tracked => {
                let updates = self.dirty_fields();
                if!updates.0.is_empty() {
                    let q = Query::update(T::collection_name())
                       .set_row(updates)
                       .with_filters(self.entity.key_filter()?);
                    ctx.update(q).await?;
                    let _ = self.snapshot.set(self.entity.clone().into());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Removes the record from the database.
    pub async fn delete(mut self, ctx: &DbContext) -> Result<(), DbError> {
        let q = Query::delete(T::collection_name())
           .with_filters(self.entity.key_filter()?);
        ctx.delete(q).await?;
        self.state = DbEntityState::Deleted;
        Ok(())
    }

    pub fn state(&self) -> &DbEntityState { &self.state }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test entity model
    #[derive(Clone, Debug)]
    struct TestUser {
        id: i32,
        name: String,
        email: String,
    }

    impl FromDbRow for TestUser {
        fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
            Ok(Self {
                id: row.take_i32("id")?,
                name: row.take_string("name")?,
                email: row.take_string("email")?,
            })
        }
    }

    impl Into<DbRow> for TestUser {
        fn into(self) -> DbRow {
            let mut row = DbRow::new();
            row.insert("id", self.id);
            row.insert("name", self.name);
            row.insert("email", self.email);
            row
        }
    }

    impl DbEntityModel for TestUser {
        fn collection_name() -> &'static str {
            "users"
        }

        fn key(&self) -> DbEntityKey {
            vec![("id".to_string(), DbValue::I32(self.id))]
        }
    }

    fn create_test_user(id: i32, name: &str, email: &str) -> TestUser {
        TestUser {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    #[test]
    fn test_entity_new_is_added_state() {
        let user = create_test_user(1, "Alice", "alice@example.com");
        let entity = DbEntity::new(user);

        assert!(matches!(entity.state(), DbEntityState::Added));
    }

    #[test]
    fn test_entity_from_db_is_tracked_state() {
        let user = create_test_user(1, "Bob", "bob@example.com");
        let row: DbRow = user.clone().into();

        let entity = DbEntity::from_db(user, row);

        assert!(matches!(entity.state(), DbEntityState::Tracked));
    }

    #[test]
    fn test_dirty_fields_detects_changes() {
        let user = create_test_user(1, "Diana", "diana@example.com");
        let row: DbRow = user.clone().into();
        let mut entity = DbEntity::from_db(user, row);

        // Modify the entity
        entity.entity.name = "Diana Updated".to_string();

        let dirty = entity.dirty_fields();
        assert!(dirty.get("name").is_some());
        assert!(dirty.get("id").is_none()); // ID didn't change
        assert!(dirty.get("email").is_none()); // email didn't change
    }

    #[test]
    fn test_dirty_fields_multiple_changes() {
        let user = create_test_user(1, "Eve", "eve@example.com");
        let row: DbRow = user.clone().into();
        let mut entity = DbEntity::from_db(user, row);

        // Modify multiple fields
        entity.entity.name = "Eve Updated".to_string();
        entity.entity.email = "eve.updated@example.com".to_string();

        let dirty = entity.dirty_fields();
        assert!(dirty.get("name").is_some());
        assert!(dirty.get("email").is_some());
        assert!(dirty.get("id").is_none());
    }

    #[test]
    fn test_dirty_fields_no_changes() {
        let user = create_test_user(1, "Frank", "frank@example.com");
        let row: DbRow = user.clone().into();
        let entity = DbEntity::from_db(user, row);

        let dirty = entity.dirty_fields();
        assert_eq!(dirty.0.len(), 0); // No changes
    }

    #[test]
    fn test_key_filter_success() {
        let user = create_test_user(42, "Grace", "grace@example.com");
        let entity = DbEntity::new(user);

        let filter = entity.entity.key_filter();
        assert!(filter.is_ok());
        assert_eq!(filter.unwrap().len(), 1);
    }

    #[test]
    fn test_key_filter_error_on_empty_key() {
        #[derive(Clone, Debug)]
        struct UserWithoutKey {
            name: String,
        }

        impl FromDbRow for UserWithoutKey {
            fn from_db_row(row: &mut DbRow) -> Result<Self, DbError> {
                Ok(Self {
                    name: row.take_string("name")?,
                })
            }
        }

        impl Into<DbRow> for UserWithoutKey {
            fn into(self) -> DbRow {
                let mut row = DbRow::new();
                row.insert("name", self.name);
                row
            }
        }

        impl DbEntityModel for UserWithoutKey {
            fn collection_name() -> &'static str {
                "users"
            }

            fn key(&self) -> DbEntityKey {
                vec![] // Empty key
            }
        }

        let user = UserWithoutKey {
            name: "Henry".to_string(),
        };

        let filter = user.key_filter();
        assert!(filter.is_err());
    }

    #[test]
    fn test_entity_snapshot_initialized_from_db() {
        let user = create_test_user(1, "Ivy", "ivy@example.com");
        let row: DbRow = user.clone().into();

        let entity = DbEntity::from_db(user, row.clone());

        // The snapshot should be exactly the row we provided
        assert_eq!(entity.snapshot.get().unwrap().0.len(), 3);
    }

    #[test]
    fn test_entity_no_snapshot_when_new() {
        let user = create_test_user(1, "Jack", "jack@example.com");
        let entity = DbEntity::new(user);

        assert!(entity.snapshot.get().is_none());
    }

    #[test]
    fn test_collection_name() {
        assert_eq!(TestUser::collection_name(), "users");
    }
}



