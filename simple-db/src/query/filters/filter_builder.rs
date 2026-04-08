//! Fluent builder API for constructing filter conditions.
//!
//! `FilterBuilder` provides a fluent interface to define query conditions. Conditions
//! are organized into logical groups: null checks, comparisons, pattern matching,
//! range checks, set membership, and logical operators. Multiple conditions are
//! combined with implicit AND logic.

use crate::{query::filters::{Filter, FilterDefinition}, types::DbValue};

pub struct FilterBuilder {
    items: FilterDefinition,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_filters(mut self, filters: FilterDefinition) -> Self {
        self.items.extend(filters);
        self
    }

    fn add(mut self, filter: Filter) -> Self {
        self.items.push(filter);
        self
    }

    // --- Null Checks ---
    pub fn is_null<F: Into<String>>(self, field: F) -> Self {
        self.add(Filter::IsNull(Box::new(field.into())))
    }

    pub fn is_not_null<F: Into<String>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(Box::new(field.into())))
    }

    // --- Basic Comparisons ---
    pub fn eq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(Box::new(field.into()), value.into()))
    }

    pub fn neq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(Box::new(field.into()), value.into()))
    }

    pub fn lt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(Box::new(field.into()), value.into()))
    }

    pub fn lte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(Box::new(field.into()), value.into()))
    }

    pub fn gt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(Box::new(field.into()), value.into()))
    }

    pub fn gte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(Box::new(field.into()), value.into()))
    }

    // --- Pattern Matching ---
    pub fn starts_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(Box::new(field.into()), value.into()))
    }

    pub fn not_starts_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotStartsWith(Box::new(field.into()), value.into()))
    }

    pub fn contains<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(Box::new(field.into()), value.into()))
    }

    pub fn not_contains<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotContains(Box::new(field.into()), value.into()))
    }

    pub fn ends_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(Box::new(field.into()), value.into()))
    }

    pub fn not_ends_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotEndsWith(Box::new(field.into()), value.into()))
    }

    pub fn regex<F: Into<String>, R: Into<String>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(Box::new(field.into()), Box::new(regex.into())))
    }

    // --- Range Checks ---
    pub fn between<F: Into<String>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(
            Box::new(field.into()), 
            Box::new((low.into(), high.into()))
        ))
    }

    pub fn not_between<F: Into<String>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(
            Box::new(field.into()), 
            Box::new((low.into(), high.into()))
        ))
    }

    // --- Set Membership ---
    pub fn is_in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(Box::new(field.into()), Box::new(v_vec)))
    }

    pub fn not_in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(Box::new(field.into()), Box::new(v_vec)))
    }

    // --- Logical Grouping (Closures) ---
    pub fn and<F>(self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let sub_builder = build(FilterBuilder::new());
        if sub_builder.items.is_empty() {
            self
        } else {
            self.add(Filter::And(Box::new(sub_builder.items)))
        }
    }

    pub fn or<F>(self, build: F) -> Self 
    where F: FnOnce(FilterBuilder) -> FilterBuilder {
        let sub_builder = build(FilterBuilder::new());
        if sub_builder.items.is_empty() {
            self
        } else {
            self.add(Filter::Or(Box::new(sub_builder.items)))
        }
    }

    pub fn not(self, filter: Filter) -> Self {
        self.add(Filter::Not(Box::new(filter)))
    }

    // --- Finalization ---
    pub fn build(self) -> FilterDefinition {
        self.items
    }
}