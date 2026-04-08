//! Database driver implementations and abstraction layer.
//!
//! This module provides the driver interface for pluggable database backends
//! and includes a memory driver implementation for testing and prototyping.

pub mod driver;
pub mod memory;