//! Chain assembly edge-case tests.

use affidavit::chain::{content_address, recompute_chain, ChainAssembler, GENESIS_SEED};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Blake3Hash;
use affidavit::verifier::verify;

fn genesis_hash() -> Blake3Hash {
    Blake3Hash::from_bytes(GENESIS_SEED)
}

fn mk_event(counter: &mut SeqCounter, payload: &[u8]) -> affidavit::types::OperationEvent {
    build_event("op", vec![object_ref("o", "artifact")], payload, counter).unwrap()
}

/// An empty ChainAssembler produces a receipt whose chain_hash equals the genesis hash.
#[test]
fn empty_chain_has_genesis_hash() {
    let receipt = ChainAssembler::new().finalize();
    assert_eq!(
        receipt.chain_hash,
        genesis_hash(),
        "empty chain must carry genesis hash"
    );
}

/// from_events + finalize produces the same chain_hash as incremental append.
#[test]
fn from_events_reconstructs_exact_chain() {
    let mut c = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for payload in [b"a".as_slice(), b"b", b"c"] {
        asm.append(mk_event(&mut c, payload)).unwrap();
    }
    let original = asm.finalize();

    // Reconstruct from the events slice
    let rebuilt = ChainAssembler::from_events(original.events.clone())
        .unwrap()
        .finalize();

    assert_eq!(
        original.chain_hash,
        rebuilt.chain_hash,
        "from_events must reconstruct an identical chain_hash"
    );
}

/// Two assemblers with the same events in opposite order produce different hashes.
#[test]
fn append_order_determines_hash() {
    let mut c1 = SeqCounter::new();
    let ev_a = mk_event(&mut c1, b"first");
    let ev_b = mk_event(&mut c1, b"second");

    let mut asm1 = ChainAssembler::new();
    asm1.append(ev_a.clone()).unwrap();
    asm1.append(ev_b.clone()).unwrap();
    let h1 = asm1.finalize().chain_hash;

    let mut asm2 = ChainAssembler::new();
    asm2.append(ev_b).unwrap();
    asm2.append(ev_a).unwrap();
    let h2 = asm2.finalize().chain_hash;

    assert_ne!(h1, h2, "different append order must produce different chain hashes");
}

/// content_address is stable: calling it twice on the same receipt yields the same hash.
#[test]
fn content_address_is_stable() {
    let mut c = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    asm.append(mk_event(&mut c, b"payload")).unwrap();
    let receipt = asm.finalize();

    let addr1 = content_address(&receipt).expect("content_address 1");
    let addr2 = content_address(&receipt).expect("content_address 2");
    assert_eq!(addr1, addr2, "content_address must be deterministic");
}

/// recompute_chain on an empty slice returns the genesis hash.
#[test]
fn recompute_chain_on_empty_returns_genesis() {
    let computed = recompute_chain(&[]).expect("recompute empty");
    assert_eq!(computed, genesis_hash(), "empty recompute must equal genesis hash");
}

/// Each append changes the chain_hash -- the chain evolves with every event.
#[test]
fn chain_grows_with_each_append() {
    let mut c = SeqCounter::new();
    let mut asm = ChainAssembler::new();

    let mut hashes: Vec<Blake3Hash> = vec![ChainAssembler::new().finalize().chain_hash];
    for payload in [b"x".as_slice(), b"y", b"z"] {
        asm.append(mk_event(&mut c, payload)).unwrap();
        hashes.push(asm.clone().finalize().chain_hash);
    }

    // Each snapshot must be unique (the chain advanced after every append)
    let unique_count = {
        let mut sorted = hashes.clone();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    };
    assert_eq!(
        unique_count,
        hashes.len(),
        "every append must produce a distinct chain_hash"
    );
}

/// A 200-event receipt built via ChainAssembler and verified must not panic.
#[test]
fn large_chain_accepts() {
    let mut c = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for i in 0u64..200 {
        asm.append(
            build_event("op", vec![object_ref(format!("o{i}"), "artifact")], format!("p{i}").as_bytes(), &mut c).unwrap()
        ).unwrap();
    }
    let receipt = asm.finalize();
    assert_eq!(receipt.events.len(), 200);
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "200-event receipt must ACCEPT: {}", verdict.reason);
}
