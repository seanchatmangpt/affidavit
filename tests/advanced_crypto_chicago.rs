// Chicago witness: the advanced-crypto capability set is GENUINELY integrated
// and its admission laws hold under chicago-tdd-tools' own assertion macros.
//
// Failing-when-fake: `assert_ok!`, `assert_err!`, and `assert_in_range!` are
// re-exported from chicago_tdd_tools. Remove the dependency and this file
// does not COMPILE; a passing run is evidence each capability's typed
// refusal fires where the doctrine says it must.
#![cfg(feature = "advanced-crypto")]

use chicago_tdd_tools::{assert_err, assert_in_range, assert_ok};

use affidavit::authority_fence::{gate_do, AuthorityFence};
use affidavit::bls_aggregate::{aggregate_committee, verify_committee, CommitteeKey};
use affidavit::causal_graph::{CausalError, CausalGraph};
use affidavit::ed25519_witness::{verify_witness, WitnessKeyPair};
use affidavit::hlc::HlcClock;
use affidavit::policy_cedar::{GateVerdict, PolicyGate};
use affidavit::replay_filter::ReplayFilter;
use affidavit::seq_bitmap::SeqContiguityCertifier;
use affidavit::smt::{verify_absence, verify_inclusion, StateTree};
use affidavit::threshold_quorum::{
    aggregate_signature, generate_quorum, round1_commit, round2_sign, signing_package,
    verify_quorum,
};
use affidavit::wasm_court::{WasmCourt, WasmCourtError};
use affidavit::zk_range::{prove_range, verify_range};
use rand_core::OsRng;

fn key(seed: u8) -> [u8; 32] {
    let mut k = [0u8; 32];
    k[0] = seed;
    k
}

/// Shape a boolean law as a Result so chicago's typed assertion macros can
/// consume it.
fn law_holds(cond: bool) -> Result<(), &'static str> {
    if cond {
        Ok(())
    } else {
        Err("law violated")
    }
}

#[test]
fn chicago_smt_inclusion_and_absence_laws() {
    let mut tree = StateTree::new();
    let root = tree.insert(&key(0x80), &key(1)).expect("insert");
    let proof = tree
        .prove_inclusion(&key(0x80))
        .expect("present key proves");
    assert_ok!(
        law_holds(verify_inclusion(&root, &key(1), &proof)),
        "inclusion proof verifies under root"
    );
    assert_ok!(
        law_holds(!verify_inclusion(&root, &key(2), &proof)),
        "bound to the wrong value must fail"
    );

    let absence = tree.prove_absence(&key(0x7F)).expect("absence witness");
    assert_ok!(
        law_holds(verify_absence(&root, &key(1), &absence)),
        "absence witness verifies under root"
    );
    assert_ok!(
        law_holds(!verify_absence(&root, &key(2), &absence)),
        "absence witness must not admit a tombstone value mismatch"
    );
}

#[test]
fn chicago_authority_fence_refuses_revoked_and_admits_clean() {
    let mut fence = AuthorityFence::new();
    assert_ok!(fence.revoke(&key(0x80)), "revocation commits");
    assert_err!(
        fence.prepare_do_permit(&key(0x80)),
        "revoked id gets no permit"
    );
    let permit = fence
        .prepare_do_permit(&key(0x40))
        .expect("clean id permitted");
    assert_ok!(gate_do(&permit, fence.root()), "DO admitted");

    fence.revoke(&key(0x01)).expect("second revocation");
    assert_err!(gate_do(&permit, fence.root()), "stale permit refused");
}

#[test]
fn chicago_threshold_quorum_signs_and_binds() {
    let message = b"chicago quorum witness message";
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    // The zero signature must NOT verify — assert_err! is the court here.
    assert_err!(
        verify_quorum(&quorum.group_key, message, &[0u8; 64]),
        "zero signature refused"
    );

    let chosen: Vec<_> = quorum.shares.keys().copied().take(3).collect();
    let mut commitments = Vec::new();
    let mut nonces = Vec::new();
    for p in &chosen {
        let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
        nonces.push(n);
        commitments.push((*p, c));
    }
    let package = signing_package(&commitments, message, quorum.threshold).expect("package");
    let mut shares = Vec::new();
    for p in &chosen {
        let s = round2_sign(&quorum.shares[p], &nonces[shares.len()], &package).expect("round2");
        shares.push((*p, s));
    }
    let signature = aggregate_signature(
        &package,
        &shares,
        &quorum.public_key_package,
        quorum.threshold,
    )
    .expect("aggregate");
    assert_ok!(
        verify_quorum(&quorum.group_key, message, &signature),
        "group key verifies"
    );
    assert_err!(
        verify_quorum(&quorum.group_key, b"tampered message", &signature),
        "message binding enforced"
    );
}

