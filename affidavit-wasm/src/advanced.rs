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
    let bytes =
        hex_decode(s).ok_or_else(|| err("bad_field", format!("`{name}` is not valid hex")))?;
    bytes
        .try_into()
        .map_err(|_| err("bad_field", format!("`{name}` must be {HASH_LEN} bytes")))
}

/// Domain-separation tag for the commitment digest. Same tag the root trust
/// plane digests under (`src/crypto_trust_canonical.rs::DOMAIN_TAG`), so a
/// wasm commitment and a root-plane digest are members of one key space, not
/// two collision-adjacent hash domains.
const JCS_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// `jcs_canonicalize`: RFC 8785 canonical bytes + domain-separated BLAKE3
/// commitment for an arbitrary JSON document. The canonical form is the
/// byte-exact preimage any verifier must reproduce before hashing or signing.
///
/// BEHAVIOR CHANGE (register rows 60 + 80; both breaking, both pinned by
/// `tests/jcs_parity.rs`):
/// 1. Duplicate object keys are REFUSED with a typed `bad_json` error whose
///    message begins `malformed:` — the former parse layer silently took the
///    last occurrence (RFC 8785 / I-JSON require unique names). Detection is
///    a depth- and escape-aware raw-byte scan over the input, so a duplicate
///    is caught at ANY object depth, including keys that only collide after
///    escape decoding (`"a"` vs `"\\u0061"`).
/// 2. The commitment is domain-separated: BLAKE3 in key-derivation mode
///    under `JCS_DOMAIN_TAG` (`Hasher::new_derive_key`), NOT the unkeyed
///    `blake3(canonical)` previously emitted. Verifiers pinning the old
///    unkeyed digest formula must migrate; the untagged digest of the same
///    canonical bytes will not verify.
///
/// Key ordering is RFC 8785 \u00a73.2.3: object keys sort by their UTF-16
/// code-unit sequence (ES6 `Array.prototype.sort` semantics), NOT by Unicode
/// scalar value. `serde_jcs` orders by scalar value, which disagrees for
/// BMP-high keys (>= U+E000) against supplementary-plane keys, so the
/// ordering pass here is implemented directly: keys are compared as
/// `Vec<u16>` from `str::encode_utf16`, lexicographically. Scalar/number
/// serialization is delegated to the serde_json serializer (the same
/// serializer `serde_jcs` drives), so escaping and number formatting bytes
/// are unchanged from the previously-admitted surface.
pub fn op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> {
    let raw = field(req, "json")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`json` must be a string containing JSON"))?;
    if let Some(name) = first_duplicate_key(raw) {
        return Err(err(
            "bad_json",
            format!("malformed: duplicate object key `{name}`; RFC 8785/I-JSON require unique member names"),
        ));
    }
    let parsed: Value = serde_json::from_str(raw).map_err(|e| err("bad_json", e.to_string()))?;
    let canonical = jcs_serialize(&parsed);
    let commitment = jcs_commitment(&canonical);
    Ok(obj(
        json!({"canonical_json": canonical, "commitment": commitment}),
    ))
}

/// Domain-separated commitment over the canonical bytes: BLAKE3 keyed via the
/// key-derivation mode under [`JCS_DOMAIN_TAG`] (external blake3 crate API),
/// not an unkeyed hash — the same domain tag the root trust plane uses.
fn jcs_commitment(canonical: &str) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(JCS_DOMAIN_TAG);
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// One open container during the raw-byte duplicate-key scan.
struct ScanFrame {
    is_object: bool,
    /// Member names already seen in this object (escape-decoded).
    keys: Vec<String>,
    /// Object frames: `true` while the next string token is a member NAME.
    expect_key: bool,
}

/// Depth- and escape-aware raw-byte scan for duplicate object member names at
/// ANY object depth. Returns the first colliding name, if any. Raw bytes, not
/// the parsed `Value`: by the time this runs, `serde_json` has already
/// last-wins-folded any duplicates out of the parsed tree, so only the input
/// text still carries the evidence. String tokens are fully escape-decoded
/// (including `\\uXXXX` surrogate pairs), so `"a"` and `"\\u0061"` collide.
fn first_duplicate_key(raw: &str) -> Option<String> {
    let b = raw.as_bytes();
    let mut i = 0usize;
    let mut stack: Vec<ScanFrame> = Vec::new();
    while i < b.len() {
        match b[i] {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            b'{' => stack.push(ScanFrame {
                is_object: true,
                keys: Vec::new(),
                expect_key: true,
            }),
            b'[' => stack.push(ScanFrame {
                is_object: false,
                keys: Vec::new(),
                expect_key: false,
            }),
            b'}' | b']' => {
                stack.pop();
            }
            b',' => {
                if let Some(top) = stack.last_mut() {
                    top.expect_key = top.is_object;
                }
            }
            b'"' => {
                let (name, next) = scan_json_string(b, i);
                if matches!(
                    stack.last(),
                    Some(ScanFrame {
                        is_object: true,
                        expect_key: true,
                        ..
                    })
                ) {
                    let top = stack.last_mut().expect("just matched");
                    if top.keys.iter().any(|k| k == &name) {
                        return Some(name);
                    }
                    top.keys.push(name);
                    top.expect_key = false;
                }
                i = next;
                continue;
            }
            // Numbers, `true`/`false`/`null` literals, `:` — no key
            // information; the input has already parsed as valid JSON, so
            // byte-stepping over them is sound.
            _ => {}
        }
        i += 1;
    }
    None
}

