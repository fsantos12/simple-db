use simple_db_core::{
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery},
    types::{DbCursor, DbError, DbResult, DbValue},
};
use sqlx::{query::Query, sqlite::SqliteArguments, Executor, Sqlite};

use crate::{
    queries::{compile_delete_query, compile_find_query, compile_insert_query, compile_update_query},
    types::SqliteDbCursor,
};

// ==========================================
// Shared execution functions
// ==========================================

/// Compiles and runs a SELECT query, returning a streaming cursor over the results.
pub(crate) async fn exec_find(executor: impl Executor<'_, Database = Sqlite>, query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
    let (sql, params) = compile_find_query(query);
    let rows = build_query(&sql, &params)
        .fetch_all(executor)
        .await
        .map_err(DbError::driver)?;
    let stream = futures::stream::iter(rows.into_iter().map(Ok));
    Ok(Box::new(SqliteDbCursor::new(Box::pin(stream))))
}

/// Compiles and runs an INSERT statement, returning the number of rows inserted.
pub(crate) async fn exec_insert(executor: impl Executor<'_, Database = Sqlite>, query: InsertQuery) -> DbResult<u64> {
    let (sql, params) = compile_insert_query(query);
    if sql.is_empty() {
        return Ok(0);
    }
    let result = build_query(&sql, &params)
        .execute(executor)
        .await
        .map_err(DbError::driver)?;
    Ok(result.rows_affected())
}

/// Compiles and runs an UPDATE statement, returning the number of rows affected.
pub(crate) async fn exec_update(executor: impl Executor<'_, Database = Sqlite>, query: UpdateQuery) -> DbResult<u64> {
    let (sql, params) = compile_update_query(query);
    if sql.is_empty() {
        return Ok(0);
    }
    let result = build_query(&sql, &params)
        .execute(executor)
        .await
        .map_err(DbError::driver)?;
    Ok(result.rows_affected())
}

/// Compiles and runs a DELETE statement, returning the number of rows deleted.
pub(crate) async fn exec_delete(executor: impl Executor<'_, Database = Sqlite>, query: DeleteQuery) -> DbResult<u64> {
    let (sql, params) = compile_delete_query(query);
    let result = build_query(&sql, &params)
        .execute(executor)
        .await
        .map_err(DbError::driver)?;
    Ok(result.rows_affected())
}

// ==========================================
// Query building
// ==========================================

/// Builds a parameterised sqlx query by binding each [`DbValue`] in order.
pub(crate) fn build_query<'q>(sql: &'q str, params: &[DbValue]) -> Query<'q, Sqlite, SqliteArguments<'q>> {
    let mut q = sqlx::query(sql);
    for param in params {
        q = bind_value(q, param);
    }
    q
}

/// Maps a single `DbValue` to the closest SQLite-native type and binds it.
///
/// SQLite type affinity rules applied:
/// - All integer types → INTEGER (i64)
/// - f32/f64 → REAL (f64)
/// - String, char, temporal, uuid, json, decimal → TEXT
/// - Vec<u8> → BLOB
/// - NULL → NULL
fn bind_value<'q>(q: Query<'q, Sqlite, SqliteArguments<'q>>, value: &DbValue) -> Query<'q, Sqlite, SqliteArguments<'q>> {
    if value.is_null()            { q.bind(None::<i64>) }
    else if value.is_bool()       { q.bind(value.as_bool().unwrap()) }
    else if value.is_i8()         { q.bind(value.as_i8().unwrap() as i64) }
    else if value.is_i16()        { q.bind(value.as_i16().unwrap() as i64) }
    else if value.is_i32()        { q.bind(value.as_i32().unwrap() as i64) }
    else if value.is_i64()        { q.bind(value.as_i64().unwrap()) }
    else if value.is_i128()       { q.bind(value.as_i128().unwrap().to_string()) }
    else if value.is_u8()         { q.bind(value.as_u8().unwrap() as i64) }
    else if value.is_u16()        { q.bind(value.as_u16().unwrap() as i64) }
    else if value.is_u32()        { q.bind(value.as_u32().unwrap() as i64) }
    else if value.is_u64()        { q.bind(value.as_u64().unwrap() as i64) }
    else if value.is_u128()       { q.bind(value.as_u128().unwrap().to_string()) }
    else if value.is_f32()        { q.bind(value.as_f32().unwrap() as f64) }
    else if value.is_f64()        { q.bind(value.as_f64().unwrap()) }
    else if value.is_decimal()    { q.bind(value.as_decimal().unwrap().to_string()) }
    else if value.is_char()       { q.bind(value.as_char().unwrap().to_string()) }
    else if value.is_string()     { q.bind(value.as_string().unwrap().to_owned()) }
    else if value.is_bytes()      { q.bind(value.as_bytes().unwrap().to_owned()) }
    else if value.is_uuid()       { q.bind(value.as_uuid().unwrap().to_string()) }
    else if value.is_json()       { q.bind(value.as_json().unwrap().to_string()) }
    else if value.is_date()       { q.bind(value.as_date().unwrap().to_string()) }
    else if value.is_time()       { q.bind(value.as_time().unwrap().to_string()) }
    else if value.is_timestamp()  { q.bind(value.as_timestamp().unwrap().to_string()) }
    else if value.is_timestampz() { q.bind(value.as_timestampz().unwrap().to_rfc3339()) }
    else                          { q.bind(None::<i64>) }
}
