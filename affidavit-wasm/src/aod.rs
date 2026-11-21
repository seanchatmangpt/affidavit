//! AOD Historical Witness ABI — AFFI-SPEC-26.11.19 Auditor Isolation (Mode B).
//!
//! Exposes [`affidavit_core::verify_witness`] only. This module must **never**
//! call the Live Gate mapper, accept a compiled ontology type, or return
//! Region 3 buffer bytes. The attestation is inert digests + counts.
//!
//! TODO(ggen): once sibling `ggen-marketplace` packs are available in this
//! environment, re-render `abi_meta` / registry from `ontology/affi-wasm.ttl`
//! (`ggen sync run`) so the hand-projected `verify_witness` op entry matches
//! the manufactured surfaces byte-for-byte.

use affidavit_core::{verify_witness, AodRefusal, WitnessAttestation};
use serde_json::{json, Map, Value};

use crate::abi::{err, field, hex_decode, obj, Res};

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn refusal_name(r: AodRefusal) -> &'static str {
    match r {
        AodRefusal::UnalignedInput => "UnalignedInput",
        AodRefusal::TruncatedFile => "TruncatedFile",
        AodRefusal::InvalidSentinel => "InvalidSentinel",
        AodRefusal::NonZeroReserved => "NonZeroReserved",
        AodRefusal::InvalidTrailerOffset => "InvalidTrailerOffset",
        AodRefusal::UnknownOntology => "UnknownOntology",
        AodRefusal::BufferCountMismatch => "BufferCountMismatch",
        AodRefusal::BoundsOrSlack => "BoundsOrSlack",
        AodRefusal::FixedStrideViolation => "FixedStrideViolation",
        AodRefusal::NonZeroPadding => "NonZeroPadding",
        AodRefusal::DigestMismatch => "DigestMismatch",
        AodRefusal::PqcSealFailure => "PqcSealFailure",
        AodRefusal::LeaseSpentOrExpired => "LeaseSpentOrExpired",
        // Non-exhaustive: new codes surface as Unknown until this match is extended.
        _ => "Unknown",
    }
}

fn attestation_body(att: &WitnessAttestation) -> Map<String, Value> {
    // Auditor isolation: owned digests/counts only — never Region 3 pages.
    obj(json!({
        "mode": "historical_witness",
        "accepted": true,
        "ontology_hash": hex_encode(&att.ontology_hash),
        "payload_digest": hex_encode(&att.payload_digest),
        "lease_epoch": att.lease_epoch,
        "lease_nullifier": hex_encode(&att.lease_nullifier),
        "folded_receipt": hex_encode(&att.folded_receipt),
        "row_count": att.row_count,
        "buffer_count": att.buffer_count,
        "region3_mapped": false,
    }))
}

fn refusal_body(r: AodRefusal) -> Map<String, Value> {
    obj(json!({
        "mode": "historical_witness",
        "accepted": false,
        "refusal": refusal_name(r),
        "refusal_code": format!("0x{:04X}", r.machine_code()),
        "region3_mapped": false,
    }))
}

/// `verify_witness`: Historical Witness over a sealed `.aod` (hex).
///
/// Request: `{ "op":"verify_witness", "aod_hex":"<hex>" }`.
/// Response is a total verdict (`ok:true`): `accepted` plus inert attestation
/// fields, or a typed `refusal` / `refusal_code`. Always `region3_mapped:false`.
/// No Live Gate / compiled-ontology request fields are accepted or required.
pub fn op_verify_witness(req: &Value) -> Res<Map<String, Value>> {
    // Refuse Live Gate knobs at the ABI edge — Historical Witness stands alone.
    if req.get("ontology_hex").is_some()
        || req.get("compiled_ontology").is_some()
        || req.get("map_live").is_some()
    {
        return Err(err(
            "bad_field",
            "verify_witness is Historical Witness only: refuse Live Gate request fields",
        ));
    }

    let hex = field(req, "aod_hex")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`aod_hex` must be a hex string"))?;
    let raw = hex_decode(hex).ok_or_else(|| err("bad_field", "`aod_hex` is not valid hex"))?;

    match verify_witness(&raw) {
        Ok(att) => Ok(attestation_body(&att)),
        Err(r) => Ok(refusal_body(r)),
    }
}

#[cfg(test)]
mod isolation_unit {
    use super::*;
    use affidavit_core::{seal_aod, SealParams};

    #[test]
    fn response_never_carries_region3_keys() {
        let ontology = [0x11u8; 32];
        let lease = [0x22u8; 32];
        let tip = [0x33u8; 32];
        let page = [0xABu8; 64];
        let raw = seal_aod(
            &[affidavit_core::AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 1,
                lease_epoch: 1,
                lease_nullifier: &lease,
                folded_receipt: &tip,
            },
        )
        .expect("seal");
        let req = json!({"op":"verify_witness","aod_hex": hex_encode(&raw)});
        let body = op_verify_witness(&req).expect("op");
        assert_eq!(body.get("region3_mapped"), Some(&json!(false)));
        assert_eq!(body.get("accepted"), Some(&json!(true)));
        for forbidden in [
            "buffers",
            "pages",
            "region3",
            "data",
            "view",
            "aod_view",
            "buffer_bytes",
            "page_bytes",
        ] {
            assert!(
                body.get(forbidden).is_none(),
                "Region 3 key `{forbidden}` must not appear: {body:?}"
            );
        }
        // Page content must not leak as any string value.
        let dumped = serde_json::to_string(&Value::Object(body)).unwrap();
        assert!(
            !dumped.contains(&hex_encode(&page)),
            "Region 3 page bytes leaked into response"
        );
    }

    #[test]
    fn live_gate_fields_are_typed_errors() {
        let req = json!({
            "op": "verify_witness",
            "aod_hex": "00",
            "ontology_hex": "00"
        });
        let e = op_verify_witness(&req).expect_err("live knobs refused");
        assert_eq!(e.code, "bad_field");
    }
}
