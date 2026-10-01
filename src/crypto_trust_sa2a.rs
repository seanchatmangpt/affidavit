//! SA2A wire interop (RFC-SA2A-007-errata) for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! The bridge: an affidavit [`SignatureEnvelope`] + signature IS a valid SA2A
//! approval and vice versa. The SA2A signed message is exactly
//! `SA2A-C2-APPROVAL-v1 || 0x00 || JCS(approval document)` (ctp:sa2a-interop-v1
//! ctp:approvalTag), where the approval document's field names are exactly the
//! errata set (ctp:sa2a-interop-v1 ctp:targetFieldSet). [`envelope_to_approval`]
//! and [`approval_to_envelope`] are mechanical inverse mappings;
//! [`sa2a_signing_input`] frames the SA2A pre-image.
//!
//! Domain separation is the interlock tooth: the affidavit pre-image
//! ([`crate::crypto_trust_envelope::SignatureEnvelope::signing_input`],
//! domain-separated under [`crate::crypto_trust_canonical::DOMAIN_TAG`]) and
//! the SA2A pre-image (tag || 0x00 || JCS) are different byte strings, so a
//! signature made over one NEVER verifies over the other. The falsifier test
//! witnesses this both ways with a real ES256 key: cross-domain signatures are
//! refused, proving the two domains are distinct bytes, not decorative names.
//!
//! House law: refusal-as-value. Every refusal is a typed [`Sa2aWireError`]
//! variant; nothing here panics, coerces, or fakes.
//!
//! Consumed query columns (sa2a.rq): approval_tag, envelope_fields.

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_envelope::{SignatureEnvelope, ENVELOPE_VERSION};
use crate::crypto_trust_keys::{AlgorithmId, KeyId};

/// SA2A signed-message domain tag (ctp:sa2a-interop-v1 ctp:approvalTag).
/// [`sa2a_signing_input`] frames it as `tag || 0x00 || JCS(approval)`; its
/// trailing component is also the approval schema version `v`.
pub const SA2A_APPROVAL_TAG: &str = "SA2A-C2-APPROVAL-v1";

/// The graph's ctp:SignatureEnvelopeField names in ctp:fieldOrder — the
/// authority the field-coverage test holds [`envelope_to_approval`] to: every
/// envelope field must survive the bridge, so a graph-side field change breaks
/// the render loudly instead of silently dropping a signed field from the
/// SA2A wire form.
pub const ENVELOPE_FIELD_NAMES: [&str; 12] = [
    "version",
    "algorithm",
    "key_id",
    "profile",
    "policy_epoch",
    "revocation_epoch",
    "generation",
    "nonce",
    "not_before",
    "expires_at",
    "subject_digest",
    "audience",
];

/// The RFC-SA2A-007-errata approval: the foreign wire form of a signed
/// envelope. The struct field NAMES are exactly the errata set
/// (ctp:sa2a-interop-v1 ctp:targetFieldSet), so the derived serde
/// serialization IS the approval document [`sa2a_signing_input`] canonicalizes.
/// Binary envelope fields travel as lowercase hex (`nonce` 32 characters,
/// `effect_digest` 64).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Sa2aApproval {
    /// Approval schema version: the trailing component of
    /// [`SA2A_APPROVAL_TAG`] ("SA2A-C2-APPROVAL-v1" -> "1"). Derived, never
    /// hand-typed, so a graph-side tag bump moves `v` with it.
    pub v: String,
    /// Algorithm identity: the graph's `ctp:algorithmName` (e.g. "ES256").
    pub alg: String,
    /// Key id: the registry-unique `afk1_...` identifier.
    pub kid: String,
    /// The envelope's `subject_digest` as 64 lowercase-hex characters.
    pub effect_digest: String,
    /// The SA2A principal the approval is issued to. Not an envelope field —
    /// the envelope carries no principal; the bridge binds it at mapping time
    /// and it lives inside the signed bytes.
    pub principal: String,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u32,
    /// The envelope's nonce as 32 lowercase-hex characters.
    pub nonce: String,
    pub not_before: u64,
    /// The envelope's `expires_at`.
    pub expires: u64,
    pub audience: String,
}

