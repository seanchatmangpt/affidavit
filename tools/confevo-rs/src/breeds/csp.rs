//! The `csp_ac3` breed: finite-domain constraint satisfaction by AC-3 arc
//! consistency followed by backtracking search.
//!
//! ## Contract
//!
//! Variables and constraints are given as facts:
//!
//! ```text
//! { "key": "csp-var",        "value": "V1:R,G,B" }   # V1 ∈ {R, G, B}
//! { "key": "csp-var",        "value": "V2:R,G,B" }
//! { "key": "csp-constraint", "value": "V1!=V2" }      # V1 ≠ V2
//! ```
//!
//! Supported binary operators are `!=`, `==`, `<`, `>`, `<=`, `>=` (numeric when
//! both operands parse as numbers, lexicographic otherwise) and `=>` (implication
//! over the `off`/`on` truth tokens, which is how a Cargo feature graph is encoded:
//! `a => b` forbids the pair `a=on, b=off`).
//!
//! ## Algorithm
//!
//! AC-3 first prunes each variable's domain to arc-consistent values (recording a
//! `csp-revise` step whenever it removes one); if any domain empties, the problem
//! is unsatisfiable and the breed says so. Otherwise a deterministic backtracking
//! search (variables in declared order, values in domain order) finds a complete
//! consistent assignment — the generated configuration.

use std::collections::VecDeque;

use super::{BreedResult, Contract, Fact, Trace, Verdict};

/// The `csp_ac3` breed (see [module docs](self)).
#[derive(Debug, Clone, Copy)]
pub struct CspAc3;

impl super::Breed for CspAc3 {
    fn id(&self) -> &'static str {
        "csp_ac3"
    }

    fn run(&self, contract: &Contract) -> BreedResult {
        let mut problem = match Problem::parse(contract) {
            Ok(p) => p,
            Err(e) => {
                let mut t = Trace::new();
                t.push("csp-init", format!("contract error: {e}"), 0);
                return BreedResult::finalize(
                    "CspAc3",
                    &contract.intent,
                    Verdict::Unknown,
                    format!("UNKNOWN: {e}"),
                    contract.facts.clone(),
                    t,
                    0,
                );
            }
        };

        let mut trace = Trace::new();
        trace.push(
            "csp-init",
            format!(
                "{} vars, {} constraints",
                problem.vars.len(),
                problem.constraints.len()
            ),
            0,
        );

        let consistent = problem.ac3(&mut trace);
        let rules = problem.constraints.len();

        if !consistent {
            trace.push("csp-verdict", "a domain emptied under AC-3", 0);
            return BreedResult::finalize(
                "CspAc3",
                &contract.intent,
                Verdict::Unsat,
                "UNSAT: arc consistency emptied a domain".to_string(),
                contract.facts.clone(),
                trace,
                rules,
            );
        }

        match problem.search(&mut trace) {
            Some(assignment) => {
                let mut facts = contract.facts.clone();
                let mut parts = Vec::new();
                for (i, name) in problem.names.iter().enumerate() {
                    let val = &assignment[i];
                    facts.push(Fact::new("assign", format!("{name}={val}")));
                    parts.push(format!("{name}={val}"));
                }
                trace.push("csp-verdict", "complete consistent assignment", 0);
                BreedResult::finalize(
                    "CspAc3",
                    &contract.intent,
                    Verdict::Sat,
                    format!("SAT: {}", parts.join(", ")),
                    facts,
                    trace,
                    rules,
                )
            }
            None => {
                trace.push("csp-verdict", "search exhausted with no assignment", 0);
                BreedResult::finalize(
                    "CspAc3",
                    &contract.intent,
                    Verdict::Unsat,
                    "UNSAT: no complete consistent assignment exists".to_string(),
                    contract.facts.clone(),
                    trace,
                    rules,
                )
            }
        }
    }
}

/// A binary constraint operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Ne,
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
    /// Implication over `off`/`on`: forbids `left=on, right=off`.
    Imp,
}

impl Op {
    /// `true` if `(l, r)` satisfies this operator.
    fn holds(self, l: &str, r: &str) -> bool {
        match self {
            Op::Eq => l == r,
            Op::Ne => l != r,
            Op::Imp => !(l == "on" && r == "off"),
            Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                let ord = match (l.parse::<f64>(), r.parse::<f64>()) {
                    (Ok(a), Ok(b)) => a.partial_cmp(&b),
                    _ => Some(l.cmp(r)),
                };
                match (self, ord) {
                    (Op::Lt, Some(o)) => o == std::cmp::Ordering::Less,
                    (Op::Gt, Some(o)) => o == std::cmp::Ordering::Greater,
                    (Op::Le, Some(o)) => o != std::cmp::Ordering::Greater,
                    (Op::Ge, Some(o)) => o != std::cmp::Ordering::Less,
                    _ => false,
                }
            }
        }
    }
}

