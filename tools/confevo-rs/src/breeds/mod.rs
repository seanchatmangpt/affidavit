//! Cognitive **breeds**: wasm4pm's "Old-AI" symbolic reasoners, ported as
//! zero-dependency Rust and wired to the feature space.
//!
//! [wasm4pm](https://github.com/seanchatmangpt/wasm4pm) ships 39 classical-AI
//! reasoning systems ("breeds") behind `wpm cognition`, each invoked as a
//! **contract**: a JSON *intent* (facts, goals, rules) in, a JSON *result*
//! (verdict, explanation, inference trace) out. confevo ports the two breeds that
//! directly *generate configurations* by reasoning over constraints:
//!
//! * [`SAT_CDCL`] — `sat_cdcl`: Boolean satisfiability (DPLL with unit propagation
//!   and conflict-driven backtracking). A Cargo feature is a boolean variable and
//!   an implication `a = ["b"]` is the clause `¬a ∨ b`, so finding a valid
//!   configuration *is* a SAT instance.
//! * [`CSP_AC3`] — `csp_ac3`: constraint satisfaction via AC-3 arc consistency plus
//!   backtracking search over finite domains.
//!
//! ## Semantic, not numeric
//!
//! The genetic algorithm in [`crate::evolve`] is a *numeric* optimizer: it samples
//! bit-vectors and climbs a fitness *number*, with no understanding of what a
//! feature means. A breed is *semantic*: it reasons over the logical structure of
//! the feature graph. The payoff is conclusiveness — a breed can hand back a
//! **model** (a provably valid configuration) or a **refutation** (a proof that no
//! configuration can satisfy your requirements, e.g. "you cannot enable
//! `predictive` without the forbidden `core`"). The GA can only ever report a low
//! score.
//!
//! ## Determinism
//!
//! Like the rest of confevo, breeds are bit-for-bit reproducible: variable and
//! value orderings are fixed, and `run_id`/`output_hash` are content hashes (no
//! wall-clock, no randomness). The same contract always yields the same result.
//!
//! See [`encode`] for the bridge from a [`crate::FeatureSpace`] to a contract and
//! back to a [`crate::Genome`].

mod csp;
pub mod encode;
mod json;
mod sat;

pub use csp::CspAc3;
pub use sat::SatCdcl;

/// One `{key, value}` entry in a contract's `facts` list.
///
/// Facts are how a breed receives its problem (clauses, variables, constraints,
/// goals) and how it reports structured pieces of its answer (assignments).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// The fact's discriminator (e.g. `"clause:0"`, `"csp-var"`, `"assign"`).
    pub key: String,
    /// The fact's payload, in the breed's own mini-syntax (e.g. `"1 -2"`).
    pub value: String,
}

impl Fact {
    /// Construct a fact from any string-like key and value.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Fact {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// A cognition **contract**: the input handed to a breed.
///
/// This mirrors wasm4pm's `intent.json` shape. confevo only reads `intent` and
/// `facts` (the other contract arrays — `candidates`, `cases`, `rules`, `goals`,
/// `state` — are accepted and ignored), which is all the implemented breeds need.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contract {
    /// A human-readable statement of the problem (echoed into the result).
    pub intent: String,
    /// The problem facts (clauses / variables / constraints).
    pub facts: Vec<Fact>,
}

impl Contract {
    /// Build a contract from an intent string and a list of facts.
    pub fn new(intent: impl Into<String>, facts: Vec<Fact>) -> Self {
        Contract {
            intent: intent.into(),
            facts,
        }
    }

    /// Parse a contract from `intent.json` text (wasm4pm contract format).
    ///
    /// Reads the `intent` string and the `facts` array of `{key, value}` objects;
    /// any other fields present in the document are ignored. Returns a description
    /// of the first structural problem on malformed input.
    pub fn from_json(text: &str) -> Result<Contract, String> {
        let v = json::parse(text)?;
        let intent = v
            .get("intent")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let mut facts = Vec::new();
        if let Some(arr) = v.get("facts").and_then(|x| x.as_array()) {
            for (i, f) in arr.iter().enumerate() {
                let key = f
                    .get("key")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| format!("facts[{i}] missing string \"key\""))?;
                let value = f
                    .get("value")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| format!("facts[{i}] missing string \"value\""))?;
                facts.push(Fact::new(key, value));
            }
        }
        Ok(Contract { intent, facts })
    }

    /// Iterate the values of every fact whose key equals `key`.
    fn values_for<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.facts
            .iter()
            .filter(move |f| f.key == key)
            .map(|f| f.value.as_str())
    }
}

