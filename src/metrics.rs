//! Combinatorial Maximalism: Prometheus Metrics & SLO Enforcement.
//!
//! This module implements the Features 4.2 (Metrics) and 4.5 (SLO) of the
//! affidavit observability suite. It provides a `MetricsCollector` capable of
//! recording throughput, latency, and error rates, computing SLIs over a
//! rolling window, and enforcing SLO thresholds.
//!
//! Includes a custom Prometheus text format exporter for maximalist portability.

use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow;
use thiserror::Error;

// --- SLO Constants (AC-5.3) ---

/// 99th percentile stage latency threshold (milliseconds).
pub const SLO_LATENCY_P99_MS: f64 = 100.0;
/// Maximum allowed error rate percentage.
pub const SLO_ERROR_RATE_PCT: f64 = 0.1;
/// Minimum required availability percentage.
pub const SLO_AVAILABILITY_PCT: f64 = 99.9;

// --- Types ---

/// Service Level Indicators computed from current observations.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceLevelIndicators {
    /// 99th percentile stage latency in milliseconds.
    pub latency_p99_ms: f64,
    /// Error rate as a percentage (0.0–100.0).
    pub error_rate_pct: f64,
    /// Availability as a percentage (0.0–100.0).
    pub availability_pct: f64,
}

/// Violations of Service Level Objectives.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SloViolation {
    #[error("p99 latency SLO breach: observed {observed_ms}ms > threshold {threshold_ms}ms")]
    LatencyP99 { observed_ms: f64, threshold_ms: f64 },

    #[error("error rate SLO breach: observed {observed_pct:.3}% > threshold {threshold_pct:.3}%")]
    ErrorRate { observed_pct: f64, threshold_pct: f64 },

    #[error("availability SLO breach: observed {observed_pct:.3}% < threshold {threshold_pct:.3}%")]
    Availability { observed_pct: f64, threshold_pct: f64 },
}

/// A recorded observation with a timestamp for rolling window purges.
struct Observation<T> {
    timestamp: Instant,
    value: T,
}

/// The MetricsCollector records observations and computes SLIs.
///
/// It uses a Mutex-protected inner state to allow shared access across threads.
/// Observations older than `window_duration` are purged during updates.
pub struct MetricsCollector {
    inner: Mutex<CollectorState>,
    window_duration: Duration,
}

struct CollectorState {
    /// Count of receipts verified (throughput base).
    verified_total: VecDeque<Observation<()>>,
    /// Count of receipts accepted (availability base).
    accepted_total: VecDeque<Observation<()>>,
    /// Count of stage errors (error rate base).
    stage_errors_total: VecDeque<Observation<()>>,
    /// Stage latencies for p99 computation.
    stage_latencies: VecDeque<Observation<Duration>>,
}

impl MetricsCollector {
    /// Create a new collector with a default 5-minute rolling window.
    pub fn new() -> Self {
        Self::with_window(Duration::from_secs(300))
    }

    /// Create a "noop" collector (effectively just a fresh collector in this impl).
    pub fn new_noop() -> Self {
        Self::new()
    }

    pub fn with_window(window_duration: Duration) -> Self {
        Self {
            inner: Mutex::new(CollectorState {
                verified_total: VecDeque::new(),
                accepted_total: VecDeque::new(),
                stage_errors_total: VecDeque::new(),
                stage_latencies: VecDeque::new(),
            }),
            window_duration,
        }
    }

    /// Record a receipt verification attempt and its outcome.
    pub fn record_receipt_verified(&self, accepted: bool) {
        let mut state = self.inner.lock().unwrap();
        let now = Instant::now();
        state.verified_total.push_back(Observation { timestamp: now, value: () });
        if accepted {
            state.accepted_total.push_back(Observation { timestamp: now, value: () });
        }
        self.purge_expired(&mut state, now);
    }

    /// Record a failure in a specific pipeline stage.
    pub fn record_stage_error(&self, _stage: &str) {
        let mut state = self.inner.lock().unwrap();
        let now = Instant::now();
        state.stage_errors_total.push_back(Observation { timestamp: now, value: () });
        self.purge_expired(&mut state, now);
    }

    /// Record the latency of a pipeline stage.
    pub fn record_stage_latency(&self, _stage: &str, duration: Duration) {
        let mut state = self.inner.lock().unwrap();
        let now = Instant::now();
        state.stage_latencies.push_back(Observation { timestamp: now, value: duration });
        self.purge_expired(&mut state, now);
    }

