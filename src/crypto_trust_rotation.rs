//! Key rotation ceremony and the profile migration law for the affidavit cryptographic trust plane.
// Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//
// Consumed query columns (rotation.rq): migrations, domain_tag.
//
// The migration table below is a GRAPH FACT, not a hardcoded policy: each row
// is one ctp:ProfileMigration individual (from | to | kind).
// [`admission_allowed`] reads only this table — admitting or revoking a
// migration in the ontology changes the ceremony's lawful transitions on the
// next sync. The ceremony gate runs FIRST: an unadmitted profile pair is a
// typed [`RotationError::MigrationRefused`], never a silent success.
//
// Successor-signature law: the NEW (hybrid) key signs the rotation record —
// the successor attests its own takeover. Both hybrid halves (ES256 DER +
// ML-DSA-65) sign the domain-separated BLAKE3 digest of the JCS-canonical
// record-minus-signature; the wire form is `es256_der || mldsa65`, and the
// split is unambiguous because the ML-DSA-65 half is fixed-length.
//
// House law: refusal-as-value (typed [`RotationError`], never panics); every
// variant witnessed by tests; teeth-before-tamper (the honest round trip is
// proven before each tamper falsifier); no unwrap/expect outside tests.

use crate::crypto_trust_canonical::{digest, jcs};
use crate::crypto_trust_es256::Es256SigningKey;
use crate::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, KeyId, PublicKeyMaterial,
};
use crate::crypto_trust_pqc::{
    hybrid_sign, hybrid_verify, ml_dsa65_from_seed, HybridSecret, HybridSignature,
    ML_DSA_65_SIGNATURE_LEN,
};

/// Trust-plane domain tag as bound by rotation.rq (ctp:policy-v1
/// ctp:domainTag). Cross-rule consistency with
/// [`crate::crypto_trust_canonical::DOMAIN_TAG`] is tested; the canonical
/// module remains the single owner of the constant.
pub const ROTATION_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Domain-separation label for rotation-record digests: every digest this
/// module computes mixes the trust-plane domain tag AND this label, so a
/// rotation digest can never collide with an envelope, seal, or PQ randomizer
/// digest even over identical bytes.
pub const ROTATION_DIGEST_LABEL: &[u8] = b"ctp.rotation.record.v1";

/// The admitted profile migration table, rendered from the graph's
/// ctp:ProfileMigration individuals as (from, to, kind) in the rotation.rq
/// projection order (lexicographic over "from|to|kind"). This table — not a
/// hand-typed list — is the authority [`admission_allowed`] and
/// [`assert_not_downgrade_under`] judge against, so a graph-side migration
/// change breaks the render loudly instead of drifting silently.
pub const MIGRATIONS: &[(&str, &str, &str)] = &[
    ("CLASSICAL", "HYBRID", "upgrade"),
    ("HYBRID", "CLASSICAL", "downgrade"),
    ("HYBRID", "PQC", "upgrade"),
];

/// The record a rotation ceremony produces: the takeover binding signed BY
/// the successor key. Every field except `successor_signature` is the signed
/// payload — JCS-canonicalized, domain-digested, then covered by BOTH hybrid
/// halves of the successor signature.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RotationRecord {
    /// Trust-plane id of the retiring key.
    pub old_key_id: String,
    /// Trust-plane id of the successor key (fingerprint of its hybrid
    /// public material, bound to the hybrid algorithm identity).
    pub new_key_id: String,
    /// Profile the rotation migrates FROM (rendered profileName form).
    pub from_profile: String,
    /// Profile the rotation migrates TO (rendered profileName form).
    pub to_profile: String,
    /// Epoch seconds at which the takeover was attested.
    pub rotated_at: u64,
    /// Both hybrid halves over the record digest, `es256_der || mldsa65`.
    pub successor_signature: Vec<u8>,
}

/// The signed payload of a [`RotationRecord`] (everything but the signature).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct SignedRotationFields {
    old_key_id: String,
    new_key_id: String,
    from_profile: String,
    to_profile: String,
    rotated_at: u64,
}

impl RotationRecord {
    fn signed_fields(&self) -> SignedRotationFields {
        SignedRotationFields {
            old_key_id: self.old_key_id.clone(),
            new_key_id: self.new_key_id.clone(),
            from_profile: self.from_profile.clone(),
            to_profile: self.to_profile.clone(),
            rotated_at: self.rotated_at,
        }
    }
}

