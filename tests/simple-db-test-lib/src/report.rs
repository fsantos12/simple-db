use std::time::Duration;

use crate::bench::BenchResult;

pub struct TestCaseResult {
    pub name: &'static str,
    pub passed: bool,
    pub error: Option<String>,
    pub elapsed: Duration,
}

pub struct RunReport {
    pub driver_name: String,
    pub runs: u32,
    pub test_results: Vec<TestCaseResult>,
    pub bench_results: Vec<BenchResult>,
}

impl RunReport {
    pub fn print(&self) {
        if !self.test_results.is_empty() {
            self.print_tests();
        }
        if !self.bench_results.is_empty() {
            self.print_bench();
        }
    }

    fn print_tests(&self) {
        println!("\n=== Test Suite ===");
        println!("Driver: {}\n", self.driver_name);

        let mut passed = 0usize;
        let mut failed = 0usize;

        for r in &self.test_results {
            let status = if r.passed { "PASS" } else { "FAIL" };
            let elapsed = fmt_duration(r.elapsed);
            if r.passed {
                println!("  {status}  {:<45} ({})", r.name, elapsed);
                passed += 1;
            } else {
                let err = r.error.as_deref().unwrap_or("unknown error");
                println!("  {status}  {:<45} ({})  →  {}", r.name, elapsed, err);
                failed += 1;
            }
        }

        let total = passed + failed;
        println!("\nPassed: {passed}/{total}   Failed: {failed}/{total}");
    }

    fn print_bench(&self) {
        println!("\n=== Benchmark Report ===");
        println!("Driver: {} | Runs: {}\n", self.driver_name, self.runs);

        let col_name = 36usize;
        let col_dur = 10usize;

        println!(
            "  {:<col_name$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$}",
            "Operation", "Total", "Min", "Avg", "Max", "P50", "P95", "P99",
            col_name = col_name, col_dur = col_dur,
        );
        let sep = format!(
            "  {:-<col_name$} {:-<col_dur$} {:-<col_dur$} {:-<col_dur$} {:-<col_dur$} {:-<col_dur$} {:-<col_dur$} {:-<col_dur$}",
            "", "", "", "", "", "", "", "",
            col_name = col_name, col_dur = col_dur,
        );
        println!("{sep}");

        for r in &self.bench_results {
            println!(
                "  {:<col_name$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$} {:>col_dur$}",
                r.name,
                fmt_duration(r.total),
                fmt_duration(r.min),
                fmt_duration(r.avg),
                fmt_duration(r.max),
                fmt_duration(r.p50),
                fmt_duration(r.p95),
                fmt_duration(r.p99),
                col_name = col_name, col_dur = col_dur,
            );
        }
        println!();
    }
}

pub fn fmt_duration(d: Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.1}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}
