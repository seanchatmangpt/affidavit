//! SJ projection parity court (register row 44).
//!
//! Lane scope: tests/sj_projection_parity.rs + one disclosed, documented
//! [dev-dependencies] entry in Cargo.toml (`clnrm-sj-core`, a renamed path dep
//! on the REAL /Users/sac/clnrm/crates/clnrm-core — never the stubs/ fence).
//!
//! SCOPE LAW (lane law): this court consumes ONLY the clnrm-core
//! projection/shape surface:
//!   receipts::{SjReceipt, SjIdentity, SjReplay, SjStanding,
//!              to_sj_receipt, to_sj_receipt_resolved, to_sj_ttl, ReceiptStore,
//!              TestReceipt (as the shape carrier)}
//! The sha2-based digest functions inside clnrm-core (TestReceipt::compute_id,
//! ContentHash::from_bytes) are NEVER consumed into any affidavit receipt
//! commitment. Affidavit receipts and their commitments stay BLAKE3-only
//! (src/chain.rs). sha2 content addresses appear only as clnrm-internal
//! identifiers needed to make the SjReceipt shape constructible and its
//! standing law exercisable; they are asserted never to equal any BLAKE3
//! commitment of the affidavit chain (differential-scope assertion).
//!
//! Contract under test: the 5-field receipt contract
//! {identity, authority, consequence, replay, standing}.
//!
//! Standing vocabulary mapping (asserted below):
//!   clnrm `SjStanding::ALIVE`   → repo `RepoStanding::Alive`         (ALIVE)
//!   clnrm `SjStanding::BLOCKED` → repo `RepoStanding::Blocked` (BLOCKED)
//!   clnrm's vocabulary is a strict 2-element subset of the repo's
//!   UNKNOWN|PARTIAL_ALIVE|ALIVE|BLOCKED|BUILD_BROKEN|UNSUPPORTED set plus
//!   typed REFUSED; the 5 unmapped repo standings have no clnrm carrier and
//!   are asserted to fail typed, never silently.

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use clnrm_sj_core::capabilities::scenario::ScenarioId;
use clnrm_sj_core::environment::sigma::ContentHash;
use clnrm_sj_core::receipts::sj_projection::to_sj_receipt_resolved;
use clnrm_sj_core::receipts::{
    to_sj_receipt, to_sj_ttl, ReceiptStore, SjReceipt, SjStanding, TestReceipt,
};

const SIGMA: &str = "affidavit-core-v1";

/// The repo's standing vocabulary (src/sj_record.rs `RepoStanding`), as the
/// schema strings it serializes to. Local copy: `sj_record` is behind the
/// `crypto-trust` feature plane; the parity contract binds at the string
/// vocabulary level, which is the schema-facing plane.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RepoStanding {
    Unknown,
    PartialAlive,
    Alive,
    Blocked,
    BuildBroken,
    Unsupported,
    Refused(String),
}

impl RepoStanding {
    fn as_str(&self) -> String {
        match self {
            RepoStanding::Unknown => "UNKNOWN".to_string(),
            RepoStanding::PartialAlive => "PARTIAL_ALIVE".to_string(),
            RepoStanding::Alive => "ALIVE".to_string(),
            RepoStanding::Blocked => "BLOCKED".to_string(),
            RepoStanding::BuildBroken => "BUILD_BROKEN".to_string(),
            RepoStanding::Unsupported => "UNSUPPORTED".to_string(),
            RepoStanding::Refused(r) => format!("REFUSED({r})"),
        }
    }
}

