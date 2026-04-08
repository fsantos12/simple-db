//! Fluent builder API for constructing filter conditions.
//!
//! `FilterBuilder` provides a fluent interface to define query conditions. Conditions
//! are organized into logical groups: null checks, comparisons, pattern matching,
//! range checks, set membership, and logical operators. Multiple conditions are
//! combined with implicit AND logic.

use crate::{query::filters::{Filter, FilterDefinition}, types::DbValue};
use smol_str::SmolStr;

pub struct FilterBuilder {
    items: FilterDefinition,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self { items: FilterDefinition::new() }
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
        self.add(Filter::Between(
            field.into(),
            Box::new((low.into(), high.into()))
        ))
    }

    pub fn not_between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(
            field.into(),
            Box::new((low.into(), high.into()))
        ))
    }

    // --- Set Membership ---
    pub fn is_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(field.into(), Box::new(v_vec)))
    }

    pub fn not_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let v_vec: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(field.into(), Box::new(v_vec)))
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