//! End-to-end witness that affidavit operations emit OTel-shaped observable spans.
//!
//! Demonstrates `affidavit::tracing::{clear_spans, trace_verify, captured_spans, SpanRecord}`:
//! a traced operation records exactly one `SpanRecord` with the right `operation`
//! name and `target` attribute, while a freshly-cleared sink is empty (the negative
//! control proving the span is real emission, not a hardcoded constant).
//!
//! See the doc on `trace_verify` / `captured_spans` in src/tracing.rs.

use affidavit::tracing::{
    captured_spans, clear_spans, trace_assemble, trace_emit, trace_show, trace_verify, SpanRecord,
};

fn main() {
    // Negative control: after clearing, no operation has run, so no span exists.
    // If capture returned a constant, this would fail.
    clear_spans();
    let before = captured_spans();
    assert!(
        before.is_empty(),
        "negative control failed: expected no spans before any traced op, got {before:?}"
    );

    // Run a real traced operation. The wrapper must run the closure AND record a span.
    let result = trace_verify("receipt.json", || {
        // stand-in for a real verify body
        "verified"
    });
    assert_eq!(
        result, "verified",
        "trace_verify must return the inner closure result"
    );

    // Exactly one span must have been recorded, with the right operation + target.
    let spans = captured_spans();
    assert_eq!(
        spans.len(),
        1,
        "expected exactly one observable span after one traced op, got {spans:?}"
    );
    assert_eq!(
        spans[0],
        SpanRecord {
            operation: "verify".to_string(),
            target: "receipt.json".to_string(),
        },
        "span must carry the real operation name and target attribute"
    );

    println!("OK: observable span emitted and witnessed: {:?}", spans[0]);

    // --- All four operation wrappers emit their own span with the right target ---
    // (emit/assemble/verify/show — the full traced surface). Clear, run each, and
    // assert the accumulated spans match operation + target exactly.
    clear_spans();
    trace_emit("artifact.seed", 1, || ());
    trace_assemble(3, || ());
    trace_verify("r.json", || ());
    trace_show("r.json", || ());
    let all = captured_spans();
    assert_eq!(all.len(), 4, "four traced ops -> four spans; got {all:?}");
    assert_eq!(
        all[0],
        SpanRecord {
            operation: "emit".into(),
            target: "artifact.seed".into()
        }
    );
    assert_eq!(
        all[1],
        SpanRecord {
            operation: "assemble".into(),
            target: "3".into()
        },
        "assemble's target is the event count"
    );
    assert_eq!(
        all[2],
        SpanRecord {
            operation: "verify".into(),
            target: "r.json".into()
        }
    );
    assert_eq!(
        all[3],
        SpanRecord {
            operation: "show".into(),
            target: "r.json".into()
        }
    );

    println!("OK: all 4 trace wrappers (emit/assemble/verify/show) emit spans: {all:?}");
}
