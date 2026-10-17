//! Host-side PQC ABI courts: real FIPS 204 (ML-DSA-65) keys generated in-test
//! with the external `ml-dsa` crate, exercised through the JSON ABI export
//! path (`abi::call`), plus the compiled wasm module through the real wasmi
//! runtime for the differential check. Real collaborators, no mocks: genuine
//! signatures are accepted; a single flipped bit is refused.
#![cfg(not(target_arch = "wasm32"))]

mod common;

use affidavit_wasm::abi::call;
use serde_json::{json, Value};

use ml_dsa::{Keypair, KeyInit, MlDsa65, Signer, SigningKey};

/// Deterministic 32-byte seed for the in-test key (FIPS 204 ML-DSA-65).
const SEED: [u8; 32] = [7u8; 32];

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Real ML-DSA-65 keypair + signature from the external RustCrypto crate.
fn keypair() -> (Vec<u8>, SigningKey<MlDsa65>) {
    let sk = SigningKey::<MlDsa65>::new(&ml_dsa::Seed::from(SEED));
    let vk = sk.verifying_key().encode();
    (vk.to_vec(), sk)
}

fn run(req: Value) -> (Value, Vec<u8>) {
    let raw = serde_json::to_vec(&req).unwrap();
    let out = call(&raw);
    (serde_json::from_slice(&out).unwrap(), out)
}

/// The compact JSON bytes of the events array — exactly what `pqc_assemble`
/// binds the ML-DSA-65 verdict to.
fn bound_bytes(events: &Value) -> Vec<u8> {
    serde_json::to_vec(events).unwrap()
}

#[test]
fn pqc_verify_accepts_a_genuine_signature_over_the_export_path() {
    let (pk, sk) = keypair();
    let sig = sk.sign(b"envelope bytes").encode();
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "envelope bytes",
        "sig_hex": hex_encode(&sig),
        "pk_hex": hex_encode(&pk),
        "profile": "pqc",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], true);
    assert_eq!(r["algorithm"], "ML-DSA-65");
    assert_eq!(r["profile"], "pqc(ml-dsa-65)");
    assert!(r.get("refusal").is_none());
}

#[test]
fn pqc_verify_refuses_a_single_flipped_bit() {
    let (pk, sk) = keypair();
    let mut sig = sk.sign(b"envelope bytes").encode().to_vec();
    sig[0] ^= 0x01;
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg_hex": hex_encode(b"envelope bytes"),
        "sig_hex": hex_encode(&sig),
        "pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], false);
    assert_eq!(r["refusal"], "ml_dsa65_rejected");
}

#[test]
fn pqc_verify_refuses_cross_key_and_malformed_inputs_without_panicking() {
    let (pk, sk) = keypair();
    let sig = sk.sign(b"envelope bytes").encode();
    // Cross key: signature over a different message.
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "another message",
        "sig_hex": hex_encode(&sig),
        "pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["accepted"], false);
    // Malformed pk/sig lengths collapse to a refusal, never a trap.
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "envelope bytes",
        "sig_hex": "00",
        "pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], true);
    assert_eq!(r["accepted"], false);
    // Malformed hex is a typed ABI error, not a panic.
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "envelope bytes",
        "sig_hex": "zz",
        "pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["code"], "bad_field");
    // Unknown profile value is typed.
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "envelope bytes",
        "sig_hex": hex_encode(&sig),
        "pk_hex": hex_encode(&pk),
        "profile": "lattice-max",
    }));
    assert_eq!(r["error"]["code"], "bad_field");
}

#[test]
fn hybrid_profile_demands_and_checks_the_same_mldsa65_signature() {
    let (pk, sk) = keypair();
    let good = sk.sign(b"envelope bytes").encode();
    let mut bad = good;
    bad[10] ^= 0x80;
    for profile in ["hybrid", "pqc"] {
        let (r, _) = run(json!({
            "op": "pqc_verify",
            "msg": "envelope bytes",
            "sig_hex": hex_encode(&good),
            "pk_hex": hex_encode(&pk),
            "profile": profile,
        }));
        assert_eq!(r["accepted"], true, "{profile}");
        let (r, _) = run(json!({
            "op": "pqc_verify",
            "msg": "envelope bytes",
            "sig_hex": hex_encode(&bad),
            "pk_hex": hex_encode(&pk),
            "profile": profile,
        }));
        assert_eq!(r["accepted"], false, "{profile}");
        assert_eq!(r["refusal"], "ml_dsa65_rejected");
    }
}

