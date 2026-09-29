//! Signed revocation publication (CTP-CRL-v1) for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Publication law: revocation state is verifier-local until it is published.
//! [`publish`] mirrors the trust plane's revocation list into a
//! [`SignedRevocationList`], signs the JCS (RFC 8785) canonicalization of the
//! publication minus its `signature` field with the issuer's ES256 key, and
//! stamps the revocation epoch it was made under. Any party holding the
//! issuer's SEC1 public key can [`verify_publication`] the bytes and
//! [`apply_to`] them: apply verifies the signature FIRST, then the epoch
//! freshness law (the publication's epoch may lag the verifier's current
//! epoch by at most the staleness bound — the same grace
//! [`MAX_REVOCATION_STALENESS_SECONDS`] gives signatures, per the lifecycle
//! E-J law), then merges every record. A refusal is atomic: an apply that
//! returns `Err` leaves the target list byte-for-byte unchanged.
//!
//! Determinism law: the same list, key, epoch, and instant always render the
//! same publication bytes — RFC 6979 makes the signature deterministic and
//! the record mirror is sorted by `kid`, so whole-struct equality is a test.
//!
//! House law: refusals are typed values ([`RevocationPubError`]), never
//! panics; no `unwrap`/`expect` on production paths.

// Consumed query columns (revocation.rq): crl_format, this_epoch, domain_tag.

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};
use crate::crypto_trust_keys::KeyId;
use crate::crypto_trust_lifecycle::RevocationList;

/// Publication wire-format identity
/// (`ctp:revocation-pub-v1` `ctp:publicationFormat`).
pub const CRL_FORMAT: &str = "CTP-CRL-v1";

/// The revocation epoch this format counts from
/// (`ctp:revocation-pub-v1` `ctp:thisEpoch`); live publications stamp
/// `max(revoked_at) + 1` per the lifecycle revocation-epoch law.
pub const THIS_EPOCH: u64 = 0;

/// Trust-plane domain tag (`ctp:policy-v1` `ctp:domainTag`). Carried so the
/// publication states the plane it serves; the signed preimage is the
/// publication's own JCS bytes.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// One mirrored revocation record: the publishable projection of the
/// lifecycle's per-key revocation (kid, when it died, why).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RevocationRecordMirror {
    pub kid: String,
    pub revoked_at: u64,
    pub reason: String,
}

/// A signed, publishable revocation list (CTP-CRL-v1).
///
/// The signature covers the JCS canonicalization of every other field: the
/// preimage is this struct with `signature` removed, canonicalized per
/// [`crate::crypto_trust_canonical`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SignedRevocationList {
    pub format: String,
    pub issuer_kid: String,
    pub epoch: u64,
    pub published_at: u64,
    pub revoked: Vec<RevocationRecordMirror>,
    pub signature: Vec<u8>,
}

/// Typed refusal of a revocation-publication transition. Refusals are data:
/// every variant carries the evidence a verifier needs to act on the denial.
#[derive(Debug, thiserror::Error)]
pub enum RevocationPubError {
    /// The publication does not declare the CTP-CRL-v1 wire format.
    #[error("bad publication format {0:?}: expected {CRL_FORMAT}")]
    BadFormat(String),
    /// The issuer signature is malformed or does not verify over the
    /// publication's canonical preimage.
    #[error("publication signature does not verify")]
    BadSignature,
    /// The publication is stale: its epoch (or signing instant) lags the
    /// verifier's state beyond the staleness grace.
    #[error(
        "stale publication: epoch {published} vs current {current}, max staleness {max_staleness}"
    )]
    StaleEpoch {
        published: u64,
        current: u64,
        max_staleness: u64,
    },
    /// Canonicalization or (de)serialization of the publication failed.
    #[error("serialization: {0}")]
    Serialization(String),
    /// The local revocation list could not be mirrored through its public
    /// surface.
    #[error("lifecycle: {0}")]
    Lifecycle(String),
}

