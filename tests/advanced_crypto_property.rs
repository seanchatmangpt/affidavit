//! Property-based invariant tests for the advanced-crypto capability set.
//!
//! Generated inputs stress the two properties the doctrine cares most about:
//! inclusion proofs never falsely verify, and structural refusals never
//! false-positive on lawful structure. (The `arbitrary` crate is not
//! integrated: this repository has no `cargo-fuzz` harness for it to feed.)
#![cfg(feature = "advanced-crypto")]

use proptest::prelude::*;

use affidavit::causal_graph::CausalGraph;
use affidavit::seq_bitmap::SeqContiguityCertifier;
use affidavit::smt::{verify_absence, verify_inclusion, StateTree};
use affidavit::zk_range::{prove_range, verify_range};

fn key_pair_strategy() -> impl Strategy<Value = ([u8; 32], [u8; 32])> {
    (prop::collection::vec(any::<u8>(), 64)).prop_map(|bytes| {
        let mut k = [0u8; 32];
        let mut v = [0u8; 32];
        k.copy_from_slice(&bytes[..32]);
        v.copy_from_slice(&bytes[32..]);
        (k, v)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn smt_every_committed_key_proves_and_no_other_value_binds(
        entries in prop::collection::vec(key_pair_strategy(), 1..12),
        foreign in key_pair_strategy(),
    ) {
        let mut tree = StateTree::new();
        let mut roots = Vec::new();
        for (k, v) in &entries {
            roots.push(tree.insert(k, v).expect("insert"));
        }
        let root = tree.root().expect("non-empty");
        let _ = roots;
        for (k, v) in &entries {
            let proof = tree.prove_inclusion(k).expect("proof for committed key");
            prop_assert!(verify_inclusion(&root, v, &proof), "own value binds");
            if foreign.0 != *v {
                prop_assert!(
                    !verify_inclusion(&root, &foreign.0, &proof),
                    "foreign value must not bind"
                );
            }
        }
    }

    #[test]
    fn smt_absence_only_for_uncommitted_keys(
        entries in prop::collection::vec(key_pair_strategy(), 1..8),
        probe in key_pair_strategy(),
    ) {
        let mut tree = StateTree::new();
        for (k, v) in &entries {
            tree.insert(k, v).expect("insert");
        }
        let root = tree.root().expect("non-empty");
        let committed = entries.iter().any(|(k, _)| *k == probe.0);
        if committed {
            prop_assert!(tree.prove_absence(&probe.0).is_err(), "committed key has no absence witness");
        } else {
            let witness = tree.prove_absence(&probe.0).expect("absence witness");
            let neighbor_value = tree.get(&witness.neighbor_key).expect("get").expect("present");
            prop_assert!(verify_absence(&root, &neighbor_value, &witness), "absence verifies");
        }
    }

    #[test]
    fn strictly_forward_dags_always_admit_and_order(
        edges in prop::collection::vec((0u8..24, 0u8..24), 0..48),
    ) {
        let mut g = CausalGraph::new();
        for (a, b) in &edges {
            if a < b {
                g.add_dependency(*a, *b);
            }
        }
        let order = g.topological_order().expect("acyclic by construction");
        for (a, b) in &edges {
            if a < b {
                let pa = order.iter().position(|x| x == a).expect("node in order");
                let pb = order.iter().position(|x| x == b).expect("node in order");
                prop_assert!(pa < pb, "topological order respects every dependency");
            }
        }
    }

    #[test]
    fn seq_gaps_always_named_at_first_missing(
        gap in 1u32..99,
    ) {
        let mut cert = SeqContiguityCertifier::new();
        for seq in 0..100u32 {
            if seq != gap {
                cert.record(seq).expect("fresh");
            }
        }
        let verdict = cert.verify_contiguous();
        match verdict {
            Err(err) => prop_assert_eq!(format!("{err}"), format!("sequence gap detected at {gap}")),
            Ok(()) => prop_assert!(false, "gap {gap} must be detected"),
        }
    }

    #[test]
    fn zk_range_in_range_always_verifies(value in any::<u32>()) {
        // The declared 32-bit range always contains the 32-bit value.
        const LABEL: &[u8] = b"affidavit:proptest:zk-range";
        let witness = prove_range(u64::from(value), 32, LABEL, &mut rand_core::OsRng)
            .expect("u32 value fits 32 bits");
        prop_assert!(
            verify_range(&witness.commitment, &witness.proof, 32, LABEL).is_ok(),
            "in-range proof verifies"
        );
    }

    #[test]
    fn zk_range_over_range_always_refused(
        bits in 1usize..32,
        overflow in 1u64..=(1 << 16),
    ) {
        // Value guaranteed to exceed 2^bits when bits < 32: construct it so.
        let value = (1u64 << bits) + (overflow % 7);
        const LABEL: &[u8] = b"affidavit:proptest:zk-range";
        prop_assert!(
            prove_range(value, bits, LABEL, &mut rand_core::OsRng).is_err(),
            "over-range value must be refused before proving"
        );
    }
}
