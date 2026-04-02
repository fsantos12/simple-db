use std::{collections::HashMap, sync::{Arc, RwLock}};

use async_trait::async_trait;

use crate::{driver::driver::{DbResult, DbRow, Driver}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery, filters::{Filter, FilterDefinition}}, value::DbValue};

/// An In-Memory database driver that stores data in HashMaps and Vectors.
#[derive(Default, Clone)]
pub struct MemoryDriver {
    /// storage maps "collection_name" -> Vec<DbRow>
    storage: Arc<RwLock<HashMap<String, Vec<DbRow>>>>,
}

impl MemoryDriver {
    /// Creates a new instance of the MemoryDriver.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// A helper function to evaluate if a given row matches the provided filter definition.
    fn matches_filter(&self, row: &DbRow, filter: FilterDefinition) -> bool {
        if filter.is_empty() { return true; }

    }

    fn evaluate_filter(&self, row: &DbRow, filter: &Filter) -> bool {
        match filter {
            // --- Null Checks ---
            Filter::IsNull(field) => {
                value = row.get(field);
                
            },
            Filter::IsNotNull(field) => row.get(field).is_some(),

            // --- Basic Comparisons ---
            Filter::Eq(field, val) => row.get(field),
            Filter::Neq(field, val) => todo!(),
            Filter::Lt(field, val) => todo!(),
            Filter::Lte(field, val) => todo!(),
            Filter::Gt(field, val) => todo!(),
            Filter::Gte(field, val) => todo!(),

            // --- Pattern Matching ---
            Filter::Like(field, val) => todo!(),
            Filter::NotLike(field, val) => todo!(),

            // --- Regex Matching ---
            Filter::Regex(field, val) => todo!(),

            // --- Range Checks ---
            Filter::Between(field, val1, val2) => todo!(),
            Filter::NotBetween(field, val1, val2) => todo!(),

            // --- Set Membership ---
            Filter::In(field, vals) => todo!(),
            Filter::NotIn(field_, vals) => todo!(),

            // --- Logical Operators ---
            Filter::And(filters) => todo!(),
            Filter::Or(filters) => todo!(),
            Filter::Not(filter) => todo!(),
        }
    }

    fn is_value_null(value: Option<&DbValue>) -> bool {
        match value {
            None => true,
            Some(val) => val.is_null(),
        }
    }
}

#[async_trait]
impl Driver for MemoryDriver {
    async fn find(&self, query: FindQuery) -> DbResult<Vec<DbRow>> {
        // Todo: Implementation
        Ok(Vec::new())
    }

    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        // Todo: Implementation
        Ok(0)
    }

    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        // Todo: Implementation
        Ok(0)
    }

    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        // Todo: Implementation
        Ok(0)
    }
}