#[test]
fn classical_profile_skips_the_pq_check() {
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "envelope bytes",
        "sig_hex": "00",
        "pk_hex": "00",
        "profile": "classical",
    }));
    assert_eq!(r["ok"], true);
    assert_eq!(r["accepted"], true);
}

#[test]
fn pqc_assemble_seals_and_binds_the_mldsa65_verdict() {
    let (pk, sk) = keypair();
    let events = json!([{"event_type": "build", "payload": "pqc-sealed"}]);
    let sig = sk.sign(&bound_bytes(&events)).encode();
    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": events,
        "ml_dsa_sig_hex": hex_encode(&sig),
        "ml_dsa_pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["pqc"]["accepted"], true);
    assert_eq!(r["pqc"]["algorithm"], "ML-DSA-65");
    assert_eq!(r["pqc"]["profile"], "pqc(ml-dsa-65)");
    // The seal itself went through the same path as `assemble`.
    let (v, _) = run(json!({"op": "verify", "receipt": r["receipt"].clone()}));
    assert_eq!(v["accepted"], true, "{v}");
}

#[test]
fn pqc_assemble_refuses_the_seal_when_a_bit_flips() {
    let (pk, sk) = keypair();
    let events = json!([{"event_type": "build", "payload": "pqc-sealed"}]);
    let mut sig = sk.sign(&bound_bytes(&events)).encode().to_vec();
    sig[100] ^= 0x02;
    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": events,
        "ml_dsa_sig_hex": hex_encode(&sig),
        "ml_dsa_pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["code"], "pqc_refused");
    assert_eq!(r["error"]["refusal"], "ml_dsa65_rejected");
    assert_eq!(r["error"]["profile"], "pqc(ml-dsa-65)");
}

#[test]
fn pqc_assemble_refuses_when_the_events_do_not_match_the_signature() {
    // Signature over one events array, request carries a different one.
    let (pk, sk) = keypair();
    let signed = json!([{"event_type": "build", "payload": "pqc-sealed"}]);
    let sig = sk.sign(&bound_bytes(&signed)).encode();
    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": json!([{"event_type": "build", "payload": "tampered"}]),
        "ml_dsa_sig_hex": hex_encode(&sig),
        "ml_dsa_pk_hex": hex_encode(&pk),
    }));
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["code"], "pqc_refused");
}

#[test]
fn pqc_assemble_classical_profile_seals_without_a_pq_check() {
    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": [{"event_type": "build", "payload": "plain"}],
        "ml_dsa_sig_hex": "00",
        "ml_dsa_pk_hex": "00",
        "pqc_profile": "classical",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["pqc"]["profile"], "classical");
    assert_eq!(r["pqc"]["accepted"], true);
}

#[test]
fn pqc_ops_are_advertised_and_differentially_identical_in_wasm() {
    // capabilities advertises the new ops and error code.
    let (caps, _) = run(json!({"op": "capabilities"}));
    let ops = caps["ops"].as_array().unwrap();
    assert!(ops.contains(&json!("pqc_verify")));
    assert!(ops.contains(&json!("pqc_assemble")));
    assert!(caps["error_codes"]
        .as_array()
        .unwrap()
        .contains(&json!("pqc_refused")));

    // The exact registry example requests are byte-identical natively and in
    // the real compiled wasm module (wasmi), and succeed in both.
    let doc: Value =
        serde_json::from_str(include_str!("../registry/op-examples.json")).unwrap();
    let mut host = common::Host::new();
    for ex in doc["examples"].as_array().unwrap() {
        let op = ex["op"].as_str().unwrap();
        if !op.starts_with("pqc_") {
            continue;
        }
        let raw = serde_json::to_vec(&ex["request"]).unwrap();
        let in_wasm = host.call_raw(&raw);
        assert_eq!(in_wasm, call(&raw), "native vs wasm diverged on `{op}`");
        let v: Value = serde_json::from_slice(&in_wasm).unwrap();
        assert_eq!(v["ok"], true, "wasm example for `{op}` must work: {v}");
    }
}

