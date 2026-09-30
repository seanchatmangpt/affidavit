//! The signature-envelope law for consumers that verify **without crypto math**:
//! the twelve-field envelope document model, byte-level JSON decode, the
//! JCS-subset canonical bytes, and the domain-separated signing pre-image.
//!
//! This is a zero-dependency, `no_std` port of the rendered plane's
//! `crypto_trust_envelope` + `crypto_trust_canonical` law (RFC-SA2A-007-errata
//! E-E) over this crate's own discipline. What it deliberately does **not**
//! contain: any EC or ML-DSA arithmetic. `affidavit-core` has no dependencies
//! at all, so the signature *verifier* stays external — a consumer decodes an
//! envelope here, recomputes the exact bytes that must be signed, and hands
//! those bytes (plus the signature) to its own verifier. The law this module
//! guarantees is *which bytes are bound*, not whether a signature over them is
//! valid.
//!
//! ## The envelope law (ported verbatim from the rendered plane)
//!
//! The algorithm identity, key id, nonce, and the policy/revocation epochs live
//! **inside** the signed bytes — a signature over bytes that omit them would
//! not bind them. The signing pre-image is exactly:
//!
//! ```text
//! domain_separated(DOMAIN_TAG, [jcs(envelope_document)])
//!   = DOMAIN_TAG || 0x00 || DOMAIN_TAG || 0x00
//!     || u64_be(len(jcs_document)) || jcs_document
//! ```
//!
//! The envelope document is the twelve graph-declared fields
//! ([`ENVELOPE_FIELDS`]); its canonical form is the JCS (RFC 8785) subset this
//! module implements: the document's values are strings, unsigned integers and
//! arrays of small integers, so the subset is exact for every representable
//! envelope — string escaping per RFC 8785 §3.2.2.2 (minimal, lowercase hex),
//! keys in UTF-16 code-unit order (§3.2.3 — for the all-ASCII field registry
//! this is the fixed order the canonical writer emits), integers as plain
//! decimal, with the rendered plane's typed refusal of integer literals beyond
//! 2^53 (not I-JSON; JCS numbers are IEEE-754 doubles).
//!
//! ## Discipline
//!
//! * The decode + recompute path is borrowed and allocation-free
//!   ([`EnvelopeRef`]) — usable in a WASM sandbox, an HSM, or an on-chain
//!   runtime. The owned [`SignatureEnvelope`] (escape-decoding decoder, `Vec`
//!   outputs) lives behind the `alloc` feature, exactly like
//!   [`crate::chain::ChainBuilder`].
//! * Refusals are typed values ([`EnvelopeError`]); nothing here panics,
//!   coerces, or fakes.
//! * The no-alloc decoder refuses escape sequences in string fields (it hands
//!   back zero-copy slices of the input); the `alloc` decoder decodes them.
//!   Both are total over their documented domain and tested to agree.
//!
//! ## Doctrine, preserved
//!
//! *Certify, don't decide.* This module proves which bytes a signature must
//! cover. It never claims a signature is valid — that judgment belongs to the
//! external verifier that holds the public key.

use core::fmt;

/// Signed-envelope version (rendered `ctp:policy-v1 ctp:envelopeVersion`).
/// Decode refuses any document whose `version` field differs, typed
/// [`EnvelopeError::WrongVersion`].
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// Domain-separation tag mixed into every signing pre-image (rendered
/// `ctp:policy-v1 ctp:domainTag`, shared with the rendered plane's
/// `crypto_trust_canonical` module).
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// The signed-bytes field registry, mirroring the graph's
/// ctp:SignatureEnvelopeField individuals as (ctp:fieldName, ctp:fieldOrder).
/// The field-set tests hold the document model and the canonical writer to
/// this registry, so a graph-side field change breaks the port loudly.
pub const ENVELOPE_FIELDS: [(&str, u32); 12] = [
    ("version", 1),
    ("algorithm", 2),
    ("key_id", 3),
    ("profile", 4),
    ("policy_epoch", 5),
    ("revocation_epoch", 6),
    ("generation", 7),
    ("nonce", 8),
    ("not_before", 9),
    ("expires_at", 10),
    ("subject_digest", 11),
    ("audience", 12),
];

/// Largest integer exactly representable as an IEEE-754 double: 2^53. JCS
/// numbers are doubles; integer literals beyond this are not I-JSON and are
/// refused ([`EnvelopeError::NonCanonicalNumber`]) instead of silently
/// coercing their value.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_992;

/// Bound on nesting of *unknown* fields skipped during decode, so a hostile
/// document cannot walk the skipper unboundedly. Mirrors serde_json's own
/// recursion limit.
const MAX_SKIP_DEPTH: usize = 128;

/// Typed refusal of an envelope operation; refusal-as-value, never a panic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvelopeError {
    /// Not JSON, not the envelope shape, a missing field, a string field the
    /// decoder cannot represent, or a nonce/subject_digest array of the wrong
    /// length. The payload is a static explanation.
    Malformed(&'static str),
    /// The `version` field differed from [`ENVELOPE_VERSION`].
    WrongVersion,
    /// An integer field exceeded 2^53 (not I-JSON); carries the field name.
    NonCanonicalNumber(&'static str),
    /// The caller's output buffer could not hold the requested bytes; no byte
    /// was written.
    BufferTooSmall {
        /// Bytes the recomputed form needs.
        needed: usize,
        /// Bytes the caller provided.
        given: usize,
    },
    /// The validity window had not opened; carries the `not_before` instant.
    NotYetValid(u64),
    /// The validity window had closed; carries the `expires_at` instant.
    Expired(u64),
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvelopeError::Malformed(m) => write!(f, "malformed envelope: {m}"),
            EnvelopeError::WrongVersion => {
                write!(f, "unknown envelope version (expected {ENVELOPE_VERSION})")
            }
            EnvelopeError::NonCanonicalNumber(field) => write!(
                f,
                "number not canonical in `{field}`: integers beyond 2^53 are not I-JSON"
            ),
            EnvelopeError::BufferTooSmall { needed, given } => {
                write!(f, "buffer too small: needed {needed} bytes, given {given}")
            }
            EnvelopeError::NotYetValid(nb) => write!(f, "envelope not valid before {nb}"),
            EnvelopeError::Expired(ex) => write!(f, "envelope expired at {ex}"),
        }
    }
}

/// A signature algorithm identity, mirroring the rendered plane's
/// `AlgorithmId` serde wire form (`SCREAMING_SNAKE_CASE`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Algorithm {
    /// ES256
    Es256,
    /// ES256+ML-DSA-65 composite
    HybridEs256MlDsa65,
    /// ML-DSA-65
    MlDsa65,
    /// SLH-DSA-SHA2-128s
    SlhDsa128s,
}

impl Algorithm {
    /// The wire form the rendered plane's serde writes inside the signed
    /// document (e.g. `"ML_DSA65"` — not the display name `"ML-DSA-65"`).
    #[inline]
    pub fn wire_str(self) -> &'static str {
        match self {
            Algorithm::Es256 => "ES256",
            Algorithm::HybridEs256MlDsa65 => "HYBRID_ES256_ML_DSA65",
            Algorithm::MlDsa65 => "ML_DSA65",
            Algorithm::SlhDsa128s => "SLH_DSA128S",
        }
    }

    /// Parse the wire form; [`EnvelopeError::Malformed`] for anything else.
    pub fn from_wire(s: &str) -> Result<Self, EnvelopeError> {
        match s {
            "ES256" => Ok(Algorithm::Es256),
            "HYBRID_ES256_ML_DSA65" => Ok(Algorithm::HybridEs256MlDsa65),
            "ML_DSA65" => Ok(Algorithm::MlDsa65),
            "SLH_DSA128S" => Ok(Algorithm::SlhDsa128s),
            _ => Err(EnvelopeError::Malformed(
                "unknown algorithm wire form (expected ES256, HYBRID_ES256_ML_DSA65, \
                 ML_DSA65, or SLH_DSA128S)",
            )),
        }
    }
}

/// An assurance profile, mirroring the rendered plane's `CryptoProfile`
/// serde wire form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    /// Classical cryptography only.
    Classical,
    /// Composite classical + post-quantum material.
    Hybrid,
    /// Post-quantum only.
    Pqc,
}

