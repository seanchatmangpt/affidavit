//! The bridge between a [`FeatureSpace`] and the cognitive breeds.
//!
//! This module is what makes "semantic configuration generation" concrete. It
//! compiles a feature space plus a [`ConfigQuery`] (features you *require* and
//! features you *forbid*) into a breed [`Contract`], runs the chosen breed, and
//! decodes the resulting model back into a [`Genome`] — a provably valid
//! configuration, or a proof that none exists.
//!
//! ## The encoding
//!
//! Each feature becomes a boolean variable. The feature graph's implication edges
//! become logical constraints:
//!
//! | Feature-space fact | SAT (`sat_cdcl`) | CSP (`csp_ac3`) |
//! | --- | --- | --- |
//! | `a` enables `b` | clause `¬a ∨ b` | constraint `a => b` over `{off,on}` |
//! | require `f` | unit clause `f` | domain of `f` pinned to `{on}` |
//! | forbid `f` | unit clause `¬f` | domain of `f` pinned to `{off}` |
//!
//! Because implications are transitive, requiring a feature whose closure reaches a
//! forbidden one is **unsatisfiable** — and the breed proves it. This is the exact
//! shape of the affidavit situation: `predictive` transitively needs `core`, and
//! `core` pulls in the broken upstream, so `--require predictive --forbid core` has
//! no solution and the breed says so with a refutation, not a low score.

use std::collections::BTreeSet;

use super::{run_named, BreedResult, Contract, Fact};
use crate::genome::Genome;
use crate::space::FeatureSpace;

/// Which cognitive breed to reason with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// Boolean satisfiability (`sat_cdcl`).
    SatCdcl,
    /// Constraint satisfaction via AC-3 (`csp_ac3`).
    CspAc3,
}

impl Engine {
    /// Parse an engine from a breed id or short alias (`sat`, `csp`).
    pub fn from_id(s: &str) -> Option<Engine> {
        match s {
            "sat_cdcl" | "sat" => Some(Engine::SatCdcl),
            "csp_ac3" | "csp" => Some(Engine::CspAc3),
            _ => None,
        }
    }

    /// The breed contract id this engine dispatches to.
    pub fn breed_id(self) -> &'static str {
        match self {
            Engine::SatCdcl => "sat_cdcl",
            Engine::CspAc3 => "csp_ac3",
        }
    }
}

/// A request to generate a configuration: features that must be on, and off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigQuery {
    /// Features that must be enabled in the generated configuration.
    pub require: Vec<String>,
    /// Features that must not be enabled (directly or by implication).
    pub forbid: Vec<String>,
}

impl ConfigQuery {
    /// An empty query (any valid configuration is acceptable).
    pub fn new() -> Self {
        ConfigQuery::default()
    }

    /// Add a required feature (builder style).
    pub fn require(mut self, f: impl Into<String>) -> Self {
        self.require.push(f.into());
        self
    }

    /// Add a forbidden feature (builder style).
    pub fn forbid(mut self, f: impl Into<String>) -> Self {
        self.forbid.push(f.into());
        self
    }
}

/// The outcome of semantic configuration generation.
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// The breed contract id that produced this result.
    pub engine_id: String,
    /// `true` if a valid configuration exists for the query.
    pub feasible: bool,
    /// The generated configuration, when feasible.
    pub genome: Option<Genome>,
    /// Forbidden features that the requirements transitively force on (the
    /// structural reason an infeasible query fails). Empty when feasible.
    pub clash: Vec<String>,
    /// A one-line, human-readable explanation of the outcome.
    pub explanation: String,
    /// The full breed contract result (verdict, trace, provenance).
    pub result: BreedResult,
}

/// Validate that every named feature exists in `space`.
fn check_known(space: &FeatureSpace, names: &[String], role: &str) -> Result<(), String> {
    for n in names {
        if !space.contains(n) {
            return Err(format!("{role} feature {n:?} is not in the feature space"));
        }
    }
    Ok(())
}

