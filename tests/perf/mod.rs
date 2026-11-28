//! Budgets, baselines and reporting for the volume tests (FR-FND-07).

use std::fmt::Write as _;

use std::time::Duration;

use serde_json::{Value, json};

/// Regressions smaller than this many milliseconds are noise, whatever the ratio.
pub const NOISE_FLOOR_MS: f64 = 100.0;
/// A metric more than this fraction slower than its baseline fails (FR-FND-07).
pub const MAX_REGRESSION: f64 = 0.25;

/// Repetitions for the quick measurements; the slowest is reported (a p95 stand-in).
pub const REPEATS: usize = 5;

/// Milliseconds as a float.
pub fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000.0
}

/// Raw timings from one library size (warm start and rescan: slowest of [`REPEATS`]).
pub struct Measured {
    pub cold: Duration,
    pub first: Option<Duration>,
    pub warm: f64,
    pub rescan: f64,
    pub ten: Duration,
    pub rebuild: Duration,
    pub thumbs: Duration,
}

impl Measured {
    pub fn metrics(&self, count: usize) -> Vec<Metric> {
        let b = budgets(count).map_or([None; 6], |b| b.map(Some));
        let m = |key, label, ms, budget_ms, enforced| Metric {
            key,
            label,
            ms,
            budget_ms,
            enforced,
        };
        vec![
            m(
                "cold_scan",
                "Cold scan (walk, stat, EXIF)",
                ms(self.cold),
                b[0],
                true,
            ),
            m(
                "first_500_visible",
                "First 500 photos visible to another reader",
                self.first.map_or(f64::INFINITY, ms),
                b[4],
                true,
            ),
            m(
                "warm_start",
                "Warm start: open + full listing",
                self.warm,
                b[1],
                true,
            ),
            m(
                "rescan_no_change",
                "Rescan, no changes",
                self.rescan,
                b[2],
                true,
            ),
            m(
                "rescan_10_changed",
                "Rescan after 10 changed files",
                ms(self.ten),
                None,
                false,
            ),
            m(
                "rebuild",
                "Rebuild (delete catalog + full scan)",
                ms(self.rebuild),
                b[3],
                true,
            ),
            m(
                "thumbnails",
                "Thumbnails, whole library (256 px)",
                ms(self.thumbs),
                b[5],
                false,
            ),
        ]
    }
}

/// One measured number.
#[derive(Debug, Clone)]
pub struct Metric {
    /// Stable key, used in baselines.
    pub key: &'static str,
    /// What it measures, for the report.
    pub label: &'static str,
    /// Measured milliseconds (for repeated measurements: the slowest run).
    pub ms: f64,
    /// FR-FND-07 budget in milliseconds, if one applies at this size.
    pub budget_ms: Option<f64>,
    /// Whether exceeding the budget fails the test (false = informational).
    pub enforced: bool,
}

/// FR-FND-07 budgets (milliseconds) for `count` photos: cold scan, warm start,
/// no-change rescan, rebuild (1.2x cold scan, FR-FND-02), first 500 listable
/// (FR-LIB-09, stated for 10k; the same 2 s is applied at 1k), thumbnails
/// (informational).
pub fn budgets(count: usize) -> Option<[f64; 6]> {
    match count {
        1_000 => Some([5_000.0, 1_000.0, 1_000.0, 6_000.0, 2_000.0, 60_000.0]),
        10_000 => Some([50_000.0, 1_500.0, 5_000.0, 60_000.0, 2_000.0, 600_000.0]),
        _ => None,
    }
}

/// Compares `metrics` with the recorded baseline for `platform` and `count`.
/// Returns the failures (budget or regression) as messages.
pub fn check(metrics: &[Metric], baseline: Option<&Value>) -> Vec<String> {
    let mut failures = Vec::new();
    for m in metrics {
        if let (true, Some(budget)) = (m.enforced, m.budget_ms)
            && m.ms > budget
        {
            failures.push(format!(
                "{}: {:.0} ms is over the {:.0} ms budget",
                m.key, m.ms, budget
            ));
        }
        let Some(base) = baseline.and_then(|b| b.get(m.key)).and_then(Value::as_f64) else {
            continue;
        };
        if m.ms > base * (1.0 + MAX_REGRESSION) && m.ms - base > NOISE_FLOOR_MS {
            failures.push(format!(
                "{}: {:.0} ms is more than {:.0}% slower than the baseline {:.0} ms",
                m.key,
                m.ms,
                MAX_REGRESSION * 100.0,
                base
            ));
        }
    }
    failures
}

/// The baseline for `platform` and `count` from `perf/baselines.json`, if any.
pub fn baseline(file: &Value, platform: &str, count: usize) -> Option<Value> {
    file.get("platforms")?
        .get(platform)?
        .get(count.to_string())
        .cloned()
}

/// A Markdown table of the metrics (for logs and the CI job summary).
pub fn table(title: &str, metrics: &[Metric]) -> String {
    let mut out =
        format!("### {title}\n\n| Metric | Measured | Budget | Enforced |\n|---|---:|---:|---|\n");
    for m in metrics {
        let budget = m
            .budget_ms
            .map_or_else(|| "-".to_owned(), |b| format!("{b:.0} ms"));
        let enforced = if m.enforced { "yes" } else { "informational" };
        let _ = writeln!(
            out,
            "| {} | {:.0} ms | {budget} | {enforced} |",
            m.label, m.ms
        );
    }
    out
}

/// The machine-readable result document.
pub fn result_json(count: usize, extra: &Value, metrics: &[Metric]) -> Value {
    let values: serde_json::Map<String, Value> = metrics
        .iter()
        .map(|m| (m.key.to_owned(), json!((m.ms * 10.0).round() / 10.0)))
        .collect();
    json!({
        "count": count,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "cpus": std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        "environment": extra,
        "metrics_ms": values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric(ms: f64) -> Metric {
        Metric {
            key: "k",
            label: "k",
            ms,
            budget_ms: Some(1_000.0),
            enforced: true,
        }
    }

    #[test]
    fn regression_rule_has_a_noise_floor() {
        let base = json!({ "k": 100.0 });
        assert!(
            check(&[metric(190.0)], Some(&base)).is_empty(),
            "90% slower but only 90 ms"
        );
        assert_eq!(check(&[metric(260.0)], Some(&base)).len(), 1);
        assert_eq!(check(&[metric(1_500.0)], None).len(), 1, "over budget");
    }
}
