//! # simple-db-test-lib
//!
//! Shared test infrastructure for the simple-db driver integration tests.
//!
//! Provides a common test suite, benchmarks, runner, and reporting utilities
//! that every driver test binary links against.
//!
//! ## Modules
//!
//! - [`harness`] — [`TestHarness`] trait: implemented by each driver test app
//! - [`suite`] — full suite of correctness tests (CRUD, filters, aggregations, transactions)
//! - [`bench`] — latency benchmarks with percentile statistics
//! - [`runner`] — [`TestRunner`] wires harness + config + suite + bench together
//! - [`report`] — formatted console output for test and benchmark results

pub mod bench;
pub mod harness;
pub mod report;
pub mod runner;
pub mod suite;
