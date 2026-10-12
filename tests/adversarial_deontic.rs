#![cfg(feature = "crypto-trust")]
//! Adversarial Skeptic Lane 4: Deontic Context Parameter Enforcement (FR-PQC-06)
//!
//! Validates:
//! 1. Domain-separated context string construction:
//!    `Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`
//! 2. ML-DSA-65 signature binding to deontic context.
//! 3. Strict fail-closed rejection on mismatched jurisdiction, constraint hash, or epoch (`0xE301`).
//! 4. Strict rejection on bit-flipped signature or message (`0xE301`).
//! 5. ML-KEM decapsulation failure mapping to machine code `0xE302`.
//! 6. Formal oracle divergence mapping to machine code `0xE303`.

use affidavit::crypto_trust_pqc::{
    ml_dsa65_from_seed, ML_DSA_65_PUBLIC_KEY_LEN, ML_DSA_65_SEED_LEN, ML_DSA_65_SIGNATURE_LEN,
};
use affidavit::deontic::{
    construct_deontic_context, construct_deontic_context_raw, enforce_deontic_context_match,
    sign_with_deontic_context, verify_formal_oracle_consensus,
    verify_formal_oracle_signature_agreement, verify_ml_kem_decapsulation,
    verify_with_deontic_context, verify_with_deontic_context_params, DeonticContext,
    CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH, CODE_FORMAL_ORACLE_DIVERGENCE,
    CODE_ML_KEM_DECAPSULATION_FAILURE, MAX_ML_DSA_CONTEXT_LEN, STRATA_CTX_PREFIX,
};

#[test]
fn test_deontic_context_string_construction_canonical() {
    let jurisdiction = "JURISDICTION-SOVEREIGN-COURT-01";
    let constraint_hash: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
        0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20,
    ];
    let lease_epoch = 0x0102030405060708u64;

    let ctx_bytes = construct_deontic_context(jurisdiction, &constraint_hash, lease_epoch)
        .expect("canonical context construction succeeds");

    // Must start with "STRATA:"
    assert_eq!(&ctx_bytes[..7], STRATA_CTX_PREFIX);

    // Followed by Jurisdiction
    let jur_len = jurisdiction.len();
    assert_eq!(&ctx_bytes[7..7 + jur_len], jurisdiction.as_bytes());

    // Followed by DeonticConstraintHash
    let hash_start = 7 + jur_len;
    assert_eq!(&ctx_bytes[hash_start..hash_start + 32], &constraint_hash);

    // Followed by LeaseEpoch (8 bytes big-endian)
    let epoch_start = hash_start + 32;
    assert_eq!(&ctx_bytes[epoch_start..epoch_start + 8], &lease_epoch.to_be_bytes());
    assert_eq!(ctx_bytes.len(), 7 + jur_len + 32 + 8);
}

