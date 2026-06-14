//! Adversarial proof: why tampered receipts cannot survive verify.
//!
//! Shows three attack vectors an adversary might attempt, and which pipeline
//! stage catches each attempt. Educational — proves the anti-forgery property
//! without relying on test assertions alone.
//!
//! Attack 1: flip a payload commitment byte → chain_integrity catches it
//!   (the stored chain_hash no longer matches the recomputed one)
//! Attack 2: wrong format version → check_format catches it
//! Attack 3: reorder events then re-seal the chain → continuity catches it
//!   (seq numbers are now out of order relative to positions)

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Blake3Hash;
use affidavit::verifier::verify;

fn honest_receipt() -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for (ty, payload) in [
        ("emit", b"code v1".as_slice()),
        ("transform", b"binary"),
        ("release", b"signed"),
    ] {
        let ev = build_event(ty, vec![object_ref("artifact", "artifact")], payload, &mut counter)
            .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

fn main() {
    // Baseline: honest receipt must accept all stages.
    let receipt = honest_receipt();
    let v = verify(&receipt);
    assert!(v.accepted, "honest receipt must accept: {}", v.reason);
    println!("baseline: honest receipt ACCEPTED — {}", v.reason);

    // Attack 1: flip a payload commitment byte.
    // The stored chain_hash was computed from the original commitment, so after
    // the flip it no longer matches what recompute_chain produces. chain_integrity
    // catches this because it recomputes rather than trusting the stored hash.
    let mut r1 = honest_receipt();
    let original_hex = r1.events[1].payload_commitment.as_hex().to_string();
    let mut chars: Vec<char> = original_hex.chars().collect();
    // Flip the first hex nibble (a→b or anything→a).
    chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
    let tampered_hex: String = chars.into_iter().collect();
    r1.events[1].payload_commitment = Blake3Hash::from_hex(tampered_hex);
    // We do NOT recompute chain_hash — the mismatch is what the stage catches.
    let v1 = verify(&r1);
    assert!(!v1.accepted, "flipped commitment must be rejected");
    let caught1 = v1
        .outcomes
        .iter()
        .find(|o| !o.passed)
        .map(|o| o.stage.as_str())
        .unwrap_or("(none)");
    println!("attack 1 (flip commitment byte): REJECTED at stage '{caught1}'");
    assert_eq!(caught1, "chain_integrity", "flipped commitment must trip chain_integrity");

    // Attack 2: wrong format version.
    // The verifier checks that format_version == "core/v1". Any other string
    // fails check_format at stage 2.
    let mut r2 = honest_receipt();
    r2.format_version = "evil/v99".to_string();
    let v2 = verify(&r2);
    assert!(!v2.accepted, "wrong format version must be rejected");
    let caught2 = v2
        .outcomes
        .iter()
        .find(|o| !o.passed)
        .map(|o| o.stage.as_str())
        .unwrap_or("(none)");
    println!("attack 2 (wrong format version): REJECTED at stage '{caught2}'");
    assert_eq!(caught2, "check_format", "wrong version must trip check_format");

    // Attack 3: reorder events then re-seal the chain.
    // After swapping events[0] and events[1], the seq numbers no longer match
    // their positions: position 0 has seq=1 and position 1 has seq=0.
    // Re-sealing makes chain_integrity pass (the chain now matches the swapped
    // bytes), so continuity is the catching stage.
    let mut r3 = honest_receipt();
    r3.events.swap(0, 1);
    // Re-seal so chain_integrity passes — continuity must catch the out-of-order seq.
    r3.chain_hash = recompute_chain(&r3.events).expect("recompute swapped chain");
    let v3 = verify(&r3);
    assert!(!v3.accepted, "reordered events must be rejected");
    let caught3 = v3
        .outcomes
        .iter()
        .find(|o| !o.passed)
        .map(|o| o.stage.as_str())
        .unwrap_or("(none)");
    println!("attack 3 (reorder + re-seal): REJECTED at stage '{caught3}'");
    assert_eq!(caught3, "continuity", "reordered seq must trip continuity");

    println!("\nOK: all three attack vectors are caught by the certify pipeline");
}
