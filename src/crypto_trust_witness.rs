//! Witness cosigning over signed transparency-log tree heads for the affidavit
//! cryptographic trust plane. Rendered by ggen sync from affidavit-trust-plane-pack
//! (the pack is authoritative; never edit this file).
//!
//! WITNESS LAW: a witness here is a REAL second local ES256 key that cosigns a
//! [`crate::crypto_trust_log::SignedTreeHead`] over the log module's OWN
//! domain-separated head pre-image
//! `digest(HEAD_SIGNING_DOMAIN, [kid, tree_size-le, head])` — the exact bytes
//! the log's signer signed (the recomputation is pinned to the log module by
//! an RFC 6979 byte-identity tooth in the tests). The witness set is admitted
//! when at least `min` cosignatures verify under DISTINCT witness kids: the
//! quorum court's independence law ([`crate::crypto_trust_quorum`]) applied to
//! heads. A witness is a real second local key; network-reachable remote
//! witnesses (cosigning pools, key-transparency style) are FUTURE WORK behind
//! the same [`WitnessSignature`] wire form — this module fakes nothing.
//!
//! Refusal-vs-verdict law (the asymmetry, documented identically to the
//! quorum's and witnessed by tests):
//! - STRUCTURAL violations refuse the whole cosignature set as typed
//!   [`WitnessError`] values — witness-count floor (`InsufficientWitnesses`),
//!   a repeated kid among cosignatures OR repeated pk-table rows
//!   (`DuplicateWitness`), a kid with no public key in the verification table
//!   (`UnknownWitness`), an UNDECODABLE cosignature (`MalformedCosignature`),
//!   and an undecodable head (`Serialization` on its `head_hex` hex). A
//!   contaminated set is never counted.
//! - A WELL-FORMED cosignature that does not verify is an observation, not a
//!   refusal: it simply does not count toward the witness threshold, and the
//!   court returns a DECIDED NEGATIVE ([`CosignVerdict`] with `cosigned:
//!   false`). A forgery is evidence about the witness set, not a parse
//!   failure.
//!
//! Threshold law: `min` is a per-call parameter; [`MIN_WITNESSES`] is the
//! graph-rendered floor ([`ctp:witness-v1`](crate) `ctp:minWitnesses`). There
//! is no graph policy ceiling for witnesses — reachability is bounded by the
//! caller's pk table: a cosignature naming a kid outside the table is refused
//! (`UnknownWitness`), so an unsatisfiable threshold is always caught
//! STRUCTURALLY (count floor or unknown kid), never decided silently.
//! `min = 0` is the vacuous quorum: any cosignature set that passes the
//! structural courts (in particular the empty set) satisfies it with zero
//! valid cosignatures — callers wanting a real threshold must pass `min >= 1`.
//!
//! House law: certify-don't-decide — a cosigned head is cryptographic
//! evidence per witness set. The plane NEVER grants authority: what a cosigned
//! head may DO remains SA2A's decision. Refusals are typed values, never
//! panics; no `unwrap`/`expect` on production paths.

// Consumed query columns (witness.rq): min_witnesses, domain_tag.

use crate::crypto_trust_canonical::digest;
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};
use crate::crypto_trust_log::{SignedTreeHead, HEAD_SIGNING_DOMAIN};
use std::collections::BTreeSet;

/// Graph-rendered witness floor (`ctp:witness-v1` `ctp:minWitnesses`). Callers
/// may pass an explicit `min` per adjudication; this is the policy's standing
/// default.
pub const MIN_WITNESSES: usize = 2;

/// Trust-plane domain tag (`ctp:policy-v1` `ctp:domainTag`). The witness court
/// never mixes it into bytes itself — cosignatures live in the LOG module's
/// own head-signing domain ([`HEAD_SIGNING_DOMAIN`], the single owner) — the
/// rendered tag is asserted equal to it by the `consts_render_pack_ontology`
/// court.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// One witness's cosignature over a [`SignedTreeHead`] pre-image: which
/// witness kid signed, with the DER (X9.62 `ECDSA-Sig-Value`) ES256 signature
/// bytes. The kid is CLAIMED here and resolved against the caller's pk table
/// by [`verify_cosigned`] — a kid the table does not carry is a typed refusal,
/// never a guess.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WitnessSignature {
    /// The witness's key id — one witness, one kid, once.
    pub kid: String,
    /// DER-encoded ES256 signature over the domain-separated head pre-image.
    pub signature: Vec<u8>,
}

