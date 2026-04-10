use crate::types::{DbValue, TypeError, DbError};

pub trait DbRow {
    fn get_by_index(&self, index: usize) -> Option<&DbValue>;
    fn get_by_name(&self, name: &str) -> Option<&DbValue>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_by_index_as<'a, T>(&'a self, index: usize) -> Result<T, DbError>
    where T: TryFrom<&'a DbValue, Error = DbError> {
        let value = self.get_by_index(index).ok_or_else(|| TypeError::IndexOutOfBounds(index))?;
        T::try_from(value)
    }

    fn get_by_name_as<'a, T>(&'a self, name: &str) -> Result<T, DbError>
    where T: TryFrom<&'a DbValue, Error = DbError> {
        let value = self.get_by_name(name).ok_or_else(|| TypeError::ColumnMissing(name.to_string()))?;
        T::try_from(value)
    }
}