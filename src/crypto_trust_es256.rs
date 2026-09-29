//! ES256 (P-256, RFC 6979 deterministic ECDSA) signing provider for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! The signing provider is certify-don't-decide: it signs and verifies exact
//! byte strings; it never confers authority or judges admissibility. Domain
//! separation happens one layer up (the envelope canonicalizes under
//! [`DOMAIN_TAG`] before handing bytes here), so this module signs exactly what
//! it is given — pinned by the RFC 6979 A.2.5 known-answer vector below.
//!
//! Determinism law (from the graph, `ctp:alg-ES256`): signatures are RFC 6979
//! deterministic ECDSA over P-256 with SHA-256 — the same message bytes under
//! the same key always yield the same DER signature, no RNG on the signing
//! path. `from_seed` is unrelated to RFC 6979: it is key derivation that makes
//! the KEY deterministic (for reproducible test fixtures); the nonce law is
//! RFC 6979 itself and is witnessed by the A.2.5 vector.
//!
//! House law: refusals are typed values ([`Es256Error`]), never panics; no
//! `unwrap`/`expect` on production paths.

// Consumed query columns (es256.rq): alg_name, spec_ref, pk_len, deterministic, domain_tag.

use p256::ecdsa::{DerSignature, Signature, SigningKey, VerifyingKey};
use signature::{Signer, Verifier};

use crate::crypto_trust_keys::{fingerprint_public_key, AlgorithmId, PublicKeyMaterial};

/// Algorithm identity (`ctp:alg-ES256` `ctp:algorithmName`).
pub const ALGORITHM_NAME: &str = "ES256";
/// Authoritative parameter-set reference (`ctp:parameterSet` `ctp:specRef`).
pub const SPEC_REF: &str = "RFC 6979 / SEC 2";
/// Graph-declared public-key length in bytes: the SEC1 uncompressed point
/// `0x04 || X || Y` (`ctp:publicKeyLength`).
pub const PUBLIC_KEY_LENGTH: usize = 65;
/// Whether signing is deterministic per the graph (`ctp:deterministic`):
/// RFC 6979 nonces make signatures reproducible for a fixed key and message.
pub const DETERMINISTIC: bool = true;
/// Trust-plane domain tag (`ctp:policy-v1` `ctp:domainTag`). Applied by the
/// envelope layer before signing; carried here so the provider can state the
/// domain it serves without deciding anything about it.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// A P-256 signing key held in process memory (provider kind `SOFTWARE` per
/// the graph: key material exportable). For non-exportable custody see
/// `crate::crypto_trust_enclave`.
pub struct Es256SigningKey {
    secret: SigningKey,
}

impl Es256SigningKey {
    /// Generates a cryptographically random key from the OS entropy source.
    pub fn generate() -> Result<Self, Es256Error> {
        let secret = SigningKey::random(&mut rand_core::OsRng);
        Ok(Es256SigningKey { secret })
    }

    /// Derives a key from a 32-byte big-endian scalar seed.
    ///
    /// This is KEY derivation for deterministic (test/fixture) keys, not the
    /// RFC 6979 nonce law. The seed must decode to a valid non-zero P-256
    /// scalar; anything else is refused with [`Es256Error::P256`].
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self, Es256Error> {
        let secret = SigningKey::from_slice(seed).map_err(|_| {
            Es256Error::P256(String::from("seed is not a valid non-zero P-256 scalar"))
        })?;
        Ok(Es256SigningKey { secret })
    }

    /// The public key in SEC1 uncompressed encoding (`0x04 || X || Y`, 65
    /// bytes — [`PUBLIC_KEY_LENGTH`]).
    pub fn public_key_sec1(&self) -> Vec<u8> {
        self.secret
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec()
    }

    /// Signs a message with RFC 6979 deterministic ECDSA (P-256, SHA-256) and
    /// returns the DER (X9.62 `ECDSA-Sig-Value`) encoding. Deterministic: the
    /// same key and message bytes always produce the same signature.
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let signature: Signature = self.secret.sign(msg);
        signature.to_der().as_bytes().to_vec()
    }

    /// The trust-plane fingerprint of this key's public material, bound to
    /// [`AlgorithmId::Es256`] and the domain tag by the key registry.
    pub fn key_id_fingerprint(&self) -> crate::crypto_trust_keys::KeyFingerprint {
        fingerprint_public_key(
            AlgorithmId::Es256,
            &PublicKeyMaterial::Es256Sec1(self.public_key_sec1()),
        )
    }
}