/// Typed refusal of a SA2A wire-bridge operation; refusal-as-value, never a
/// panic.
#[derive(Debug, thiserror::Error)]
pub enum Sa2aWireError {
    /// The envelope's `version` is not [`ENVELOPE_VERSION`]; the payload
    /// carries the refused version.
    #[error("unknown envelope version {0}")]
    WrongVersion(String),
    /// The approval's `alg` is not a graph-declared `ctp:algorithmName`; the
    /// payload carries the refused string.
    #[error("unknown algorithm {0}")]
    UnknownAlgorithm(String),
    /// A hex field is not exactly lowercase hex — a non-hex digit, an
    /// uppercase digit, or an odd length; the payload carries the refused
    /// string.
    #[error("malformed hex: {0}")]
    MalformedHex(String),
    /// The approval is structurally wrong where hex cannot explain it: a hex
    /// field decodes to the wrong byte length for its fixed-width envelope
    /// field, or the document cannot canonicalize (an integer beyond 2^53).
    #[error("malformed approval: {0}")]
    Malformed(String),
}

/// Lowercase-hex encoding (house pattern: no hex-crate dependency).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// One lowercase-hex digit — lowercase only. The wire form is the
/// [`hex_encode`] alphabet, and the bridge certifies rather than coerces:
/// uppercase is refused, not silently accepted.
fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// Decodes exactly `N` bytes of lowercase hex. Refusals, typed (in this
/// order): [`Sa2aWireError::MalformedHex`] for odd length or a non-lowercase
/// digit; [`Sa2aWireError::Malformed`] for even-length hex of the wrong
/// character count (`2 * N`) for the fixed-width envelope field.
fn hex_decode_fixed<const N: usize>(hex: &str) -> Result<[u8; N], Sa2aWireError> {
    let bytes = hex.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(Sa2aWireError::MalformedHex(hex.to_string()));
    }
    if bytes.len() != N * 2 {
        return Err(Sa2aWireError::Malformed(format!(
            "hex field must be {} characters ({} bytes), got {}",
            N * 2,
            N,
            bytes.len()
        )));
    }
    let mut out = [0u8; N];
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        let hi = hex_digit(pair[0]).ok_or_else(|| Sa2aWireError::MalformedHex(hex.to_string()))?;
        let lo = hex_digit(pair[1]).ok_or_else(|| Sa2aWireError::MalformedHex(hex.to_string()))?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

/// The approval schema version carried in `v`: the trailing `-v<N>` component
/// of the graph-rendered [`SA2A_APPROVAL_TAG`] minus its `v` marker
/// ("SA2A-C2-APPROVAL-v1" -> "1").
fn approval_schema_version() -> &'static str {
    let tail = SA2A_APPROVAL_TAG
        .rsplit_once('-')
        .map(|(_, version)| version)
        .unwrap_or(SA2A_APPROVAL_TAG);
    tail.strip_prefix('v').unwrap_or(tail)
}

/// Bridges an affidavit envelope to the SA2A wire form: a mechanical field
/// mapping (`nonce` -> lowercase hex, `subject_digest` -> `effect_digest`
/// lowercase hex, `expires_at` -> `expires`, envelope `version` -> the
/// derived schema version `v`). `principal` is bound here because the
/// envelope carries no principal. Refuses
/// [`Sa2aWireError::WrongVersion`] when `env.version` is not
/// [`ENVELOPE_VERSION`].
pub fn envelope_to_approval(
    env: &SignatureEnvelope,
    principal: &str,
) -> Result<Sa2aApproval, Sa2aWireError> {
    if env.version != ENVELOPE_VERSION {
        return Err(Sa2aWireError::WrongVersion(env.version.clone()));
    }
    Ok(Sa2aApproval {
        v: approval_schema_version().to_string(),
        alg: env.algorithm.as_str().to_string(),
        kid: env.key_id.0.clone(),
        effect_digest: hex_encode(&env.subject_digest),
        principal: principal.to_string(),
        policy_epoch: env.policy_epoch,
        revocation_epoch: env.revocation_epoch,
        generation: env.generation,
        nonce: hex_encode(&env.nonce),
        not_before: env.not_before,
        expires: env.expires_at,
        audience: env.audience.clone(),
    })
}

