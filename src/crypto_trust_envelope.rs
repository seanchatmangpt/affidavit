//! Signature envelope and nonce/replay evidence for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! RFC-SA2A-007-errata E-E: the algorithm identity, key id, and nonce — and the
//! policy/revocation epochs that gate them — live INSIDE the signed bytes
//! (graph-ordered fields 2 algorithm, 3 key_id, 5 policy_epoch, 6
//! revocation_epoch, 8 nonce). They are never carried in an unsigned header: a
//! signature over bytes that omit them would not bind them, so a verifier
//! refuses any envelope whose transport stripped them.
//!
//! Domain separation: [`crate::crypto_trust_canonical::DOMAIN_TAG`] — rendered
//! from the same ctp:policy-v1 ctp:domainTag this module's policy row binds —
//! is mixed into every signing pre-image. This module deliberately does not
//! re-declare it: the canonical module is the single owner.
//!
//! House law: refusal-as-value. Every refusal is a typed [`EnvelopeError`]
//! variant; nothing here panics, coerces, or fakes.

use crate::crypto_trust_canonical::{domain_separated, jcs, DOMAIN_TAG};
use crate::crypto_trust_keys::{AlgorithmId, CryptoProfile, KeyId};
use std::collections::BTreeMap;

/// Signed-envelope version (ctp:policy-v1 ctp:envelopeVersion). `from_bytes`
/// refuses any document whose `version` field differs, typed
/// [`EnvelopeError::WrongVersion`].
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// The signed-bytes field registry, rendered from the graph's
/// ctp:SignatureEnvelopeField individuals as (ctp:fieldName, ctp:fieldOrder)
/// in canonical order. This registry — not a hand-typed list — is the
/// authority the field-set test holds [`SignatureEnvelope::envelope_document`]
/// to, so a graph-side field change breaks the render loudly instead of
/// drifting silently.
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

/// The RFC-SA2A-007-errata signature envelope: the twelve graph-declared
/// fields in declaration order. The struct field NAMES are the ctp:fieldName
/// values; the field-set test holds the derived wire form to
/// [`ENVELOPE_FIELDS`], and the algorithm/profile wire forms are owned by the
/// keys module's serde attributes — this module keeps no second copy of them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignatureEnvelope {
    pub version: String,
    pub algorithm: AlgorithmId,
    pub key_id: KeyId,
    pub profile: CryptoProfile,
    pub policy_epoch: u64,
    pub revocation_epoch: u64,
    pub generation: u32,
    pub nonce: [u8; 16],
    pub not_before: u64,
    pub expires_at: u64,
    pub subject_digest: [u8; 32],
    pub audience: String,
}

/// Typed refusal of an envelope operation; refusal-as-value, never a panic.
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    #[error("malformed envelope: {0}")]
    Malformed(String),
    #[error("envelope expired at {0}")]
    Expired(u64),
    #[error("envelope not valid before {0}")]
    NotYetValid(u64),
    #[error("nonce replay for key {0}")]
    ReplayRejected(String),
    #[error("unknown envelope version {0}")]
    WrongVersion(String),
}

/// Lowercase-hex encoding for journal keys and nothing else (document bytes
/// are JSON integers, not hex — the canonical module owns byte canonicality).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Journal key for the graph's replay tuple (ctp:nonce-policy-v1
/// ctp:replayKey = "kid,nonce"): `kid + ":" + lowercase_hex(nonce)`. The
/// suffix is unambiguous — lowercase hex digits and `:` are disjoint
/// alphabets and the suffix has fixed length 33 — so distinct (kid, nonce)
/// pairs can never produce the same key.
fn nonce_journal_key(kid: &str, nonce: &[u8; 16]) -> String {
    format!("{kid}:{}", hex_encode(nonce))
}

