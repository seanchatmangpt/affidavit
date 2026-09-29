//! The `core/v1` receipt: canonical bytes, the rolling BLAKE3 chain, and the
//! 7-stage certify pipeline — a faithful port of the root crate's
//! `types::canonical_bytes`, `chain::recompute_chain` and `verifier::verify`.
//!
//! Doctrine, preserved: this module *certifies* a receipt against the format
//! standard. It never decides whether the recorded work was honest.

use serde::{Deserialize, Serialize};

/// Format version this verifier certifies (root `chain::FORMAT_VERSION`).
pub const FORMAT_VERSION: &str = "core/v1";

/// Genesis seed. Derived from this crate's version, never typed, so a receipt
/// only verifies under the exact release that assembled it (root bug B4). The
/// root crate's release-identity tests hold this version equal to `affi`'s.
pub const GENESIS_SEED: &str = concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis");

const HASH_HEX_LEN: usize = 64;

/// A qualified object reference (root `types::ObjectRef`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    /// Stable identifier of the referenced object.
    pub id: String,
    /// OCEL object type.
    pub obj_type: String,
    /// Optional role qualifier (serialized as `null` when absent).
    #[serde(default)]
    pub qualifier: Option<String>,
}

/// One append-only operation-event (root `types::OperationEvent`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationEvent {
    /// Event identifier, unique within the receipt.
    pub id: String,
    /// Monotonic logical sequence number.
    pub seq: u64,
    /// Kind of operation recorded.
    pub event_type: String,
    /// Objects this event relates to.
    pub objects: Vec<ObjectRef>,
    /// BLAKE3 commitment (lowercase hex) to the payload bytes.
    pub payload_commitment: String,
}

/// A receipt as it arrives from a host. Deliberately permissive: the verifier,
/// not the parser, decides validity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    /// Format version string.
    pub format_version: String,
    /// Ordered events.
    pub events: Vec<OperationEvent>,
    /// Stored rolling chain hash (lowercase hex).
    pub chain_hash: String,
}

/// One stage's result (root `types::CheckOutcome`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckOutcome {
    /// Stage name, e.g. `chain_integrity`.
    pub stage: String,
    /// Whether the stage passed.
    pub passed: bool,
    /// Human-readable detail (identical wording to `affi verify`).
    pub detail: String,
}

/// The verdict: accepted iff every stage passed (root `types::Verdict`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Verdict {
    /// True iff all stages passed.
    pub accepted: bool,
    /// Profile evaluated, spelled as `affi verify --format json` spells it
    /// (the root `ProfileId` variant name, `CoreV1`).
    pub profile: &'static str,
    /// Per-stage outcomes in pipeline order.
    pub outcomes: Vec<CheckOutcome>,
    /// `all stages passed`, or `<stage>: <detail>` of the first failure.
    pub reason: String,
}

/// BLAKE3 of `bytes` as lowercase hex.
pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Canonical bytes: compact JSON with every object's keys sorted (byte-wise).
/// `serde_json::Map` is a `BTreeMap` (no `preserve_order`), so converting to a
/// `Value` sorts keys at every depth — identical to the root `canonical_bytes`.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&serde_json::to_value(value)?)
}

fn fold_event(prev_hex: &str, event: &OperationEvent) -> Result<String, serde_json::Error> {
    let event_bytes = canonical_bytes(event)?;
    let mut buf = Vec::with_capacity(prev_hex.len() + event_bytes.len());
    buf.extend_from_slice(prev_hex.as_bytes());
    buf.extend_from_slice(&event_bytes);
    Ok(blake3_hex(&buf))
}

/// Recompute the rolling chain hash: `h0 = blake3(GENESIS_SEED)`,
/// `hn = blake3(h(n-1).hex || canonical_bytes(event_n))`.
pub fn recompute_chain(events: &[OperationEvent]) -> Result<String, serde_json::Error> {
    let mut acc = blake3_hex(GENESIS_SEED.as_bytes());
    for event in events {
        acc = fold_event(&acc, event)?;
    }
    Ok(acc)
}

