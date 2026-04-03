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
    storage: Arc<RwLock<HashMap<String, Vec<DbRow>>>>,
}

impl MemoryDriver {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn matches_filter(&self, row: &DbRow, filter: FilterDefinition) -> bool {
        if filter.is_empty() { return true; }
        filter.into_iter().all(|f| self.evaluate_filter(row, &f))
    }

    fn evaluate_filter(&self, row: &DbRow, filter: &Filter) -> bool {
        match filter {
            Filter::IsNull(field) => row.get(field).is_none_or(|row_val| row_val.is_null()),
            Filter::IsNotNull(field) => row.get(field).is_some_and(|row_val| !row_val.is_null()),
            Filter::Eq(field, val) => row.get(field).is_some_and(|row_val| strict_eq(row_val, val)),
            Filter::Neq(field, val) => row.get(field).is_some_and(|row_val| !strict_eq(row_val, val)),
            Filter::Lt(field, val) => row.get(field).is_some_and(|row_val| strict_partial_cmp(row_val, val) == Some(Ordering::Less)),
            Filter::Lte(field, val) => row.get(field).is_some_and(|row_val| matches!(strict_partial_cmp(row_val, val), Some(Ordering::Less | Ordering::Equal))),
            Filter::Gt(field, val) => row.get(field).is_some_and(|row_val| strict_partial_cmp(row_val, val) == Some(Ordering::Greater)),
            Filter::Gte(field, val) => row.get(field).is_some_and(|row_val| matches!(strict_partial_cmp(row_val, val), Some(Ordering::Greater | Ordering::Equal))),
            Filter::StartsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(prefix))) = (row.get(field), val) {
                    text.starts_with(prefix)
                } else { false }
            },
            Filter::NotStartsWith(field, val) => !self.evaluate_filter(row, &Filter::StartsWith(field.clone(), val.clone())),
            Filter::EndsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(suffix))) = (row.get(field), val) {
                    text.ends_with(suffix)
                } else { false }
            },
            Filter::NotEndsWith(field, val) => !self.evaluate_filter(row, &Filter::EndsWith(field.clone(), val.clone())),
            Filter::Contains(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(substr))) = (row.get(field), val) {
                    text.contains(substr)
                } else { false }
            },
            Filter::NotContains(field, val) => !self.evaluate_filter(row, &Filter::Contains(field.clone(), val.clone())),
            Filter::Regex(field, val) => {
                if let Some(DbValue::String(Some(text))) = row.get(field) {
                    regex::Regex::new(val).is_ok_and(|re| re.is_match(text))
                } else { false }
            },
            Filter::Between(field, val1, val2) => row.get(field).is_some_and(|row_val| {
                let gte_val1 = matches!(strict_partial_cmp(row_val, val1), Some(Ordering::Greater | Ordering::Equal));
                let lte_val2 = matches!(strict_partial_cmp(row_val, val2), Some(Ordering::Less | Ordering::Equal));
                gte_val1 && lte_val2
            }),
            Filter::NotBetween(field, val1, val2) => !self.evaluate_filter(row, &Filter::Between(field.clone(), val1.clone(), val2.clone())),
            Filter::In(field, vals) => row.get(field).is_some_and(|row_val| {
                vals.iter().any(|v| strict_eq(row_val, v))
            }),
            Filter::NotIn(field, vals) => !self.evaluate_filter(row, &Filter::In(field.clone(), vals.clone())),
            Filter::And(filters) => filters.iter().all(|f| self.evaluate_filter(row, f)),
            Filter::Or(filters) => filters.iter().any(|f| self.evaluate_filter(row, f)),
            Filter::Not(filter) => !self.evaluate_filter(row, filter),
        }
    }

    fn apply_sorts(&self, rows: &mut [DbRow], sorts: &SortDefinition) {
        if sorts.is_empty() { return; }
        if sorts.into_iter().any(|s| matches!(s, Sort::Random)) {
            let mut rng = ThreadRng::default();
            rows.shuffle(&mut rng);
            if sorts.len() == 1 { return; }
        }
        rows.sort_by(|row_a, row_b| {
            for sort in sorts.into_iter() {
                let (field, is_asc, nulls_first) = match sort {
                    Sort::Asc(f) => (f, true, false),
                    Sort::Desc(f) => (f, false, true),
                    Sort::AscNullsFirst(f) => (f, true, true),
                    Sort::AscNullsLast(f) => (f, true, false),
                    Sort::DescNullsFirst(f) => (f, false, true),
                    Sort::DescNullsLast(f) => (f, false, false),
                    Sort::Random => continue, 
                };
                let val_a = row_a.get(field);
                let val_b = row_b.get(field);
                let cmp = match (val_a, val_b) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => if nulls_first { Ordering::Less } else { Ordering::Greater },
                    (Some(_), None) => if nulls_first { Ordering::Greater } else { Ordering::Less },
                    (Some(a), Some(b)) => strict_partial_cmp(a, b).unwrap_or(Ordering::Equal),
                };
                if cmp != Ordering::Equal {
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
        let storage = self.storage.read().unwrap();
        let Some(table) = storage.get(&query.collection) else {
            return Ok(Vec::new()); 
        };
        let mut results: Vec<DbRow> = table.iter()
            .filter(|row| self.matches_filter(row, query.filters.clone()))
            .cloned().collect();
        self.apply_sorts(&mut results, &query.sorts);
        let offset = query.offset.unwrap_or(0);
        let iter = results.into_iter().skip(offset);
        let paginated_results = if let Some(limit) = query.limit {
            iter.take(limit).collect()
        } else { iter.collect() };
        Ok(paginated_results)
    }

    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().unwrap();
        let table = storage.entry(query.collection).or_insert_with(Vec::new);
        let count = query.values.len() as u64;
        table.extend(query.values);
        Ok(count)
    }

    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().unwrap();
        let Some(table) = storage.get_mut(&query.collection) else {
            return Ok(0);
        };
        let mut updated_count = 0;
        for row in table.iter_mut() {
            if self.matches_filter(row, query.filters.clone()) {
                for (field, new_value) in &query.updates.0 {
                    row.insert(field.clone(), new_value.clone());
                }
                updated_count += 1;
            }
        }
        Ok(updated_count)
    }

    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().unwrap();
        let Some(table) = storage.get_mut(&query.collection) else {
            return Ok(0);
        };
        let initial_len = table.len();
        table.retain(|row| !self.matches_filter(row, query.filters.clone()));
        Ok((initial_len - table.len()) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_memory_driver_full_lifecycle() {
        let driver = MemoryDriver::new();
        let col = "users";

        let insert_start = Instant::now();
        driver.insert(InsertQuery::new(col)
            .insert([("id", DbValue::from(1)), ("name", DbValue::from("Alice")), ("age", DbValue::from(30)), ("status", DbValue::from("active"))])
            .insert([("id", DbValue::from(2)), ("name", DbValue::from("Bob")), ("age", DbValue::from(25)), ("status", DbValue::from("inactive"))])
            .insert([("id", DbValue::from(3)), ("name", DbValue::from("Charlie")), ("age", DbValue::from(35)), ("status", DbValue::from("active"))])
            .insert([("id", DbValue::from(4)), ("name", DbValue::from("David")), ("age", DbValue::from(28)), ("status", DbValue::from("active"))])
        ).await.unwrap();
        println!("Insert took: {:?}", insert_start.elapsed());

        let find_query = FindQuery::new(col)
            .filter(FilterDefinition::empty().eq("status", "active").gt("age", 27))
            .order_by(SortDefinition::empty().desc("age"));

        let results = driver.find(find_query).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(matches!(results[0].get("name"), Some(DbValue::String(Some(s))) if s == "Charlie"));

        let update_query = UpdateQuery::new(col)
            .filter(FilterDefinition::empty().eq("status", "active"))
            .set("status", "premium");

        let updated = driver.update(update_query).await.unwrap();
        assert_eq!(updated, 3);

        let delete_query = DeleteQuery::new(col).filter(FilterDefinition::empty().lte("age", 26));
        let deleted = driver.delete(delete_query).await.unwrap();
        assert_eq!(deleted, 1);
    }

    #[tokio::test]
    async fn test_sort_random_performance() {
        let driver = MemoryDriver::new();
        let col = "large_set";

        let mut query = InsertQuery::new(col);
        for i in 0..1000 {
            query = query.insert([("id", i), ("val", i * 2)]);
        }

        driver.insert(query).await.unwrap();

        let rand_query = FindQuery::new(col).order_by(SortDefinition::empty().random());
        let start = Instant::now();
        let _ = driver.find(rand_query).await.unwrap();
        println!("Random shuffle (1000 rows) took: {:?}", start.elapsed());
    }
}