/// Verifies an ES256 signature (DER-encoded, as produced by
/// [`Es256SigningKey::sign`]) over `msg` against a SEC1 uncompressed public
/// key. Returns `Ok(false)` for a well-formed signature that does not verify;
/// malformed inputs are typed refusals.
pub fn verify_es256(
    public_key_sec1: &[u8],
    msg: &[u8],
    sig_der: &[u8],
) -> Result<bool, Es256Error> {
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key_sec1)
        .map_err(|_| Es256Error::MalformedPublicKey)?;
    let der = DerSignature::from_bytes(sig_der).map_err(|_| Es256Error::MalformedSignature)?;
    let signature = Signature::try_from(der).map_err(|_| Es256Error::MalformedSignature)?;
    Ok(verifying_key.verify(msg, &signature).is_ok())
}

/// Typed refusals of the ES256 provider. Values, never panics.
#[derive(Debug, thiserror::Error)]
pub enum Es256Error {
    /// A failure reported by the p256 layer (e.g. a seed that is not a valid
    /// non-zero P-256 scalar).
    #[error("p256: {0}")]
    P256(String),
    /// The public key bytes are not a valid SEC1 P-256 point encoding.
    #[error("malformed public key")]
    MalformedPublicKey,
    /// The signature bytes are not a valid DER ECDSA-Sig-Value.
    #[error("malformed signature")]
    MalformedSignature,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 6979 A.2.5 (P-256, SHA-256), message "sample":
    /// private scalar x, nonce k, and the expected signature components r, s.
    const RFC6979_X: &str = "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721";
    #[allow(dead_code)]
    const RFC6979_K: &str = "A6E3C57DD01ABE90086538398355DD4C3B17AA873382B0F24D6129493D8AAD60";
    const RFC6979_R: &str = "EFD48B2AACB6A8FD1140DD9CD45E81D69D2C877B56AAF991C34D0EA84EAF3716";
    const RFC6979_S: &str = "F7CB1C942D657C41D436C7A1B6E29F65F3E900DBB9AFF4064DC4AB2F843ACDA8";
    const RFC6979_MESSAGE: &[u8] = b"sample";

    fn hex_decode(hex: &str) -> Vec<u8> {
        assert_eq!(hex.len() % 2, 0, "hex must be byte-aligned");
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex digit"))
            .collect()
    }

    /// Splits a DER ECDSA-Sig-Value into its (r, s) INTEGER contents.
    /// Structure assumed: SEQUENCE { INTEGER r, INTEGER s } with short-form
    /// lengths — exactly what [`super::Es256SigningKey::sign`] emits.
    fn der_split_rs(der: &[u8]) -> (&[u8], &[u8]) {
        assert_eq!(der[0], 0x30, "DER must open with SEQUENCE");
        let seq_len = usize::from(der[1]);
        assert_eq!(
            der.len(),
            2 + seq_len,
            "SEQUENCE length must cover the rest"
        );
        assert_eq!(der[2], 0x02, "r must be an INTEGER");
        let r_len = usize::from(der[3]);
        let r = &der[4..4 + r_len];
        let after_r = 4 + r_len;
        assert_eq!(der[after_r], 0x02, "s must be an INTEGER");
        let s_len = usize::from(der[after_r + 1]);
        let s = &der[after_r + 2..after_r + 2 + s_len];
        assert_eq!(after_r + 2 + s_len, der.len(), "no trailing bytes");
        (r, s)
    }

    /// DER INTEGER content for an unsigned big-endian scalar: a 0x00 sign
    /// byte is prepended when the leading byte has its high bit set (this
    /// vector's r and s both do), per X.690.
    fn der_integer_content(scalar: &[u8]) -> Vec<u8> {
        let mut content = Vec::with_capacity(scalar.len() + 1);
        if scalar[0] & 0x80 != 0 {
            content.push(0x00);
        }
        content.extend_from_slice(scalar);
        content
    }

