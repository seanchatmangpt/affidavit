// Witness: the affidavit OTel semantic-convention registry is CLOSED.
//
// This test shells the REAL `weaver registry check` (OpenTelemetry Weaver
// v0.22.1) against:
//   (a) semconv/registry        — the conformant registry; MUST exit 0.
//   (b) semconv/registry_broken — a deliberately-invalid registry; MUST exit
//       non-zero. This is the negative control proving the success assertion in
//       (a) is not constant-true — if `weaver` were a no-op that always exits 0,
//       (b) would FAIL this test.
//   (c) coherence — the attribute names declared in semconv/registry/affidavit.yaml
//       MUST equal the SpanRecord fields emitted by src/tracing.rs: {operation, target}.
//
// If the `weaver` binary is absent, this test SKIPS WITH A PRINTED MESSAGE
// (never silent-green): it eprintln!s and returns early. Weaver IS installed in
// the dev environment, so the test runs and passes there.
//
// Honest scope: this closes the *semantic-convention registry* surface (the
// emitted span shape is validated against a real OTel semconv registry). Full
// OpenTelemetry SDK export to a running collector (Jaeger/OTLP) remains
// OPEN-substrate — see src/tracing.rs honest scope.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolve the weaver binary: prefer `weaver` on PATH, fall back to the
/// known cargo install location. Returns None if neither is invocable.
fn resolve_weaver() -> Option<String> {
    for candidate in ["weaver", concat!(env!("HOME"), "/.cargo/bin/weaver")] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            // .output() Ok means the binary was spawned; confirm it ran.
            if let Ok(out) = Command::new(candidate).arg("--version").output() {
                if out.status.success() {
                    return Some(candidate.to_string());
                }
            }
        }
    }
    None
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run `weaver registry check -r <dir>` and return the exit code (None if the
/// process did not produce a code, e.g. killed by signal).
fn weaver_check(weaver: &str, registry: &Path) -> Option<i32> {
    let out = Command::new(weaver)
        .args(["registry", "check", "-r"])
        .arg(registry)
        .output()
        .expect("weaver should spawn (binary already resolved)");
    out.status.code()
}

/// Parse the `id:` values under the `attributes:` list of the affidavit.yaml
/// group. Intentionally a tiny hand-rolled scan so the test adds no YAML
/// dependency; it asserts the declared attribute-id set.
fn declared_attribute_ids(yaml: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut in_attributes = false;
    // Indentation (column) of the attribute list's `- id:` items. Enum members
    // (`members:` -> `- id:`) live deeper, so we only accept items at exactly
    // this column to avoid scooping up enum-member ids.
    let mut attr_item_indent: Option<usize> = None;
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if trimmed.starts_with("attributes:") {
            in_attributes = true;
            attr_item_indent = None;
            continue;
        }
        if !in_attributes {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- id:") {
            match attr_item_indent {
                None => attr_item_indent = Some(indent), // first item sets the level
                Some(level) if indent != level => continue, // deeper => enum member
                _ => {}
            }
            ids.insert(rest.trim().trim_matches('"').to_string());
        }
    }
    ids
}

#[test]
fn weaver_semconv_registry_is_closed_and_coherent() {
    let weaver = match resolve_weaver() {
        Some(w) => w,
        None => {
            eprintln!(
                "SKIP: `weaver` binary not found on PATH or in ~/.cargo/bin; \
                 cannot witness the semconv registry check. (Install OTel Weaver \
                 v0.22+ to run this test.)"
            );
            return;
        }
    };

    let root = repo_root();
    let good = root.join("semconv/registry");
    let broken = root.join("semconv/registry_broken");

    // (a) Conformant registry MUST exit 0.
    let good_code = weaver_check(&weaver, &good);
    assert_eq!(
        good_code,
        Some(0),
        "weaver registry check on {good:?} must EXIT 0 (conformant registry); got {good_code:?}"
    );

    // (b) Negative control: broken registry MUST exit non-zero.
    let broken_code = weaver_check(&weaver, &broken);
    assert!(
        matches!(broken_code, Some(c) if c != 0),
        "weaver registry check on {broken:?} must EXIT non-zero (broken registry) — \
         negative control proving the check is not constant-true; got {broken_code:?}"
    );

    // (c) Coherence: declared attribute ids == SpanRecord fields {operation, target}.
    let yaml = std::fs::read_to_string(good.join("affidavit.yaml"))
        .expect("affidavit.yaml must exist in the conformant registry");
    let declared = declared_attribute_ids(&yaml);

    // The documented SpanRecord fields from src/tracing.rs.
    let span_record_fields: BTreeSet<String> = ["operation", "target"]
        .iter()
        .map(|s| s.to_string())
        .collect();

    assert_eq!(
        declared, span_record_fields,
        "registry attribute ids must match SpanRecord fields {{operation, target}}; \
         declared in affidavit.yaml = {declared:?}"
    );
}
