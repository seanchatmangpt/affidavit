// Reference witness: the court's boundary-verdict carriers Admission<T,W> and
// Refusal<R,W> directly (COVERAGE.md §2.1 — admission/refusal carriers).
//
// Admission<T,W> carries an admitted value and is the ONLY producer of
// Evidence<T, Admitted, W> (via into_evidence → Evidence::sealed, which is
// crate-private). Refusal<R,W> carries a NAMED reason R (never a bare error) and
// yields it via into_reason. These are the two outcomes of every Admit::admit.

use wasm4pm_compat::admission::{Admission, Refusal};
use wasm4pm_compat::witness::Ocel20;

// A named refusal reason — the discipline is "name the law", never a string blob.
#[derive(Debug, PartialEq, Eq)]
enum MyLaw {
    DanglingLink,
}

#[test]
fn admission_carries_value_and_mints_admitted_evidence() {
    let a = Admission::<&str, Ocel20>::new("admitted-log");
    assert_eq!(a.value, "admitted-log", "admission carries its value");
    // into_evidence is the sole path to Admitted; the result holds the same value.
    let ev = a.into_evidence();
    assert_eq!(ev.value, "admitted-log", "Admitted evidence holds the admitted value");
}

#[test]
fn refusal_carries_a_named_reason() {
    let r = Refusal::<MyLaw, Ocel20>::new(MyLaw::DanglingLink);
    assert_eq!(r.reason, MyLaw::DanglingLink, "refusal carries its named reason");
    // into_reason yields the named law back — auditable, not a stack trace.
    assert_eq!(r.into_reason(), MyLaw::DanglingLink);
}
