//! End-to-end tests for the **semantic** path: cognitive breeds reasoning over a
//! feature space, in contrast to the numeric genetic algorithm. These lock in the
//! guarantees a caller relies on: a derived configuration honors the implication
//! graph, an over-constrained query is *proved* infeasible (not just scored low),
//! the two breeds agree, and the raw `intent.json` → `result.json` contract is
//! faithful to wasm4pm's shape.

use confevo::breeds::{run_named, supported_breeds};
use confevo::manifest::feature_space_from_str;
use confevo::{generate_config, ConfigQuery, Contract, Engine};

/// The same affidavit-shaped manifest the numeric tests use: real implication
/// chains plus the non-edge forms (`dep:`, `crate/feature`) the parser skips.
const MANIFEST: &str = r#"
[package]
name = "demo"
version = "0.0.0"

[features]
default = ["core", "json-output"]
core = ["dep:wasm4pm-compat"]
discovery = ["core"]
conformance = ["discovery"]
predictive = ["conformance"]
metrics = ["otel"]
otel = ["dep:opentelemetry"]
json-output = []
ui = []
lsp = ["tokio/rt"]
gpu = []
"#;

fn space() -> confevo::FeatureSpace {
    feature_space_from_str(MANIFEST, false).expect("manifest parses")
}

#[test]
fn derived_config_honors_the_implication_graph() {
    let s = space();
    for engine in [Engine::SatCdcl, Engine::CspAc3] {
        let cfg = generate_config(&s, &ConfigQuery::new().require("predictive"), engine).unwrap();
        assert!(cfg.feasible, "{engine:?} should find a config");
        let g = cfg.genome.unwrap();
        // Requiring predictive must transitively enable the whole chain.
        for f in ["predictive", "conformance", "discovery", "core"] {
            assert!(g.contains(f), "{engine:?}: expected {f} enabled");
        }
    }
}

#[test]
fn over_constrained_query_is_proved_infeasible_by_both_breeds() {
    let s = space();
    let q = ConfigQuery::new().require("predictive").forbid("core");
    for engine in [Engine::SatCdcl, Engine::CspAc3] {
        let cfg = generate_config(&s, &q, engine).unwrap();
        assert!(!cfg.feasible, "{engine:?} must report infeasible");
        assert_eq!(cfg.result.selected, "unsat", "{engine:?} must PROVE unsat");
        assert!(cfg.genome.is_none());
        // The structural diagnostic names the unavoidable forbidden feature.
        assert_eq!(cfg.clash, vec!["core".to_string()]);
    }
}

#[test]
fn the_two_breeds_agree_on_every_query() {
    let s = space();
    let queries = [
        ConfigQuery::new(),
        ConfigQuery::new().require("ui"),
        ConfigQuery::new().require("metrics"),
        ConfigQuery::new().require("conformance"),
        ConfigQuery::new().require("predictive").forbid("core"),
        ConfigQuery::new().require("metrics").forbid("otel"),
        ConfigQuery::new().require("ui").forbid("gpu"),
    ];
    for q in queries {
        let sat = generate_config(&s, &q, Engine::SatCdcl).unwrap();
        let csp = generate_config(&s, &q, Engine::CspAc3).unwrap();
        assert_eq!(
            sat.feasible, csp.feasible,
            "breeds disagree on feasibility for {q:?}"
        );
        // When feasible, both must produce configurations consistent with the graph
        // (not necessarily identical — SAT yields minimal, CSP yields first-found —
        // but both must be valid closures containing the requirements).
        if sat.feasible {
            for f in &q.require {
                assert!(sat.genome.as_ref().unwrap().contains(f));
                assert!(csp.genome.as_ref().unwrap().contains(f));
            }
        }
    }
}

#[test]
fn generation_is_bit_for_bit_deterministic() {
    let s = space();
    let q = ConfigQuery::new().require("metrics");
    let a = generate_config(&s, &q, Engine::SatCdcl).unwrap();
    let b = generate_config(&s, &q, Engine::SatCdcl).unwrap();
    assert_eq!(a.result.to_json(), b.result.to_json());
    assert_eq!(a.genome, b.genome);
    // run_id / output_hash are content hashes, so they are stable across runs.
    assert_eq!(a.result.run_id, b.result.run_id);
    assert_eq!(a.result.output_hash, b.result.output_hash);
}

#[test]
fn raw_contract_round_trips_through_wasm4pm_shape() {
    // A wasm4pm-format intent.json for graph coloring.
    let intent = r#"{
        "intent": "solve coloring",
        "candidates": [],
        "facts": [
            {"key": "csp-var", "value": "V1:R,G,B"},
            {"key": "csp-var", "value": "V2:R,G,B"},
            {"key": "csp-constraint", "value": "V1!=V2"}
        ],
        "cases": [], "rules": [], "goals": [], "state": []
    }"#;
    let contract = Contract::from_json(intent).unwrap();
    let result = run_named("csp_ac3", &contract).unwrap();

    assert_eq!(result.breed, "CspAc3");
    assert_eq!(result.status, "ok");
    assert_eq!(result.selected, "sat");
    assert!(result.explanation.starts_with("SAT: "));
    // The two color assignments must differ.
    let assigns: Vec<&str> = result
        .facts
        .iter()
        .filter(|f| f.key == "assign")
        .map(|f| f.value.as_str())
        .collect();
    assert_eq!(assigns.len(), 2);
    assert_ne!(assigns[0], assigns[1]);

    // The emitted result.json must be valid, balanced, and carry the contract keys.
    let json = result.to_json();
    assert_eq!(json.matches('{').count(), json.matches('}').count());
    assert_eq!(json.matches('[').count(), json.matches(']').count());
    for key in [
        "\"breed\"",
        "\"selected\"",
        "\"inference_trace\"",
        "\"run_id\"",
        "\"output_hash\"",
    ] {
        assert!(json.contains(key), "result.json missing {key}");
    }
}

#[test]
fn pigeonhole_is_unsat_via_sat_breed() {
    // PHP(2,1): two pigeons, one hole. p1=x1, p2=x2 each "in the hole"; at least
    // one must be (x1 ∨ x2) but they cannot share it (¬x1 ∨ ¬x2)… that's still SAT.
    // The genuine contradiction: each pigeon must be placed (x1) (x2) AND not both
    // (¬x1 ∨ ¬x2) ⇒ UNSAT.
    let contract = Contract::new(
        "decide PHP(2,1)",
        vec![
            confevo::Fact::new("clause:0", "1"),
            confevo::Fact::new("clause:1", "2"),
            confevo::Fact::new("clause:2", "-1 -2"),
        ],
    );
    let r = run_named("sat_cdcl", &contract).unwrap();
    assert_eq!(r.selected, "unsat");
    assert!(r.inference_trace.iter().any(|t| t.kind == "sat-conflict"));
}

#[test]
fn supported_breeds_are_runnable() {
    // Every advertised breed id must dispatch (no dangling names).
    for id in supported_breeds() {
        let err = run_named(id, &Contract::default());
        assert!(err.is_ok(), "advertised breed {id} failed to dispatch");
    }
}
