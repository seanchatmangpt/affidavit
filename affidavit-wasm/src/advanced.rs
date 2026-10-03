//! Advanced witness verification ops — verifier-side only.
//!
//! These ops check witnesses computed by a prover elsewhere. The edge module
//! never originates proofs and never holds tree state: it consumes a root,
//! a claim, and a witness, and returns a decidable verdict. (Certify, Don't
//! Decide — the module cannot vouch for the prover's honesty, only for the
//! mathematics of the check.)

use monotree::hasher::Blake3 as MonotreeBlake3;
use serde_json::{json, Map, Value};

use crate::abi::{err, field, hex_decode, obj, Res};

/// 32-byte digest width for roots, keys, values, and commitments.
const HASH_LEN: usize = 32;

fn fixed_hex(req: &Value, name: &str) -> Res<[u8; HASH_LEN]> {
    let s = field(req, name)?
        .as_str()
        .ok_or_else(|| err("bad_field", format!("`{name}` must be a hex string")))?;
    let bytes = hex_decode(s)
        .ok_or_else(|| err("bad_field", format!("`{name}` is not valid hex")))?;
    bytes
        .try_into()
        .map_err(|_| err("bad_field", format!("`{name}` must be {HASH_LEN} bytes")))
}

/// `jcs_canonicalize`: RFC 8785 canonical bytes + BLAKE3 commitment for an
/// arbitrary JSON document. The canonical form is the byte-exact preimage
/// any verifier must reproduce before hashing or signing.
pub fn op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> {
    let raw = field(req, "json")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`json` must be a string containing JSON"))?;
    let parsed: Value =
        serde_json::from_str(raw).map_err(|e| err("bad_json", e.to_string()))?;
    let canonical =
        serde_jcs::to_string(&parsed).map_err(|e| err("bad_field", e.to_string()))?;
    let commitment = blake3::hash(canonical.as_bytes()).to_hex().to_string();
    Ok(obj(
        json!({"canonical_json": canonical, "commitment": commitment}),
    ))
}

/// `smt_absence_verify`: zero-trust revocation check. Verifies that the
/// queried key routes into the neighbor's empty sibling slot at the live
/// root — the witness carries the divergent neighbor's inclusion proof and
/// the claimed prefix depth; no tree state is needed at the edge.
pub fn op_smt_absence_verify(req: &Value) -> Res<Map<String, Value>> {
    let root = fixed_hex(req, "root_hex")?;
    let queried = fixed_hex(req, "queried_key_hex")?;
    let neighbor = fixed_hex(req, "neighbor_key_hex")?;
    let neighbor_value = fixed_hex(req, "neighbor_value_hex")?;
    let claimed_prefix_bits = field(req, "claimed_prefix_bits")?
        .as_u64()
        .ok_or_else(|| err("bad_field", "`claimed_prefix_bits` must be an integer"))?;
    let steps_value = field(req, "proof_steps")?
        .as_array()
        .ok_or_else(|| err("bad_field", "`proof_steps` must be an array"))?;
    let mut proof: Vec<(bool, Vec<u8>)> = Vec::with_capacity(steps_value.len());
    for (i, step) in steps_value.iter().enumerate() {
        let right = step
            .get("right")
            .and_then(Value::as_bool)
            .ok_or_else(|| err("bad_field", format!("proof_steps[{i}].right must be a boolean")))?;
        let cut_hex = step
            .get("cut_hex")
            .and_then(Value::as_str)
            .ok_or_else(|| err("bad_field", format!("proof_steps[{i}].cut_hex must be a string")))?;
        let cut = hex_decode(cut_hex)
            .ok_or_else(|| err("bad_field", format!("proof_steps[{i}].cut_hex is not valid hex")))?;
        proof.push((right, cut));
    }

    // Cryptographic half: the neighbor's inclusion proof folds to the root.
    let inclusion_holds = monotree::verify_proof(
        &MonotreeBlake3,
        Some(&root),
        &neighbor_value,
        Some(&proof),
    );
    // Prefix-law half: the queried key agrees with the neighbor on exactly
    // `claimed_prefix_bits` and diverges on the next bit, so it routes into
    // the neighbor's empty sibling subtree.
    let prefix_holds = claimed_prefix_bits < 256
        && common_prefix_bits(&queried, &neighbor) == claimed_prefix_bits
        && bit_at(&queried, claimed_prefix_bits) != bit_at(&neighbor, claimed_prefix_bits);

    Ok(obj(json!({"admitted": inclusion_holds && prefix_holds})))
}