/// A parsed binary constraint between two variable indices.
struct Constraint {
    left: usize,
    right: usize,
    op: Op,
}

struct Problem {
    names: Vec<String>,
    vars: Vec<Vec<String>>,
    constraints: Vec<Constraint>,
}

impl Problem {
    fn parse(contract: &Contract) -> Result<Problem, String> {
        let mut names = Vec::new();
        let mut vars = Vec::new();
        for v in contract.values_for("csp-var") {
            let (name, domain) = v
                .split_once(':')
                .ok_or_else(|| format!("csp-var {v:?} must be NAME:d1,d2,..."))?;
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(format!("csp-var {v:?} has an empty name"));
            }
            let domain: Vec<String> = domain
                .split(',')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect();
            if domain.is_empty() {
                return Err(format!("csp-var {name:?} has an empty domain"));
            }
            names.push(name);
            vars.push(domain);
        }
        if names.is_empty() {
            return Err("no csp-var facts found".to_string());
        }

        let index = |n: &str| names.iter().position(|x| x == n);
        let mut constraints = Vec::new();
        for c in contract.values_for("csp-constraint") {
            let (l, op, r) = split_constraint(c)?;
            let li = index(l).ok_or_else(|| format!("constraint {c:?}: unknown var {l:?}"))?;
            let ri = index(r).ok_or_else(|| format!("constraint {c:?}: unknown var {r:?}"))?;
            constraints.push(Constraint {
                left: li,
                right: ri,
                op,
            });
        }

        Ok(Problem {
            names,
            vars,
            constraints,
        })
    }

    /// AC-3: prune domains to arc consistency. Returns `false` if a domain empties.
    fn ac3(&mut self, trace: &mut Trace) -> bool {
        // Each arc names the variable to revise (`x`), its partner (`y`), the
        // constraint, and whether `x` is the constraint's left operand.
        let mut queue: VecDeque<(usize, usize, usize, bool)> = VecDeque::new();
        for (ci, c) in self.constraints.iter().enumerate() {
            queue.push_back((c.left, c.right, ci, true));
            queue.push_back((c.right, c.left, ci, false));
        }

        while let Some((x, y, ci, x_is_left)) = queue.pop_front() {
            if self.revise(x, y, ci, x_is_left, trace) {
                if self.vars[x].is_empty() {
                    return false;
                }
                // Re-enqueue arcs pointing into x from its other neighbors.
                for (cj, c) in self.constraints.iter().enumerate() {
                    if c.left == x && c.right != y {
                        queue.push_back((c.right, x, cj, false));
                    } else if c.right == x && c.left != y {
                        queue.push_back((c.left, x, cj, true));
                    }
                }
            }
        }
        true
    }

    /// Remove from `D(x)` every value with no support in `D(y)`; return whether any
    /// value was removed.
    fn revise(
        &mut self,
        x: usize,
        y: usize,
        ci: usize,
        x_is_left: bool,
        trace: &mut Trace,
    ) -> bool {
        let op = self.constraints[ci].op;
        let y_domain = self.vars[y].clone();
        let mut removed = false;
        let mut kept = Vec::with_capacity(self.vars[x].len());
        for vx in &self.vars[x] {
            let supported = y_domain.iter().any(|vy| {
                if x_is_left {
                    op.holds(vx, vy)
                } else {
                    op.holds(vy, vx)
                }
            });
            if supported {
                kept.push(vx.clone());
            } else {
                removed = true;
                trace.push(
                    "csp-revise",
                    format!(
                        "{}: drop {vx:?} (no support in {})",
                        self.names[x], self.names[y]
                    ),
                    0,
                );
            }
        }
        if removed {
            self.vars[x] = kept;
        }
        removed
    }

    /// Backtracking search for a complete consistent assignment over the pruned
    /// domains. Variables are tried in declared order, values in domain order.
    fn search(&self, trace: &mut Trace) -> Option<Vec<String>> {
        let mut assignment: Vec<String> = vec![String::new(); self.vars.len()];
        if self.backtrack(0, &mut assignment, trace) {
            Some(assignment)
        } else {
            None
        }
    }

    fn backtrack(&self, idx: usize, assignment: &mut [String], trace: &mut Trace) -> bool {
        if idx == self.vars.len() {
            return true;
        }
        for val in &self.vars[idx] {
            if self.consistent(idx, val, assignment) {
                assignment[idx] = val.clone();
                trace.push("csp-assign", format!("{}={val}", self.names[idx]), idx);
                if self.backtrack(idx + 1, assignment, trace) {
                    return true;
                }
            }
        }
        assignment[idx] = String::new();
        false
    }

    /// `true` if assigning `val` to variable `idx` violates no constraint against
    /// the already-assigned variables (those with index `< idx`).
    fn consistent(&self, idx: usize, val: &str, assignment: &[String]) -> bool {
        for c in &self.constraints {
            let (other, other_is_left) = if c.left == idx {
                (c.right, false)
            } else if c.right == idx {
                (c.left, true)
            } else {
                continue;
            };
            if other >= idx {
                continue; // not yet assigned
            }
            let ov = &assignment[other];
            let ok = if other_is_left {
                c.op.holds(ov, val)
            } else {
                c.op.holds(val, ov)
            };
            if !ok {
                return false;
            }
        }
        true
    }
}