impl SignatureEnvelope {
    /// The canonical envelope document: a JSON object whose keys are exactly
    /// the twelve graph-declared field names ([`ENVELOPE_FIELDS`]). The
    /// derived serialization of `Self` IS the document, so the wire forms of
    /// `algorithm`/`profile` come from the keys module's serde attributes and
    /// the 16/32-byte binary fields serialize as JSON arrays of integers —
    /// the same shape [`SignatureEnvelope::from_bytes`] parses.
    pub fn envelope_document(&self) -> serde_json::Value {
        // `serde_json::to_value` is total for this field set (strings,
        // unit-variant enums, integers, byte arrays — every Serialize impl is
        // infallible), so the fallback is structurally unreachable. The typed
        // mapping of a failure is observable through the fallible paths
        // [`SignatureEnvelope::to_bytes`] and
        // [`SignatureEnvelope::signing_input_checked`], which refuse with
        // [`EnvelopeError::Malformed`] before any fallback value could reach
        // signed bytes.
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    fn document_checked(&self) -> Result<serde_json::Value, EnvelopeError> {
        serde_json::to_value(self).map_err(|e| EnvelopeError::Malformed(e.to_string()))
    }

    /// The exact signing pre-image:
    /// `domain_separated(DOMAIN_TAG, [jcs(envelope_document)])`. Deterministic:
    /// `PartialEq`-equal envelopes produce byte-identical pre-images (witnessed
    /// by test). Refuses [`EnvelopeError::Malformed`] when the document cannot
    /// canonicalize — reachable when any integer field exceeds 2^53 (JCS
    /// refuses non-I-JSON integers rather than coercing their value).
    pub fn signing_input_checked(&self) -> Result<Vec<u8>, EnvelopeError> {
        let canonical =
            jcs(&self.document_checked()?).map_err(|e| EnvelopeError::Malformed(e.to_string()))?;
        Ok(domain_separated(DOMAIN_TAG, &[canonical.as_bytes()]))
    }

    /// Infallible form of [`SignatureEnvelope::signing_input_checked`] for
    /// pipelines that have already admitted the envelope through
    /// [`SignatureEnvelope::to_bytes`]. For an envelope whose document cannot
    /// canonicalize this yields the empty pre-image — a value no verifier will
    /// accept, and unreachable in lawful pipelines because `to_bytes` refuses
    /// first with [`EnvelopeError::Malformed`].
    pub fn signing_input(&self) -> Vec<u8> {
        self.signing_input_checked().unwrap_or_default()
    }

    /// Canonical signed bytes: JCS of [`SignatureEnvelope::envelope_document`]
    /// (including the `version` field). Deterministic: byte-identical for
    /// `PartialEq`-equal envelopes. Refuses [`EnvelopeError::Malformed`] when
    /// the document cannot canonicalize (any integer field beyond 2^53).
    pub fn to_bytes(&self) -> Result<Vec<u8>, EnvelopeError> {
        let canonical =
            jcs(&self.document_checked()?).map_err(|e| EnvelopeError::Malformed(e.to_string()))?;
        Ok(canonical.into_bytes())
    }

    /// Parse and admit canonical signed bytes. Refusals, typed:
    /// - [`EnvelopeError::Malformed`] — not JSON, not the envelope shape, or a
    ///   `nonce`/`subject_digest` array whose length serde rejects (16/32);
    /// - [`EnvelopeError::WrongVersion`] — `version` differs from
    ///   [`ENVELOPE_VERSION`]; the payload carries the refused version.
    pub fn from_bytes(b: &[u8]) -> Result<Self, EnvelopeError> {
        let value: serde_json::Value =
            serde_json::from_slice(b).map_err(|e| EnvelopeError::Malformed(e.to_string()))?;
        let envelope: SignatureEnvelope =
            serde_json::from_value(value).map_err(|e| EnvelopeError::Malformed(e.to_string()))?;
        if envelope.version != ENVELOPE_VERSION {
            return Err(EnvelopeError::WrongVersion(envelope.version));
        }
        Ok(envelope)
    }

    /// Admit the envelope's validity window at instant `now` (seconds since
    /// the trust-plane epoch). Boundaries are inclusive: `now == not_before`
    /// is live and `now == expires_at` is live. Refusals, typed:
    /// [`EnvelopeError::NotYetValid`] carries the not_before instant,
    /// [`EnvelopeError::Expired`] carries the expires_at instant.
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

/// Replay-evidence journal over the graph's replay tuple
/// (ctp:nonce-policy-v1 ctp:replayKey = "kid,nonce"). Internal key:
/// `kid + ":" + lowercase_hex(nonce)` (see [`nonce_journal_key`] — collision-
/// free by construction).
///
/// The window is half-open `[first_seen, first_seen + window_seconds)`: a
/// repeat strictly inside the window is refused
/// ([`EnvelopeError::ReplayRejected`]); a repeat at or beyond the boundary
/// replaces the entry (freshness restamp) and is admitted. The journal grows
/// only with distinct (kid, nonce) pairs; callers bound it with
/// [`NonceJournal::prune`] — eviction is explicit and counted, never silent.
#[derive(Debug, Default)]
pub struct NonceJournal {
    seen: BTreeMap<String, (String, u64)>,
}

impl NonceJournal {
    /// Record sight of `(kid, nonce)` at instant `at`. Returns
    /// [`EnvelopeError::ReplayRejected`] carrying `kid` when the pair was
    /// already seen strictly inside the acceptance window
    /// (`at - seen_at < window_seconds`); at or beyond the boundary the entry
    /// is replaced and the record is admitted. The journal never prunes
    /// implicitly — call [`NonceJournal::prune`].
    pub fn record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) -> Result<(), EnvelopeError> {
        let key = nonce_journal_key(kid, &nonce);
        if let Some(entry) = self.seen.get(&key) {
            if at.saturating_sub(entry.1) < window_seconds {
                return Err(EnvelopeError::ReplayRejected(kid.to_string()));
            }
        }
        self.seen.insert(key, (kid.to_string(), at));
        Ok(())
    }

    /// The instant `(kid, nonce)` was last recorded, if present.
    pub fn seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> {
        self.seen.get(&nonce_journal_key(kid, nonce)).map(|e| e.1)
    }

    /// Evict every entry whose recorded instant lies at or beyond the window
    /// boundary (`now - seen_at >= window_seconds`); returns the number of
    /// entries evicted. The boundary matches [`NonceJournal::record`]: an
    /// entry pruned at `now` is exactly an entry whose re-record at `now`
    /// would be admitted.
    pub fn prune(&mut self, now: u64, window_seconds: u64) -> usize {
        let before = self.seen.len();
        self.seen
            .retain(|_, entry| now.saturating_sub(entry.1) < window_seconds);
        before - self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_envelope() -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::MlDsa65,
            key_id: KeyId("k-test-1".to_string()),
            profile: CryptoProfile::Pqc,
            policy_epoch: 7,
            revocation_epoch: 3,
            generation: 2,
            nonce: [0x10; 16],
            not_before: 1_000,
            expires_at: 2_000,
            subject_digest: [0x22; 32],
            audience: "affidavit.verifier".to_string(),
        }
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
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
    fn document_keys_equal_graph_field_registry() {
        let doc = test_envelope().envelope_document();
        let obj = doc.as_object().expect("document is an object");
        let names: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
        let registry: std::collections::BTreeSet<&str> =
            ENVELOPE_FIELDS.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, registry);
        assert_eq!(obj.len(), 12);
        let mut orders: Vec<u32> = ENVELOPE_FIELDS.iter().map(|(_, o)| *o).collect();
        orders.sort_unstable();
        assert_eq!(orders, Vec::from_iter(1u32..=12));
    }

