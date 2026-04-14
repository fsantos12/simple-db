use std::vec::IntoIter;

use async_trait::async_trait;
use simple_db_driver::types::DbCursor;
use simple_db_query::types::{DbError, DbRow};

pub struct MemoryCursor {
    // Usamos o IntoIter do próprio Rust. 
    // Ele consome o Vec, deitando fora os elementos à medida que avança.
    iterator: IntoIter<Box<dyn DbRow>>,
}

impl MemoryCursor {
    pub fn new(rows: Vec<Box<dyn DbRow>>) -> Self {
        Self {
            iterator: rows.into_iter(),
        }
    }
}

#[async_trait]
impl DbCursor for MemoryCursor {
    async fn next(&mut self) -> Result<Option<Box<dyn DbRow>>, DbError> {
        // O método .next() do IntoIter devolve Some(linha) ou None.
        // Envolvemos num Ok() porque a nossa trait exige um Result.
        Ok(self.iterator.next())
    }
}