/// One entry in a breed's [`BreedResult::inference_trace`].
///
/// The trace is the breed's "show your work": each step records what kind of
/// inference happened (`sat-propagate`, `csp-revise`, …), a human-readable detail,
/// and the search depth at which it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    /// Zero-based position in the trace.
    pub step: usize,
    /// The kind of inference (breed-specific tag).
    pub kind: String,
    /// A human-readable description of the step.
    pub detail: String,
    /// Search/recursion depth at which the step occurred.
    pub depth: usize,
}

/// An append-only inference trace, used by breeds to record their reasoning.
#[derive(Debug, Default)]
pub struct Trace {
    steps: Vec<TraceStep>,
}

impl Trace {
    /// A fresh, empty trace.
    pub fn new() -> Self {
        Trace { steps: Vec::new() }
    }

    /// Append a step; the step index is assigned automatically.
    pub fn push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) {
        let step = self.steps.len();
        self.steps.push(TraceStep {
            step,
            kind: kind.to_string(),
            detail: detail.into(),
            depth,
        });
    }

    /// Number of steps recorded so far.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// `true` if no steps have been recorded.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// The verdict a breed reaches about its contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The problem is satisfiable; a model is reported in the facts/explanation.
    Sat,
    /// The problem is unsatisfiable; the breed proved no solution exists.
    Unsat,
    /// The breed could not decide within its step budget.
    Unknown,
}

impl Verdict {
    /// The lowercase tag used in the result's `selected` field (`"sat"`, …).
    pub fn tag(self) -> &'static str {
        match self {
            Verdict::Sat => "sat",
            Verdict::Unsat => "unsat",
            Verdict::Unknown => "unknown",
        }
    }
}

/// The structured output of running a breed against a [`Contract`].
///
/// Mirrors wasm4pm's `result.json`: the breed name, an overall status, the
/// (echoed input plus solution) facts, the selected verdict, a human explanation,
/// the inference trace, and deterministic provenance fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreedResult {
    /// The breed's display name (e.g. `"SatCdcl"`).
    pub breed: String,
    /// Execution status; `"ok"` unless the contract was malformed.
    pub status: String,
    /// Input facts plus any solution facts the breed added.
    pub facts: Vec<Fact>,
    /// The verdict tag (`"sat"` / `"unsat"` / `"unknown"`).
    pub selected: String,
    /// A one-line, human-readable summary of the outcome.
    pub explanation: String,
    /// The step-by-step inference trace.
    pub inference_trace: Vec<TraceStep>,
    /// Deterministic run identifier (content hash of breed + intent).
    pub run_id: String,
    /// Deterministic content hash of the result's verdict + facts.
    pub output_hash: String,
    /// Number of inference steps (== `inference_trace.len()`).
    pub inference_step_count: usize,
    /// Number of clauses / constraints the breed evaluated.
    pub rules_evaluated: usize,
}

impl BreedResult {
    /// Assemble a result, computing the deterministic `run_id`/`output_hash`.
    #[allow(clippy::too_many_arguments)]
    fn finalize(
        breed: &str,
        intent: &str,
        verdict: Verdict,
        explanation: String,
        facts: Vec<Fact>,
        trace: Trace,
        rules_evaluated: usize,
    ) -> Self {
        let selected = verdict.tag().to_string();
        // run_id is a hash of *what was asked*; output_hash a hash of *the answer*.
        let run_id = format!("run-{}", fnv1a_hex(&format!("{breed}|{intent}")));
        let mut digest = String::new();
        digest.push_str(&selected);
        digest.push('|');
        digest.push_str(&explanation);
        for f in &facts {
            digest.push('|');
            digest.push_str(&f.key);
            digest.push('=');
            digest.push_str(&f.value);
        }
        BreedResult {
            breed: breed.to_string(),
            status: "ok".to_string(),
            facts,
            selected,
            explanation,
            inference_trace: trace.steps,
            run_id,
            output_hash: fnv1a_hex(&digest),
            inference_step_count: 0, // set below
            rules_evaluated,
        }
        .with_step_count()
    }

    fn with_step_count(mut self) -> Self {
        self.inference_step_count = self.inference_trace.len();
        self
    }

    /// `true` if the breed reported a satisfiable verdict.
    pub fn is_sat(&self) -> bool {
        self.selected == "sat"
    }