#[test]
fn chicago_zk_range_proves_bounds_not_values() {
    const LABEL: &[u8] = b"affidavit:chicago:zk-range";
    let witness = prove_range(999, 32, LABEL, &mut OsRng).expect("in-range proves");
    assert_ok!(
        verify_range(&witness.commitment, &witness.proof, 32, LABEL),
        "bound verified without disclosure"
    );
    let mut tampered = witness.proof.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;
    assert_err!(
        verify_range(&witness.commitment, &tampered, 32, LABEL),
        "tampered proof refused"
    );
    assert_err!(
        prove_range(1 << 32, 32, LABEL, &mut OsRng),
        "over-ceiling value refused before proving"
    );
}

#[test]
fn chicago_causal_graph_refuses_cycles_and_orders_dags() {
    let mut dag = CausalGraph::new();
    dag.add_dependency("resolve", "construct");
    dag.add_dependency("construct", "actuate");
    let order = dag.topological_order().expect("DAG orders");
    assert_in_range!(order.len(), 3, 3);

    let mut cyclic = CausalGraph::new();
    cyclic.add_dependency(1u32, 2);
    cyclic.add_dependency(2, 1);
    assert_err!(cyclic.topological_order(), "cycle refused with typed error");
    assert!(matches!(
        cyclic.verify_acyclic(),
        Err(CausalError::CycleDetected(_))
    ));
}

#[test]
fn chicago_ed25519_witness_binds_signer() {
    let kp = WitnessKeyPair::generate();
    let sig = kp.sign(b"witness payload");
    assert_ok!(
        verify_witness(&kp.public(), b"witness payload", &sig),
        "own key verifies"
    );
    let other = WitnessKeyPair::generate();
    assert_err!(
        verify_witness(&other.public(), b"witness payload", &sig),
        "foreign key refused"
    );
}