/// Typed refusals of the witness court. Values, never panics. Every variant is
/// witnessed by a test; structural refusals contaminate the WHOLE set.
#[derive(Debug, thiserror::Error)]
pub enum WitnessError {
    #[error("only {got}/{needed} witness cosignatures present")]
    InsufficientWitnesses { got: usize, needed: usize },
    #[error("duplicate witness {0} — witness independence violated")]
    DuplicateWitness(String),
    #[error("witness {0} has no public key in the verification table")]
    UnknownWitness(String),
    #[error("cosignature {0} invalid")]
    MalformedCosignature(String),
    #[error("serialization: {0}")]
    Serialization(String),
}

/// The outcome of a witness adjudication: a DECISION, never an authority
/// grant. `cosigned: false` is a decided negative (well-formed cosignatures
/// that failed to verify), distinct from the typed [`WitnessError`] refusals
/// for structural violations.
#[derive(Debug, Clone, PartialEq)]
pub struct CosignVerdict {
    /// Whether the valid-witness count met the requested `min`.
    pub cosigned: bool,
    /// Cosignatures that verified under their table public keys.
    pub valid_witnesses: usize,
    /// Distinct kids among the valid cosignatures. Equal to `valid_witnesses`
    /// under the duplicate-kid court; carried as its own field so the verdict
    /// stays honest if the independence law ever widens.
    pub distinct_witnesses: usize,
}

/// The witness collector for one head: holds the [`SignedTreeHead`], exposes
/// its pre-image, and manufactures cosignatures from real witness keys.
/// Certify-don't-decide: it signs exactly what it is given and adjudicates
/// nothing — [`verify_cosigned`] is the court.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessCosign {
    head: SignedTreeHead,
}

impl WitnessCosign {
    /// Binds the collector to `head`.
    pub fn new(head: SignedTreeHead) -> Self {
        WitnessCosign { head }
    }

    /// The head being witnessed.
    pub fn head(&self) -> &SignedTreeHead {
        &self.head
    }

    /// The domain-separated head pre-image, recomputed from the head's OWN
    /// fields over the log module's own domain
    /// ([`HEAD_SIGNING_DOMAIN`]): `digest(HEAD_SIGNING_DOMAIN, [kid,
    /// tree_size-le, head])`. An undecodable `head_hex` is a typed
    /// [`WitnessError::Serialization`] — the log's wire form is never trusted
    /// without recomputation.
    pub fn preimage(&self) -> Result<[u8; 32], WitnessError> {
        head_preimage(&self.head)
    }

    /// Collects one cosignature per witness over the head pre-image, in
    /// witness order. Deterministic: RFC 6979 makes the same head bytes under
    /// the same witness key always yield the same DER signature. Manufacture,
    /// not adjudication: the witness set is signed exactly as given — a
    /// duplicated kid is data the court (not this method) refuses.
    pub fn collect(
        &self,
        witnesses: &[(&str, &Es256SigningKey)],
    ) -> Result<Vec<WitnessSignature>, WitnessError> {
        let preimage = self.preimage()?;
        Ok(witnesses
            .iter()
            .map(|(kid, key)| WitnessSignature {
                kid: (*kid).to_string(),
                signature: key.sign(&preimage),
            })
            .collect())
    }
}

/// The domain-separated ES256 pre-image for a tree head, recomputed from the
/// head's own fields: `digest(HEAD_SIGNING_DOMAIN, [kid, tree_size-le, head])`
/// — the log module's own law ([`HEAD_SIGNING_DOMAIN`] is borrowed, never
/// copied). The little-endian size and the length-prefixed parts of [`digest`]
/// keep pre-images unambiguous under concatenation.
fn head_preimage(head: &SignedTreeHead) -> Result<[u8; 32], WitnessError> {
    let head_bytes = hex_decode_32(&head.head_hex)?;
    Ok(digest(
        HEAD_SIGNING_DOMAIN,
        &[
            head.kid.as_bytes(),
            &head.tree_size.to_le_bytes(),
            &head_bytes,
        ],
    ))
}

