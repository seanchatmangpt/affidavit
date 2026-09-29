//! The signature-envelope binding surface, exercised through the JSON ABI
//! exactly as a host would drive it (native rlib; the wasm32 parity of the
//! shared ABI is proven by `wasm_abi.rs`'s `native_and_wasm_answer_byte_identically`).
//!
//! The vectors are the rendered plane's known-answer envelope surface
//! (`fixtures/crypto_trust_kat_vectors.json`, `surfaces.envelope`), hardcoded
//! here so the binding is held to the real generator's output, not to itself:
//! if either the core law or this ABI drifts from what `affi` signs, these
//! pins break loudly.

use affidavit_wasm::abi::call;
use serde_json::{json, Value};

/// KAT vector env-000-ES256: canonical envelope document bytes.
const V0_CANONICAL: &str = r#"{"algorithm":"ES256","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_a9c3bd446195e2e2","nonce":[25,107,217,46,141,94,95,54,120,72,204,59,49,154,244,42],"not_before":1700000000,"policy_epoch":1,"profile":"CLASSICAL","revocation_epoch":0,"subject_digest":[206,63,11,26,126,80,218,245,190,191,110,203,240,54,52,94,36,141,20,88,5,127,63,121,250,128,109,66,210,110,68,97],"version":"CTP-ENVELOPE-v1"}"#;
/// KAT vector env-000-ES256: expected signing pre-image.
const V0_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";
/// KAT vector env-001-ML-DSA-65: canonical envelope document bytes.
const V1_CANONICAL: &str = r#"{"algorithm":"ML_DSA65","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_932a436a743d67cd","nonce":[49,148,240,225,154,244,132,33,20,7,183,98,118,211,47,50],"not_before":1700000000,"policy_epoch":1,"profile":"PQC","revocation_epoch":0,"subject_digest":[18,60,249,28,128,193,211,38,120,198,222,80,164,85,43,52,17,76,4,173,228,175,245,220,104,114,253,206,55,248,114,68],"version":"CTP-ENVELOPE-v1"}"#;
/// KAT vector env-001-ML-DSA-65: expected signing pre-image.
const V1_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";
/// KAT vector env-002-ES256+ML-DSA-65: canonical envelope document bytes.
const V2_CANONICAL: &str = r#"{"algorithm":"HYBRID_ES256_ML_DSA65","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_51deddb3d53698f6","nonce":[231,162,64,114,47,222,37,99,40,110,128,89,143,102,158,46],"not_before":1700000000,"policy_epoch":1,"profile":"HYBRID","revocation_epoch":0,"subject_digest":[106,178,119,6,136,110,176,195,153,50,156,0,35,174,153,58,254,185,106,67,71,36,166,195,68,213,19,13,220,96,173,91],"version":"CTP-ENVELOPE-v1"}"#;
/// KAT vector env-002-ES256+ML-DSA-65: expected signing pre-image.
const V2_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";

/// One JSON request through the ABI, parsed.
fn run(request: Value) -> Value {
    let raw = serde_json::to_vec(&request).unwrap();
    serde_json::from_slice(&call(&raw)).expect("response is JSON")
}

fn check_request(canonical: &str, signing_input_hex: &str) -> Value {
    run(json!({
        "op": "verify_signature_input",
        "envelope_json": canonical,
        "expected_signing_input_hex": signing_input_hex,
    }))
}

#[test]
fn every_kat_vector_verifies_against_its_rendered_pre_image() {
    for (doc, expected, id) in [
        (V0_CANONICAL, V0_SIGNING_INPUT_HEX, "env-000-ES256"),
        (V1_CANONICAL, V1_SIGNING_INPUT_HEX, "env-001-ML-DSA-65"),
        (
            V2_CANONICAL,
            V2_SIGNING_INPUT_HEX,
            "env-002-ES256+ML-DSA-65",
        ),
    ] {
        let r = check_request(doc, expected);
        assert_eq!(r["ok"], true, "{id}: {r}");
        assert_eq!(r["op"], "verify_signature_input");
        assert_eq!(r["verified"], true, "{id}: {r}");
        // The recomputed pre-image is echoed so a host can see which bytes
        // must be signed even when it disagrees.
        assert_eq!(r["signing_input_hex"], expected, "{id}");
    }
}

#[test]
fn a_tampered_field_changes_the_binding_and_is_reported_as_a_mismatch() {
    // Mutate one character inside `audience`: the document still decodes and
    // canonicalizes, so this is verified:false, not an error — the check is
    // proven non-vacuous by the mutation landing exactly one byte away.
    let tampered = V0_CANONICAL.replace("\"affidavit.kat\"", "\"affidavit.kau\"");
    assert_ne!(tampered, V0_CANONICAL);
    let r = check_request(&tampered, V0_SIGNING_INPUT_HEX);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], false, "{r}");
    assert_ne!(r["signing_input_hex"], V0_SIGNING_INPUT_HEX);
    // And against ITS OWN honest pre-image the tampered document verifies:
    // the check binds the document, not the fixture.
    let own = r["signing_input_hex"].as_str().unwrap();
    assert_eq!(check_request(&tampered, own)["verified"], true);
}

