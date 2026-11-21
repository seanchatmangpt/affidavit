//! Court: `verify_witness` Historical Witness ABI (AFFI-SPEC-26.11.19 DoD).
//!
//! Auditor isolation:
//! 1. Honest sealed `.aod` admits with `region3_mapped: false`.
//! 2. Response never carries Region 3 page bytes or Live Gate knobs.
//! 3. Truncated / tampered envelopes refuse with typed `0xE5xx` codes.
//! 4. `ontology_hex` / `map_live` request fields are typed `bad_field`.
//! 5. Module source for the op does not name `map_live` or `CompiledOntology`.
use affidavit_core::{seal_aod, AodBufferSpec, SealParams};
use affidavit_wasm::abi::call;
use serde_json::{json, Value};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn seal_empty() -> Vec<u8> {
    let ontology = [0x44u8; 32];
    let lease = [0x55u8; 32];
    let tip = [0x66u8; 32];
    seal_aod(
        &[],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 0,
            lease_epoch: 9,
            lease_nullifier: &lease,
            folded_receipt: &tip,
        },
    )
    .expect("seal empty aod")
}

fn seal_with_page(page: &[u8]) -> Vec<u8> {
    let ontology = [0x11u8; 32];
    let lease = [0x22u8; 32];
    let tip = [0x33u8; 32];
    seal_aod(
        &[AodBufferSpec { bytes: page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 1,
            lease_epoch: 1,
            lease_nullifier: &lease,
            folded_receipt: &tip,
        },
    )
    .expect("seal with page")
}

#[test]
fn honest_witness_admits_through_the_abi() {
    let raw = seal_empty();
    let req = serde_json::to_vec(&json!({
        "op": "verify_witness",
        "aod_hex": hex(&raw),
    }))
    .unwrap();
    let res: Value = serde_json::from_slice(&call(&req)).unwrap();
    assert_eq!(res["ok"], true, "{res}");
    assert_eq!(res["op"], "verify_witness");
    assert_eq!(res["mode"], "historical_witness");
    assert_eq!(res["accepted"], true);
    assert_eq!(res["region3_mapped"], false);
    assert_eq!(
        res["ontology_hash"],
        "4444444444444444444444444444444444444444444444444444444444444444"
    );
    assert_eq!(res["lease_epoch"], 9);
    assert_eq!(res["row_count"], 0);
    assert_eq!(res["buffer_count"], 0);
}

#[test]
fn auditor_isolation_no_region3_data_in_response() {
    let page = [0xABu8; 64];
    let raw = seal_with_page(&page);
    let req = serde_json::to_vec(&json!({
        "op": "verify_witness",
        "aod_hex": hex(&raw),
    }))
    .unwrap();
    let out = call(&req);
    let res: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(res["ok"], true, "{res}");
    assert_eq!(res["accepted"], true);
    assert_eq!(res["region3_mapped"], false);
    assert_eq!(res["buffer_count"], 1);
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains(&hex(&page)),
        "Region 3 page bytes must not appear in the JSON response"
    );
    for forbidden in [
        "buffers",
        "pages",
        "region3",
        "aod_view",
        "buffer_bytes",
        "page_bytes",
        "map_live",
    ] {
        assert!(
            res.get(forbidden).is_none(),
            "forbidden key `{forbidden}` present: {res}"
        );
    }
}

#[test]
fn truncated_file_refuses_with_e502() {
    let req = serde_json::to_vec(&json!({
        "op": "verify_witness",
        "aod_hex": "00",
    }))
    .unwrap();
    let res: Value = serde_json::from_slice(&call(&req)).unwrap();
    assert_eq!(res["ok"], true, "verdict is total: {res}");
    assert_eq!(res["accepted"], false);
    assert_eq!(res["refusal_code"], "0xE502");
    assert_eq!(res["refusal"], "TruncatedFile");
    assert_eq!(res["region3_mapped"], false);
}

#[test]
fn nonzero_flags_refuse_with_e504() {
    let mut raw = seal_empty();
    // Flip header flags → NonZeroReserved before digest check.
    raw[4] ^= 0x01;
    let req = serde_json::to_vec(&json!({
        "op": "verify_witness",
        "aod_hex": hex(&raw),
    }))
    .unwrap();
    let res: Value = serde_json::from_slice(&call(&req)).unwrap();
    assert_eq!(res["ok"], true, "{res}");
    assert_eq!(res["accepted"], false);
    assert_eq!(res["refusal_code"], "0xE504");
    assert_eq!(res["region3_mapped"], false);
}

#[test]
fn live_gate_request_fields_are_typed_errors() {
    let raw = seal_empty();
    for knob in ["ontology_hex", "compiled_ontology", "map_live"] {
        let mut body = json!({
            "op": "verify_witness",
            "aod_hex": hex(&raw),
        });
        body[knob] = json!("deadbeef");
        let req = serde_json::to_vec(&body).unwrap();
        let res: Value = serde_json::from_slice(&call(&req)).unwrap();
        assert_eq!(res["ok"], false, "knob `{knob}`: {res}");
        assert_eq!(res["error"]["code"], "bad_field");
    }
}

#[test]
fn aod_module_source_has_no_live_gate_linkage() {
    let src = include_str!("../src/aod.rs");
    // Only `verify_witness` / refusal types from affidavit_core — never Live Gate.
    let use_lines: Vec<&str> = src
        .lines()
        .filter(|l| l.trim_start().starts_with("use "))
        .collect();
    let joined = use_lines.join("\n");
    assert!(
        !joined.contains("map_live") && !joined.contains("CompiledOntology"),
        "aod.rs use-lines must not import Live Gate symbols: {joined}"
    );
    assert!(
        !src.contains("map_live(") && !src.contains("CompiledOntology {"),
        "aod.rs must not call Live Gate APIs"
    );
    assert!(
        src.contains("verify_witness"),
        "aod.rs must call verify_witness"
    );
}
