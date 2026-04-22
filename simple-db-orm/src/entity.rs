use async_trait::async_trait;
use simple_db_core::{driver::DbExecutor, query::{FilterDefinition, InsertQuery, UpdateQuery, FindQuery}, types::{DbRow, DbValue, DbResult}};

#[async_trait]
pub trait DbEntityTrait: Clone {
    fn collection_name() -> &'static str;
    fn primary_key(&self) -> Vec<(&'static str, DbValue)>;

    fn to_db(&self) -> Vec<(&'static str, DbValue)>;
    fn from_db(row: &dyn DbRow) -> Self;

    fn primary_key_filter(&self) -> FilterDefinition {
        use simple_db_core::query::FilterBuilder;
        let pk = self.primary_key();
        pk.into_iter()
            .fold(FilterBuilder::new(), |builder, (key, val)| builder.eq(key, val))
            .build()
    }

    /// Returns all matching entities as tracked (change-detected).
    ///
    /// The closure receives a pre-built `FindQuery` for this collection so you
    /// can add filters, sorts, pagination, and projections without repeating
    /// the collection name:
    ///
    /// ```rust,ignore
    /// UserEntity::find(&ctx, |q| q.filter(filter!(eq("active", true))).order_by(sort!(asc("name"))).limit(20)).await?
    /// ```
    async fn find<F>(executor: &dyn DbExecutor, build: F) -> DbResult<Vec<DbEntity<Self>>>
    where
        F: FnOnce(FindQuery) -> FindQuery + Send,
    {
        let query = build(FindQuery::new(Self::collection_name()));
        let mut cursor = executor.find(query).await?;

        let mut entities = Vec::new();
        while let Some(row) = cursor.next().await? {
            entities.push(DbEntity::from_db(row.as_ref()));
        }

        Ok(entities)
    }

    /// Returns all matching entities as detached (read-only, no change tracking).
    ///
    /// Same query builder API as [`find`](DbEntityTrait::find).
    async fn find_readonly<F>(executor: &dyn DbExecutor, build: F) -> DbResult<Vec<DbEntity<Self>>>
    where
        F: FnOnce(FindQuery) -> FindQuery + Send,
    {
        let query = build(FindQuery::new(Self::collection_name()));
        let mut cursor = executor.find(query).await?;

        let mut entities = Vec::new();
        while let Some(row) = cursor.next().await? {
            entities.push(DbEntity::from_db_readonly(row.as_ref()));
        }

        Ok(entities)
    }

    /// Returns the first matching entity as tracked, or `None` if no row matched.
    ///
    /// Automatically applies `LIMIT 1`. Use the closure to add a filter or sort
    /// so the "first" row is deterministic:
    ///
    /// ```rust,ignore
    /// UserEntity::find_one(&ctx, |q| q.filter(filter!(eq("id", 42)))).await?
    /// ```
    async fn find_one<F>(executor: &dyn DbExecutor, build: F) -> DbResult<Option<DbEntity<Self>>>
    where
        F: FnOnce(FindQuery) -> FindQuery + Send,
    {
        let query = build(FindQuery::new(Self::collection_name())).limit(1);
        let mut cursor = executor.find(query).await?;

        if let Some(row) = cursor.next().await? {
            Ok(Some(DbEntity::from_db(row.as_ref())))
        } else {
            Ok(None)
        }
    }

    /// Returns the first matching entity as detached (read-only), or `None`.
    ///
    /// Same as [`find_one`](DbEntityTrait::find_one) but without change tracking.
    async fn find_one_readonly<F>(executor: &dyn DbExecutor, build: F) -> DbResult<Option<DbEntity<Self>>>
    where
        F: FnOnce(FindQuery) -> FindQuery + Send,
    {
        let query = build(FindQuery::new(Self::collection_name())).limit(1);
        let mut cursor = executor.find(query).await?;

        if let Some(row) = cursor.next().await? {
            Ok(Some(DbEntity::from_db_readonly(row.as_ref())))
        } else {
            Ok(None)
        }
    }

    /// Returns every entity in the collection as tracked.
    async fn find_all(executor: &dyn DbExecutor) -> DbResult<Vec<DbEntity<Self>>> {
        let query = FindQuery::new(Self::collection_name());
        let mut cursor = executor.find(query).await?;

        let mut entities = Vec::new();
        while let Some(row) = cursor.next().await? {
            entities.push(DbEntity::from_db(row.as_ref()));
        }

        Ok(entities)
    }

    /// Returns every entity in the collection as detached (read-only).
    async fn find_all_readonly(executor: &dyn DbExecutor) -> DbResult<Vec<DbEntity<Self>>> {
        let query = FindQuery::new(Self::collection_name());
        let mut cursor = executor.find(query).await?;

        let mut entities = Vec::new();
        while let Some(row) = cursor.next().await? {
            entities.push(DbEntity::from_db_readonly(row.as_ref()));
        }

        Ok(entities)
    }
}

/// Represents the tracking state of an entity.
///
/// - **Untracked**: A new entity that hasn't been saved to the database yet
/// - **Tracked(original)**: An entity loaded from the database with its original value for change detection
/// - **Detached**: An entity that was loaded from the database but is no longer being tracked
#[derive(Debug, Clone)]
pub enum TrackingState<T> {
    Untracked,
    Tracked(T),
    Detached,
}

impl<T> TrackingState<T> {
    /// Returns `true` if the entity is being tracked.
    pub fn is_tracked(&self) -> bool {
        matches!(self, TrackingState::Tracked(_))
    }

    /// Returns `true` if the entity is untracked.
    pub fn is_untracked(&self) -> bool {
        matches!(self, TrackingState::Untracked)
    }

    /// Returns `true` if the entity is detached.
    pub fn is_detached(&self) -> bool {
        matches!(self, TrackingState::Detached)
    }

    /// Returns a reference to the original value if tracked.
    pub fn original(&self) -> Option<&T> {
        match self {
            TrackingState::Tracked(original) => Some(original),
            _ => None,
        }
    }
}

/// A flexible entity wrapper that can operate in tracked or untracked mode.
///
/// `DbEntity<T>` manages:
/// - The current state of the entity (`value`)
/// - The tracking state (untracked, tracked with original, or detached)
///
/// When untracked, no tracking overhead is incurred. Use this for:
/// - New entities that will be inserted
/// - Entities that don't need change detection
///
/// When tracked, the original value is stored for:
/// - Detecting which fields changed
/// - Optimized updates
///
/// When detached, no tracking occurs. Use this for:
/// - Read-only data returned to clients
/// - Entities loaded for display only
#[derive(Debug, Clone)]
pub struct DbEntity<T: DbEntityTrait> {
    /// The current state of the entity
    value: T,
    /// The tracking state with optional original value
    state: TrackingState<T>,
}

impl<T: DbEntityTrait> DbEntity<T> {
    // =========================================================================
    // CONSTRUCTORS
    // =========================================================================

    /// Creates a new untracked entity.
    ///
    /// Untracked entities are new and haven't been saved to the database yet.
    /// No tracking overhead is incurred.
    pub fn new(entity: T) -> Self {
        Self {
            value: entity,
            state: TrackingState::Untracked,
        }
    }

    /// Creates a tracked entity from a database row.
    ///
    /// Tracked entities exist in the database and are monitored for changes.
    /// The original value is stored for change detection.
    pub fn from_db(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity.clone(),
            state: TrackingState::Tracked(entity),
        }
    }

    /// Creates a detached (read-only) entity from a database row.
    ///
    /// Detached entities are loaded from the database but not tracked.
    /// Perfect for returning read-only data to clients.
    /// No change detection or update capability.
    pub fn from_db_readonly(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity,
            state: TrackingState::Detached,
        }
    }

