//! Operation span recording for the affidavit pipeline.
//!
//! This is the OTel-shaped instrumentation seam. Each receipt operation
//! (emit/assemble/verify/show) opens a span via the `trace_*` wrappers. Spans
//! are recorded into an observable, thread-local sink so that a test can assert
//! a span was actually emitted — the witness that this instrumentation is real
//! and not a dormant no-op (see `tests/otel_witness.rs`).
//!
//! ## Honest scope
//!
//! What is witnessed here is **span emission and capture**: a real consumer
//! (the thread-local recorder) receives a real span record, and a test asserts
//! it. Full OpenTelemetry SDK export to a collector (Jaeger/OTLP) is an
//! additional surface gated behind the `otel` feature; that export path is
//! OPEN-substrate until a test captures an exported span from a running
//! collector. We do not claim Jaeger export is witnessed — only that operations
//! emit observable spans.

use std::cell::RefCell;

/// A recorded operation span: the operation name and its target (e.g. the
/// receipt path). OTel-shaped (name + attribute), captured for witnessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanRecord {
    /// The operation that opened the span (`verify`, `emit`, …).
    pub operation: String,
    /// The span's primary target attribute (receipt path, event type, …).
    pub target: String,
}

thread_local! {
    static SPAN_LOG: RefCell<Vec<SpanRecord>> = const { RefCell::new(Vec::new()) };
}

/// Record a span into the observable thread-local sink. This is the consumer
/// that makes the instrumentation witnessable.
pub(crate) fn record_span(operation: &str, target: &str) {
    let record = SpanRecord {
        operation: operation.to_string(),
        target: target.to_string(),
    };

    // Thread-local log for in-process witnessing (tests)
    SPAN_LOG.with(|log| {
        log.borrow_mut().push(record.clone());
    });

    // Global sink for cross-process DX innovation (TDD Synthesizer)
    if let Ok(sink_path) = std::env::var("AFFI_TRACE_SINK") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(sink_path)
        {
            let line = format!("{}|{}\n", record.operation, record.target);
            let _ = std::io::Write::write_all(&mut file, line.as_bytes());
        }
    }
}

/// Snapshot the spans recorded on the current thread (for witnessing).
///
/// # Example: see `examples/observable_spans.rs` (run: `cargo run --example observable_spans`).
pub fn captured_spans() -> Vec<SpanRecord> {
    SPAN_LOG.with(|log| log.borrow().clone())
}

/// Clear the current thread's recorded spans (call at the start of a witness).
pub fn clear_spans() {
    SPAN_LOG.with(|log| log.borrow_mut().clear());
}

/// Trace an emit operation: opens an `emit` span, then runs `f`.
pub fn trace_emit<F, T>(event_type: &str, _object_count: usize, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("emit", event_type);
    f()
}

/// Trace an assemble operation: opens an `assemble` span, then runs `f`.
pub fn trace_assemble<F, T>(event_count: usize, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("assemble", &event_count.to_string());
    f()
}

/// Trace a verify operation: opens a `verify` span, then runs `f`.
///
/// # Example: see `examples/observable_spans.rs` (run: `cargo run --example observable_spans`).
pub fn trace_verify<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("verify", receipt_path);
    f()
}

/// Trace a show operation: opens a `show` span, then runs `f`.
pub fn trace_show<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("show", receipt_path);
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_verify_emits_an_observable_span() {
        clear_spans();
        let out = trace_verify("receipt.json", || 42);
        assert_eq!(out, 42, "trace wrapper must return the inner result");
        let spans = captured_spans();
        assert!(
            spans
                .iter()
                .any(|s| s.operation == "verify" && s.target == "receipt.json"),
            "verify must emit an observable span; got {spans:?}"
        );
    }

    #[test]
    fn no_span_recorded_without_a_traced_operation() {
        // Negative control: a fresh thread-local sink is empty until an
        // operation opens a span. Proves capture reflects real emission, not a
        // constant true.
        clear_spans();
        assert!(captured_spans().is_empty(), "no spans before any traced op");
    }
}