/// `range_proof_verify`: bulletproofs bounded-metric check. Verifies that a
/// Pedersen commitment carries a value within `[0, 2^bits)` under the same
/// domain label the prover used — without the value ever being present.
pub fn op_range_proof_verify(req: &Value) -> Res<Map<String, Value>> {
    let commitment = fixed_hex(req, "commitment_hex")?;
    let proof_hex = field(req, "proof_hex")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`proof_hex` must be a hex string"))?;
    let proof_bytes = hex_decode(proof_hex)
        .ok_or_else(|| err("bad_field", "`proof_hex` is not valid hex"))?;
    let bits = field(req, "bits")?
        .as_u64()
        .ok_or_else(|| err("bad_field", "`bits` must be an integer"))? as usize;
    let label = field(req, "label")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`label` must be a string"))?;

    if bits == 0 || bits > 64 {
        return Err(err("bad_field", "`bits` must be in 1..=64"));
    }
    // merlin transcripts require a `\'static` label, so the label space is
    // a REGISTRY, not free text: only domain labels the kernel itself uses
    // for proving are admitted. This is an admission gate, not a filter.
    let registered: &[&str] = &["affidavit:zk-range:v1:effect-budget"];
    let label_bytes = match registered.iter().find(|l| **l == label) {
        Some(l) => l.as_bytes(),
        None => {
            return Err(err(
                "bad_field",
                format!("unknown domain label `{label}`; admitted labels: {registered:?}"),
            ))
        }
    };
    use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
    use curve25519_dalek_ng::ristretto::CompressedRistretto;
    use merlin::Transcript;

    let gens = BulletproofGens::new(bits.next_power_of_two().max(32), 1);
    let admitted = RangeProof::from_bytes(&proof_bytes)
        .map(|proof| {
            let mut transcript = Transcript::new(label_bytes);
            proof
                .verify_single(
                    &gens,
                    &PedersenGens::default(),
                    &mut transcript,
                    &CompressedRistretto(commitment),
                    bits,
                )
                .is_ok()
        })
        .unwrap_or(false);

    Ok(obj(json!({"admitted": admitted})))
}

/// Common prefix length in bits between two 32-byte keys, MSB-first.
fn common_prefix_bits(a: &[u8; HASH_LEN], b: &[u8; HASH_LEN]) -> u64 {
    for (index, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        if x != y {
            let xor = x ^ y;
            let first_diff_bit = index as u64 * 8 + u64::from(xor.leading_zeros());
            return first_diff_bit;
        }
    }
    256
}

/// The bit at `index` (MSB-first) of a 32-byte key. Index must be < 256.
fn bit_at(key: &[u8; HASH_LEN], index: u64) -> bool {
    let byte = (index / 8) as usize;
    let offset = 7 - (index % 8);
    (key[byte] >> offset) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn jcs_canonicalizes_and_commits() {
        // Keys arrive unsorted; the canonical form sorts them, strips
        // whitespace, and the commitment is the BLAKE3 of those bytes.
        let req = json!({"op":"jcs_canonicalize","json":"{ \"b\": 1, \"a\": 2 }"});
        let out = op_jcs_canonicalize(&req).expect("canonicalize");
        assert_eq!(out["canonical_json"], r#"{"a":2,"b":1}"#);
        let commitment = out["commitment"].as_str().expect("hex");
        assert_eq!(commitment.len(), 64, "BLAKE3 hex digest");
    }

    #[test]
    fn jcs_refuses_malformed_json() {
        let req = json!({"op":"jcs_canonicalize","json":"{not json"});
        assert!(op_jcs_canonicalize(&req).is_err());
    }

    #[test]
    fn prefix_helpers_agree_with_a_divergent_pair() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[0] = 0b1000_0000;
        b[0] = 0b0111_1111;
        assert_eq!(common_prefix_bits(&a, &b), 0);
        assert_ne!(bit_at(&a, 0), bit_at(&b, 0));

        b[0] = 0b1100_0000;
        assert_eq!(common_prefix_bits(&a, &b), 1);
        assert_ne!(bit_at(&a, 1), bit_at(&b, 1));
    }

    #[test]
    fn range_proof_refuses_garbage_without_panic() {
        let req = json!({
            "op":"range_proof_verify",
            "commitment_hex": format!("{:0>64}", "0"),
            "proof_hex": "00",
            "bits": 32,
            "label": "affidavit:zk-range:v1:effect-budget"
        });
        let out = op_range_proof_verify(&req).expect("total");
        assert_eq!(out["admitted"], Value::Bool(false), "garbage proof refused");
    }

    #[test]
    fn range_proof_refuses_out_of_range_bits() {
        let req = json!({
            "op":"range_proof_verify",
            "commitment_hex": format!("{:0>64}", "0"),
            "proof_hex": "00",
            "bits": 65,
            "label": "l"
        });
        assert!(op_range_proof_verify(&req).is_err());
    }

    #[test]
    fn smt_absence_refuses_tampered_witness_without_tree_state() {
        // All-zero vectors: root != fold(zero leaf, []) so the inclusion
        // half fails, and the prefix law fails too (identical keys).
        let req = json!({
            "op":"smt_absence_verify",
            "root_hex": format!("{:0>64}", "0"),
            "queried_key_hex": format!("{:0>64}", "0"),
            "neighbor_key_hex": format!("{:0>64}", "0"),
            "neighbor_value_hex": format!("{:0>64}", "0"),
            "claimed_prefix_bits": 0,
            "proof_steps": []
        });
        let out = op_smt_absence_verify(&req).expect("total");
        assert_eq!(out["admitted"], Value::Bool(false));
    }
}
