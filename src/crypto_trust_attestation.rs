//! Key attestation records for the affidavit cryptographic trust plane:
//! signed claims attaching custody attributes (attesting device, secure
//! hardware, non-exportability) to a key record.
//
// Consumed query columns (attestation.rq): attestation_format, domain_tag.
// Rendered by ggen (affidavit-trust-plane-pack) from the ctp: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// Capability (boundary item: "attested attributes attached to keys"): an
// [`AttestationRecord`] is the ES256-signed statement that a key's custody
// attributes hold at an epoch. The attributes a record may claim are
// LICENSED by the attested key's origin (ctp:KeyOrigin), never by the
// caller's word: a `HardwareAttested` key licenses kind "hardware" with
// `secure_hardware`/`non_exportable` true and the origin's device model; an
// `Imported` key licenses kind "import" with software attributes; a
// `Generated` key licenses kind "self". Every claim is cross-checked against
// the licensed derivation and a contradiction is a typed
// [`AttestationError::AttributeMismatch`] refusal — the record is never
// rendered from claims the key record itself contradicts.
//
// Preimage law: the signature covers exactly the attestation record minus its
// own `signature` field, serialized JCS (RFC 8785, the pack's
// canonicalization at [`crate::crypto_trust_canonical`]).
// [`verify_attestation`] rebuilds that preimage from the received record
// alone, so verification needs no out-of-band bytes, and any mutation of a
// signed field moves the preimage and decides the signature false.
//
// House law: refusals are typed values ([`AttestationError`]), never panics;
// no `unwrap`/`expect` on production paths.

use serde::{Deserialize, Serialize};

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_es256::{verify_es256, Es256Error, Es256SigningKey};
use crate::crypto_trust_keys::{KeyOrigin, KeyRecord};

/// Wire format identifier (ctp:attest-v1 ctp:attestationFormat). Stamped in
/// every rendered record's `format` field; [`verify_attestation`] refuses any
/// other value with [`AttestationError::BadFormat`].
pub const ATTESTATION_FORMAT: &str = "CTP-ATTEST-v1";

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag).
/// Conformance-tested equal to
/// `crate::crypto_trust_canonical::DOMAIN_TAG` — both render the same graph
/// fact.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// The closed attestation-kind vocabulary (ctp:attestationKind):
/// "self" | "hardware" | "import". Serde renders kebab-case, so the wire
/// strings are exactly the graph's closed vocabulary (pinned by test).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttestationKind {
    /// Software self-attestation (licensed by a `Generated` key origin).
    #[serde(rename = "self")]
    Self_,
    /// Hardware-backed attestation (licensed by a `HardwareAttested` origin).
    Hardware,
    /// Imported-key attestation (licensed by an `Imported` key origin).
    Import,
}

impl AttestationKind {
    /// The wire string for this kind, exactly what
    /// [`AttestationRecord::kind`] carries.
    pub fn as_str(self) -> &'static str {
        match self {
            AttestationKind::Self_ => "self",
            AttestationKind::Hardware => "hardware",
            AttestationKind::Import => "import",
        }
    }
}

/// A signed statement attaching attested custody attributes to one key
/// record (ctp:AttestationRecord). The `signature` field is excluded from
/// its own signed preimage; every other field participates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationRecord {
    /// Wire format identifier; verification refuses any value other than
    /// [`ATTESTATION_FORMAT`].
    pub format: String,
    /// The attested key's registry id (ctp:attestsKey → the key record).
    pub key_id: String,
    /// The attestation kind as its wire string ("self" | "hardware" |
    /// "import"); produced only from [`AttestationKind`] (enum-backed).
    pub kind: String,
    /// Attesting device model, present only for hardware-kind attestations.
    pub device_model: Option<String>,
    /// Whether the attested key lives in secure hardware.
    pub secure_hardware: bool,
    /// Whether the attested key's material is non-exportable.
    pub non_exportable: bool,
    /// Attestation epoch in unix seconds. Must stay a JCS safe integer
    /// (<= 2^53) so the signed preimage canonicalizes.
    pub attested_at: u64,
    /// ES256 (DER) signature over the JCS preimage of this record minus this
    /// field, made by the attesting [`Es256SigningKey`].
    pub signature: Vec<u8>,
}