/// Inverse bridge: SA2A approval back to an affidavit envelope. The envelope
/// `version` is pinned to [`ENVELOPE_VERSION`] and the assurance `profile` is
/// derived from the algorithm (the graph law: profile is a function of
/// algorithm), so the mapping is lossless for graph-lawful approvals.
/// Refusals, typed and in this order:
/// [`Sa2aWireError::UnknownAlgorithm`] when `alg` is not a graph-declared
/// `ctp:algorithmName`; [`Sa2aWireError::MalformedHex`] /
/// [`Sa2aWireError::Malformed`] from the fixed-width hex decodes of `nonce`
/// and `effect_digest`.
pub fn approval_to_envelope(a: &Sa2aApproval) -> Result<SignatureEnvelope, Sa2aWireError> {
    let algorithm = AlgorithmId::all()
        .iter()
        .copied()
        .find(|alg| alg.as_str() == a.alg)
        .ok_or_else(|| Sa2aWireError::UnknownAlgorithm(a.alg.clone()))?;
    let nonce = hex_decode_fixed::<16>(&a.nonce)?;
    let subject_digest = hex_decode_fixed::<32>(&a.effect_digest)?;
    Ok(SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm,
        key_id: KeyId(a.kid.clone()),
        profile: algorithm.profile(),
        policy_epoch: a.policy_epoch,
        revocation_epoch: a.revocation_epoch,
        generation: a.generation,
        nonce,
        not_before: a.not_before,
        expires_at: a.expires,
        subject_digest,
        audience: a.audience.clone(),
    })
}

/// The checked SA2A signing pre-image, EXACTLY the RFC-SA2A-007-errata
/// layout: `SA2A_APPROVAL_TAG || 0x00 || jcs(approval document)`. Deliberately
/// NOT [`crate::crypto_trust_canonical::domain_separated`]: the foreign wire
/// owns its own framing, and the differing bytes ARE the domain separation the
/// falsifier test witnesses. Refuses [`Sa2aWireError::Malformed`] when the
/// document cannot canonicalize — reachable when any integer field exceeds
/// 2^53 (JCS refuses non-I-JSON integers rather than coercing their value).
pub fn sa2a_signing_input_checked(a: &Sa2aApproval) -> Result<Vec<u8>, Sa2aWireError> {
    let document = serde_json::to_value(a).map_err(|e| Sa2aWireError::Malformed(e.to_string()))?;
    let canonical = jcs(&document).map_err(|e| Sa2aWireError::Malformed(e.to_string()))?;
    let mut out = Vec::with_capacity(SA2A_APPROVAL_TAG.len() + 1 + canonical.len());
    out.extend_from_slice(SA2A_APPROVAL_TAG.as_bytes());
    out.push(0x00);
    out.extend_from_slice(canonical.as_bytes());
    Ok(out)
}

/// Infallible form of [`sa2a_signing_input_checked`]. For an approval whose
/// document cannot canonicalize this yields the empty pre-image — a value no
/// SA2A verifier will accept (same law as
/// [`crate::crypto_trust_envelope::SignatureEnvelope::signing_input`]).
pub fn sa2a_signing_input(a: &Sa2aApproval) -> Vec<u8> {
    sa2a_signing_input_checked(a).unwrap_or_default()
}

