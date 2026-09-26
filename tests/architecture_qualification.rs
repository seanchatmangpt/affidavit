//! Adversarial falsifiers for architecture qualification receipts (v26.9.26).
//!
//! Chicago style: every assertion runs the real certify/replay/supersede code
//! and inspects the real returned receipt or typed refusal. No test doubles.

use affidavit::{
    ArchitectureQualificationReceipt as Receipt, ArchitectureRefusal as Refusal,
    ArchitectureStanding as Standing, ARCHITECTURE_RECEIPT_SCHEMA,
};
use std::time::Instant;

const ABB: &str = "sha256:abb-0001";
const CONTRACT: &str = "sha256:contract-0001";
const SBB: &str = "sha256:sbb-0001";
const SUBJECT: &str = "git:fb09f991fab7995c1b7564f668065ef379303800";
const PRODUCER: &str = "sha256:producer-autofde-lab";

fn ev(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn certify_with(evidence: Vec<String>, artifacts: Vec<String>) -> Result<Receipt, Refusal> {
    Receipt::certify(
        ABB,
        CONTRACT,
        SBB,
        SUBJECT,
        evidence,
        PRODUCER,
        artifacts,
        Standing::Qualified,
    )
}

fn qualified() -> Receipt {
    certify_with(
        ev(&["sha256:ev-xaas", "sha256:ev-autofde", "sha256:ev-runtime"]),
        ev(&["sha256:art-b", "sha256:art-a"]),
    )
    .expect("well-formed receipt certifies")
}

fn replay(r: &Receipt) -> Result<(), Refusal> {
    r.verify_replay(ABB, CONTRACT, SBB, SUBJECT)
}

#[test]
fn well_formed_receipt_replays_and_never_confers_do() {
    let r = qualified();
    assert_eq!(r.schema, ARCHITECTURE_RECEIPT_SCHEMA);
    assert!(!r.confers_do_authority);
    assert!(r.receipt_digest.starts_with("blake3:"));
    assert_eq!(r.receipt_digest.len(), "blake3:".len() + 64);
    assert_eq!(replay(&r), Ok(()));
}

#[test]
fn reordered_evidence_yields_identical_receipt() {
    let a = qualified();
    let b = certify_with(
        ev(&["sha256:ev-runtime", "sha256:ev-xaas", "sha256:ev-autofde"]),
        ev(&["sha256:art-a", "sha256:art-b"]),
    )
    .unwrap();
    assert_eq!(a, b);
    assert_eq!(a.receipt_digest, b.receipt_digest);
}

#[test]
fn duplicate_evidence_delivery_is_idempotent() {
    let a = qualified();
    let b = certify_with(
        ev(&[
            "sha256:ev-xaas",
            "sha256:ev-autofde",
            "sha256:ev-xaas",
            "sha256:ev-runtime",
            "sha256:ev-autofde",
        ]),
        ev(&["sha256:art-a", "sha256:art-b", "sha256:art-a"]),
    )
    .unwrap();
    assert_eq!(a.receipt_digest, b.receipt_digest);
    assert_eq!(b.qualification_evidence_digests.len(), 3);
    assert_eq!(b.artifact_digests.len(), 2);
}

#[test]
fn distinct_evidence_changes_the_digest() {
    let a = qualified();
    let b = certify_with(ev(&["sha256:ev-other"]), ev(&["sha256:art-a"])).unwrap();
    assert_ne!(a.receipt_digest, b.receipt_digest);
}

#[test]
fn malformed_digests_are_refused_per_field() {
    let cases: &[(&str, &str, &str, &str, &str, &str)] = &[
        ("", CONTRACT, SBB, SUBJECT, PRODUCER, "abb_digest"),
        ("abb", CONTRACT, SBB, SUBJECT, PRODUCER, "abb_digest"),
        (ABB, "sha256:", SBB, SUBJECT, PRODUCER, "contract_digest"),
        (ABB, CONTRACT, ":deadbeef", SUBJECT, PRODUCER, "sbb_digest"),
        (
            ABB,
            CONTRACT,
            SBB,
            "SHA256:x",
            PRODUCER,
            "exact_subject_digest",
        ),
        (
            ABB,
            CONTRACT,
            SBB,
            "sha256:a b",
            PRODUCER,
            "exact_subject_digest",
        ),
        (ABB, CONTRACT, SBB, SUBJECT, "producer", "producer_digest"),
    ];
    for (abb, contract, sbb, subject, producer, field) in cases {
        let got = Receipt::certify(
            *abb,
            *contract,
            *sbb,
            *subject,
            ev(&["sha256:ev"]),
            *producer,
            vec![],
            Standing::Qualified,
        );
        assert_eq!(
            got,
            Err(Refusal::MalformedDigest { field }),
            "case abb={abb} contract={contract} sbb={sbb} subject={subject}"
        );
    }
    assert_eq!(
        certify_with(ev(&["sha256:ok", "not-a-digest"]), vec![]),
        Err(Refusal::MalformedDigest {
            field: "qualification_evidence_digests"
        })
    );
    assert_eq!(
        certify_with(ev(&["sha256:ok"]), ev(&["sha256:\t"])),
        Err(Refusal::MalformedDigest {
            field: "artifact_digests"
        })
    );
}

#[test]
fn empty_subject_is_missing_not_malformed() {
    assert_eq!(
        Receipt::certify(
            ABB,
            CONTRACT,
            SBB,
            "",
            ev(&["sha256:ev"]),
            PRODUCER,
            vec![],
            Standing::Qualified
        ),
        Err(Refusal::MissingExactSubject)
    );
}

#[test]
fn forged_do_authority_is_refused() {
    let mut r = qualified();
    r.confers_do_authority = true;
    assert_eq!(replay(&r), Err(Refusal::DoAuthorityForbidden));
    assert_eq!(r.verify_integrity(), Err(Refusal::DoAuthorityForbidden));
}

#[test]
fn tampered_fields_fail_replay() {
    let base = qualified();

    let mut forged_evidence = base.clone();
    forged_evidence
        .qualification_evidence_digests
        .push("sha256:zz-forged".into());
    assert_eq!(replay(&forged_evidence), Err(Refusal::ReplayMismatch));

    let mut forged_standing = base.clone();
    forged_standing.standing = Standing::Candidate;
    assert_eq!(replay(&forged_standing), Err(Refusal::ReplayMismatch));

    let mut forged_producer = base.clone();
    forged_producer.producer_digest = "sha256:someone-else".into();
    assert_eq!(replay(&forged_producer), Err(Refusal::ReplayMismatch));

    let mut forged_digest = base.clone();
    forged_digest.receipt_digest = format!("blake3:{}", "0".repeat(64));
    assert_eq!(replay(&forged_digest), Err(Refusal::ReplayMismatch));

    let mut forged_prior = base.clone();
    forged_prior.prior_receipt_digest = Some("blake3:ghost".into());
    assert_eq!(replay(&forged_prior), Err(Refusal::ReplayMismatch));
}

#[test]
fn unknown_standing_smuggled_after_certification_is_refused() {
    let mut r = qualified();
    r.standing = Standing::Unknown;
    assert_eq!(replay(&r), Err(Refusal::UnknownPromotion));
}

#[test]
fn evidence_stripped_after_certification_is_refused() {
    let mut r = qualified();
    r.qualification_evidence_digests.clear();
    assert_eq!(replay(&r), Err(Refusal::MissingEvidence));
}

#[test]
fn non_canonical_evidence_order_is_refused_on_replay() {
    let mut r = qualified();
    r.qualification_evidence_digests.reverse();
    assert_eq!(replay(&r), Err(Refusal::ReplayMismatch));
    let mut dup = qualified();
    let first = dup.artifact_digests[0].clone();
    dup.artifact_digests.insert(0, first);
    assert_eq!(replay(&dup), Err(Refusal::ReplayMismatch));
}

#[test]
fn wrong_schema_is_refused() {
    let mut r = qualified();
    r.schema = "affidavit.architecture-qualification.v0".into();
    assert_eq!(replay(&r), Err(Refusal::SchemaMismatch));
}

#[test]
fn stale_subject_is_cross_subject_reuse() {
    let r = qualified();
    assert_eq!(
        r.verify_replay(
            ABB,
            CONTRACT,
            SBB,
            "git:0000000000000000000000000000000000000000"
        ),
        Err(Refusal::CrossSubjectReuse)
    );
}

#[test]
fn supersession_requires_a_real_replacement_and_intact_prior() {
    let current = qualified();
    assert_eq!(
        current.supersede(SBB, "git:next", ev(&["sha256:ev"])),
        Err(Refusal::NotAReplacement)
    );

    let mut tampered = current.clone();
    tampered.producer_digest = "sha256:forged".into();
    assert_eq!(
        tampered.supersede("sha256:sbb-0002", "git:next", ev(&["sha256:ev"])),
        Err(Refusal::ReplayMismatch)
    );

    assert_eq!(
        current.supersede("sha256:sbb-0002", "git:next", vec![]),
        Err(Refusal::MissingEvidence)
    );
}

#[test]
fn chain_verification_detects_wrong_prior_and_reuse() {
    let a = qualified();
    let b = a
        .supersede("sha256:sbb-0002", "git:next", ev(&["sha256:ev-b"]))
        .unwrap();
    assert_eq!(b.verify_chain(&a), Ok(()));
    assert_eq!(
        b.verify_replay(ABB, CONTRACT, "sha256:sbb-0002", "git:next"),
        Ok(())
    );

    // A second-generation link verifies against its own prior only.
    let c = b
        .supersede("sha256:sbb-0003", "git:next2", ev(&["sha256:ev-c"]))
        .unwrap();
    assert_eq!(c.verify_chain(&b), Ok(()));
    assert_eq!(c.verify_chain(&a), Err(Refusal::ChainBroken));

    // A fresh qualified receipt is not a supersession link.
    assert_eq!(a.verify_chain(&a), Err(Refusal::ChainBroken));

    // A tampered prior is refused before the link is considered.
    let mut forged_prior = a.clone();
    forged_prior.contract_digest = "sha256:contract-9999".into();
    assert_eq!(b.verify_chain(&forged_prior), Err(Refusal::ReplayMismatch));
}

#[test]
fn json_roundtrip_is_verified_and_unknown_fields_are_refused() {
    let r = qualified();
    let json = r.to_json();
    assert_eq!(Receipt::from_json_verified(&json), Ok(r.clone()));

    let smuggled = json.replacen('{', "{\"do_lease\":\"granted\",", 1);
    assert_eq!(
        Receipt::from_json_verified(&smuggled),
        Err(Refusal::Malformed)
    );
    assert_eq!(
        Receipt::from_json_verified("{not json"),
        Err(Refusal::Malformed)
    );
    let forged = json.replace(
        "\"confers_do_authority\":false",
        "\"confers_do_authority\":true",
    );
    assert_ne!(forged, json);
    assert_eq!(
        Receipt::from_json_verified(&forged),
        Err(Refusal::DoAuthorityForbidden)
    );
}

#[test]
fn refusal_display_is_typed() {
    assert_eq!(
        Refusal::MalformedDigest {
            field: "sbb_digest"
        }
        .to_string(),
        "MALFORMED_DIGEST[sbb_digest]"
    );
    assert_eq!(Refusal::ChainBroken.to_string(), "ChainBroken");
}

/// Regression bound (deterministic timing). Measured on the v26.9.26 lane
/// (aarch64-apple-darwin, debug profile): see
/// `benches/architecture_receipts_baseline.json`. The bound is ~10x the
/// measured debug cost so it only trips on an algorithmic regression.
#[test]
fn certify_and_replay_stay_within_regression_bound() {
    const N: usize = 2_000;
    let evidence: Vec<String> = (0..16).map(|i| format!("sha256:ev-{i:04}")).collect();
    let start = Instant::now();
    for _ in 0..N {
        let r = certify_with(evidence.clone(), ev(&["sha256:art-a"])).unwrap();
        replay(&r).unwrap();
    }
    let per_op_us = start.elapsed().as_secs_f64() * 1e6 / N as f64;
    assert!(
        per_op_us < 2_000.0,
        "certify+replay regressed: {per_op_us:.1} us/op (bound 2000 us/op)"
    );
}
