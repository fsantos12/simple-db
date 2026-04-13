use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{builders::filters::Filter, types::DbValue};

pub struct FilterBuilder(SmallVec<[Filter; 8]>);

impl FilterBuilder {
    // --- Constructors ---
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    // --- add & extend ---
    pub fn add(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    pub fn extends<I>(mut self, filters: I) -> Self
    where I: IntoIterator<Item = Filter>,
    {
        self.0.extend(filters);
        self
    }

    // --- Null Checks ---
    pub fn is_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNull(field.into()))
    }

    pub fn is_not_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(field.into()))
    }

    // --- Basic Comparisons ---
    pub fn eq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(field.into(), value.into()))
    }

    pub fn neq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(field.into(), value.into()))
    }

    pub fn lt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(field.into(), value.into()))
    }

    pub fn lte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(field.into(), value.into()))
    }

    pub fn gt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(field.into(), value.into()))
    }

    pub fn gte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(field.into(), value.into()))
    }

    // --- Pattern Matching ---
    pub fn starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(field.into(), value.into()))
    }

    pub fn not_starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotStartsWith(field.into(), value.into()))
    }

    pub fn contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(field.into(), value.into()))
    }

    pub fn not_contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotContains(field.into(), value.into()))
    }

    pub fn ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(field.into(), value.into()))
    }

    pub fn not_ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotEndsWith(field.into(), value.into()))
    }

    pub fn regex<F: Into<SmolStr>, R: Into<SmolStr>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(field.into(), regex.into()))
    }

    // --- Range Checks ---
    pub fn between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(field.into(), (low.into(), high.into())))
    }

    pub fn not_between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(field.into(), (low.into(), high.into())))
    }

    // --- Set Membership ---
    pub fn is_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(field.into(), v_vec)) // Otimizado
    }

    pub fn not_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(field.into(), v_vec)) // Otimizado
    }

    // --- Logical Grouping (Closures) ---
    pub fn and<F>(self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let sub_builder = build(FilterBuilder::new());
        if sub_builder.0.is_empty() {
            self
        } else {
            self.add(Filter::And(sub_builder.0.into_vec()))
        }
    }

    pub fn or<F>(self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let sub_builder = build(FilterBuilder::new());
        if sub_builder.0.is_empty() {
            self
        } else {
            self.add(Filter::Or(sub_builder.0.into_vec()))
        }
    }

    pub fn not<F>(self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let mut filters = build(FilterBuilder::new()).0;
        match filters.len() {
            0 => self,
            1 => self.add(Filter::Not(Box::new(filters.pop().expect("exactly one filter")))),
            _ => self.add(Filter::Not(Box::new(Filter::And(filters.into_vec())))),
        }
    }

    // --- Finalization ---
    pub fn build(self) -> SmallVec<[Filter; 8]> {
        self.0
    }
}