/// Encode `space` + `query` as a `sat_cdcl` contract.
///
/// Variable `i+1` corresponds to `space.features()[i]`. Implication edges become
/// binary clauses and the query becomes unit clauses.
pub fn encode_sat(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> {
    check_known(space, &query.require, "required")?;
    check_known(space, &query.forbid, "forbidden")?;

    let names = space.features();
    let idx = |f: &str| names.iter().position(|x| x == f).map(|p| (p + 1) as i32);

    let mut facts = Vec::new();
    let mut n = 0usize;
    for f in names {
        let a = idx(f).expect("feature is in space");
        for t in space.implications_of(f) {
            let b = idx(t).expect("edge target is in space");
            // a enables t  ⇔  ¬a ∨ t
            facts.push(Fact::new(format!("clause:{n}"), format!("{} {}", -a, b)));
            n += 1;
        }
    }
    for f in &query.require {
        let a = idx(f).expect("checked");
        facts.push(Fact::new(format!("clause:{n}"), format!("{a}")));
        n += 1;
    }
    for f in &query.forbid {
        let a = idx(f).expect("checked");
        facts.push(Fact::new(format!("clause:{n}"), format!("{}", -a)));
        n += 1;
    }
    facts.push(Fact::new("vars", names.len().to_string()));

    Ok(Contract::new(query_intent(query, "sat_cdcl"), facts))
}

/// Encode `space` + `query` as a `csp_ac3` contract.
///
/// Each feature is a variable over `{off, on}` (off first → minimal configs).
/// Implications become `=>` constraints; the query pins domains. A feature both
/// required and forbidden gets a self-`!=` constraint, which AC-3 reduces to the
/// empty domain — a clean unsatisfiability proof.
pub fn encode_csp(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> {
    check_known(space, &query.require, "required")?;
    check_known(space, &query.forbid, "forbidden")?;

    let require: BTreeSet<&str> = query.require.iter().map(String::as_str).collect();
    let forbid: BTreeSet<&str> = query.forbid.iter().map(String::as_str).collect();

    let mut facts = Vec::new();
    for f in space.features() {
        let domain = match (require.contains(f.as_str()), forbid.contains(f.as_str())) {
            (_, true) => "off", // forbid pins off (require+forbid handled below)
            (true, false) => "on",
            (false, false) => "off,on",
        };
        facts.push(Fact::new("csp-var", format!("{f}:{domain}")));
    }
    for f in space.features() {
        for t in space.implications_of(f) {
            facts.push(Fact::new("csp-constraint", format!("{f}=>{t}")));
        }
    }
    // Direct require∧forbid contradiction: an always-false self constraint.
    for f in &query.require {
        if forbid.contains(f.as_str()) {
            facts.push(Fact::new("csp-constraint", format!("{f}!={f}")));
        }
    }

    Ok(Contract::new(query_intent(query, "csp_ac3"), facts))
}

fn query_intent(query: &ConfigQuery, breed: &str) -> String {
    format!(
        "generate Cargo feature configuration via {breed} (require: [{}], forbid: [{}])",
        query.require.join(", "),
        query.forbid.join(", ")
    )
}

/// Generate a configuration for `space` satisfying `query`, using `engine`.
///
/// Runs the chosen breed and decodes its model into a [`Genome`] when feasible.
/// The returned [`SemanticConfig`] always carries the full breed [`BreedResult`]
/// (verdict + inference trace) so the reasoning is auditable.
pub fn generate_config(
    space: &FeatureSpace,
    query: &ConfigQuery,
    engine: Engine,
) -> Result<SemanticConfig, String> {
    let contract = match engine {
        Engine::SatCdcl => encode_sat(space, query)?,
        Engine::CspAc3 => encode_csp(space, query)?,
    };
    let result = run_named(engine.breed_id(), &contract)?;

    // Structural diagnostic, independent of the solver: which forbidden features
    // does the requirement set transitively force on?
    let req_set: BTreeSet<String> = query.require.iter().cloned().collect();
    let closure = space.closure(&req_set);
    let mut clash: Vec<String> = query
        .forbid
        .iter()
        .filter(|f| closure.contains(*f))
        .cloned()
        .collect();
    clash.sort();
    clash.dedup();

    let feasible = result.is_sat();
    let (genome, explanation) = if feasible {
        let g = decode(space, engine, &result);
        let feats = g.features();
        let label = if feats.is_empty() {
            "<none>".to_string()
        } else {
            feats.join(",")
        };
        (
            Some(g),
            format!("feasible: a valid configuration is `{label}`"),
        )
    } else {
        let why = if clash.is_empty() {
            "the requirements are mutually unsatisfiable".to_string()
        } else {
            format!(
                "required features transitively enable forbidden {} — no configuration can avoid it",
                clash.join(", ")
            )
        };
        (None, format!("infeasible: {why}"))
    };

    Ok(SemanticConfig {
        engine_id: engine.breed_id().to_string(),
        feasible,
        genome,
        clash,
        explanation,
        result,
    })
}

/// Decode a satisfiable breed result into the genome it represents.
fn decode(space: &FeatureSpace, engine: Engine, result: &BreedResult) -> Genome {
    let names = space.features();
    let mut feats: BTreeSet<String> = BTreeSet::new();
    for f in &result.facts {
        if f.key != "assign" {
            continue;
        }
        match engine {
            Engine::SatCdcl => {
                // "xN=1" ⇒ feature names[N-1] enabled.
                if let Some((var, val)) = f.value.split_once('=') {
                    if val == "1" {
                        if let Some(n) = var.strip_prefix('x').and_then(|s| s.parse::<usize>().ok())
                        {
                            if n >= 1 && n <= names.len() {
                                feats.insert(names[n - 1].clone());
                            }
                        }
                    }
                }
            }
            Engine::CspAc3 => {
                // "feat=on" ⇒ feature enabled.
                if let Some((name, val)) = f.value.split_once('=') {
                    if val == "on" && space.contains(name) {
                        feats.insert(name.to_string());
                    }
                }
            }
        }
    }
    // Return the canonical (closure-consistent) genome.
    Genome::new(feats).canonical(space)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space() -> FeatureSpace {
        // predictive -> conformance -> discovery -> core ; metrics -> otel
        FeatureSpace::new(
            [
                "core",
                "discovery",
                "conformance",
                "predictive",
                "otel",
                "metrics",
                "ui",
            ],
            [
                ("discovery", vec!["core"]),
                ("conformance", vec!["discovery"]),
                ("predictive", vec!["conformance"]),
                ("metrics", vec!["otel"]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn sat_generates_a_valid_config_respecting_implications() {
        let s = space();
        let q = ConfigQuery::new().require("predictive");
        let cfg = generate_config(&s, &q, Engine::SatCdcl).unwrap();
        assert!(cfg.feasible);
        let g = cfg.genome.unwrap();
        // Requiring predictive must pull in its whole implication chain.
        for f in ["predictive", "conformance", "discovery", "core"] {
            assert!(g.contains(f), "expected {f} enabled");
        }
    }

    #[test]
    fn csp_generates_a_valid_config_respecting_implications() {
        let s = space();
        let q = ConfigQuery::new().require("metrics");
        let cfg = generate_config(&s, &q, Engine::CspAc3).unwrap();
        assert!(cfg.feasible);
        let g = cfg.genome.unwrap();
        assert!(g.contains("metrics"));
        assert!(g.contains("otel"));
    }

    #[test]
    fn require_predictive_forbid_core_is_provably_infeasible_sat() {
        let s = space();
        let q = ConfigQuery::new().require("predictive").forbid("core");
        let cfg = generate_config(&s, &q, Engine::SatCdcl).unwrap();
        assert!(!cfg.feasible);
        assert_eq!(cfg.result.selected, "unsat");
        assert_eq!(cfg.clash, vec!["core".to_string()]);
        assert!(cfg.explanation.contains("forbidden core"));
    }

    #[test]
    fn require_predictive_forbid_core_is_provably_infeasible_csp() {
        let s = space();
        let q = ConfigQuery::new().require("predictive").forbid("core");
        let cfg = generate_config(&s, &q, Engine::CspAc3).unwrap();
        assert!(!cfg.feasible);
        assert_eq!(cfg.result.selected, "unsat");
        assert_eq!(cfg.clash, vec!["core".to_string()]);
    }

    #[test]
    fn both_engines_agree_on_feasibility() {
        let s = space();
        for q in [
            ConfigQuery::new(),
            ConfigQuery::new().require("ui"),
            ConfigQuery::new().require("predictive").forbid("core"),
            ConfigQuery::new().require("metrics").forbid("otel"),
        ] {
            let sat = generate_config(&s, &q, Engine::SatCdcl).unwrap();
            let csp = generate_config(&s, &q, Engine::CspAc3).unwrap();
            assert_eq!(
                sat.feasible, csp.feasible,
                "engines disagree on {q:?}: sat={} csp={}",
                sat.feasible, csp.feasible
            );
        }
    }

    #[test]
    fn empty_query_is_feasible_with_minimal_config() {
        let s = space();
        let cfg = generate_config(&s, &ConfigQuery::new(), Engine::SatCdcl).unwrap();
        assert!(cfg.feasible);
        // Minimal model (false-first) with no requirements ⇒ nothing enabled.
        assert!(cfg.genome.unwrap().is_empty());
    }

    #[test]
    fn unknown_feature_is_rejected() {
        let s = space();
        let q = ConfigQuery::new().require("nonexistent");
        assert!(generate_config(&s, &q, Engine::SatCdcl).is_err());
    }

    #[test]
    fn generation_is_deterministic() {
        let s = space();
        let q = ConfigQuery::new().require("conformance");
        let a = generate_config(&s, &q, Engine::SatCdcl).unwrap();
        let b = generate_config(&s, &q, Engine::SatCdcl).unwrap();
        assert_eq!(a.result.to_json(), b.result.to_json());
        assert_eq!(a.genome, b.genome);
    }
}
