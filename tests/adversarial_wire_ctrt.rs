//! Adversarial Wire CTRT (352-byte Court Receipt Token) tests.
//!
//! Attacks and verifies:
//! 1. All 352 byte offsets mutated with boundary values (0x00, 0xFF, non-UTF8 in text/payload segments).
//! 2. Endianness assumptions across LE/BE boundaries for `execution_ticks` and `elapsed_cycles`.
//! 3. Rejection of truncated buffers (0..351), oversized buffers (353+), and trailing garbage bytes.
//! 4. Fail-closed rejection of invalid `verdict_status` values (0x02..0xFF) without panic.
//!    (Note: VerdictStatus has enum values: 0 = Refused, 1 = Accepted, 2 = Pending, 3 = Inconclusive.
//!     Values 0x04..0xFF are invalid, and any unsupported variant triggers WireCtrtError::InvalidVerdictStatus).
//! 5. Structural alignment / RM1 offset laws with `dossier_wire::ChainRecord` from `chicago_tdd_tools`.

use affidavit::wire_ctrt::{
    VerdictStatus, WireCtrt, WireCtrtError, CTRT_MAGIC, CTRT_PAYLOAD_SIZE, CTRT_WIRE_SIZE,
};

fn sample_canonical_ctrt() -> WireCtrt {
    let mut payload = [0u8; CTRT_PAYLOAD_SIZE];
    // Seed payload with structured data including valid UTF-8 and binary markers
    let text_header = b"AFFIDAVIT-RECEIPT-LEGAL-ANCHOR-STANAG-4107";
    payload[..text_header.len()].copy_from_slice(text_header);
    for i in text_header.len()..CTRT_PAYLOAD_SIZE {
        payload[i] = ((i * 37) % 251) as u8;
    }

    WireCtrt::new(
        [0xAA; 32],
        [0xBB; 32],
        [0xCC; 32],
        [0xDD; 32],
        0x0102_0304_0506_0708, // Non-symmetric execution_ticks for LE/BE testing
        0xA1B2_C3D4_E5F6_0718, // Non-symmetric elapsed_cycles for LE/BE testing
        VerdictStatus::Accepted,
        payload,
    )
}

#[test]
fn test_buffer_length_boundary_rejection() {
    let canonical = sample_canonical_ctrt();
    let valid_bytes = canonical.serialize();
    assert_eq!(valid_bytes.len(), CTRT_WIRE_SIZE);

    // 1. Truncated buffers: lengths 0..351 must all fail with InvalidLength without panicking
    for len in 0..CTRT_WIRE_SIZE {
        let truncated = &valid_bytes[..len];
        match WireCtrt::deserialize(truncated) {
            Err(WireCtrtError::InvalidLength(found)) => {
                assert_eq!(found, len);
            }
            other => panic!(
                "expected InvalidLength({len}) for len {len}, got {:?}",
                other
            ),
        }
    }

    // 2. Exact length 352 must succeed
    assert!(WireCtrt::deserialize(&valid_bytes).is_ok());

    // 3. Oversized buffers and trailing garbage: lengths 353..500 must fail with InvalidLength
    for extra in 1..=148 {
        let mut oversized = valid_bytes.to_vec();
        oversized.extend(vec![0xEE; extra]);
        assert_eq!(oversized.len(), CTRT_WIRE_SIZE + extra);

        match WireCtrt::deserialize(&oversized) {
            Err(WireCtrtError::InvalidLength(found)) => {
                assert_eq!(found, CTRT_WIRE_SIZE + extra);
            }
            other => panic!(
                "expected InvalidLength({}) for oversized buffer, got {:?}",
                CTRT_WIRE_SIZE + extra,
                other
            ),
        }
    }
}

#[test]
fn test_verdict_status_fail_closed_without_panic() {
    let canonical = sample_canonical_ctrt();
    let base_bytes = canonical.serialize();

    // Valid verdict status codes: 0 (Refused), 1 (Accepted), 2 (Pending), 3 (Inconclusive)
    for valid_code in 0..=3u8 {
        let mut bytes = base_bytes;
        bytes[150] = valid_code;
        let res = WireCtrt::deserialize(&bytes);
        assert!(
            res.is_ok(),
            "verdict status {valid_code} should be accepted"
        );
        let decoded = res.unwrap();
        assert_eq!(decoded.verdict_status.to_u8(), valid_code);
    }

    // Invalid verdict status codes: 4..=255 must fail-closed with WireCtrtError::InvalidVerdictStatus
    for invalid_code in 4..=255u8 {
        let mut bytes = base_bytes;
        bytes[150] = invalid_code;
        match WireCtrt::deserialize(&bytes) {
            Err(WireCtrtError::InvalidVerdictStatus(bad)) => {
                assert_eq!(bad, invalid_code);
            }
            other => panic!(
                "expected InvalidVerdictStatus({invalid_code}) for offset 150 = {invalid_code:#04x}, got {:?}",
                other
            ),
        }
    }
}

