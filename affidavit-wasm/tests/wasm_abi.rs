//! Runs the *compiled* WebAssembly module in a real wasm runtime (wasmi) and
//! drives it through the JSON ABI exactly as a host such as Wasmex would. Set
//! `AFFIDAVIT_WASM` to test a prebuilt module; otherwise the module is built
//! once into `target/wasm-abi`.
//!
//! The fixtures under `tests/fixtures/` were produced by the real `affi` binary
//! (`affi receipt emit` x3 -> `assemble` -> `verify`), so passing here means the
//! wasm module agrees with `affi`, not merely with itself.
#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::Host;
use serde_json::{json, Value};

const GOLDEN: &str = include_str!("fixtures/golden_receipt.json");
const TAMPERED: &str = include_str!("fixtures/tampered_receipt.json");

fn golden() -> Value {
    serde_json::from_str(GOLDEN).unwrap()
}

/// KAT vector env-000-ES256 (rendered plane, fixtures/crypto_trust_kat_vectors.json
/// surfaces.envelope[0]): canonical document + expected signing pre-image.
const KAT_CANONICAL: &str = r#"{"algorithm":"ES256","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_a9c3bd446195e2e2","nonce":[25,107,217,46,141,94,95,54,120,72,204,59,49,154,244,42],"not_before":1700000000,"policy_epoch":1,"profile":"CLASSICAL","revocation_epoch":0,"subject_digest":[206,63,11,26,126,80,218,245,190,191,110,203,240,54,52,94,36,141,20,88,5,127,63,121,250,128,109,66,210,110,68,97],"version":"CTP-ENVELOPE-v1"}"#;
const KAT_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";

