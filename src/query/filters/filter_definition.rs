use crate::{query::filters::filter::Filter, types::DbValue};

#[derive(Debug, Clone, Default)]
pub struct FilterDefinition(Vec<Filter>);

impl FilterDefinition {
    // Contructors
    pub fn new(filters: Vec<Filter>) -> Self {
        Self(filters)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    // Vec like methods
    pub fn push(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    pub fn append(mut self, other: &mut FilterDefinition) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn pop(&mut self) -> Option<Filter> {
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

    // --- Null Checks ---
    pub fn is_null<F: Into<String>>(self, field: F) -> Self {
        self.push(Filter::IsNull(field.into()))
    }

    pub fn is_not_null<F: Into<String>>(self, field: F) -> Self {
        self.push(Filter::IsNotNull(field.into()))
    }

    // --- Basic Comparisons ---
    pub fn eq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Eq(field.into(), value.into()))
    }

    pub fn neq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Neq(field.into(), value.into()))
    }

    pub fn lt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Lt(field.into(), value.into()))
    }

    pub fn lte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Lte(field.into(), value.into()))
    }

    pub fn gt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Gt(field.into(), value.into()))
    }

    pub fn gte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Gte(field.into(), value.into()))
    }

    // --- Pattern Matching ---
    pub fn starts_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::StartsWith(field.into(), value.into()))
    }

    pub fn not_starts_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::NotStartsWith(field.into(), value.into()))
    }

    pub fn ends_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::EndsWith(field.into(), value.into()))
    }

    pub fn not_ends_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::NotEndsWith(field.into(), value.into()))
    }

    pub fn contains<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::Contains(field.into(), value.into()))
    }

    pub fn not_contains<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.push(Filter::NotContains(field.into(), value.into()))
    }

    // --- Regex Matching ---
    pub fn regex<F: Into<String>, R: Into<String>>(self, field: F, regex: R) -> Self {
        self.push(Filter::Regex(field.into(), regex.into()))
    }

    // --- Range Checks ---
    pub fn between<F: Into<String>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.push(Filter::Between(field.into(), low.into(), high.into()))
    }

    pub fn not_between<F: Into<String>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.push(Filter::NotBetween(field.into(), low.into(), high.into()))
    }

    // --- Set Membership ---
    pub fn r#in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        self.push(Filter::In(
            field.into(),
            values.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn not_in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        self.push(Filter::NotIn(
            field.into(),
            values.into_iter().map(Into::into).collect(),
        ))
    }

    // --- Logical Operators ---
    pub fn and(self, filters: Vec<Filter>) -> Self {
        self.push(Filter::And(filters))
    }

    pub fn or(self, filters: Vec<Filter>) -> Self {
        self.push(Filter::Or(filters))
    }

    pub fn not(self, filter: Filter) -> Self {
        self.push(Filter::Not(Box::new(filter)))
    }
}

impl From<Vec<Filter>> for FilterDefinition {
    fn from(v: Vec<Filter>) -> Self { Self(v) }
}

impl From<FilterDefinition> for Vec<Filter> {
    fn from(d: FilterDefinition) -> Self { d.0 }
}

impl IntoIterator for FilterDefinition {
    type Item = Filter;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a FilterDefinition {
    type Item = &'a Filter;
    type IntoIter = std::slice::Iter<'a, Filter>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<Filter> for FilterDefinition {
    fn extend<T: IntoIterator<Item = Filter>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}