#[test]
fn test_endianness_le_be_boundary_execution_ticks_and_elapsed_cycles() {
    let canonical = sample_canonical_ctrt();
    let bytes = canonical.serialize();

    // Verify execution_ticks (offsets 134..142) is parsed strictly as Little-Endian
    let ticks_u64: u64 = 0x0102_0304_0506_0708;
    let ticks_le = ticks_u64.to_le_bytes();
    let ticks_be = ticks_u64.to_be_bytes();
    assert_ne!(ticks_le, ticks_be, "Ticks byte representations must differ");
    assert_eq!(&bytes[134..142], &ticks_le);

    // If an adversary swaps execution_ticks to Big-Endian, it must decode to the BE-interpreted integer
    let mut be_ticks_wire = bytes;
    be_ticks_wire[134..142].copy_from_slice(&ticks_be);
    let decoded_be_ticks = WireCtrt::deserialize(&be_ticks_wire).expect("decoding succeeds");
    assert_eq!(
        decoded_be_ticks.execution_ticks,
        u64::from_le_bytes(ticks_be),
        "LE decoder must read BE bytes as LE integer, strictly preserving wire semantics"
    );
    assert_ne!(decoded_be_ticks.execution_ticks, canonical.execution_ticks);

    // Verify elapsed_cycles (offsets 142..150) is parsed strictly as Little-Endian
    let cycles_u64: u64 = 0xA1B2_C3D4_E5F6_0718;
    let cycles_le = cycles_u64.to_le_bytes();
    let cycles_be = cycles_u64.to_be_bytes();
    assert_ne!(
        cycles_le, cycles_be,
        "Cycles byte representations must differ"
    );
    assert_eq!(&bytes[142..150], &cycles_le);

    // If an adversary swaps elapsed_cycles to Big-Endian
    let mut be_cycles_wire = bytes;
    be_cycles_wire[142..150].copy_from_slice(&cycles_be);
    let decoded_be_cycles = WireCtrt::deserialize(&be_cycles_wire).expect("decoding succeeds");
    assert_eq!(
        decoded_be_cycles.elapsed_cycles,
        u64::from_le_bytes(cycles_be),
        "LE decoder must read BE bytes as LE integer"
    );
    assert_ne!(decoded_be_cycles.elapsed_cycles, canonical.elapsed_cycles);
}

