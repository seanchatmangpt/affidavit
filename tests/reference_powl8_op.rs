// Reference witness: the compact POWL operator wire-format `Powl8Op` (u8
// discriminant) and its round-trip / validation law (COVERAGE.md §2 — POWL8 op).
//
// Powl8Op is a #[repr(u8)] companion to PowlNodeKind: discriminants 0..=8 map to
// operators; `TryFrom<u8>` refuses any other byte with Powl8OpError::InvalidDiscriminant.
// This witnesses: every valid byte decodes; the u8 round-trips; an out-of-range
// byte is refused by name (the wire-format validation law).

use core::convert::TryFrom;
use wasm4pm_compat::powl8_op::{Powl8Op, Powl8OpError};

#[test]
fn valid_discriminants_decode_and_round_trip() {
    // All 9 valid discriminants decode, and (op as u8) recovers the byte.
    for byte in 0u8..=8 {
        let op = Powl8Op::try_from(byte).expect("0..=8 are valid POWL8 ops");
        assert_eq!(op as u8, byte, "discriminant round-trips: {byte}");
    }
    // Spot-check named mappings.
    assert_eq!(Powl8Op::try_from(0).unwrap(), Powl8Op::NoOp);
    assert_eq!(Powl8Op::try_from(2).unwrap(), Powl8Op::Choice);
    assert_eq!(Powl8Op::try_from(8).unwrap(), Powl8Op::ChoiceGraph);
}

#[test]
fn out_of_range_byte_is_refused_by_name() {
    assert_eq!(Powl8Op::try_from(9), Err(Powl8OpError::InvalidDiscriminant));
    assert_eq!(Powl8Op::try_from(255), Err(Powl8OpError::InvalidDiscriminant));
}
