//! SJ-aligned record court (backlog [19]): adversarial + lifecycle proof over
//! the REAL chain assembler, REAL JCS substrate, and REAL digest law — no
//! mocks, assertions on final state (Chicago school).
//!
//! A test PASSING means the attack was REFUSED. Standing: ALIVE if green at
//! this commit.

#![cfg(feature = "crypto-trust")]

use affidavit::sj_record::{
    Authority, AuthorityCeiling, BrokenTerm, CommitRecord, ReplayCommand, SjCampaign,
    SjCampaignDraft, SjRecord, SjRefusal, StandingValue,
};

/// A fully admitted baseline draft; every attack mutates one face.
fn baseline_draft() -> SjCampaignDraft {
    SjCampaignDraft {
        work_order_id: "AFFI-26923-19".to_string(),
        origin_ceiling: Some(AuthorityCeiling::Construct),
        origin_grant: "lease-771".to_string(),
        origin_actor: "lane:aff-record".to_string(),
        provider_name: "affidavit.cli".to_string(),
        provider_execution_id: "exec-19".to_string(),
        subject: "sj-aligned record".to_string(),
        repo: "affidavit".to_string(),
        subject_sha: "a".repeat(40),
        base_sha: "b".repeat(40),
        commits: vec![CommitRecord {
            sha: format!("1{}", "c".repeat(39)),
            summary: "feat(affidavit): sj-aligned record implementation".to_string(),
            court_results: vec!["verify:ACCEPT".to_string()],
        }],
        residue_declaration: "markdown companion stays human-facing".to_string(),
        files_changed: vec!["src/sj_record.rs".to_string()],
        remote_effects: vec!["push:branch".to_string()],
        replay_commands: vec![ReplayCommand {
            cmd: "cargo test --features crypto-trust --test sj_record".to_string(),
            exit: 0,
            cwd: "/Users/sac/affidavit".to_string(),
            summary: Some("new-record court".to_string()),
            output_sha256: None,
        }],
        durable_location: Some("docs/sjira/v26.10.8/".to_string()),
        standing: StandingValue::Alive,
        derived_from: "cargo test --features crypto-trust --test sj_record".to_string(),
        broken_term: None,
        predecessor_work_order_ids: vec![],
        authority: Authority {
            ceiling: AuthorityCeiling::Do,
            grant: "lease-771".to_string(),
            actor: "lane:aff-record".to_string(),
        },
    }
}

