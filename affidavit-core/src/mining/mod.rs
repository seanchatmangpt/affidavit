//! Process mining over receipt chains — van der Aalst-style control-flow analysis.
//!
//! A receipt chain *is* an event log: each event's `event_type` is an *activity*
//! and `seq` gives the order, so one receipt is one *trace*. This module brings
//! the foundational process-mining lens to that log, holding the same discipline
//! as the rest of the crate (zero dependencies, no `unsafe`). It needs `alloc`
//! for its maps, so it lives behind the `alloc` feature.
//!
//! - [`Trace`] — the activity sequence of a receipt.
//! - [`DirectlyFollowsGraph`] — the directly-follows relation (process *discovery*).
//! - [`footprint`] — the α-algorithm footprint (causality / parallel / choice).
//! - [`conformance`] — token-replay *conformance checking* against a model.
//! - [`stats`] — log statistics (activity frequencies, trace variants).
//!
//! Doctrine, preserved: process mining here *certifies* a trace against a
//! discovered model; it never decides whether the recorded work was honest.
//!
//! # Example: from a sealed receipt to a conformance verdict
//!
//! ```
//! use affidavit_core::{ChainBuilder, Digest, Fnv256};
//! use affidavit_core::mining::{DirectlyFollowsGraph, Trace};
//! use affidavit_core::mining::conformance::{replay, ConformanceVerdict};
//!
//! // A sealed receipt is an event log of exactly one trace.
//! let receipt = ChainBuilder::<Fnv256>::new()
//!     .event("build", "evt-0", Digest([1u8; 32]))
//!     .event("test", "evt-1", Digest([2u8; 32]))
//!     .finalize();
//!
//! // Project its events into a trace, then discover a directly-follows model.
//! let events: Vec<_> = receipt.events().iter().map(|e| e.borrow()).collect();
//! let trace = Trace::from_events(&events);
//! let model = DirectlyFollowsGraph::discover(&[trace.clone()]);
//! assert!(model.follows("build", "test"));
//!
//! // The trace conforms to the model it was discovered from.
//! assert_eq!(replay(&model, &trace).verdict(), ConformanceVerdict::Conformant);
//! ```

use crate::chain::Event;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

pub mod conformance;
pub mod footprint;
pub mod stats;

pub use conformance::{replay, ConformanceResult, ConformanceVerdict};
pub use footprint::{AlphaRelation, Footprint};
pub use stats::LogStatistics;

/// A trace: the ordered activities (event types) of a receipt.
///
/// Borrows the activity strings from the events, so building a trace allocates
/// only the pointer vector, not the strings.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Trace<'a> {
    /// Activities in sequence (one per event, in the order given).
    pub activities: Vec<&'a str>,
}

impl<'a> Trace<'a> {
    /// Project a borrowed event slice into a trace of activities.
    pub fn from_events(events: &[Event<'a>]) -> Self {
        Trace {
            activities: events.iter().map(|e| e.event_type).collect(),
        }
    }

    /// Build a trace directly from a slice of activity names.
    pub fn from_activities(activities: &[&'a str]) -> Self {
        Trace {
            activities: activities.to_vec(),
        }
    }

    /// Number of activities (events) in the trace.
    pub fn len(&self) -> usize {
        self.activities.len()
    }

    /// True if the trace has no activities.
    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }
}

/// The directly-follows graph (DFG): the heart of process discovery.
///
/// Records, across one or more traces, how often activity `a` is *directly
/// followed by* activity `b`, plus the multiset of start and end activities.
/// Backed by `BTreeMap`/`BTreeSet` so iteration order is deterministic — a
/// discovered model is reproducible, which the verifier-grade ethos demands.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DirectlyFollowsGraph {
    /// `(a, b) -> count` that `a` was directly followed by `b`.
    pub edges: BTreeMap<(String, String), u64>,
    /// Every activity observed.
    pub activities: BTreeSet<String>,
    /// Start activities (first in a trace) with frequency.
    pub start: BTreeMap<String, u64>,
    /// End activities (last in a trace) with frequency.
    pub end: BTreeMap<String, u64>,
    /// Number of traces folded in.
    pub trace_count: u64,
}

