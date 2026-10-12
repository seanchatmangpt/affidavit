//! Specification conformance test for STRATA-SPEC-26.10.12-AFFIDAVIT-SOVEREIGN.
//!
//! Validates:
//! 1. All numeric machine codes 0xE001 through 0xE402 against their respective error conditions.
//! 2. Fail-closed semantics across build profiles and rejection states.
//! 3. Absence of panics on arbitrary boundary, empty, corrupted, and oversized inputs.

#![cfg(all(feature = "crypto-trust", feature = "git-provenance"))]

use affidavit::git_merkle::GitMerkleError;
use affidavit::replay_nullifier::{
    compute_nullifier, EpochKey, ReplayNullifierError, ReplayNullifierRegistry,
};
use affidavit::trajectory_folding::TrajectoryFoldError;
use affidavit::wire_ctrt::{
    constant_time_eq, VerdictStatus, WireCtrt, WireCtrtError, CTRT_MAGIC, CTRT_VERSION_1,
    CTRT_WIRE_SIZE,
};

#[test]
fn test_numeric_machine_codes_0xe001_through_0xe402() {
    // ── 0xE001 .. 0xE004: Wire CTRT Error Codes ─────────────────────────
    // 0xE001: Payload length != 352 bytes
    let err_e001 = WireCtrtError::InvalidLength(351);
    assert_eq!(err_e001.machine_code(), 0xE001, "0xE001 must be InvalidLength");

    // 0xE002: Magic bytes mismatch (not CTRT)
    let err_e002 = WireCtrtError::InvalidMagic(*b"BADM");
    assert_eq!(err_e002.machine_code(), 0xE002, "0xE002 must be InvalidMagic");

    // 0xE003: Unsupported wire version identifier
    let err_e003 = WireCtrtError::UnsupportedVersion(99);
    assert_eq!(err_e003.machine_code(), 0xE003, "0xE003 must be UnsupportedVersion");

    // 0xE004: Unrecognized verdict status code
    let err_e004 = WireCtrtError::InvalidVerdictStatus(0xFF);
    assert_eq!(err_e004.machine_code(), 0xE004, "0xE004 must be InvalidVerdictStatus");

    // ── 0xE101 .. 0xE102: Replay Nullifier Error Codes ───────────────────
    // 0xE101: Nullifier collision within active epoch
    let err_e101 = ReplayNullifierError::ReplayDetected("deadbeef".to_string());
    assert_eq!(err_e101.machine_code(), 0xE101, "0xE101 must be ReplayDetected");

    // 0xE102: Persistent nullifier storage corrupt / deserialization error
    let err_e102 = ReplayNullifierError::InvalidPersistenceStorage("corrupt table".to_string());
    assert_eq!(err_e102.machine_code(), 0xE102, "0xE102 must be InvalidPersistenceStorage");

    // ── 0xE201 .. 0xE202: Trajectory Folding Error Codes ─────────────────
    // 0xE201: Parent linkage mismatch between receipts
    let err_e201 = TrajectoryFoldError::ParentLinkageMismatch {
        expected: "exp".to_string(),
        found: "fnd".to_string(),
    };
    assert_eq!(err_e201.machine_code(), 0xE201, "0xE201 must be ParentLinkageMismatch");

    // 0xE202: Incompatible verdict status during fold
    let err_e202 = TrajectoryFoldError::IncompatibleVerdict;
    assert_eq!(err_e202.machine_code(), 0xE202, "0xE202 must be IncompatibleVerdict");

    // ── 0xE301 .. 0xE303: Post-Quantum Cryptographic Error Codes ─────────
    // 0xE301: ML-DSA-65 signature check failure / verification failure
    let err_e301_pqc = affidavit::crypto_trust_pqc::PqcError::MalformedSignature;
    assert_eq!(err_e301_pqc.machine_code(), 0xE301, "0xE301 must be MalformedSignature / MlDsa failure");

    let err_e301_affidavit = affidavit::error::PqcError::Verification("bad signature".to_string());
    assert_eq!(err_e301_affidavit.machine_code(), 0xE301, "0xE301 must be Verification failure");

    // 0xE302: Decapsulation failure / malformed key
    let err_e302_pqc = affidavit::crypto_trust_pqc::PqcError::MalformedPublicKey;
    assert_eq!(err_e302_pqc.machine_code(), 0xE302, "0xE302 must be MalformedPublicKey / decapsulation failure");

    let err_e302_affidavit = affidavit::error::PqcError::Encapsulation("kem failure".to_string());
    assert_eq!(err_e302_affidavit.machine_code(), 0xE302, "0xE302 must be Encapsulation failure");

    // 0xE303: Formal oracle detected output divergence
    let err_e303_pqc = affidavit::crypto_trust_pqc::PqcError::SlhDsa("divergence".to_string());
    assert_eq!(err_e303_pqc.machine_code(), 0xE303, "0xE303 must be SlhDsa / formal divergence");

    let err_e303_affidavit = affidavit::error::PqcError::Chain(
        affidavit::error::ChainError::Decode(serde_json::from_str::<()>("invalid").unwrap_err())
    );
    assert_eq!(err_e303_affidavit.machine_code(), 0xE303, "0xE303 must be Chain divergence");

    // ── 0xE401 .. 0xE402: Git Merkle Subsystem Error Codes ───────────────
    // 0xE401: Git object not found in database
    let err_e401 = GitMerkleError::ObjectNotFound("missing_sha".to_string());
    assert_eq!(err_e401.machine_code(), 0xE401, "0xE401 must be ObjectNotFound");

    // 0xE402: Git tree corrupt or unparseable
    let err_e402 = GitMerkleError::TreeCorruptOrUnparseable("malformed_tree".to_string());
    assert_eq!(err_e402.machine_code(), 0xE402, "0xE402 must be TreeCorruptOrUnparseable");
}

