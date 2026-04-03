use async_trait::async_trait;

use crate::{
    driver::driver::Driver,
    query::{InsertQuery, UpdateQuery, DeleteQuery, filters::FilterDefinition},
    types::{DbError, DbRow, DbValue, FromDbRow}
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DbEntityState {
    Unchanged,
    Added,
    Modified,
    Deleted,
}

pub type DbEntityKey = Vec<(String, DbValue)>;

pub trait DbEntityModel: FromDbRow + Into<DbRow> + Send + Sync + 'static {
    fn collection_name() -> &'static str;
    fn key(&self) -> DbEntityKey;
    
    fn key_hash(&self) -> String {
        let keys: Vec<String> = self.key().into_iter()
            .map(|(_, v)| format!("{:?}", v))
            .collect();
        format!("{}::{}", Self::collection_name(), keys.join("::"))
    }

    fn key_filter(&self) -> Result<FilterDefinition, DbError> {
        let key_pairs = self.key();
        if key_pairs.is_empty() {
            return Err(DbError::MappingError(format!(
                "Entity '{}' provided an empty key. Operations aborted to prevent accidental mass-deletion/update.",
                Self::collection_name()
            )));
        }

        let mut filter = FilterDefinition::empty();
        for (field, value) in key_pairs {
            filter = filter.eq(field, value);
        }

        Ok(filter)
    }
}

pub struct DbEntity<T: DbEntityModel> {
    inner: T,
    snapshot: Option<DbRow>,
    state: DbEntityState,
}

impl<T: DbEntityModel + Clone> DbEntity<T> {
    /// Use this for brand new records (translates to INSERT)
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            snapshot: None,
            state: DbEntityState::Added,
        }
    }

    /// Use this when loading records from the database
    pub fn from_snapshot(inner: T, snapshot: DbRow) -> Self {
        Self {
            inner,
            snapshot: Some(snapshot),
            state: DbEntityState::Unchanged,
        }
    }

    /// Read-only access to the model. Does NOT trigger a state change.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Mutable access. Automatically flags the entity as Modified!
    pub fn inner_mut(&mut self) -> &mut T {
        if self.state == DbEntityState::Unchanged {
            self.state = DbEntityState::Modified;
        }
        &mut self.inner
    }

    /// Read the current state for the DbContext to check
    pub fn state(&self) -> DbEntityState {
        self.state
    }

    /// Mark the entity for deletion (translates to DELETE)
    pub fn mark_deleted(&mut self) {
        self.state = DbEntityState::Deleted;
    }
    
    /// Consume the wrapper and return the raw model if tracking is no longer needed
    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[async_trait]
pub trait DbTrackedEntity: Send + Sync {
    fn get_state(&self) -> DbEntityState;
    fn set_state(&mut self, state: DbEntityState);
    async fn save_to_db(&mut self, driver: &dyn Driver) -> Result<(), DbError>;
}

#[async_trait]
impl<T: DbEntityModel + Clone> DbTrackedEntity for DbEntity<T> {
    fn get_state(&self) -> DbEntityState {
        self.state // Implicitly copied
    }

    fn set_state(&mut self, state: DbEntityState) {
        self.state = state;
    }

    async fn save_to_db(&mut self, driver: &dyn Driver) -> Result<(), DbError> {
        match self.state {
            DbEntityState::Added => {
                let row: DbRow = self.inner.clone().into();
                let query = InsertQuery::new(T::collection_name()).add_row(row.clone());
                driver.insert(query).await?;
                
                self.snapshot = Some(row);
            }
            DbEntityState::Modified => {
                if let Some(snapshot) = &self.snapshot {
                    let current_row: DbRow = self.inner.clone().into();
                    let mut diff = DbRow::new();

                    for (k, v) in &current_row.0 {
                        // Compare current against snapshot
                        let is_same = snapshot.0.get(k).map(|orig| orig == v).unwrap_or(false);
                        if !is_same {
                            diff.insert(k.clone(), v.clone());
                        }
                    }

                    if !diff.0.is_empty() {
                        let query = UpdateQuery::new(T::collection_name())
                            .filter(self.inner.key_filter()?)
                            .set_row(diff);
                            
                        driver.update(query).await?;
                    }
                    
                    self.snapshot = Some(current_row);
                } else {
                    return Err(DbError::MappingError(
                        "Cannot update an entity that lacks a snapshot. Was it loaded from the DB?".into()
                    ));
                }
            }
            DbEntityState::Deleted => {
                let query = DeleteQuery::new(T::collection_name())
                    .filter(self.inner.key_filter()?);
                    
                driver.delete(query).await?;
                self.snapshot = None;
            }
            DbEntityState::Unchanged => {
                // Do nothing
            }
        }
        Ok(())
    }
}