    /// Render this result as a JSON document in wasm4pm's `result.json` shape.
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"breed\": \"{}\",\n", esc(&self.breed)));
        s.push_str(&format!("  \"status\": \"{}\",\n", esc(&self.status)));
        s.push_str("  \"candidates\": [],\n");
        s.push_str("  \"facts\": [");
        s.push_str(&facts_json(&self.facts));
        s.push_str("],\n");
        s.push_str(&format!("  \"selected\": \"{}\",\n", esc(&self.selected)));
        s.push_str(&format!(
            "  \"explanation\": \"{}\",\n",
            esc(&self.explanation)
        ));
        s.push_str("  \"inference_trace\": [");
        let parts: Vec<String> = self
            .inference_trace
            .iter()
            .map(|t| {
                format!(
                    "\n    {{\"step\": {}, \"kind\": \"{}\", \"detail\": \"{}\", \"depth\": {}}}",
                    t.step,
                    esc(&t.kind),
                    esc(&t.detail),
                    t.depth
                )
            })
            .collect();
        s.push_str(&parts.join(","));
        if !self.inference_trace.is_empty() {
            s.push('\n');
            s.push_str("  ");
        }
        s.push_str("],\n");
        s.push_str(&format!("  \"run_id\": \"{}\",\n", esc(&self.run_id)));
        s.push_str(&format!(
            "  \"output_hash\": \"{}\",\n",
            esc(&self.output_hash)
        ));
        s.push_str(&format!(
            "  \"inference_step_count\": {},\n",
            self.inference_step_count
        ));
        s.push_str(&format!(
            "  \"rules_evaluated\": {}\n",
            self.rules_evaluated
        ));
        s.push_str("}\n");
        s
    }
}

fn facts_json(facts: &[Fact]) -> String {
    let parts: Vec<String> = facts
        .iter()
        .map(|f| {
            format!(
                "\n    {{\"key\": \"{}\", \"value\": \"{}\"}}",
                esc(&f.key),
                esc(&f.value)
            )
        })
        .collect();
    let joined = parts.join(",");
    if facts.is_empty() {
        joined
    } else {
        format!("{joined}\n  ")
    }
}

/// A symbolic reasoner that maps a [`Contract`] to a [`BreedResult`].
pub trait Breed {
    /// The breed's stable contract id (e.g. `"sat_cdcl"`).
    fn id(&self) -> &'static str;
    /// Run the breed against `contract`.
    fn run(&self, contract: &Contract) -> BreedResult;
}

/// The `sat_cdcl` breed instance.
pub const SAT_CDCL: SatCdcl = SatCdcl;
/// The `csp_ac3` breed instance.
pub const CSP_AC3: CspAc3 = CspAc3;

/// The breed contract ids confevo implements.
pub fn supported_breeds() -> &'static [&'static str] {
    &["sat_cdcl", "csp_ac3"]
}

/// Run the breed named `id` against `contract` (the `wpm cognition run` analog).
///
/// Returns an error naming the supported breeds when `id` is not one confevo has
/// ported.
pub fn run_named(id: &str, contract: &Contract) -> Result<BreedResult, String> {
    match id {
        "sat_cdcl" => Ok(SAT_CDCL.run(contract)),
        "csp_ac3" => Ok(CSP_AC3.run(contract)),
        other => Err(format!(
            "unknown breed {other:?}; confevo implements: {}",
            supported_breeds().join(", ")
        )),
    }
}

/// FNV-1a 64-bit hash of `s`, rendered as 16 lowercase hex digits.
///
/// Used for the deterministic `run_id` / `output_hash` provenance fields so a
/// contract always produces an identical, reproducible result.
fn fnv1a_hex(s: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("{h:016x}")
}

/// Escape a string for embedding in a JSON document.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_id_and_output_hash_are_deterministic() {
        let c = Contract::new("decide x", vec![Fact::new("clause:0", "1")]);
        let r1 = run_named("sat_cdcl", &c).unwrap();
        let r2 = run_named("sat_cdcl", &c).unwrap();
        assert_eq!(r1.run_id, r2.run_id);
        assert_eq!(r1.output_hash, r2.output_hash);
        assert_eq!(r1.to_json(), r2.to_json());
    }

    #[test]
    fn unknown_breed_lists_supported() {
        let err = run_named("eliza", &Contract::default()).unwrap_err();
        assert!(err.contains("sat_cdcl"));
        assert!(err.contains("csp_ac3"));
    }

    #[test]
    fn result_json_is_balanced_and_has_keys() {
        let c = Contract::new("decide", vec![Fact::new("clause:0", "1 2")]);
        let j = run_named("sat_cdcl", &c).unwrap().to_json();
        assert_eq!(j.matches('{').count(), j.matches('}').count());
        assert_eq!(j.matches('[').count(), j.matches(']').count());
        for k in [
            "\"breed\"",
            "\"selected\"",
            "\"inference_trace\"",
            "\"output_hash\"",
        ] {
            assert!(j.contains(k), "missing {k}");
        }
    }

    #[test]
    fn contract_round_trips_through_json() {
        let text = r#"{"intent":"decide","facts":[{"key":"clause:0","value":"1 -2"}]}"#;
        let c = Contract::from_json(text).unwrap();
        assert_eq!(c.intent, "decide");
        assert_eq!(c.facts, vec![Fact::new("clause:0", "1 -2")]);
    }
}