impl Profile {
    /// The wire form inside the signed document.
    #[inline]
    pub fn wire_str(self) -> &'static str {
        match self {
            Profile::Classical => "CLASSICAL",
            Profile::Hybrid => "HYBRID",
            Profile::Pqc => "PQC",
        }
    }

    /// Parse the wire form; [`EnvelopeError::Malformed`] for anything else.
    pub fn from_wire(s: &str) -> Result<Self, EnvelopeError> {
        match s {
            "CLASSICAL" => Ok(Profile::Classical),
            "HYBRID" => Ok(Profile::Hybrid),
            "PQC" => Ok(Profile::Pqc),
            _ => Err(EnvelopeError::Malformed(
                "unknown profile wire form (expected CLASSICAL, HYBRID, or PQC)",
            )),
        }
    }
}

/// A borrowed, zero-allocation signature envelope: the twelve graph-declared
/// fields, with the string fields borrowed from the decoded input.
///
/// Obtain one from [`EnvelopeRef::from_json`]; every field is public so the
/// caller can inspect exactly what was admitted. The canonical bytes and the
/// signing pre-image are recomputable allocation-free via
/// [`EnvelopeRef::write_canonical`] and [`EnvelopeRef::write_signing_input`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EnvelopeRef<'a> {
    /// Envelope format tag; admitted only when equal to [`ENVELOPE_VERSION`].
    pub version: &'a str,
    /// Signature algorithm identity (bound inside the signed bytes).
    pub algorithm: Algorithm,
    /// Key identifier (bound inside the signed bytes).
    pub key_id: &'a str,
    /// Assurance profile (bound inside the signed bytes).
    pub profile: Profile,
    /// Policy epoch gating the algorithm/key admission.
    pub policy_epoch: u64,
    /// Revocation epoch gating the key's revocation state.
    pub revocation_epoch: u64,
    /// Key generation counter.
    pub generation: u32,
    /// 16-byte replay-protection nonce.
    pub nonce: [u8; 16],
    /// Window opens (inclusive), seconds since the trust-plane epoch.
    pub not_before: u64,
    /// Window closes (inclusive), seconds since the trust-plane epoch.
    pub expires_at: u64,
    /// 32-byte digest of the subject this envelope certifies.
    pub subject_digest: [u8; 32],
    /// Intended relying-party audience.
    pub audience: &'a str,
}

impl<'a> EnvelopeRef<'a> {
    /// Decode an envelope document from its JSON bytes, borrowing every string
    /// field from `input` — allocation-free and `no_std`.
    ///
    /// Refusals, typed:
    /// - [`EnvelopeError::Malformed`] — not JSON, not the twelve-field shape,
    ///   a missing field, a nonce/subject_digest array of the wrong length,
    ///   an integer outside its field type, an unknown algorithm/profile wire
    ///   form, or an **escape sequence inside a string field** (this decoder
    ///   hands back zero-copy slices; use the `alloc`
    ///   [`SignatureEnvelope::from_json`] to decode escapes);
    /// - [`EnvelopeError::WrongVersion`] — `version` differs from
    ///   [`ENVELOPE_VERSION`].
    ///
    /// Unknown fields are skipped (bounds-checked, structurally validated) and
    /// ignored, mirroring the rendered plane's serde admission; duplicate
    /// known fields take the last occurrence, as serde does.
    pub fn from_json(input: &'a [u8]) -> Result<Self, EnvelopeError> {
        let fields = parse_fields::<BorrowedStr>(&mut Parser::new(input))?;
        let version = require("missing field `version`", fields.version)?;
        check_version(version)?;
        Ok(EnvelopeRef {
            version,
            algorithm: require("missing field `algorithm`", fields.algorithm)?,
            key_id: require("missing field `key_id`", fields.key_id)?,
            profile: require("missing field `profile`", fields.profile)?,
            policy_epoch: require("missing field `policy_epoch`", fields.policy_epoch)?,
            revocation_epoch: require("missing field `revocation_epoch`", fields.revocation_epoch)?,
            generation: require("missing field `generation`", fields.generation)?,
            nonce: require("missing field `nonce`", fields.nonce)?,
            not_before: require("missing field `not_before`", fields.not_before)?,
            expires_at: require("missing field `expires_at`", fields.expires_at)?,
            subject_digest: require("missing field `subject_digest`", fields.subject_digest)?,
            audience: require("missing field `audience`", fields.audience)?,
        })
    }

    /// The exact length in bytes of [`EnvelopeRef::write_canonical`]'s output.
    ///
    /// Total and allocation-free. This does *not* pre-check the 2^53 bound
    /// ([`EnvelopeRef::check_canonical`] does); a document with an
    /// out-of-range integer still has a well-defined length — it just may not
    /// be written.
    pub fn canonical_len(&self) -> usize {
        // 2 braces + 11 commas + (quoted key + colon) for each registry field.
        let mut n = 2 + (ENVELOPE_FIELDS.len() - 1);
        for (k, _) in ENVELOPE_FIELDS {
            n += k.len() + 3;
        }
        for s in [
            self.algorithm.wire_str(),
            self.audience,
            self.key_id,
            self.profile.wire_str(),
            self.version,
        ] {
            n += escaped_len(s);
        }
        for v in [
            self.expires_at,
            u64::from(self.generation),
            self.not_before,
            self.policy_epoch,
            self.revocation_epoch,
        ] {
            n += decimal_len(v);
        }
        // brackets + commas + decimal digits of every entry
        n += 2
            + (self.nonce.len() - 1)
            + self
                .nonce
                .iter()
                .map(|&b| decimal_len(u64::from(b)))
                .sum::<usize>();
        n += 2
            + (self.subject_digest.len() - 1)
            + self
                .subject_digest
                .iter()
                .map(|&b| decimal_len(u64::from(b)))
                .sum::<usize>();
        n
    }

    /// Refuse, typed [`EnvelopeError::NonCanonicalNumber`], any integer field
    /// whose value exceeds 2^53 — the rendered plane's JCS refusal, mirrored
    /// before any byte is written.
    pub fn check_canonical(&self) -> Result<(), EnvelopeError> {
        // `generation` is u32: structurally below 2^53, nothing to check.
        for (field, v) in [
            ("policy_epoch", self.policy_epoch),
            ("revocation_epoch", self.revocation_epoch),
            ("not_before", self.not_before),
            ("expires_at", self.expires_at),
        ] {
            if v > MAX_SAFE_INTEGER {
                return Err(EnvelopeError::NonCanonicalNumber(field));
            }
        }
        Ok(())
    }