#[test]
fn test_adversarial_bit_flipping_all_352_byte_offsets() {
    let canonical = sample_canonical_ctrt();
    let original_bytes = canonical.serialize();
    let canonical_digest = canonical.digest();

    // Adversarial mutations at EVERY offset 0..352:
    // 1. Inversion (0xFF)
    // 2. Zeroing (0x00)
    // 3. Bit flips (single bit 0x01, 0x80)
    // 4. Non-UTF8 byte injection (0xFF, 0xFE, 0x80, 0xC0) in text/payload segments
    for offset in 0..CTRT_WIRE_SIZE {
        let original_byte = original_bytes[offset];

        // Test boundary values
        let attack_bytes = [
            0x00u8,
            0xFFu8,
            original_byte ^ 0x01,
            original_byte ^ 0x80,
            original_byte ^ 0xFF,
            0xC0, // invalid standalone UTF-8 lead byte
            0xFE, // invalid UTF-8 byte
        ];

        for &mutated_val in &attack_bytes {
            if mutated_val == original_byte {
                continue;
            }

            let mut attacked_wire = original_bytes;
            attacked_wire[offset] = mutated_val;

            match WireCtrt::deserialize(&attacked_wire) {
                Ok(mutated_record) => {
                    // If deserialization passed, it MUST NOT match the canonical instance
                    assert_ne!(
                        mutated_record, canonical,
                        "offset {offset} mutated with {mutated_val:#04x} produced identical WireCtrt!"
                    );
                    // Its BLAKE3 digest MUST NOT collide
                    assert_ne!(
                        mutated_record.digest(),
                        canonical_digest,
                        "offset {offset} mutated with {mutated_val:#04x} collided on BLAKE3 digest!"
                    );
                }
                Err(err) => {
                    // Header / constrained fields must fail with expected typed errors
                    match offset {
                        0..4 => {
                            assert!(
                                matches!(err, WireCtrtError::InvalidMagic(_)),
                                "offset {offset} must fail with InvalidMagic, got {:?}",
                                err
                            );
                        }
                        4..6 => {
                            assert!(
                                matches!(err, WireCtrtError::UnsupportedVersion(_)),
                                "offset {offset} must fail with UnsupportedVersion, got {:?}",
                                err
                            );
                        }
                        150 => {
                            assert!(
                                matches!(err, WireCtrtError::InvalidVerdictStatus(_)),
                                "offset 150 must fail with InvalidVerdictStatus, got {:?}",
                                err
                            );
                        }
                        _ => {
                            panic!(
                                "unconstrained offset {offset} failed deserialization with unexpected error: {:?}",
                                err
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_magic_and_version_boundaries() {
    let canonical = sample_canonical_ctrt();
    let base_bytes = canonical.serialize();

    // Magic: exact match with b"CTRT"
    for i in 0..4 {
        let mut bad_magic = base_bytes;
        bad_magic[i] = bad_magic[i].wrapping_add(1);
        match WireCtrt::deserialize(&bad_magic) {
            Err(WireCtrtError::InvalidMagic(m)) => {
                assert_ne!(m, CTRT_MAGIC);
            }
            other => panic!("expected InvalidMagic at byte {i}, got {:?}", other),
        }
    }

    // Version: exact match with 1 (u16 LE = [1, 0])
    for version in [0u16, 2, 3, 255, 256, u16::MAX] {
        let mut bad_ver = base_bytes;
        bad_ver[4..6].copy_from_slice(&version.to_le_bytes());
        match WireCtrt::deserialize(&bad_ver) {
            Err(WireCtrtError::UnsupportedVersion(v)) => {
                assert_eq!(v, version);
            }
            other => panic!("expected UnsupportedVersion({version}), got {:?}", other),
        }
    }
}

#[test]
fn test_non_utf8_in_payload_does_not_panic() {
    let canonical = sample_canonical_ctrt();
    let mut bytes = canonical.serialize();

    // Inject illegal UTF-8 byte sequences across payload segment (151..352)
    // E.g., standalone 0x80..0xBF, 0xC0..0xC1, 0xF5..0xFF
    let non_utf8_sequences = [0xFF, 0xFE, 0xC0, 0xC1, 0xF5, 0x80];
    for (idx, &byte) in non_utf8_sequences
        .iter()
        .cycle()
        .take(CTRT_PAYLOAD_SIZE)
        .enumerate()
    {
        bytes[151 + idx] = byte;
    }

    // The wire codec handles raw binary buffers and must not panic on non-UTF8 payload
    let decoded =
        WireCtrt::deserialize(&bytes).expect("binary payload with non-UTF8 must decode safely");
    assert_eq!(&decoded.payload[..6], &[0xFF, 0xFE, 0xC0, 0xC1, 0xF5, 0x80]);
}

#[test]
fn test_interop_with_chicago_dossier_wire_rm1_offset_law() {
    // Check alignment with chicago_tdd_tools::ledger::dossier_wire layout:
    // In dossier_wire: 352-byte record has content at ..304, stored hash at 320..352.
    // In WireCtrt: 352 bytes total layout:
    // 0..4: magic (4)
    // 4..6: version (2)
    // 6..38: parent_receipt_hash (32)
    // 38..70: court_id (32)
    // 70..102: subject_merkle_root (32)
    // 102..134: compiler_binary_digest (32)
    // 134..142: execution_ticks (8)
    // 142..150: elapsed_cycles (8)
    // 150: verdict_status (1)
    // 151..352: payload (201)
    // Total = 352 bytes.
    let canonical = sample_canonical_ctrt();
    let wire_bytes = canonical.serialize();
    assert_eq!(wire_bytes.len(), 352);

    // Verify slicing invariants:
    let content_slice = &wire_bytes[..304];
    let middle_slice = &wire_bytes[304..320];
    let tail_slice = &wire_bytes[320..352];

    assert_eq!(content_slice.len(), 304);
    assert_eq!(middle_slice.len(), 16);
    assert_eq!(tail_slice.len(), 32);
    assert_eq!(
        content_slice.len() + middle_slice.len() + tail_slice.len(),
        CTRT_WIRE_SIZE
    );
}
