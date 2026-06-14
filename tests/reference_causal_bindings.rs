// Reference witness: the causal-net structural binding components — CausalBinding
// (input/output binding obligations), InputBinding/OutputBinding (typed binding
// pairs), DependencyMeasure (COVERAGE.md §2 — causal-net components).
//
// A causal net's nodes carry input/output binding obligations (the AND/XOR
// split-join semantics of Heuristics-Miner output). This witnesses constructing
// these components into a net and reading them back; validate() admits the net.

use wasm4pm_compat::causal_net::{
    CausalBinding, CausalNet, DependencyMeasure, InputBinding, OutputBinding,
};

#[test]
fn causal_bindings_and_measures_construct() {
    let input = CausalBinding {
        source_tasks: vec!["a".into()],
        target_tasks: vec!["b".into()],
    };
    let output = CausalBinding {
        source_tasks: vec!["b".into()],
        target_tasks: vec!["c".into()],
    };
    let net = CausalNet {
        nodes: vec!["a".into(), "b".into(), "c".into()],
        dependency_measures: vec![("a".into(), "b".into(), 0.9), ("b".into(), "c".into(), 0.8)],
        inputs: vec![input.clone()],
        outputs: vec![output.clone()],
    };

    assert_eq!(net.validate(), Ok(()), "well-formed causal net admits");
    assert_eq!(net.inputs[0].source_tasks, vec!["a".to_string()]);
    assert_eq!(net.outputs[0].target_tasks, vec!["c".to_string()]);
    assert_eq!(net.dependency_measures.len(), 2);
}

#[test]
fn typed_binding_pairs_and_dependency_measure() {
    // Typed binding pairs (zero-cost structural labels) and the dependency score.
    let ib = InputBinding("a", "b");
    let ob = OutputBinding("b", "c");
    assert_eq!(ib.0, "a");
    assert_eq!(ob.1, "c");

    let d = DependencyMeasure(0.75);
    assert_eq!(d.0, 0.75, "dependency measure carries its score");
}