    /// Full DER frame for SEQUENCE { INTEGER r, INTEGER s } (short form).
    fn der_frame(r: &[u8], s: &[u8]) -> Vec<u8> {
        let r_content = der_integer_content(r);
        let s_content = der_integer_content(s);
        let seq_len = 2 + r_content.len() + 2 + s_content.len();
        let mut out = Vec::with_capacity(2 + seq_len);
        out.extend_from_slice(&[0x30, seq_len as u8]);
        out.extend_from_slice(&[0x02, r_content.len() as u8]);
        out.extend_from_slice(&r_content);
        out.extend_from_slice(&[0x02, s_content.len() as u8]);
        out.extend_from_slice(&s_content);
        out
    }

    /// Normalizes an INTEGER content to the raw 32-byte scalar: strips DER
    /// sign bytes, then left-pads to 32 bytes for comparison with the vector.
    fn to_scalar32(bytes: &[u8]) -> [u8; 32] {
        let stripped = {
            let mut b = bytes;
            while b.len() > 1 && b[0] == 0x00 {
                b = &b[1..];
            }
            b
        };
        assert!(stripped.len() <= 32, "scalar must fit 32 bytes");
        let mut out = [0u8; 32];
        out[32 - stripped.len()..].copy_from_slice(stripped);
        out
    }

    /// The raw 32-byte big-endian scalar for a hex vector string.
    fn scalar32_from_hex(hex: &str) -> [u8; 32] {
        let bytes = hex_decode(hex);
        assert_eq!(bytes.len(), 32);
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        out
    }

    fn rfc6979_key() -> Es256SigningKey {
        Es256SigningKey::from_seed(&scalar32_from_hex(RFC6979_X)).expect("valid RFC 6979 scalar")
    }

    #[test]
    fn graph_facts_are_rendered() {
        assert_eq!(ALGORITHM_NAME, "ES256");
        assert_eq!(SPEC_REF, "RFC 6979 / SEC 2");
        assert_eq!(PUBLIC_KEY_LENGTH, 65);
        assert_eq!(DETERMINISTIC.to_string(), "true");
        assert!(!DOMAIN_TAG.is_empty());
    }

    /// RFC 6979 A.2.5 known-answer: the deterministic nonce law is witnessed
    /// by the exact (r, s) the signer must emit for the sample key/message.
    #[test]
    fn rfc6979_a_2_5_known_answer_vector() {
        let key = rfc6979_key();
        let der = key.sign(RFC6979_MESSAGE);

        let r = hex_decode(RFC6979_R);
        let s = hex_decode(RFC6979_S);

        // Full-DER equality against the X.690 frame computed from the vector:
        // r (0xEF..) and s (0xF7..) both carry the high bit, so DER signed
        // INTEGER encoding pads each with a 0x00 sign byte (0x02 0x21 00 ...),
        // giving a 72-byte SEQUENCE — 0x30 0x46.
        let expected = der_frame(&r, &s);
        assert_eq!(expected[0], 0x30);
        assert_eq!(usize::from(expected[1]), 0x46);
        assert_eq!(der, expected, "full DER must equal the RFC 6979 vector");

        // Component-wise authority: parse the produced DER and compare r, s.
        let (got_r, got_s) = der_split_rs(&der);
        assert_eq!(to_scalar32(got_r), scalar32_from_hex(RFC6979_R));
        assert_eq!(to_scalar32(got_s), scalar32_from_hex(RFC6979_S));
    }

    /// Teeth before tamper: a valid signature verifies true FIRST, then every
    /// tamper is refused.
    #[test]
    fn verify_round_trip_then_message_tamper_is_false() {
        let key = rfc6979_key();
        let pk = key.public_key_sec1();
        let sig = key.sign(RFC6979_MESSAGE);

        // Valid first.
        assert!(
            matches!(verify_es256(&pk, RFC6979_MESSAGE, &sig), Ok(true)),
            "untampered round trip must verify"
        );

        // Flip one message byte -> false.
        let mut tampered_msg = RFC6979_MESSAGE.to_vec();
        tampered_msg[0] ^= 0x01;
        assert!(
            matches!(verify_es256(&pk, &tampered_msg, &sig), Ok(false)),
            "flipped message byte must not verify"
        );
    }

