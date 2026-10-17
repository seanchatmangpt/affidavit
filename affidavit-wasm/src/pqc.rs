//! Post-quantum verification ops over the `affidavit_core::pqc` seam.
//!
//! The court, the FIPS 204 (ML-DSA-65) and FIPS 205 (SLH-DSA-SHA2-128s)
//! contracts and the external `ml-dsa` and `slh-dsa` adapters all live in
//! `affidavit-core`; this module is the WASM JSON-ABI projection of that
//! seam. No arithmetic on lattice or hash internals happens here - bytes are
//! routed to the core court and its verdict is reported verbatim.
//!
//! Ops:
//!
//! * `pqc_verify` - verify ML-DSA-65 and/or SLH-DSA-SHA2-128s signatures over
//!   a message and report the verdict JSON. A failed verification is a
//!   verdict, not an ABI error: like `verify`, a verifier answers "no"
//!   rather than throwing.
//! * `pqc_assemble` - the admit-and-seal extension: seals an event chain via
//!   the same deterministic seal path as `assemble`, then binds the PQC
//!   verdict into the seal. The signed message is the compact JSON
//!   serialization of the request's `events` array (`serde_json::to_vec`),
//!   so a caller can compute and sign it without knowing the sealing
//!   internals. Profile-gated: when the declared profile demands an
//!   algorithm and its verification fails, the seal is refused with a typed
//!   `pqc_refused` error - never a receipt without its signature check.
//!
//! Doctrine, preserved: *certify, don't decide.* The court certifies what
//! the external FIPS 204/205 verifiers said; nothing here judges honesty.

use affidavit_core::pqc::{
    ExternalMlDsa65, ExternalSlhDsa128s, PqcCourt, PqcProfile, PqcRefusal, PqcVerdict, Suite,
};
use serde_json::{json, Map, Value};

use crate::abi::{err, field, hex_decode, obj, AbiError, Res};
use crate::receipt;

/// The empty input for a suite whose keys the request does not carry.
#[cfg(test)]
const EMPTY: &[u8] = &[];

/// The full court: both external verifiers registered, so any profile that
/// demands an algorithm always reaches its verifier.
fn court() -> PqcCourt {
    PqcCourt::new()
        .with_ml_dsa(&ExternalMlDsa65)
        .with_slh_dsa(&ExternalSlhDsa128s)
}

/// Decode a hex field, typed error on malformed input.
fn hex_field(req: &Value, name: &str) -> Res<Vec<u8>> {
    let s = field(req, name)?
        .as_str()
        .ok_or_else(|| err("bad_field", format!("`{name}` must be a hex string")))?;
    hex_decode(s).ok_or_else(|| err("bad_field", format!("`{name}` is not valid hex")))
}

/// Optional hex field: empty when the field is absent or null.
fn hex_optional(req: &Value, name: &str) -> Res<Vec<u8>> {
    match req.get(name) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(_) => hex_field(req, name),
    }
}

/// Message bytes from `msg` (UTF-8 text) or `msg_hex`.
fn msg_bytes(req: &Value) -> Res<Vec<u8>> {
    if let Some(m) = req.get("msg") {
        let s = m
            .as_str()
            .ok_or_else(|| err("bad_field", "`msg` must be a string"))?;
        return Ok(s.as_bytes().to_vec());
    }
    hex_field(req, "msg_hex")
}

/// Parse the declared profile from the given string value.
///
/// Grammar: `classical` | `hybrid` | `pqc` | `pqc(slh-dsa-sha2-128s)` |
/// `pqc(dual)`. The bare `pqc` names the default ML-DSA-65 suite; the
/// parenthesized forms name the others (matching `PqcProfile`'s `Display`).
fn parse_profile(s: &str, key: &str) -> Res<PqcProfile> {
    match s {
        "classical" => Ok(PqcProfile::Classical),
        "hybrid" => Ok(PqcProfile::Hybrid),
        "pqc" => Ok(PqcProfile::Pqc(Suite::MlDsa65)),
        "pqc(slh-dsa-sha2-128s)" => Ok(PqcProfile::Pqc(Suite::SlhDsa128s)),
        "pqc(dual)" => Ok(PqcProfile::Pqc(Suite::Dual)),
        other => Err(err(
            "bad_field",
            format!(
                "`{key}` must be one of `classical`, `hybrid`, `pqc`, \
                 `pqc(slh-dsa-sha2-128s)`, `pqc(dual)`; got `{other}`"
            ),
        )),
    }
}

/// Profile from the request's optional profile field.
fn profile_field(req: &Value, key: &str) -> Res<PqcProfile> {
    match req.get(key) {
        None | Some(Value::Null) => Ok(PqcProfile::Pqc(Suite::MlDsa65)),
        Some(v) => {
            let s = v
                .as_str()
                .ok_or_else(|| err("bad_field", format!("`{key}` must be a string")))?;
            parse_profile(s, key)
        }
    }
}

/// Typed refusal code for the response `refusal` field.
fn refusal_code(r: &PqcRefusal) -> &'static str {
    match r {
        PqcRefusal::MissingMlDsa65Verifier => "missing_ml_dsa65_verifier",
        PqcRefusal::MlDsa65Rejected => "ml_dsa65_rejected",
        PqcRefusal::MissingSlhDsaVerifier => "missing_slh_dsa_verifier",
        PqcRefusal::SlhDsaRejected => "slh_dsa_rejected",
    }
}