    /// Write the canonical document bytes (the JCS subset described in the
    /// module docs) into `out`, returning the number of bytes written.
    ///
    /// Refusals, typed, before any byte is written:
    /// [`EnvelopeError::NonCanonicalNumber`] (an integer field beyond 2^53)
    /// and [`EnvelopeError::BufferTooSmall`] (size the buffer with
    /// [`EnvelopeRef::canonical_len`]).
    pub fn write_canonical(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> {
        self.check_canonical()?;
        let needed = self.canonical_len();
        if out.len() < needed {
            return Err(EnvelopeError::BufferTooSmall {
                needed,
                given: out.len(),
            });
        }
        let mut w = SliceOut::new(out);
        w.push(b'{');
        w.escaped_key_value("algorithm", self.algorithm.wire_str());
        w.push(b',');
        w.escaped_key_value("audience", self.audience);
        w.push(b',');
        w.raw_key_value("expires_at", self.expires_at);
        w.push(b',');
        w.raw_key_value("generation", u64::from(self.generation));
        w.push(b',');
        w.escaped_key_value("key_id", self.key_id);
        w.push(b',');
        w.byte_array_key_value("nonce", &self.nonce);
        w.push(b',');
        w.raw_key_value("not_before", self.not_before);
        w.push(b',');
        w.raw_key_value("policy_epoch", self.policy_epoch);
        w.push(b',');
        w.escaped_key_value("profile", self.profile.wire_str());
        w.push(b',');
        w.raw_key_value("revocation_epoch", self.revocation_epoch);
        w.push(b',');
        w.byte_array_key_value("subject_digest", &self.subject_digest);
        w.push(b',');
        w.escaped_key_value("version", self.version);
        w.push(b'}');
        debug_assert_eq!(w.pos, needed, "canonical_len must predict write_canonical");
        Ok(w.pos)
    }

    /// The exact length in bytes of [`EnvelopeRef::write_signing_input`]'s
    /// output: the domain-separated wrapper plus the canonical document.
    pub fn signing_input_len(&self) -> usize {
        // DOMAIN_TAG || 0x00 || DOMAIN_TAG || 0x00 || u64_be(len) || canonical
        self.canonical_len() + 2 * DOMAIN_TAG.len() + 2 + 8
    }

    /// Write the signing pre-image —
    /// `DOMAIN_TAG || 0x00 || DOMAIN_TAG || 0x00 || u64_be(len) || canonical`
    /// — into `out`, returning the number of bytes written. These are the
    /// exact bytes an external verifier's signature must cover. Refusals as in
    /// [`EnvelopeRef::write_canonical`]; on any refusal no byte is written.
    pub fn write_signing_input(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> {
        self.check_canonical()?;
        let needed = self.signing_input_len();
        if out.len() < needed {
            return Err(EnvelopeError::BufferTooSmall {
                needed,
                given: out.len(),
            });
        }
        let mut w = SliceOut::new(out);
        w.push_str(DOMAIN_TAG);
        w.push(0x00);
        w.push_str(DOMAIN_TAG);
        w.push(0x00);
        w.push_usize_be_u64(self.canonical_len());
        // Infallible here: the buffer was sized above and the integer bound
        // was already checked.
        let n = self
            .write_canonical(w.remaining_mut())
            .map_err(|_| EnvelopeError::Malformed("canonical write into a sized buffer failed"))?;
        w.skip(n);
        debug_assert_eq!(w.pos, needed, "signing_input_len must predict the write");
        Ok(w.pos)
    }

    /// Admit the validity window at instant `now` (seconds since the
    /// trust-plane epoch). Boundaries are inclusive: `now == not_before` is
    /// live and `now == expires_at` is live. Refusals, typed:
    /// [`EnvelopeError::NotYetValid`] / [`EnvelopeError::Expired`].
    pub fn window_live(&self, now: u64) -> Result<(), EnvelopeError> {
        if now < self.not_before {
            return Err(EnvelopeError::NotYetValid(self.not_before));
        }
        if now > self.expires_at {
            return Err(EnvelopeError::Expired(self.expires_at));
        }
        Ok(())
    }
}

/// The owned signature envelope: the same twelve graph-declared fields with
/// heap-backed strings and an escape-decoding JSON decoder. Behind the `alloc`
/// feature, exactly like [`crate::chain::ChainBuilder`]; the verify path
/// ([`EnvelopeRef`]) needs none of it.
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SignatureEnvelope {
    /// Envelope format tag; admitted only when equal to [`ENVELOPE_VERSION`].
    pub version: alloc::string::String,
    /// Signature algorithm identity (bound inside the signed bytes).
    pub algorithm: Algorithm,
    /// Key identifier (bound inside the signed bytes).
    pub key_id: alloc::string::String,
    /// Assurance profile (bound inside the signed bytes).
    pub profile: Profile,
    /// Policy epoch gating the algorithm/key admission.
    pub policy_epoch: u64,
    /// Revocation epoch gating the key's revocation state.
    pub revocation_epoch: u64,
    /// Key generation counter.
    pub generation: u32,
    /// 16-byte replay-protection nonce.
    pub nonce: [u8; 16],
    /// Window opens (inclusive), seconds since the trust-plane epoch.
    pub not_before: u64,
    /// Window closes (inclusive), seconds since the trust-plane epoch.
    pub expires_at: u64,
    /// 32-byte digest of the subject this envelope certifies.
    pub subject_digest: [u8; 32],
    /// Intended relying-party audience.
    pub audience: alloc::string::String,
}

#[cfg(feature = "alloc")]
impl SignatureEnvelope {
    /// Decode an envelope document from its JSON bytes, decoding escape
    /// sequences into owned strings.
    ///
    /// Refusals, typed, as [`EnvelopeRef::from_json`] except that escapes are
    /// supported (malformed escapes — bad hex, lone surrogates — are still
    /// [`EnvelopeError::Malformed`]). Wrong `version` is
    /// [`EnvelopeError::WrongVersion`].
    pub fn from_json(input: &[u8]) -> Result<Self, EnvelopeError> {
        let fields = parse_fields::<OwnedString>(&mut Parser::new(input))?;
        let env = SignatureEnvelope {
            version: require("missing field `version`", fields.version)?,
            algorithm: require("missing field `algorithm`", fields.algorithm)?,
            key_id: require("missing field `key_id`", fields.key_id)?,
            profile: require("missing field `profile`", fields.profile)?,
            policy_epoch: require("missing field `policy_epoch`", fields.policy_epoch)?,
            revocation_epoch: require("missing field `revocation_epoch`", fields.revocation_epoch)?,
            generation: require("missing field `generation`", fields.generation)?,
            nonce: require("missing field `nonce`", fields.nonce)?,
            not_before: require("missing field `not_before`", fields.not_before)?,
            expires_at: require("missing field `expires_at`", fields.expires_at)?,
            subject_digest: require("missing field `subject_digest`", fields.subject_digest)?,
            audience: require("missing field `audience`", fields.audience)?,
        };
        check_version(&env.version)?;
        Ok(env)
    }

    /// Borrow as the zero-allocation [`EnvelopeRef`] view.
    #[inline]
    pub fn borrow(&self) -> EnvelopeRef<'_> {
        EnvelopeRef {
            version: &self.version,
            algorithm: self.algorithm,
            key_id: &self.key_id,
            profile: self.profile,
            policy_epoch: self.policy_epoch,
            revocation_epoch: self.revocation_epoch,
            generation: self.generation,
            nonce: self.nonce,
            not_before: self.not_before,
            expires_at: self.expires_at,
            subject_digest: self.subject_digest,
            audience: &self.audience,
        }
    }

    /// Canonical signed bytes: the JCS-subset serialization of the envelope
    /// document (including the `version` field). Deterministic: byte-identical
    /// for `PartialEq`-equal envelopes. Refuses
    /// [`EnvelopeError::NonCanonicalNumber`] when any integer field exceeds
    /// 2^53.
    pub fn to_bytes(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> {
        self.borrow().check_canonical()?;
        let mut out = alloc::vec![0u8; self.borrow().canonical_len()];
        let written = self.borrow().write_canonical(&mut out)?;
        out.truncate(written);
        Ok(out)
    }

    /// The exact signing pre-image:
    /// `DOMAIN_TAG || 0x00 || DOMAIN_TAG || 0x00 || u64_be(len) || canonical`.
    /// Deterministic; refuses as [`SignatureEnvelope::to_bytes`].
    pub fn signing_input(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> {
        self.borrow().check_canonical()?;
        let mut out = alloc::vec![0u8; self.borrow().signing_input_len()];
        let written = self.borrow().write_signing_input(&mut out)?;
        out.truncate(written);
        Ok(out)
    }

    /// Admit the validity window at instant `now`; see
    /// [`EnvelopeRef::window_live`].
    pub fn window_live(&self, now: u64) -> Result<(), EnvelopeError> {
        self.borrow().window_live(now)
    }
}

#[cfg(feature = "alloc")]
impl From<EnvelopeRef<'_>> for SignatureEnvelope {
    fn from(r: EnvelopeRef<'_>) -> Self {
        SignatureEnvelope {
            version: alloc::string::String::from(r.version),
            algorithm: r.algorithm,
            key_id: alloc::string::String::from(r.key_id),
            profile: r.profile,
            policy_epoch: r.policy_epoch,
            revocation_epoch: r.revocation_epoch,
            generation: r.generation,
            nonce: r.nonce,
            not_before: r.not_before,
            expires_at: r.expires_at,
            subject_digest: r.subject_digest,
            audience: alloc::string::String::from(r.audience),
        }
    }
}

// ---------------------------------------------------------------------------
// JSON decode machinery (no_std, allocation-free)
// ---------------------------------------------------------------------------

/// A decoded string field: a borrowed slice (no-alloc path) or an owned
/// `String` (`alloc` path). Implemented by [`BorrowedStr`] and [`OwnedString`]
/// so one object-walk parser serves both disciplines.
trait StrSink<'a> {
    /// The decoded string representation.
    type Str: AsRef<str>;
    /// Decode one raw JSON string body (the bytes between the quotes).
    fn decode(raw: &'a [u8]) -> Result<Self::Str, EnvelopeError>;
}

/// Borrowed sink: strings must be escape-free so the decoded value is exactly
/// a slice of the input (zero-copy, no allocation).
struct BorrowedStr;

impl<'a> StrSink<'a> for BorrowedStr {
    type Str = &'a str;
    fn decode(raw: &'a [u8]) -> Result<&'a str, EnvelopeError> {
        if raw.contains(&b'\\') {
            return Err(EnvelopeError::Malformed(
                "escape sequence in an envelope string field: the no-alloc decoder returns \
                 zero-copy slices; decode escapes with the alloc SignatureEnvelope::from_json",
            ));
        }
        core::str::from_utf8(raw).map_err(|_| EnvelopeError::Malformed("string field is not UTF-8"))
    }
}

/// Owned sink (behind `alloc`): decodes `\"`, `\\`, `\/`, `\b \f \n \r \t`
/// and `\uXXXX` (with surrogate pairs) into an owned `String`.
#[cfg(feature = "alloc")]
struct OwnedString;

#[cfg(feature = "alloc")]
impl<'a> StrSink<'a> for OwnedString {
    type Str = alloc::string::String;
    fn decode(raw: &'a [u8]) -> Result<alloc::string::String, EnvelopeError> {
        let mut out: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(raw.len());
        let mut i = 0;
        while i < raw.len() {
            let b = raw[i];
            if b != b'\\' {
                out.push(b);
                i += 1;
                continue;
            }
            let Some(d) = raw.get(i + 1).copied() else {
                return Err(EnvelopeError::Malformed("unterminated escape"));
            };
            i += 2; // past '\' and the escape designator
            match d {
                b'"' => out.push(b'"'),
                b'\\' => out.push(b'\\'),
                b'/' => out.push(b'/'),
                b'b' => out.push(0x08),
                b'f' => out.push(0x0C),
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b't' => out.push(b'\t'),
                b'u' => {
                    let mut cp =
                        decode_hex4(raw, i).ok_or(EnvelopeError::Malformed("bad \\u hex"))?;
                    i += 4;
                    if (0xD800..=0xDBFF).contains(&cp) {
                        // High surrogate: require an immediately following low surrogate.
                        if raw.get(i) != Some(&b'\\') || raw.get(i + 1) != Some(&b'u') {
                            return Err(EnvelopeError::Malformed("lone high surrogate"));
                        }
                        let lo = decode_hex4(raw, i + 2)
                            .ok_or(EnvelopeError::Malformed("bad surrogate \\u hex"))?;
                        if !(0xDC00..=0xDFFF).contains(&lo) {
                            return Err(EnvelopeError::Malformed("lone high surrogate"));
                        }
                        cp = 0x1_0000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                        i += 6;
                    } else if (0xDC00..=0xDFFF).contains(&cp) {
                        return Err(EnvelopeError::Malformed("lone low surrogate"));
                    }
                    encode_utf8_into(cp, &mut out);
                }
                _ => return Err(EnvelopeError::Malformed("unknown escape designator")),
            }
        }
        alloc::string::String::from_utf8(out)
            .map_err(|_| EnvelopeError::Malformed("string field is not UTF-8"))
    }
}

/// Decode one `\uXXXX` body at `raw[i..i+4]` into a code point.
#[cfg(feature = "alloc")]
fn decode_hex4(raw: &[u8], i: usize) -> Option<u32> {
    let h = |b: u8| -> Option<u32> {
        match b {
            b'0'..=b'9' => Some(u32::from(b - b'0')),
            b'a'..=b'f' => Some(u32::from(b - b'a' + 10)),
            b'A'..=b'F' => Some(u32::from(b - b'A' + 10)),
            _ => None,
        }
    };
    if i + 4 > raw.len() {
        return None;
    }
    Some((h(raw[i])? << 12) | (h(raw[i + 1])? << 8) | (h(raw[i + 2])? << 4) | h(raw[i + 3])?)
}

/// Push a scalar code point's UTF-8 encoding (surrogates already combined).
#[cfg(feature = "alloc")]
fn encode_utf8_into(cp: u32, out: &mut alloc::vec::Vec<u8>) {
    let ch = char::from_u32(cp).unwrap_or(char::REPLACEMENT_CHARACTER);
    let mut tmp = [0u8; 4];
    out.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
}

/// The twelve fields mid-parse, before the missing-field admission check.
struct Fields<S> {
    version: Option<S>,
    algorithm: Option<Algorithm>,
    key_id: Option<S>,
    profile: Option<Profile>,
    policy_epoch: Option<u64>,
    revocation_epoch: Option<u64>,
    generation: Option<u32>,
    nonce: Option<[u8; 16]>,
    not_before: Option<u64>,
    expires_at: Option<u64>,
    subject_digest: Option<[u8; 32]>,
    audience: Option<S>,
}

/// Unwrap one decoded field or refuse with its static missing-field message.
fn require<T>(missing: &'static str, v: Option<T>) -> Result<T, EnvelopeError> {
    v.ok_or(EnvelopeError::Malformed(missing))
}

/// The single object-walk parser, generic over the string sink. Enforces the
/// rendered plane's serde admission shape: all twelve fields required, unknown
/// fields skipped (bounds-checked) and ignored, duplicate fields take the last
/// occurrence.
fn parse_fields<'a, K: StrSink<'a>>(p: &mut Parser<'a>) -> Result<Fields<K::Str>, EnvelopeError> {
    let mut f = Fields {
        version: None,
        algorithm: None,
        key_id: None,
        profile: None,
        policy_epoch: None,
        revocation_epoch: None,
        generation: None,
        nonce: None,
        not_before: None,
        expires_at: None,
        subject_digest: None,
        audience: None,
    };

    p.skip_ws();
    p.expect_byte(b'{')?;
    p.skip_ws();
    if p.peek() == Some(b'}') {
        p.bump();
    } else {
        loop {
            p.skip_ws();
            p.expect_byte(b'"')?;
            let key = scan_string_raw(p)?;
            p.skip_ws();
            p.expect_byte(b':')?;
            p.skip_ws();
            match key {
                b"version" => f.version = Some(K::decode(scan_string_value(p)?)?),
                b"algorithm" => {
                    let s = K::decode(scan_string_value(p)?)?;
                    f.algorithm = Some(Algorithm::from_wire(s.as_ref())?);
                }
                b"key_id" => f.key_id = Some(K::decode(scan_string_value(p)?)?),
                b"profile" => {
                    let s = K::decode(scan_string_value(p)?)?;
                    f.profile = Some(Profile::from_wire(s.as_ref())?);
                }
                b"policy_epoch" => f.policy_epoch = Some(scan_u64(p)?),
                b"revocation_epoch" => f.revocation_epoch = Some(scan_u64(p)?),
                b"generation" => {
                    let v = scan_u64(p)?;
                    f.generation = Some(
                        u32::try_from(v)
                            .map_err(|_| EnvelopeError::Malformed("generation exceeds u32"))?,
                    );
                }
                b"nonce" => {
                    let mut n = [0u8; 16];
                    scan_byte_array(p, &mut n)?;
                    f.nonce = Some(n);
                }
                b"not_before" => f.not_before = Some(scan_u64(p)?),
                b"expires_at" => f.expires_at = Some(scan_u64(p)?),
                b"subject_digest" => {
                    let mut n = [0u8; 32];
                    scan_byte_array(p, &mut n)?;
                    f.subject_digest = Some(n);
                }
                b"audience" => f.audience = Some(K::decode(scan_string_value(p)?)?),
                _ => skip_value(p)?,
            }
            p.skip_ws();
            match p.next_byte() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(EnvelopeError::Malformed("expected ',' or '}' in object")),
            }
        }
    }
    p.skip_ws();
    if p.pos != p.input.len() {
        return Err(EnvelopeError::Malformed("trailing bytes after document"));
    }
    Ok(f)
}

/// A minimal byte-cursor over the JSON input.
struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Parser { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) {
        self.pos += 1;
    }

    fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek();
        if b.is_some() {
            self.bump();
        }
        b
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn expect_byte(&mut self, want: u8) -> Result<(), EnvelopeError> {
        match self.next_byte() {
            Some(b) if b == want => Ok(()),
            _ => Err(EnvelopeError::Malformed("unexpected byte")),
        }
    }
}

/// Scan a JSON string body starting after the opening quote; returns the raw
/// bytes between the quotes. Validates escape syntax (so every consumer of
/// the raw body agrees on where the string ends) and refuses raw C0 control
/// bytes, mirroring JSON's grammar.
fn scan_string_raw<'a>(p: &mut Parser<'a>) -> Result<&'a [u8], EnvelopeError> {
    let start = p.pos;
    loop {
        let Some(b) = p.next_byte() else {
            return Err(EnvelopeError::Malformed("unterminated string"));
        };
        match b {
            b'"' => return Ok(&p.input[start..p.pos - 1]),
            b'\\' => {
                let Some(d) = p.next_byte() else {
                    return Err(EnvelopeError::Malformed("unterminated escape"));
                };
                match d {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                    b'u' => {
                        for _ in 0..4 {
                            match p.next_byte() {
                                Some(h) if h.is_ascii_hexdigit() => {}
                                _ => return Err(EnvelopeError::Malformed("bad \\u hex")),
                            }
                        }
                    }
                    _ => return Err(EnvelopeError::Malformed("unknown escape designator")),
                }
            }
            b if b < 0x20 => return Err(EnvelopeError::Malformed("raw control byte in string")),
            _ => {}
        }
    }
}

