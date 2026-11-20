//! SJ projection parity court (register row 44).
//!
//! Soft-gated: the real clnrm-core 26.5.x path pin is Mac-local and is no
//! longer declared in Cargo.toml (it broke every other host). Re-enable by
//! wiring a local path dep named `clnrm_sj_core` and restoring the full body
//! from git history (pre-soft-gate).
//!
//! This ignored court is NOT a vacuous ALIVE for row 44 — standing stays OPEN
//! until the portable real edge is admitted. `--ignored` must panic.

#[test]
#[ignore = "requires local clnrm-core 26.5.x path dep; row 44 remains OPEN"]
fn sj_projection_parity_requires_local_clnrm_edge() {
    panic!(
        "REFUSED_UNSUPPORTED: SJ projection parity needs a portable clnrm-core \
         26.5.x edge; do not treat this ignored court as ALIVE"
    );
}
