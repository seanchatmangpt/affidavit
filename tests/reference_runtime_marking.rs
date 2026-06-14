// Reference witness: RuntimeMarking<'a> — the borrowed initial-marking view over a
// PetriNet (COVERAGE.md §2 — Petri runtime marking).
//
// PetriNet::initial_marking() returns a RuntimeMarking that borrows the net and
// answers tokens_on(place_id) by FNV-hashing the id into the net's packed
// initial_marking table. This witnesses: a token placed on "p0" is read back via
// the RuntimeMarking; an absent place reads 0. Failing-when-fake: if tokens_on
// looked at the wrong table or hash, the placed token would not be found.

use wasm4pm_compat::dense_kernel::{fnv1a_64, PackedKeyTable};
use wasm4pm_compat::models::PetriNet;

#[test]
fn runtime_marking_reads_tokens_placed_on_the_net() {
    let mut net = PetriNet::default();
    // Place 3 tokens on "p0" in the net's initial marking (keyed by FNV hash of id).
    let mut marking = PackedKeyTable::new();
    marking.insert(fnv1a_64(b"p0"), "p0".to_string(), 3usize);
    net.initial_marking = marking;

    let rt = net.initial_marking();
    assert_eq!(rt.tokens_on("p0"), 3, "the borrowed view reads the placed tokens");
    assert_eq!(rt.tokens_on("absent"), 0, "an unmarked place has zero tokens");
}