// ---------------------------------------------------------------------------
// SLH-DSA-SHA2-128s (`pqc(slh-dsa-sha2-128s)`) and Dual (`pqc(dual)`) courts:
// the ABI-level coverage the audit found missing. Without these, a wiring
// swap of the (&slh_sig, &slh_pk) tuple in `op_pqc_verify`/`op_pqc_assemble`
// passes the whole suite — the SLH half was never exercised through `call`.
// ---------------------------------------------------------------------------

use slh_dsa::signature::{SignatureEncoding, Signer as SlhSigner, Verifier as SlhVerifier};
use slh_dsa::{Sha2_128s, Signature as SlhSignature, SigningKey as SlhSigningKey, VerifyingKey as SlhVerifyingKey};

/// Deterministic 16-byte seeds for the in-test SLH-DSA-SHA2-128s key
/// (FIPS 205 `slh_keygen_internal`; n = 16 for the 128s parameter set).
const SLH_SK_SEED: [u8; 16] = [11u8; 16];
const SLH_SK_PRF: [u8; 16] = [22u8; 16];
const SLH_PK_SEED: [u8; 16] = [33u8; 16];

/// Real SLH-DSA-SHA2-128s keypair + signature from the external RustCrypto
/// crate, via the deterministic KAT keygen path. `salt` varies the seeds so
/// distinct calls mint distinct keys (the positive control needs two).
fn slh_keypair() -> (Vec<u8>, SlhSigningKey<Sha2_128s>) {
    slh_keypair_salt(0)
}

fn slh_keypair_salt(salt: u8) -> (Vec<u8>, SlhSigningKey<Sha2_128s>) {
    let sk = SlhSigningKey::<Sha2_128s>::slh_keygen_internal(
        &SLH_SK_SEED.map(|b| b.wrapping_add(salt)),
        &SLH_SK_PRF.map(|b| b.wrapping_add(salt)),
        &SLH_PK_SEED.map(|b| b.wrapping_add(salt)),
    );
    let vk = sk.verifying_key();
    (vk.to_vec(), sk)
}

#[test]
fn pqc_verify_slh_dsa_128s_profile_accepts_a_genuine_signature() {
    let (pk, sk) = slh_keypair();
    let sig = sk.sign(b"slh envelope").to_vec();
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "slh envelope",
        "sig_hex": "00",
        "pk_hex": "00",
        "slh_dsa_sig_hex": hex_encode(&sig),
        "slh_dsa_pk_hex": hex_encode(&pk),
        "profile": "pqc(slh-dsa-sha2-128s)",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], true, "{r}");
    assert_eq!(r["algorithm"], "SLH-DSA-SHA2-128s");
    assert_eq!(r["profile"], "pqc(slh-dsa-sha2-128s)");
    assert!(r.get("refusal").is_none());
}

#[test]
fn pqc_verify_slh_dsa_128s_profile_refuses_a_single_flipped_bit() {
    let (pk, sk) = slh_keypair();
    let mut sig = sk.sign(b"slh envelope").to_vec();
    sig[0] ^= 0x01;
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "slh envelope",
        "sig_hex": "00",
        "pk_hex": "00",
        "slh_dsa_sig_hex": hex_encode(&sig),
        "slh_dsa_pk_hex": hex_encode(&pk),
        "profile": "pqc(slh-dsa-sha2-128s)",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], false);
    assert_eq!(r["refusal"], "slh_dsa_rejected");
}