/// Decode a JSON string literal starting at `start` (a `b'"'` byte); returns
/// the escape-decoded name and the index just past the closing quote.
fn scan_json_string(b: &[u8], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start + 1;
    while i < b.len() {
        match b[i] {
            b'"' => return (out, i + 1),
            b'\\' => {
                i += 1;
                if i >= b.len() {
                    return (out, i);
                }
                match b[i] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let hi = hex4(b, i + 1);
                        i += 4;
                        if (0xD800..=0xDBFF).contains(&hi) {
                            // High surrogate: the low half follows as \uXXXX.
                            if i + 2 < b.len() && b[i + 1] == b'\\' && b[i + 2] == b'u' {
                                let lo = hex4(b, i + 3);
                                i += 5;
                                let cp =
                                    0x10000 + ((hi - 0xD800) << 10) + (lo.wrapping_sub(0xDC00));
                                if let Some(c) = char::from_u32(cp) {
                                    out.push(c);
                                }
                            }
                        } else if let Some(c) = char::from_u32(hi) {
                            out.push(c);
                        }
                    }
                    other => out.push(other as char),
                }
            }
            byte => out.push(byte as char),
        }
        i += 1;
    }
    (out, i)
}

/// Four hex digits at `at` as a u32 (0 on malformed — inputs are pre-validated
/// by the serde parse, so this path is defensive only).
fn hex4(b: &[u8], at: usize) -> u32 {
    let mut v = 0u32;
    for k in 0..4 {
        let digit = b
            .get(at + k)
            .and_then(|&c| (c as char).to_digit(16))
            .unwrap_or(0);
        v = v * 16 + digit;
    }
    v
}

/// RFC 8785 canonical serialization with UTF-16 code-unit key ordering.
/// Scalars delegate to the existing admitted encoders: strings escape via
/// serde_json's serializer, numbers format via `serde_jcs` (RFC 8785 ES6
/// number bytes). Only the key-ordering pass is implemented here.
fn jcs_serialize(v: &Value) -> String {
    let mut out = String::new();
    jcs_write(&mut out, v);
    out
}

