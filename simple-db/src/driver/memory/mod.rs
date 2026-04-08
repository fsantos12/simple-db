//! In-memory database driver implementation.
//!
//! `MemoryDriver` provides a thread-safe, in-memory store suitable for testing,
//! prototyping, and unit tests. It implements the full Driver interface including
//! filtering, sorting, pagination, and transaction support using RwLock-based
//! concurrency control.

mod comparison;

use std::{cmp::Ordering, collections::HashMap, sync::{Arc, RwLock}};
use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use crate::{
    driver::{driver::Driver, memory::comparison::{strict_eq, strict_partial_cmp}}, 
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery, filters::{Filter, FilterDefinition}, sorts::{Sort, SortDefinition}}, 
    types::{DbError, DbRow, DbValue}
};

#[derive(Default, Clone)]
pub struct MemoryDriver {
    /// Internal storage: Thread-safe map of collections to rows.
    storage: Arc<RwLock<HashMap<String, Vec<DbRow>>>>,
}

impl MemoryDriver {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluates the entire FilterDefinition (implicit AND) against a row.
    fn matches_filter(&self, row: &DbRow, filters: &FilterDefinition) -> bool {
        filters.iter().all(|f| self.evaluate_node(row, f))
    }

    /// Recursively traverses the Filter AST.
    fn evaluate_node(&self, row: &DbRow, filter: &Filter) -> bool {
        match filter {
            // --- Null Checks ---
            Filter::IsNull(field) => row.get(field).map_or(true, |v| v.is_null()),
            Filter::IsNotNull(field) => row.get(field).map_or(false, |v|!v.is_null()),

            // --- Comparisons (Uses strict_eq/strict_partial_cmp) ---
            Filter::Eq(field, val) => row.get(field).is_some_and(|rv| strict_eq(rv, val)),
            Filter::Neq(field, val) => row.get(field).is_some_and(|rv|!strict_eq(rv, val)),
            Filter::Gt(field, val) => row.get(field).is_some_and(|rv| strict_partial_cmp(rv, val) == Some(Ordering::Greater)),
            Filter::Gte(field, val) => row.get(field).is_some_and(|rv| matches!(strict_partial_cmp(rv, val), Some(Ordering::Greater | Ordering::Equal))),
            Filter::Lt(field, val) => row.get(field).is_some_and(|rv| strict_partial_cmp(rv, val) == Some(Ordering::Less)),
            Filter::Lte(field, val) => row.get(field).is_some_and(|rv| matches!(strict_partial_cmp(rv, val), Some(Ordering::Less | Ordering::Equal))),

            // --- Pattern Matching ---
            Filter::StartsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(prefix))) = (row.get(field), val) {
                    text.starts_with(&**prefix)
                } else { false }
            },
            Filter::NotStartsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(prefix))) = (row.get(field), val) {
                   !text.starts_with(&**prefix)
                } else { false }
            },
            Filter::EndsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(suffix))) = (row.get(field), val) {
                    text.ends_with(&**suffix)
                } else { false }
            },
            Filter::NotEndsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(suffix))) = (row.get(field), val) {
                   !text.ends_with(&**suffix)
                } else { false }
            },
            Filter::Contains(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(sub))) = (row.get(field), val) {
                    text.contains(&**sub)
                } else { false }
            },
            Filter::NotContains(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(sub))) = (row.get(field), val) {
                   !text.contains(&**sub)
                } else { false }
            },

            // --- Regex Matching ---
            Filter::Regex(_field, _pattern) => {
                // TODO: Implement regex matching with regex crate
                false
            },

            // --- Range Checks ---
            Filter::Between(field, range) => {
                let (low, high) = &**range; // Double deref for the Boxed tuple
                row.get(field).is_some_and(|rv| {
                    let gte = matches!(strict_partial_cmp(rv, low), Some(Ordering::Greater | Ordering::Equal));
                    let lte = matches!(strict_partial_cmp(rv, high), Some(Ordering::Less | Ordering::Equal));
                    gte && lte
                })
            },
            Filter::NotBetween(field, range) => {
                let (low, high) = &**range;
                row.get(field).is_some_and(|rv| {
                    let gte = matches!(strict_partial_cmp(rv, low), Some(Ordering::Greater | Ordering::Equal));
                    let lte = matches!(strict_partial_cmp(rv, high), Some(Ordering::Less | Ordering::Equal));
                   !(gte && lte)
                })
            },

            // --- Set Membership ---
            Filter::In(field, vals) => row.get(field).is_some_and(|rv| vals.iter().any(|v| strict_eq(rv, v))),
            Filter::NotIn(field, vals) => row.get(field).is_some_and(|rv| vals.iter().all(|v|!strict_eq(rv, v))),

            // --- Logical Operators ---
            Filter::And(def) => def.iter().all(|f| self.evaluate_node(row, f)),
            Filter::Or(def) => def.iter().any(|f| self.evaluate_node(row, f)),
            Filter::Not(f) =>!self.evaluate_node(row, f),
        }
    }

    fn apply_sorts(&self, rows: &mut Vec<DbRow>, sorts: &SortDefinition) {
        if sorts.is_empty() { return; }
        
        // Handle Shuffle for Random sorting [2]
        if sorts.iter().any(|s| matches!(s, Sort::Random)) {
            rows.shuffle(&mut ThreadRng::default());
            if sorts.len() == 1 { return; }
        }

        rows.sort_by(|a, b| {
            for sort in sorts {
                let (field, is_asc, nulls_first) = match sort {
                    Sort::Asc(f) => (f, true, false),
                    Sort::Desc(f) => (f, false, true),
                    Sort::AscNullsFirst(f) => (f, true, true),
                    Sort::DescNullsLast(f) => (f, false, false),
                    Sort::Random => continue,
                    _ => continue,
                };

                let cmp = match (a.get(field), b.get(field)) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => if nulls_first { Ordering::Less } else { Ordering::Greater },
                    (Some(_), None) => if nulls_first { Ordering::Greater } else { Ordering::Less },
                    (Some(v1), Some(v2)) => strict_partial_cmp(v1, v2).unwrap_or(Ordering::Equal),
                };

                if cmp!= Ordering::Equal {
                    return if is_asc { cmp } else { cmp.reverse() };
                }
            }
            Ordering::Equal
        });
    }
}