#[test]
fn lifecycle_and_wire_roundtrip_verify() {
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();

    // Digest-bound identity: 64-hex blake3, re-derived from the sealed base.
    let digest_hex = record.subject_digest_hex().unwrap();
    assert_eq!(digest_hex.len(), 64);
    assert!(digest_hex.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(
        record.document.identity.subject_digest.value, digest_hex,
        "minted document must carry the re-derived digest"
    );
    assert_eq!(record.document.identity.subject_digest.algorithm, "blake3");

    // Chain-head binding + event claims match the carrier.
    assert_eq!(
        record.document.replay_binding.chain_head_hash,
        record.base.chain_hash.as_hex().to_string()
    );
    let claimed: Vec<_> = record.base.events.iter().map(|e| e.id.clone()).collect();
    assert_eq!(record.document.replay_binding.event_ids, claimed);
    assert_eq!(record.base.events.len(), 3); // commit + court + residue

    record.verify().unwrap();

    // Wire roundtrip re-verifies on deserialize.
    let json = record.to_json().unwrap();
    let roundtripped = SjRecord::from_json(&json).unwrap();
    assert_eq!(roundtripped.document, record.document);
    assert_eq!(roundtripped.base, record.base);
}

#[test]
fn canonical_json_is_deterministic_across_producers() {
    // Falsifier 2 of the design doc: two producers, same campaign record,
    // same subject_digest (JCS law).
    let a = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let b = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    assert_eq!(
        a.canonical_json().unwrap(),
        b.canonical_json().unwrap(),
        "JCS law broken: identical records produced different canonical bytes"
    );
    assert_eq!(
        a.subject_digest_hex().unwrap(),
        b.subject_digest_hex().unwrap()
    );
}

#[test]
fn tampered_commit_sha_refuses_as_event_claim_mismatch() {
    // Design falsifier 1: an edited commit payload must not deserialize.
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let attacked = record.clone();
    let json = attacked.to_json().unwrap();

    // Flip one hex digit of the commit sha inside the serialized document.
    let needle = format!("\"commits\":[\"1{}\"", "c".repeat(39));
    let flipped = format!("\"commits\":[\"1{}\"", "d".repeat(39));
    assert!(json.contains(&needle));
    let tampered = json.replacen(&needle, &flipped, 1);
    let err = SjRecord::from_json(&tampered).unwrap_err();
    assert_eq!(err, SjRefusal::EventClaimMismatch);
    let _ = attacked;
}

#[test]
fn tampered_standing_refuses_via_residue_rebuild() {
    // The standing declaration rides IN the chain; flipping ALIVE→BUILD_BROKEN
    // cannot rebuild the carrier events.
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let mut attacked = record.clone();
    attacked.document.standing.value = StandingValue::BuildBroken;
    // Keep the schema's broken_term conditional satisfied so the refusal is
    // the chain law (event rebuild), not the document admission gate.
    attacked.document.standing.broken_term = Some(BrokenTerm::AdmissionVacuous);
    assert_eq!(
        attacked.verify().unwrap_err(),
        SjRefusal::EventClaimMismatch
    );
}

#[test]
fn tampered_chain_head_refuses() {
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let mut attacked = record.clone();
    attacked.document.replay_binding.chain_head_hash = "0".repeat(64);
    assert_eq!(attacked.verify().unwrap_err(), SjRefusal::ChainHeadMismatch);
}

#[test]
fn tampered_subject_digest_refuses() {
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let mut attacked = record.clone();
    attacked.document.identity.subject_digest.value = "f".repeat(64);
    assert_eq!(
        attacked.verify().unwrap_err(),
        SjRefusal::DigestMismatch {
            expected: record.subject_digest_hex().unwrap(),
            claimed: "f".repeat(64),
        }
    );
}

#[test]
fn blocked_standing_without_broken_term_refuses() {
    // Schema allOf conditional: BLOCKED/BUILD_BROKEN/REFUSED require
    // standing.broken_term.
    let mut draft = baseline_draft();
    draft.standing = StandingValue::Blocked(Some("LFS budget".to_string()));
    draft.broken_term = None;
    assert_eq!(
        SjCampaign::new(draft).unwrap_err(),
        SjRefusal::MissingBrokenTerm("BLOCKED:LFS budget".to_string())
    );

    // With the term, the same standing admits and verifies.
    let mut draft = baseline_draft();
    draft.standing = StandingValue::Blocked(Some("LFS budget".to_string()));
    draft.broken_term = Some(BrokenTerm::RNotFedBack);
    let record = SjCampaign::new(draft).unwrap().finalize().unwrap();
    record.verify().unwrap();
    assert_eq!(
        record.document.standing.value.as_str(),
        "BLOCKED:LFS budget"
    );
}

#[test]
fn bad_git_anchor_refuses_typed() {
    let mut draft = baseline_draft();
    draft.subject_sha = "deadbeef".to_string();
    assert_eq!(
        SjCampaign::new(draft).unwrap_err(),
        SjRefusal::BadSha {
            key: "identity.subject_sha",
            value: "deadbeef".to_string(),
        }
    );
}

#[test]
fn empty_replay_refuses_markdown_shaped_record() {
    // Design falsifier 4: a record still markdown-shaped (no replay
    // commands) must not pass schema admission.
    let mut draft = baseline_draft();
    draft.replay_commands.clear();
    assert_eq!(SjCampaign::new(draft).unwrap_err(), SjRefusal::EmptyReplay);
}

#[test]
fn empty_required_key_refuses_typed() {
    let mut draft = baseline_draft();
    draft.work_order_id = "  ".to_string();
    assert_eq!(
        SjCampaign::new(draft).unwrap_err(),
        SjRefusal::EmptyKey {
            key: "work_order_id",
        }
    );
}

// -- CDT local-severance test vector (spec: ggen
//    `docs/specs/cdt-revocation/README.md`, ported from `test-vector.rs`
//    grounded at a82fd95e). Real chain assembler, real BLAKE3, zero mocks
//    (Chicago).

/// Local check walk (spec section 2.2): resolve ancestors to their CURRENT
/// canonical versions, fold the path log, compare against the admission-time
/// root table. Sub-ms, offline.
fn cdt_path_log<'a>(
    current: &[&'a SjRecord],
    leaf: &SjRecord,
) -> Vec<affidavit::types::OperationEvent> {
    let by_id: std::collections::BTreeMap<&str, &SjRecord> = current
        .iter()
        .map(|r| (r.document.work_order_id.as_str(), *r))
        .collect();
    let mut order: Vec<&SjRecord> = vec![];
    let mut cur = leaf;
    loop {
        order.push(cur);
        match cur
            .document
            .replay_binding
            .predecessor_work_order_ids
            .as_slice()
        {
            [] => break,
            [one] => cur = by_id[one.as_str()],
            _ => panic!("vector uses single-parent chains"),
        }
    }
    order.reverse();
    order.iter().flat_map(|r| r.base.events.clone()).collect()
}

fn cdt_derivation_digest(events: &[affidavit::types::OperationEvent]) -> String {
    affidavit::chain::recompute_chain(events)
        .expect("canonical events always fold")
        .as_hex()
        .to_string()
}

/// 5-record chain WO1 <- WO2 <- WO3 <- WO4 <- WO5.
fn cdt_record_for(n: usize, tag: &str, preds: Vec<String>) -> SjRecord {
    let draft = SjCampaignDraft {
        work_order_id: format!("WO{n}"),
        origin_ceiling: Some(AuthorityCeiling::Construct),
        origin_grant: "lease-cdt".to_string(),
        origin_actor: "lane:cdt-revocation".to_string(),
        provider_name: "affidavit.cli".to_string(),
        provider_execution_id: format!("cdt-exec-{tag}{n}"),
        subject: "cdt revocation vector".to_string(),
        repo: "affidavit".to_string(),
        subject_sha: format!("{n:040}"),
        base_sha: "b".repeat(40),
        commits: vec![CommitRecord {
            sha: format!("{tag}{n}{}", "c".repeat(38)),
            summary: format!("record {n}"),
            court_results: vec!["verify:ACCEPT".to_string()],
        }],
        residue_declaration: "test vector residue".to_string(),
        files_changed: vec!["docs/specs/cdt-revocation/".to_string()],
        remote_effects: vec!["push:branch".to_string()],
        replay_commands: vec![ReplayCommand {
            cmd: format!("check WO{n}"),
            exit: 0,
            cwd: "/tmp/cdt-scratch".to_string(),
            summary: None,
            output_sha256: None,
        }],
        durable_location: None,
        standing: StandingValue::Alive,
        derived_from: "cargo test --features crypto-trust --test sj_record".to_string(),
        broken_term: None,
        predecessor_work_order_ids: preds,
        authority: Authority {
            ceiling: AuthorityCeiling::Do,
            grant: "lease-cdt".to_string(),
            actor: "lane:cdt-revocation".to_string(),
        },
    };
    SjCampaign::new(draft)
        .expect("draft admits")
        .finalize()
        .expect("record finalizes")
}

#[test]
fn cdt_revocation_local_severance_detection() {
    // 5-record chain WO1 <- WO2 <- WO3 <- WO4 <- WO5.
    let r1 = cdt_record_for(1, "a", vec![]);
    let r2 = cdt_record_for(2, "a", vec!["WO1".to_string()]);
    let r3 = cdt_record_for(3, "a", vec!["WO2".to_string()]);
    let r4 = cdt_record_for(4, "a", vec!["WO3".to_string()]);
    let r5 = cdt_record_for(5, "a", vec!["WO4".to_string()]);
    let admission = vec![&r1, &r2, &r3, &r4, &r5];

    // Admission roots X1..X5 (hand-verified math, spec section 4.2).
    let root_table: Vec<String> = admission
        .iter()
        .map(|r| cdt_derivation_digest(&cdt_path_log(&admission, r)))
        .collect();

    // Pre-revocation: every record's local check passes.
    for (i, r) in admission.iter().enumerate() {
        assert_eq!(
            cdt_derivation_digest(&cdt_path_log(&admission, r)),
            root_table[i],
            "WO{} must pass pre-revocation",
            i + 1
        );
    }

    // Revoke node 2: supersede WO2 with a distinct-commit WO2'. The canonical
    // version of WO2 swaps to WO2'; ledger current versions become
    // [R1, R2', R3, R4, R5].
    let r2_prime = cdt_record_for(2, "f", vec!["WO1".to_string()]);
    // Distinctness precondition (spec section 2.3): E2' != E2.
    assert_ne!(
        r2.base.events, r2_prime.base.events,
        "revocation must change bytes"
    );

    let current = vec![&r1, &r2_prime, &r3, &r4, &r5];
    let new_root_table: Vec<String> = current
        .iter()
        .map(|r| cdt_derivation_digest(&cdt_path_log(&current, r)))
        .collect();

    // Post-revocation local checks against the ADMISSION-TIME root table.
    // WO1 passes (sibling of the severed subtree stays valid); WO3, WO4, WO5
    // SEVERED. Edge identified at depth 2 (spec section 4.2).
    assert_eq!(
        cdt_derivation_digest(&cdt_path_log(&current, &r1)),
        root_table[0],
        "WO1 unaffected"
    );
    for i in [2usize, 3, 4] {
        assert_ne!(
            cdt_derivation_digest(&cdt_path_log(&current, current[i])),
            root_table[i],
            "WO{} must detect severance locally",
            i + 1
        );
    }
    // The refreshed canonical lineage is internally consistent.
    for i in 0..5 {
        assert_eq!(
            cdt_derivation_digest(&cdt_path_log(&current, current[i])),
            new_root_table[i],
            "current canonical lineage must be internally consistent"
        );
    }
    // Edge identification: first diverging prefix depth is 2 (WO2 -> WO3).
    let severed_at = (1..=5).find(|&d| {
        cdt_derivation_digest(&cdt_path_log(&current, current[d - 1])) != root_table[d - 1]
    });
    assert_eq!(severed_at, Some(2), "severed edge is WO2 -> WO3");

    // Mutation check: a "revocation" that leaves the bytes identical is not a
    // revocation. Reproduce the byte-identical case and confirm the
    // distinctness precondition (E2' != E2) would refuse it.
    let r2_byte_identical = cdt_record_for(2, "a", vec!["WO1".to_string()]);
    assert_eq!(
        r2.base.events, r2_byte_identical.base.events,
        "same inputs must reproduce byte-identical events (deterministic assembler)"
    );
    assert_eq!(
        r2.document.replay_binding.chain_head_hash,
        r2_byte_identical.document.replay_binding.chain_head_hash,
        "byte-identical events fold to the identical chain head"
    );
}

#[test]
fn jcs_number_law_refuses_beyond_2pow53() {
    // The canonical substrate refuses integers beyond 2^53 typed.
    let record = SjCampaign::new(baseline_draft())
        .unwrap()
        .finalize()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&record.to_json().unwrap()).unwrap();
    // Sanity: the real substrate accepts the record itself.
    assert!(affidavit::crypto_trust_canonical::jcs(&value).is_ok());
    // And refuses a 2^53 integer.
    let big: serde_json::Value = serde_json::from_str("{\"n\":9007199254740993}").unwrap();
    assert!(affidavit::crypto_trust_canonical::jcs(&big).is_err());
}
