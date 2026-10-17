//! Court: the crypto-trust CLI value plane's DEFAULT-BINARY surface.
//!
//! Split of labor:
//!
//! * NEGATIVE layer (this file, always compiled): the default binary — built
//!   WITHOUT `--features crypto-trust` — must refuse every crypto-trust CLI
//!   verb with the typed refusal (CLI execution error, exit 224,
//!   `REFUSED_UNSUPPORTED`, naming the `crypto-trust` feature). Integration
//!   tests reach the binary via `CARGO_BIN_EXE_affi`, which is always built
//!   with default features only, so the positive (real key/envelope/evidence)
//!   paths cannot be exercised from this file.
//!
//! * POSITIVE layer (documented, not duplicated here): the crypto-trust plane
//!   battery covers the real paths — keys/envelope/evidence flows are
//!   exercised in the plane's lib tests (`src/handlers.rs`
//!   `evidence_lane_tests` and siblings, `#[cfg(all(test,
//!   feature = "crypto-trust"))]`) and the dedicated CLI suites
//!   (`tests/crypto_trust_keys_cli.rs`, `tests/crypto_trust_envelope_cli.rs`,
//!   `tests/crypto_trust_evidence_cli.rs`, `tests/crypto_trust_e2e.rs`),
//!   which run under `--features crypto-trust`.
//!
//! A verb whose refusal stops firing is a regression: it means the stub layer
//! was removed or its exit code changed while the feature is off — the
//! default binary would then crash-loop or mislead instead of refusing
//! lawfully. 13 verbs, one assertion per verb, no skips: a court that cannot
//! fail is not a court.
//!
//! Gate: `cargo test --test cli_value_trust_plane` exit 0.

use std::process::Command;

/// Stable diagnostic: CLI execution error, exit 224 (below 256, so the raw
/// code reaches the process status unmodded).
const EXIT_REFUSED_UNSUPPORTED: i32 = 224;

/// The 13 crypto-trust CLI verbs: (noun, verb, minimal real args).
/// Args match the handler signatures so dispatch reaches the refusal stub,
/// not a clap arity error (an arity error would exit 220/221, not 224).
const VERBS: &[(&str, &str, &[&str])] = &[
    ("keys", "generate", &["ES256", "custodian-x"]),
    ("keys", "list", &[]),
    ("keys", "import", &["ES256", "a1b2", "custodian-x"]),
    ("keys", "revoke", &["kid-x", "compromise"]),
    ("keys", "rotate", &["kid-x"]),
    (
        "envelope",
        "sign",
        &["/tmp/affi-vtp-nope.json", "/tmp/affi-vtp-nokey"],
    ),
    ("envelope", "verify", &["/tmp/affi-vtp-nope.json"]),
    ("envelope", "list", &[]),
    ("envelope", "export", &["/tmp/affi-vtp-nope.sealed"]),
    ("evidence", "journal", &["repo:main"]),
    ("evidence", "crl-publish", &["kid-x", "7"]),
    ("evidence", "crl-apply", &["/tmp/affi-vtp-nope.crl"]),
    ("evidence", "heads", &[]),
];

fn run_affi(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn refusal_inventory_is_exactly_13_verbs() {
    assert_eq!(VERBS.len(), 13, "the crypto-trust CLI plane has 13 verbs");
}

#[test]
fn every_crypto_trust_verb_refuses_unsupported_on_default_binary() {
    let mut refused = 0;
    for (noun, verb, args) in VERBS {
        let mut argv: Vec<&str> = vec![noun, verb];
        argv.extend_from_slice(args);
        let (code, stdout, stderr) = run_affi(&argv);
        let combined = format!("{stdout}{stderr}");
        assert_eq!(
            code, EXIT_REFUSED_UNSUPPORTED,
            "{noun} {verb} must exit 224 REFUSED_UNSUPPORTED on the default binary; got {code}: {combined}"
        );
        assert!(
            combined.contains("REFUSED_UNSUPPORTED"),
            "{noun} {verb} output must name the typed refusal: {combined}"
        );
        assert!(
            combined.contains("crypto-trust"),
            "{noun} {verb} refusal must name the missing feature plane: {combined}"
        );
        refused += 1;
    }
    assert_eq!(refused, 13, "all 13 verbs must fire the refusal");
}

#[test]
fn non_plane_verb_still_works_while_plane_is_refused() {
    // Positive control that the binary itself is healthy and the refusals are
    // not a wholesale CLI breakage: a core verb still dispatches and runs.
    let (code, stdout, stderr) = run_affi(&["doctor"]);
    let combined = format!("{stdout}{stderr}");
    assert_ne!(
        code, 224,
        "a core verb must not be swept into the crypto-trust refusal: {combined}"
    );
    assert!(
        !combined.contains("REFUSED_UNSUPPORTED"),
        "core verb output must not carry the plane refusal: {combined}"
    );
}