    /// Compute current SLIs from observations in the rolling window.
    pub fn compute_sli(&self) -> anyhow::Result<ServiceLevelIndicators> {
        let mut state = self.inner.lock().unwrap();
        self.purge_expired(&mut state, Instant::now());

        let verified = state.verified_total.len() as f64;
        let accepted = state.accepted_total.len() as f64;
        let errors = state.stage_errors_total.len() as f64;

        // AC-5.10: Zero-state handling
        if verified == 0.0 {
            return Ok(ServiceLevelIndicators {
                latency_p99_ms: 0.0,
                error_rate_pct: 0.0,
                availability_pct: 100.0,
            });
        }

        // AC-5.2: p99 Latency
        let mut latencies: Vec<f64> = state.stage_latencies.iter()
            .map(|o| o.value.as_secs_f64() * 1000.0)
            .collect();
        
        let latency_p99_ms = if latencies.is_empty() {
            0.0
        } else {
            latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            // AC-5.2: For 100 samples, p99 should capture the outlier.
            // Using a simple "nearest rank" or "at least" logic.
            let index = (latencies.len() * 99 / 100).min(latencies.len() - 1);
            latencies[index]
        };

        // AC-5.3: Error Rate
        let error_rate_pct = (errors / verified) * 100.0;

        // AC-5.4: Availability
        let availability_pct = (accepted / verified) * 100.0;

        Ok(ServiceLevelIndicators {
            latency_p99_ms,
            error_rate_pct,
            availability_pct,
        })
    }

    /// Enforce SLO thresholds against current SLIs.
    pub fn check_slo(&self) -> anyhow::Result<(), SloViolation> {
        let sli = self.compute_sli().map_err(|_| {
             // Fallback for SDK failure (AC-5.5 variation)
             SloViolation::LatencyP99 { observed_ms: f64::NAN, threshold_ms: SLO_LATENCY_P99_MS }
        })?;

        // AC-5.6
        if sli.latency_p99_ms > SLO_LATENCY_P99_MS || sli.latency_p99_ms.is_nan() {
            return Err(SloViolation::LatencyP99 {
                observed_ms: sli.latency_p99_ms,
                threshold_ms: SLO_LATENCY_P99_MS,
            });
        }

        // AC-5.7
        if sli.error_rate_pct > SLO_ERROR_RATE_PCT {
            return Err(SloViolation::ErrorRate {
                observed_pct: sli.error_rate_pct,
                threshold_pct: SLO_ERROR_RATE_PCT,
            });
        }

        // AC-5.8
        if sli.availability_pct < SLO_AVAILABILITY_PCT {
            return Err(SloViolation::Availability {
                observed_pct: sli.availability_pct,
                threshold_pct: SLO_AVAILABILITY_PCT,
            });
        }

        Ok(())
    }

    fn purge_expired(&self, state: &mut CollectorState, now: Instant) {
        let window = self.window_duration;
        let is_expired = |ts: Instant| now.duration_since(ts) > window;

        while state.verified_total.front().map_or(false, |o| is_expired(o.timestamp)) {
            state.verified_total.pop_front();
        }
        while state.accepted_total.front().map_or(false, |o| is_expired(o.timestamp)) {
            state.accepted_total.pop_front();
        }
        while state.stage_errors_total.front().map_or(false, |o| is_expired(o.timestamp)) {
            state.stage_errors_total.pop_front();
        }
        while state.stage_latencies.front().map_or(false, |o| is_expired(o.timestamp)) {
            state.stage_latencies.pop_front();
        }
    }
}

// --- Prometheus Exporter (Maximalist implementation) ---

pub struct PrometheusExporter<'a> {
    collector: &'a MetricsCollector,
}

impl<'a> PrometheusExporter<'a> {
    pub fn new(collector: &'a MetricsCollector) -> Self {
        Self { collector }
    }

