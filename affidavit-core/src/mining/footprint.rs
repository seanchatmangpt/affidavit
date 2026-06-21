//! α-algorithm footprint matrix.
//!
//! Van der Aalst's α-algorithm reads four ordered relations off the
//! directly-follows relation `>` of a log. For every ordered pair `(a, b)` of
//! activities it asks two yes/no questions — does `a > b`, and does `b > a`? —
//! and folds the answers into one of four relations:
//!
//! | `a > b` | `b > a` | relation        | symbol  |
//! |---------|---------|-----------------|---------|
//! | yes     | no      | causality       | `a -> b`|
//! | no      | yes     | reverse causality | `a <- b`|
//! | yes     | yes     | parallel        | `a \|\| b`|
//! | no      | no      | choice          | `a # b` |
//!
//! The full table over all ordered pairs is the *footprint* of the log: a
//! fingerprint of its control flow that two logs share iff they induce the same
//! α-relations. Like the rest of the module it is purely derived from a
//! [`DirectlyFollowsGraph`] and decides nothing about whether the work was
//! honest — it only *certifies* the relations the log exhibits.

use crate::mining::DirectlyFollowsGraph;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

/// The α-algorithm relation between an ordered pair of activities (a, b),
/// derived from the directly-follows relation `>` of a DFG.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlphaRelation {
    /// a -> b : causality (a > b and not b > a).
    Causality,
    /// a <- b : reverse causality (b > a and not a > b).
    ReverseCausality,
    /// a || b : parallel (a > b and b > a).
    Parallel,
    /// a # b : choice / no direct relation (neither a > b nor b > a).
    Choice,
}

/// The α-footprint of a log: the relation between every ordered pair of activities.
///
/// Stores the sorted activity universe alongside the set of ordered pairs
/// `(x, y)` for which `x > y` holds in the source DFG. The four α-relations are
/// then read off that set on demand, so the footprint is compact (it never
/// materializes the quadratic matrix) yet fully determined.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Footprint {
    /// Every activity covered, sorted and de-duplicated.
    activities: Vec<String>,
    /// Ordered pairs `(x, y)` for which `x > y` holds (`x` is directly followed by `y`).
    follows: BTreeSet<(String, String)>,
}

impl Footprint {
    /// Compute the footprint from a discovered DFG.
    pub fn from_dfg(dfg: &DirectlyFollowsGraph) -> Self {
        let activities: Vec<String> = dfg.activities.iter().cloned().collect();
        let mut follows = BTreeSet::new();
        for a in &activities {
            for b in &activities {
                if dfg.follows(a, b) {
                    follows.insert((a.clone(), b.clone()));
                }
            }
        }
        Footprint {
            activities,
            follows,
        }
    }

    /// Whether `x > y` holds in the captured directly-follows relation.
    fn gt(&self, x: &str, y: &str) -> bool {
        // `BTreeSet::contains` over a `(String, String)` needs an owned-ish key;
        // build the borrowed-key tuple cheaply via the `(&str, &str)` Borrow
        // chain so this stays allocation-free.
        self.follows
            .iter()
            .any(|(a, b)| a.as_str() == x && b.as_str() == y)
    }

    /// The α-relation between `a` and `b` (Choice if either is unknown).
    pub fn relation(&self, a: &str, b: &str) -> AlphaRelation {
        let ab = self.gt(a, b);
        let ba = self.gt(b, a);
        match (ab, ba) {
            (true, false) => AlphaRelation::Causality,
            (false, true) => AlphaRelation::ReverseCausality,
            (true, true) => AlphaRelation::Parallel,
            (false, false) => AlphaRelation::Choice,
        }
    }

    /// The activities covered, sorted.
    pub fn activities(&self) -> &[String] {
        &self.activities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mining::Trace;

    #[test]
    fn sequential_trace_yields_causality_and_choice() {
        // [a, b, c]: a > b, b > c (and nothing else).
        let dfg = DirectlyFollowsGraph::discover(&[Trace::from_activities(&["a", "b", "c"])]);
        let fp = Footprint::from_dfg(&dfg);

        assert_eq!(fp.relation("a", "b"), AlphaRelation::Causality);
        assert_eq!(fp.relation("b", "a"), AlphaRelation::ReverseCausality);
        assert_eq!(fp.relation("b", "c"), AlphaRelation::Causality);
        assert_eq!(fp.relation("c", "b"), AlphaRelation::ReverseCausality);
        // a and c never directly follow each other in either direction.
        assert_eq!(fp.relation("a", "c"), AlphaRelation::Choice);
        assert_eq!(fp.relation("c", "a"), AlphaRelation::Choice);
        // An activity never directly follows itself here.
        assert_eq!(fp.relation("a", "a"), AlphaRelation::Choice);

        assert_eq!(fp.activities(), ["a", "b", "c"]);
    }

    #[test]
    fn short_loop_yields_parallel() {
        // [a, b, a, b]: a > b and b > a, so the α-relation is parallel both ways.
        let dfg = DirectlyFollowsGraph::discover(&[Trace::from_activities(&["a", "b", "a", "b"])]);
        let fp = Footprint::from_dfg(&dfg);

        assert!(dfg.follows("a", "b"));
        assert!(dfg.follows("b", "a"));
        assert_eq!(fp.relation("a", "b"), AlphaRelation::Parallel);
        assert_eq!(fp.relation("b", "a"), AlphaRelation::Parallel);
        assert_eq!(fp.activities(), ["a", "b"]);
    }

    #[test]
    fn unknown_activities_are_choice() {
        let dfg = DirectlyFollowsGraph::discover(&[Trace::from_activities(&["a", "b", "c"])]);
        let fp = Footprint::from_dfg(&dfg);

        // Neither side is in the activity set -> no `>` either way -> Choice.
        assert_eq!(fp.relation("x", "y"), AlphaRelation::Choice);
        // One side known, the other not -> still Choice.
        assert_eq!(fp.relation("a", "zzz"), AlphaRelation::Choice);
        assert_eq!(fp.relation("zzz", "a"), AlphaRelation::Choice);
    }

    #[test]
    fn relation_is_symmetric_under_swap() {
        // Causality on one side must read as reverse causality on the other,
        // and parallel/choice must be swap-stable. Checked over a small log.
        let dfg = DirectlyFollowsGraph::discover(&[
            Trace::from_activities(&["a", "b", "c"]),
            Trace::from_activities(&["a", "c", "b"]),
        ]);
        let fp = Footprint::from_dfg(&dfg);

        for a in fp.activities() {
            for b in fp.activities() {
                let forward = fp.relation(a, b);
                let backward = fp.relation(b, a);
                let expected = match forward {
                    AlphaRelation::Causality => AlphaRelation::ReverseCausality,
                    AlphaRelation::ReverseCausality => AlphaRelation::Causality,
                    AlphaRelation::Parallel => AlphaRelation::Parallel,
                    AlphaRelation::Choice => AlphaRelation::Choice,
                };
                assert_eq!(backward, expected, "swap mismatch for ({a}, {b})");
            }
        }
    }
}