/// Typed refusals of the attestation boundary. Values, never panics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttestationError {
    /// The record's `format` field is not [`ATTESTATION_FORMAT`].
    #[error("attestation format refused: {0}")]
    BadFormat(String),
    /// The signature bytes are malformed (not decodable DER) or cannot be
    /// verified at all; a well-formed signature that merely fails to verify
    /// decides `Ok(false)` instead.
    #[error("attestation signature is malformed and cannot be verified")]
    BadSignature,
    /// A claimed attribute contradicts the licensed derivation from the key
    /// record's origin (or the signer identity contradicts the signing key).
    #[error("attested attribute contradicts the key record: {0}")]
    AttributeMismatch(String),
    /// The signed preimage did not canonicalize (JCS refuses, e.g. an epoch
    /// beyond the 2^53 safe-integer range) or JSON serialization failed.
    #[error("attestation serialization failed: {0}")]
    Serialization(String),
}

/// The attributes a key record's origin LICENSES — the single source of
/// attribute truth for [`attest_key`]. Claims are checked against this, never
/// the other way around.
struct LicensedAttributes {
    kind: AttestationKind,
    device_model: Option<String>,
    secure_hardware: bool,
    non_exportable: bool,
}

/// Derives the licensed attributes from the key record's origin:
/// `HardwareAttested` licenses kind "hardware" with `secure_hardware`/
/// `non_exportable` true and the origin's device; `Imported` licenses kind
/// "import" with software attributes; `Generated` licenses kind "self".
fn licensed_attributes(record: &KeyRecord) -> LicensedAttributes {
    match &record.origin {
        KeyOrigin::HardwareAttested { device } => LicensedAttributes {
            kind: AttestationKind::Hardware,
            device_model: Some(device.clone()),
            secure_hardware: true,
            non_exportable: true,
        },
        KeyOrigin::Imported { source: _ } => LicensedAttributes {
            kind: AttestationKind::Import,
            device_model: None,
            secure_hardware: false,
            non_exportable: false,
        },
        KeyOrigin::Generated => LicensedAttributes {
            kind: AttestationKind::Self_,
            device_model: None,
            secure_hardware: false,
            non_exportable: false,
        },
    }
}

/// Builds the signed preimage of `att`: the record serialized to JSON with
/// its own `signature` field removed, then canonicalized JCS. Both signing
/// and verification rebuild the preimage through this one function, so the
/// signed bytes and the verified bytes cannot drift.
fn attestation_preimage(att: &AttestationRecord) -> Result<Vec<u8>, AttestationError> {
    let mut value =
        serde_json::to_value(att).map_err(|e| AttestationError::Serialization(e.to_string()))?;
    let object = value.as_object_mut().ok_or_else(|| {
        AttestationError::Serialization("attestation JSON is not an object".to_string())
    })?;
    object.remove("signature");
    let canonical = jcs(&value).map_err(|e| AttestationError::Serialization(e.to_string()))?;
    Ok(canonical.into_bytes())
}