impl DirectlyFollowsGraph {
    /// Discover a DFG from a set of traces.
    pub fn discover(traces: &[Trace<'_>]) -> Self {
        let mut dfg = DirectlyFollowsGraph::default();
        for trace in traces {
            dfg.add_trace(trace);
        }
        dfg
    }

    /// Fold a single trace into the graph.
    pub fn add_trace(&mut self, trace: &Trace<'_>) {
        self.trace_count += 1;
        let acts = &trace.activities;
        if acts.is_empty() {
            // An empty trace is still a process instance — it just contributes
            // no activities, starts, ends, or edges.
            return;
        }
        for &a in acts {
            self.activities.insert(String::from(a));
        }
        *self.start.entry(String::from(acts[0])).or_insert(0) += 1;
        *self
            .end
            .entry(String::from(acts[acts.len() - 1]))
            .or_insert(0) += 1;
        for window in acts.windows(2) {
            let key = (String::from(window[0]), String::from(window[1]));
            *self.edges.entry(key).or_insert(0) += 1;
        }
    }

    /// How often `a` was directly followed by `b` (0 if never).
    pub fn directly_follows(&self, a: &str, b: &str) -> u64 {
        self.edges
            .get(&(String::from(a), String::from(b)))
            .copied()
            .unwrap_or(0)
    }

    /// Whether `a > b` holds in the log (directly-follows at least once).
    pub fn follows(&self, a: &str, b: &str) -> bool {
        self.directly_follows(a, b) > 0
    }

    /// Sorted list of activities (borrowed).
    pub fn activity_list(&self) -> Vec<&str> {
        self.activities.iter().map(String::as_str).collect()
    }

    /// Whether `a` was ever a start activity.
    pub fn is_start(&self, a: &str) -> bool {
        self.start.contains_key(a)
    }

    /// Whether `a` was ever an end activity.
    pub fn is_end(&self, a: &str) -> bool {
        self.end.contains_key(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::Digest;
    use crate::Event;

    fn ev(seq: u64, ty: &'static str) -> Event<'static> {
        Event {
            seq,
            event_id: "id",
            event_type: ty,
            commitment: Digest([1u8; 32]),
        }
    }

    #[test]
    fn trace_from_events_projects_activities() {
        let events = [ev(0, "build"), ev(1, "test"), ev(2, "deploy")];
        let trace = Trace::from_events(&events);
        assert_eq!(trace.activities, ["build", "test", "deploy"]);
        assert_eq!(trace.len(), 3);
        assert!(!trace.is_empty());
    }

    #[test]
    fn discovery_counts_directly_follows() {
        // [a,b,c] and [a,b,b,c]: a>b twice, b>c twice, b>b once.
        let t1 = Trace::from_activities(&["a", "b", "c"]);
        let t2 = Trace::from_activities(&["a", "b", "b", "c"]);
        let dfg = DirectlyFollowsGraph::discover(&[t1, t2]);

        assert_eq!(dfg.trace_count, 2);
        assert_eq!(dfg.directly_follows("a", "b"), 2);
        assert_eq!(dfg.directly_follows("b", "c"), 2);
        assert_eq!(dfg.directly_follows("b", "b"), 1);
        assert_eq!(dfg.directly_follows("c", "a"), 0);
        assert!(dfg.follows("a", "b"));
        assert!(!dfg.follows("c", "a"));

        assert!(dfg.is_start("a"));
        assert!(dfg.is_end("c"));
        assert_eq!(dfg.start.get("a"), Some(&2));
        assert_eq!(dfg.end.get("c"), Some(&2));
        assert_eq!(dfg.activity_list(), ["a", "b", "c"]);
    }

    #[test]
    fn empty_trace_counts_but_adds_nothing() {
        let dfg = DirectlyFollowsGraph::discover(&[Trace::default()]);
        assert_eq!(dfg.trace_count, 1);
        assert!(dfg.activities.is_empty());
        assert!(dfg.edges.is_empty());
    }
}