/// Scan an unsigned decimal integer literal. Mirrors the rendered plane's
/// serde admission of `u64` fields: negative or non-integer literals are
/// typed refusals, leading zeros are invalid JSON, and overflow past u64 is
/// refused rather than wrapped.
fn scan_u64(p: &mut Parser<'_>) -> Result<u64, EnvelopeError> {
    p.skip_ws();
    let Some(first) = p.peek() else {
        return Err(EnvelopeError::Malformed("expected a number"));
    };
    if first == b'-' {
        return Err(EnvelopeError::Malformed(
            "negative number where an unsigned integer is required",
        ));
    }
    if !first.is_ascii_digit() {
        return Err(EnvelopeError::Malformed("expected a number"));
    }
    let mut acc: u64 = 0;
    let mut digits = 0usize;
    while let Some(c) = p.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        digits += 1;
        if digits > 20 {
            return Err(EnvelopeError::Malformed("number exceeds u64"));
        }
        acc = acc
            .checked_mul(10)
            .and_then(|a| a.checked_add(u64::from(c - b'0')))
            .ok_or(EnvelopeError::Malformed("number exceeds u64"))?;
        p.bump();
    }
    if digits > 1 && first == b'0' {
        return Err(EnvelopeError::Malformed("leading zero in number"));
    }
    if matches!(p.peek(), Some(b'.' | b'e' | b'E')) {
        return Err(EnvelopeError::Malformed(
            "non-integer number where an unsigned integer is required",
        ));
    }
    Ok(acc)
}

