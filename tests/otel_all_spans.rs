// Reference witness: all operation span wrappers emit observable spans
// (COVERAGE.md — OTel span surface, complements the verify-only otel_witness).
//
// affidavit::tracing exposes trace_emit/trace_assemble/trace_verify/trace_show
// and the new DX-verb wrappers trace_model/trace_conformance/trace_graph/
// trace_stats/trace_replay/trace_diagnose, each recording an observable span
// into the thread-local sink. This witnesses every wrapper emits its named span
// with its target — failing-when-fake: drop a record_span call and that
// operation's span vanishes.

use affidavit::tracing::{
    captured_spans, clear_spans, trace_assemble, trace_bench, trace_conformance, trace_diagnose,
    trace_emit, trace_graph, trace_inspect, trace_model, trace_mutate, trace_replay, trace_show,
    trace_stats, trace_verify,
};

#[test]
fn all_four_operation_wrappers_emit_named_spans() {
    clear_spans();

    let _ = trace_emit("create", 1, || ());
    let _ = trace_assemble(3, || ());
    let _ = trace_verify("r.json", || ());
    let _ = trace_show("r.json", || ());

    let spans = captured_spans();
    let ops: std::collections::BTreeSet<&str> = spans.iter().map(|s| s.operation.as_str()).collect();
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
    assert_eq!(out, 8, "the wrapper is transparent to the inner computation's result");
}

#[test]
fn model_emits_an_observable_span() {
    clear_spans();
    let _ = trace_model("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "model" && s.target == "test-receipt.json"),
        "model must emit an observable span; got {spans:?}"
    );
}

#[test]
fn conformance_emits_an_observable_span() {
    clear_spans();
    let _ = trace_conformance("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "conformance" && s.target == "test-receipt.json"),
        "conformance must emit an observable span; got {spans:?}"
    );
}

#[test]
fn graph_emits_an_observable_span() {
    clear_spans();
    let _ = trace_graph("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "graph" && s.target == "test-receipt.json"),
        "graph must emit an observable span; got {spans:?}"
    );
}

#[test]
fn stats_emits_an_observable_span() {
    clear_spans();
    let _ = trace_stats("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "stats" && s.target == "test-receipt.json"),
        "stats must emit an observable span; got {spans:?}"
    );
}

#[test]
fn replay_emits_an_observable_span() {
    clear_spans();
    let _ = trace_replay("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "replay" && s.target == "test-receipt.json"),
        "replay must emit an observable span; got {spans:?}"
    );
}

#[test]
fn diagnose_emits_an_observable_span() {
    clear_spans();
    let _ = trace_diagnose("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "diagnose" && s.target == "test-receipt.json"),
        "diagnose must emit an observable span; got {spans:?}"
    );
}

#[test]
fn all_thirteen_operation_wrappers_emit_named_spans() {
    clear_spans();

    let _ = trace_emit("create", 1, || ());
    let _ = trace_assemble(3, || ());
    let _ = trace_verify("r.json", || ());
    let _ = trace_show("r.json", || ());
    let _ = trace_model("r.json", || ());
    let _ = trace_conformance("r.json", || ());
    let _ = trace_graph("r.json", || ());
    let _ = trace_stats("r.json", || ());
    let _ = trace_replay("r.json", || ());
    let _ = trace_diagnose("r.json", || ());
    let _ = trace_inspect("r.json", || ());
    let _ = trace_mutate("r.json", || ());
    let _ = trace_bench("bench-label", || ());

    let spans = captured_spans();
    let ops: std::collections::BTreeSet<&str> =
        spans.iter().map(|s| s.operation.as_str()).collect();
    assert!(ops.contains("emit"), "emit span emitted; got {ops:?}");
    assert!(ops.contains("assemble"), "assemble span emitted");
    assert!(ops.contains("verify"), "verify span emitted");
    assert!(ops.contains("show"), "show span emitted");
    assert!(ops.contains("model"), "model span emitted");
    assert!(ops.contains("conformance"), "conformance span emitted");
    assert!(ops.contains("graph"), "graph span emitted");
    assert!(ops.contains("stats"), "stats span emitted");
    assert!(ops.contains("replay"), "replay span emitted");
    assert!(ops.contains("diagnose"), "diagnose span emitted");
    assert!(ops.contains("inspect"), "inspect span emitted");
    assert!(ops.contains("mutate"), "mutate span emitted");
    assert!(ops.contains("bench"), "bench span emitted");
    assert_eq!(ops.len(), 13, "exactly the thirteen operation spans");
}

#[test]
fn inspect_emits_an_observable_span() {
    clear_spans();
    let _ = trace_inspect("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "inspect" && s.target == "test-receipt.json"),
        "inspect must emit an observable span; got {spans:?}"
    );
}

#[test]
fn mutate_emits_an_observable_span() {
    clear_spans();
    let _ = trace_mutate("test-receipt.json", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "mutate" && s.target == "test-receipt.json"),
        "mutate must emit an observable span; got {spans:?}"
    );
}

#[test]
fn bench_emits_an_observable_span() {
    clear_spans();
    let _ = trace_bench("bench-label", || ());
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "bench" && s.target == "bench-label"),
        "bench must emit an observable span; got {spans:?}"
    );
}
