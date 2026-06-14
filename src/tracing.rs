//! Operation span recording for the affidavit pipeline.
//!
//! This is the OTel-shaped instrumentation seam. Each receipt operation
//! (emit/assemble/verify/show/model/conformance/graph/stats/replay/diagnose)
//! opens a span via the `trace_*` wrappers. Spans
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
fn record_span(operation: &str, target: &str) {
    SPAN_LOG.with(|log| {
        log.borrow_mut().push(SpanRecord {
            operation: operation.to_string(),
            target: target.to_string(),
        });
    });
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

/// Trace a model operation: opens a `model` span, then runs `f`.
/// Covers wasm4pm process discovery on an admitted receipt.
pub fn trace_model<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("model", receipt_path);
    f()
}

/// Trace a conformance operation: opens a `conformance` span, then runs `f`.
/// Covers fitness/activity_coverage/simplicity metrics via wasm4pm.
pub fn trace_conformance<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("conformance", receipt_path);
    f()
}

/// Trace a graph operation: opens a `graph` span, then runs `f`.
/// Covers directly-follows graph discovery via wasm4pm.
pub fn trace_graph<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("graph", receipt_path);
    f()
}

/// Trace a stats operation: opens a `stats` span, then runs `f`.
/// Covers aggregate counts + conformance metrics.
pub fn trace_stats<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("stats", receipt_path);
    f()
}

/// Trace a replay operation: opens a `replay` span, then runs `f`.
/// Covers event trace replay in lawful seq order.
pub fn trace_replay<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("replay", receipt_path);
    f()
}

/// Trace a diagnose operation: opens a `diagnose` span, then runs `f`.
/// Covers LSP diagnostic generation from verify outcomes.
pub fn trace_diagnose<F, T>(receipt_path: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    record_span("diagnose", receipt_path);
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
            spans.iter().any(|s| s.operation == "verify" && s.target == "receipt.json"),
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
