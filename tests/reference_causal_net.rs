// Reference witness: the Causal-Net law (CausalNet) — all 3 CausalNetRefusal
// variants fired against real violations. With this, EVERY reachable refusal
// enum in wasm4pm-compat is witnessed (COVERAGE.md §2.6).
//
// CausalNet::validate enforces:
//   - MissingActivity — a node label must be non-empty
//   - InvalidDependencyScore — dependency scores must be finite and in [0,1]
//   - DisconnectedGraph — (for >1 node) every node must appear in some arc

use wasm4pm_compat::causal_net::{CausalNet, CausalNetRefusal};

fn net(nodes: Vec<&str>, deps: Vec<(&str, &str, f64)>) -> CausalNet {
    CausalNet {
        nodes: nodes.into_iter().map(String::from).collect(),
        dependency_measures: deps
            .into_iter()
            .map(|(s, t, w)| (s.to_string(), t.to_string(), w))
            .collect(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    }
}

#[test]
fn causal_net_refuses_missing_activity() {
    let n = net(vec!["a", ""], vec![]); // empty node label
    assert_eq!(n.validate(), Err(CausalNetRefusal::MissingActivity));
}

#[test]
fn causal_net_refuses_invalid_dependency_score() {
    let n = net(vec!["a", "b"], vec![("a", "b", 1.5)]); // score > 1.0
    assert_eq!(n.validate(), Err(CausalNetRefusal::InvalidDependencyScore));
    let nan = net(vec!["a", "b"], vec![("a", "b", f64::NAN)]);
    assert_eq!(
        nan.validate(),
        Err(CausalNetRefusal::InvalidDependencyScore)
    );
}

#[test]
fn causal_net_refuses_disconnected_graph() {
    // "c" is isolated — referenced by no arc.
    let n = net(vec!["a", "b", "c"], vec![("a", "b", 0.9)]);
    assert_eq!(n.validate(), Err(CausalNetRefusal::DisconnectedGraph));
}

#[test]
fn causal_net_admits_well_formed_net() {
    let n = net(vec!["a", "b"], vec![("a", "b", 0.9)]);
    assert_eq!(
        n.validate(),
        Ok(()),
        "non-empty nodes, valid score, connected → admits"
    );
}