    #[test]
    fn envelope_round_trips_through_canonical_bytes() {
        let e = test_envelope();
        let bytes = e.to_bytes().expect("envelope canonicalizes");
        let back = SignatureEnvelope::from_bytes(&bytes).expect("round-trip parses");
        assert_eq!(back, e);
        // Byte-identical replay of the canonical form.
        assert_eq!(e.to_bytes().expect("deterministic"), bytes);
    }

    #[test]
    fn signing_input_is_deterministic_and_domain_separated() {
        let a = test_envelope();
        let b = test_envelope();
        assert_eq!(a.signing_input(), b.signing_input());

        // Pin the exact layout against the canonical module primitives:
        // domain_separated(DOMAIN_TAG, [jcs(document)]).
        let canonical = jcs(&a.envelope_document()).expect("document canonicalizes");
        let expected = domain_separated(DOMAIN_TAG, &[canonical.as_bytes()]);
        assert_eq!(a.signing_input(), expected);
        assert_eq!(
            a.signing_input_checked().expect("pre-image canonicalizes"),
            expected
        );

        // A different domain tag must produce different bytes.
        assert_ne!(
            a.signing_input(),
            domain_separated("other.domain", &[canonical.as_bytes()])
        );
    }

    #[test]
    fn signing_input_changes_when_any_field_changes() {
        type Mutation = fn(SignatureEnvelope) -> SignatureEnvelope;
        let base = test_envelope();
        let base_input = base.signing_input();
        let base_bytes = base.to_bytes().expect("base canonicalizes");
        let mutations: [(&str, Mutation); 12] = [
            ("version", |mut e| {
                e.version = "CTP-ENVELOPE-v1-test".to_string();
                e
            }),
            ("algorithm", |mut e| {
                e.algorithm = AlgorithmId::SlhDsa128s;
                e
            }),
            ("key_id", |mut e| {
                e.key_id = KeyId("k-other".to_string());
                e
            }),
            ("profile", |mut e| {
                e.profile = CryptoProfile::Hybrid;
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
                e.audience = "affidavit.other".to_string();
                e
            }),
        ];
        assert_eq!(mutations.len(), ENVELOPE_FIELDS.len());
        for (field, mutate) in mutations {
            let mutated = mutate(test_envelope());
            assert_ne!(mutated, base, "{field} mutation must change the value");
            assert_ne!(
                mutated.signing_input(),
                base_input,
                "{field} mutation must change the signing pre-image"
            );
            assert_ne!(
                mutated.to_bytes().expect("mutated canonicalizes"),
                base_bytes,
                "{field} mutation must change the canonical bytes"
            );
        }
    }

