//! Host-side court for zk-range domain-label separation: real bulletproofs
//! range proofs generated host-side with the external crate, exercised
//! through the JSON ABI export path (`abi::call`). Real collaborators, no
//! mocks: an honestly generated proof under a registered label is admitted;
//! the same proof under a different registered label is refused (domain
//! separation); a bit-flipped proof is refused under its own label.
#![cfg(not(target_arch = "wasm32"))]

use affidavit_wasm::abi::call;
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use serde_json::{json, Value};

const LABEL_A: &str = "affidavit:zk-range:v1:effect-budget";
const LABEL_B: &str = "affidavit:zk-range:v1:receipt-amount";

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

fn run(req: Value) -> Value {
    let raw = serde_json::to_vec(&req).unwrap();
    let out = call(&raw);
    serde_json::from_slice(&out).unwrap()
}

/// Real bulletproofs range proof over `value` with `bits` width, proven
/// under `label` — the identical mathematics the edge module's verifier
/// replays inside `range_proof_verify`.
fn prove(value: u64, bits: usize, label: &'static str) -> (String, String) {
    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let pc_gens = PedersenGens::default();
    // Deterministic blinding scalar from BLAKE3 (blake3 is a direct dep; no
    // RNG crate is reachable from this test target without new Cargo deps).
    let mut b = [0u8; 32];
    b.copy_from_slice(blake3::hash(label.as_bytes()).as_bytes());
    let blinding = Scalar::from_bytes_mod_order(b);
    let mut transcript = Transcript::new(label.as_bytes());
    let (proof, commitment) =
        RangeProof::prove_single(&gens, &pc_gens, &mut transcript, value, &blinding, bits)
            .expect("host-side prove");
    (
        hex_encode(&proof.to_bytes()),
        hex_encode(commitment.as_bytes()),
    )
}

#[test]
fn positive_control_new_label_admits_an_honest_proof() {
    let (proof_hex, commitment_hex) = prove(4242, 32, LABEL_B);
    let r = run(json!({
        "op": "range_proof_verify",
        "commitment_hex": commitment_hex,
        "proof_hex": proof_hex,
        "bits": 32,
        "label": LABEL_B,
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["admitted"], true, "{r}");
}

#[test]
fn proof_under_label_a_is_refused_under_label_b() {
    let (proof_hex, commitment_hex) = prove(4242, 32, LABEL_A);
    let r = run(json!({
        "op": "range_proof_verify",
        "commitment_hex": commitment_hex,
        "proof_hex": proof_hex,
        "bits": 32,
        "label": LABEL_B,
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["admitted"], false, "{r}");
}

#[test]
fn bit_flipped_proof_is_refused_under_its_own_label() {
    let (proof_hex, commitment_hex) = prove(4242, 32, LABEL_A);
    let mut raw = hex_decode(&proof_hex);
    raw[0] ^= 0x01;
    let r = run(json!({
        "op": "range_proof_verify",
        "commitment_hex": commitment_hex,
        "proof_hex": hex_encode(&raw),
        "bits": 32,
        "label": LABEL_A,
    }));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["admitted"], false, "{r}");
}