/// Content address of a receipt: `blake3(canonical_bytes(receipt))`.
pub fn content_address(receipt: &Receipt) -> Result<String, serde_json::Error> {
    Ok(blake3_hex(&canonical_bytes(receipt)?))
}

/// Seal `events` into a receipt, recomputing `seq` from position is the
/// caller's job; this only folds the chain.
pub fn seal(events: Vec<OperationEvent>) -> Result<Receipt, serde_json::Error> {
    let chain_hash = recompute_chain(&events)?;
    Ok(Receipt {
        format_version: FORMAT_VERSION.to_string(),
        events,
        chain_hash,
    })
}

fn is_well_formed_hash(hex: &str) -> bool {
    hex.len() == HASH_HEX_LEN
        && hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
}

fn outcome(stage: &str, passed: bool, detail: String) -> CheckOutcome {
    CheckOutcome {
        stage: stage.to_string(),
        passed,
        detail,
    }
}

/// Stage 1–7 certify pipeline. Pure and deterministic over the receipt.
pub fn verify(receipt: &Receipt) -> Verdict {
    let outcomes = vec![
        stage_decode(receipt),
        stage_check_format(receipt),
        stage_chain_integrity(receipt),
        stage_continuity(receipt),
        stage_verify_commitments(receipt),
        stage_evaluate_profile(receipt),
    ];
    finish(outcomes)
}

/// Stage 7 — `emit_verdict`: accepted iff every prior stage passed.
pub fn finish(outcomes: Vec<CheckOutcome>) -> Verdict {
    let reason = match outcomes.iter().find(|o| !o.passed) {
        Some(o) => format!("{}: {}", o.stage, o.detail),
        None => "all stages passed".to_string(),
    };
    Verdict {
        accepted: outcomes.iter().all(|o| o.passed),
        profile: "CoreV1",
        outcomes,
        reason,
    }
}

fn stage_decode(r: &Receipt) -> CheckOutcome {
    let passed = !r.format_version.trim().is_empty();
    let detail = if passed {
        format!("{} event(s), format_version present", r.events.len())
    } else {
        "format_version is empty or unparseable".to_string()
    };
    outcome("decode", passed, detail)
}

fn stage_check_format(r: &Receipt) -> CheckOutcome {
    let passed = r.format_version == FORMAT_VERSION;
    let detail = if passed {
        format!("format_version == {FORMAT_VERSION}")
    } else {
        format!(
            "expected format_version {FORMAT_VERSION}, found {}",
            r.format_version
        )
    };
    outcome("check_format", passed, detail)
}

fn stage_chain_integrity(r: &Receipt) -> CheckOutcome {
    match recompute_chain(&r.events) {
        Ok(computed) => {
            let passed = computed == r.chain_hash;
            let detail = if passed {
                "recomputed chain hash matches stored chain_hash".to_string()
            } else {
                format!(
                    "chain hash mismatch: stored {}, recomputed {}",
                    r.chain_hash, computed
                )
            };
            outcome("chain_integrity", passed, detail)
        }
        Err(e) => outcome(
            "chain_integrity",
            false,
            format!("could not canonicalize an event: {e}"),
        ),
    }
}

fn stage_continuity(r: &Receipt) -> CheckOutcome {
    let mut seen = std::collections::BTreeSet::new();
    for (index, event) in r.events.iter().enumerate() {
        let expected = index as u64;
        if event.seq != expected {
            return outcome(
                "continuity",
                false,
                format!(
                    "seq gap at position {index}: expected {expected}, found {}",
                    event.seq
                ),
            );
        }
        if !seen.insert(event.id.as_str()) {
            return outcome(
                "continuity",
                false,
                format!("duplicate event id: {}", event.id),
            );
        }
    }
    outcome(
        "continuity",
        true,
        format!(
            "{} event(s) with contiguous seq and unique ids",
            r.events.len()
        ),
    )
}