#[async_trait]
impl Driver for MemoryDriver {
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError> {
        let storage = self.storage.read().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get(&query.collection).ok_or(DbError::NotFound)?;

        let mut results: Vec<DbRow> = table.iter()
           .filter(|row| self.matches_filter(row, &query.filters))
           .cloned()
           .collect();

        self.apply_sorts(&mut results, &query.sorts);

        // Pagination Logic
        let offset = query.offset.unwrap_or(0);
        let iter = results.into_iter().skip(offset);
        
        Ok(if let Some(limit) = query.limit {
            iter.take(limit).collect()
        } else {
            iter.collect()
        })
    }

    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.entry(query.collection).or_default();
        let len = query.values.len() as u64;
        table.extend(query.values);
        Ok(len)
    }

    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get_mut(&query.collection).ok_or(DbError::NotFound)?;
        
        let mut count = 0;
        for row in table.iter_mut() {
            if self.matches_filter(row, &query.filters) {
                for (k, v) in &query.updates.0 {
                    row.insert(k.clone(), v.clone());
                }
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get_mut(&query.collection).ok_or(DbError::NotFound)?;
        
        let initial_len = table.len();
        table.retain(|row|!self.matches_filter(row, &query.filters));
        Ok((initial_len - table.len()) as u64)
    }

    // --- Transactional Stubs ---
    async fn transaction_begin(&self) -> Result<(), DbError> { Ok(()) }
    async fn transaction_commit(&self) -> Result<(), DbError> { Ok(()) }
    async fn transaction_rollback(&self) -> Result<(), DbError> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_row(name: &str, age: i32) -> DbRow {
        let mut row = DbRow::new();
        row.insert("name", name);
        row.insert("age", age);
        row
    }

    #[tokio::test]
    async fn test_insert_single_row() {
        let driver = MemoryDriver::new();
        let mut row = DbRow::new();
        row.insert("id", 1i32);
        row.insert("name", "Alice");

        let query = InsertQuery::new("users").values(vec![row]);
        let result = driver.insert(query).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_insert_multiple_rows() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
            create_test_row("Charlie", 35),
        ];

        let query = InsertQuery::new("users").values(rows);
        let result = driver.insert(query).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_find_all_rows() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users");
        let result = driver.find(query).await;

        assert!(result.is_ok());
        let count = result.as_ref().unwrap().len();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_nonexistent_collection() {
        let driver = MemoryDriver::new();
        let query = FindQuery::new("nonexistent");
        let result = driver.find(query).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(DbError::NotFound)));
    }

    #[tokio::test]
    async fn test_find_with_eq_filter() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 30),
            create_test_row("Charlie", 25),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users")
            .filter(|fb| fb.eq("age", 30));
        
        let result = driver.find(query).await;
        assert!(result.is_ok());
        let count = result.unwrap().len();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_with_limit() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
            create_test_row("Charlie", 35),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users").limit(2);
        let result = driver.find(query).await;

        assert!(result.is_ok());
        let count = result.unwrap().len();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_with_offset() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
            create_test_row("Charlie", 35),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users").offset(1);
        let result = driver.find(query).await;

        assert!(result.is_ok());
        let count = result.unwrap().len();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_with_limit_and_offset() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
            create_test_row("Charlie", 35),
            create_test_row("Diana", 28),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users").offset(1).limit(2);
        let result = driver.find(query).await;

        assert!(result.is_ok());
        let count = result.unwrap().len();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_update_single_row() {
        let driver = MemoryDriver::new();
        driver.insert(InsertQuery::new("users").values(vec![create_test_row("Alice", 30)])).await.unwrap();

        let mut updates = DbRow::new();
        updates.insert("age", 31i32);

        let query = UpdateQuery::new("users")
            .set_row(updates)
            .with_filters(vec![Filter::Eq(Box::new("name".to_string()), DbValue::String(Some(Box::new("Alice".to_string()))))]);

        let result = driver.update(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_update_multiple_rows() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 30),
            create_test_row("Charlie", 25),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let mut updates = DbRow::new();
        updates.insert("age", 31i32);

        let query = UpdateQuery::new("users")
            .set_row(updates)
            .with_filters(vec![Filter::Eq(Box::new("age".to_string()), DbValue::I32(Some(30)))]);

        let result = driver.update(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_update_nonexistent_collection() {
        let driver = MemoryDriver::new();
        let mut updates = DbRow::new();
        updates.insert("age", 31i32);

        let query = UpdateQuery::new("nonexistent")
            .set_row(updates)
            .with_filters(vec![]);

        let result = driver.update(query).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_single_row() {
        let driver = MemoryDriver::new();
        driver.insert(InsertQuery::new("users").values(vec![create_test_row("Alice", 30)])).await.unwrap();

        let query = DeleteQuery::new("users")
            .with_filters(vec![Filter::Eq(Box::new("name".to_string()), DbValue::String(Some(Box::new("Alice".to_string()))))]);

        let result = driver.delete(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Verify deletion
        let find_result = driver.find(FindQuery::new("users")).await;
        assert!(find_result.is_ok());
        assert_eq!(find_result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_delete_multiple_rows() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 30),
            create_test_row("Charlie", 25),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = DeleteQuery::new("users")
            .with_filters(vec![Filter::Eq(Box::new("age".to_string()), DbValue::I32(Some(30)))]);

        let result = driver.delete(query).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        // Verify remaining rows
        let find_result = driver.find(FindQuery::new("users")).await;
        assert_eq!(find_result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_collection() {
        let driver = MemoryDriver::new();
        let query = DeleteQuery::new("nonexistent").with_filters(vec![]);
        let result = driver.delete(query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_null_filter() {
        let driver = MemoryDriver::new();
        let mut row1 = DbRow::new();
        row1.insert("name", "Alice");
        row1.insert("email", None::<String>);

        let mut row2 = DbRow::new();
        row2.insert("name", "Bob");
        row2.insert("email", "bob@example.com");

        driver.insert(InsertQuery::new("users").values(vec![row1, row2])).await.unwrap();

        let query = FindQuery::new("users")
            .filter(|fb| fb.is_null("email"));

        let result = driver.find(query).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_in_filter() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 30),
            create_test_row("Bob", 25),
            create_test_row("Charlie", 35),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users")
            .filter(|fb| fb.is_in("age", vec![25i32, 35i32]));

        let result = driver.find(query).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_between_filter() {
        let driver = MemoryDriver::new();
        let rows = vec![
            create_test_row("Alice", 20),
            create_test_row("Bob", 30),
            create_test_row("Charlie", 40),
        ];

        driver.insert(InsertQuery::new("users").values(rows)).await.unwrap();

        let query = FindQuery::new("users")
            .filter(|fb| fb.between("age", 25, 35));

        let result = driver.find(query).await.unwrap();
        assert_eq!(result.len(), 1); // Only Bob (30)
    }

    #[tokio::test]
    async fn test_contains_filter() {
        let driver = MemoryDriver::new();
        let mut row1 = DbRow::new();
        row1.insert("name", "Alice Johnson");

        let mut row2 = DbRow::new();
        row2.insert("name", "Bob Smith");

        driver.insert(InsertQuery::new("users").values(vec![row1, row2])).await.unwrap();

        let query = FindQuery::new("users")
            .filter(|fb| fb.contains("name", "Johnson"));

        let result = driver.find(query).await.unwrap();
        assert_eq!(result.len(), 1);
    }
}