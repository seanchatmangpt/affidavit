//! Token-replay conformance checking against a directly-follows model.
//!
//! Given a model discovered as a [`DirectlyFollowsGraph`], [`replay`] walks a
//! [`Trace`] step by step: every consecutive activity pair `(a, b)` must be a
//! directly-follows edge in the model, the first activity must be a model start,
//! and the last a model end. The result is a [`ConformanceResult`] carrying a
//! replay-fitness ratio plus the boundary checks, from which a
//! [`ConformanceVerdict`] follows.
//!
//! Doctrine, preserved: this *certifies* a trace against a discovered model; it
//! never decides whether the recorded work was honest.

use crate::mining::{DirectlyFollowsGraph, Trace};

/// Verdict of conformance checking a trace against a process model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConformanceVerdict {
    /// Every move replayed legally and the start/end activities are allowed.
    Conformant,
    /// At least one move, start, or end did not fit the model.
    NonConformant,
}

/// The result of replaying one trace against a reference model.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConformanceResult {
    /// Number of consecutive (a,b) steps that are edges in the model.
    pub legal_moves: u64,
    /// Total consecutive steps in the trace (= len - 1, or 0).
    pub total_moves: u64,
    /// Whether the trace's first activity is a model start activity (vacuously true if empty).
    pub start_ok: bool,
    /// Whether the trace's last activity is a model end activity (vacuously true if empty).
    pub end_ok: bool,
    /// Index in the trace of the first illegal step (a,b), if any.
    pub first_violation: Option<usize>,
}

impl ConformanceResult {
    /// Replay fitness = legal_moves / total_moves (1.0 when total_moves == 0).
    pub fn fitness(&self) -> f64 {
        if self.total_moves == 0 {
            1.0
        } else {
            self.legal_moves as f64 / self.total_moves as f64
        }
    }

    /// True iff legal_moves == total_moves AND start_ok AND end_ok.
    pub fn is_conformant(&self) -> bool {
        self.legal_moves == self.total_moves && self.start_ok && self.end_ok
    }

    /// The verdict derived from `is_conformant`.
    pub fn verdict(&self) -> ConformanceVerdict {
        if self.is_conformant() {
            ConformanceVerdict::Conformant
        } else {
            ConformanceVerdict::NonConformant
        }
    }
}

/// Replay `trace` against `model` via token-replay: each consecutive activity
/// pair must be a directly-follows edge in the model, the first activity must be
/// a model start, and the last a model end.
pub fn replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) -> ConformanceResult {
    let acts = &trace.activities;
    let total_moves = acts.len().saturating_sub(1) as u64;

    let mut legal_moves = 0u64;
    let mut first_violation = None;
    for i in 0..acts.len().saturating_sub(1) {
        if model.follows(acts[i], acts[i + 1]) {
            legal_moves += 1;
        } else if first_violation.is_none() {
            first_violation = Some(i);
        }
    }

    let start_ok = match acts.first() {
        Some(first) => model.is_start(first),
        None => true,
    };
    let end_ok = match acts.last() {
        Some(last) => model.is_end(last),
        None => true,
    };

    ConformanceResult {
        legal_moves,
        total_moves,
        start_ok,
        end_ok,
        first_violation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> DirectlyFollowsGraph {
        DirectlyFollowsGraph::discover(&[Trace::from_activities(&["a", "b", "c"])])
    }

    #[test]
    fn conformant_trace_replays_fully() {
        let m = model();
        let result = replay(&m, &Trace::from_activities(&["a", "b", "c"]));
        assert_eq!(result.legal_moves, 2);
        assert_eq!(result.total_moves, 2);
        assert_eq!(result.first_violation, None);
        assert!(result.start_ok);
        assert!(result.end_ok);
        assert!((result.fitness() - 1.0).abs() < 1e-9);
        assert!(result.is_conformant());
        assert_eq!(result.verdict(), ConformanceVerdict::Conformant);
    }

    #[test]
    fn illegal_move_is_flagged_but_boundaries_hold() {
        // a > c is not an edge; a is a start, c is an end.
        let m = model();
        let result = replay(&m, &Trace::from_activities(&["a", "c"]));
        assert_eq!(result.legal_moves, 0);
        assert_eq!(result.total_moves, 1);
        assert_eq!(result.first_violation, Some(0));
        assert!(result.start_ok);
        assert!(result.end_ok);
        assert!(!result.is_conformant());
        assert_eq!(result.verdict(), ConformanceVerdict::NonConformant);
    }

    #[test]
    fn legal_move_with_bad_start_is_nonconformant() {
        // (b,c) is a legal edge, but b is not a model start.
        let m = model();
        let result = replay(&m, &Trace::from_activities(&["b", "c"]));
        assert_eq!(result.legal_moves, 1);
        assert_eq!(result.total_moves, 1);
        assert_eq!(result.first_violation, None);
        assert!(!result.start_ok);
        assert!(result.end_ok);
        assert!(!result.is_conformant());
        assert_eq!(result.verdict(), ConformanceVerdict::NonConformant);
    }

    #[test]
    fn single_activity_has_no_moves_but_bad_end() {
        // [a]: zero moves (vacuously full fitness), a is a start but not an end.
        let m = model();
        let result = replay(&m, &Trace::from_activities(&["a"]));
        assert_eq!(result.legal_moves, 0);
        assert_eq!(result.total_moves, 0);
        assert_eq!(result.first_violation, None);
        assert!(result.start_ok);
        assert!(!result.end_ok);
        assert!((result.fitness() - 1.0).abs() < 1e-9);
        assert!(!result.is_conformant());
        assert_eq!(result.verdict(), ConformanceVerdict::NonConformant);
    }

    #[test]
    fn empty_trace_is_vacuously_conformant() {
        let m = model();
        let result = replay(&m, &Trace::from_activities(&[]));
        assert_eq!(result.legal_moves, 0);
        assert_eq!(result.total_moves, 0);
        assert_eq!(result.first_violation, None);
        assert!(result.start_ok);
        assert!(result.end_ok);
        assert!((result.fitness() - 1.0).abs() < 1e-9);
        assert!(result.is_conformant());
        assert_eq!(result.verdict(), ConformanceVerdict::Conformant);
    }
}
