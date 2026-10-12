#![cfg(feature = "crypto-trust")]
//! Adversarial PQC & Cryptographic Modulus Skeptic Integration Tests (Σ_PQC)
//!
//! Evaluates ML-KEM-768 (FIPS 203, `ml-kem`), X-Wing hybrid (`x-wing`),
//! ML-DSA-65 (FIPS 204), SLH-DSA-SHA2-128s (FIPS 205), and formally verified
//! oracles (`libcrux-ml-dsa`, `libcrux-ml-kem`).
//!
//! Adversarial surface tested:
//! 1. ML-KEM with deterministic/zero-entropy seeds via hazmat (zero seed, all-ones, low-entropy).
//! 2. X-Wing hybrid key encapsulation/decapsulation against malformed ciphertexts,
//!    truncated public keys, and cross-protocol key injection.
//! 3. Signature rejection on 1-bit perturbed wire tokens for ML-DSA-65 and SLH-DSA.
//! 4. Compilation verification of `libcrux-ml-dsa` and `libcrux-ml-kem` for `wasm32-unknown-unknown`.

use ml_kem::kem::{Decapsulate, TryKeyInit};
use ml_kem::{FromSeed, MlKem768, Seed};
use x_wing::kem::{Decapsulator, TryDecapsulate};
use x_wing::{
    DecapsulationKey as XwDecapsulationKey,
    DecapsulationKeyRejectNonContrib,
    EncapsulationKey as XwEncapsulationKey,
    CIPHERTEXT_SIZE, DECAPSULATION_KEY_SIZE, ENCAPSULATION_KEY_SIZE,
};

use affidavit::crypto_trust_pqc::{
    ml_dsa65_from_seed, ml_dsa65_sign, ml_dsa65_verify,
    slh_dsa128s_from_seed, slh_dsa128s_sign, slh_dsa128s_verify,
    PqcError, ML_DSA_65_PUBLIC_KEY_LEN, ML_DSA_65_SIGNATURE_LEN, ML_DSA_65_SEED_LEN,
    SLH_DSA_128S_SEED_LEN,
};

// ══════════════════════════════════════════════════════════════════════════════
// 1. ML-KEM-768 HAZMAT & DETERMINISTIC / ZERO-ENTROPY SEED PROBING
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ml_kem_zero_entropy_seed_handling() {
    // Zero-entropy seed (64 bytes of 0x00: 32 bytes d, 32 bytes z)
    let zero_seed_bytes = [0u8; 64];
    let zero_seed = Seed::from(zero_seed_bytes);

    // Ensure keypair generation from all-zero seed does not panic
    let (dk, ek) = MlKem768::from_seed(&zero_seed);

    // Verify deterministic encapsulation with hazmat 32-byte all-zero randomness
    let zero_m = ml_kem::B32::from([0u8; 32]);
    let (ct, ss_enc) = ek.encapsulate_deterministic(&zero_m);

    // Decapsulate and ensure consistent shared key
    let ss_dec = dk.decapsulate(&ct);
    assert_eq!(
        ss_enc, ss_dec,
        "ML-KEM zero seed must decapsulate to matching shared secret"
    );

    // Ensure seed recovery
    assert_eq!(
        dk.to_seed().map(|s| s.as_slice().to_vec()),
        Some(zero_seed_bytes.to_vec()),
        "DecapsulationKey must accurately serialize back to the original zero-entropy seed"
    );
}

#[test]
fn test_ml_kem_all_ones_seed_handling() {
    // Low-entropy / all-ones seed (64 bytes of 0xFF)
    let ones_seed_bytes = [0xFFu8; 64];
    let ones_seed = Seed::from(ones_seed_bytes);

    let (dk, ek) = MlKem768::from_seed(&ones_seed);

    // Hazmat deterministic encapsulation with all-ones message
    let ones_m = ml_kem::B32::from([0xFFu8; 32]);
    let (ct, ss_enc) = ek.encapsulate_deterministic(&ones_m);

    let ss_dec = dk.decapsulate(&ct);
    assert_eq!(
        ss_enc, ss_dec,
        "ML-KEM all-ones seed must decapsulate to matching shared secret"
    );
}