/// Typed refusals of the rotation ceremony. Values, never panics.
#[derive(Debug, thiserror::Error)]
pub enum RotationError {
    #[error("migration {0:?} -> {1:?} is not admitted by the profile migration policy")]
    MigrationRefused(CryptoProfile, CryptoProfile),
    #[error("successor signature invalid")]
    SuccessorSignatureInvalid,
    #[error("rotation record tampered")]
    RecordTampered,
    #[error("provider: {0}")]
    Provider(String),
    #[error("serialization: {0}")]
    Serialization(String),
}

/// Domain-separated BLAKE3 digest over the JCS-canonical signed fields:
/// `digest(ROTATION_DOMAIN_TAG, [ROTATION_DIGEST_LABEL, jcs(fields)])`.
/// Refusals are typed: a `rotated_at` beyond JCS double precision (> 2^53,
/// the canonical module's certify-don't-coerce law) is
/// [`RotationError::Serialization`], never a silent rounding.
fn rotation_digest(fields: &SignedRotationFields) -> Result<[u8; 32], RotationError> {
    let value =
        serde_json::to_value(fields).map_err(|e| RotationError::Serialization(e.to_string()))?;
    let canonical = jcs(&value).map_err(|e| RotationError::Serialization(e.to_string()))?;
    Ok(digest(
        ROTATION_DOMAIN_TAG,
        &[ROTATION_DIGEST_LABEL, canonical.as_bytes()],
    ))
}

/// Hybrid signature wire form: `es256_der || mldsa65`. The ML-DSA-65 half is
/// the fixed FIPS 204 encoding ([`ML_DSA_65_SIGNATURE_LEN`]), so the trailing
/// split is unambiguous.
fn encode_hybrid_signature(sig: &HybridSignature) -> Vec<u8> {
    let mut out = Vec::with_capacity(sig.es256_der.len() + sig.mldsa65.len());
    out.extend_from_slice(&sig.es256_der);
    out.extend_from_slice(&sig.mldsa65);
    out
}

/// Inverse of [`encode_hybrid_signature`]; `None` when the bytes are too
/// short to contain the fixed-length ML-DSA-65 half.
fn decode_hybrid_signature(bytes: &[u8]) -> Option<HybridSignature> {
    if bytes.len() < ML_DSA_65_SIGNATURE_LEN {
        return None;
    }
    let (es256_der, mldsa65) = bytes.split_at(bytes.len() - ML_DSA_65_SIGNATURE_LEN);
    Some(HybridSignature {
        es256_der: es256_der.to_vec(),
        mldsa65: mldsa65.to_vec(),
    })
}

/// Whether the graph admits migrating `from` -> `to`: true iff a
/// ctp:ProfileMigration row names exactly this pair. Derived from
/// [`MIGRATIONS`] — never hardcoded — so admission is a pure function of the
/// rendered graph facts.
#[must_use]
pub fn admission_allowed(from: CryptoProfile, to: CryptoProfile) -> bool {
    MIGRATIONS.iter().any(|(f, t, _)| {
        f.eq_ignore_ascii_case(from.as_str()) && t.eq_ignore_ascii_case(to.as_str())
    })
}

/// The migration kind the graph records for `from` -> `to`, if the pair is
/// admitted at all.
#[must_use]
fn migration_kind(from: CryptoProfile, to: CryptoProfile) -> Option<&'static str> {
    MIGRATIONS
        .iter()
        .find(|(f, t, _)| {
            f.eq_ignore_ascii_case(from.as_str()) && t.eq_ignore_ascii_case(to.as_str())
        })
        .map(|(_, _, kind)| *kind)
}

/// Admission as a typed result: the single gate every rotation path must pass
/// FIRST. Refusal: [`RotationError::MigrationRefused`] when the graph holds
/// no migration row for the pair.
fn admission_checked(from: CryptoProfile, to: CryptoProfile) -> Result<(), RotationError> {
    if admission_allowed(from, to) {
        Ok(())
    } else {
        Err(RotationError::MigrationRefused(from, to))
    }
}

/// Downgrade policy for [`assert_not_downgrade_under`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationPolicy {
    /// When true (the default), a graph row whose ctp:migrationKind is
    /// "downgrade" is refused even though the graph admits the edge.
    pub strict_upgrades_only: bool,
}

impl Default for MigrationPolicy {
    fn default() -> Self {
        MigrationPolicy {
            strict_upgrades_only: true,
        }
    }
}

