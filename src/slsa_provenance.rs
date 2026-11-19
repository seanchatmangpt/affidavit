//! Offline verifier for sealed in-toto Statement v1 / SLSA provenance envelopes
//! (FNOX-SJ-PROV-3).
//!
//! Envelope shape (produced by fnox `in_toto_gcp.seal_statement`):
//! `{statement, digest (blake3 hex of canonical statement bytes), kms_signature
//! (base64 ECDSA P-256 over blake3(digest bytes), prehashed), ...}`.
//!
//! Canonical statement bytes = compact JSON, sorted keys, ASCII-escaped
//! (matches Python `json.dumps(sort_keys=True, separators=(",", ":"))`).
//! No network, no clock: every input is the envelope plus a caller-supplied
//! SEC1 public key.

use p256::ecdsa::signature::hazmat::PrehashVerifier;
use p256::ecdsa::{Signature, VerifyingKey};
use serde_json::Value;

pub const IN_TOTO_STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
pub const SLSA_PREDICATE_TYPE: &str = "https://slsa.dev/provenance/v1";

#[derive(Debug, PartialEq, Eq)]
pub enum SlsaError {
    NotAnEnvelope(String),
    DigestMismatch { recomputed: String, sealed: String },
    WrongStatementType(String),
    WrongPredicate { expected: String, found: String },
    BadSignatureEncoding(String),
    BadPublicKey(String),
    SignatureInvalid(String),
    UnsupportedNumber,
}

impl std::fmt::Display for SlsaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SlsaError {}

#[derive(Debug, PartialEq, Eq)]
pub struct Verified {
    pub digest: String,
    pub predicate_type: String,
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e && (c as u32) != 0x7f => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_canonical(v: &Value, out: &mut String) -> Result<(), SlsaError> {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_f64() {
                return Err(SlsaError::UnsupportedNumber);
            }
            out.push_str(&n.to_string());
        }
        Value::String(s) => write_string(out, s),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(x, out)?;
            }
            out.push(']');
        }
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, k);
                out.push(':');
                write_canonical(&m[*k], out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Canonical statement bytes (see module docs).
pub fn statement_bytes(statement: &Value) -> Result<Vec<u8>, SlsaError> {
    let mut s = String::new();
    write_canonical(statement, &mut s)?;
    Ok(s.into_bytes())
}

/// blake3 hex over the canonical statement bytes.
pub fn statement_digest(statement: &Value) -> Result<String, SlsaError> {
    Ok(blake3::hash(&statement_bytes(statement)?).to_hex().to_string())
}

fn b64_decode(s: &str) -> Result<Vec<u8>, SlsaError> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.trim_end_matches('=').bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => {
                return Err(SlsaError::BadSignatureEncoding(format!(
                    "invalid base64 byte {c:#x}"
                )))
            }
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, SlsaError> {
    if s.len() % 2 != 0 {
        return Err(SlsaError::BadSignatureEncoding("odd hex length".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| SlsaError::BadSignatureEncoding(e.to_string()))
        })
        .collect()
}

/// Verify a sealed envelope offline.
///
/// Order: shape -> statement digest (tamper) -> statement `_type` ->
/// predicate type -> ECDSA P-256 prehashed verify of `kms_signature` over
/// `blake3(digest bytes)` with the SEC1-encoded public key.
pub fn verify_sealed_envelope(
    sealed: &Value,
    sec1_public_key: &[u8],
    expected_predicate_type: &str,
) -> Result<Verified, SlsaError> {
    let statement = sealed
        .get("statement")
        .ok_or_else(|| SlsaError::NotAnEnvelope("no 'statement' key".into()))?;
    let sealed_digest = sealed
        .get("digest")
        .and_then(Value::as_str)
        .ok_or_else(|| SlsaError::NotAnEnvelope("no 'digest' string".into()))?;
    let recomputed = statement_digest(statement)?;
    if recomputed != sealed_digest {
        return Err(SlsaError::DigestMismatch {
            recomputed,
            sealed: sealed_digest.to_string(),
        });
    }
    let stype = statement.get("_type").and_then(Value::as_str).unwrap_or("");
    if stype != IN_TOTO_STATEMENT_TYPE {
        return Err(SlsaError::WrongStatementType(stype.to_string()));
    }
    let found = statement
        .get("predicateType")
        .and_then(Value::as_str)
        .unwrap_or("");
    if found != expected_predicate_type {
        return Err(SlsaError::WrongPredicate {
            expected: expected_predicate_type.to_string(),
            found: found.to_string(),
        });
    }
    let sig_b64 = sealed
        .get("kms_signature")
        .and_then(Value::as_str)
        .ok_or_else(|| SlsaError::NotAnEnvelope("no 'kms_signature' string".into()))?;
    let sig_bytes = b64_decode(sig_b64)?;
    let sig = Signature::from_der(&sig_bytes)
        .or_else(|_| Signature::from_slice(&sig_bytes))
        .map_err(|e| SlsaError::BadSignatureEncoding(e.to_string()))?;
    let key = VerifyingKey::from_sec1_bytes(sec1_public_key)
        .map_err(|e| SlsaError::BadPublicKey(e.to_string()))?;
    let digest_bytes = hex_decode(&recomputed)?;
    let msg = blake3::hash(&digest_bytes);
    key.verify_prehash(msg.as_bytes(), &sig)
        .map_err(|e| SlsaError::SignatureInvalid(e.to_string()))?;
    Ok(Verified {
        digest: recomputed,
        predicate_type: found.to_string(),
    })
}

/// Decode a lowercase/uppercase hex SEC1 public key (fixture convenience).
pub fn sec1_from_hex(hex: &str) -> Result<Vec<u8>, SlsaError> {
    hex_decode(hex.trim())
}