    /// Render the current metrics in Prometheus text format.
    pub fn render(&self) -> String {
        let sli = self.collector.compute_sli().unwrap_or(ServiceLevelIndicators {
            latency_p99_ms: 0.0,
            error_rate_pct: 0.0,
            availability_pct: 100.0,
        });

        let mut out = String::new();
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let state = self.collector.inner.lock().unwrap();

        // Throughput
        out.push_str("# HELP affidavit_receipts_verified_total Total count of receipts processed by the verifier.\n");
        out.push_str("# TYPE affidavit_receipts_verified_total counter\n");
        out.push_str(&format!("affidavit_receipts_verified_total {} {}\n", state.verified_total.len(), timestamp_ms));

        // Successes (for availability)
        out.push_str("affidavit_receipts_verified_total{verdict=\"accept\"} ");
        out.push_str(&format!("{} {}\n", state.accepted_total.len(), timestamp_ms));

        // Errors
        out.push_str("# HELP affidavit_stage_errors_total Total count of pipeline stage failures.\n");
        out.push_str("# TYPE affidavit_stage_errors_total counter\n");
        out.push_str(&format!("affidavit_stage_errors_total {} {}\n", state.stage_errors_total.len(), timestamp_ms));

        // SLIs as Gauges (AC-5.9)
        out.push_str("# HELP affidavit_slo_p99_latency_ms Current p99 stage latency SLI.\n");
        out.push_str("# TYPE affidavit_slo_p99_latency_ms gauge\n");
        out.push_str(&format!("affidavit_slo_p99_latency_ms {} {}\n", sli.latency_p99_ms, timestamp_ms));

        out.push_str("# HELP affidavit_slo_error_rate_pct Current error rate SLI.\n");
        out.push_str("# TYPE affidavit_slo_error_rate_pct gauge\n");
        out.push_str(&format!("affidavit_slo_error_rate_pct {} {}\n", sli.error_rate_pct, timestamp_ms));

        out.push_str("# HELP affidavit_slo_availability_pct Current availability SLI.\n");
        out.push_str("# TYPE affidavit_slo_availability_pct gauge\n");
        out.push_str(&format!("affidavit_slo_availability_pct {} {}\n", sli.availability_pct, timestamp_ms));

        out
    }
}

impl fmt::Display for PrometheusExporter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_state() {
        let collector = MetricsCollector::new();
        let sli = collector.compute_sli().unwrap();
        assert_eq!(sli.latency_p99_ms, 0.0);
        assert_eq!(sli.error_rate_pct, 0.0);
        assert_eq!(sli.availability_pct, 100.0);
        assert!(collector.check_slo().is_ok());
    }

    #[test]
    fn test_p99_computation() {
        let collector = MetricsCollector::new();
        // 99 observations of 10ms, 1 observation of 500ms (AC-5.2)
        for _ in 0..99 {
            collector.record_stage_latency("test", Duration::from_millis(10));
        }
        collector.record_stage_latency("test", Duration::from_millis(500));
        
        // We also need at least one verified receipt for sli to not be zero-state
        collector.record_receipt_verified(true);

        let sli = collector.compute_sli().unwrap();
        assert!(sli.latency_p99_ms >= 400.0 && sli.latency_p99_ms <= 600.0, 
                "p99 should be near 500ms, got {}", sli.latency_p99_ms);
    }

    #[test]
    fn test_error_rate_breach() {
        let collector = MetricsCollector::new();
        // 1000 verified, 2 errors (0.2%, above 0.1% threshold)
        for _ in 0..1000 {
            collector.record_receipt_verified(true);
        }
        collector.record_stage_error("test");
        collector.record_stage_error("test");

        let sli = collector.compute_sli().unwrap();
        assert_eq!(sli.error_rate_pct, 0.2);
        
        let result = collector.check_slo();
        assert!(matches!(result, Err(SloViolation::ErrorRate { .. })));
    }

    #[test]
    fn test_availability_breach() {
        let collector = MetricsCollector::new();
        // 1000 verified, 998 accepted (99.8%, below 99.9% threshold)
        for _ in 0..998 {
            collector.record_receipt_verified(true);
        }
        collector.record_receipt_verified(false);
        collector.record_receipt_verified(false);

        let sli = collector.compute_sli().unwrap();
        assert_eq!(sli.availability_pct, 99.8);
        
        let result = collector.check_slo();
        assert!(matches!(result, Err(SloViolation::Availability { .. })));
    }

    #[test]
    fn test_prometheus_output() {
        let collector = MetricsCollector::new();
        collector.record_receipt_verified(true);
        collector.record_stage_latency("decode", Duration::from_millis(5));
        
        let exporter = PrometheusExporter::new(&collector);
        let output = exporter.render();
        
        assert!(output.contains("affidavit_receipts_verified_total 1"));
        assert!(output.contains("affidavit_slo_p99_latency_ms 5"));
        assert!(output.contains("TYPE affidavit_stage_errors_total counter"));
    }
}
