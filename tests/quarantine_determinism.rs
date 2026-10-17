//! Court: the `affi receipt fix` quarantine sidecar
//! (`<stem>.quarantine.why.json`) must be a pure function of the subject
//! receipt — same subject, cmp-identical sidecar bytes, any day it runs.
//! The former `quarantined_at` wall-clock field was a determinism defect;
//! it is replaced by `subject_chain_hash`, the subject receipt's own chain
//! hash (receipt-grounded, replayable).

use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

/// Build a REJECT subject: a structurally parseable receipt whose stored
/// chain_hash does not recompute (fails the chain_integrity stage).
fn write_reject_subject(tag: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join(format!("subject-{tag}.json"));
    let body = format!(
        "{{\"format_version\":\"core/v1\",\"events\":[],\"chain_hash\":\"{}\"}}",
        "a".repeat(64)
    );
    std::fs::write(&path, &body).expect("write subject");
    (dir, path)
}

fn sidecar_path(subject: &Path) -> PathBuf {
    let stem = subject.file_stem().and_then(|s| s.to_str()).expect("stem");
    subject.with_file_name(format!("{stem}.quarantine.why.json"))
}

fn sidecar_bytes(subject: &Path) -> Vec<u8> {
    std::fs::read(sidecar_path(subject)).expect("sidecar written")
}

/// Vacuity gate: the quarantine action must actually quarantine — the
/// subject is renamed and the sidecar written; otherwise the determinism
/// court could pass vacuously over a no-op.
#[test]
fn quarantine_actually_moves_subject_and_writes_sidecar() {
    let (_dir, subject) = write_reject_subject("vacuity");
    affidavit::handlers::fix_receipt(
        subject.display().to_string(),
        Some("quarantine".to_string()),
        false,
        None,
    )
    .expect("fix_receipt (quarantine) succeeds");
    assert!(
        !subject.exists(),
        "subject must have been renamed out of place"
    );
    let sidecar = sidecar_bytes(&subject);
    assert!(!sidecar.is_empty(), "sidecar must be non-empty");
    let text = String::from_utf8(sidecar).expect("utf8 sidecar");
    assert!(
        text.contains("subject_chain_hash"),
        "sidecar must ground on the subject chain hash: {text}"
    );
    assert!(
        !text.contains("quarantined_at"),
        "wall-clock field must be gone: {text}"
    );
}

/// The determinism law: two runs over the same REJECT subject produce
/// cmp-identical sidecar bytes. The subject is restored to the same path
/// between runs so the whole file — including the `original` path field —
/// is compared byte-for-byte, like `cmp`.
#[test]
fn quarantine_sidecar_is_deterministic_across_runs() {
    let (_dir, subject) = write_reject_subject("det");
    let subject_str = subject.display().to_string();
    let subject_bytes = std::fs::read(&subject).expect("read subject");
    let sidecar = sidecar_path(&subject);

    affidavit::handlers::fix_receipt(
        subject_str.clone(),
        Some("quarantine".to_string()),
        false,
        None,
    )
    .expect("run 1");
    let run1 = sidecar_bytes(&subject);

    // Restore the subject at the identical path and run again.
    let _ = std::fs::remove_file(&sidecar);
    let _ = std::fs::remove_file(subject.with_file_name(format!(
        "{}.quarantine.json",
        subject.file_stem().and_then(|s| s.to_str()).expect("stem")
    )));
    std::fs::write(&subject, &subject_bytes).expect("restore subject");

    affidavit::handlers::fix_receipt(subject_str, Some("quarantine".to_string()), false, None)
        .expect("run 2");
    let run2 = sidecar_bytes(&subject);

    assert_eq!(
        run1, run2,
        "cmp-identical sidecar bytes required for the same subject"
    );
}