/// Downgrade law: refuses `from` -> `to` when the GRAPH marks the migration
/// kind "downgrade" AND the policy's `strict_upgrades_only` flag is set. A
/// pair the graph does not admit at all is not this gate's concern —
/// [`admission_checked`] refuses it first on the rotation path.
pub fn assert_not_downgrade_under(
    from: CryptoProfile,
    to: CryptoProfile,
    policy: &MigrationPolicy,
) -> Result<(), RotationError> {
    if policy.strict_upgrades_only && migration_kind(from, to) == Some("downgrade") {
        Err(RotationError::MigrationRefused(from, to))
    } else {
        Ok(())
    }
}

/// [`assert_not_downgrade_under`] under the default policy
/// (`strict_upgrades_only = true`).
pub fn assert_not_downgrade(from: CryptoProfile, to: CryptoProfile) -> Result<(), RotationError> {
    assert_not_downgrade_under(from, to, &MigrationPolicy::default())
}

/// The rotation ceremony type: admission gate first, then the successor-key
/// signature over the domain-digested record, never the reverse order. The
/// free functions carry the bodies; the type namespaces the ceremony for
/// callers that want the two-step ritual under one name.
pub struct RotationCeremony;

impl RotationCeremony {
    /// CLASSICAL -> HYBRID rotation ([`rotate_es256_to_hybrid`]).
    pub fn rotate_es256_to_hybrid(
        old: &Es256SigningKey,
        hybrid_secret: &HybridSecret,
        at: u64,
    ) -> Result<RotationRecord, RotationError> {
        rotate_es256_to_hybrid(old, hybrid_secret, at)
    }

    /// Record verification ([`verify_rotation`]).
    pub fn verify_rotation(
        record: &RotationRecord,
        hybrid_pk_es256: &[u8],
        hybrid_pk_mldsa65: &[u8],
    ) -> Result<bool, RotationError> {
        verify_rotation(record, hybrid_pk_es256, hybrid_pk_mldsa65)
    }
}

/// Rotates a CLASSICAL (ES256) key to a HYBRID (ES256+ML-DSA-65) successor at
/// epoch time `at`. Order of judgment:
///
/// 1. Admission FIRST: CLASSICAL -> HYBRID must be a graph migration row,
///    else [`RotationError::MigrationRefused`] (the table decides, not this
///    function).
/// 2. Successor identity: `new_key_id` is the trust-plane fingerprint of the
///    hybrid public material (classical half then PQ half, bound to
///    `AlgorithmId::HybridEs256MlDsa65`).
/// 3. Successor attests takeover: BOTH hybrid halves sign
///    `digest(ROTATION_DOMAIN_TAG, [ROTATION_DIGEST_LABEL, jcs(fields)])`.
pub fn rotate_es256_to_hybrid(
    old: &Es256SigningKey,
    hybrid_secret: &HybridSecret,
    at: u64,
) -> Result<RotationRecord, RotationError> {
    let from = CryptoProfile::Classical;
    let to = CryptoProfile::Hybrid;
    admission_checked(from, to)?;

    let es256_pk = hybrid_secret.es256.public_key_sec1();
    let mldsa65_kp = ml_dsa65_from_seed(&hybrid_secret.mldsa65_seed);
    let new_key_id = KeyId::from_fingerprint(&fingerprint_public_key(
        AlgorithmId::HybridEs256MlDsa65,
        &PublicKeyMaterial::Hybrid {
            es256: es256_pk.clone(),
            mldsa65: mldsa65_kp.public.clone(),
        },
    ))
    .to_string();

    let fields = SignedRotationFields {
        old_key_id: KeyId::from_fingerprint(&old.key_id_fingerprint()).to_string(),
        new_key_id: new_key_id.clone(),
        from_profile: from.as_str().to_ascii_uppercase(),
        to_profile: to.as_str().to_ascii_uppercase(),
        rotated_at: at,
    };
    let record_digest = rotation_digest(&fields)?;
    let signature = hybrid_sign(hybrid_secret, &record_digest)
        .map_err(|e| RotationError::Provider(e.to_string()))?;

    Ok(RotationRecord {
        old_key_id: fields.old_key_id,
        new_key_id,
        from_profile: fields.from_profile,
        to_profile: fields.to_profile,
        rotated_at: fields.rotated_at,
        successor_signature: encode_hybrid_signature(&signature),
    })
}