#[test]
fn a_tampered_expectation_is_a_mismatch_not_an_error() {
    let mut expected = V1_SIGNING_INPUT_HEX.to_string();
    let last = expected.len() - 1;
    let flip = if expected.ends_with('d') { "e" } else { "d" };
    expected.replace_range(last.., flip);
    let r = check_request(V1_CANONICAL, &expected);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], false, "{r}");
    assert_eq!(r["signing_input_hex"], V1_SIGNING_INPUT_HEX);
}

#[test]
fn unknown_fields_never_enter_the_signed_bytes() {
    let mut doc = String::from("{");
    doc.push_str(r#""zz_unknown":{"nested":[1,2,{"deep":true}]},"#);
    doc.push_str(&V2_CANONICAL[1..]);
    let r = check_request(&doc, V2_SIGNING_INPUT_HEX);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], true, "{r}");
}

#[test]
fn escape_decoded_strings_admit_the_same_signed_bytes() {
    // serde-parity: a transport escape decodes before canonicalization, so
    // the escaped document binds exactly the rendered plane's pre-image.
    let doc = V0_CANONICAL.replacen(
        "\"audience\":\"affidavit.kat\"",
        "\"audience\":\"\\u0061ffidavit.kat\"",
        1,
    );
    assert_ne!(doc, V0_CANONICAL);
    let r = check_request(&doc, V0_SIGNING_INPUT_HEX);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], true, "{r}");
}

#[test]
fn refusals_cross_the_boundary_as_structured_typed_errors() {
    let code = |r: Value| -> String {
        assert_eq!(r["ok"], false, "{r}");
        r["error"]["code"].as_str().unwrap().to_string()
    };

    // Wrong envelope version: the rendered plane's typed WrongVersion.
    let doc = V0_CANONICAL.replace("CTP-ENVELOPE-v1", "CTP-ENVELOPE-v0");
    let r = check_request(&doc, V0_SIGNING_INPUT_HEX);
    assert_eq!(code(r), "wrong_version");

    // Not an envelope at all.
    let r = check_request("not json", V0_SIGNING_INPUT_HEX);
    assert_eq!(code(r), "malformed");
    let r = check_request("{}", V0_SIGNING_INPUT_HEX);
    assert_eq!(code(r), "malformed");

    // A nonce of the wrong length.
    let doc = V0_CANONICAL.replacen(",154,244,42]", ",154,42]", 1);
    let r = check_request(&doc, V0_SIGNING_INPUT_HEX);
    assert_eq!(code(r), "malformed");

    // An integer beyond 2^53 is not I-JSON and is refused, never coerced.
    let doc = V0_CANONICAL.replacen("\"policy_epoch\":1", "\"policy_epoch\":9007199254740993", 1);
    let r = check_request(&doc, V0_SIGNING_INPUT_HEX);
    assert_eq!(code(r), "non_canonical_number");

    // Bad expectation hex.
    let r = check_request(V0_CANONICAL, "zz");
    assert_eq!(code(r), "bad_hex");
    let r = check_request(V0_CANONICAL, "abc");
    assert_eq!(code(r), "bad_hex");

    // Request-shape refusals from the ABI proper.
    let r = run(json!({"op": "verify_signature_input"}));
    assert_eq!(code(r), "missing_field");
    let r = run(json!({"op": "verify_signature_input", "envelope_json": "{}"}));
    assert_eq!(code(r), "missing_field");
    let r = run(json!({
        "op": "verify_signature_input",
        "envelope_json": V0_CANONICAL,
        "expected_signing_input_hex": V0_SIGNING_INPUT_HEX,
        "extra": true,
    }));
    assert_eq!(r["ok"], true, "unknown request fields are ignored: {r}");
    let r = run(json!({
        "op": "verify_signature_input",
        "envelope_json": 3,
        "expected_signing_input_hex": V0_SIGNING_INPUT_HEX,
    }));
    assert_eq!(code(r), "bad_field");
    let r = run(json!({
        "op": "verify_signature_input",
        "envelope_json": V0_CANONICAL,
        "expected_signing_input_hex": 3,
    }));
    assert_eq!(code(r), "bad_field");
}

#[test]
fn the_abi_stays_total_over_hostile_inputs() {
    for junk in [
        &b""[..],
        b"\xff\xfe",
        b"{\"op\":\"verify_signature_input\"}",
        b"{\"op\":\"verify_signature_input\",\"envelope_json\":\"\\\"\",\"expected_signing_input_hex\":\"00\"}",
    ] {
        let r: Value = serde_json::from_slice(&call(junk)).expect("structured response");
        assert_eq!(r["ok"], false, "{junk:?}: {r}");
    }
    // And the module is still healthy afterwards.
    assert_eq!(run(json!({"op": "capabilities"}))["ok"], true);
    // The op is advertised in capabilities.
    let caps = run(json!({"op": "capabilities"}));
    assert!(caps["ops"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o == "verify_signature_input"));
}

#[test]
fn mismatch_details_are_honest_about_lengths() {
    // A truncated expectation (valid hex, wrong length) is a clean mismatch:
    // the response carries the full recomputed pre-image either way.
    let short = &V0_SIGNING_INPUT_HEX[..64];
    let r = check_request(V0_CANONICAL, short);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["verified"], false, "{r}");
    assert_eq!(r["signing_input_hex"], V0_SIGNING_INPUT_HEX);
    assert_eq!(r["canonical_bytes_len"], V0_CANONICAL.len());
}
