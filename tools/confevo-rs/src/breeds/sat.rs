//! The `sat_cdcl` breed: Boolean satisfiability by DPLL with unit propagation and
//! conflict-driven backtracking.
//!
//! ## Contract
//!
//! The formula is given as `clause:*` facts in DIMACS-style notation: each fact's
//! value is a space-separated list of nonzero integers, where `k` is the positive
//! literal of variable `k` and `-k` its negation. An optional `vars` fact pins the
//! variable count (otherwise it is inferred from the largest literal). Example:
//!
//! ```text
//! { "key": "clause:0", "value": "1 2" }     # (x1 ∨ x2)
//! { "key": "clause:1", "value": "-1 2" }    # (¬x1 ∨ x2)
//! ```
//!
//! ## Why it matters for configuration generation
//!
//! A Cargo feature is a boolean variable; an implication `a = ["b"]` is exactly the
//! clause `¬a ∨ b`; a "must enable" / "must not enable" requirement is a unit
//! clause. So generating a valid feature configuration *is* solving a SAT instance
//! — and when requirements conflict through the implication graph, the solver
//! returns a **proof** of unsatisfiability rather than a merely-low score.
//!
//! ## Algorithm
//!
//! Classic DPLL: unit-propagate to a fixpoint (deriving forced assignments and
//! detecting conflicts), then pick the lowest-index unassigned variable and try
//! `false` before `true` (yielding *minimal* models — only what requirements
//! force gets enabled), backtracking on conflict. This is sound and complete; the
//! "CDCL" lineage shows up as the conflict-driven trace it records.

use super::{BreedResult, Contract, Trace, Verdict};

/// The `sat_cdcl` breed (see [module docs](self)).
#[derive(Debug, Clone, Copy)]
pub struct SatCdcl;

/// Guard against pathological search blow-ups; feature SAT instances are tiny, so
/// this is never hit in practice — it only bounds adversarial raw contracts.
const MAX_STEPS: usize = 2_000_000;

impl super::Breed for SatCdcl {
    fn id(&self) -> &'static str {
        "sat_cdcl"
    }

    fn run(&self, contract: &Contract) -> BreedResult {
        let parsed = parse_clauses(contract);
        let (clauses, num_vars) = match parsed {
            Ok(v) => v,
            Err(e) => {
                let mut t = Trace::new();
                t.push("sat-init", format!("contract error: {e}"), 0);
                return BreedResult::finalize(
                    "SatCdcl",
                    &contract.intent,
                    Verdict::Unknown,
                    format!("UNKNOWN: {e}"),
                    contract.facts.clone(),
                    t,
                    0,
                );
            }
        };

        let mut solver = Solver {
            clauses: &clauses,
            num_vars,
            trace: Trace::new(),
            steps: 0,
            aborted: false,
        };
        solver.trace.push(
            "sat-init",
            format!("{num_vars} vars, {} clauses", clauses.len()),
            0,
        );

        let assign = vec![None; num_vars + 1];
        let model = solver.dpll(assign, 0);

        let mut facts = contract.facts.clone();
        let (verdict, explanation) = if solver.aborted {
            solver.trace.push("sat-verdict", "step budget exhausted", 0);
            (
                Verdict::Unknown,
                "UNKNOWN: exceeded step budget".to_string(),
            )
        } else if let Some(model) = model {
            let mut parts = Vec::new();
            for (v, slot) in model.iter().enumerate().take(num_vars + 1).skip(1) {
                let b = slot.unwrap_or(false);
                facts.push(super::Fact::new("assign", format!("x{v}={}", b as u8)));
                parts.push(format!("x{v}={}", b as u8));
            }
            solver
                .trace
                .push("sat-verdict", "all variables assigned", 0);
            (Verdict::Sat, format!("SAT: {}", parts.join(", ")))
        } else {
            solver
                .trace
                .push("sat-verdict", "no satisfying assignment exists", 0);
            (
                Verdict::Unsat,
                "UNSAT: the clauses are mutually unsatisfiable".to_string(),
            )
        };

        let rules = clauses.len();
        BreedResult::finalize(
            "SatCdcl",
            &contract.intent,
            verdict,
            explanation,
            facts,
            solver.trace,
            rules,
        )
    }
}

/// A clause is a disjunction of literals (nonzero `i32`: sign = polarity).
type Clause = Vec<i32>;

struct Solver<'a> {
    clauses: &'a [Clause],
    num_vars: usize,
    trace: Trace,
    steps: usize,
    aborted: bool,
}

