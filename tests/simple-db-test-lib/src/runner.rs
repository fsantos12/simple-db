use crate::{
    bench,
    harness::TestHarness,
    report::{RunReport, TestCaseResult},
    suite,
};

/// Configuration for a [`TestRunner`] run.
pub struct RunnerConfig {
    /// Number of iterations per benchmark operation.
    pub runs: u32,
    /// Whether to execute the correctness test suite.
    pub run_tests: bool,
    /// Whether to execute the benchmark suite.
    pub run_bench: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self { runs: 100, run_tests: true, run_bench: true }
    }
}

impl RunnerConfig {
    /// Parses `--runs N`, `--bench-only`, and `--test-only` from `std::env::args`.
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut cfg = RunnerConfig::default();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--runs" => {
                    i += 1;
                    if let Some(n) = args.get(i).and_then(|s| s.parse::<u32>().ok()) {
                        cfg.runs = n;
                    }
                }
                "--bench-only" => {
                    cfg.run_tests = false;
                    cfg.run_bench = true;
                }
                "--test-only" => {
                    cfg.run_tests = true;
                    cfg.run_bench = false;
                }
                "--help" | "-h" => {
                    eprintln!("Usage: <driver-test> [--runs N] [--bench-only] [--test-only]");
                    eprintln!();
                    eprintln!("  --runs N       Benchmark iterations (default: 100)");
                    eprintln!("  --bench-only   Skip correctness tests");
                    eprintln!("  --test-only    Skip benchmarks");
                    std::process::exit(0);
                }
                _ => {}
            }
            i += 1;
        }
        cfg
    }
}

/// Orchestrates the test and benchmark suites for a single database driver.
pub struct TestRunner {
    harness: Box<dyn TestHarness>,
    config: RunnerConfig,
}

impl TestRunner {
    /// Creates a new runner with the given harness and configuration.
    pub fn new(harness: impl TestHarness + 'static, config: RunnerConfig) -> Self {
        Self { harness: Box::new(harness), config }
    }

    /// Runs tests and/or benchmarks according to the configuration and returns a [`RunReport`].
    pub async fn run(self) -> RunReport {
        let mut report = RunReport {
            driver_name: self.harness.driver_name().to_string(),
            runs: self.config.runs,
            test_results: vec![],
            bench_results: vec![],
        };

        if self.config.run_tests {
            report.test_results = self.run_tests().await;
        }
        if self.config.run_bench {
            report.bench_results = self.run_benchmarks().await;
        }
        report
    }

    async fn run_tests(&self) -> Vec<TestCaseResult> {
        macro_rules! test {
            ($name:literal, $func:path) => {{
                let db = self.harness.create_context().await;
                let start = std::time::Instant::now();
                let outcome: suite::TestResult = $func(&db).await;
                let elapsed = start.elapsed();
                TestCaseResult {
                    name: $name,
                    passed: outcome.is_ok(),
                    error: outcome.err(),
                    elapsed,
                }
            }};
        }

        vec![
            // Connectivity
            test!("test_ping", suite::test_ping),
            // Insert
            test!("test_insert_single", suite::test_insert_single),
            test!("test_insert_multiple_rows", suite::test_insert_multiple_rows),
            test!("test_insert_bulk", suite::test_insert_bulk),
            // Find – basic
            test!("test_find_all", suite::test_find_all),
            test!("test_find_with_projection", suite::test_find_with_projection),
            test!("test_find_with_limit", suite::test_find_with_limit),
            test!("test_find_with_offset", suite::test_find_with_offset),
            test!("test_find_with_sort_asc", suite::test_find_with_sort_asc),
            test!("test_find_with_sort_desc", suite::test_find_with_sort_desc),
            // Find – filters
            test!("test_find_filter_eq", suite::test_find_filter_eq),
            test!("test_find_filter_neq", suite::test_find_filter_neq),
            test!("test_find_filter_gt", suite::test_find_filter_gt),
            test!("test_find_filter_lt", suite::test_find_filter_lt),
            test!("test_find_filter_gte", suite::test_find_filter_gte),
            test!("test_find_filter_lte", suite::test_find_filter_lte),
            test!("test_find_filter_between", suite::test_find_filter_between),
            test!("test_find_filter_is_in", suite::test_find_filter_is_in),
            test!("test_find_filter_not_in", suite::test_find_filter_not_in),
            test!("test_find_filter_contains", suite::test_find_filter_contains),
            test!("test_find_filter_starts_with", suite::test_find_filter_starts_with),
            test!("test_find_filter_ends_with", suite::test_find_filter_ends_with),
            test!("test_find_filter_is_null", suite::test_find_filter_is_null),
            test!("test_find_filter_is_not_null", suite::test_find_filter_is_not_null),
            test!("test_find_filter_or", suite::test_find_filter_or),
            test!("test_find_filter_not", suite::test_find_filter_not),
            // Find – aggregations
            test!("test_find_count_all", suite::test_find_count_all),
            test!("test_find_sum", suite::test_find_sum),
            test!("test_find_avg", suite::test_find_avg),
            test!("test_find_min_max", suite::test_find_min_max),
            test!("test_find_group_by", suite::test_find_group_by),
            // Update
            test!("test_update_with_filter", suite::test_update_with_filter),
            test!("test_update_all", suite::test_update_all),
            // Delete
            test!("test_delete_with_filter", suite::test_delete_with_filter),
            test!("test_delete_all", suite::test_delete_all),
            // Transactions
            test!("test_transaction_commit", suite::test_transaction_commit),
            test!("test_transaction_rollback", suite::test_transaction_rollback),
        ]
    }

    async fn run_benchmarks(&self) -> Vec<crate::bench::BenchResult> {
        macro_rules! bench {
            ($func:path) => {{
                let db = self.harness.create_context().await;
                $func(&db, self.config.runs).await
            }};
        }

        vec![
            bench!(bench::bench_insert_single),
            bench!(bench::bench_insert_bulk_10),
            bench!(bench::bench_find_all),
            bench!(bench::bench_find_with_filter),
            bench!(bench::bench_update_with_filter),
            bench!(bench::bench_delete_with_filter),
            bench!(bench::bench_transaction_commit),
        ]
    }
}
