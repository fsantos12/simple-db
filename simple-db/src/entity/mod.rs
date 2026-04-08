//! Entity model management and change tracking.
//!
//! This module provides ORM abstractions for mapping database rows to Rust types
//! and tracking entity state changes (Added, Tracked, Modified, Deleted, Detached)
//! for unit-of-work patterns.

mod entity;

pub use entity::{DbEntityState, DbEntityKey, DbEntityModel, DbEntity};