/// The SA2A pre-image for an affidavit envelope: bridge, then frame. Refuses
/// [`Sa2aWireError::WrongVersion`] when `env.version` is not
/// [`ENVELOPE_VERSION`]; otherwise [`sa2a_signing_input_checked`] applies.
pub fn envelope_sa2a_signing_input(
    env: &SignatureEnvelope,
    principal: &str,
) -> Result<Vec<u8>, Sa2aWireError> {
    let approval = envelope_to_approval(env, principal)?;
    sa2a_signing_input_checked(&approval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};

    const PRINCIPAL: &str = "subject-a";

    /// A graph-lawful envelope under every admitted algorithm (profile = the
    /// algorithm's graph profile, version pinned).
    fn test_envelope(algorithm: AlgorithmId) -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm,
            key_id: KeyId("k-test-1".to_string()),
            profile: algorithm.profile(),
            policy_epoch: 7,
            revocation_epoch: 3,
            generation: 2,
            nonce: [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
                0x0f, 0x10,
            ],
            not_before: 1_000,
            expires_at: 2_000,
            subject_digest: [0x22; 32],
            audience: "affidavit.verifier".to_string(),
        }
    }

    /// The pinned errata example approval — the exact document behind the
    /// byte-exactness vector below.
    fn pinned_approval() -> Sa2aApproval {
        Sa2aApproval {
            v: "1".to_string(),
            alg: "ES256".to_string(),
            kid: "afk1_0123456789abcdef".to_string(),
            effect_digest: "abcd".repeat(16),
            principal: PRINCIPAL.to_string(),
            policy_epoch: 7,
            revocation_epoch: 3,
            generation: 2,
            nonce: "0102030405060708090a0b0c0d0e0f10".to_string(),
            not_before: 1_000,
            expires: 2_000,
            audience: "affidavit.verifier".to_string(),
        }
    }

    /// The RFC-SA2A-007-errata errata example: a fixed approval, the exact
    /// `sa2a_signing_input` bytes as a hex literal —
    /// `"SA2A-C2-APPROVAL-v1" || 0x00 || JCS({...})` with the twelve approval
    /// fields in JCS key order (alg, audience, effect_digest, expires,
    /// generation, kid, nonce, not_before, policy_epoch, principal,
    /// revocation_epoch, v). A graph-side tag change or a template-side
    /// framing change breaks this loudly instead of drifting silently.
    #[test]
    fn jcs_errata_example_is_byte_exact() {
        let input = sa2a_signing_input(&pinned_approval());
        assert_eq!(input.len(), 341);
        let expected_hex = "534132412d43322d415050524f56414c2d7631007b22616c67223a224553323536222c2261756469656e6365223a226166666964617669742e7665726966696572222c226566666563745f646967657374223a2261626364616263646162636461626364616263646162636461626364616263646162636461626364616263646162636461626364616263646162636461626364222c2265787069726573223a323030302c2267656e65726174696f6e223a322c226b6964223a2261666b315f30313233343536373839616263646566222c226e6f6e6365223a223031303230333034303530363037303830393061306230633064306530663130222c226e6f745f6265666f7265223a313030302c22706f6c6963795f65706f6368223a372c227072696e636970616c223a227375626a6563742d61222c227265766f636174696f6e5f65706f6368223a332c2276223a2231227d";
        assert_eq!(hex_encode(&input), expected_hex);
        // The framing is the tag, one NUL, then the JCS document — never
        // domain_separated.
        assert!(input.starts_with(SA2A_APPROVAL_TAG.as_bytes()));
        assert_eq!(input[SA2A_APPROVAL_TAG.len()], 0x00);
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(SA2A_APPROVAL_TAG, "SA2A-C2-APPROVAL-v1");
        assert_eq!(approval_schema_version(), "1");
        assert_eq!(
            ENVELOPE_FIELD_NAMES,
            [
                "version",
                "algorithm",
                "key_id",
                "profile",
                "policy_epoch",
                "revocation_epoch",
                "generation",
                "nonce",
                "not_before",
                "expires_at",
                "subject_digest",
                "audience",
            ]
        );
    }

    #[test]
    fn approval_document_is_exactly_the_errata_field_set() {
        let value = serde_json::to_value(pinned_approval()).expect("approval document serializes");
        let obj = value.as_object().expect("document is an object");
        let mut names: Vec<&str> = obj.keys().map(String::as_str).collect();
        names.sort_unstable();
        // ctp:sa2a-interop-v1 ctp:targetFieldSet, sorted — the derived serde
        // form must carry EXACTLY these names and nothing else.
        assert_eq!(
            names,
            vec![
                "alg",
                "audience",
                "effect_digest",
                "expires",
                "generation",
                "kid",
                "nonce",
                "not_before",
                "policy_epoch",
                "principal",
                "revocation_epoch",
                "v",
            ]
        );
        assert_eq!(obj.len(), 12);
        // And the serde form round-trips.
        let back: Sa2aApproval = serde_json::from_value(value).expect("approval document parses");
        assert_eq!(back, pinned_approval());
    }

    /// Teeth first: the positive bridge paths, then the graph field registry.
    #[test]
    fn envelope_round_trips_to_approval_and_back() {
        for algorithm in AlgorithmId::all() {
            let env = test_envelope(*algorithm);
            let approval = envelope_to_approval(&env, PRINCIPAL).expect("lawful envelope bridges");
            assert_eq!(approval.alg, algorithm.as_str());
            assert_eq!(approval.v, "1");
            let back = approval_to_envelope(&approval).expect("lawful approval bridges back");
            assert_eq!(&back, &env, "round trip must be exact for {algorithm:?}");
            // And the inverse round trip is exact too.
            let approval_again = envelope_to_approval(&back, PRINCIPAL).expect("bridges");
            assert_eq!(approval_again, approval);
        }
    }

    /// Every graph-declared envelope field survives the bridge, name by name:
    /// an unmatched name panics, so a graph-side field addition (or rename)
    /// breaks this test loudly instead of silently dropping a signed field.
    #[test]
    fn every_graph_envelope_field_survives_the_bridge() {
        let env = test_envelope(AlgorithmId::Es256);
        let approval = envelope_to_approval(&env, PRINCIPAL).expect("bridges");
        let back = approval_to_envelope(&approval).expect("bridges back");
        assert_eq!(back, env);
        for name in ENVELOPE_FIELD_NAMES {
            match name {
                "version" => assert_eq!(back.version, env.version),
                "algorithm" => assert_eq!(back.algorithm, env.algorithm),
                "key_id" => assert_eq!(back.key_id, env.key_id),
                "profile" => assert_eq!(back.profile, env.profile),
                "policy_epoch" => assert_eq!(back.policy_epoch, env.policy_epoch),
                "revocation_epoch" => assert_eq!(back.revocation_epoch, env.revocation_epoch),
                "generation" => assert_eq!(back.generation, env.generation),
                "nonce" => assert_eq!(back.nonce, env.nonce),
                "not_before" => assert_eq!(back.not_before, env.not_before),
                "expires_at" => assert_eq!(back.expires_at, env.expires_at),
                "subject_digest" => assert_eq!(back.subject_digest, env.subject_digest),
                "audience" => assert_eq!(back.audience, env.audience),
                other => panic!("graph envelope field `{other}` is not carried by the bridge"),
            }
        }
    }

    #[test]
    fn nonce_and_digest_hex_round_trip() {
        let env = test_envelope(AlgorithmId::Es256);
        let approval = envelope_to_approval(&env, PRINCIPAL).expect("bridges");
        // 16-byte nonce -> 32 lowercase-hex characters; 32-byte digest -> 64.
        assert_eq!(approval.nonce.len(), 32);
        assert_eq!(approval.effect_digest.len(), 64);
        for field in [&approval.nonce, &approval.effect_digest] {
            assert!(
                field
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "wire hex must be lowercase: {field}"
            );
        }
        assert_eq!(hex_encode(&env.nonce), approval.nonce);
        assert_eq!(hex_encode(&env.subject_digest), approval.effect_digest);
        // And the fixed-width decodes restore the exact bytes.
        assert_eq!(
            hex_decode_fixed::<16>(&approval.nonce).expect("nonce decodes"),
            env.nonce
        );
        assert_eq!(
            hex_decode_fixed::<32>(&approval.effect_digest).expect("digest decodes"),
            env.subject_digest
        );
    }

    #[test]
    fn sa2a_signing_input_is_deterministic_and_principal_bound() {
        let a = pinned_approval();
        let b = pinned_approval();
        assert_eq!(sa2a_signing_input(&a), sa2a_signing_input(&b));
        assert_eq!(
            sa2a_signing_input_checked(&a).expect("canonicalizes"),
            sa2a_signing_input(&a)
        );
        // The principal lives inside the signed bytes: changing it changes
        // the pre-image.
        let mut other = pinned_approval();
        other.principal = "subject-b".to_string();
        assert_ne!(sa2a_signing_input(&a), sa2a_signing_input(&other));
    }

    /// FALSIFIER (the interlock tooth): a real ES256 signature over
    /// `SignatureEnvelope::signing_input` does NOT verify over
    /// `sa2a_signing_input`, and vice versa — the two domains are different
    /// byte strings, so cross-domain signatures are refused. Domain
    /// separation is real bytes, not decorative names.
    #[test]
    fn falsifier_cross_domain_signatures_are_refused() {
        let seed = [0x42u8; 32];
        let key = Es256SigningKey::from_seed(&seed).expect("valid P-256 scalar");
        let pk = key.public_key_sec1();
        let env = test_envelope(AlgorithmId::Es256);

        let envelope_input = env.signing_input();
        let sa2a_input =
            envelope_sa2a_signing_input(&env, PRINCIPAL).expect("lawful envelope frames");
        // Different domains -> different bytes.
        assert_ne!(envelope_input, sa2a_input);
        assert!(!sa2a_input.is_empty());

        // An affidavit-domain signature is refused on the SA2A wire ...
        let envelope_signature = key.sign(&envelope_input);
        assert!(
            matches!(
                verify_es256(&pk, &envelope_input, &envelope_signature),
                Ok(true)
            ),
            "in-domain envelope signature must verify first (teeth first)"
        );
        assert!(
            matches!(
                verify_es256(&pk, &sa2a_input, &envelope_signature),
                Ok(false)
            ),
            "envelope-domain signature must NOT verify over the SA2A pre-image"
        );

        // ... and a SA2A-domain signature is refused on the envelope wire.
        let sa2a_signature = key.sign(&sa2a_input);
        assert!(
            matches!(verify_es256(&pk, &sa2a_input, &sa2a_signature), Ok(true)),
            "in-domain SA2A signature must verify"
        );
        assert!(
            matches!(
                verify_es256(&pk, &envelope_input, &sa2a_signature),
                Ok(false)
            ),
            "SA2A-domain signature must NOT verify over the envelope pre-image"
        );

        // The bridge is the lawful path: envelope -> approval -> its own
        // signed pre-image. The approval the bridge yields frames exactly the
        // bytes a SA2A signer signs.
        let approval = envelope_to_approval(&env, PRINCIPAL).expect("bridges");
        assert_eq!(sa2a_signing_input(&approval), sa2a_input);
    }

    #[test]
    fn wrong_version_refused_by_variant() {
        let mut env = test_envelope(AlgorithmId::Es256);
        env.version = "CTP-ENVELOPE-v0".to_string();
        match envelope_to_approval(&env, PRINCIPAL) {
            Err(Sa2aWireError::WrongVersion(v)) => assert_eq!(v, "CTP-ENVELOPE-v0"),
            other => panic!("expected WrongVersion, got {other:?}"),
        }
        // The framing path refuses through the same variant.
        match envelope_sa2a_signing_input(&env, PRINCIPAL) {
            Err(Sa2aWireError::WrongVersion(v)) => assert_eq!(v, "CTP-ENVELOPE-v0"),
            other => panic!("expected WrongVersion, got {other:?}"),
        }
    }

    #[test]
    fn unknown_algorithm_refused_by_variant() {
        let mut approval = pinned_approval();
        approval.alg = "RS256".to_string();
        match approval_to_envelope(&approval) {
            Err(Sa2aWireError::UnknownAlgorithm(alg)) => assert_eq!(alg, "RS256"),
            other => panic!("expected UnknownAlgorithm, got {other:?}"),
        }
        // Empty is not a graph-declared algorithm name either.
        let mut empty = pinned_approval();
        empty.alg = String::new();
        assert!(matches!(
            approval_to_envelope(&empty),
            Err(Sa2aWireError::UnknownAlgorithm(_))
        ));
    }

    #[test]
    fn malformed_hex_refused_by_variant() {
        // Non-hex digits (even length).
        let mut approval = pinned_approval();
        approval.nonce = "zz".repeat(16);
        match approval_to_envelope(&approval) {
            Err(Sa2aWireError::MalformedHex(s)) => assert_eq!(s, approval.nonce),
            other => panic!("expected MalformedHex, got {other:?}"),
        }
        // Odd length.
        let mut odd = pinned_approval();
        odd.nonce = "abc".to_string();
        assert!(matches!(
            approval_to_envelope(&odd),
            Err(Sa2aWireError::MalformedHex(_))
        ));
        // Uppercase is refused, not coerced: the wire alphabet is lowercase.
        let mut upper = pinned_approval();
        upper.effect_digest = "ABCD".repeat(16);
        assert!(matches!(
            approval_to_envelope(&upper),
            Err(Sa2aWireError::MalformedHex(_))
        ));
    }

    #[test]
    fn malformed_length_refused_by_variant() {
        // Valid lowercase hex, wrong byte count for the fixed-width field.
        let mut short_nonce = pinned_approval();
        short_nonce.nonce = "aabb".to_string(); // 2 bytes, not 16
        match approval_to_envelope(&short_nonce) {
            Err(Sa2aWireError::Malformed(msg)) => assert!(msg.contains("32 characters")),
            other => panic!("expected Malformed, got {other:?}"),
        }
        let mut short_digest = pinned_approval();
        short_digest.effect_digest = "ab".repeat(30); // 30 bytes, not 32
        assert!(matches!(
            approval_to_envelope(&short_digest),
            Err(Sa2aWireError::Malformed(_))
        ));
    }

    #[test]
    fn non_canonical_integer_refused_malformed() {
        let mut beyond_double = pinned_approval();
        beyond_double.policy_epoch = 9_007_199_254_740_993; // 2^53 + 1
        match sa2a_signing_input_checked(&beyond_double) {
            Err(Sa2aWireError::Malformed(_)) => {}
            other => panic!("expected Malformed for non-I-JSON epoch, got {other:?}"),
        }
        // The infallible form refuses with the empty pre-image — never wrong
        // bytes.
        assert!(sa2a_signing_input(&beyond_double).is_empty());
        // Boundary: exactly 2^53 is exactly representable and frames fine.
        let mut at_boundary = pinned_approval();
        at_boundary.policy_epoch = 9_007_199_254_740_992;
        assert!(sa2a_signing_input_checked(&at_boundary).is_ok());
    }
}