#[test]
fn chicago_policy_gate_is_decidable() {
    let gate = PolicyGate::from_policies(
        "permit(principal == Agent::\"curator\", action == Action::\"emit\", resource);\n",
    )
    .expect("policy compiles");
    assert!(
        matches!(
            gate.evaluate(
                r#"Agent::"curator""#,
                r#"Action::"emit""#,
                r#"Receipt::"c1""#
            ),
            Ok(GateVerdict::Admitted)
        ),
        "permitted authority admitted"
    );
    assert!(
        matches!(
            gate.evaluate(r#"Agent::"other""#, r#"Action::"emit""#, r#"Receipt::"c1""#),
            Ok(GateVerdict::Refused(_))
        ),
        "unpermitted authority refused"
    );
    assert_err!(
        PolicyGate::from_policies("permit(broken"),
        "malformed policy is a typed parse refusal"
    );
}

#[test]
fn chicago_replay_screen_is_exact_on_negatives() {
    let mut rf = ReplayFilter::new(1_000);
    assert!(
        rf.witness(&key(7)).expect("fresh"),
        "first sight is not a replay"
    );
    assert!(
        !rf.witness(&key(7)).expect("capacity"),
        "second sight is replay"
    );
    assert_in_range!(rf.len(), 1, 1);
    assert!(rf.retract(&key(7)), "lease retraction");
    assert!(!rf.probably_seen(&key(7)), "negative screens are exact");
}

#[test]
fn chicago_seq_bitmap_names_the_gap() {
    let mut cert = SeqContiguityCertifier::new();
    for seq in [0u32, 1, 2, 4] {
        cert.record(seq).expect("fresh");
    }
    assert_err!(cert.verify_contiguous(), "gap refused");
    assert!(cert.record(2).is_err(), "duplicate refused");
}

#[test]
fn chicago_hlc_preserves_causality() {
    let mut clock = HlcClock::with_skew_bound(1_000);
    let first = clock.send();
    let received = clock.receive(first).expect("own stamp within skew");
    let next = clock.send();
    assert_ok!(
        law_holds(
            HlcClock::happened_before(first, received) && HlcClock::happened_before(received, next)
        ),
        "HLC order strictly preserves causality"
    );
}

#[test]
fn chicago_bls_committee_aggregates_non_interactively() {
    const CONTEXT: &[u8] = b"affidavit:chicago:bls";
    let members: Vec<CommitteeKey> = (0..5).map(|_| CommitteeKey::generate(&mut OsRng)).collect();
    let signatures: Vec<_> = members.iter().map(|m| m.sign(CONTEXT, b"admit")).collect();
    let public_keys: Vec<_> = members.iter().map(|m| m.public()).collect();
    let (sig, pk) = aggregate_committee(&signatures, &public_keys).expect("aggregate");
    assert_ok!(
        law_holds(verify_committee(CONTEXT, b"admit", &sig, &pk)),
        "one pairing check admits the committee"
    );
    assert_ok!(
        law_holds(!verify_committee(CONTEXT, b"forge", &sig, &pk)),
        "message binding holds under aggregation"
    );
}

#[test]
fn chicago_wasm_court_bounds_execution() {
    let court = WasmCourt::new().expect("court");
    let wat = r#"(module (func (export "run")))"#;
    let remaining = court
        .run_bounded(wat.as_bytes(), "run", 10_000)
        .expect("terminates");
    assert_in_range!(remaining, 1, 10_000);
    let burner = r#"(module (func (export "run") (loop (br 0))))"#;
    assert_err!(
        court.run_bounded(burner.as_bytes(), "run", 1_000),
        "infinite loop killed"
    );
    assert!(matches!(
        court.run_bounded(burner.as_bytes(), "run", 1_000),
        Err(WasmCourtError::OutOfFuel { budget: 1_000 })
    ));
}

#[test]
fn chicago_fast_path_gates_before_heavy_crypto() {
    use affidavit::authority_fence::fast_path::{
        verify_fast_path, AdmittedProof, ConsequenceClaim, FastPathRefusal,
    };
    use affidavit::hlc::HlcClock;
    use affidavit::replay_filter::ReplayFilter;
    use affidavit::threshold_quorum::{
        aggregate_signature, generate_quorum, round1_commit, round2_sign, signing_package,
        verify_quorum,
    };

    let mut fence = AuthorityFence::new();
    let mut replay = ReplayFilter::new(1_000);
    let mut clock = HlcClock::with_skew_bound(1_000);

    // Prepare the heavy witness NOW but verify it only AFTER the gate.
    let message = b"fast-path gated consequence";
    let quorum = generate_quorum(5, 3, &mut OsRng).expect("dealer");
    let chosen: Vec<_> = quorum.shares.keys().copied().take(3).collect();
    let mut commitments = Vec::new();
    let mut nonces = Vec::new();
    for p in &chosen {
        let (n, c) = round1_commit(&quorum.shares[p], &mut OsRng).expect("round1");
        nonces.push(n);
        commitments.push((*p, c));
    }
    let package = signing_package(&commitments, message, quorum.threshold).expect("package");
    let mut shares = Vec::new();
    for p in &chosen {
        let s = round2_sign(&quorum.shares[p], &nonces[shares.len()], &package).expect("round2");
        shares.push((*p, s));
    }
    let signature = aggregate_signature(
        &package,
        &shares,
        &quorum.public_key_package,
        quorum.threshold,
    )
    .expect("aggregate");

    // Stage the fast-path gate first.
    let stamp = clock.send();
    let claim = ConsequenceClaim {
        consequence_id: key(0xA1),
        authority_id: key(0xA2),
        received: stamp,
    };
    let proof: AdmittedProof = verify_fast_path(&mut fence, &mut replay, &mut clock, &claim)
        .expect("fast path admits clean claim");

    // Heavy cryptography runs only after the gate admitted.
    assert_ok!(
        verify_quorum(&quorum.group_key, message, &signature),
        "group signature holds"
    );

    // Re-confirm at the live root immediately before actuation.
    assert_ok!(
        law_holds(proof.confirm(&fence).is_ok()),
        "permit confirms at live root"
    );

    // Replay of the same consequence id: refused at the screen — the
    // refusal variant carries no witness, so no crypto was reachable.
    assert_eq!(
        verify_fast_path(&mut fence, &mut replay, &mut clock, &claim),
        Err(FastPathRefusal::Replay),
        "replay refused at the cheapest stage"
    );

    // Revoked authority: refused at the fence stage.
    fence.revoke(&key(0xA2)).expect("revoke the authority");
    let second = ConsequenceClaim {
        consequence_id: key(0xA3),
        authority_id: key(0xA2),
        received: clock.send(),
    };
    assert_eq!(
        verify_fast_path(&mut fence, &mut replay, &mut clock, &second),
        Err(FastPathRefusal::Revoked),
        "revoked authority refused at the fence stage"
    );
}
