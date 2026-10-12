use affidavit::trajectory_folding::{
    fold_outcome, fold_receipts, CtrtReceipt, TrajectoryFoldError, COURT_RECORD_LEN,
    COURT_RECEIPT_V1_LEN,
};

fn create_receipt(
    parent: [u8; 32],
    ticks: u64,
    wall: u64,
    name: &str,
    outcome: u8,
    thermal: u8,
    budget_met: bool,
    positive_control: [u8; 32],
) -> CtrtReceipt {
    let mut flags = 0x02; // FLAG_FAIL_CLOSED
    if positive_control != [0u8; 32] {
        flags |= 0x01; // FLAG_POSITIVE_CONTROL
    }

    let mut r = CtrtReceipt {
        flags,
        subject_hash: *blake3::hash(name.as_bytes()).as_bytes(),
        parent_receipt_hash: parent,
        timestamp_ms: 1700000000000,
        run_id: 42,
        total_ticks: ticks,
        wall_ms: wall,
        thermal,
        outcome,
        budget_met,
        positive_control,
        metadata_digest: *blake3::hash(format!("meta_{name}").as_bytes()).as_bytes(),
        prev_self_hash: [0u8; 32],
        name: name.to_string(),
        chain_hash: [0u8; 32],
    };
    let encoded = r.encode().expect("valid encode");
    r.chain_hash.copy_from_slice(&encoded[COURT_RECEIPT_V1_LEN..]);
    r
}

#[test]
fn test_monoidal_associativity_ticks_wall_and_properties() {
    // Construct three sequentially linked valid receipts A, B, C
    let r_a = create_receipt([0u8; 32], 100, 20, "rA", 0, 1, true, [0u8; 32]);
    let r_b = create_receipt(r_a.chain_hash, 250, 40, "rB", 1, 2, true, [0u8; 32]);
    let r_c = create_receipt(r_b.chain_hash, 300, 60, "rC", 2, 3, false, [0u8; 32]);

    // (R_A ⊕ R_B)
    let r_ab = fold_receipts(&r_a, &r_b).expect("fold A and B");
    // (R_B ⊕ R_C)
    let r_bc = fold_receipts(&r_b, &r_c).expect("fold B and C");

    // Check (R_A ⊕ R_B) properties
    assert_eq!(r_ab.parent_receipt_hash, r_a.parent_receipt_hash);
    assert_eq!(r_ab.total_ticks, 350);
    assert_eq!(r_ab.wall_ms, 60);

    // Monoidal accumulator properties must be associative across (A ⊕ B) ⊕ C vs A ⊕ (B ⊕ C):
    // 1. Total ticks: (100 + 250) + 300 == 100 + (250 + 300) = 650
    let total_ticks_ab_c = r_ab.total_ticks.saturating_add(r_c.total_ticks);
    let total_ticks_a_bc = r_a.total_ticks.saturating_add(r_bc.total_ticks);
    assert_eq!(total_ticks_ab_c, total_ticks_a_bc);
    assert_eq!(total_ticks_ab_c, 650);

    // 2. Wall time: (20 + 40) + 60 == 20 + (40 + 60) = 120
    let wall_ab_c = r_ab.wall_ms.saturating_add(r_c.wall_ms);
    let wall_a_bc = r_a.wall_ms.saturating_add(r_bc.wall_ms);
    assert_eq!(wall_ab_c, wall_a_bc);
    assert_eq!(wall_ab_c, 120);

    // 3. Outcomes (severity-ordered join, SX3-F1): the numeric wire tags are not
    //    severity-ordered (Fail = tag 1 outranks Skip = tag 2), so the join is
    //    `fold_outcome`, not `u8::max`. A=Pass(0), B=Fail(1), C=Skip(2):
    //    join(join(0,1),2) == join(0,join(1,2)) == Fail (1) — Fail masks Skip.
    assert_eq!(
        fold_outcome(r_ab.outcome, r_c.outcome),
        fold_outcome(r_a.outcome, r_bc.outcome)
    );
    assert_eq!(fold_outcome(r_ab.outcome, r_c.outcome), 1);

    // 6. Full-field associativity of the fold operator itself (R29 law): the folded
    //    receipt is the root-anchored summary speaking for the chain tip, so
    //    (A ⊕ B) ⊕ C == A ⊕ (B ⊕ C) must hold on EVERY field, not just accumulators.
    let ab_c = fold_receipts(&r_ab, &r_c).expect("fold (A⊕B)⊕C");
    let a_bc = fold_receipts(&r_a, &r_bc).expect("fold A⊕(B⊕C)");
    assert_eq!(ab_c.flags, a_bc.flags);
    assert_eq!(ab_c.subject_hash, a_bc.subject_hash);
    assert_eq!(ab_c.parent_receipt_hash, a_bc.parent_receipt_hash);
    assert_eq!(ab_c.timestamp_ms, a_bc.timestamp_ms);
    assert_eq!(ab_c.run_id, a_bc.run_id);
    assert_eq!(ab_c.total_ticks, a_bc.total_ticks);
    assert_eq!(ab_c.wall_ms, a_bc.wall_ms);
    assert_eq!(ab_c.thermal, a_bc.thermal);
    assert_eq!(ab_c.outcome, a_bc.outcome);
    assert_eq!(ab_c.budget_met, a_bc.budget_met);
    assert_eq!(ab_c.positive_control, a_bc.positive_control);
    assert_eq!(ab_c.metadata_digest, a_bc.metadata_digest);
    assert_eq!(ab_c.prev_self_hash, a_bc.prev_self_hash);
    assert_eq!(ab_c.name, a_bc.name);
    assert_eq!(ab_c.chain_hash, a_bc.chain_hash);
    // Root-anchored, tip-speaking: parent from A, chain from C.
    assert_eq!(ab_c.parent_receipt_hash, r_a.parent_receipt_hash);
    assert_eq!(ab_c.chain_hash, r_c.chain_hash);
    // Tip-derived identity fields come from C, not concatenations.
    assert_eq!(ab_c.name, r_c.name);
    assert_eq!(ab_c.run_id, r_c.run_id);
    assert_eq!(ab_c.timestamp_ms, r_c.timestamp_ms);

    // 4. Budget met (boolean conjunction): (true && true) && false == true && (true && false)
    assert_eq!(r_ab.budget_met && r_c.budget_met, r_a.budget_met && r_bc.budget_met);
    assert_eq!(r_ab.budget_met && r_c.budget_met, false);

    // 5. Thermal (semilattice max): max(max(1, 2), 3) == max(1, max(2, 3))
    assert_eq!(r_ab.thermal.max(r_c.thermal), r_a.thermal.max(r_bc.thermal));
    assert_eq!(r_ab.thermal.max(r_c.thermal), 3);
}

