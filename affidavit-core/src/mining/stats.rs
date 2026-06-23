//! Log statistics: activity frequencies and trace variants.
//!
//! The most basic process-mining lens on an event log is to *count*: how many
//! events and traces it holds, how often each activity occurs, and which
//! distinct activity-sequences (*variants*) the traces fall into and with what
//! multiplicity. [`LogStatistics`] folds a slice of [`Trace`]s into exactly
//! those aggregates.
//!
//! Everything is backed by `BTreeMap`, so iteration and tie-breaking are
//! deterministic — the same log always yields the same statistics, including
//! the same [`most_frequent_variant`](LogStatistics::most_frequent_variant).
//! Like the rest of the module these counts are purely derived from the log and
//! decide nothing about whether the recorded work was honest; they only
//! *certify* the shape of the log.

use crate::mining::Trace;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Aggregate statistics over a set of traces (an event log).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LogStatistics {
    /// Total number of events (sum of trace lengths).
    pub event_count: u64,
    /// Number of traces (process instances).
    pub trace_count: u64,
    /// Per-activity occurrence counts across the whole log.
    pub activity_frequency: BTreeMap<String, u64>,
    /// Trace variants: each distinct activity-sequence mapped to how many traces had it.
    pub variants: BTreeMap<Vec<String>, u64>,
}

impl LogStatistics {
    /// Compute statistics from a set of traces.
    pub fn from_traces(traces: &[Trace<'_>]) -> Self {
        let mut activity_frequency: BTreeMap<String, u64> = BTreeMap::new();
        let mut variants: BTreeMap<Vec<String>, u64> = BTreeMap::new();
        let mut event_count: u64 = 0;

        for trace in traces {
            event_count += trace.len() as u64;
            for &activity in &trace.activities {
                *activity_frequency
                    .entry(String::from(activity))
                    .or_insert(0) += 1;
            }
            // The variant key is the activity sequence as owned strings; an empty
            // trace keys the empty-sequence variant.
            let key: Vec<String> = trace.activities.iter().map(|&a| String::from(a)).collect();
            *variants.entry(key).or_insert(0) += 1;
        }

        LogStatistics {
            event_count,
            trace_count: traces.len() as u64,
            activity_frequency,
            variants,
        }
    }

    /// Number of distinct activities seen.
    pub fn distinct_activities(&self) -> usize {
        self.activity_frequency.len()
    }

    /// Number of distinct trace variants.
    pub fn distinct_variants(&self) -> usize {
        self.variants.len()
    }

    /// The most frequent variant and its count (ties broken by the smallest
    /// sequence in `BTreeMap` order). `None` if there are no traces.
    pub fn most_frequent_variant(&self) -> Option<(&[String], u64)> {
        let mut best: Option<(&Vec<String>, u64)> = None;
        // `BTreeMap` iterates keys in ascending order, so keeping only the
        // *strictly* greater count makes the first (smallest) key win ties.
        for (key, &count) in &self.variants {
            match best {
                Some((_, best_count)) if count <= best_count => {}
                _ => best = Some((key, count)),
            }
        }
        best.map(|(key, count)| (key.as_slice(), count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log() -> [Trace<'static>; 3] {
        [
            Trace::from_activities(&["a", "b", "c"]),
            Trace::from_activities(&["a", "b", "c"]),
            Trace::from_activities(&["a", "b"]),
        ]
    }

    #[test]
    fn counts_events_and_traces() {
        let stats = LogStatistics::from_traces(&log());
        assert_eq!(stats.event_count, 8);
        assert_eq!(stats.trace_count, 3);
    }

    #[test]
    fn counts_activity_frequencies() {
        let stats = LogStatistics::from_traces(&log());
        assert_eq!(stats.activity_frequency.get("a"), Some(&3));
        assert_eq!(stats.activity_frequency.get("b"), Some(&3));
        assert_eq!(stats.activity_frequency.get("c"), Some(&2));
        assert_eq!(stats.distinct_activities(), 3);
    }

    #[test]
    fn groups_distinct_variants() {
        let stats = LogStatistics::from_traces(&log());
        assert_eq!(stats.distinct_variants(), 2);

        let abc: Vec<String> = ["a", "b", "c"].iter().map(|&s| String::from(s)).collect();
        let ab: Vec<String> = ["a", "b"].iter().map(|&s| String::from(s)).collect();
        assert_eq!(stats.variants.get(&abc), Some(&2));
        assert_eq!(stats.variants.get(&ab), Some(&1));
    }

    #[test]
    fn picks_most_frequent_variant() {
        let stats = LogStatistics::from_traces(&log());
        let expected: &[String] = &[String::from("a"), String::from("b"), String::from("c")];
        assert_eq!(stats.most_frequent_variant(), Some((expected, 2)));
    }

    #[test]
    fn most_frequent_variant_breaks_ties_by_smallest_key() {
        // Two distinct variants, each occurring once: the BTreeMap-smaller key
        // ([a] < [b]) must win the tie deterministically.
        let traces = [
            Trace::from_activities(&["b"]),
            Trace::from_activities(&["a"]),
        ];
        let stats = LogStatistics::from_traces(&traces);
        let expected: &[String] = &[String::from("a")];
        assert_eq!(stats.most_frequent_variant(), Some((expected, 1)));
    }

    #[test]
    fn empty_trace_is_the_empty_variant() {
        let traces = [Trace::default(), Trace::default()];
        let stats = LogStatistics::from_traces(&traces);
        assert_eq!(stats.event_count, 0);
        assert_eq!(stats.trace_count, 2);
        assert_eq!(stats.distinct_activities(), 0);
        assert_eq!(stats.distinct_variants(), 1);

        let empty: &[String] = &[];
        assert_eq!(stats.variants.get(empty), Some(&2));
        assert_eq!(stats.most_frequent_variant(), Some((empty, 2)));
    }

    #[test]
    fn empty_log_has_no_most_frequent_variant() {
        let stats = LogStatistics::from_traces(&[]);
        assert_eq!(stats.event_count, 0);
        assert_eq!(stats.trace_count, 0);
        assert_eq!(stats.distinct_activities(), 0);
        assert_eq!(stats.distinct_variants(), 0);
        assert_eq!(stats.most_frequent_variant(), None);
    }
}