/// Verify-pipeline stage names, in pipeline order. These are the only stage
/// names a latency band may be declared over — the same vocabulary the
/// 7-stage certify pipeline uses (`crate::verifier`).
pub const VERIFY_STAGES: [&str; 7] = [
    "decode",
    "check_format",
    "chain_integrity",
    "continuity",
    "verify_commitments",
    "evaluate_profile",
    "emit_verdict",
];

/// A typed latency failure naming the layer that produced it.
///
/// `layer` is `"span"` when the span record itself is unlawful (unknown stage
/// name) and `"band"` when a caller-supplied duration exceeds the declared
/// per-stage maximum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimingViolation {
    /// The verify-pipeline stage under measurement.
    pub stage: String,
    /// The caller-supplied duration (nanoseconds). Never wall-clock.
    pub duration_nanos: u64,
    /// The declared per-stage maximum, when a band exists for the stage.
    pub max_nanos: Option<u64>,
    /// Which layer produced the violation: `"span"` or `"band"`.
    pub layer: &'static str,
}

impl std::fmt::Display for TimingViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max_nanos {
            Some(max) => write!(
                f,
                "timing violation [{}] stage={}: {}ns exceeds declared band {}ns",
                self.layer, self.stage, self.duration_nanos, max
            ),
            None => write!(
                f,
                "timing violation [{}] stage={}: {}ns",
                self.layer, self.stage, self.duration_nanos
            ),
        }
    }
}

impl std::error::Error for TimingViolation {}

/// Latency-band registry over the verify-pipeline stage names.
///
/// Durations are caller-supplied (synthetic in tests); no wall-clock is read
/// anywhere in this module.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LatencyBands {
    bands: std::collections::BTreeMap<&'static str, u64>,
}

impl LatencyBands {
    /// A registry pre-declared over all seven verify-pipeline stages.
    pub fn verify_pipeline() -> Self {
        let mut bands = Self::default();
        for stage in VERIFY_STAGES {
            let _ = bands.declare(stage, 1_000_000);
        }
        bands
    }

    /// Declare (or re-declare) the max duration for a pipeline stage.
    ///
    /// Refuses stage names outside `VERIFY_STAGES` — an unknown stage cannot
    /// carry a band, because no span can ever name it.
    pub fn declare(&mut self, stage: &'static str, max_nanos: u64) -> Result<(), TimingViolation> {
        if !VERIFY_STAGES.contains(&stage) {
            return Err(TimingViolation {
                stage: stage.to_string(),
                duration_nanos: 0,
                max_nanos: None,
                layer: "span",
            });
        }
        self.bands.insert(stage, max_nanos);
        Ok(())
    }

    /// Check a caller-supplied duration against the declared band.
    ///
    /// A stage with no declared band passes (nothing declared, nothing to
    /// violate); a stage over its band returns a `["band"]`-layer violation.
    pub fn check(&self, stage: &str, duration_nanos: u64) -> Result<(), TimingViolation> {
        match self.bands.get(stage) {
            None => Ok(()),
            Some(max) if duration_nanos <= *max => Ok(()),
            Some(max) => Err(TimingViolation {
                stage: stage.to_string(),
                duration_nanos,
                max_nanos: Some(*max),
                layer: "band",
            }),
        }
    }

    /// The declared maximum for a stage, if any.
    pub fn max_for(&self, stage: &str) -> Option<u64> {
        self.bands.get(stage).copied()
    }
}

