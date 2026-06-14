// Reference witness: CancellationRegion — the cancellation-pattern membership set
// (COVERAGE.md §2 — Petri cancellation region).
//
// A CancellationRegion is the set of net nodes whose tokens are withdrawn when a
// cancellation fires (van der Aalst's cancel-region / WCP-19,20). This witnesses
// construction from a &str iterator (exercising the Into<String> bound) and the
// members() read-back, order preserved. Failing-when-fake: if members were dropped
// or reordered, the assert fails.

use wasm4pm_compat::petri::CancellationRegion;

#[test]
fn cancellation_region_collects_members_from_str_iter() {
    let region = CancellationRegion::new(["t_a", "t_b", "p_mid"]);
    assert_eq!(region.members(), &["t_a".to_string(), "t_b".to_string(), "p_mid".to_string()]);
    assert_eq!(region.members().len(), 3, "all three members retained");
}

#[test]
fn empty_cancellation_region_has_no_members() {
    let empty = CancellationRegion::new(Vec::<String>::new());
    assert!(empty.members().is_empty(), "an empty region cancels nothing");
}
