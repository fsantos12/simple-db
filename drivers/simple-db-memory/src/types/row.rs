use std::{collections::HashMap, sync::Arc};

use simple_db_query::types::{DbRow, DbValue};

pub struct MemoryRow {
    pub column_mapping: Arc<HashMap<String, usize>>,
    pub values: Vec<DbValue>,
}

impl MemoryRow {
    pub fn insert(row: Vec<(String, DbValue)>) -> Self {

    }

    pub fn bulk_insert(rows: Vec<Vec<(String, DbValue)>>) -> Self {

    }
}

impl DbRow for MemoryRow {
    fn get_by_index(&self, index: usize) -> Option<&DbValue> {
        self.values.get(index)
    }

    fn get_by_name(&self, name: &str) -> Option<&DbValue> {
        self.column_mapping.get(name).and_then(|&index| self.get_by_index(index))
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}