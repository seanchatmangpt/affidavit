//! End-to-end integration tests exercising the full library stack:
//! manifest parsing → feature space → GA → report, with the hermetic synthetic
//! evaluator (no `cargo`, no subprocess). These lock in the public guarantees a
//! caller relies on: determinism, monotonicity-under-elitism, and honesty (the
//! synthetic model never fakes a build for a poisoned space).

use confevo::manifest::feature_space_from_str;
use confevo::report::{to_json, to_markdown, Mode};
use confevo::{run_ga, GaConfig, SyntheticEvaluator};

/// A realistic, affidavit-shaped manifest with implication chains and the
/// non-edge value forms (`dep:`, `crate/feature`) the parser must skip.
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

[dependencies]
wasm4pm-compat = { version = "26.6.13", optional = true }
"#;

fn space() -> confevo::FeatureSpace {
    feature_space_from_str(MANIFEST, false).expect("manifest parses")
}

#[test]
fn manifest_yields_expected_universe_and_edges() {
    let s = space();
    // `default` excluded; the remaining 10 features form the universe.
    assert_eq!(s.len(), 10);
    assert!(s.contains("core"));
    assert!(!s.contains("default"));

    // discovery -> core is a real intra-crate edge.
    let disc: Vec<&String> = s.implications_of("discovery").collect();
    assert_eq!(disc, vec![&"core".to_string()]);

    // `lsp = ["tokio/rt"]` is a cross-crate enable, NOT an intra-crate edge.
    assert_eq!(s.implications_of("lsp").count(), 0);

    // predictive transitively closes to conformance/discovery/core.
    let closure = s.closure(&["predictive".to_string()].into_iter().collect());
    let mut got: Vec<String> = closure.into_iter().collect();
    got.sort();
    assert_eq!(got, vec!["conformance", "core", "discovery", "predictive"]);
}

#[test]
fn full_run_is_deterministic() {
    let s = space();
    let cfg = GaConfig {
        generations: 6,
        population: 8,
        seed: 42,
        ..Default::default()
    };

    let mut e1 = SyntheticEvaluator::new(6, [("core", 550u64)]);
    let r1 = run_ga(&mut e1, &s, &cfg).unwrap();
    let mut e2 = SyntheticEvaluator::new(6, [("core", 550u64)]);
    let r2 = run_ga(&mut e2, &s, &cfg).unwrap();

    assert_eq!(r1.best_genome, r2.best_genome);
    assert_eq!(r1.best_eval, r2.best_eval);

    // Reports are byte-identical too.
    let j1 = to_json(&r1, &cfg, Mode::DryRun, "Cargo.toml", s.len());
    let j2 = to_json(&r2, &cfg, Mode::DryRun, "Cargo.toml", s.len());
    assert_eq!(j1, j2);
}

#[test]
fn best_so_far_never_regresses() {
    let s = space();
    let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
    let cfg = GaConfig {
        generations: 10,
        population: 10,
        seed: 123,
        elitism: 2,
        ..Default::default()
    };
    let r = run_ga(&mut e, &s, &cfg).unwrap();
    let scores: Vec<f64> = r.history.iter().map(|h| h.best_score).collect();
    for w in scores.windows(2) {
        assert!(
            w[1] >= w[0],
            "best regressed across generations: {scores:?}"
        );
    }
}

#[test]
fn poisoned_space_never_fakes_a_build() {
    let s = space();
    let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
    let cfg = GaConfig {
        generations: 8,
        population: 12,
        seed: 7,
        ..Default::default()
    };
    let r = run_ga(&mut e, &s, &cfg).unwrap();
    assert!(!r.best_eval.builds);
    // The markdown frames a no-build run as a diagnostic map, not a success.
    let md = to_markdown(&r, &cfg, Mode::DryRun, "Cargo.toml", s.len());
    assert!(md.contains("No configuration built successfully"));
}

#[test]
fn generic_model_needs_no_domain_knowledge() {
    // A caller with no per-project poison table still gets a deterministic,
    // gradient-bearing landscape from the generic synthetic evaluator.
    let s = space();
    let mut e = SyntheticEvaluator::generic();
    let cfg = GaConfig {
        generations: 5,
        population: 6,
        seed: 1,
        ..Default::default()
    };
    let r = run_ga(&mut e, &s, &cfg).unwrap();
    assert_eq!(r.history.len(), 5);
    assert!(!r.best_eval.builds);
}