fn stage_verify_commitments(r: &Receipt) -> CheckOutcome {
    for event in &r.events {
        if !is_well_formed_hash(&event.payload_commitment) {
            return outcome(
                "verify_commitments",
                false,
                format!(
                    "event {} has a malformed commitment (expected {HASH_HEX_LEN} lowercase hex chars)",
                    event.id
                ),
            );
        }
    }
    outcome(
        "verify_commitments",
        true,
        "all commitments are well-formed BLAKE3 digests".to_string(),
    )
}

fn stage_evaluate_profile(r: &Receipt) -> CheckOutcome {
    for event in &r.events {
        if event.event_type.trim().is_empty() {
            return outcome(
                "evaluate_profile",
                false,
                format!("event {} has an empty event_type", event.id),
            );
        }
        if event.payload_commitment.is_empty() {
            return outcome(
                "evaluate_profile",
                false,
                format!("event {} is missing a commitment", event.id),
            );
        }
    }
    outcome(
        "evaluate_profile",
        true,
        format!("profile {FORMAT_VERSION} satisfied"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn event(id: &str, seq: u64, ty: &str, payload: &[u8]) -> OperationEvent {
        OperationEvent {
            id: id.into(),
            seq,
            event_type: ty.into(),
            objects: vec![ObjectRef {
                id: format!("obj-{id}"),
                obj_type: "artifact".into(),
                qualifier: None,
            }],
            payload_commitment: blake3_hex(payload),
        }
    }

    fn two() -> Receipt {
        seal(vec![
            event("e0", 0, "emit", b"payload-zero"),
            event("e1", 1, "emit", b"payload-one"),
        ])
        .expect("seal")
    }

    #[test]
    fn canonical_json_sorts_keys_and_keeps_null_qualifier() {
        // Guards the no-`preserve_order` invariant AND the exact byte layout
        // the root crate hashes.
        let bytes = canonical_bytes(&event("a", 3, "t", b"x")).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert_eq!(
            text,
            format!(
                "{{\"event_type\":\"t\",\"id\":\"a\",\"objects\":[{{\"id\":\"obj-a\",\"obj_type\":\"artifact\",\"qualifier\":null}}],\"payload_commitment\":\"{}\",\"seq\":3}}",
                blake3_hex(b"x")
            )
        );
    }

    #[test]
    fn valid_receipt_accepts() {
        let v = verify(&two());
        assert!(v.accepted, "{}", v.reason);
        assert_eq!(v.reason, "all stages passed");
        assert_eq!(v.outcomes.len(), 6);
    }

    #[test]
    fn genesis_seed_is_derived_from_the_package_version() {
        assert_eq!(
            GENESIS_SEED,
            format!("affidavit-v{}-genesis", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn tampered_commitment_breaks_chain_integrity_only() {
        let mut r = two();
        r.events[1].payload_commitment = blake3_hex(b"tampered");
        let v = verify(&r);
        assert!(!v.accepted);
        assert!(v.reason.starts_with("chain_integrity: chain hash mismatch"));
    }

    #[test]
    fn seq_gap_and_duplicate_id_fail_continuity() {
        let mut r = two();
        r.events[1].seq = 2;
        r.chain_hash = recompute_chain(&r.events).unwrap();
        assert!(verify(&r)
            .reason
            .starts_with("continuity: seq gap at position 1"));

        let mut r = two();
        r.events[1].id = "e0".into();
        r.chain_hash = recompute_chain(&r.events).unwrap();
        assert_eq!(verify(&r).reason, "continuity: duplicate event id: e0");
    }

    #[test]
    fn wrong_format_and_malformed_commitment_are_named() {
        let mut r = two();
        r.format_version = "1.0.0".into();
        assert!(verify(&r).reason.starts_with("check_format:"));

        let mut r = two();
        r.events[0].payload_commitment = "ABC".into();
        r.chain_hash = recompute_chain(&r.events).unwrap();
        assert!(verify(&r).reason.starts_with("verify_commitments:"));
    }

    #[test]
    fn empty_chain_is_the_genesis_hash() {
        let r = seal(vec![]).unwrap();
        assert_eq!(r.chain_hash, blake3_hex(GENESIS_SEED.as_bytes()));
        assert!(verify(&r).accepted);
    }
}