/// Build an affidavit 5-event receipt chain through the public
/// emit/assemble API (`ChainAssembler` + `build_event`) and finalize into the
/// sealed, immutable `Receipt`.
fn affidavit_chain() -> (
    affidavit::types::Receipt,
    Vec<affidavit::types::OperationEvent>,
) {
    let event_types = ["build", "test", "audit-log", "test", "commit"];
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for (seq, ty) in event_types.iter().enumerate() {
        let ev = build_event(
            *ty,
            vec![object_ref(format!("obj:{seq}").as_str(), "artifact")],
            format!("payload-{seq}").as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    assert_eq!(asm.events().len(), 5, "5-event chain assembled");
    let events: Vec<_> = asm.events().to_vec();
    (asm.finalize(), events)
}

/// Project an affidavit event chain into clnrm SjReceipt shapes.
///
/// Bridge per the 5-field contract:
/// - identity: `SjIdentity { id, subject, base_sha }` = clnrm content
///   address, affidavit event identity, pinned ontology hash.
/// - authority: `SjReceipt` exposes no authority field; the contract cell is
///   carried by `TestReceipt.metadata["sj:authority"]` (covered by the
///   receipt content address, so tamper-evident). Registered PARITY GAP.
/// - consequence: carried as the scenario subject plus
///   `metadata["sj:consequence"]` (event type + payload commitment).
/// - replay: `SjReplay { commands, durable_location }`.
/// - standing: computed by the projection law, never declared.
fn bridge_to_sj(
    events: &[affidavit::types::OperationEvent],
    authority: &str,
) -> (Vec<TestReceipt>, ReceiptStore) {
    let store = ReceiptStore::new();
    let mut prev: Option<ContentHash> = None;
    let mut out = Vec::new();
    for ev in events {
        let mut r = TestReceipt {
            scenario_id: ScenarioId::from(format!("affidavit:{}:{}", ev.seq, ev.event_type)),
            sigma_hash: ContentHash::from_string(SIGMA),
            previous_receipt: prev.clone(),
            ..TestReceipt::default()
        };
        r.metadata
            .insert("sj:authority".to_string(), authority.to_string());
        r.metadata.insert(
            "sj:consequence".to_string(),
            format!("{}#{}", ev.event_type, ev.payload_commitment.as_hex()),
        );
        r.id = r.compute_id();
        store.put(r.clone()).expect("store put");
        prev = Some(r.id.clone());
        out.push(r);
    }
    (out, store)
}

/// The repo's standing vocabulary, as strings (incl. one typed-REFUSED form).
const REPO_VOCAB: [&str; 7] = [
    "UNKNOWN",
    "PARTIAL_ALIVE",
    "ALIVE",
    "BLOCKED",
    "BUILD_BROKEN",
    "UNSUPPORTED",
    "REFUSED(reason)",
];

/// Map a clnrm standing token onto the repo vocabulary. Typed failure
/// (`OUT_OF_VOCAB:<token>`) for anything outside the repo's set — an
/// out-of-vocab standing never maps silently.
fn map_standing(clnrm_token: &str) -> Result<RepoStanding, String> {
    match clnrm_token {
        "ALIVE" => Ok(RepoStanding::Alive),
        "BLOCKED" => Ok(RepoStanding::Blocked),
        other => Err(format!("OUT_OF_VOCAB:{other}")),
    }
}

/// Map a repo standing onto the clnrm vocabulary. Only the 2-element shared
/// subset has a carrier; the other repo standings fail typed
/// (`NO_CLNRM_CARRIER:<standing>`), never a silent lossy round-up.
fn map_standing_to_clnrm(v: &RepoStanding) -> Result<&'static str, String> {
    match v {
        RepoStanding::Alive => Ok("ALIVE"),
        RepoStanding::Blocked => Ok("BLOCKED"),
        other => Err(format!("NO_CLNRM_CARRIER:{}", other.as_str())),
    }
}

#[test]
fn sj_field_for_field_identity_5_receipt_chain() {
    let (receipt, events) = affidavit_chain();
    assert_eq!(receipt.events.len(), 5, "5-receipt chain assembled");
    let (clnrm_receipts, store) = bridge_to_sj(&events, "affidavit/core-v1");

    assert_eq!(clnrm_receipts.len(), 5);
    for (seq, (r, ev)) in clnrm_receipts.iter().zip(events.iter()).enumerate() {
        let sj = to_sj_receipt_resolved(r, &store);
        // identity
        assert_eq!(
            sj.identity.id,
            r.id.as_str(),
            "identity.id = content address"
        );
        assert_eq!(
            sj.identity.subject,
            format!("affidavit:{}:{}", seq, ev.event_type),
            "identity.subject = event identity"
        );
        assert_eq!(
            sj.identity.base_sha, SIGMA,
            "identity.base_sha = ontology hash"
        );
        // authority (parity-gap carrier, hash-covered by content address)
        assert_eq!(
            r.metadata.get("sj:authority").map(String::as_str),
            Some("affidavit/core-v1"),
            "authority cell carried hash-covered"
        );
        // consequence (parity-gap carrier, hash-covered by content address)
        assert_eq!(
            r.metadata.get("sj:consequence").map(String::as_str),
            Some(format!("{}#{}", ev.event_type, ev.payload_commitment.as_hex()).as_str()),
            "consequence cell carried hash-covered"
        );
        // replay
        assert_eq!(
            sj.replay.commands,
            vec!["weaver proof verify".to_string()],
            "replay.commands fixed set"
        );
        assert_eq!(
            sj.replay.durable_location,
            format!("clnrm://receipts/{}", r.id.as_str()),
            "replay.durable_location = content-addressed key"
        );
        // standing: ALIVE — full chain resolves through the store
        assert_eq!(sj.standing, SjStanding::ALIVE, "resolved chain → ALIVE");
        // receipt (chain link): genesis anchors own id; links carry prev id
        if seq == 0 {
            assert_eq!(sj.receipt, r.id.as_str(), "genesis anchors own id");
        } else {
            assert_eq!(
                sj.receipt,
                clnrm_receipts[seq - 1].id.as_str(),
                "link carries previous id"
            );
        }
    }
}

#[test]
fn sj_standing_vocabulary_maps_1_to_1() {
    // clnrm → repo, 1:1 on the shared subset.
    assert_eq!(map_standing("ALIVE"), Ok(RepoStanding::Alive));
    assert_eq!(map_standing("BLOCKED"), Ok(RepoStanding::Blocked));
    // Repo → clnrm on the same subset.
    assert_eq!(map_standing_to_clnrm(&RepoStanding::Alive), Ok("ALIVE"));
    assert_eq!(map_standing_to_clnrm(&RepoStanding::Blocked), Ok("BLOCKED"));
    // clnrm's vocabulary is a strict subset of the repo vocabulary.
    for s in [SjStanding::ALIVE, SjStanding::BLOCKED] {
        let token = s.as_str().trim_start_matches("sj:");
        assert!(
            REPO_VOCAB.contains(&token),
            "{token} is in the repo vocabulary"
        );
    }
    // Repo standings with no clnrm carrier → typed failure, never silent.
    for v in [
        RepoStanding::Unknown,
        RepoStanding::PartialAlive,
        RepoStanding::BuildBroken,
        RepoStanding::Unsupported,
        RepoStanding::Refused("REFUSED_TEST".to_string()),
    ] {
        let mapped = map_standing_to_clnrm(&v);
        assert!(mapped.is_err(), "{} must not map silently", v.as_str());
        assert_eq!(
            mapped.unwrap_err(),
            format!("NO_CLNRM_CARRIER:{}", v.as_str()),
            "typed carrier-gap failure"
        );
    }
}

#[test]
fn sj_out_of_vocab_standing_fails_typed() {
    for bad in ["DEGRADED", "sj:ALIVE", "alive", "ALMOST_ALIVE", ""] {
        let err = map_standing(bad).unwrap_err();
        assert_eq!(
            err,
            format!("OUT_OF_VOCAB:{bad}"),
            "typed prefix on {bad:?}"
        );
    }
    // Positive control: in-vocab tokens still map (the court can fail).
    assert!(map_standing("ALIVE").is_ok());
    assert!(map_standing("BLOCKED").is_ok());
}

#[test]
fn sj_mutation_control_broken_chain_link_refuses() {
    let (_, events) = affidavit_chain();
    let (clnrm_receipts, store) = bridge_to_sj(&events, "affidavit/core-v1");
    let head = clnrm_receipts.last().unwrap();

    // Baseline (positive control): resolved projection of the head is ALIVE.
    let head_sj = to_sj_receipt_resolved(head, &store);
    assert_eq!(head_sj.standing, SjStanding::ALIVE, "baseline head ALIVE");

    // Mutation 1: forge a head whose previous link points outside the store
    // — the resolved projection must refuse it: BLOCKED (broken link).
    let mut forged = head.clone();
    forged.previous_receipt = Some(ContentHash::from_string("f".repeat(64)));
    forged.id = forged.compute_id();
    let forged_sj = to_sj_receipt_resolved(&forged, &store);
    assert_eq!(
        forged_sj.standing,
        SjStanding::BLOCKED,
        "broken link → BLOCKED"
    );

    // Mutation 2: tamper a receipt body without recomputing its content
    // address — validate() refuses (declared id != computed id); the
    // projection law rounds nothing up: BLOCKED.
    let mut tampered = head.clone();
    tampered.scenario_id = ScenarioId::from("tampered".to_string());
    assert_eq!(to_sj_receipt(&tampered).standing, SjStanding::BLOCKED);

    // Mutation 3 (store plane): ReceiptStore.put refuses an orphan whose
    // previous link is absent from the store.
    let mut orphan = TestReceipt::default();
    orphan.previous_receipt = Some(ContentHash::from_string("f".repeat(64)));
    orphan.id = orphan.compute_id();
    assert!(store.put(orphan).is_err(), "store refuses orphan link");

    // Mutation 4 (affidavit plane): tamper the serialized receipt —
    // deserialization re-verifies the rolling BLAKE3 chain and refuses.
    let (receipt, _) = affidavit_chain();
    let mut bytes = affidavit::chain::serialize_receipt(&receipt).expect("serialize");
    let key = b"chain_hash\":";
    let kpos = bytes
        .windows(key.len())
        .position(|w| w == key)
        .expect("chain_hash key present");
    let q1 = bytes[kpos..].iter().position(|b| *b == b'"').unwrap() + kpos;
    bytes[q1 + 1] ^= 0x01;
    assert!(
        affidavit::chain::deserialize_receipt(&bytes).is_err(),
        "tampered affidavit chain refused at deserialize"
    );
}

#[test]
fn sj_ttl_projection_is_deterministic_and_scope_clean() {
    let (receipt, events) = affidavit_chain();
    let (clnrm_receipts, store) = bridge_to_sj(&events, "affidavit/core-v1");
    let projected: Vec<SjReceipt> = clnrm_receipts
        .iter()
        .map(|r| to_sj_receipt_resolved(r, &store))
        .collect();

    let ttl1 = to_sj_ttl(&projected);
    let ttl2 = to_sj_ttl(&projected);
    assert_eq!(ttl1, ttl2, "deterministic TTL for the same input set");
    for r in &projected {
        assert!(ttl1.contains(&r.identity.id), "each identity emitted");
    }
    assert!(ttl1.contains("sj:ALIVE"), "ALIVE token emitted");
    assert!(!ttl1.contains("sj:BLOCKED"), "nothing rounds up");

    // sha2 scope law: no clnrm sha2 content address ever equals an affidavit
    // BLAKE3 commitment (both hex; assert disjoint SETS, not lengths).
    let mut blake3_set: std::collections::HashSet<String> = events
        .iter()
        .map(|ev| ev.payload_commitment.as_hex().to_string())
        .collect();
    blake3_set.insert(receipt.chain_hash.as_hex().to_string());
    for r in &clnrm_receipts {
        assert!(
            !blake3_set.contains(r.id.as_str()),
            "sha2 content address must never appear as an affidavit commitment"
        );
    }
}
