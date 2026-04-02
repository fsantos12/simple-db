use crate::query::sorts::sort::Sort;

#[derive(Debug, Clone, Default)]
pub struct SortDefinition(Vec<Sort>);

impl SortDefinition {
    // Contructors
    pub fn new(sorts: Vec<Sort>) -> Self {
        Self(sorts)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    // Vec like methods
    pub fn push(mut self, sort: Sort) -> Self {
        self.0.push(sort);
        self
    }

    pub fn append(mut self, other: &mut SortDefinition) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn pop(&mut self) -> Option<Sort> {
        self.0.pop()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    // --- Basic ---
    pub fn asc<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::Asc(field.into()))
    }

    pub fn desc<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::Desc(field.into()))
    }

    // --- Null Handling ---
    pub fn asc_nulls_first<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::AscNullsFirst(field.into()))
    }

    pub fn asc_nulls_last<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::AscNullsLast(field.into()))
    }

    pub fn desc_nulls_first<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::DescNullsFirst(field.into()))
    }

    pub fn desc_nulls_last<F: Into<String>>(self, field: F) -> Self {
        self.push(Sort::DescNullsLast(field.into()))
    }

    // --- Special Cases ---
    pub fn random(self) -> Self {
        self.push(Sort::Random)
    }
}


impl From<Vec<Sort>> for SortDefinition {
    fn from(v: Vec<Sort>) -> Self { Self(v) }
}

impl From<SortDefinition> for Vec<Sort> {
    fn from(d: SortDefinition) -> Self { d.0 }
}

impl IntoIterator for SortDefinition {
    type Item = Sort;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a SortDefinition {
    type Item = &'a Sort;
    type IntoIter = std::slice::Iter<'a, Sort>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<Sort> for SortDefinition {
    fn extend<T: IntoIterator<Item = Sort>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}