    #[test]
    fn window_boundaries_are_inclusive() {
        let e = test_envelope(); // not_before 1_000, expires_at 2_000
        assert!(e.window_live(1_000).is_ok(), "now == not_before is live");
        assert!(e.window_live(2_000).is_ok(), "now == expires_at is live");
        match e.window_live(999) {
            Err(EnvelopeError::NotYetValid(nb)) => assert_eq!(nb, 1_000),
            other => panic!("expected NotYetValid(1000), got {other:?}"),
        }
        match e.window_live(2_001) {
            Err(EnvelopeError::Expired(ex)) => assert_eq!(ex, 2_000),
            other => panic!("expected Expired(2000), got {other:?}"),
        }
    }

    #[test]
    fn wrong_version_refused_by_variant_with_payload() {
        let mut e = test_envelope();
        e.version = "CTP-ENVELOPE-v0".to_string();
        let bytes = e.to_bytes().expect("v0 document still canonicalizes");
        match SignatureEnvelope::from_bytes(&bytes) {
            Err(EnvelopeError::WrongVersion(v)) => assert_eq!(v, "CTP-ENVELOPE-v0"),
            other => panic!("expected WrongVersion, got {other:?}"),
        }
    }

    #[test]
    fn malformed_bytes_refused_by_variant() {
        match SignatureEnvelope::from_bytes(b"{\"version\": not-json") {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed for non-JSON, got {other:?}"),
        }
        match SignatureEnvelope::from_bytes(b"{}") {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed for missing fields, got {other:?}"),
        }
    }

    #[test]
    fn malformed_nonce_length_refused_by_serde() {
        let mut doc = test_envelope().envelope_document();
        doc["nonce"] = serde_json::json!(vec![1u8; 15]);
        let bytes = serde_json::to_vec(&doc).expect("document serializes");
        match SignatureEnvelope::from_bytes(&bytes) {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed for 15-byte nonce, got {other:?}"),
        }
        doc["nonce"] = serde_json::json!(vec![1u8; 17]);
        let bytes = serde_json::to_vec(&doc).expect("document serializes");
        match SignatureEnvelope::from_bytes(&bytes) {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed for 17-byte nonce, got {other:?}"),
        }
    }

