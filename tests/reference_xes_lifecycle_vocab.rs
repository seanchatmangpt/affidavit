// Reference witness: the full XES lifecycle-transition vocabulary (COVERAGE.md §2 —
// XES lifecycle completeness). Was 1 of 14 witnessed (Complete only); this is the
// exhaustive as_str↔parse round-trip + is_terminal taxonomy + closed-alphabet refusal.
//
// The XES lifecycle alphabet brackets activities into intervals (start…complete,
// suspend/resume) — the basis for duration and waiting-time mining. This witnesses
// EVERY variant round-trips variant → as_str → parse → same variant (a typo in
// either match arm breaks it), that the alphabet is CLOSED (an off-alphabet string
// parses to None), and the is_terminal taxonomy.

use wasm4pm_compat::xes::XesLifecycleTransition::{self, *};

const ALL: [XesLifecycleTransition; 14] = [
    Schedule, Assign, Start, Suspend, Resume, InProgress, Abort, Withdraw, Complete, Unknown,
    AutoSkip, ManualSkip, Reassign, Plan,
];

#[test]
fn every_lifecycle_variant_round_trips_through_as_str_and_parse() {
    for t in ALL {
        let s = t.as_str();
        assert_eq!(
            XesLifecycleTransition::parse(s),
            Some(t),
            "{t:?} must round-trip via its own as_str() string {s:?}"
        );
    }
}

#[test]
fn the_lifecycle_alphabet_is_closed() {
    // A value outside the standard alphabet is NOT silently coerced.
    assert_eq!(XesLifecycleTransition::parse("custom"), None);
    assert_eq!(XesLifecycleTransition::parse(""), None);
    assert_eq!(
        XesLifecycleTransition::parse("Complete"),
        None,
        "case-sensitive: capital-C is off-alphabet"
    );
}

#[test]
fn terminal_transitions_are_exactly_the_five_end_states() {
    // No-wildcard match makes this taxonomy total: a new variant forces a decision.
    for t in ALL {
        let expected = match t {
            Complete | Abort | Withdraw | ManualSkip | AutoSkip => true,
            Schedule | Assign | Start | Suspend | Resume | InProgress | Unknown | Reassign
            | Plan => false,
        };
        assert_eq!(t.is_terminal(), expected, "{t:?} terminal classification");
    }
}
