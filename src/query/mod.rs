use crate::{query::{
    filters::FilterDefinition, groups::GroupDefinition, projections::ProjectionDefinition, sorts::SortDefinition
}, types::{DbRow, DbValue}};

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
    pub values: Vec<DbRow>
}

impl InsertQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            values: Vec::new()
        }
    }

    pub fn add_row(mut self, row: DbRow) -> Self {
        self.values.push(row);
        self
    }

    pub fn add_rows<I>(mut self, rows: I) -> Self 
    where I: IntoIterator<Item = DbRow> {
        self.values.extend(rows);
        self
    }

    pub fn insert<I, K, V>(self, row: I) -> Self
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let db_row: DbRow = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.add_row(db_row)
    }

    pub fn bulk_insert<I, R, K, V>(self, rows: I) -> Self
    where I: IntoIterator<Item = R>, R: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let prepared_rows = rows.into_iter().map(|row| {
            row.into_iter().map(|(k, v)| (k.into(), v.into())).collect::<DbRow>()
        });
        self.add_rows(prepared_rows)
    }
}

// ==========================================
// Update
// ==========================================
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub collection: String,
    pub updates: DbRow,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    pub fn new(collection: impl Into<String>) -> Self {
        Self {
            collection: collection.into(),
            updates: DbRow::new(),
            filters: FilterDefinition::empty(),
        }
    }

    pub fn set_row(mut self, row: DbRow) -> Self {
        self.updates.0.extend(row.0);
        self
    }

    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.insert(field, value);
        self
    }

    pub fn set_values<I, K, V>(mut self, values: I) -> Self 
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        for (k, v) in values {
            self.updates.insert(k, v);
        }
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