    // =========================================================================
    // GETTERS
    // =========================================================================

    /// Returns a reference to the current entity value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the entity value.
    ///
    /// For untracked entities, this allows transformations without overhead.
    /// For tracked entities, changes are monitored.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consumes the `DbEntity` and returns the inner entity value.
    pub fn into_inner(self) -> T {
        self.value
    }

    // =========================================================================
    // STATUS
    // =========================================================================

    /// Returns a reference to the current tracking state.
    pub fn get_state(&self) -> &TrackingState<T> {
        &self.state
    }

    /// Returns `true` if the entity is being tracked.
    pub fn is_tracked(&self) -> bool {
        self.state.is_tracked()
    }

    /// Returns `true` if the entity is untracked.
    pub fn is_untracked(&self) -> bool {
        self.state.is_untracked()
    }

    /// Returns `true` if the entity is detached.
    pub fn is_detached(&self) -> bool {
        self.state.is_detached()
    }

    // =========================================================================
    // CRUD
    // =========================================================================

    /// Saves the entity to the database.
    ///
    /// - If tracked: Updates only the changed fields (excluding primary key)
    /// - If untracked: Inserts a new record (all fields from to_db())
    /// - If detached: Does nothing (detached entities cannot be saved)
    ///
    /// After successful save, the entity becomes tracked with the new original.
    pub async fn save(&mut self, executor: &dyn DbExecutor) -> DbResult<()>
    where
        T: PartialEq,
    {
        match &self.state {
            TrackingState::Untracked => {
                let fields = self.value.to_db();
                let row: Vec<(String, DbValue)> = fields
                    .into_iter()
                    .map(|(field, value)| (field.to_string(), value))
                    .collect();
                let insert_query = InsertQuery::new(T::collection_name()).insert(row);
                executor.insert(insert_query).await?;
                self.state = TrackingState::Tracked(self.value.clone());
                Ok(())
            }
            TrackingState::Tracked(original) => {
                let current_fields = self.value.to_db();
                let original_fields = original.to_db();
                let pk_names: Vec<&str> = self.value.primary_key()
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();

                let changed_fields: Vec<(String, DbValue)> = current_fields
                    .iter()
                    .zip(original_fields.iter())
                    .filter(|(current, orig)| !pk_names.contains(&current.0) && current.1 != orig.1)
                    .map(|(current, _)| (current.0.to_string(), current.1.clone()))
                    .collect();

                if !changed_fields.is_empty() {
                    let filter = self.value.primary_key_filter();
                    let mut update_query = UpdateQuery::new(T::collection_name()).filter(filter);
                    for (field, value) in changed_fields {
                        update_query = update_query.set(field, value);
                    }
                    executor.update(update_query).await?;
                }

                self.state = TrackingState::Tracked(self.value.clone());
                Ok(())
            }
            TrackingState::Detached => Ok(()),
        }
    }

    /// Deletes the entity from the database.
    ///
    /// Only tracked entities can be deleted. After deletion, the entity becomes detached.
    pub async fn delete(&mut self, executor: &dyn DbExecutor) -> DbResult<()> {
        if self.is_tracked() {
            use simple_db_core::query::DeleteQuery;
            let filter = self.value.primary_key_filter();
            let delete_query = DeleteQuery::new(T::collection_name()).filter(filter);
            executor.delete(delete_query).await?;
            self.state = TrackingState::Detached;
        }
        Ok(())
    }
}
