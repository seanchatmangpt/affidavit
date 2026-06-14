// Reference witness: ParityComparer::assert_epsilon_close — the cross-impl float
// parity check (pm4py vs wasm4pm agreement within epsilon), COVERAGE.md §2 —
// parity comparison.
//
// Process-mining parity testing compares a reference implementation's metric to
// ours; they must agree within a tolerance. assert_epsilon_close passes when
// |actual - expected| < 1e-6 and panics otherwise. This witnesses both the
// accept path and (via catch) the reject path.

use wasm4pm_compat::multiperspective::ParityComparer;

#[test]
fn parity_accepts_values_within_epsilon() {
    // Within 1e-6 → no panic.
    ParityComparer::assert_epsilon_close(0.8333333, 0.8333334);
    ParityComparer::assert_epsilon_close(1.0, 1.0);
    ParityComparer::assert_epsilon_close(0.0, 0.0000001);
}

#[test]
fn parity_rejects_divergent_values() {
    // A divergence beyond epsilon must panic — caught here to assert the rejection.
    let diverged = std::panic::catch_unwind(|| {
        ParityComparer::assert_epsilon_close(0.83, 0.91); // diff 0.08 >> 1e-6
    });
    assert!(diverged.is_err(), "a parity violation beyond epsilon must panic");
}