/// Scan a JSON array of exactly `out.len()` byte-valued integer literals into
/// `out`; any other length, out-of-range entry, or non-integer literal is a
/// typed refusal (the rendered plane's serde `[u8; N]` admission).
fn scan_byte_array(p: &mut Parser<'_>, out: &mut [u8]) -> Result<(), EnvelopeError> {
    p.skip_ws();
    p.expect_byte(b'[')?;
    p.skip_ws();
    let mut count = 0usize;
    if p.peek() == Some(b']') {
        p.bump();
    } else {
        loop {
            let v = scan_u64(p)?;
            if v > 255 {
                return Err(EnvelopeError::Malformed("byte array entry out of range"));
            }
            if count >= out.len() {
                return Err(EnvelopeError::Malformed("byte array has the wrong length"));
            }
            out[count] = v as u8;
            count += 1;
            p.skip_ws();
            match p.next_byte() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(EnvelopeError::Malformed("expected ',' or ']' in array")),
            }
        }
    }
    if count != out.len() {
        return Err(EnvelopeError::Malformed("byte array has the wrong length"));
    }
    Ok(())
}

/// Strictly skip one JSON value of any shape (an unknown field). A bounded,
/// iterative structural validator: mirrors serde_json's refusal of adjacent
/// scalars, mismatched brackets, broken literals, and depth beyond
/// [`MAX_SKIP_DEPTH`].
fn skip_value(p: &mut Parser<'_>) -> Result<(), EnvelopeError> {
    // Frame stack: bit 0 = container kind (0 array, 1 object).
    let mut stack = [0u8; MAX_SKIP_DEPTH];
    let mut sp = 0usize;

    loop {
        // Parse one value (scalar, string, or container opening).
        p.skip_ws();
        match p.peek() {
            None => return Err(EnvelopeError::Malformed("unexpected end of document")),
            Some(b'"') => {
                p.bump();
                scan_string_raw(p)?;
                if sp == 0 {
                    return Ok(());
                }
            }
            Some(open @ (b'{' | b'[')) => {
                let obj = open == b'{';
                p.bump();
                if sp == MAX_SKIP_DEPTH {
                    return Err(EnvelopeError::Malformed("document nesting too deep"));
                }
                stack[sp] = u8::from(obj);
                sp += 1;
                p.skip_ws();
                let close = if obj { b'}' } else { b']' };
                if p.peek() != Some(close) {
                    if obj {
                        // First key of a non-empty object.
                        p.expect_byte(b'"')?;
                        scan_string_raw(p)?;
                        p.skip_ws();
                        p.expect_byte(b':')?;
                    }
                    continue; // parse the first element/value
                }
                p.bump(); // empty container
                sp -= 1;
                if sp == 0 {
                    return Ok(());
                }
            }
            Some(c) if c == b'-' || c.is_ascii_digit() || c == b't' || c == b'f' || c == b'n' => {
                let start = p.pos;
                while matches!(
                    p.peek(),
                    Some(b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'+' | b'.')
                ) {
                    p.bump();
                }
                if !scalar_token_is_valid(&p.input[start..p.pos]) {
                    return Err(EnvelopeError::Malformed("invalid scalar value"));
                }
                if sp == 0 {
                    return Ok(());
                }
            }
            Some(_) => return Err(EnvelopeError::Malformed("invalid value")),
        }

        // A complete value inside a container: expect ',' (plus a key when the
        // frame is an object), or the container's closer (then repeat for the
        // parent frame).
        loop {
            p.skip_ws();
            let Some(c) = p.peek() else {
                return Err(EnvelopeError::Malformed("unexpected end of document"));
            };
            let obj = stack[sp - 1] & 1 == 1;
            match c {
                b',' => {
                    p.bump();
                    if obj {
                        p.skip_ws();
                        p.expect_byte(b'"')?;
                        scan_string_raw(p)?;
                        p.skip_ws();
                        p.expect_byte(b':')?;
                    }
                    break; // parse the next value
                }
                closer @ (b'}' | b']') => {
                    let want = if obj { b'}' } else { b']' };
                    if closer != want {
                        return Err(EnvelopeError::Malformed("mismatched bracket"));
                    }
                    p.bump();
                    sp -= 1;
                    if sp == 0 {
                        return Ok(());
                    }
                    // Stay in this loop: the parent now awaits ',' or its closer.
                }
                _ => return Err(EnvelopeError::Malformed("expected ',' or a closer")),
            }
        }
    }
}

/// Consume the opening quote, then scan the string body: the shape every
/// known-field *value* site needs.
fn scan_string_value<'a>(p: &mut Parser<'a>) -> Result<&'a [u8], EnvelopeError> {
    p.expect_byte(b'"')?;
    scan_string_raw(p)
}