    #[test]
    fn tampered_signature_bytes_are_refused() {
        let key = rfc6979_key();
        let pk = key.public_key_sec1();
        let sig = key.sign(RFC6979_MESSAGE);

        // Flip the SEQUENCE tag byte -> DER structure refused, typed value.
        let mut broken_structure = sig.clone();
        broken_structure[0] ^= 0xFF;
        assert!(matches!(
            verify_es256(&pk, RFC6979_MESSAGE, &broken_structure),
            Err(Es256Error::MalformedSignature)
        ));

        // Flip the last scalar byte -> well-formed DER that cannot verify.
        let mut broken_scalar = sig.clone();
        let last = broken_scalar.len() - 1;
        broken_scalar[last] ^= 0x01;
        assert!(
            matches!(
                verify_es256(&pk, RFC6979_MESSAGE, &broken_scalar),
                Ok(false)
            ),
            "flipped signature scalar must not verify"
        );
    }

    #[test]
    fn wrong_key_does_not_verify() {
        let signer = rfc6979_key();
        let other = {
            let mut seed = scalar32_from_hex(RFC6979_X);
            seed[0] ^= 0x01;
            Es256SigningKey::from_seed(&seed).expect("valid other scalar")
        };
        let sig = signer.sign(RFC6979_MESSAGE);
        assert!(
            matches!(
                verify_es256(&other.public_key_sec1(), RFC6979_MESSAGE, &sig),
                Ok(false)
            ),
            "a signature must not verify under a different key"
        );
    }

    #[test]
    fn generate_produces_distinct_keys() {
        let a = Es256SigningKey::generate().expect("generate");
        let b = Es256SigningKey::generate().expect("generate");
        assert_ne!(a.public_key_sec1(), b.public_key_sec1());
    }

    #[test]
    fn from_seed_is_deterministic_and_fingerprint_stable() {
        let seed = scalar32_from_hex(RFC6979_X);
        let a = Es256SigningKey::from_seed(&seed).expect("seed key");
        let b = Es256SigningKey::from_seed(&seed).expect("seed key");
        assert_eq!(a.public_key_sec1(), b.public_key_sec1());
        assert_eq!(a.sign(RFC6979_MESSAGE), b.sign(RFC6979_MESSAGE));
        assert_eq!(a.key_id_fingerprint(), b.key_id_fingerprint());

        let mut other_seed = seed;
        other_seed[31] ^= 0x01;
        let c = Es256SigningKey::from_seed(&other_seed).expect("seed key");
        assert_ne!(a.public_key_sec1(), c.public_key_sec1());
        assert_ne!(a.key_id_fingerprint(), c.key_id_fingerprint());
    }

    #[test]
    fn public_key_is_uncompressed_sec1_at_declared_length() {
        let key = Es256SigningKey::generate().expect("generate");
        let pk = key.public_key_sec1();
        assert_eq!(pk.len(), PUBLIC_KEY_LENGTH);
        assert_eq!(pk[0], 0x04, "SEC1 uncompressed point marker");
    }

    #[test]
    fn malformed_inputs_are_typed_refusals() {
        // Public key too short to be a P-256 point.
        assert!(matches!(
            verify_es256(&[0x04, 0x01], RFC6979_MESSAGE, &[0x30, 0x00]),
            Err(Es256Error::MalformedPublicKey)
        ));
        // Empty DER.
        let key = rfc6979_key();
        assert!(matches!(
            verify_es256(&key.public_key_sec1(), RFC6979_MESSAGE, &[]),
            Err(Es256Error::MalformedSignature)
        ));
    }

    #[test]
    fn zero_seed_is_refused_as_value() {
        let result = Es256SigningKey::from_seed(&[0u8; 32]);
        assert!(
            matches!(result, Err(Es256Error::P256(_))),
            "the zero scalar is not a valid key"
        );
    }
}