#[test]
fn exports_the_abi_and_reports_its_release_identity() {
    let mut h = Host::new();
    assert_eq!(h.abi_version.call(&mut h.store, ()).unwrap(), 1);
    let caps = h.call(json!({"op": "capabilities"}));
    assert_eq!(caps["ok"], true);
    assert_eq!(caps["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        caps["genesis_seed"],
        format!("affidavit-v{}-genesis", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn accepts_a_real_affi_receipt() {
    let mut h = Host::new();
    let v = h.call(json!({"op": "verify", "receipt": golden()}));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["accepted"], true, "{v}");
    assert_eq!(v["profile"], "CoreV1");
    assert_eq!(v["reason"], "all stages passed");
    let stages: Vec<&str> = v["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["stage"].as_str().unwrap())
        .collect();
    assert_eq!(
        stages,
        [
            "decode",
            "check_format",
            "chain_integrity",
            "continuity",
            "verify_commitments",
            "evaluate_profile"
        ]
    );
}

#[test]
fn rejects_a_tampered_receipt_with_the_exact_reason_affi_gives() {
    // Copied from `affi receipt verify tampered_receipt.json` on the same
    // fixture: same stored and recomputed hashes, same wording.
    let mut h = Host::new();
    let receipt: Value = serde_json::from_str(TAMPERED).unwrap();
    let v = h.call(json!({"op": "verify", "receipt": receipt}));
    assert_eq!(v["accepted"], false);
    assert_eq!(
        v["reason"],
        "chain_integrity: chain hash mismatch: stored d6cd5e0c07671707d34d5e65c42964a82f06776b93d0e33457b5a6dcf7765be0, recomputed 1f1d040c94289281daa2c13b18d4cda9b1a28c59ba323285febc20971f461d7a"
    );
}

#[test]
fn assemble_reproduces_affi_byte_for_byte() {
    // The golden receipt came from three `affi receipt emit` calls whose payload
    // files held these exact bytes (with trailing newline).
    let mut h = Host::new();
    let r = h.call(json!({"op": "assemble", "events": [
        {"event_type": "build",  "objects": ["repo:git:main"],             "payload": "compile step\n"},
        {"event_type": "test",   "objects": ["suite:test-suite:unit"],     "payload": "unit tests\n"},
        {"event_type": "deploy", "objects": ["svc:service"],               "payload": "ship it\n"},
    ]}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(
        r["receipt"],
        golden(),
        "wasm assemble diverged from real affi output"
    );
    assert_eq!(r["chain_hash"], golden()["chain_hash"]);
}

#[test]
fn native_and_wasm_answer_byte_identically() {
    // Same request, two targets (native rlib vs compiled wasm32): responses must
    // be identical bytes. Catches float, hasher or ordering drift across targets.
    let mut h = Host::new();
    let requests = [
        json!({"op": "capabilities"}),
        json!({"op": "verify", "receipt": golden()}),
        json!({"op": "commit", "payload": "héllo wörld ✓"}),
        json!({"op": "mine", "receipts": [golden(), golden()]}),
        json!({"op": "conform", "model": [golden()], "trace": golden()}),
        json!({
            "op": "verify_signature_input",
            "envelope_json": KAT_CANONICAL,
            "expected_signing_input_hex": KAT_SIGNING_INPUT_HEX,
        }),
        json!({"op": "nope"}),
    ];
    for req in requests {
        let raw = serde_json::to_vec(&req).unwrap();
        assert_eq!(
            h.call_raw(&raw),
            affidavit_wasm::abi::call(&raw),
            "target divergence on {req}"
        );
    }
}

#[test]
fn verifies_a_kat_envelope_inside_the_sandbox() {
    // The compiled module recomputes the envelope's signing pre-image and
    // matches it against the rendered plane's known-answer vector — the
    // signature-binding surface witnessed on the deployed subject, not just
    // natively.
    let mut h = Host::new();
    let r = h.call(json!({
        "op": "verify_signature_input",
        "envelope_json": KAT_CANONICAL,
        "expected_signing_input_hex": KAT_SIGNING_INPUT_HEX,
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], true, "{r}");
    assert_eq!(r["signing_input_hex"], KAT_SIGNING_INPUT_HEX);

    // One flipped bit in the expectation: a mismatch, not an error.
    let mut expected = KAT_SIGNING_INPUT_HEX.to_string();
    let last = expected.len() - 1;
    let flip = if expected.ends_with('d') { "e" } else { "d" };
    expected.replace_range(last.., flip);
    let r = h.call(json!({
        "op": "verify_signature_input",
        "envelope_json": KAT_CANONICAL,
        "expected_signing_input_hex": expected,
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], false, "{r}");
}

#[test]
fn mines_and_conforms_over_a_real_receipt() {
    let mut h = Host::new();
    let m = h.call(json!({"op": "mine", "receipts": [golden()]}));
    assert_eq!(m["activities"], json!(["build", "deploy", "test"]));
    assert_eq!(m["all_accepted"], true);
    let c = h.call(json!({"op": "conform", "model": [golden()], "trace": golden()}));
    assert_eq!(c["verdict"], "conformant");
    assert_eq!(c["fitness"], 1.0);
}

#[test]
fn errors_cross_the_boundary_as_structured_json() {
    let mut h = Host::new();
    for (raw, code) in [
        (&b"garbage"[..], "bad_json"),
        (b"{}", "missing_field"),
        (br#"{"op":"nope"}"#, "unknown_op"),
        (b"", "bad_json"),
    ] {
        let r: Value = serde_json::from_slice(&h.call_raw(raw)).unwrap();
        assert_eq!(r["ok"], false);
        assert_eq!(r["error"]["code"], code);
    }
    // The module is still healthy after a run of failures.
    assert_eq!(h.call(json!({"op": "capabilities"}))["ok"], true);
}

#[test]
fn buffers_are_reclaimed_so_memory_does_not_grow_per_call() {
    let mut h = Host::new();
    let req = json!({"op": "verify", "receipt": golden()});
    for _ in 0..50 {
        h.call(req.clone()); // warm up allocator arenas
    }
    let warm = h.memory_pages();
    for _ in 0..2000 {
        assert_eq!(h.call(req.clone())["accepted"], true);
    }
    assert_eq!(
        h.memory_pages(),
        warm,
        "linear memory grew across 2000 identical calls: a buffer is leaking"
    );
}

#[test]
fn large_receipts_verify_inside_the_sandbox() {
    let mut h = Host::new();
    let events: Vec<Value> = (0..2000)
        .map(|i| {
            let ty = ["build", "test", "deploy"][i % 3];
            json!({"event_type": ty, "payload": format!("p{i}")})
        })
        .collect();
    let sealed = h.call(json!({"op": "assemble", "events": events}));
    assert_eq!(sealed["ok"], true);
    let v = h.call(json!({"op": "verify", "receipt": sealed["receipt"]}));
    assert_eq!(v["accepted"], true, "{}", v["reason"]);
}
