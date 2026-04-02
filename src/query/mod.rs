use crate::query::{
    db_values::DbValue, filters::FilterDefinition, groups::GroupDefinition, projections::ProjectionDefinition, sorts::SortDefinition
};

pub mod db_values;
pub mod projections;
pub mod filters;
pub mod sorts;
pub mod groups;

// ==========================================
// Find
// ==========================================
#[derive(Debug, Clone)]
pub struct FindQuery {
    pub collection: String,
    pub projections: ProjectionDefinition,
    pub filters: FilterDefinition,
    pub sorts: SortDefinition,
    pub groups: GroupDefinition,
    pub limit: Option<usize>,
    pub offset: Option<usize>
}

impl FindQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            projections: ProjectionDefinition::empty(),
            filters: FilterDefinition::empty(),
            sorts: SortDefinition::empty(),
            groups: GroupDefinition::empty(),
            limit: None,
            offset: None
        }
    }

    pub fn project(mut self, projections: ProjectionDefinition) -> Self {
        self.projections = projections;
        self
    }

    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters = filters;
        self
    }

    pub fn order_by(mut self, sorts: SortDefinition) -> Self {
        self.sorts = sorts;
        self
    }

    pub fn group_by(mut self, groups: GroupDefinition) -> Self {
        self.groups = groups;
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

// ==========================================
// Insert
// ==========================================
#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub collection: String,
    pub values: Vec<Vec<(String, DbValue)>>
}

impl InsertQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            values: Vec::new()
        }
    }

    pub fn insert<I, K, V>(mut self, row: I) -> Self 
    where 
        I: IntoIterator<Item = (K, V)>, 
        K: Into<String>, 
        V: Into<DbValue> 
    {
        self.values.push(
            row.into_iter()
               .map(|(k, v)| (k.into(), v.into()))
               .collect()
        );
        self
    }

    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self 
    where 
        I: IntoIterator<Item = R>, 
        R: IntoIterator<Item = (K, V)>, 
        K: Into<String>, 
        V: Into<DbValue> 
    {
        self.values.extend(
            rows.into_iter().map(|row| {
                row.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
            })
        );
        self
    }
}

// ==========================================
// Update
// ==========================================
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub collection: String,
    pub updates: Vec<(String, DbValue)>,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            updates: Vec::new(),
            filters: FilterDefinition::empty(),
        }
    }

    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.push((field.into(), value.into()));
        self
    }

    pub fn set_values<I, K, V>(mut self, values: I) -> Self 
    where 
        I: IntoIterator<Item = (K, V)>, 
        K: Into<String>, 
        V: Into<DbValue> 
    {
        self.updates.extend(
            values.into_iter().map(|(k, v)| (k.into(), v.into()))
        );
        self
    }

    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters = filters;
        self
    }
}

// ==========================================
// DELETE
// ==========================================
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub collection: String,
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            filters: FilterDefinition::empty(),
        }
    }

    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters = filters;
        self
    }
}

// ==========================================
// Query
// ==========================================
pub struct  Query;

impl Query {
    pub fn find<C: Into<String>>(collection: C) -> FindQuery {
        FindQuery::new(collection)
    }

    pub fn insert<C: Into<String>>(collection: C) -> InsertQuery {
        InsertQuery::new(collection)
    }

    pub fn update<C: Into<String>>(collection: C) -> UpdateQuery {
        UpdateQuery::new(collection)
    }

    pub fn delete<C: Into<String>>(collection: C) -> DeleteQuery {
        DeleteQuery::new(collection)
    }
}