/// Attests a key record: derives the licensed attributes from
/// `record.origin`, cross-checks the caller's `kind` and `device` claims
/// against them (any contradiction is
/// [`AttestationError::AttributeMismatch`], naming the contradicted
/// attribute), then signs the JCS preimage of the resulting record minus its
/// signature with the attesting `signer`.
///
/// `device` is the caller's claim as `(device_model, secure_hardware,
/// non_exportable)`. A hardware-attested key requires the claim to name the
/// origin's exact device with both flags true; a software-origin key refuses
/// any device claim. `signer_kid` must be the signing key's own fingerprint
/// id ([`Es256SigningKey::key_id_fingerprint`] hex) — an attestation signed
/// under a mislabeled signer is refused, never rendered.
pub fn attest_key(
    record: &KeyRecord,
    kind: AttestationKind,
    device: Option<(&str, bool, bool)>,
    signer: &Es256SigningKey,
    signer_kid: &str,
    at: u64,
) -> Result<AttestationRecord, AttestationError> {
    let signer_fingerprint = signer.key_id_fingerprint().as_hex();
    if signer_kid != signer_fingerprint {
        return Err(AttestationError::AttributeMismatch(format!(
            "signer_kid {signer_kid:?} is not the signing key's fingerprint id {signer_fingerprint}"
        )));
    }

    let licensed = licensed_attributes(record);

    if kind != licensed.kind {
        return Err(AttestationError::AttributeMismatch(format!(
            "attestation kind {:?} contradicts key origin licensed as {:?}",
            kind.as_str(),
            licensed.kind.as_str()
        )));
    }

    match (&licensed.device_model, device) {
        (Some(licensed_model), Some((claimed_model, secure_hardware, non_exportable))) => {
            if claimed_model != licensed_model.as_str() {
                return Err(AttestationError::AttributeMismatch(format!(
                    "claimed device model {claimed_model:?} contradicts attesting device {licensed_model:?}"
                )));
            }
            if secure_hardware != licensed.secure_hardware {
                return Err(AttestationError::AttributeMismatch(
                    "claimed secure_hardware contradicts the hardware-attested key record"
                        .to_string(),
                ));
            }
            if non_exportable != licensed.non_exportable {
                return Err(AttestationError::AttributeMismatch(
                    "claimed non_exportable contradicts the hardware-attested key record"
                        .to_string(),
                ));
            }
        }
        (Some(licensed_model), None) => {
            return Err(AttestationError::AttributeMismatch(format!(
                "hardware-attested key requires its device claims for attesting device {licensed_model:?}"
            )));
        }
        (None, Some((claimed_model, _, _))) => {
            return Err(AttestationError::AttributeMismatch(format!(
                "software-origin key record licenses no device; refusing claimed device {claimed_model:?}"
            )));
        }
        (None, None) => {}
    }

    let mut attestation = AttestationRecord {
        format: ATTESTATION_FORMAT.to_string(),
        key_id: record.id.0.clone(),
        kind: kind.as_str().to_string(),
        device_model: licensed.device_model,
        secure_hardware: licensed.secure_hardware,
        non_exportable: licensed.non_exportable,
        attested_at: at,
        signature: Vec::new(),
    };
    let preimage = attestation_preimage(&attestation)?;
    attestation.signature = signer.sign(&preimage);
    Ok(attestation)
}

