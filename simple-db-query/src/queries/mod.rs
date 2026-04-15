//! **Query Module**
//!
//! This module provides the main entry point [`Query`] and four CRUD query types:
//! - [`FindQuery`]: SELECT queries with filtering, sorting, grouping, and pagination
//! - [`InsertQuery`]: INSERT queries with single or bulk row insertion
//! - [`UpdateQuery`]: UPDATE queries with column updates and WHERE filters
//! - [`DeleteQuery`]: DELETE queries with WHERE filters
//!
//! All queries use a builder pattern for ergonomic, type-safe construction.

mod find;
mod insert;
mod update;
mod delete;

pub use find::FindQuery;
pub use insert::InsertQuery;
pub use update::UpdateQuery;
pub use delete::DeleteQuery;

// ==========================================
// Query Builder Entry Point
// ==========================================

/// Static entry point for building queries.
///
/// All four CRUD operations are available as associated functions that return
/// their respective builder. The builders are then used to add projections,
/// filters, sorts, and other clauses via method chaining.
///
/// # Examples
///
/// ```rust
/// use simple_db_query::Query;
///
/// // SELECT query
/// let q = Query::find("users")
///     .project(|b| b.field("name").field("email"))
///     .filter(|b| b.gt("age", 18i32))
///     .order_by(|b| b.asc("name"))
///     .limit(10);
///
/// // INSERT query
/// let q = Query::insert("users")
///     .insert(vec![("name", "Alice"), ("email", "alice@example.com")]);
///
/// // UPDATE query
/// let q = Query::update("users")
///     .set("email", "newemail@example.com")
///     .filter(|b| b.eq("id", 1i32));
///
/// // DELETE query
/// let q = Query::delete("users")
///     .filter(|b| b.eq("id", 1i32));
/// ```
pub struct Query;

impl Query {
    /// Creates a new [`FindQuery`] (SELECT) targeting `collection`.
    pub fn find<C: Into<String>>(collection: C) -> FindQuery {
        FindQuery::new(collection)
    }

    /// Creates a new [`InsertQuery`] (INSERT) targeting `collection`.
    pub fn insert<C: Into<String>>(collection: C) -> InsertQuery {
        InsertQuery::new(collection)
    }

    /// Creates a new [`UpdateQuery`] (UPDATE) targeting `collection`.
    pub fn update<C: Into<String>>(collection: C) -> UpdateQuery {
        UpdateQuery::new(collection)
    }

    /// Creates a new [`DeleteQuery`] (DELETE) targeting `collection`.
    pub fn delete<C: Into<String>>(collection: C) -> DeleteQuery {
        DeleteQuery::new(collection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_query_creation() {
        // Test basic FindQuery creation
        let query = Query::find("users");
        assert_eq!(query.collection, "users");
        assert_eq!(query.projections.len(), 0);
        assert_eq!(query.filters.len(), 0);
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_find_query_with_single_projection() {
        // Test selecting a single field
        let query = Query::find("users")
            .project(|b| b.field("name"));
        
        assert_eq!(query.collection, "users");
        assert!(!query.projections.is_empty());
    }

    #[test]
    fn test_find_query_with_multiple_projections() {
        // Test selecting multiple fields
        let query = Query::find("users")
            .project(|b| b
                .field("id")
                .field("name")
                .field("email")
            );
        
        assert_eq!(query.projections.len(), 3);
    }

    #[test]
    fn test_find_query_with_filter() {
        // Test adding filter conditions
        let query = Query::find("users")
            .filter(|b| b.eq("age", 25));
        
        assert!(!query.filters.is_empty());
    }

    #[test]
    fn test_find_query_with_multiple_filters() {
        // Test multiple filter conditions (implicit AND)
        let query = Query::find("users")
            .filter(|b| b.eq("age", 25))
            .filter(|b| b.eq("active", true));
        
        assert_eq!(query.filters.len(), 2);
    }

    #[test]
    fn test_find_query_with_limit_offset() {
        // Test pagination with LIMIT and OFFSET
        let query = Query::find("users")
            .limit(10)
            .offset(20);
        
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_find_query_with_order_by() {
        // Test sorting results
        let query = Query::find("users")
            .order_by(|b| b.asc("name"));
        
        assert!(!query.sorts.is_empty());
    }

    #[test]
    fn test_find_query_complex_composition() {
        // Test building a complex query with all components
        let query = Query::find("users")
            .project(|b| b
                .field("id")
                .field("name")
                .field("email")
            )
            .filter(|b| b.gt("age", 18))
            .filter(|b| b.eq("active", true))
            .order_by(|b| b.desc("created_at"))
            .limit(50)
            .offset(0);
        
        assert_eq!(query.projections.len(), 3);
        assert_eq!(query.filters.len(), 2);
        assert!(!query.sorts.is_empty());
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_insert_query_creation() {
        // Test creating a new InsertQuery
        let query = Query::insert("users");
        assert_eq!(query.collection, "users");
    }

    #[test]
    fn test_insert_query_with_values() {
        // Test inserting a single row
        let row = vec![
            ("name", "Alice"),
            ("email", "alice@example.com"),
        ];
        let query = Query::insert("users")
            .insert(row);
        
        assert_eq!(query.values.len(), 1);
    }

    #[test]
    fn test_bulk_insert_query() {
        // Test batch inserting multiple rows
        let row1 = vec![
            ("name", "Alice"),
            ("email", "alice@example.com"),
        ];
        let row2 = vec![
            ("name", "Bob"),
            ("email", "bob@example.com"),
        ];
        
        let query = Query::insert("users")
            .insert(row1)
            .insert(row2);
        
        assert_eq!(query.values.len(), 2);
    }

    #[test]
    fn test_update_query_creation() {
        // Test creating a new UpdateQuery
        let query = Query::update("users");
        assert_eq!(query.collection, "users");
    }

    #[test]
    fn test_update_query_with_filter() {
        // Test updating with WHERE conditions
        let query = Query::update("users")
            .set("email", "newemail@example.com")
            .filter(|b| b.eq("id", 1));
        
        assert!(!query.updates.is_empty());
        assert!(!query.filters.is_empty());
    }

    #[test]
    fn test_update_query_multiple_fields() {
        // Test updating multiple fields
        let query = Query::update("users")
            .set("email", "new@example.com")
            .set("updated_at", "2024-04-13");
        
        assert_eq!(query.updates.len(), 2);
    }

    #[test]
    fn test_delete_query_creation() {
        // Test creating a new DeleteQuery
        let query = Query::delete("users");
        assert_eq!(query.collection, "users");
    }

    #[test]
    fn test_delete_query_with_filter() {
        // Test deleting with WHERE conditions
        let query = Query::delete("users")
            .filter(|b| b.eq("id", 1));
        
        assert!(!query.filters.is_empty());
    }

    #[test]
    fn test_delete_query_with_multiple_filters() {
        // Test deleting with multiple conditions
        let query = Query::delete("users")
            .filter(|b| b.lt("age", 18))
            .filter(|b| b.eq("archived", true));
        
        assert_eq!(query.filters.len(), 2);
    }
}