/// The suite's algorithm name for the response `algorithm` field.
fn suite_name(profile: &PqcProfile) -> &'static str {
    match profile {
        PqcProfile::Classical => "none",
        PqcProfile::Hybrid | PqcProfile::Pqc(Suite::MlDsa65) => "ML-DSA-65",
        PqcProfile::Pqc(Suite::SlhDsa128s) => "SLH-DSA-SHA2-128s",
        PqcProfile::Pqc(Suite::Dual) => "ML-DSA-65 + SLH-DSA-SHA2-128s",
    }
}

/// Render a `PqcVerdict` as response fields (verdict JSON, never an error).
fn verdict_fields(profile: &PqcProfile, verdict: &PqcVerdict) -> Map<String, Value> {
    let (accepted, verdict_refusal) = match verdict {
        PqcVerdict::Accept => (true, None),
        PqcVerdict::Refuse(r) => (false, Some(refusal_code(r))),
    };
    let mut out = obj(json!({
        "profile": profile.to_string(),
        "algorithm": suite_name(profile),
        "accepted": accepted,
    }));
    if let Some(code) = verdict_refusal {
        out.insert("refusal".into(), json!(code));
    }
    out
}

/// `pqc_verify` op: `{"op":"pqc_verify","msg"|"msg_hex","sig_hex","pk_hex",
/// "slh_dsa_sig_hex"?,"slh_dsa_pk_hex"?,"profile"?}` -> verdict JSON.
///
/// `sig_hex`/`pk_hex` carry the ML-DSA-65 pair; `slh_dsa_sig_hex`/
/// `slh_dsa_pk_hex` carry the SLH-DSA pair (both required for `pqc(dual)`,
/// which judges ML-DSA-65 first, then SLH-DSA).
pub fn op_pqc_verify(req: &Value) -> Res<Map<String, Value>> {
    let profile = profile_field(req, "profile")?;
    let msg = msg_bytes(req)?;
    let ml_sig = hex_field(req, "sig_hex")?;
    let ml_pk = hex_field(req, "pk_hex")?;
    let slh_sig = hex_optional(req, "slh_dsa_sig_hex")?;
    let slh_pk = hex_optional(req, "slh_dsa_pk_hex")?;
    let verdict = court().judge(profile, &msg, (&ml_sig, &ml_pk), (&slh_sig, &slh_pk));
    Ok(verdict_fields(&profile, &verdict))
}

/// `pqc_assemble` op: seal `events` and bind the PQC verdict into the seal.
/// Request: `{"op":"pqc_assemble","events":[...],"ml_dsa_sig_hex",
/// "ml_dsa_pk_hex",("slh_dsa_sig_hex"/"slh_dsa_pk_hex")?,"pqc_profile"?}`.
/// The signed message is the compact JSON serialization of the `events`
/// array. Profile-gated seal: a profile whose demanded algorithm fails is a
/// typed `pqc_refused` error; `classical` needs no signature bytes.
pub fn op_pqc_assemble(req: &Value) -> Res<Map<String, Value>> {
    let profile = profile_field(req, "pqc_profile")?;
    let events = field(req, "events")?
        .as_array()
        .ok_or_else(|| err("bad_field", "`events` must be an array"))?;
    let ml_sig = hex_field(req, "ml_dsa_sig_hex")?;
    let ml_pk = hex_field(req, "ml_dsa_pk_hex")?;
    let slh_sig = hex_optional(req, "slh_dsa_sig_hex")?;
    let slh_pk = hex_optional(req, "slh_dsa_pk_hex")?;

    // The caller-signable message: compact JSON of the events array.
    let bound = serde_json::to_vec(events).map_err(|e| err("internal", e.to_string()))?;

    // Same deterministic seal path as the `assemble` op.
    let sealed = crate::abi::assemble(req)?;
    let verdict = court().judge(profile, &bound, (&ml_sig, &ml_pk), (&slh_sig, &slh_pk));
    match verdict {
        PqcVerdict::Accept => {
            let mut out = sealed;
            out.insert(
                "pqc".into(),
                json!({
                    "profile": profile.to_string(),
                    "algorithm": suite_name(&profile),
                    "accepted": true,
                    "msg_commitment": receipt::blake3_hex(&bound),
                    "pk_commitment": receipt::blake3_hex(&ml_pk),
                }),
            );
            Ok(out)
        }
        PqcVerdict::Refuse(r) => Err(AbiError {
            code: "pqc_refused",
            message: format!("pqc_assemble refused: {r}"),
            details: Some(json!({
                "profile": profile.to_string(),
                "algorithm": suite_name(&profile),
                "refusal": refusal_code(&r),
            })),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classical_profile_never_consults_the_verifier() {
        let v = court().judge(PqcProfile::Classical, b"m", (b"s", b"k"), (EMPTY, EMPTY));
        assert_eq!(v, PqcVerdict::Accept);
    }

    #[test]
    fn profile_grammar_covers_every_suite_and_refuses_unknown_values() {
        assert_eq!(
            parse_profile("classical", "p").unwrap(),
            PqcProfile::Classical
        );
        assert_eq!(parse_profile("hybrid", "p").unwrap(), PqcProfile::Hybrid);
        assert_eq!(
            parse_profile("pqc", "p").unwrap(),
            PqcProfile::Pqc(Suite::MlDsa65)
        );
        assert_eq!(
            parse_profile("pqc(slh-dsa-sha2-128s)", "p").unwrap(),
            PqcProfile::Pqc(Suite::SlhDsa128s)
        );
        assert_eq!(
            parse_profile("pqc(dual)", "p").unwrap(),
            PqcProfile::Pqc(Suite::Dual)
        );
        assert!(parse_profile("pqc(lattice-max)", "p").is_err());
        // Display names round-trip for the pqc profiles.
        assert_eq!(PqcProfile::Pqc(Suite::Dual).to_string(), "pqc(dual)");
    }
}
