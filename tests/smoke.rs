//! Smoke court for the tests/common matrix harness.
//!
//! Proves the harness can fail (a court that cannot fail is not a court):
//! positive control must fire on a real ACCEPT, refusal control must fire on
//! a real typed refusal, the two world builders must genuinely differ, and
//! scratch dirs must be unique. Lives at `tests/` root (not
//! `tests/common/smoke.rs`) because cargo autodiscovers test targets only at
//! the `tests/` root — a file under `tests/common/` is never compiled as a
//! target and the court would silently execute zero tests.
//!
//! Owned by the harness lane. `cargo test --test smoke` must exit 0.

mod common;

use common::*;

#[test]
fn harness_positive_control_fires() {
    let world = fixture_world("smoke-positive");
    let out = affi(&["receipt", "verify", world.receipt.to_str().unwrap()]);
    assert_positive(&out, &world);
}

#[test]
fn harness_refusal_control_fires() {
    let out = affi(&["receipt", "notarealverb"]);
    assert_typed_refusal(&out);
}

#[test]
fn fixture_and_other_worlds_differ() {
    let a = fixture_world("smoke-fixture");
    let b = other_world("smoke-other");
    assert_ne!(
        std::fs::read_to_string(&a.receipt).unwrap(),
        std::fs::read_to_string(&b.receipt).unwrap(),
        "the two world builders must produce deliberately-different receipts"
    );
    assert_eq!(a.n_events, 2);
    assert_eq!(b.n_events, 4); // 3 events + license
}

#[test]
fn outputs_differ_separates_worlds_and_unites_reruns() {
    let a = fixture_world("smoke-diff-a");
    let b = other_world("smoke-diff-b");
    let oa = affi(&["receipt", "verify", a.receipt.to_str().unwrap()]);
    let ob = affi(&["receipt", "verify", b.receipt.to_str().unwrap()]);
    assert!(outputs_differ(&oa, &ob, a.dir(), b.dir()));
    // Identical world, re-run: normalized outputs must NOT differ (determinism).
    let oa2 = affi(&["receipt", "verify", a.receipt.to_str().unwrap()]);
    assert!(!outputs_differ(&oa, &oa2, a.dir(), a.dir()));
}

#[test]
fn scratch_dirs_are_unique() {
    let a = scratch_dir("smoke-scratch");
    let b = scratch_dir("smoke-scratch");
    assert_ne!(a.path(), b.path());
    assert!(a.path().is_dir());
}

#[test]
fn affi_capture_round_trip() {
    let (code, stdout, _stderr) = affi_capture(&["--version"]);
    assert_eq!(code, 0);
    assert!(!stdout.trim().is_empty());
}