/// Verifies an attestation record against an attesting public key (SEC1
/// uncompressed, as produced by [`Es256SigningKey::public_key_sec1`]).
///
/// Verdict order: a foreign `format` is [`AttestationError::BadFormat`]; a
/// preimage that fails to canonicalize is
/// [`AttestationError::Serialization`]; undecodable signature or key bytes
/// are [`AttestationError::BadSignature`]; a well-formed signature that does
/// not verify decides `Ok(false)` — tampering is a decision, not a panic.
pub fn verify_attestation(
    att: &AttestationRecord,
    signer_pk: &[u8],
) -> Result<bool, AttestationError> {
    if att.format != ATTESTATION_FORMAT {
        return Err(AttestationError::BadFormat(att.format.clone()));
    }
    let preimage = attestation_preimage(att)?;
    match verify_es256(signer_pk, &preimage, &att.signature) {
        Ok(verified) => Ok(verified),
        Err(Es256Error::MalformedPublicKey) | Err(Es256Error::MalformedSignature) => {
            Err(AttestationError::BadSignature)
        }
        Err(Es256Error::P256(detail)) => Err(AttestationError::Serialization(detail)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        attest_key, verify_attestation, AttestationError, AttestationKind, AttestationRecord,
        ATTESTATION_FORMAT, DOMAIN_TAG,
    };
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord,
        PublicKeyMaterial,
    };
    use serde_json::Value;

    // -- fixtures ---------------------------------------------------------

    fn fixture_signer() -> crate::crypto_trust_es256::Es256SigningKey {
        crate::crypto_trust_es256::Es256SigningKey::from_seed(&[7u8; 32])
            .expect("[7u8; 32] is a valid non-zero P-256 scalar")
    }

    fn other_fixture_signer() -> crate::crypto_trust_es256::Es256SigningKey {
        crate::crypto_trust_es256::Es256SigningKey::from_seed(&[9u8; 32])
            .expect("[9u8; 32] is a valid non-zero P-256 scalar")
    }

    fn fixture_signer_kid(signer: &crate::crypto_trust_es256::Es256SigningKey) -> String {
        signer.key_id_fingerprint().as_hex()
    }

    fn key_record(origin: KeyOrigin) -> KeyRecord {
        let public_key = PublicKeyMaterial::Es256Sec1(vec![0x04; 65]);
        let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public_key);
        KeyRecord {
            id: KeyId("ctp-attest-fixture-key".to_string()),
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: "custodian-fixture".to_string(),
                device: None,
                org: None,
            },
            origin,
            public_key,
            created_epoch: 1_700_000_000,
        }
    }

    fn hardware_record() -> KeyRecord {
        key_record(KeyOrigin::HardwareAttested {
            device: "YubiKey 5 NFC".to_string(),
        })
    }

    fn valid_hardware_attestation() -> (
        AttestationRecord,
        crate::crypto_trust_es256::Es256SigningKey,
    ) {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let attestation = attest_key(
            &hardware_record(),
            AttestationKind::Hardware,
            Some(("YubiKey 5 NFC", true, true)),
            &signer,
            &kid,
            1_700_000_100,
        )
        .expect("consistent hardware claims are attested");
        (attestation, signer)
    }

    // -- teeth: attest -> verify round trip -------------------------------

    #[test]
    fn hardware_attestation_round_trips_and_verifies() {
        let (attestation, signer) = valid_hardware_attestation();

        assert_eq!(attestation.format, ATTESTATION_FORMAT);
        assert_eq!(attestation.key_id, "ctp-attest-fixture-key");
        assert_eq!(attestation.kind, "hardware");
        assert_eq!(attestation.device_model.as_deref(), Some("YubiKey 5 NFC"));
        assert!(attestation.secure_hardware);
        assert!(attestation.non_exportable);
        assert_eq!(attestation.attested_at, 1_700_000_100);

        let verified = verify_attestation(&attestation, &signer.public_key_sec1())
            .expect("a well-formed record verifies to a decision");
        assert!(verified);
    }

    #[test]
    fn software_key_self_attestation_round_trips() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let record = key_record(KeyOrigin::Generated);

        let attestation = attest_key(&record, AttestationKind::Self_, None, &signer, &kid, 7)
            .expect("the licensed self-attestation is attested");

        assert_eq!(attestation.kind, "self");
        assert_eq!(attestation.device_model, None);
        assert!(!attestation.secure_hardware);
        assert!(!attestation.non_exportable);

        let verified = verify_attestation(&attestation, &signer.public_key_sec1())
            .expect("a well-formed record verifies to a decision");
        assert!(verified);
    }

    #[test]
    fn imported_key_attestation_round_trips() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let record = key_record(KeyOrigin::Imported {
            source: "pkcs12-bundle".to_string(),
        });

        let attestation = attest_key(&record, AttestationKind::Import, None, &signer, &kid, 11)
            .expect("the licensed import attestation is attested");

        assert_eq!(attestation.kind, "import");
        let verified = verify_attestation(&attestation, &signer.public_key_sec1())
            .expect("a well-formed record verifies to a decision");
        assert!(verified);
    }

    // -- determinism -------------------------------------------------------

    #[test]
    fn attestation_is_deterministic() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let record = hardware_record();

        let first = attest_key(
            &record,
            AttestationKind::Hardware,
            Some(("YubiKey 5 NFC", true, true)),
            &signer,
            &kid,
            42,
        )
        .expect("first attestation renders");
        let second = attest_key(
            &record,
            AttestationKind::Hardware,
            Some(("YubiKey 5 NFC", true, true)),
            &signer,
            &kid,
            42,
        )
        .expect("second attestation renders");

        assert_eq!(first, second, "RFC 6979 signing plus pure attribute derivation: identical inputs, byte-identical record");
        let first_bytes = serde_json::to_vec(&first).expect("serializes");
        let second_bytes = serde_json::to_vec(&second).expect("serializes");
        assert_eq!(first_bytes, second_bytes);
    }

    // -- AttributeMismatch: claims contradict the licensed attributes ------

    #[test]
    fn hardware_key_with_self_kind_and_non_exportable_claim_is_refused() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);

        let refusal = attest_key(
            &hardware_record(),
            AttestationKind::Self_,
            Some(("YubiKey 5 NFC", true, true)),
            &signer,
            &kid,
            42,
        );

        assert!(
            matches!(refusal, Err(AttestationError::AttributeMismatch(_))),
            "kind self contradicts a HardwareAttested key even with non_exportable true claimed"
        );
    }

    #[test]
    fn contradicting_secure_hardware_claim_is_refused() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);

        let refusal = attest_key(
            &hardware_record(),
            AttestationKind::Hardware,
            Some(("YubiKey 5 NFC", false, true)),
            &signer,
            &kid,
            42,
        );

        assert!(matches!(
            refusal,
            Err(AttestationError::AttributeMismatch(_))
        ));
    }

    #[test]
    fn foreign_device_model_claim_is_refused() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);

        let refusal = attest_key(
            &hardware_record(),
            AttestationKind::Hardware,
            Some(("SoloKeys Lite", true, true)),
            &signer,
            &kid,
            42,
        );

        assert!(matches!(
            refusal,
            Err(AttestationError::AttributeMismatch(_))
        ));
    }

    #[test]
    fn hardware_key_without_device_claims_is_refused() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);

        let refusal = attest_key(
            &hardware_record(),
            AttestationKind::Hardware,
            None,
            &signer,
            &kid,
            42,
        );

        assert!(matches!(
            refusal,
            Err(AttestationError::AttributeMismatch(_))
        ));
    }

    #[test]
    fn software_key_claiming_a_device_is_refused() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let record = key_record(KeyOrigin::Generated);

        let refusal = attest_key(
            &record,
            AttestationKind::Self_,
            Some(("YubiKey 5 NFC", false, false)),
            &signer,
            &kid,
            42,
        );

        assert!(matches!(
            refusal,
            Err(AttestationError::AttributeMismatch(_))
        ));
    }

    #[test]
    fn foreign_signer_kid_is_refused() {
        let signer = fixture_signer();
        let record = key_record(KeyOrigin::Generated);

        let refusal = attest_key(
            &record,
            AttestationKind::Self_,
            None,
            &signer,
            "not-a-fingerprint",
            42,
        );

        assert!(
            matches!(refusal, Err(AttestationError::AttributeMismatch(_))),
            "an attestation signed under a mislabeled signer kid is refused"
        );
    }

    // -- verify teeth: BadFormat, BadSignature, decided-false tampering -----

    #[test]
    fn wrong_format_is_bad_format_refusal() {
        let (mut attestation, signer) = valid_hardware_attestation();
        attestation.format = "CTP-ATTEST-v0".to_string();

        let verdict = verify_attestation(&attestation, &signer.public_key_sec1());

        assert!(
            matches!(verdict, Err(AttestationError::BadFormat(offending)) if offending == "CTP-ATTEST-v0")
        );
    }

    #[test]
    fn malformed_signature_bytes_are_bad_signature() {
        let (mut attestation, signer) = valid_hardware_attestation();
        attestation.signature = vec![0xFF; 8];

        let verdict = verify_attestation(&attestation, &signer.public_key_sec1());

        assert!(matches!(verdict, Err(AttestationError::BadSignature)));
    }

    #[test]
    fn malformed_signing_key_is_bad_signature() {
        let (attestation, _signer) = valid_hardware_attestation();

        let verdict = verify_attestation(&attestation, &[0x01; 10]);

        assert!(matches!(verdict, Err(AttestationError::BadSignature)));
    }

    #[test]
    fn tampered_signature_decides_false() {
        let (mut attestation, signer) = valid_hardware_attestation();
        let last = attestation.signature.len() - 1;
        attestation.signature[last] ^= 0x01;

        let verdict = verify_attestation(&attestation, &signer.public_key_sec1())
            .expect("a well-formed DER flip still yields a decision");

        assert!(
            !verdict,
            "a flipped signature byte must decide false, not error"
        );
    }

    #[test]
    fn tampered_payload_decides_false() {
        let (mut attestation, signer) = valid_hardware_attestation();
        attestation.attested_at += 1;

        let verdict = verify_attestation(&attestation, &signer.public_key_sec1())
            .expect("the record itself is well-formed");

        assert!(
            !verdict,
            "any mutation of a signed field moves the preimage and decides false"
        );
    }

    #[test]
    fn wrong_signing_key_decides_false() {
        let (attestation, _signer) = valid_hardware_attestation();
        let other = other_fixture_signer();

        let verdict = verify_attestation(&attestation, &other.public_key_sec1())
            .expect("the record itself is well-formed");

        assert!(!verdict, "a valid signature from another key decides false");
    }

    // -- Serialization refusal ---------------------------------------------

    #[test]
    fn unsafe_integer_epoch_is_serialization_refusal() {
        let signer = fixture_signer();
        let kid = fixture_signer_kid(&signer);
        let record = key_record(KeyOrigin::Generated);

        let refusal = attest_key(
            &record,
            AttestationKind::Self_,
            None,
            &signer,
            &kid,
            u64::MAX,
        );

        assert!(
            matches!(refusal, Err(AttestationError::Serialization(_))),
            "an epoch beyond the JCS 2^53 safe-integer range refuses instead of rendering an unverifiable record"
        );
    }

    // -- serde + conformance ------------------------------------------------

    #[test]
    fn kind_serde_is_kebab_vocabulary() {
        assert_eq!(
            serde_json::to_string(&AttestationKind::Self_).expect("serializes"),
            "\"self\""
        );
        assert_eq!(
            serde_json::to_string(&AttestationKind::Hardware).expect("serializes"),
            "\"hardware\""
        );
        assert_eq!(
            serde_json::to_string(&AttestationKind::Import).expect("serializes"),
            "\"import\""
        );
        let parsed: AttestationKind = serde_json::from_str("\"import\"").expect("parses");
        assert_eq!(parsed, AttestationKind::Import);
    }

    #[test]
    fn record_serde_round_trips_with_all_eight_fields() {
        let (attestation, _signer) = valid_hardware_attestation();

        let json = serde_json::to_string(&attestation).expect("serializes");
        let reparsed: AttestationRecord = serde_json::from_str(&json).expect("parses");
        assert_eq!(reparsed, attestation);

        let value: Value = serde_json::to_value(&attestation).expect("serializes");
        let object = value.as_object().expect("an attestation is a JSON object");
        assert_eq!(object.len(), 8, "format, key_id, kind, device_model, secure_hardware, non_exportable, attested_at, signature");
        assert!(object.contains_key("signature"));
    }

    #[test]
    fn graph_facts_conform() {
        assert_eq!(ATTESTATION_FORMAT, "CTP-ATTEST-v1");
        assert_eq!(DOMAIN_TAG, crate::crypto_trust_canonical::DOMAIN_TAG);
    }
}