/// Decodes exactly 64 hex characters into a 32-byte digest; anything else is a
/// typed [`WitnessError::Serialization`] — never a coercion.
fn hex_decode_32(s: &str) -> Result<[u8; 32], WitnessError> {
    let bytes = s.as_bytes();
    if bytes.len() != 64 {
        return Err(WitnessError::Serialization(format!(
            "digest hex must be 64 characters, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        let hi = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| WitnessError::Serialization(format!("invalid hex digit in {s:?}")))?;
        let lo = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| WitnessError::Serialization(format!("invalid hex digit in {s:?}")))?;
        out[i] = ((hi << 4) | lo) as u8;
    }
    Ok(out)
}

/// The witness court: adjudicates a cosignature set over `head` against the
/// threshold `min`, resolving each claimed kid against `pks` (a kid →
/// SEC1-uncompressed ES256 public key table).
///
/// Order of admission (first hit wins):
/// 1. pre-image precondition — the head's `head_hex` must decode into the 32
///    bytes the pre-image commits to (else [`WitnessError::Serialization`]):
///    no court runs over bytes that cannot exist;
/// 2. witness-count floor — `min > cosigs.len()` refuses
///    [`WitnessError::InsufficientWitnesses`] with `needed = min,
///    got = len`;
/// 3. independence court — a repeated kid among the cosignatures, or a
///    repeated kid row in `pks`, refuses [`WitnessError::DuplicateWitness`]:
///    one witness, one kid, once, on both sides of the table;
/// 4. evidence — every cosignature is verified against its TABLE public key:
///    a kid the table does not carry refuses
///    [`WitnessError::UnknownWitness`]; UNDECODABLE signature bytes refuse
///    [`WitnessError::MalformedCosignature`]; a well-formed signature that
///    does not verify simply does not count (decided);
/// 5. verdict — `valid >= min` decides `cosigned: true`; otherwise the
///    decided negative `cosigned: false`. A `min = 0` call is the vacuous
///    quorum: satisfied by any set that clears the courts, trivially by the
///    empty set (with `valid_witnesses = distinct_witnesses = 0`).
pub fn verify_cosigned(
    head: &SignedTreeHead,
    cosigs: &[WitnessSignature],
    min: usize,
    pks: &[(&str, &[u8])],
) -> Result<CosignVerdict, WitnessError> {
    // 1. Pre-image precondition: the head's own wire form must decode before
    // any cosignature is examined.
    let preimage = head_preimage(head)?;
    // 2. Witness-count floor: enough cosignatures to even reach min.
    if min > cosigs.len() {
        return Err(WitnessError::InsufficientWitnesses {
            got: cosigs.len(),
            needed: min,
        });
    }
    // 3. Independence court: one witness, one kid, once — among the
    // cosignatures AND in the pk table (a duplicated table row is ambiguous
    // evidence, never silently first-match).
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for cosig in cosigs {
        if !seen.insert(cosig.kid.as_str()) {
            return Err(WitnessError::DuplicateWitness(cosig.kid.clone()));
        }
    }
    let mut table_kids: BTreeSet<&str> = BTreeSet::new();
    for (kid, _) in pks {
        if !table_kids.insert(*kid) {
            return Err(WitnessError::DuplicateWitness((*kid).to_string()));
        }
    }
    // 4. Evidence: verify each cosignature against its table public key.
    let mut valid_witnesses = 0usize;
    let mut valid_kids: BTreeSet<&str> = BTreeSet::new();
    for cosig in cosigs {
        let Some((_, pk)) = pks.iter().find(|(kid, _)| *kid == cosig.kid) else {
            return Err(WitnessError::UnknownWitness(cosig.kid.clone()));
        };
        match verify_es256(pk, &preimage, &cosig.signature) {
            Ok(true) => {
                valid_witnesses += 1;
                valid_kids.insert(cosig.kid.as_str());
            }
            // Decided: well-formed but wrong — evidence about the set, not a
            // parse failure. It does not count.
            Ok(false) => {}
            Err(_) => return Err(WitnessError::MalformedCosignature(cosig.kid.clone())),
        }
    }
    // 5. Verdict: a decided outcome, never an authority grant.
    Ok(CosignVerdict {
        cosigned: valid_witnesses >= min,
        valid_witnesses,
        distinct_witnesses: valid_kids.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
        KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
    };
    use crate::crypto_trust_lifecycle::RevocationList;
    use crate::crypto_trust_log::TrustLogOps;
    use crate::crypto_trust_verify::{TrustPolicy, VerificationEngine};

    /// The clock used across the court (mirrors crypto_trust_log).
    const NOW: u64 = 1_700_000_500;
    /// The transparency-log signer's key id.
    const SIGNER_KID: &str = "afk1_transparency-log-ops";

    /// Builds a real ES256 signing key and its registry record (mirrors the
    /// rendered crypto_trust_log court fixture — real bodies throughout).
    fn es256_fixture(tag: u8) -> (Es256SigningKey, KeyRecord) {
        let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid scalar seed");
        let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
        let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
        let record = KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: "subject-a".to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: public,
            created_epoch: NOW - 1_000,
        };
        (signing, record)
    }

    /// A fresh, in-window envelope naming `kid`.
    /// The domain-separated digest of a subject string — the exact law
    /// `certify_signed` enforces against `env.subject_digest`.
    fn subject_bound(subject: &str) -> [u8; 32] {
        crate::crypto_trust_canonical::digest(
            crate::crypto_trust_verify::DOMAIN_TAG,
            &[subject.as_bytes()],
        )
    }

    fn envelope(kid: &KeyId, nonce: [u8; 16]) -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid.clone(),
            profile: CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch: 0,
            generation: 1,
            nonce,
            not_before: NOW - 1_000,
            expires_at: NOW + 1_000,
            subject_digest: subject_bound("subject-a"),
            audience: "affidavit.seal".to_string(),
        }
    }

    /// An engine holding `record` under the graph-default policy, clock at NOW.
    fn engine_with(record: &KeyRecord) -> VerificationEngine {
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(NOW),
        )
    }

    /// Mints a REAL valid receipt through the real verification engine.
    fn mint(tag: u8) -> crate::crypto_trust_verify::CryptoStandingReceipt {
        let (signing, record) = es256_fixture(tag);
        let env = envelope(&record.id, [tag; 16]);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        engine_with(&record)
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("VALID verdict mints a receipt")
    }

    /// The lawful leaf commitment for a receipt (the log's own binding law,
    /// recomputed independently in the court).
    fn leaf_commitment(receipt: &crate::crypto_trust_verify::CryptoStandingReceipt) -> [u8; 32] {
        blake3::hash(receipt.envelope_commitment.as_bytes()).into()
    }

    /// The log's head signer (a real, distinct key).
    fn head_signer() -> Es256SigningKey {
        Es256SigningKey::from_seed(&[98; 32]).expect("valid scalar seed")
    }

    /// A real ES256 witness key: deterministic from a distinct seed.
    fn witness(seed_byte: u8) -> Es256SigningKey {
        let mut seed = [0x77u8; 32];
        seed[0] = seed_byte;
        Es256SigningKey::from_seed(&seed).expect("valid witness scalar")
    }

    /// A REAL transparency log with `tags` receipts recorded through the real
    /// log-ops surface, and its current head published by the log module's own
    /// signer. Every head these tests see is a genuine rendered-log artifact.
    fn real_head(tags: &[u8]) -> (TrustLogOps, SignedTreeHead) {
        let mut ops = TrustLogOps::new();
        let signer = head_signer();
        for (i, tag) in tags.iter().enumerate() {
            let receipt = mint(*tag);
            ops.record(
                &receipt,
                leaf_commitment(&receipt),
                &signer,
                SIGNER_KID,
                NOW + i as u64,
            )
            .unwrap_or_else(|e| panic!("record {i} must be lawful: {e}"));
        }
        let head = ops
            .publish_head(&signer, SIGNER_KID, NOW + 100)
            .expect("publish is lawful");
        (ops, head)
    }

    // -- teeth first: the pre-image IS the log module's own law --------------

    #[test]
    fn graph_consts_are_rendered() {
        assert_eq!(MIN_WITNESSES, 2);
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        // Single-owner law: cosignatures live in the LOG module's head-signing
        // domain — the graph tag and the log const must be the same string,
        // and both the canonical module's DOMAIN_TAG.
        assert_eq!(DOMAIN_TAG, HEAD_SIGNING_DOMAIN);
        assert_eq!(
            HEAD_SIGNING_DOMAIN,
            crate::crypto_trust_canonical::DOMAIN_TAG
        );
    }

    #[test]
    fn witness_preimage_reproduces_the_log_module_s_own_signature() {
        let (_ops, head) = real_head(&[41, 42]);
        let signer = head_signer();

        // Recomputing the pre-image from the head's OWN fields and signing it
        // with the head's own key must reproduce the stored signature BYTE FOR
        // BYTE (RFC 6979 determinism): the witness court signs exactly the
        // log module's pre-image or this test dies.
        let cosign = WitnessCosign::new(head.clone());
        let preimage = cosign.preimage().expect("head decodes");
        assert_eq!(
            signer.sign(&preimage),
            head.signature,
            "the recomputed pre-image must be the log module's own"
        );

        // The log module's own verifier agrees on the same bytes.
        assert!(matches!(
            crate::crypto_trust_log::verify_head(&head, &signer.public_key_sec1()),
            Ok(true)
        ));

        // Every field is bound in: tampering the kid (or any head field)
        // changes the recomputed pre-image.
        let mut tampered = head.clone();
        tampered.kid = "afk1_someone_else".to_string();
        assert_ne!(
            WitnessCosign::new(tampered).preimage().expect("decodes"),
            preimage,
            "the kid is bound into the pre-image"
        );
        let mut tampered = head.clone();
        tampered.tree_size += 1;
        assert_ne!(
            WitnessCosign::new(tampered).preimage().expect("decodes"),
            preimage,
            "the tree size is bound into the pre-image"
        );
    }

    // -- teeth: 2-of-2 real witnesses cosign and verify -----------------------

    #[test]
    fn two_of_two_real_witnesses_cosign_and_verify() {
        let (_ops, head) = real_head(&[41, 42, 43]);
        let a = witness(0xA1);
        let b = witness(0xB2);
        let c = witness(0xC3);
        let rows = [
            ("witness-a".to_string(), a.public_key_sec1()),
            ("witness-b".to_string(), b.public_key_sec1()),
            ("witness-c".to_string(), c.public_key_sec1()),
        ];
        let table: Vec<(&str, &[u8])> = rows
            .iter()
            .map(|(kid, pk)| (kid.as_str(), pk.as_slice()))
            .collect();

        // THE TOOTH: two real distinct witness keys cosign the REAL head;
        // the court admits the set at the graph floor.
        let cosign = WitnessCosign::new(head.clone());
        let cosigs = cosign
            .collect(&[("witness-a", &a), ("witness-b", &b)])
            .expect("two real witnesses cosign");
        let verdict =
            verify_cosigned(&head, &cosigs, MIN_WITNESSES, &table).expect("structural courts pass");
        assert_eq!(
            verdict,
            CosignVerdict {
                cosigned: true,
                valid_witnesses: 2,
                distinct_witnesses: 2,
            }
        );

        // A larger set still satisfies the same floor; min = n satisfies too,
        // and re-adjudication decides identically.
        let three = cosign
            .collect(&[("witness-a", &a), ("witness-b", &b), ("witness-c", &c)])
            .expect("three real witnesses cosign");
        let wider =
            verify_cosigned(&head, &three, MIN_WITNESSES, &table).expect("structural courts pass");
        assert_eq!(wider.valid_witnesses, 3);
        let full = verify_cosigned(&head, &three, 3, &table).expect("structural courts pass");
        let again = verify_cosigned(&head, &three, 3, &table).expect("structural courts pass");
        assert_eq!(again, full, "re-adjudication must decide identically");
        assert!(full.cosigned);
        assert_eq!(full.distinct_witnesses, 3);
    }

    // -- every WitnessError variant witnessed ---------------------------------

    #[test]
    fn insufficient_witnesses_variant_witnessed_with_exact_counts() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let pk_a = a.public_key_sec1();
        let table = [("witness-a", pk_a.as_slice())];
        let cosigs = WitnessCosign::new(head.clone())
            .collect(&[("witness-a", &a)])
            .expect("collect is lawful");

        match verify_cosigned(&head, &cosigs, 2, &table) {
            Err(WitnessError::InsufficientWitnesses { got, needed }) => {
                assert_eq!((got, needed), (1, 2));
            }
            outcome => panic!("expected InsufficientWitnesses, got {outcome:?}"),
        }
    }

    #[test]
    fn duplicate_witness_variant_witnessed_same_kid_twice() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let b = witness(0xB2);
        let pk_a = a.public_key_sec1();
        let table = [("witness-a", pk_a.as_slice())];
        let preimage = WitnessCosign::new(head.clone())
            .preimage()
            .expect("head decodes");

        // The SAME kid twice — the second cosignature even carries a DIFFERENT
        // real signer's signature: exactly the padding the independence court
        // must refuse before any signature is examined.
        let cosigs = [
            WitnessSignature {
                kid: "witness-a".to_string(),
                signature: a.sign(&preimage),
            },
            WitnessSignature {
                kid: "witness-a".to_string(),
                signature: b.sign(&preimage),
            },
        ];
        match verify_cosigned(&head, &cosigs, 2, &table) {
            Err(WitnessError::DuplicateWitness(kid)) => assert_eq!(kid, "witness-a"),
            outcome => panic!("expected DuplicateWitness, got {outcome:?}"),
        }

        // The same law on the evidence side: a duplicated pk-table row is
        // ambiguous evidence and refuses too.
        let cosigs = [WitnessSignature {
            kid: "witness-a".to_string(),
            signature: a.sign(&preimage),
        }];
        let doubled_table = [
            ("witness-a", pk_a.as_slice()),
            ("witness-a", pk_a.as_slice()),
        ];
        match verify_cosigned(&head, &cosigs, 1, &doubled_table) {
            Err(WitnessError::DuplicateWitness(kid)) => assert_eq!(kid, "witness-a"),
            outcome => panic!("expected DuplicateWitness for the table, got {outcome:?}"),
        }
    }

    #[test]
    fn unknown_witness_variant_witnessed_kid_without_public_key() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let ghost = witness(0x66);
        let pk_a = a.public_key_sec1();
        let table = [("witness-a", pk_a.as_slice())];
        let preimage = WitnessCosign::new(head.clone())
            .preimage()
            .expect("head decodes");

        // A real signature from a real key the table has never seen.
        let cosigs = [WitnessSignature {
            kid: "witness-ghost".to_string(),
            signature: ghost.sign(&preimage),
        }];
        match verify_cosigned(&head, &cosigs, 1, &table) {
            Err(WitnessError::UnknownWitness(kid)) => assert_eq!(kid, "witness-ghost"),
            outcome => panic!("expected UnknownWitness, got {outcome:?}"),
        }
    }

    #[test]
    fn malformed_cosignature_variant_witnessed_undecodable_bytes() {
        let (_ops, head) = real_head(&[41]);
        let pk_a = witness(0xA1).public_key_sec1();
        let table = [("witness-a", pk_a.as_slice())];

        // Bytes that are not DER ECDSA-Sig-Value.
        let cosigs = [WitnessSignature {
            kid: "witness-a".to_string(),
            signature: vec![0xFF, 0x00],
        }];
        match verify_cosigned(&head, &cosigs, 1, &table) {
            Err(WitnessError::MalformedCosignature(kid)) => assert_eq!(kid, "witness-a"),
            outcome => panic!("expected MalformedCosignature, got {outcome:?}"),
        }

        // Empty bytes: same typed refusal.
        let cosigs = [WitnessSignature {
            kid: "witness-a".to_string(),
            signature: Vec::new(),
        }];
        match verify_cosigned(&head, &cosigs, 1, &table) {
            Err(WitnessError::MalformedCosignature(_)) => {}
            outcome => panic!("expected MalformedCosignature, got {outcome:?}"),
        }
    }

    #[test]
    fn serialization_variant_witnessed_unusable_head_hex() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let pk_a = a.public_key_sec1();
        let table = [("witness-a", pk_a.as_slice())];

        // Non-hex head hex: the typed Serialization refusal before any court.
        let mut broken = head.clone();
        broken.head_hex = "zz".repeat(32);
        match verify_cosigned(&broken, &[], 0, &table) {
            Err(WitnessError::Serialization(msg)) => {
                assert!(msg.contains("invalid hex digit"), "{msg}");
            }
            outcome => panic!("expected Serialization, got {outcome:?}"),
        }

        // Wrong length: the same typed refusal.
        let mut short = head.clone();
        short.head_hex = "abcd".to_string();
        match verify_cosigned(&short, &[], 0, &table) {
            Err(WitnessError::Serialization(msg)) => {
                assert!(msg.contains("must be 64 characters"), "{msg}");
            }
            outcome => panic!("expected Serialization, got {outcome:?}"),
        }

        // The collector refuses the same head the same way.
        match WitnessCosign::new(broken).preimage() {
            Err(WitnessError::Serialization(_)) => {}
            outcome => panic!("expected Serialization, got {outcome:?}"),
        }
    }

    // -- the decided negative: well-formed-but-wrong does not refuse ----------

    #[test]
    fn decided_negative_one_of_two_cosignatures_is_a_forgery_not_a_refusal() {
        let (_ops, head) = real_head(&[41, 42]);
        let a = witness(0xA1);
        let b = witness(0xB2);
        let rows = [
            ("witness-a".to_string(), a.public_key_sec1()),
            ("witness-b".to_string(), b.public_key_sec1()),
        ];
        let table: Vec<(&str, &[u8])> = rows
            .iter()
            .map(|(kid, pk)| (kid.as_str(), pk.as_slice()))
            .collect();
        let preimage = WitnessCosign::new(head.clone())
            .preimage()
            .expect("head decodes");

        // An ATTACKER's key signs well-formed DER, but the cosignature claims
        // witness-b: verifiable shape, wrong key. It must NOT refuse — it
        // just does not count.
        let attacker = witness(0xEE);
        let cosigs = [
            WitnessSignature {
                kid: "witness-a".to_string(),
                signature: a.sign(&preimage),
            },
            WitnessSignature {
                kid: "witness-b".to_string(),
                signature: attacker.sign(&preimage),
            },
        ];
        let verdict =
            verify_cosigned(&head, &cosigs, MIN_WITNESSES, &table).expect("structural courts pass");
        assert_eq!(
            verdict,
            CosignVerdict {
                cosigned: false,
                valid_witnesses: 1,
                distinct_witnesses: 1,
            },
            "a well-formed forgery is a decided negative, not a refusal"
        );
    }

    // -- the vacuous threshold ------------------------------------------------

    #[test]
    fn min_zero_is_the_vacuous_witness_quorum() {
        let (_ops, head) = real_head(&[41]);
        let verdict = verify_cosigned(&head, &[], 0, &[]).expect("min=0 clears the courts");
        assert_eq!(
            verdict,
            CosignVerdict {
                cosigned: true,
                valid_witnesses: 0,
                distinct_witnesses: 0,
            }
        );
    }

    // -- collection is deterministic manufacture ------------------------------

    #[test]
    fn collect_is_deterministic_and_preserves_witness_order() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let b = witness(0xB2);
        let cosign = WitnessCosign::new(head.clone());
        let preimage = cosign.preimage().expect("head decodes");

        let one = cosign
            .collect(&[("witness-a", &a), ("witness-b", &b)])
            .expect("collect is lawful");
        let two = cosign
            .collect(&[("witness-a", &a), ("witness-b", &b)])
            .expect("collect is lawful");
        assert_eq!(one, two, "RFC 6979 makes collection deterministic");
        assert_eq!(one.len(), 2);
        assert_eq!(one[0].kid, "witness-a");
        assert_eq!(one[1].kid, "witness-b");
        assert_eq!(one[0].signature, a.sign(&preimage));
        assert_eq!(one[1].signature, b.sign(&preimage));

        // The collector carries the head verbatim.
        assert_eq!(cosign.head(), &head);
    }

    // -- wire form ------------------------------------------------------------

    #[test]
    fn witness_signature_serde_round_trips() {
        let (_ops, head) = real_head(&[41]);
        let a = witness(0xA1);
        let cosign = WitnessCosign::new(head);
        let cosig = &cosign
            .collect(&[("witness-a", &a)])
            .expect("collect is lawful")[0];
        let json = serde_json::to_string(cosig).expect("cosignature serializes");
        let back: WitnessSignature = serde_json::from_str(&json).expect("cosignature parses");
        assert_eq!(&back, cosig);
    }

    // -- error display strings are wire stable ----------------------------------

    #[test]
    fn error_display_strings_are_wire_stable() {
        assert_eq!(
            WitnessError::InsufficientWitnesses { got: 1, needed: 2 }.to_string(),
            "only 1/2 witness cosignatures present"
        );
        assert_eq!(
            WitnessError::DuplicateWitness("w".to_string()).to_string(),
            "duplicate witness w — witness independence violated"
        );
        assert_eq!(
            WitnessError::UnknownWitness("w".to_string()).to_string(),
            "witness w has no public key in the verification table"
        );
        assert_eq!(
            WitnessError::MalformedCosignature("w".to_string()).to_string(),
            "cosignature w invalid"
        );
        assert_eq!(
            WitnessError::Serialization("head".to_string()).to_string(),
            "serialization: head"
        );
    }
}