#[test]
fn pqc_verify_dual_profile_accepts_only_when_both_halves_are_genuine() {
    let (pk, sk) = keypair();
    let (slh_pk, slh_sk) = slh_keypair();
    let ml_sig = sk.sign(b"dual envelope").encode();
    let slh_sig = slh_sk.sign(b"dual envelope").to_vec();

    // Both genuine: accept.
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "dual envelope",
        "sig_hex": hex_encode(&ml_sig),
        "pk_hex": hex_encode(&pk),
        "slh_dsa_sig_hex": hex_encode(&slh_sig),
        "slh_dsa_pk_hex": hex_encode(&slh_pk),
        "profile": "pqc(dual)",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], true, "{r}");
    assert_eq!(r["algorithm"], "ML-DSA-65 + SLH-DSA-SHA2-128s");
    assert_eq!(r["profile"], "pqc(dual)");
    assert!(r.get("refusal").is_none());

    // Tampered SLH half: dual refuses with the SLH refusal code (ML-DSA-65
    // passes first, so the SLH verifier is what says no — this is exactly the
    // tuple-swap sensitivity the audit flagged).
    let mut bad_slh = slh_sig.clone();
    bad_slh[200] ^= 0x04;
    let (r, _) = run(json!({
        "op": "pqc_verify",
        "msg": "dual envelope",
        "sig_hex": hex_encode(&ml_sig),
        "pk_hex": hex_encode(&pk),
        "slh_dsa_sig_hex": hex_encode(&bad_slh),
        "slh_dsa_pk_hex": hex_encode(&slh_pk),
        "profile": "pqc(dual)",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["accepted"], false);
    assert_eq!(r["refusal"], "slh_dsa_rejected");
}

#[test]
fn pqc_assemble_dual_profile_binds_both_verdicts_and_refuses_on_a_slh_bit_flip() {
    let (pk, sk) = keypair();
    let (slh_pk, slh_sk) = slh_keypair();
    let events = json!([{"event_type": "build", "payload": "dual-sealed"}]);
    let bound = bound_bytes(&events);
    let ml_sig = sk.sign(&bound).encode();
    let slh_sig = slh_sk.sign(&bound).to_vec();

    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": events,
        "ml_dsa_sig_hex": hex_encode(&ml_sig),
        "ml_dsa_pk_hex": hex_encode(&pk),
        "slh_dsa_sig_hex": hex_encode(&slh_sig),
        "slh_dsa_pk_hex": hex_encode(&slh_pk),
        "pqc_profile": "pqc(dual)",
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["pqc"]["accepted"], true, "{r}");
    assert_eq!(r["pqc"]["profile"], "pqc(dual)");
    assert_eq!(r["pqc"]["algorithm"], "ML-DSA-65 + SLH-DSA-SHA2-128s");
    let (v, _) = run(json!({"op": "verify", "receipt": r["receipt"].clone()}));
    assert_eq!(v["accepted"], true, "{v}");

    // A flipped bit in the SLH half refuses the whole seal.
    let mut bad_slh = slh_sig.clone();
    bad_slh[300] ^= 0x08;
    let (r, _) = run(json!({
        "op": "pqc_assemble",
        "events": json!([{"event_type": "build", "payload": "dual-sealed"}]),
        "ml_dsa_sig_hex": hex_encode(&ml_sig),
        "ml_dsa_pk_hex": hex_encode(&pk),
        "slh_dsa_sig_hex": hex_encode(&bad_slh),
        "slh_dsa_pk_hex": hex_encode(&slh_pk),
        "pqc_profile": "pqc(dual)",
    }));
    assert_eq!(r["ok"], false, "{r}");
    assert_eq!(r["error"]["code"], "pqc_refused");
    assert_eq!(r["error"]["refusal"], "slh_dsa_rejected");
    assert_eq!(r["error"]["profile"], "pqc(dual)");
}

// Positive control: the in-test SLH keygen/sign/verify loop must be able to
// fail — a genuine key rejects a foreign key's signature. Guards against the
// court passing vacuously because the adapter accepts everything.
#[test]
fn slh_positive_control_the_test_keypair_can_disagree() {
    let (pk, _sk) = slh_keypair();
    let (_other_pk, other_sk) = slh_keypair_salt(1);
    let sig = other_sk.sign(b"slh envelope").to_vec();
    let vk = SlhVerifyingKey::<Sha2_128s>::try_from(pk.as_slice()).unwrap();
    let sigma = SlhSignature::<Sha2_128s>::try_from(sig.as_slice()).unwrap();
    assert!(vk.verify(b"slh envelope", &sigma).is_err());
}