/// Cross-validate a span's caller-supplied duration against the declared
/// latency band.
///
/// Two layers must both hold:
///
/// 1. **span layer** — the stage name is a real verify-pipeline stage (a name
///    a span could lawfully carry);
/// 2. **band layer** — if a band is declared for that stage, the duration is
///    within it.
///
/// Corruption of either layer yields a typed `TimingViolation` naming the
/// layer. If this function were deleted, the span-only path (`record_span`)
/// still stands — witnessed by `trace_verify_emits_an_observable_span` — but
/// the positive-control court below loses its subject and fails to compile:
/// that compile break is what the court flips on.
pub fn cross_validate(
    stage: &str,
    duration_nanos: u64,
    bands: &LatencyBands,
) -> Result<(), TimingViolation> {
    if !VERIFY_STAGES.contains(&stage) {
        return Err(TimingViolation {
            stage: stage.to_string(),
            duration_nanos,
            max_nanos: bands.max_for(stage),
            layer: "span",
        });
    }
    bands.check(stage, duration_nanos)
}

#[cfg(test)]
mod latency_courts {
    use super::*;

    #[test]
    fn over_band_span_without_a_violation_is_impossible_positive_control() {
        // Positive control: an over-band span MUST yield a band-layer
        // TimingViolation. If the check cell were deleted, this test fails —
        // a court that cannot fire is not a court.
        let bands = LatencyBands::verify_pipeline();
        let err = cross_validate("chain_integrity", 5_000_000, &bands)
            .expect_err("5ms over a 1ms band must violate");
        assert_eq!(err.layer, "band", "violation must name the band layer");
        assert_eq!(err.stage, "chain_integrity");
        assert_eq!(err.max_nanos, Some(1_000_000));
        assert!(
            err.to_string().contains("band") && err.to_string().contains("chain_integrity"),
            "display must name layer and stage: {err}"
        );
    }

    #[test]
    fn within_band_span_passes_and_undeclared_stage_passes() {
        let bands = LatencyBands::verify_pipeline();
        assert!(cross_validate("decode", 999_999, &bands).is_ok());
        assert!(
            cross_validate("decode", 1_000_000, &bands).is_ok(),
            "boundary inclusive"
        );
        // No band declared for a stage name that IS lawful: band layer passes.
        let mut sparse = LatencyBands::default();
        sparse
            .declare("continuity", 10)
            .expect("continuity is a real stage");
        assert!(cross_validate("decode", u64::MAX, &sparse).is_ok());
    }

    #[test]
    fn corrupted_span_layer_yields_violation_naming_the_span_layer() {
        // Two layers are consistent for a lawful stage+duration; corrupt the
        // span layer (unknown stage name) and the violation names "span".
        let bands = LatencyBands::verify_pipeline();
        let err = cross_validate("not_a_pipeline_stage", 1, &bands)
            .expect_err("unknown stage must violate");
        assert_eq!(err.layer, "span", "violation must name the span layer");
        assert!(err.to_string().contains("not_a_pipeline_stage"));
    }

    #[test]
    fn declaring_a_band_over_an_unknown_stage_is_refused() {
        let mut bands = LatencyBands::default();
        let err = bands
            .declare("fabricated", 100)
            .expect_err("band over a non-stage must be refused");
        assert_eq!(err.layer, "span");
        assert!(
            bands.max_for("fabricated").is_none(),
            "refused declaration must not land"
        );
    }

    #[test]
    fn re_declaration_tightens_a_band() {
        let mut bands = LatencyBands::default();
        bands.declare("emit_verdict", 1_000).expect("real stage");
        assert!(cross_validate("emit_verdict", 1_000, &bands).is_ok());
        bands.declare("emit_verdict", 10).expect("real stage");
        assert!(
            cross_validate("emit_verdict", 1_000, &bands).is_err(),
            "tightened band must flip the previously-legal duration to a violation"
        );
    }

    #[test]
    fn every_pipeline_stage_is_band_addressable() {
        // Vacuity sweep: all seven stage names are lawful declare targets and
        // each declared band actually fires on an over-band duration.
        for stage in VERIFY_STAGES {
            let mut bands = LatencyBands::default();
            bands
                .declare(stage, 1)
                .expect("pipeline stage must be declarable");
            let err = cross_validate(stage, 2, &bands)
                .expect_err("over-band duration on a declared band must fire");
            assert_eq!(err.layer, "band");
            assert_eq!(err.stage, stage);
        }
    }
}