impl Solver<'_> {
    /// DPLL: propagate, then branch on the lowest unassigned variable.
    ///
    /// Returns the full model on success, or `None` if this subtree is
    /// unsatisfiable. Tries `false` before `true` so the model is minimal.
    fn dpll(&mut self, mut assign: Vec<Option<bool>>, depth: usize) -> Option<Vec<Option<bool>>> {
        if self.aborted {
            return None;
        }
        if self.propagate(&mut assign, depth).is_err() {
            return None;
        }
        let next = (1..=self.num_vars).find(|&v| assign[v].is_none());
        let var = match next {
            None => return Some(assign),
            Some(v) => v,
        };
        for &val in &[false, true] {
            if self.tick() {
                return None;
            }
            self.trace
                .push("sat-decide", format!("x{var} := {}", val as u8), depth);
            let mut branch = assign.clone();
            branch[var] = Some(val);
            if let Some(model) = self.dpll(branch, depth + 1) {
                return Some(model);
            }
            self.trace
                .push("sat-backtrack", format!("x{var} != {}", val as u8), depth);
        }
        None
    }

    /// Boolean constraint propagation: repeatedly assign forced unit literals until
    /// a fixpoint, returning `Err` on a falsified (conflict) clause.
    fn propagate(&mut self, assign: &mut [Option<bool>], depth: usize) -> Result<(), ()> {
        loop {
            if self.tick() {
                return Err(());
            }
            let mut progress = false;
            for (ci, clause) in self.clauses.iter().enumerate() {
                let mut satisfied = false;
                let mut unassigned: Option<i32> = None;
                let mut unassigned_count = 0usize;
                for &lit in clause {
                    let v = lit.unsigned_abs() as usize;
                    match assign[v] {
                        Some(b) => {
                            if (lit > 0) == b {
                                satisfied = true;
                                break;
                            }
                        }
                        None => {
                            unassigned = Some(lit);
                            unassigned_count += 1;
                        }
                    }
                }
                if satisfied {
                    continue;
                }
                if unassigned_count == 0 {
                    self.trace
                        .push("sat-conflict", format!("clause {ci} falsified"), depth);
                    return Err(());
                }
                if unassigned_count == 1 {
                    let lit = unassigned.expect("count == 1");
                    let v = lit.unsigned_abs() as usize;
                    assign[v] = Some(lit > 0);
                    self.trace.push(
                        "sat-propagate",
                        format!("unit clause {ci}: x{v} := {}", (lit > 0) as u8),
                        depth,
                    );
                    progress = true;
                }
            }
            if !progress {
                return Ok(());
            }
        }
    }

    /// Advance the step counter, flipping `aborted` if the budget is exceeded.
    fn tick(&mut self) -> bool {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            self.aborted = true;
        }
        self.aborted
    }
}

/// Parse `clause:*` facts into clauses and infer the variable count.
fn parse_clauses(contract: &Contract) -> Result<(Vec<Clause>, usize), String> {
    let mut clauses = Vec::new();
    let mut max_var = 0usize;
    for f in &contract.facts {
        if !f.key.starts_with("clause") {
            continue;
        }
        let mut clause = Vec::new();
        for tok in f.value.split_whitespace() {
            let lit: i32 = tok
                .parse()
                .map_err(|_| format!("clause {:?}: bad literal {tok:?}", f.key))?;
            if lit == 0 {
                continue; // DIMACS terminator, if present
            }
            max_var = max_var.max(lit.unsigned_abs() as usize);
            clause.push(lit);
        }
        if !clause.is_empty() {
            clauses.push(clause);
        } else {
            // An explicit empty clause is the canonical contradiction.
            clauses.push(Vec::new());
        }
    }
    // Allow an explicit `vars` override (must cover the literals seen).
    for v in contract.values_for("vars") {
        if let Ok(n) = v.trim().parse::<usize>() {
            max_var = max_var.max(n);
        }
    }
    if clauses.is_empty() {
        return Err("no clause:* facts found".to_string());
    }
    Ok((clauses, max_var))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breeds::{Breed, Contract, Fact};

    fn run(facts: Vec<Fact>) -> BreedResult {
        SatCdcl.run(&Contract::new("test", facts))
    }

    #[test]
    fn satisfiable_formula_is_sat() {
        // (x1 ∨ x2) ∧ (¬x1 ∨ x2): satisfiable (e.g. x1=0, x2=1).
        let r = run(vec![
            Fact::new("clause:0", "1 2"),
            Fact::new("clause:1", "-1 2"),
        ]);
        assert_eq!(r.selected, "sat");
        // x2 must be true in every model of this formula.
        assert!(r
            .facts
            .iter()
            .any(|f| f.key == "assign" && f.value == "x2=1"));
    }

    #[test]
    fn contradiction_is_unsat() {
        // (x1) ∧ (¬x1) is unsatisfiable; the breed must prove it.
        let r = run(vec![
            Fact::new("clause:0", "1"),
            Fact::new("clause:1", "-1"),
        ]);
        assert_eq!(r.selected, "unsat");
        assert!(r.explanation.contains("UNSAT"));
        assert!(r.inference_trace.iter().any(|t| t.kind == "sat-conflict"));
    }

    #[test]
    fn unit_propagation_forces_assignments() {
        // x1 ∧ (¬x1 ∨ x2) ∧ (¬x2 ∨ x3) ⇒ x1=x2=x3=1 by pure propagation.
        let r = run(vec![
            Fact::new("clause:0", "1"),
            Fact::new("clause:1", "-1 2"),
            Fact::new("clause:2", "-2 3"),
        ]);
        assert_eq!(r.selected, "sat");
        for v in ["x1=1", "x2=1", "x3=1"] {
            assert!(r.facts.iter().any(|f| f.key == "assign" && f.value == v));
        }
        assert!(r.inference_trace.iter().any(|t| t.kind == "sat-propagate"));
    }

    #[test]
    fn minimal_model_prefers_false() {
        // A lone variable with no constraints: false-first ⇒ x1=0.
        let r = run(vec![Fact::new("clause:0", "1 -1")]); // tautology
        assert_eq!(r.selected, "sat");
        assert!(r
            .facts
            .iter()
            .any(|f| f.key == "assign" && f.value == "x1=0"));
    }

    #[test]
    fn missing_clauses_is_unknown() {
        let r = SatCdcl.run(&Contract::new("empty", vec![]));
        assert_eq!(r.selected, "unknown");
    }

    #[test]
    fn breed_id_is_stable() {
        assert_eq!(SatCdcl.id(), "sat_cdcl");
    }
}
