// Reference witness: the Petri Marking token surface — construction, tokens()
// iteration, tokens_on lookup, empty/non-empty (COVERAGE.md §2 — marking arithmetic).
//
// A Marking maps places to token counts. This witnesses multi-place markings, the
// tokens() view, per-place tokens_on lookup (0 for absent places), and the empty
// marking.

use wasm4pm_compat::petri::Marking;

#[test]
fn marking_records_per_place_token_counts() {
    let m = Marking::new([
        ("p0".to_string(), 2),
        ("p1".to_string(), 1),
        ("p2".to_string(), 0),
    ]);
    assert!(!m.is_empty(), "a marking with token entries is non-empty");
    assert_eq!(m.tokens_on("p0"), 2);
    assert_eq!(m.tokens_on("p1"), 1);
    assert_eq!(m.tokens_on("absent"), 0, "absent place → 0 tokens");

    // tokens() exposes the full (place, count) view.
    let total: usize = m.tokens().iter().map(|(_, n)| *n).sum();
    assert_eq!(total, 3, "total token count across places");
}

#[test]
fn empty_marking_has_no_tokens() {
    let e = Marking::empty();
    assert!(e.is_empty());
    assert_eq!(e.tokens_on("anything"), 0);
    assert!(e.tokens().is_empty());
}