fn jcs_write(out: &mut String, v: &Value) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        // serde_jcs implements the RFC 8785 ES6 number format (e.g. 0.0 ->
        // "0"); serde_json's Number Display does not. Numbers carry no key
        // ordering, so delegating the scalar to serde_jcs is byte-exact.
        Value::Number(n) => {
            let formatted = serde_jcs::to_string(n).expect("number serialization is infallible");
            out.push_str(&formatted);
        }
        // serde_json::to_string of a &str is infallible (no user serializers).
        Value::String(s) => {
            let escaped = serde_json::to_string(s).expect("str serialization is infallible");
            out.push_str(&escaped);
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                jcs_write(out, item);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                let escaped = serde_json::to_string(key).expect("str serialization is infallible");
                out.push_str(&escaped);
                out.push(':');
                jcs_write(out, value);
            }
            out.push('}');
        }
    }
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
        let right = step.get("right").and_then(Value::as_bool).ok_or_else(|| {
            err(
                "bad_field",
                format!("proof_steps[{i}].right must be a boolean"),
            )
        })?;
        let cut_hex = step.get("cut_hex").and_then(Value::as_str).ok_or_else(|| {
            err(
                "bad_field",
                format!("proof_steps[{i}].cut_hex must be a string"),
            )
        })?;
        let cut = hex_decode(cut_hex).ok_or_else(|| {
            err(
                "bad_field",
                format!("proof_steps[{i}].cut_hex is not valid hex"),
            )
        })?;
        proof.push((right, cut));
    }

    // Cryptographic half: the neighbor's inclusion proof folds to the root.
    let inclusion_holds =
        monotree::verify_proof(&MonotreeBlake3, Some(&root), &neighbor_value, Some(&proof));
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
    let proof_bytes =
        hex_decode(proof_hex).ok_or_else(|| err("bad_field", "`proof_hex` is not valid hex"))?;
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
    let registered: &[&str] = &[
        "affidavit:zk-range:v1:effect-budget",
        "affidavit:zk-range:v1:receipt-amount",
    ];
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
        // whitespace, and the commitment is the domain-separated BLAKE3
        // (key-derivation mode under JCS_DOMAIN_TAG) of those bytes.
        let req = json!({"op":"jcs_canonicalize","json":"{ \"b\": 1, \"a\": 2 }"});
        let out = op_jcs_canonicalize(&req).expect("canonicalize");
        assert_eq!(out["canonical_json"], r#"{"a":2,"b":1}"#);
        let commitment = out["commitment"].as_str().expect("hex");
        assert_eq!(commitment.len(), 64, "BLAKE3 hex digest");
    }

    #[test]
    fn jcs_orders_keys_by_utf16_code_units_not_scalar_value() {
        // RFC 8785 §3.2.3: U+E000 (BMP-high, code unit 0xE000) sorts AFTER
        // any supplementary-plane key, whose UTF-16 lead surrogate is
        // 0xD800..=0xDBFF. Scalar-value ordering says the opposite.
        let input = "{\"\\ud83d\\ude00\":1,\"\\ue000\":2}";
        let req = json!({"op":"jcs_canonicalize","json": input});
        let out = op_jcs_canonicalize(&req).expect("canonicalize");
        let expected = "{\"".to_owned() + "\u{1F600}" + "\":1,\"\u{E000}\":2}";
        assert_eq!(
            out["canonical_json"], expected,
            "astral key must precede BMP-high U+E000 (UTF-16 ordering)"
        );
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
    fn range_proof_registry_admits_second_label_and_refuses_unknown() {
        // Positive control at the registry gate: the new receipt-amount label
        // is a registered domain (typed Ok verdict, garbage proof -> refused)
        // while an unregistered label is a typed bad_field, not a verdict.
        let garbage = json!({
            "op":"range_proof_verify",
            "commitment_hex": format!("{:0>64}", "0"),
            "proof_hex": "00",
            "bits": 32,
            "label": "affidavit:zk-range:v1:receipt-amount"
        });
        let out = op_range_proof_verify(&garbage).expect("registered label admitted");
        assert_eq!(out["admitted"], Value::Bool(false), "garbage proof refused");

        let unknown = json!({
            "op":"range_proof_verify",
            "commitment_hex": format!("{:0>64}", "0"),
            "proof_hex": "00",
            "bits": 32,
            "label": "affidavit:zk-range:v1:not-registered"
        });
        let e = op_range_proof_verify(&unknown).expect_err("unknown label refused");
        assert_eq!(e.code, "bad_field");
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
    #[test]
    fn jcs_refuses_duplicate_keys_at_any_depth() {
        // Top-level duplicate: the former surface silently last-wins'd to 2.
        let top = json!({"op":"jcs_canonicalize","json":r#"{"a":1,"b":2,"a":3}"#});
        let e = op_jcs_canonicalize(&top).expect_err("top-level duplicate refused");
        assert_eq!(e.code, "bad_json");
        assert!(e.message.contains("malformed: duplicate object key `a`"));

        // Nested duplicate: only the inner object collides.
        let nested =
            json!({"op":"jcs_canonicalize","json":r#"{"outer":{"x":1,"y":{"x":2,"x":3}}}"#});
        let e = op_jcs_canonicalize(&nested).expect_err("nested duplicate refused");
        assert_eq!(e.code, "bad_json");
        assert!(e.message.contains("`x`"));

        // Escape-decoded collision: distinct raw bytes, same member name.
        let escaped = json!({"op":"jcs_canonicalize","json":r#"{"a":1,"\u0061":2}"#});
        let e = op_jcs_canonicalize(&escaped).expect_err("escape-decoded duplicate refused");
        assert_eq!(e.code, "bad_json");

        // Same name in SIBLING objects is lawful, not a duplicate.
        let siblings = json!({"op":"jcs_canonicalize","json":r#"{"p":{"k":1},"q":{"k":2}}"#});
        let out = op_jcs_canonicalize(&siblings).expect("sibling keys are lawful");
        assert_eq!(out["canonical_json"], r#"{"p":{"k":1},"q":{"k":2}}"#);
    }

    #[test]
    fn jcs_commitment_is_domain_separated_not_unkeyed() {
        let req = json!({"op":"jcs_canonicalize","json":r#"{"b":1,"a":2}"#});
        let out = op_jcs_canonicalize(&req).expect("canonicalize");
        let canonical = out["canonical_json"].as_str().expect("canonical");
        let commitment = out["commitment"].as_str().expect("hex");

        let mut keyed = blake3::Hasher::new_derive_key(JCS_DOMAIN_TAG);
        keyed.update(canonical.as_bytes());
        assert_eq!(commitment, keyed.finalize().to_hex().to_string());

        // The untagged digest of the same bytes must NOT verify — a host
        // still pinning the old unkeyed formula fails loudly.
        assert_ne!(
            commitment,
            blake3::hash(canonical.as_bytes()).to_hex().to_string(),
            "commitment must not be the unkeyed blake3(canonical)"
        );
    }
}