/// Verifies a rotation record against the successor hybrid public material
/// (the ES256 SEC1 point and the raw ML-DSA-65 encoded public key). Order of
/// judgment:
///
/// 1. Identity digest BEFORE the signature check: `new_key_id` must equal the
///    trust-plane fingerprint of the SUPPLIED hybrid public material. A
///    mismatch proves the verification keys are not this record's successor
///    keys — [`RotationError::SuccessorSignatureInvalid`] — before any
///    signature work runs.
/// 2. Signature check: both hybrid halves must verify over the recomputed
///    domain digest of the signed fields. With (1) having established the
///    keys are the right ones, a signature that still fails means the record
///    content no longer matches what the successor signed —
///    [`RotationError::RecordTampered`].
///
/// `Ok(true)` only for a fully verified record; every other outcome is a
/// typed refusal.
pub fn verify_rotation(
    record: &RotationRecord,
    hybrid_pk_es256: &[u8],
    hybrid_pk_mldsa65: &[u8],
) -> Result<bool, RotationError> {
    // 1. Identity digest check (before any signature work).
    let expected_new_key_id = KeyId::from_fingerprint(&fingerprint_public_key(
        AlgorithmId::HybridEs256MlDsa65,
        &PublicKeyMaterial::Hybrid {
            es256: hybrid_pk_es256.to_vec(),
            mldsa65: hybrid_pk_mldsa65.to_vec(),
        },
    ))
    .to_string();
    if record.new_key_id != expected_new_key_id {
        return Err(RotationError::SuccessorSignatureInvalid);
    }

    // 2. Signature check over the recomputed digest.
    let signature = decode_hybrid_signature(&record.successor_signature)
        .ok_or(RotationError::SuccessorSignatureInvalid)?;
    let record_digest = rotation_digest(&record.signed_fields())?;
    let ok = hybrid_verify(
        hybrid_pk_es256,
        hybrid_pk_mldsa65,
        &record_digest,
        &signature,
    )
    .map_err(|e| RotationError::Provider(e.to_string()))?;
    if ok {
        Ok(true)
    } else {
        Err(RotationError::RecordTampered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG;

    const OLD_SEED: [u8; 32] = [0x11u8; 32];
    const HYBRID_ES_SEED: [u8; 32] = [0x42u8; 32];
    const HYBRID_ML_SEED: [u8; 32] = [0xA5u8; 32];
    const OTHER_ES_SEED: [u8; 32] = [0x77u8; 32];
    const AT: u64 = 1_700_000_000;

    fn old_key() -> Es256SigningKey {
        Es256SigningKey::from_seed(&OLD_SEED).expect("valid p256 scalar")
    }

    fn hybrid_secret() -> HybridSecret {
        HybridSecret {
            es256: Es256SigningKey::from_seed(&HYBRID_ES_SEED).expect("valid p256 scalar"),
            mldsa65_seed: HYBRID_ML_SEED,
        }
    }

    fn successor_pks(secret: &HybridSecret) -> (Vec<u8>, Vec<u8>) {
        (
            secret.es256.public_key_sec1(),
            ml_dsa65_from_seed(&secret.mldsa65_seed).public,
        )
    }

    fn rotated_record() -> RotationRecord {
        rotate_es256_to_hybrid(&old_key(), &hybrid_secret(), AT).expect("admitted rotation")
    }

    /// Parses a rendered profileName ("CLASSICAL" | "HYBRID" | "PQC") into
    /// its enum identity; `None` for a name outside the closed lattice.
    #[must_use]
    fn profile_from_name(name: &str) -> Option<CryptoProfile> {
        if name.eq_ignore_ascii_case(CryptoProfile::Classical.as_str()) {
            Some(CryptoProfile::Classical)
        } else if name.eq_ignore_ascii_case(CryptoProfile::Hybrid.as_str()) {
            Some(CryptoProfile::Hybrid)
        } else if name.eq_ignore_ascii_case(CryptoProfile::Pqc.as_str()) {
            Some(CryptoProfile::Pqc)
        } else {
            None
        }
    }

    #[test]
    fn migrations_table_renders_the_graph_facts() {
        assert_eq!(
            MIGRATIONS,
            &[
                ("CLASSICAL", "HYBRID", "upgrade"),
                ("HYBRID", "CLASSICAL", "downgrade"),
                ("HYBRID", "PQC", "upgrade"),
            ]
        );
        assert_eq!(ROTATION_DOMAIN_TAG, DOMAIN_TAG);
    }

    #[test]
    fn admission_is_a_pure_function_of_the_table() {
        let profiles = [
            CryptoProfile::Classical,
            CryptoProfile::Hybrid,
            CryptoProfile::Pqc,
        ];
        for &from in &profiles {
            for &to in &profiles {
                let in_table = MIGRATIONS.iter().any(|(f, t, _)| {
                    profile_from_name(f) == Some(from) && profile_from_name(t) == Some(to)
                });
                assert_eq!(
                    admission_allowed(from, to),
                    in_table,
                    "admission must follow the table for {from:?} -> {to:?}"
                );
            }
        }
        // The two-step law: CLASSICAL -> PQC direct is NOT a graph row.
        assert!(!admission_allowed(
            CryptoProfile::Classical,
            CryptoProfile::Pqc
        ));
    }

    #[test]
    fn classical_to_pqc_direct_is_migration_refused_two_step_required() {
        let err = admission_checked(CryptoProfile::Classical, CryptoProfile::Pqc)
            .expect_err("CLASSICAL -> PQC is not a graph migration row");
        assert!(
            matches!(
                err,
                RotationError::MigrationRefused(CryptoProfile::Classical, CryptoProfile::Pqc)
            ),
            "got {err:?}"
        );
        assert_eq!(
            err.to_string(),
            "migration Classical -> Pqc is not admitted by the profile migration policy"
        );
    }

    #[test]
    fn downgrade_law_holds_for_every_table_row_under_both_policies() {
        for &(from_name, to_name, kind) in MIGRATIONS {
            let from = profile_from_name(from_name).expect("rendered name is a profile");
            let to = profile_from_name(to_name).expect("rendered name is a profile");
            let strict = MigrationPolicy::default();
            assert!(strict.strict_upgrades_only, "default must be strict");
            let strict_result = assert_not_downgrade_under(from, to, &strict);
            let lax_result = assert_not_downgrade_under(
                from,
                to,
                &MigrationPolicy {
                    strict_upgrades_only: false,
                },
            );
            match kind {
                "upgrade" => {
                    assert!(
                        strict_result.is_ok(),
                        "{from:?} -> {to:?} is an upgrade; strict must admit"
                    );
                    assert!(lax_result.is_ok());
                }
                "downgrade" => {
                    assert!(
                        matches!(
                            strict_result,
                            Err(RotationError::MigrationRefused(a, b)) if a == from && b == to
                        ),
                        "strict must refuse the downgrade row {from:?} -> {to:?}"
                    );
                    assert!(
                        lax_result.is_ok(),
                        "lax policy admits the graph's downgrade row {from:?} -> {to:?}"
                    );
                }
                other => panic!("closed kind vocabulary violated: {other}"),
            }
        }
        // The two-argument convenience applies the default (strict) policy.
        assert!(assert_not_downgrade(CryptoProfile::Hybrid, CryptoProfile::Pqc).is_ok());
        assert!(assert_not_downgrade(CryptoProfile::Hybrid, CryptoProfile::Classical).is_err());
        // A pair the graph does not admit at all is not the downgrade gate's
        // concern (admission_checked refuses it first on the rotation path).
        assert!(assert_not_downgrade(CryptoProfile::Classical, CryptoProfile::Pqc).is_ok());
    }

    #[test]
    fn rotate_then_verify_round_trip() {
        let record = rotated_record();
        let secret = hybrid_secret();
        let (es_pk, ml_pk) = successor_pks(&secret);

        assert_eq!(record.from_profile, "CLASSICAL");
        assert_eq!(record.to_profile, "HYBRID");
        assert_eq!(record.rotated_at, AT);
        assert_eq!(
            record.old_key_id,
            KeyId::from_fingerprint(&old_key().key_id_fingerprint()).to_string()
        );
        assert_eq!(
            record.new_key_id,
            KeyId::from_fingerprint(&fingerprint_public_key(
                AlgorithmId::HybridEs256MlDsa65,
                &PublicKeyMaterial::Hybrid {
                    es256: es_pk.clone(),
                    mldsa65: ml_pk.clone()
                },
            ))
            .to_string()
        );

        assert!(
            verify_rotation(&record, &es_pk, &ml_pk).expect("verify"),
            "the honest round trip must verify before any tamper falsifier"
        );
    }

    #[test]
    fn wrong_successor_pk_is_successor_signature_invalid() {
        let record = rotated_record();
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        let wrong_es = Es256SigningKey::from_seed(&OTHER_ES_SEED)
            .expect("valid p256 scalar")
            .public_key_sec1();

        let err = verify_rotation(&record, &wrong_es, &ml_pk)
            .expect_err("record must refuse under a wrong ES256 pk");
        assert!(
            matches!(err, RotationError::SuccessorSignatureInvalid),
            "got {err:?}"
        );

        let wrong_ml = ml_dsa65_from_seed(&[0x3Cu8; 32]).public;
        let err = verify_rotation(&record, &es_pk, &wrong_ml)
            .expect_err("record must refuse under a wrong ML-DSA-65 pk");
        assert!(
            matches!(err, RotationError::SuccessorSignatureInvalid),
            "got {err:?}"
        );
    }

    #[test]
    fn truncated_signature_is_successor_signature_invalid() {
        let mut record = rotated_record();
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        record
            .successor_signature
            .truncate(ML_DSA_65_SIGNATURE_LEN - 1);
        let err = verify_rotation(&record, &es_pk, &ml_pk)
            .expect_err("bytes shorter than the fixed PQ half cannot decode");
        assert!(
            matches!(err, RotationError::SuccessorSignatureInvalid),
            "got {err:?}"
        );
    }

    #[test]
    fn tampered_rotated_at_is_record_tampered() {
        let mut record = rotated_record();
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        record.rotated_at ^= 0x01;
        let err = verify_rotation(&record, &es_pk, &ml_pk)
            .expect_err("a flipped timestamp breaks the signed digest");
        assert!(matches!(err, RotationError::RecordTampered), "got {err:?}");
    }

    #[test]
    fn tampered_profile_field_is_record_tampered() {
        let mut record = rotated_record();
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        record.to_profile = String::from("PQC");
        let err = verify_rotation(&record, &es_pk, &ml_pk)
            .expect_err("a flipped profile field breaks the signed digest");
        assert!(matches!(err, RotationError::RecordTampered), "got {err:?}");
    }

    #[test]
    fn structurally_broken_signature_is_a_typed_provider_refusal() {
        let mut record = rotated_record();
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        // Same total length (the fixed-length split still succeeds), broken
        // DER structure in the classical half: the provider layer refuses.
        record.successor_signature[0] ^= 0xFF;
        let err = verify_rotation(&record, &es_pk, &ml_pk)
            .expect_err("broken DER must be refused by the ES256 provider");
        assert!(matches!(err, RotationError::Provider(_)), "got {err:?}");
    }

    #[test]
    fn rotated_at_beyond_jcs_precision_is_a_typed_serialization_refusal() {
        let err = rotate_es256_to_hybrid(&old_key(), &hybrid_secret(), u64::MAX)
            .expect_err("2^64-1 exceeds JCS double precision (2^53)");
        assert!(
            matches!(err, RotationError::Serialization(_)),
            "got {err:?}"
        );
    }

    #[test]
    fn provider_and_serialization_variants_render_typed_messages() {
        let pqc = crate::crypto_trust_pqc::PqcError::MalformedSignature;
        assert_eq!(
            RotationError::Provider(pqc.to_string()).to_string(),
            "provider: malformed signature"
        );
        assert_eq!(
            RotationError::Serialization("overflow".to_string()).to_string(),
            "serialization: overflow"
        );
    }

    #[test]
    fn rotation_records_are_deterministic_and_serde_round_trip() {
        let a = rotated_record();
        let b = rotated_record();
        let bytes_a = serde_json::to_vec(&a).expect("serialize record a");
        let bytes_b = serde_json::to_vec(&b).expect("serialize record b");
        assert_eq!(
            bytes_a, bytes_b,
            "RFC 6979 ES256 + domain-bound ML-DSA rnd give identical record bytes"
        );
        let back: RotationRecord =
            serde_json::from_str(&serde_json::to_string(&a).expect("to string"))
                .expect("deserialize record");
        assert_eq!(back, a);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        assert!(verify_rotation(&back, &es_pk, &ml_pk).expect("verify round-tripped record"));
    }

    #[test]
    fn ceremony_namespace_delegates_to_the_free_functions() {
        let record = RotationCeremony::rotate_es256_to_hybrid(&old_key(), &hybrid_secret(), AT)
            .expect("ceremony rotate");
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        assert!(
            RotationCeremony::verify_rotation(&record, &es_pk, &ml_pk).expect("ceremony verify")
        );
    }
}