#[test]
fn test_parent_linkage_mismatch_rejection_and_no_laundering() {
    let r_a = create_receipt([0u8; 32], 100, 10, "A", 0, 1, true, [0u8; 32]);
    // Create unlinked receipt with bogus parent hash
    let r_b_bogus = create_receipt([0xde; 32], 200, 20, "B_bogus", 0, 1, true, [0u8; 32]);

    // Unlinked receipts must be rejected
    let err = fold_receipts(&r_a, &r_b_bogus).unwrap_err();
    match err {
        TrajectoryFoldError::ParentLinkageMismatch { expected, found } => {
            let exp_hex: String = r_a.chain_hash.iter().map(|b| format!("{b:02x}")).collect();
            let fnd_hex: String = r_b_bogus.parent_receipt_hash.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(expected, exp_hex);
            assert_eq!(found, fnd_hex);
        }
        other => panic!("expected ParentLinkageMismatch, got {:?}", other),
    }

    // Launder attempt: attempting to fold an already folded receipt with a disconnected parent
    let r_b_valid = create_receipt(r_a.chain_hash, 200, 20, "B", 0, 1, true, [0u8; 32]);
    let r_folded = fold_receipts(&r_a, &r_b_valid).expect("valid fold");

    // Attempt to launder disconnected receipt C against r_folded
    let r_c_unlinked = create_receipt([0xca; 32], 300, 30, "C_unlinked", 0, 1, true, [0u8; 32]);
    assert!(matches!(
        fold_receipts(&r_folded, &r_c_unlinked).unwrap_err(),
        TrajectoryFoldError::ParentLinkageMismatch { .. }
    ));
}

#[test]
fn test_arithmetic_overflow_saturating_add_u64_max() {
    let r_a = create_receipt([0u8; 32], u64::MAX, u64::MAX - 10, "max_a", 0, 1, true, [0u8; 32]);
    let r_b = create_receipt(r_a.chain_hash, 50, 100, "overflow_b", 0, 1, true, [0u8; 32]);

    // fold_receipts uses saturating_add:
    // a.total_ticks.saturating_add(b.total_ticks) -> u64::MAX
    // a.wall_ms.saturating_add(b.wall_ms) -> u64::MAX
    let folded = fold_receipts(&r_a, &r_b).expect("fold should saturate gracefully without panicking");

    assert_eq!(folded.total_ticks, u64::MAX, "total_ticks must saturate to u64::MAX");
    assert_eq!(folded.wall_ms, u64::MAX, "wall_ms must saturate to u64::MAX");

    // Verify wire encoding and roundtrip decode survives u64::MAX fields intact
    let encoded = folded.encode().expect("encode saturated receipt");
    assert_eq!(encoded.len(), COURT_RECORD_LEN);

    let decoded = CtrtReceipt::from_bytes(&encoded).expect("decode saturated receipt");
    assert_eq!(decoded.total_ticks, u64::MAX);
    assert_eq!(decoded.wall_ms, u64::MAX);
}