/// The JCS-canonicalized preimage of a publication: the struct minus its
/// `signature` field. Field names come from the serde projection itself, so
/// the signed bytes and the wire bytes cannot drift apart.
fn publication_preimage(crl: &SignedRevocationList) -> Result<Vec<u8>, RevocationPubError> {
    let mut value =
        serde_json::to_value(crl).map_err(|e| RevocationPubError::Serialization(e.to_string()))?;
    let Some(object) = value.as_object_mut() else {
        return Err(RevocationPubError::Serialization(
            "publication did not serialize to a JSON object".to_string(),
        ));
    };
    object.remove("signature");
    let canonical = jcs(&value).map_err(|e| RevocationPubError::Serialization(e.to_string()))?;
    Ok(canonical.into_bytes())
}

/// Mirrors the list's records through its public surface.
///
/// The lifecycle keeps `RevocationList.records` private and exposes no
/// iterator; its serde projection is the public read surface, and it is
/// byte-faithful — [`RevocationList`] derives `Serialize` over that exact
/// map, nested under its `records` field. Records come out sorted by `kid`
/// (BTreeMap ordering), restated here so determinism does not depend on the
/// map type.
fn mirror_records(
    list: &RevocationList,
) -> Result<Vec<RevocationRecordMirror>, RevocationPubError> {
    let value = serde_json::to_value(list)
        .map_err(|e| RevocationPubError::Lifecycle(format!("revocation list: {e}")))?;
    let Some(entries) = value.get("records").and_then(|v| v.as_object()) else {
        return Err(RevocationPubError::Lifecycle(
            "revocation list did not project its records map".to_string(),
        ));
    };
    let mut mirrored = Vec::with_capacity(entries.len());
    for (kid, record) in entries {
        let revoked_at = record
            .get("revoked_at")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                RevocationPubError::Lifecycle(format!("record {kid}: missing revoked_at"))
            })?;
        let reason = record
            .get("reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RevocationPubError::Lifecycle(format!("record {kid}: missing reason"))
            })?;
        mirrored.push(RevocationRecordMirror {
            kid: kid.clone(),
            revoked_at,
            reason: reason.to_string(),
        });
    }
    mirrored.sort_by(|a, b| a.kid.cmp(&b.kid));
    Ok(mirrored)
}

/// Signs `revocations` as a CTP-CRL-v1 publication under `issuer` at `epoch`
/// and instant `at`. The issuer's key id travels in the publication so
/// verifiers can select the trust-anchor key; the signature covers the
/// canonical preimage (struct minus signature).
pub fn publish(
    revocations: &RevocationList,
    issuer: &Es256SigningKey,
    issuer_kid: &KeyId,
    epoch: u64,
    at: u64,
) -> Result<SignedRevocationList, RevocationPubError> {
    let candidate = SignedRevocationList {
        format: CRL_FORMAT.to_string(),
        issuer_kid: issuer_kid.0.clone(),
        epoch,
        published_at: at,
        revoked: mirror_records(revocations)?,
        signature: Vec::new(),
    };
    let preimage = publication_preimage(&candidate)?;
    let signature = issuer.sign(&preimage);
    Ok(SignedRevocationList {
        signature,
        ..candidate
    })
}

/// Verifies a publication against the issuer's SEC1 uncompressed public key.
///
/// A format mismatch or a malformed signature/public key is a typed refusal;
/// a well-formed signature that does not verify returns `Ok(false)` (a
/// well-formed forgery is an observation, not a parse failure). `Ok(true)`
/// means the bytes are exactly what the issuer signed.
pub fn verify_publication(
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
) -> Result<bool, RevocationPubError> {
    if crl.format != CRL_FORMAT {
        return Err(RevocationPubError::BadFormat(crl.format.clone()));
    }
    let preimage = publication_preimage(crl)?;
    verify_es256(issuer_pk_sec1, &preimage, &crl.signature)
        .map_err(|_| RevocationPubError::BadSignature)
}

