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
//! Canonical-signature law (malleability closure, BIP-62 semantics): for one
//! mathematical signature, `(r, s)` and its mirror `(r, n − s)` are BOTH valid
//! ECDSA over the same key and message — `n` is the P-256 group order
//! `0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551` (SEC 2
//! v2 §2.4.2; p256's own `Scalar::MODULUS`). A verifier that admits both lets
//! a flipped-sign re-present double through any signature-keyed surface
//! (dedupe, replay, revocation, transparency log). The plane therefore closes
//! the malleability from both ends: the verification path REFUSES high-s
//! (`s > n/2`) with [`Es256Error::MalformedSignature`], and the signing path
//! emits only the canonical low-s member. The RFC 6979 NONCE law is untouched:
//! `k` is unchanged, and `r` is invariant under `s → n − s`, so the A.2.5
//! known-answer vector still pins the nonce exactly (its literal `s` is
//! high-s; the canonical form is asserted below).
//!
//! House law: refusals are typed values ([`Es256Error`]), never panics; no
//! `unwrap`/`expect` on production paths.

// Consumed query columns (es256.rq): alg_name, spec_ref, pk_len, deterministic, domain_tag.

use p256::ecdsa::{DerSignature, Signature, SigningKey, VerifyingKey};
use p256::elliptic_curve::scalar::IsHigh;
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
    ///
    /// The nonce `k` is the RFC 6979 deterministic nonce — unchanged. The
    /// emitted signature is additionally canonicalized to the low-s member of
    /// its malleability class (`s' = n − s` when `s > n/2`, via
    /// [`Signature::normalize_s`]): the emitter side of the canonical-form
    /// law that [`verify_es256`] enforces on the verifier side. The RFC 6979
    /// A.2.5 vector's literal `s` is high-s; the KAT below pins the nonce law
    /// via the invariant `r` and the canonical-form law via the asserted
    /// mirror.
    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let signature: Signature = self.secret.sign(msg);
        // Emitter side of the canonical-signature law: keep the RFC 6979
        // nonce, emit only the low-s member. `normalize_s` returns `None`
        // when `s` is already low, in which case the signature stands.
        let canonical = signature.normalize_s().unwrap_or(signature);
        canonical.to_der().as_bytes().to_vec()
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
/// key. Returns `Ok(false)` for a well-formed low-s signature that does not
/// verify; malformed inputs are typed refusals — including a well-formed DER
/// signature carrying high-s (`s > n/2`), refused by the canonical-form law
/// (see below) even though its ECDSA arithmetic is valid.
pub fn verify_es256(
    public_key_sec1: &[u8],
    msg: &[u8],
    sig_der: &[u8],
) -> Result<bool, Es256Error> {
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key_sec1)
        .map_err(|_| Es256Error::MalformedPublicKey)?;
    let der = DerSignature::from_bytes(sig_der).map_err(|_| Es256Error::MalformedSignature)?;
    let signature = Signature::try_from(der).map_err(|_| Es256Error::MalformedSignature)?;
    // Canonical-form law (BIP-62 verifier side): refuse high-s.
    //
    // WHY: (r, s) and (r, n − s) are both valid ECDSA over the same key and
    // message, so an admissible high-s signature gives every re-presentation
    // a second, distinct, still-valid byte string — a flipped-sign re-present
    // could double through any signature-keyed surface (dedupe, replay,
    // revocation, transparency log). n is the P-256 group order
    // 0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551
    // (SEC 2 v2 §2.4.2), and the comparison is p256's own constant-time
    // `Scalar::is_high()` (`ct_gt` against `n/2`) — the same predicate
    // `Signature::normalize_s` uses. The in-tree order constant is
    // cross-checked against p256's modulus in the test
    // `hardcoded_order_matches_p256_modulus`. The plane's own signer only
    // emits the low-s member ([`Es256SigningKey::sign`]), so this refusal
    // never fires on the plane's own round trip.
    if bool::from(signature.s().is_high()) {
        return Err(Es256Error::MalformedSignature);
    }
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
    /// The signature bytes are not a valid DER ECDSA-Sig-Value — or they are
    /// well-formed DER carrying high-s (`s > n/2`, reason `"high-s"`), which
    /// the canonical-form law refuses because `(r, s)` and `(r, n − s)` are
    /// both valid ECDSA and the plane admits only the low-s member. The
    /// plane's own signer always emits low-s, so this variant cannot hit an
    /// internal round trip.
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

    /// P-256 group order n (SEC 2 v2 §2.4.2). The canonical-form law compares
    /// against `n/2`; this constant is cross-checked byte-for-byte against
    /// p256's own group modulus in
    /// [`hardcoded_order_matches_p256_modulus`], so the law's `n` is the
    /// curve's, never a transcription of a comment.
    const P256_ORDER_HEX: &str =
        "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551";

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

    /// The group order `n` as a 32-byte big-endian scalar.
    fn p256_order_bytes() -> [u8; 32] {
        scalar32_from_hex(P256_ORDER_HEX)
    }

    /// Big-endian `n − s` for `0 < s < n` (grade-school borrow propagation
    /// over the 32 bytes). This is the malleability mirror: `(r, s)` and
    /// `(r, n − s)` are the same mathematical signature.
    fn scalar_neg_mod_n(scalar: &[u8; 32], n: &[u8; 32]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let mut borrow = 0i16;
        for i in (0..32).rev() {
            let diff = i16::from(n[i]) - i16::from(scalar[i]) - borrow;
            if diff < 0 {
                out[i] = (diff + 256) as u8;
                borrow = 1;
            } else {
                out[i] = diff as u8;
                borrow = 0;
            }
        }
        assert_eq!(borrow, 0, "s must be < n for n − s");
        out
    }

    /// `s > n/2` by big-endian byte comparison against `n >> 1` — the test-side
    /// twin of p256's constant-time `Scalar::is_high` used by the production
    /// law, so the tests judge the law with visible arithmetic, not with the
    /// same black box it exercises.
    fn is_high_s(scalar: &[u8; 32]) -> bool {
        let n = p256_order_bytes();
        let mut half = [0u8; 32];
        let mut carry = 0u8;
        for i in 0..32 {
            half[i] = (n[i] >> 1) | (carry << 7);
            carry = n[i] & 1;
        }
        scalar[..] > half[..]
    }

    #[test]
    fn graph_facts_are_rendered() {
        assert_eq!(ALGORITHM_NAME, "ES256");
        assert_eq!(SPEC_REF, "RFC 6979 / SEC 2");
        assert_eq!(PUBLIC_KEY_LENGTH, 65);
        assert_eq!(DETERMINISTIC.to_string(), "true");
        assert!(!DOMAIN_TAG.is_empty());
    }

    /// RFC 6979 A.2.5 known-answer, pinned under the canonical-signature law.
    ///
    /// Premise witness: the RFC's literal `s` = `F7CB1C94…` is HIGH-s
    /// (`s > n/2`) — computed independently and asserted here. The vector
    /// therefore pins BOTH laws at once:
    ///
    /// * nonce law — `r` must stay byte-identical to the RFC vector (`r` is
    ///   invariant under `s → n − s`, so a matching `r` proves the deterministic
    ///   `k` is the RFC 6979 nonce, untouched by canonicalization);
    /// * canonical-form law — the emitted `s` must be the low-s mirror
    ///   `n − s` of the RFC literal, and verify low.
    #[test]
    fn rfc6979_a_2_5_known_answer_vector() {
        let key = rfc6979_key();
        let der = key.sign(RFC6979_MESSAGE);

        let r = hex_decode(RFC6979_R);
        let s_rfc = scalar32_from_hex(RFC6979_S);
        assert!(
            is_high_s(&s_rfc),
            "premise: the RFC 6979 A.2.5 literal s must be high-s"
        );
        let s_canonical = scalar_neg_mod_n(&s_rfc, &p256_order_bytes());
        assert!(
            !is_high_s(&s_canonical),
            "the n − s mirror of a high-s signature must be low-s"
        );

        // Full-DER equality against the X.690 frame of (r, n − s): r keeps its
        // 0x00 sign pad (0xEF..), the canonical s starts 0x08 (high bit clear,
        // no pad), so the SEQUENCE carries 33 + 32 content bytes — 0x30 0x45.
        let expected = der_frame(&r, &s_canonical);
        assert_eq!(expected[0], 0x30);
        assert_eq!(usize::from(expected[1]), 0x45);
        assert_eq!(
            der, expected,
            "full DER must equal the canonical (r, n − s) of the RFC 6979 vector"
        );

        // Component-wise authority: parse the produced DER and compare.
        let (got_r, got_s) = der_split_rs(&der);
        assert_eq!(to_scalar32(got_r), scalar32_from_hex(RFC6979_R));
        assert_eq!(to_scalar32(got_s), s_canonical);
        assert!(!is_high_s(&to_scalar32(got_s)), "emitted s must be low-s");

        // The canonical signature must itself verify: signer and verifier now
        // speak the same canonical form.
        assert!(
            matches!(
                verify_es256(&key.public_key_sec1(), RFC6979_MESSAGE, &der),
                Ok(true)
            ),
            "the canonical A.2.5 signature must round-trip through the verifier"
        );
    }

    /// The in-tree order constant is p256's own group modulus — not a
    /// transcription that can drift from the arithmetic the law runs on.
    #[test]
    fn hardcoded_order_matches_p256_modulus() {
        use p256::elliptic_curve::PrimeField;
        assert_eq!(
            <p256::Scalar as PrimeField>::MODULUS.to_ascii_uppercase(),
            P256_ORDER_HEX,
            "P256_ORDER_HEX must be the curve's own group order"
        );
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

    /// Malleability falsifier: the `s → n − s` mirror of a VALID signature,
    /// framed as well-formed DER, must be REFUSED as
    /// [`Es256Error::MalformedSignature`] — the malleability is closed, no
    /// flipped-sign re-present can double through the verifier.
    #[test]
    fn high_s_mirror_of_valid_signature_is_refused() {
        let key = rfc6979_key();
        let pk = key.public_key_sec1();
        let sig = key.sign(RFC6979_MESSAGE);

        // Teeth first: the canonical signature verifies true.
        assert!(
            matches!(verify_es256(&pk, RFC6979_MESSAGE, &sig), Ok(true)),
            "canonical signature must verify before the mirror is tried"
        );

        // Mirror construction: same r, s' = n − s, valid DER framing.
        let (r, s) = der_split_rs(&sig);
        let s_low = to_scalar32(s);
        assert!(!is_high_s(&s_low), "the signer must emit low-s");
        let s_mirror = scalar_neg_mod_n(&s_low, &p256_order_bytes());
        assert!(is_high_s(&s_mirror), "the n − s mirror of low-s must be high-s");
        let mirror_der = der_frame(&to_scalar32(r), &s_mirror);

        // Control: the identical framing with the original low-s s verifies,
        // proving the refusal below is the high-s law, not the frame.
        let reframed_original = der_frame(&to_scalar32(r), &s_low);
        assert!(
            matches!(
                verify_es256(&pk, RFC6979_MESSAGE, &reframed_original),
                Ok(true)
            ),
            "the same frame with low-s must verify — only the mirror may refuse"
        );

        // The mirror is refused as malformed even though its ECDSA arithmetic
        // is valid: exactly the malleability class the plane closes.
        assert!(
            matches!(
                verify_es256(&pk, RFC6979_MESSAGE, &mirror_der),
                Err(Es256Error::MalformedSignature)
            ),
            "the (r, n − s) mirror must be refused as MalformedSignature"
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