/// Split a constraint string like `"V1!=V2"` into `(left, op, right)`.
fn split_constraint(c: &str) -> Result<(&str, Op, &str), String> {
    // Two-character operators must be checked before single-character ones.
    for (sym, op) in [
        ("<=", Op::Le),
        (">=", Op::Ge),
        ("!=", Op::Ne),
        ("==", Op::Eq),
        ("=>", Op::Imp),
    ] {
        if let Some((l, r)) = c.split_once(sym) {
            return Ok((l.trim(), op, r.trim()));
        }
    }
    for (sym, op) in [("<", Op::Lt), (">", Op::Gt)] {
        if let Some((l, r)) = c.split_once(sym) {
            return Ok((l.trim(), op, r.trim()));
        }
    }
    Err(format!(
        "constraint {c:?}: expected one of != == <= >= < > =>"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breeds::{Breed, Contract, Fact};

    fn run(facts: Vec<Fact>) -> BreedResult {
        CspAc3.run(&Contract::new("test", facts))
    }

    #[test]
    fn graph_coloring_is_sat() {
        // Two adjacent nodes, three colors, must differ.
        let r = run(vec![
            Fact::new("csp-var", "V1:R,G,B"),
            Fact::new("csp-var", "V2:R,G,B"),
            Fact::new("csp-constraint", "V1!=V2"),
        ]);
        assert_eq!(r.selected, "sat");
        let v1 = r
            .facts
            .iter()
            .find(|f| f.key == "assign" && f.value.starts_with("V1="))
            .unwrap();
        let v2 = r
            .facts
            .iter()
            .find(|f| f.key == "assign" && f.value.starts_with("V2="))
            .unwrap();
        assert_ne!(v1.value, v2.value);
    }

    #[test]
    fn over_constrained_is_unsat_via_ac3() {
        // Three mutually-distinct vars over a two-color domain: impossible.
        let r = run(vec![
            Fact::new("csp-var", "A:R,G"),
            Fact::new("csp-var", "B:R,G"),
            Fact::new("csp-var", "C:R,G"),
            Fact::new("csp-constraint", "A!=B"),
            Fact::new("csp-constraint", "B!=C"),
            Fact::new("csp-constraint", "A!=C"),
        ]);
        assert_eq!(r.selected, "unsat");
    }

    #[test]
    fn equality_singleton_collapses_domains() {
        let r = run(vec![
            Fact::new("csp-var", "X:1,2,3"),
            Fact::new("csp-var", "Y:2"),
            Fact::new("csp-constraint", "X==Y"),
        ]);
        assert_eq!(r.selected, "sat");
        assert!(r
            .facts
            .iter()
            .any(|f| f.key == "assign" && f.value == "X=2"));
    }

    #[test]
    fn implication_over_off_on_forces_consequent() {
        // a=on requires b=on; with a pinned on, b cannot be off.
        let r = run(vec![
            Fact::new("csp-var", "a:on"),
            Fact::new("csp-var", "b:off,on"),
            Fact::new("csp-constraint", "a=>b"),
        ]);
        assert_eq!(r.selected, "sat");
        assert!(r
            .facts
            .iter()
            .any(|f| f.key == "assign" && f.value == "b=on"));
    }

    #[test]
    fn implication_conflict_is_unsat() {
        // a=on but b forced off contradicts a=>b.
        let r = run(vec![
            Fact::new("csp-var", "a:on"),
            Fact::new("csp-var", "b:off"),
            Fact::new("csp-constraint", "a=>b"),
        ]);
        assert_eq!(r.selected, "unsat");
    }

    #[test]
    fn records_revise_and_assign_steps() {
        let r = run(vec![
            Fact::new("csp-var", "V1:R,G,B"),
            Fact::new("csp-var", "V2:R"),
            Fact::new("csp-constraint", "V1!=V2"),
        ]);
        assert!(r.inference_trace.iter().any(|t| t.kind == "csp-revise"));
        assert!(r.inference_trace.iter().any(|t| t.kind == "csp-assign"));
    }

    #[test]
    fn breed_id_is_stable() {
        assert_eq!(CspAc3.id(), "csp_ac3");
    }
}