/// Validate a scalar token seen while skipping an unknown value: exactly
/// `true` / `false` / `null`, or a strict JSON number.
fn scalar_token_is_valid(token: &[u8]) -> bool {
    match token {
        b"true" | b"false" | b"null" => return true,
        _ => {}
    }
    let mut i = 0usize;
    if token.first() == Some(&b'-') {
        i += 1;
    }
    // Integer part.
    match token.get(i) {
        Some(b'0') => {
            i += 1;
            if matches!(token.get(i), Some(c) if c.is_ascii_digit()) {
                return false; // leading zero
            }
        }
        Some(c) if c.is_ascii_digit() => {
            while matches!(token.get(i), Some(c) if c.is_ascii_digit()) {
                i += 1;
            }
        }
        _ => return false,
    }
    // Fraction.
    if token.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while matches!(token.get(i), Some(c) if c.is_ascii_digit()) {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    // Exponent.
    if matches!(token.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(token.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while matches!(token.get(i), Some(c) if c.is_ascii_digit()) {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    i == token.len()
}

fn check_version(version: &str) -> Result<(), EnvelopeError> {
    if version == ENVELOPE_VERSION {
        Ok(())
    } else {
        Err(EnvelopeError::WrongVersion)
    }
}

// ---------------------------------------------------------------------------
// Canonical writing (the JCS subset; no_std, allocation-free)
// ---------------------------------------------------------------------------

/// A write cursor over a caller-provided byte buffer. Capacity is checked by
/// the callers (`BufferTooSmall` before any byte is written), so pushes only
/// saturate defensively.
struct SliceOut<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> SliceOut<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        SliceOut { buf, pos: 0 }
    }

    fn remaining_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    fn push(&mut self, b: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = b;
        }
        self.pos += 1;
    }

    fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.push(b);
        }
    }

    /// Push the 8-byte big-endian encoding of `v` (the part-length prefix).
    ///
    /// Widen to u64 BEFORE shifting: on 32-bit targets (wasm32, armv7) a
    /// `usize >> 32..=56` would be a shift overflow, and the wire form is
    /// u64 big-endian on every target — witnessed by the wasmi parity test
    /// (`tests/wasm_abi.rs`), which caught exactly this drift.
    fn push_usize_be_u64(&mut self, v: usize) {
        let v = v as u64;
        for shift in (0..8).rev() {
            self.push((v >> (shift * 8)) as u8);
        }
    }

    /// `"key":"value"` with RFC 8785 §3.2.2.2 escaping.
    fn escaped_key_value(&mut self, key: &str, value: &str) {
        self.push_escaped(key);
        self.push(b':');
        self.push_escaped(value);
    }

    /// `"key":<decimal>`.
    fn raw_key_value(&mut self, key: &str, value: u64) {
        self.push_escaped(key);
        self.push(b':');
        self.push_decimal(value);
    }

    /// `"key":[a,b,…]` over byte values.
    fn byte_array_key_value(&mut self, key: &str, bytes: &[u8]) {
        self.push_escaped(key);
        self.push(b':');
        self.push(b'[');
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 {
                self.push(b',');
            }
            self.push_decimal(u64::from(*b));
        }
        self.push(b']');
    }

    fn push_escaped(&mut self, s: &str) {
        self.push(b'"');
        for c in s.chars() {
            match c {
                '"' => self.push_str("\\\""),
                '\\' => self.push_str("\\\\"),
                '\u{0008}' => self.push_str("\\b"),
                '\u{0009}' => self.push_str("\\t"),
                '\u{000A}' => self.push_str("\\n"),
                '\u{000C}' => self.push_str("\\f"),
                '\u{000D}' => self.push_str("\\r"),
                ch if (ch as u32) < 0x20 => {
                    self.push_str("\\u00");
                    let v = ch as u32;
                    self.push(HEX[((v >> 4) & 0xf) as usize]);
                    self.push(HEX[(v & 0xf) as usize]);
                }
                c => {
                    let mut tmp = [0u8; 4];
                    self.push_str(c.encode_utf8(&mut tmp));
                }
            }
        }
        self.push(b'"');
    }

    fn push_decimal(&mut self, v: u64) {
        let mut tmp = [0u8; 20];
        let mut i = tmp.len();
        let mut v = v;
        loop {
            i -= 1;
            tmp[i] = b'0' + (v % 10) as u8;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        for &b in &tmp[i..] {
            self.push(b);
        }
    }
}

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Byte length of `s` as it will be written by [`SliceOut::push_escaped`]
/// (including the surrounding quotes). Kept in exact sync with the writer;
/// the `escaped_len_matches_writer_over_scalar_characters` test holds the two
/// together over every scalar character.
fn escaped_len(s: &str) -> usize {
    2 + s
        .chars()
        .map(|c| match c {
            '"' | '\\' | '\u{0008}' | '\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{000D}' => 2,
            ch if (ch as u32) < 0x20 => 6,
            c => c.len_utf8(),
        })
        .sum::<usize>()
}

/// Decimal digit count of `v` (1 for zero).
fn decimal_len(v: u64) -> usize {
    let mut n = 1;
    let mut v = v;
    while v >= 10 {
        v /= 10;
        n += 1;
    }
    n
}

