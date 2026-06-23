// Reference witness: all four operation span wrappers emit observable spans
// (COVERAGE.md — OTel span surface, complements the verify-only otel_witness).
//
// affidavit::tracing exposes trace_emit/trace_assemble/trace_verify/trace_show,
// each recording an observable span into the thread-local sink. This witnesses
// every wrapper emits its named span with its target — failing-when-fake: drop a
// record_span call and that operation's span vanishes.

use affidavit::tracing::{
    captured_spans, clear_spans, trace_assemble, trace_emit, trace_show, trace_verify,
};

#[test]
fn all_four_operation_wrappers_emit_named_spans() {
    clear_spans();

    trace_emit("create", 1, || ());
    trace_assemble(3, || ());
    trace_verify("r.json", || ());
    trace_show("r.json", || ());

    let spans = captured_spans();
    let ops: std::collections::BTreeSet<&str> =
        spans.iter().map(|s| s.operation.as_str()).collect();
    assert!(ops.contains("emit"), "emit span emitted; got {ops:?}");
    assert!(ops.contains("assemble"), "assemble span emitted");
    assert!(ops.contains("verify"), "verify span emitted");
    assert!(ops.contains("show"), "show span emitted");
    assert_eq!(ops.len(), 4, "exactly the four operation spans");
}

#[test]
fn span_wrappers_return_the_inner_result() {
    clear_spans();
    let out = trace_verify("x", || 7 + 1);
    assert_eq!(
        out, 8,
        "the wrapper is transparent to the inner computation's result"
    );
}