/// Applies a verified publication to the local revocation list. Atomic: every
/// refusal below leaves `revocations` untouched.
///
/// Order of admission:
/// 1. wire format and issuer signature FIRST — unverified bytes are never
///    merged;
/// 2. epoch freshness — the publication's epoch may lag `current_epoch` by at
///    most `max_staleness` (the lifecycle E-J grace: the revocation may not
///    have propagated yet), and its signing instant may lag `now` by the same
///    bound; past either, [`RevocationPubError::StaleEpoch`];
/// 3. merge — every record is applied per the lifecycle law that the latest
///    revocation governs; the returned count is the number of records merged
///    (re-applying the same publication is idempotent and returns the same
///    count).
pub fn apply_to(
    revocations: &mut RevocationList,
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
    current_epoch: u64,
    max_staleness: u64,
    now: u64,
) -> Result<usize, RevocationPubError> {
    if crl.format != CRL_FORMAT {
        return Err(RevocationPubError::BadFormat(crl.format.clone()));
    }
    let preimage = publication_preimage(crl)?;
    let verified = verify_es256(issuer_pk_sec1, &preimage, &crl.signature)
        .map_err(|_| RevocationPubError::BadSignature)?;
    if !verified {
        return Err(RevocationPubError::BadSignature);
    }
    let epoch_lag = current_epoch.saturating_sub(crl.epoch);
    if epoch_lag > max_staleness {
        return Err(RevocationPubError::StaleEpoch {
            published: crl.epoch,
            current: current_epoch,
            max_staleness,
        });
    }
    if now.saturating_sub(crl.published_at) > max_staleness {
        return Err(RevocationPubError::StaleEpoch {
            published: crl.epoch,
            current: current_epoch,
            max_staleness,
        });
    }
    for record in &crl.revoked {
        revocations.revoke(&record.kid, record.revoked_at, record.reason.clone());
    }
    Ok(crl.revoked.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_lifecycle::MAX_REVOCATION_STALENESS_SECONDS;

    const KID_A: &str = "k-a";
    const KID_B: &str = "k-b";
    const REASON: &str = "compromised";
    const PUBLISHED_AT: u64 = 1_000_000;

    fn fixture_key() -> Es256SigningKey {
        let mut seed = [0x0Cu8; 32];
        seed[0] = 0x42;
        Es256SigningKey::from_seed(&seed).expect("valid fixture scalar")
    }

    fn fixture_list() -> RevocationList {
        let mut list = RevocationList::default();
        list.revoke(KID_A, 999_000, REASON.to_string());
        list.revoke(KID_B, 999_500, "superseded".to_string());
        list
    }

    /// The list's public serde projection — the only whole-list equality
    /// surface (`records` is private by lifecycle design).
    fn projection(list: &RevocationList) -> serde_json::Value {
        serde_json::to_value(list).expect("list serializes")
    }

    fn wire(crl: &SignedRevocationList) -> String {
        serde_json::to_string(crl).expect("publication serializes")
    }

    // -- teeth first: publish -> verify -> apply round trip ------------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(CRL_FORMAT, "CTP-CRL-v1");
        assert_eq!(THIS_EPOCH, 0);
        assert!(!DOMAIN_TAG.is_empty());
    }

    #[test]
    fn publish_verify_apply_round_trip_applies_all() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());

        let crl = publish(&source, &issuer, &kid, source.current_epoch(), PUBLISHED_AT)
            .expect("publication signs");
        assert_eq!(crl.format, CRL_FORMAT);
        assert_eq!(crl.issuer_kid, kid.0);
        assert_eq!(crl.epoch, source.current_epoch());
        assert_eq!(crl.revoked.len(), 2);
        assert!(!crl.signature.is_empty());
        // Mirror is sorted by kid regardless of revocation order.
        let kids: Vec<&str> = crl.revoked.iter().map(|r| r.kid.as_str()).collect();
        assert_eq!(kids, vec![KID_A, KID_B]);

        assert!(
            verify_publication(&crl, &issuer.public_key_sec1()).expect("verifies"),
            "an untampered publication must verify"
        );

        let mut target = RevocationList::default();
        let current = target.current_epoch();
        let applied = apply_to(
            &mut target,
            &crl,
            &issuer.public_key_sec1(),
            current,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect("publication applies");
        assert_eq!(applied, 2, "every record of a verified publication applies");
        assert_eq!(projection(&target), projection(&source));
        assert!(target.is_revoked(KID_A));
        assert_eq!(target.revoked_at(KID_B), Some(999_500));
        // Re-applying the same publication is idempotent state-wise.
        let advanced = target.current_epoch();
        let again = apply_to(
            &mut target,
            &crl,
            &issuer.public_key_sec1(),
            advanced,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect("re-apply admits");
        assert_eq!(again, 2);
        assert_eq!(projection(&target), projection(&source));
    }

    // -- every variant witnessed ----------------------------------------------

    #[test]
    fn bad_signature_variant_witnessed_by_flipped_signature_byte() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let mut crl = publish(&source, &issuer, &kid, 1, PUBLISHED_AT).expect("signs");

        // Flip the DER SEQUENCE tag: malformed signature bytes -> typed refusal.
        let mut broken_structure = crl.clone();
        broken_structure.signature[0] ^= 0xFF;
        assert!(matches!(
            verify_publication(&broken_structure, &issuer.public_key_sec1()),
            Err(RevocationPubError::BadSignature)
        ));

        // Flip the last scalar byte: well-formed DER that cannot verify.
        let last = crl.signature.len() - 1;
        crl.signature[last] ^= 0x01;
        assert!(
            !verify_publication(&crl, &issuer.public_key_sec1()).expect("well-formed"),
            "a flipped signature scalar must not verify"
        );
    }

    #[test]
    fn bad_format_variant_witnessed() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let mut crl = publish(&source, &issuer, &kid, 1, PUBLISHED_AT).expect("signs");
        crl.format = "CTP-CRL-v0".to_string();
        match verify_publication(&crl, &issuer.public_key_sec1()) {
            Err(RevocationPubError::BadFormat(format)) => assert_eq!(format, "CTP-CRL-v0"),
            outcome => panic!("expected BadFormat, got {outcome:?}"),
        }
    }

    #[test]
    fn stale_epoch_variant_witnessed_past_grace() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(&source, &issuer, &kid, 1_000, PUBLISHED_AT).expect("signs");

        // Exactly at the grace boundary the publication is still live.
        let mut target = RevocationList::default();
        let at_bound = apply_to(
            &mut target,
            &crl,
            &issuer.public_key_sec1(),
            crl.epoch + MAX_REVOCATION_STALENESS_SECONDS,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        );
        assert!(
            at_bound.is_ok(),
            "lag exactly at the bound is inside the grace"
        );
        // One epoch past the grace refuses with the full evidence tuple.
        let mut untouched = RevocationList::default();
        match apply_to(
            &mut untouched,
            &crl,
            &issuer.public_key_sec1(),
            crl.epoch + MAX_REVOCATION_STALENESS_SECONDS + 1,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        ) {
            Err(RevocationPubError::StaleEpoch {
                published,
                current,
                max_staleness,
            }) => {
                assert_eq!(published, crl.epoch);
                assert_eq!(current, crl.epoch + MAX_REVOCATION_STALENESS_SECONDS + 1);
                assert_eq!(max_staleness, MAX_REVOCATION_STALENESS_SECONDS);
            }
            outcome => panic!("expected StaleEpoch, got {outcome:?}"),
        }
    }

    #[test]
    fn stale_signing_instant_refused_via_now() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(&source, &issuer, &kid, 1_000, PUBLISHED_AT).expect("signs");
        let mut target = RevocationList::default();
        // Epoch is current, but the publication was signed too long ago.
        match apply_to(
            &mut target,
            &crl,
            &issuer.public_key_sec1(),
            crl.epoch,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT + MAX_REVOCATION_STALENESS_SECONDS + 1,
        ) {
            Err(RevocationPubError::StaleEpoch { published, .. }) => {
                assert_eq!(published, crl.epoch);
            }
            outcome => panic!("expected StaleEpoch, got {outcome:?}"),
        }
    }

    // -- atomicity: a refused apply changes nothing ---------------------------

    #[test]
    fn refused_apply_leaves_list_unchanged() {
        let issuer = fixture_key();
        let other = {
            let mut seed = [0x0Cu8; 32];
            seed[0] = 0x99;
            Es256SigningKey::from_seed(&seed).expect("valid other scalar")
        };
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(&source, &issuer, &kid, 1_000, PUBLISHED_AT).expect("signs");
        let mut target = RevocationList::default();
        let before = projection(&target);

        // Forged by another key: BadSignature refusal.
        let forged = publish(&source, &other, &kid, crl.epoch, PUBLISHED_AT).expect("signs");
        assert!(matches!(
            apply_to(
                &mut target,
                &forged,
                &issuer.public_key_sec1(),
                crl.epoch,
                MAX_REVOCATION_STALENESS_SECONDS,
                PUBLISHED_AT
            ),
            Err(RevocationPubError::BadSignature)
        ));
        assert_eq!(projection(&target), before, "forged apply must be atomic");

        // Stale epoch: StaleEpoch refusal.
        assert!(matches!(
            apply_to(
                &mut target,
                &crl,
                &issuer.public_key_sec1(),
                crl.epoch + MAX_REVOCATION_STALENESS_SECONDS + 1,
                MAX_REVOCATION_STALENESS_SECONDS,
                PUBLISHED_AT
            ),
            Err(RevocationPubError::StaleEpoch { .. })
        ));
        assert_eq!(projection(&target), before, "stale apply must be atomic");

        // Bad format: BadFormat refusal.
        let mut mislabeled = crl.clone();
        mislabeled.format = String::from("not-a-crl");
        assert!(matches!(
            apply_to(
                &mut target,
                &mislabeled,
                &issuer.public_key_sec1(),
                crl.epoch,
                MAX_REVOCATION_STALENESS_SECONDS,
                PUBLISHED_AT
            ),
            Err(RevocationPubError::BadFormat(_))
        ));
        assert_eq!(
            projection(&target),
            before,
            "mislabeled apply must be atomic"
        );
    }

    #[test]
    fn wrong_issuer_key_does_not_verify() {
        let issuer = fixture_key();
        let other = {
            let mut seed = [0x0Cu8; 32];
            seed[0] = 0x77;
            Es256SigningKey::from_seed(&seed).expect("valid other scalar")
        };
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(&source, &issuer, &kid, 1, PUBLISHED_AT).expect("signs");
        assert!(
            !verify_publication(&crl, &other.public_key_sec1()).expect("well-formed"),
            "a publication must not verify under a different issuer key"
        );
    }

    // -- wire format and determinism -------------------------------------------

    #[test]
    fn publication_round_trips_over_json_wire() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl =
            publish(&source, &issuer, &kid, source.current_epoch(), PUBLISHED_AT).expect("signs");
        let encoded = wire(&crl);
        let decoded: SignedRevocationList =
            serde_json::from_str(&encoded).expect("publication deserializes");
        assert_eq!(decoded, crl);
        // The wire bytes still verify after the round trip: the preimage is
        // derived from the same fields the wire carries.
        assert!(verify_publication(&decoded, &issuer.public_key_sec1()).expect("verifies"));
    }

    #[test]
    fn same_inputs_render_identical_bytes_rfc6979() {
        let issuer = fixture_key();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let epoch = 42;
        let a = publish(&fixture_list(), &issuer, &kid, epoch, PUBLISHED_AT).expect("signs");
        let b = publish(&fixture_list(), &issuer, &kid, epoch, PUBLISHED_AT).expect("signs");
        assert_eq!(
            wire(&a),
            wire(&b),
            "RFC 6979 + sorted mirror = byte-identical"
        );
        // And the wire bytes differ once any input moves.
        let c = publish(&fixture_list(), &issuer, &kid, epoch, PUBLISHED_AT + 1).expect("signs");
        assert_ne!(wire(&a), wire(&c));
    }

    #[test]
    fn preimage_excludes_the_signature_field() {
        let issuer = fixture_key();
        let source = fixture_list();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(&source, &issuer, &kid, 7, PUBLISHED_AT).expect("signs");
        let unsigned = SignedRevocationList {
            signature: Vec::new(),
            ..crl.clone()
        };
        let from_signed = publication_preimage(&crl).expect("preimage computes");
        let from_unsigned = publication_preimage(&unsigned).expect("preimage computes");
        assert_eq!(
            from_signed, from_unsigned,
            "the preimage is the publication minus its signature"
        );
        assert!(
            !String::from_utf8_lossy(&from_signed).contains("signature"),
            "the preimage must not carry the signature field"
        );
    }

    #[test]
    fn empty_list_publishes_and_applies_zero() {
        let issuer = fixture_key();
        let kid = KeyId::from_fingerprint(&issuer.key_id_fingerprint());
        let crl = publish(
            &RevocationList::default(),
            &issuer,
            &kid,
            THIS_EPOCH,
            PUBLISHED_AT,
        )
        .expect("empty list signs");
        assert!(crl.revoked.is_empty());
        assert!(verify_publication(&crl, &issuer.public_key_sec1()).expect("verifies"));
        let mut target = RevocationList::default();
        let applied = apply_to(
            &mut target,
            &crl,
            &issuer.public_key_sec1(),
            0,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect("applies");
        assert_eq!(applied, 0);
    }
}