// ---------------------------------------------------------------------------
// Tests (run under std; the lib itself is no_std in non-test builds).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- KAT vectors: rendered-plane envelope surface
    // (fixtures/crypto_trust_kat_vectors.json, surfaces.envelope), hardcoded
    // so the port is held to the real generator's output, not to itself.

    /// KAT vector env-000-ES256: canonical document bytes.
    const V0_CANONICAL: &str = r#"{"algorithm":"ES256","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_a9c3bd446195e2e2","nonce":[25,107,217,46,141,94,95,54,120,72,204,59,49,154,244,42],"not_before":1700000000,"policy_epoch":1,"profile":"CLASSICAL","revocation_epoch":0,"subject_digest":[206,63,11,26,126,80,218,245,190,191,110,203,240,54,52,94,36,141,20,88,5,127,63,121,250,128,109,66,210,110,68,97],"version":"CTP-ENVELOPE-v1"}"#;
    /// KAT vector env-000-ES256: expected signing pre-image.
    const V0_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";
    /// KAT vector env-001-ML-DSA-65: canonical document bytes.
    const V1_CANONICAL: &str = r#"{"algorithm":"ML_DSA65","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_932a436a743d67cd","nonce":[49,148,240,225,154,244,132,33,20,7,183,98,118,211,47,50],"not_before":1700000000,"policy_epoch":1,"profile":"PQC","revocation_epoch":0,"subject_digest":[18,60,249,28,128,193,211,38,120,198,222,80,164,85,43,52,17,76,4,173,228,175,245,220,104,114,253,206,55,248,114,68],"version":"CTP-ENVELOPE-v1"}"#;
    /// KAT vector env-001-ML-DSA-65: expected signing pre-image.
    const V1_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";
    /// KAT vector env-002-ES256+ML-DSA-65: canonical document bytes.
    const V2_CANONICAL: &str = r#"{"algorithm":"HYBRID_ES256_ML_DSA65","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_51deddb3d53698f6","nonce":[231,162,64,114,47,222,37,99,40,110,128,89,143,102,158,46],"not_before":1700000000,"policy_epoch":1,"profile":"HYBRID","revocation_epoch":0,"subject_digest":[106,178,119,6,136,110,176,195,153,50,156,0,35,174,153,58,254,185,106,67,71,36,166,195,68,213,19,13,220,96,173,91],"version":"CTP-ENVELOPE-v1"}"#;
    /// KAT vector env-002-ES256+ML-DSA-65: expected signing pre-image.
    const V2_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";

    fn hex_encode(bytes: &[u8]) -> String {
        let mut s = String::new();
        for b in bytes {
            s.push(HEX[(b >> 4) as usize] as char);
            s.push(HEX[(b & 0x0f) as usize] as char);
        }
        s
    }

    /// Replace one `"field":<value>` scalar value in a canonical document,
    /// leaving the rest byte-identical. Handles the last key (whose value runs
    /// to the closing brace) as well as mid-document keys.
    fn with_field_value(doc: &str, field: &str, value: &str) -> String {
        let needle = format!("\"{field}\":");
        let at = doc.find(&needle).expect("field present in document");
        let rest = &doc[at + needle.len()..];
        let tail = match rest.find(',') {
            Some(end) => &rest[end..],
            None => &rest[rest.rfind('}').expect("document closes")..],
        };
        format!("{}{}{}{}", &doc[..at], needle, value, tail)
    }

    fn signing_input_hex_of(doc: &str) -> String {
        let env = EnvelopeRef::from_json(doc.as_bytes()).expect("document decodes");
        let mut buf = [0u8; 2048];
        let n = env.write_signing_input(&mut buf).expect("pre-image fits");
        hex_encode(&buf[..n])
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(
            ENVELOPE_FIELDS,
            [
                ("version", 1),
                ("algorithm", 2),
                ("key_id", 3),
                ("profile", 4),
                ("policy_epoch", 5),
                ("revocation_epoch", 6),
                ("generation", 7),
                ("nonce", 8),
                ("not_before", 9),
                ("expires_at", 10),
                ("subject_digest", 11),
                ("audience", 12),
            ]
        );
    }

    #[test]
    fn signing_input_matches_the_rendered_plane_kat() {
        // Stability vs the rendered plane: decode the KAT's canonical bytes and
        // recompute both the canonical form (fixed point) and the signing
        // pre-image. These pins are the port's whole admission test.
        for (doc, expected_hex) in [
            (V0_CANONICAL, V0_SIGNING_INPUT_HEX),
            (V1_CANONICAL, V1_SIGNING_INPUT_HEX),
            (V2_CANONICAL, V2_SIGNING_INPUT_HEX),
        ] {
            let env = EnvelopeRef::from_json(doc.as_bytes()).expect("KAT document decodes");
            let mut buf = [0u8; 2048];
            let n = env
                .write_signing_input(&mut buf)
                .expect("signing input fits");
            assert_eq!(n, env.signing_input_len());
            assert_eq!(
                hex_encode(&buf[..n]),
                expected_hex,
                "signing pre-image diverged from the rendered plane"
            );
            // Canonical bytes are a fixed point of decode -> write.
            let m = env.write_canonical(&mut buf).expect("canonical fits");
            assert_eq!(&buf[..m], doc.as_bytes(), "canonical bytes diverged");
            assert_eq!(m, env.canonical_len());
        }
    }

    #[test]
    fn envelope_round_trips_through_canonical_bytes() {
        let env = SignatureEnvelope::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        let bytes = env.to_bytes().expect("canonicalizes");
        assert_eq!(bytes, V0_CANONICAL.as_bytes());
        let back = SignatureEnvelope::from_json(&bytes).expect("round-trip parses");
        assert_eq!(back, env);
        // Byte-identical replay of the canonical form.
        assert_eq!(env.to_bytes().expect("deterministic"), bytes);
        // Owned and borrowed decoders agree field-for-field.
        let borrowed = EnvelopeRef::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        assert_eq!(borrowed, env.borrow());
    }

    #[test]
    fn wire_forms_render_the_serde_attributes() {
        let env = SignatureEnvelope::from_json(V1_CANONICAL.as_bytes()).expect("decodes");
        assert_eq!(env.algorithm, Algorithm::MlDsa65);
        assert_eq!(env.algorithm.wire_str(), "ML_DSA65");
        assert_eq!(env.profile, Profile::Pqc);
        let env = SignatureEnvelope::from_json(V2_CANONICAL.as_bytes()).expect("decodes");
        assert_eq!(env.algorithm, Algorithm::HybridEs256MlDsa65);
        assert_eq!(env.algorithm.wire_str(), "HYBRID_ES256_ML_DSA65");
        assert_eq!(env.profile, Profile::Hybrid);
        // Round-trip every wire form.
        for a in [
            Algorithm::Es256,
            Algorithm::HybridEs256MlDsa65,
            Algorithm::MlDsa65,
            Algorithm::SlhDsa128s,
        ] {
            assert_eq!(Algorithm::from_wire(a.wire_str()), Ok(a));
        }
        for p in [Profile::Classical, Profile::Hybrid, Profile::Pqc] {
            assert_eq!(Profile::from_wire(p.wire_str()), Ok(p));
        }
    }

    #[test]
    fn signing_input_changes_when_any_field_changes() {
        let base = SignatureEnvelope::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        let base_input = base.signing_input().expect("canonicalizes");
        let base_bytes = base.to_bytes().expect("canonicalizes");
        type Mutation = fn(SignatureEnvelope) -> SignatureEnvelope;
        let mutations: [(&str, Mutation); 12] = [
            ("version", |mut e| {
                e.version = "CTP-ENVELOPE-v1-test".into();
                e
            }),
            ("algorithm", |mut e| {
                e.algorithm = Algorithm::SlhDsa128s;
                e
            }),
            ("key_id", |mut e| {
                e.key_id = "k-other".into();
                e
            }),
            ("profile", |mut e| {
                e.profile = Profile::Hybrid;
                e
            }),
            ("policy_epoch", |mut e| {
                e.policy_epoch = 8;
                e
            }),
            ("revocation_epoch", |mut e| {
                e.revocation_epoch = 4;
                e
            }),
            ("generation", |mut e| {
                e.generation = 3;
                e
            }),
            ("nonce", |mut e| {
                e.nonce = [0x11; 16];
                e
            }),
            ("not_before", |mut e| {
                e.not_before = 1_001;
                e
            }),
            ("expires_at", |mut e| {
                e.expires_at = 2_001;
                e
            }),
            ("subject_digest", |mut e| {
                e.subject_digest = [0x23; 32];
                e
            }),
            ("audience", |mut e| {
                e.audience = "affidavit.other".into();
                e
            }),
        ];
        assert_eq!(mutations.len(), ENVELOPE_FIELDS.len());
        for (field, mutate) in mutations {
            let mutated = mutate(base.clone());
            assert_ne!(mutated, base, "{field} mutation must change the value");
            assert_ne!(
                mutated.signing_input().expect("canonicalizes"),
                base_input,
                "{field} mutation must change the signing pre-image"
            );
            assert_ne!(
                mutated.to_bytes().expect("canonicalizes"),
                base_bytes,
                "{field} mutation must change the canonical bytes"
            );
        }
    }

    #[test]
    fn wrong_version_refused_by_variant() {
        let doc = with_field_value(V0_CANONICAL, "version", "\"CTP-ENVELOPE-v0\"");
        for err in [
            EnvelopeRef::from_json(doc.as_bytes()).err(),
            SignatureEnvelope::from_json(doc.as_bytes()).err(),
        ] {
            match err {
                Some(EnvelopeError::WrongVersion) => {}
                other => panic!("expected WrongVersion, got {other:?}"),
            }
        }
    }

    #[test]
    fn malformed_documents_are_refused_by_variant() {
        let trailing = format!("{V0_CANONICAL}x");
        let trailing_ws = format!("{V0_CANONICAL}  ");
        let docs: [(&[u8], &str); 7] = [
            (b"{\"version\": not-json", "not JSON"),
            (b"{}", "empty object"),
            (b"[]", "array, not object"),
            (b"null", "null, not object"),
            (
                b"{\"algorithm\":3}",
                "number where a string field is required",
            ),
            (b"{\"algorithm\":\"ED25519\"}", "unknown wire form"),
            (trailing.as_bytes(), "trailing bytes after the document"),
        ];
        // Trailing whitespace is not trailing garbage: serde admits it.
        assert!(EnvelopeRef::from_json(trailing_ws.as_bytes()).is_ok());
        for (bad, why) in docs {
            assert!(
                matches!(
                    EnvelopeRef::from_json(bad),
                    Err(EnvelopeError::Malformed(_) | EnvelopeError::WrongVersion)
                ),
                "borrowed decoder admitted {why}"
            );
            assert!(
                matches!(
                    SignatureEnvelope::from_json(bad),
                    Err(EnvelopeError::Malformed(_) | EnvelopeError::WrongVersion)
                ),
                "owned decoder admitted {why}"
            );
        }
    }

    #[test]
    fn wrong_byte_array_lengths_are_refused() {
        // Drop the nonce's final entry: 16 -> 15 bytes.
        let doc = V0_CANONICAL.replacen(",154,244,42]", ",154,42]", 1);
        assert_ne!(doc, V0_CANONICAL);
        assert!(matches!(
            EnvelopeRef::from_json(doc.as_bytes()),
            Err(EnvelopeError::Malformed(_))
        ));
        // Add a 17th nonce byte.
        let doc = V0_CANONICAL.replacen(",154,244,42]", ",154,244,42,7]", 1);
        assert!(matches!(
            EnvelopeRef::from_json(doc.as_bytes()),
            Err(EnvelopeError::Malformed(_))
        ));
        // A 33-entry subject_digest and an out-of-range entry are refused too.
        let doc = V0_CANONICAL.replace("210,110,68,97]", "210,110,68,97,1]");
        assert!(matches!(
            EnvelopeRef::from_json(doc.as_bytes()),
            Err(EnvelopeError::Malformed(_))
        ));
        let doc = V0_CANONICAL.replacen("[25,107,", "[256,107,", 1);
        assert!(matches!(
            EnvelopeRef::from_json(doc.as_bytes()),
            Err(EnvelopeError::Malformed(_))
        ));
    }

    #[test]
    fn non_canonical_integer_refused_before_any_byte_is_written() {
        let mut env = SignatureEnvelope::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        env.policy_epoch = 9_007_199_254_740_993; // 2^53 + 1: beyond double precision
        match env.to_bytes() {
            Err(EnvelopeError::NonCanonicalNumber("policy_epoch")) => {}
            other => panic!("expected NonCanonicalNumber, got {other:?}"),
        }
        match env.signing_input() {
            Err(EnvelopeError::NonCanonicalNumber("policy_epoch")) => {}
            other => panic!("expected NonCanonicalNumber from pre-image, got {other:?}"),
        }
        // Boundary: exactly 2^53 is exactly representable and canonicalizes.
        let mut b = SignatureEnvelope::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        b.policy_epoch = 9_007_199_254_740_992;
        assert!(b.to_bytes().is_ok());
    }

    #[test]
    fn borrowed_decoder_refuses_escapes_owned_decoder_accepts_them() {
        // audience "\u0061ffidavit.kat" decodes (owned) to "affidavit.kat"
        // (\u0061 is 'a').
        let doc = V0_CANONICAL.replacen(
            "\"audience\":\"affidavit.kat\"",
            "\"audience\":\"\\u0061ffidavit.kat\"",
            1,
        );
        match EnvelopeRef::from_json(doc.as_bytes()) {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed from the no-alloc decoder, got {other:?}"),
        }
        let owned = SignatureEnvelope::from_json(doc.as_bytes()).expect("owned decoder decodes");
        assert_eq!(owned.audience, "affidavit.kat");
        // Canonical bytes carry the decoded content literally (JCS re-emits it
        // unescaped), so the escape-coded and plain documents admit the SAME
        // signed bytes — the escape is transport, not content.
        let plain = doc.replace("\\u0061ffidavit.kat", "affidavit.kat");
        assert_eq!(plain, V0_CANONICAL);
        let mut buf = [0u8; 2048];
        let n = owned
            .borrow()
            .write_signing_input(&mut buf)
            .expect("pre-image fits");
        assert_eq!(hex_encode(&buf[..n]), V0_SIGNING_INPUT_HEX);
    }

    #[test]
    fn integer_literals_outside_serde_admission_are_refused() {
        for (field, value) in [
            ("policy_epoch", "-1"),
            ("policy_epoch", "1.0"),
            ("policy_epoch", "1e3"),
            ("policy_epoch", "01"),
            ("policy_epoch", "99999999999999999999"),
            ("generation", "4294967296"),
            ("generation", "-1"),
        ] {
            let doc = with_field_value(V0_CANONICAL, field, value);
            assert!(
                matches!(
                    EnvelopeRef::from_json(doc.as_bytes()),
                    Err(EnvelopeError::Malformed(_))
                ),
                "admitted {field} = {value}"
            );
        }
    }

    #[test]
    fn duplicate_keys_take_the_last_and_unknown_keys_are_ignored() {
        // An unknown nested field must not enter the signed bytes: the
        // pre-image is unchanged from the KAT vector.
        let mut doc = String::from("{");
        doc.push_str(r#""zz_unknown":{"a":[1,{"deep":[true,false,null,-1.5e-3]}]},"#);
        doc.push_str(&V0_CANONICAL[1..]);
        assert_eq!(signing_input_hex_of(&doc), V0_SIGNING_INPUT_HEX);

        // A duplicate known key takes the last occurrence (serde semantics) —
        // and because fields are signed, the pre-image follows the last value.
        // The original `audience` is the final key, so the duplicate is
        // appended after it, before the closing brace.
        let doc = V0_CANONICAL.replacen(
            "\"version\":\"CTP-ENVELOPE-v1\"}",
            "\"version\":\"CTP-ENVELOPE-v1\",\"audience\":\"stale\"}",
            1,
        );
        let env = EnvelopeRef::from_json(doc.as_bytes()).expect("decodes");
        assert_eq!(env.audience, "stale");
        assert_ne!(signing_input_hex_of(&doc), V0_SIGNING_INPUT_HEX);
        // …and equals the pre-image of an honest document with that audience.
        assert_eq!(
            signing_input_hex_of(&doc),
            signing_input_hex_of(&with_field_value(V0_CANONICAL, "audience", "\"stale\""))
        );
    }

    #[test]
    fn buffer_too_small_is_typed_and_nothing_is_written() {
        let env = EnvelopeRef::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        let mut tiny = [0xAAu8; 8];
        match env.write_signing_input(&mut tiny) {
            Err(EnvelopeError::BufferTooSmall { needed, given }) => {
                assert_eq!(needed, env.signing_input_len());
                assert_eq!(given, 8);
            }
            other => panic!("expected BufferTooSmall, got {other:?}"),
        }
        assert!(
            tiny.iter().all(|&b| b == 0xAA),
            "no byte may be written on refusal"
        );
        match env.write_canonical(&mut tiny) {
            Err(EnvelopeError::BufferTooSmall { .. }) => {}
            other => panic!("expected BufferTooSmall, got {other:?}"),
        }
        assert!(tiny.iter().all(|&b| b == 0xAA));
    }

    #[test]
    fn escaped_len_matches_writer_over_scalar_characters() {
        let mut s = String::new();
        for cp in (0u32..0x300).chain([0x7Fu32, 0x2028, 0x10FFFF]) {
            if let Some(c) = char::from_u32(cp) {
                s.push(c);
            }
        }
        let mut buf = [0u8; 8192];
        let written = {
            let mut w = SliceOut::new(&mut buf);
            w.push_escaped(&s);
            w.pos
        };
        assert_eq!(
            written,
            escaped_len(&s),
            "escaped_len drifted from the writer"
        );
        // The emitted bytes are valid UTF-8 with the quotes at both ends.
        let text = core::str::from_utf8(&buf[..written]).expect("writer emits UTF-8");
        assert!(text.starts_with('"') && text.ends_with('"'));
    }

    #[test]
    fn window_boundaries_are_inclusive() {
        let env = EnvelopeRef::from_json(V0_CANONICAL.as_bytes()).expect("decodes");
        assert!(
            env.window_live(1_700_000_000).is_ok(),
            "now == not_before is live"
        );
        assert!(
            env.window_live(4_102_444_800).is_ok(),
            "now == expires_at is live"
        );
        match env.window_live(1_699_999_999) {
            Err(EnvelopeError::NotYetValid(nb)) => assert_eq!(nb, 1_700_000_000),
            other => panic!("expected NotYetValid, got {other:?}"),
        }
        match env.window_live(4_102_444_801) {
            Err(EnvelopeError::Expired(ex)) => assert_eq!(ex, 4_102_444_800),
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    #[test]
    fn skipper_refuses_grammatically_broken_unknown_values() {
        let template = |unknown: &str| -> String {
            let mut doc = String::from("{");
            doc.push_str(unknown);
            doc.push(',');
            doc.push_str(&V0_CANONICAL[1..]);
            doc
        };
        for bad in [
            template(r#""u":[1 2]"#),     // adjacent scalars
            template(r#""u":{"a":[1}}"#), // mismatched bracket
            template(r#""u":tru}"#),      // broken literal
            template(r#""u":[1,]"#),      // trailing comma
            template(r#""u":{"a" 1}"#),   // missing colon
        ] {
            assert!(
                matches!(
                    EnvelopeRef::from_json(bad.as_bytes()),
                    Err(EnvelopeError::Malformed(_))
                ),
                "skipper admitted {bad}"
            );
        }
        // Deeply nested unknown values are bounded, not crashing.
        let mut deep = String::new();
        for _ in 0..(MAX_SKIP_DEPTH + 8) {
            deep.push('[');
        }
        for _ in 0..(MAX_SKIP_DEPTH + 8) {
            deep.push(']');
        }
        let doc = template(&deep);
        assert!(matches!(
            EnvelopeRef::from_json(doc.as_bytes()),
            Err(EnvelopeError::Malformed(_))
        ));
    }

    #[test]
    fn display_refusals_carry_the_law() {
        assert_eq!(
            EnvelopeError::NonCanonicalNumber("policy_epoch").to_string(),
            "number not canonical in `policy_epoch`: integers beyond 2^53 are not I-JSON"
        );
        assert_eq!(
            EnvelopeError::WrongVersion.to_string(),
            "unknown envelope version (expected CTP-ENVELOPE-v1)"
        );
        assert_eq!(
            EnvelopeError::BufferTooSmall {
                needed: 500,
                given: 8
            }
            .to_string(),
            "buffer too small: needed 500 bytes, given 8"
        );
    }
}
