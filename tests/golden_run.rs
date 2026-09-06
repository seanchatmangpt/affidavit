// The golden example, executed.
//
// `examples/golden_run.sh` is the script README.md and CLAUDE.md point new users
// at, and its own final line asserts "GOLDEN RUN OK: ACCEPT(0) then REJECT(2)".
// Until v26.9.6 nothing ran it: no CI workflow and no test referenced it. It had
// been broken in two independent ways at once —
//
//   1. it invoked `cargo run` from inside a temp dir, so cargo resolved the
//      toolchain from that directory, missed `rust-toolchain.toml`, fell back to
//      stable, and died on wasm4pm-compat's `#![feature(...)]` (E0554, exit 101);
//   2. even had it built, the tampered receipt exited 1 (a framework parse
//      error) rather than the 2 the script asserts, because the chain law ran at
//      deserialization and stage 3 never executed.
//
// Documentation that has never been run is a claim, not evidence. This test
// makes the golden path a court.

use std::process::Command;

#[test]
fn the_golden_example_runs_and_demonstrates_accept_then_reject() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let script = format!("{repo_root}/examples/golden_run.sh");

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(repo_root)
        .output()
        .expect("bash is available to run the golden example");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    assert!(
        output.status.success(),
        "examples/golden_run.sh must succeed; it exited {:?}\n--- output ---\n{combined}",
        output.status.code()
    );

    // The honest half: the full pipeline accepts.
    assert!(
        combined.contains("verdict: ACCEPT"),
        "the golden run must show an ACCEPT verdict\n{combined}"
    );
    // The teeth: the tampered half must fail at stage 3 by name, not merely
    // "somewhere". A REJECT that never names chain_integrity would mean the
    // corruption was caught at the door instead of adjudicated.
    assert!(
        combined.contains("chain_integrity: FAIL"),
        "the tampered half must fail stage 3 by name\n{combined}"
    );
    assert!(
        combined.contains("exit code: 2"),
        "the tampered half must exit with the stable REJECT code 2\n{combined}"
    );
    assert!(
        combined.contains("GOLDEN RUN OK"),
        "the script's own final assertion must hold\n{combined}"
    );
}