    #[test]
    fn non_canonical_integer_refused_malformed() {
        let mut e = test_envelope();
        e.policy_epoch = 9_007_199_254_740_993; // 2^53 + 1: beyond double precision
        match e.to_bytes() {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed for non-I-JSON epoch, got {other:?}"),
        }
        match e.signing_input_checked() {
            Err(EnvelopeError::Malformed(_)) => {}
            other => panic!("expected Malformed from checked pre-image, got {other:?}"),
        }
        // The infallible form refuses with the empty pre-image — never wrong
        // bytes.
        assert!(e.signing_input().is_empty());
        // Boundary: exactly 2^53 is exactly representable and canonicalizes.
        let mut b = test_envelope();
        b.policy_epoch = 9_007_199_254_740_992;
        assert!(b.to_bytes().is_ok());
    }

    #[test]
    fn replay_within_window_refused_by_variant() {
        let mut journal = NonceJournal::default();
        let nonce = [9u8; 16];
        journal
            .record("kid-a", nonce, 1_000, 300)
            .expect("first sight admitted");
        assert_eq!(journal.seen("kid-a", &nonce), Some(1_000));
        match journal.record("kid-a", nonce, 1_100, 300) {
            Err(EnvelopeError::ReplayRejected(kid)) => assert_eq!(kid, "kid-a"),
            other => panic!("expected ReplayRejected, got {other:?}"),
        }
        // The refused attempt must not restamp the entry.
        assert_eq!(journal.seen("kid-a", &nonce), Some(1_000));
    }

    #[test]
    fn replay_at_window_boundary_replaces_entry() {
        let mut journal = NonceJournal::default();
        let nonce = [8u8; 16];
        journal
            .record("kid-a", nonce, 1_000, 300)
            .expect("first sight admitted");
        // delta == window_seconds: at the boundary, admitted, entry restamped.
        journal
            .record("kid-a", nonce, 1_300, 300)
            .expect("boundary repeat admitted");
        assert_eq!(journal.seen("kid-a", &nonce), Some(1_300));
        // And the window now anchors at the new instant.
        match journal.record("kid-a", nonce, 1_400, 300) {
            Err(EnvelopeError::ReplayRejected(_)) => {}
            other => panic!("expected replay against restamped window, got {other:?}"),
        }
    }

    #[test]
    fn journal_keys_are_collision_free_across_kids() {
        let mut journal = NonceJournal::default();
        let n1 = [1u8; 16];
        let n2 = [2u8; 16];
        journal.record("a", n1, 1_000, 300).expect("admitted");
        journal.record("a:b", n1, 1_000, 300).expect("admitted");
        journal.record("a", n2, 1_000, 300).expect("admitted");
        assert_eq!(journal.seen("a", &n1), Some(1_000));
        assert_eq!(journal.seen("a:b", &n1), Some(1_000));
        assert_eq!(journal.seen("a", &n2), Some(1_000));
        // Refusal is per (kid, nonce): refusing one pair leaves the others
        // independently admissible.
        assert!(matches!(
            journal.record("a", n1, 1_100, 300),
            Err(EnvelopeError::ReplayRejected(_))
        ));
        journal
            .record("zzz", n1, 1_100, 300)
            .expect("unrelated kid admitted");
    }

    #[test]
    fn prune_evicts_exactly() {
        let mut journal = NonceJournal::default();
        journal
            .record("a", [1u8; 16], 1_000, 300)
            .expect("admitted");
        journal
            .record("b", [2u8; 16], 1_200, 300)
            .expect("admitted");
        journal
            .record("c", [3u8; 16], 1_450, 300)
            .expect("admitted");
        // now = 1_500, window 300: keep delta < 300.
        // a: 500 -> evict; b: 300 (boundary) -> evict; c: 50 -> keep.
        assert_eq!(journal.prune(1_500, 300), 2);
        assert_eq!(journal.seen("a", &[1u8; 16]), None);
        assert_eq!(journal.seen("b", &[2u8; 16]), None);
        assert_eq!(journal.seen("c", &[3u8; 16]), Some(1_450));
        // A second prune is a no-op.
        assert_eq!(journal.prune(1_500, 300), 0);
    }
}