#[test]
fn test_deontic_signature_validation_with_matching_context() {
    let seed = [0xA5u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"STRATA sovereign covenant receipt: deontic obligation certified";

    let context = DeonticContext::new("DEFENSE-ZONE-PACIFIC", [0x42u8; 32], 9901);

    let sig = sign_with_deontic_context(&kp.seed, msg, &context)
        .expect("signing with deontic context succeeds");
    assert_eq!(sig.len(), ML_DSA_65_SIGNATURE_LEN);
    assert_eq!(kp.public.len(), ML_DSA_65_PUBLIC_KEY_LEN);

    // Verify under exact matching context
    let result = verify_with_deontic_context(&kp.public, msg, &sig, &context);
    assert!(result.is_ok(), "Authentic signature under matching deontic context must verify");

    // Also verify via params helper
    let params_res = verify_with_deontic_context_params(
        &kp.public,
        msg,
        &sig,
        "DEFENSE-ZONE-PACIFIC",
        &[0x42u8; 32],
        9901,
    );
    assert!(params_res.is_ok(), "verify_with_deontic_context_params must succeed");
}

#[test]
fn test_deontic_fail_closed_rejection_on_mismatched_jurisdiction() {
    let seed = [0xB2u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"classified payload scoped strictly to JURISDICTION-FED";

    let auth_context = DeonticContext::new("JURISDICTION-FED", [0x77u8; 32], 123);
    let sig = sign_with_deontic_context(&kp.seed, msg, &auth_context).expect("signing succeeds");

    // Adversarial attempt: present receipt under JURISDICTION-CIVIL
    let adversarial_context = DeonticContext::new("JURISDICTION-CIVIL", [0x77u8; 32], 123);
    let verify_res = verify_with_deontic_context(&kp.public, msg, &sig, &adversarial_context);

    assert!(verify_res.is_err(), "Mismatched jurisdiction MUST reject fail-closed");
    let err = verify_res.unwrap_err();
    assert_eq!(err.machine_code(), CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH);
    assert_eq!(err.machine_code(), 0xE301);
}

#[test]
fn test_deontic_fail_closed_rejection_on_mismatched_constraint_hash() {
    let seed = [0xC3u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"payload subject to specific constraint hash set";

    let auth_hash = [0x55u8; 32];
    let auth_context = DeonticContext::new("JURISDICTION-SEC", auth_hash, 456);
    let sig = sign_with_deontic_context(&kp.seed, msg, &auth_context).expect("signing succeeds");

    // Adversarial sweep: 1-bit mutation across multiple byte positions in constraint hash
    for byte_idx in [0, 15, 31] {
        for bit in 0..8 {
            let mut tampered_hash = auth_hash;
            tampered_hash[byte_idx] ^= 1 << bit;

            let adversarial_context = DeonticContext::new("JURISDICTION-SEC", tampered_hash, 456);
            let verify_res = verify_with_deontic_context(&kp.public, msg, &sig, &adversarial_context);

            assert!(verify_res.is_err(), "Bit-flipped constraint hash MUST reject");
            let err = verify_res.unwrap_err();
            assert_eq!(
                err.machine_code(),
                0xE301,
                "Constraint hash mismatch must yield 0xE301"
            );
        }
    }
}

#[test]
fn test_deontic_fail_closed_rejection_on_mismatched_epoch() {
    let seed = [0xD4u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"lease-bounded authorization ticket";

    let epoch_initial = 1_000_000u64;
    let auth_context = DeonticContext::new("JURISDICTION-TIME", [0x33u8; 32], epoch_initial);
    let sig = sign_with_deontic_context(&kp.seed, msg, &auth_context).expect("signing succeeds");

    // Adversarial replay in subsequent epochs
    let invalid_epochs = [
        epoch_initial - 1, // Past epoch (stale replay)
        epoch_initial + 1, // Future epoch (premature assertion)
        0u64,              // Genesis epoch
        u64::MAX,          // Epoch overflow
    ];

    for &bad_epoch in &invalid_epochs {
        let replay_context = DeonticContext::new("JURISDICTION-TIME", [0x33u8; 32], bad_epoch);
        let verify_res = verify_with_deontic_context(&kp.public, msg, &sig, &replay_context);

        assert!(
            verify_res.is_err(),
            "Epoch mismatch (epoch {bad_epoch}) MUST reject fail-closed"
        );
        let err = verify_res.unwrap_err();
        assert_eq!(err.machine_code(), 0xE301);
    }
}

#[test]
fn test_deontic_fail_closed_rejection_on_corrupted_signature() {
    let seed = [0xE5u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"signature corruption detection probe";

    let context = DeonticContext::new("JURISDICTION-PROOF", [0x88u8; 32], 77);
    let sig = sign_with_deontic_context(&kp.seed, msg, &context).expect("signing succeeds");

    // 1-bit flip in signature
    let mut bad_sig = sig.clone();
    bad_sig[0] ^= 0x01;

    let res = verify_with_deontic_context(&kp.public, msg, &bad_sig, &context);
    assert!(res.is_err(), "Corrupted signature MUST be rejected");
    assert_eq!(res.unwrap_err().machine_code(), 0xE301);
}

#[test]
fn test_deontic_fail_closed_rejection_on_corrupted_message() {
    let seed = [0xF6u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"authentic original message";

    let context = DeonticContext::new("JURISDICTION-TAMPER", [0x99u8; 32], 88);
    let sig = sign_with_deontic_context(&kp.seed, msg, &context).expect("signing succeeds");

    let tampered_msg = b"tampered altered message";
    let res = verify_with_deontic_context(&kp.public, tampered_msg, &sig, &context);
    assert!(res.is_err(), "Tampered message MUST be rejected");
    assert_eq!(res.unwrap_err().machine_code(), 0xE301);
}

#[test]
fn test_deontic_context_overflow_rejection() {
    // FIPS 204 enforces |ctx| <= 255. Ensure overflow fails with 0xE301.
    let oversized_jur = vec![b'X'; MAX_ML_DSA_CONTEXT_LEN + 1];
    let res = construct_deontic_context_raw(&oversized_jur, &[0u8; 32], &8u64.to_be_bytes());

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.machine_code(), 0xE301);
}

#[test]
fn test_deontic_ml_kem_decapsulation_failure_machine_code_0xe302() {
    let expected = [0x11u8; 32];
    let mismatched = [0x22u8; 32];

    let err = verify_ml_kem_decapsulation(&expected, &mismatched).unwrap_err();
    assert_eq!(
        err.machine_code(),
        CODE_ML_KEM_DECAPSULATION_FAILURE,
        "ML-KEM decapsulation failure MUST map to 0xE302"
    );
    assert_eq!(err.machine_code(), 0xE302);
}

#[test]
fn test_deontic_formal_oracle_divergence_machine_code_0xe303() {
    let err = verify_formal_oracle_consensus(true, false).unwrap_err();
    assert_eq!(
        err.machine_code(),
        CODE_FORMAL_ORACLE_DIVERGENCE,
        "Formal oracle divergence MUST map to 0xE303"
    );
    assert_eq!(err.machine_code(), 0xE303);

    let sig_err = verify_formal_oracle_signature_agreement(b"sig-a", b"sig-b").unwrap_err();
    assert_eq!(sig_err.machine_code(), 0xE303);
}

#[test]
fn test_deontic_enforce_context_match_all_fields() {
    let c1 = DeonticContext::new("ALPHA", [0x01; 32], 10);
    assert!(enforce_deontic_context_match(&c1, &c1).is_ok());

    // Jurisdiction mismatch
    let c_diff_j = DeonticContext::new("BETA", [0x01; 32], 10);
    assert_eq!(enforce_deontic_context_match(&c1, &c_diff_j).unwrap_err().machine_code(), 0xE301);

    // Hash mismatch
    let c_diff_h = DeonticContext::new("ALPHA", [0x02; 32], 10);
    assert_eq!(enforce_deontic_context_match(&c1, &c_diff_h).unwrap_err().machine_code(), 0xE301);

    // Epoch mismatch
    let c_diff_e = DeonticContext::new("ALPHA", [0x01; 32], 11);
    assert_eq!(enforce_deontic_context_match(&c1, &c_diff_e).unwrap_err().machine_code(), 0xE301);
}