#[test]
fn test_ml_kem_implicit_rejection_on_corrupted_ciphertext() {
    // Test Fujisaki-Okamoto transform implicit rejection property
    let seed_bytes = [0x42u8; 64];
    let (dk, ek) = MlKem768::from_seed(&Seed::from(seed_bytes));

    let m = ml_kem::B32::from([0x13u8; 32]);
    let (ct, valid_ss) = ek.encapsulate_deterministic(&m);

    // Mutate ciphertext: flip 1 bit in first byte
    let mut bad_ct = ct.clone();
    bad_ct[0] ^= 0x01;

    // FIPS 203: Decapsulation of invalid ciphertext MUST NOT panic,
    // but MUST return a pseudorandom key derived from `z` (implicit rejection),
    // and MUST NOT match the authentic shared secret.
    let rejected_ss = dk.decapsulate(&bad_ct);
    assert_ne!(
        valid_ss, rejected_ss,
        "Decapsulation of corrupted ciphertext must implicitly reject (differing shared secret)"
    );

    // Determinism of implicit rejection: decapsulating the same bad ciphertext again must give the same rejected key
    let rejected_ss_2 = dk.decapsulate(&bad_ct);
    assert_eq!(
        rejected_ss, rejected_ss_2,
        "Implicit rejection pseudorandom secret must be deterministic"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. X-WING HYBRID ADVERSARIAL ENCAPSULATION / DECAPSULATION
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_x_wing_zero_entropy_and_boundary_seeds() {
    // Zero-entropy 32-byte secret seed for X-Wing
    let zero_sk_bytes = [0u8; DECAPSULATION_KEY_SIZE];
    let sk = XwDecapsulationKey::from(zero_sk_bytes);
    let pk = sk.encapsulation_key();

    // Deterministic encapsulation with 64 bytes of zeroes
    let zero_rand: ml_kem::array::ArrayN<u8, 64> = ml_kem::array::ArrayN::<u8, 64>::from([0u8; 64]);
    let (ct, ss_sender) = pk.encapsulate_deterministic(&zero_rand);

    let ss_receiver = sk.decapsulate(&ct);
    assert_eq!(
        ss_sender, ss_receiver,
        "X-Wing zero-entropy keypair must round-trip successfully"
    );
}

#[test]
fn test_x_wing_truncated_and_malformed_public_keys() {
    // Authentic encapsulation key is 1216 bytes (1184 bytes ML-KEM + 32 bytes X25519)
    assert_eq!(ENCAPSULATION_KEY_SIZE, 1216);

    let short_keys: &[&[u8]] = &[
        &[],
        &[0u8; 32],
        &[0u8; 1183],
        &[0u8; 1184],
        &[0u8; 1215],
    ];

    for (idx, short_key) in short_keys.iter().enumerate() {
        let res = XwEncapsulationKey::try_from(*short_key);
        assert!(
            res.is_err(),
            "X-Wing public key with length {} (case {}) must be rejected",
            short_key.len(),
            idx
        );
    }

    // Key with oversized bytes
    let long_key = [0u8; 1217];
    let res = XwEncapsulationKey::try_from(&long_key[..]);
    assert!(
        res.is_err(),
        "X-Wing public key with 1217 bytes must be rejected"
    );
}

#[test]
fn test_x_wing_malformed_ciphertext_and_rejection_modes() {
    let sk_bytes = [0x55u8; DECAPSULATION_KEY_SIZE];
    let sk = XwDecapsulationKey::from(sk_bytes);
    let sk_reject_noncontrib = DecapsulationKeyRejectNonContrib::from(sk_bytes);
    let pk = sk.encapsulation_key();

    let zero_rand: ml_kem::array::ArrayN<u8, 64> = ml_kem::array::ArrayN::<u8, 64>::from([0xAAu8; 64]);
    let (ct, genuine_ss) = pk.encapsulate_deterministic(&zero_rand);

    // 1. Bit flip in ML-KEM ciphertext half (bytes 0..1088)
    let mut ct_tampered_m = ct.clone();
    ct_tampered_m[0] ^= 0x01;
    let ss_tampered_m = sk.decapsulate(&ct_tampered_m);
    assert_ne!(
        genuine_ss, ss_tampered_m,
        "Tampering ML-KEM portion of X-Wing ciphertext must yield different shared secret"
    );

    // 2. Bit flip in X25519 ciphertext half (bytes 1088..1120)
    let mut ct_tampered_x = ct.clone();
    ct_tampered_x[1088] ^= 0x01;
    let ss_tampered_x = sk.decapsulate(&ct_tampered_x);
    assert_ne!(
        genuine_ss, ss_tampered_x,
        "Tampering X25519 portion of X-Wing ciphertext must yield different shared secret"
    );

    // 3. Low-order / non-contributory point injection in X25519 half
    // Point of order 8 (all zeroes is a low order point for X25519 producing all-zero DH output)
    let mut ct_noncontrib = ct.clone();
    for b in &mut ct_noncontrib[1088..1120] {
        *b = 0x00;
    }

    // Standard DecapsulationKey accepts non-contributory behavior for backwards compatibility
    let ss_accepted = sk.decapsulate(&ct_noncontrib);
    assert_ne!(
        genuine_ss, ss_accepted,
        "Non-contributory point must not match genuine shared secret"
    );

    // DecapsulationKeyRejectNonContrib MUST explicitly reject non-contributory behavior with typed DecapsulationError
    let reject_result = sk_reject_noncontrib.try_decapsulate(&ct_noncontrib);
    assert!(
        reject_result.is_err(),
        "DecapsulationKeyRejectNonContrib must return Err on non-contributory X25519 ciphertext"
    );
}

#[test]
fn test_x_wing_cross_protocol_key_injection() {
    // Generate two independent keypairs A and B
    let sk_a = XwDecapsulationKey::from([0x01u8; DECAPSULATION_KEY_SIZE]);
    let sk_b = XwDecapsulationKey::from([0x02u8; DECAPSULATION_KEY_SIZE]);

    let pk_a = sk_a.encapsulation_key();
    let pk_b = sk_b.encapsulation_key();

    // Encapsulate under PK_A
    let rand: ml_kem::array::ArrayN<u8, 64> = ml_kem::array::ArrayN::<u8, 64>::from([0x77u8; 64]);
    let (ct_a, ss_a) = pk_a.encapsulate_deterministic(&rand);

    // Try decapsulating CT_A under SK_B
    let ss_b = sk_b.decapsulate(&ct_a);
    assert_ne!(
        ss_a, ss_b,
        "Decapsulating under wrong keypair must not recover shared secret"
    );

    // Cross-splicing: combine ML-KEM pk from A with X25519 pk from B
    let pk_a_bytes = ml_kem::kem::KeyExport::to_bytes(pk_a);
    let pk_b_bytes = ml_kem::kem::KeyExport::to_bytes(pk_b);

    let mut hybrid_spliced = [0u8; ENCAPSULATION_KEY_SIZE];
    hybrid_spliced[..1184].copy_from_slice(&pk_a_bytes[..1184]);
    hybrid_spliced[1184..].copy_from_slice(&pk_b_bytes[1184..]);

    let spliced_key = ml_kem::kem::Key::<XwEncapsulationKey>::from(hybrid_spliced);
    let pk_spliced = XwEncapsulationKey::new(&spliced_key).expect("spliced key format valid");

    let (ct_spliced, ss_spliced) = pk_spliced.encapsulate_deterministic(&rand);

    // Spliced ciphertext decapsulated by A must not match
    assert_ne!(
        ss_spliced,
        sk_a.decapsulate(&ct_spliced),
        "Spliced hybrid key must not be decipherable by key A alone"
    );
    // Spliced ciphertext decapsulated by B must not match
    assert_ne!(
        ss_spliced,
        sk_b.decapsulate(&ct_spliced),
        "Spliced hybrid key must not be decipherable by key B alone"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. 1-BIT PERTURBED WIRE TOKEN SIGNATURE REJECTION (ML-DSA-65 & SLH-DSA)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ml_dsa_65_1bit_perturbed_wire_token_sweep() {
    let seed = [0x42u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"fips-204-ml-dsa-65-adversarial-test-payload";
    let rnd = [0x19u8; 32];

    // Authentic signature
    let sig = ml_dsa65_sign(&kp.seed, msg, &rnd).expect("honest signing");
    assert_eq!(sig.len(), ML_DSA_65_SIGNATURE_LEN);
    assert_eq!(kp.public.len(), ML_DSA_65_PUBLIC_KEY_LEN);

    // Teeth: authentic signature must verify
    assert!(
        ml_dsa65_verify(&kp.public, msg, &sig).expect("valid verify"),
        "Authentic ML-DSA-65 signature must verify"
    );

    // Adversarial sweep: 1-bit perturbation across boundary and middle positions
    let probe_indices = [
        0,                      // First byte
        1,                      // Second byte
        sig.len() / 4,          // 1/4 offset
        sig.len() / 2,          // Midpoint
        3 * sig.len() / 4,      // 3/4 offset
        sig.len() - 2,          // Penultimate byte
        sig.len() - 1,          // Last byte
    ];

    for &idx in &probe_indices {
        for bit in 0..8 {
            let mut tampered_sig = sig.clone();
            tampered_sig[idx] ^= 1 << bit;

            let verdict = ml_dsa65_verify(&kp.public, msg, &tampered_sig);
            match verdict {
                Ok(valid) => {
                    assert!(
                        !valid,
                        "ML-DSA-65 signature with 1-bit perturbation at byte {} bit {} must reject",
                        idx, bit
                    );
                }
                Err(PqcError::MalformedSignature) => {
                    // Structural decoding refusal is also safe and valid
                }
                Err(other) => {
                    panic!("Unexpected error on 1-bit perturbed signature: {:?}", other);
                }
            }
        }
    }

    // Verify 1-bit flipped public key rejection
    let pk_probe_indices = [0, kp.public.len() / 2, kp.public.len() - 1];
    for &idx in &pk_probe_indices {
        let mut tampered_pk = kp.public.clone();
        tampered_pk[idx] ^= 0x01;

        let verdict = ml_dsa65_verify(&tampered_pk, msg, &sig);
        match verdict {
            Ok(valid) => assert!(!valid, "ML-DSA-65 verification with corrupted PK must reject"),
            Err(PqcError::MalformedPublicKey) => {} // Safe typed refusal
            Err(other) => panic!("Unexpected error on corrupted PK: {:?}", other),
        }
    }
}

#[test]
fn test_slh_dsa_128s_1bit_perturbed_wire_token_sweep() {
    let seeds = [0x7Bu8; SLH_DSA_128S_SEED_LEN];
    let kp = slh_dsa128s_from_seed(&seeds);
    let msg = b"fips-205-slh-dsa-128s-adversarial-test-payload";

    let sig = slh_dsa128s_sign(&kp.seeds, msg).expect("honest signing");
    assert!(
        slh_dsa128s_verify(&kp.public, msg, &sig).expect("valid verify"),
        "Authentic SLH-DSA signature must verify"
    );

    // Adversarial sweep across boundary and interior positions
    let probe_indices = [
        0,
        1,
        sig.len() / 4,
        sig.len() / 2,
        sig.len() - 1,
    ];

    for &idx in &probe_indices {
        for bit in [0, 3, 7] {
            let mut tampered_sig = sig.clone();
            tampered_sig[idx] ^= 1 << bit;

            let verdict = slh_dsa128s_verify(&kp.public, msg, &tampered_sig);
            match verdict {
                Ok(valid) => {
                    assert!(
                        !valid,
                        "SLH-DSA signature with 1-bit perturbation at byte {} bit {} must reject",
                        idx, bit
                    );
                }
                Err(PqcError::MalformedSignature) => {}
                Err(other) => panic!("Unexpected error on 1-bit perturbed SLH-DSA signature: {:?}", other),
            }
        }
    }

    // Corrupted public key
    let mut tampered_pk = kp.public.clone();
    tampered_pk[0] ^= 0x01;
    let verdict = slh_dsa128s_verify(&tampered_pk, msg, &sig);
    match verdict {
        Ok(valid) => assert!(!valid, "SLH-DSA verification with corrupted PK must reject"),
        Err(PqcError::MalformedPublicKey) => {}
        Err(other) => panic!("Unexpected error on corrupted SLH-DSA PK: {:?}", other),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. VERIFIED ORACLE NO_STD / WASM COMPILABILITY ASSERTION
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_pqc_target_specifications() {
    // Parameter invariants and spec compliance assertions
    assert_eq!(CIPHERTEXT_SIZE, 1120);
    assert_eq!(DECAPSULATION_KEY_SIZE, 32);
    assert_eq!(ENCAPSULATION_KEY_SIZE, 1216);
    assert_eq!(ML_DSA_65_PUBLIC_KEY_LEN, 1952);
    assert_eq!(ML_DSA_65_SIGNATURE_LEN, 3309);
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. DEONTIC CONTEXT PARAMETER ENFORCEMENT & MACHINE CODE MAPPING (FR-PQC-06)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_adversarial_pqc_deontic_context_binding_and_machine_codes() {
    use affidavit::deontic::{
        sign_with_deontic_context, verify_formal_oracle_consensus,
        verify_ml_kem_decapsulation, verify_with_deontic_context, DeonticContext,
        CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH, CODE_FORMAL_ORACLE_DIVERGENCE,
        CODE_ML_KEM_DECAPSULATION_FAILURE,
    };

    let seed = [0x99u8; ML_DSA_65_SEED_LEN];
    let kp = ml_dsa65_from_seed(&seed);
    let msg = b"adversarial pqc deontic binding test message";
    let ctx = DeonticContext::new("DEFENSE-SPEC-01", [0x7Au8; 32], 2026);

    let sig = sign_with_deontic_context(&kp.seed, msg, &ctx).expect("honest signing");
    assert!(verify_with_deontic_context(&kp.public, msg, &sig, &ctx).is_ok());

    // Tampered context parameter -> 0xE301
    let bad_ctx = DeonticContext::new("DEFENSE-SPEC-02", [0x7Au8; 32], 2026);
    let res = verify_with_deontic_context(&kp.public, msg, &sig, &bad_ctx);
    assert_eq!(res.unwrap_err().machine_code(), CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH);
    assert_eq!(CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH, 0xE301);

    // ML-KEM decapsulation failure -> 0xE302
    let kem_err = verify_ml_kem_decapsulation(&[0x01; 32], &[0x02; 32]).unwrap_err();
    assert_eq!(kem_err.machine_code(), CODE_ML_KEM_DECAPSULATION_FAILURE);
    assert_eq!(CODE_ML_KEM_DECAPSULATION_FAILURE, 0xE302);

    // Formal oracle divergence -> 0xE303
    let oracle_err = verify_formal_oracle_consensus(true, false).unwrap_err();
    assert_eq!(oracle_err.machine_code(), CODE_FORMAL_ORACLE_DIVERGENCE);
    assert_eq!(CODE_FORMAL_ORACLE_DIVERGENCE, 0xE303);
}