#[test]
fn test_fail_closed_semantics_and_boundary_resistance() {
    // 1. Empty buffer rejection: fail closed with 0xE001
    let empty_buf: &[u8] = &[];
    match WireCtrt::deserialize(empty_buf) {
        Err(e) => {
            assert_eq!(e.machine_code(), 0xE001);
            assert_eq!(e, WireCtrtError::InvalidLength(0));
        }
        Ok(_) => panic!("Empty buffer must fail closed"),
    }

    // 2. Truncated buffer rejection (1..351 bytes): all fail closed with 0xE001
    for len in 1..CTRT_WIRE_SIZE {
        let mut buf = vec![0u8; len];
        if len >= 4 {
            buf[..4].copy_from_slice(&CTRT_MAGIC);
        }
        match WireCtrt::deserialize(&buf) {
            Err(e) => assert_eq!(e.machine_code(), 0xE001),
            Ok(_) => panic!("Truncated buffer of length {len} must fail closed"),
        }
    }

    // 3. Oversized buffer rejection (353..500 bytes): all fail closed with 0xE001
    for extra in 1..=148 {
        let len = CTRT_WIRE_SIZE + extra;
        let mut buf = vec![0u8; len];
        buf[..4].copy_from_slice(&CTRT_MAGIC);
        buf[4..6].copy_from_slice(&CTRT_VERSION_1.to_le_bytes());
        buf[150] = VerdictStatus::Accepted.to_u8();
        match WireCtrt::deserialize(&buf) {
            Err(e) => assert_eq!(e.machine_code(), 0xE001),
            Ok(_) => panic!("Oversized buffer of length {len} must fail closed"),
        }
    }

    // 4. Invalid magic rejection: fail closed with 0xE002
    let mut valid_token = vec![0u8; CTRT_WIRE_SIZE];
    valid_token[..4].copy_from_slice(b"BOGU");
    valid_token[4..6].copy_from_slice(&CTRT_VERSION_1.to_le_bytes());
    valid_token[150] = VerdictStatus::Accepted.to_u8();
    match WireCtrt::deserialize(&valid_token) {
        Err(e) => assert_eq!(e.machine_code(), 0xE002),
        Ok(_) => panic!("Invalid magic must fail closed"),
    }

    // 5. Unsupported version rejection: fail closed with 0xE003
    let mut bad_version = valid_token.clone();
    bad_version[..4].copy_from_slice(&CTRT_MAGIC);
    bad_version[4..6].copy_from_slice(&2u16.to_le_bytes());
    match WireCtrt::deserialize(&bad_version) {
        Err(e) => assert_eq!(e.machine_code(), 0xE003),
        Ok(_) => panic!("Version != 1 must fail closed"),
    }

    // 6. Invalid verdict status (0x04..=0xFF): all fail closed with 0xE004
    for invalid_status in 4u8..=255u8 {
        let mut bad_status = valid_token.clone();
        bad_status[..4].copy_from_slice(&CTRT_MAGIC);
        bad_status[4..6].copy_from_slice(&CTRT_VERSION_1.to_le_bytes());
        bad_status[150] = invalid_status;
        match WireCtrt::deserialize(&bad_status) {
            Err(e) => assert_eq!(e.machine_code(), 0xE004),
            Ok(_) => panic!("Invalid verdict status {invalid_status} must fail closed"),
        }
    }
}

#[test]
fn test_no_panics_on_adversarial_boundary_inputs() {
    // Fuzz all-zeros, all-ones, alternating bits across wire deserialization
    let boundary_patterns = [
        vec![0x00u8; CTRT_WIRE_SIZE],
        vec![0xFFu8; CTRT_WIRE_SIZE],
        vec![0xAAu8; CTRT_WIRE_SIZE],
        vec![0x55u8; CTRT_WIRE_SIZE],
    ];

    for pattern in &boundary_patterns {
        let result = WireCtrt::deserialize(pattern);
        assert!(result.is_err(), "Boundary pattern must fail closed without panic");
    }

    // Constant time branchless comparison never panics
    assert_eq!(constant_time_eq(&[], &[]), 1);
    assert_eq!(constant_time_eq(&[1, 2, 3], &[1, 2, 3]), 1);
    assert_eq!(constant_time_eq(&[1, 2, 3], &[1, 2, 4]), 0);
    assert_eq!(constant_time_eq(&[1, 2, 3], &[1, 2]), 0);

    // Nullifier engine boundary behavior
    let key: EpochKey = [0x5Au8; 32];
    let wire_bytes = [0x00u8; 352];
    let nullifier = compute_nullifier(&key, &wire_bytes);

    let mut registry = ReplayNullifierRegistry::new(key, 1024);
    assert!(registry.admit_nullifier(nullifier).is_ok());
    // Second observation must fail closed with 0xE101
    match registry.admit_nullifier(nullifier) {
        Err(e) => assert_eq!(e.machine_code(), 0xE101),
        Ok(_) => panic!("Replay must fail closed"),
    }
}
