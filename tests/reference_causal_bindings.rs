// Reference witness: the causal-net structural binding components — CausalBinding
// (dynamic input/output binding obligations), InputBinding/OutputBinding (static
// typed binding pairs), and DependencyMeasure (COVERAGE.md §2 — causal-net components).
//
// wasm4pm-compat v26.8.7 makes the dynamic net's boundary and short-loop
// metadata explicit and makes the static dependency measure a type-level rational.
// This witness constructs the complete current dynamic shape and the static laws
// without conflating structural annotations with miner execution.

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
        initial_node: Some("a".into()),
        final_node: Some("c".into()),
        dependency_measures: vec![("a".into(), "b".into(), 0.9), ("b".into(), "c".into(), 0.8)],
        inputs: vec![input.clone()],
        outputs: vec![output.clone()],
        loops_len1: vec![],
        loops_len2: vec![],
    };

    assert_eq!(net.validate(), Ok(()), "well-formed causal net admits");
    assert_eq!(net.initial_node.as_deref(), Some("a"));
    assert_eq!(net.final_node.as_deref(), Some("c"));
    assert_eq!(net.inputs[0].source_tasks, vec!["a".to_string()]);
    assert_eq!(net.outputs[0].target_tasks, vec!["c".to_string()]);
    assert_eq!(net.dependency_measures.len(), 2);
    assert!(net.loops_len1.is_empty());
    assert!(net.loops_len2.is_empty());
}

#[test]
fn typed_binding_pairs_and_dependency_measure() {
    // Static typed binding pairs are zero-cost labels for compile-time obligations.
    let ib = InputBinding::new("a", "b");
    let ob = OutputBinding::new("b", "c");
    assert_eq!(ib.source, "a");
    assert_eq!(ib.target, "b");
    assert_eq!(ob.source, "b");
    assert_eq!(ob.target, "c");

    // 3/4 is admitted at the type boundary because 0 <= NUM <= DEN and DEN > 0.
    // Invalid fractions such as DependencyMeasure<5, 4> fail at compile time.
    let d = DependencyMeasure::<3, 4>::new();
    assert_eq!(d.num(), 3);
    assert_eq!(d.den(), 4);
    assert_eq!(d.as_f64(), 0.75);
}
