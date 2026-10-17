//! Court: `smt_inclusion_verify` over the real JSON ABI (`abi::call`).
//!
//! Three cells: an honest witness (real monotree tree, real proof) admits;
//! a tampered proof step refuses; a wrong root refuses. A positive control
//! proves the harness can fail: a malformed request is a typed error, and a
//! bogus op name is `unknown_op` — the court cannot pass vacuously.
use affidavit_wasm::abi::call;
use serde_json::{json, Value};

type M = monotree::Monotree<monotree::database::MemoryDB, monotree::hasher::Blake3>;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Build a real monotree (MemoryDB + BLAKE3), insert `n` keys in the
/// canonical ascending order, return the tree + root + keys.
fn tree_with(n: usize, value: [u8; 32]) -> (M, Option<monotree::Hash>, Vec<[u8; 32]>) {
    let mut tree = M::default();
    let mut keys: Vec<[u8; 32]> = (0..n)
        .map(|i| {
            let mut k = [0u8; 32];
            k[0] = (i as u8) * 16 + 1;
            k
        })
        .collect();
    keys.sort();
    let mut root = None;
    for k in &keys {
        root = tree.insert(root.as_ref(), k, &value).expect("insert");
    }
    (tree, root, keys)
}

fn request(root: &monotree::Hash, key: &[u8; 32], value: &[u8; 32], steps: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "op": "smt_inclusion_verify",
        "root_hex": hex(root),
        "leaf_key_hex": hex(key),
        "leaf_value_hex": hex(value),
        "proof_steps": steps,
    }))
    .expect("request json")
}

fn steps_json(proof: &[(bool, Vec<u8>)]) -> Vec<Value> {
    proof
        .iter()
        .map(|(right, cut)| json!({ "right": right, "cut_hex": hex(cut) }))
        .collect()
}

#[test]
fn honest_witness_admits_through_the_abi() {
    let (mut tree, root, keys) = tree_with(3, [7u8; 32]);
    let root = root.expect("non-empty tree has a root");
    let proof = tree
        .get_merkle_proof(Some(&root), &keys[1])
        .expect("proof")
        .expect("proof present");
    let req = request(&root, &keys[1], &[7u8; 32], steps_json(&proof));
    let res: Value = serde_json::from_slice(&call(&req)).expect("json response");
    assert_eq!(res["ok"], json!(true), "response: {res}");
    assert_eq!(res["admitted"], json!(true), "honest witness admits: {res}");
}

#[test]
fn tampered_proof_step_refuses_through_the_abi() {
    let (mut tree, root, keys) = tree_with(2, [9u8; 32]);
    let root = root.expect("root");
    let mut proof = tree
        .get_merkle_proof(Some(&root), &keys[0])
        .expect("proof")
        .expect("proof present");
    proof[0].1[0] ^= 0x01;
    let req = request(&root, &keys[0], &[9u8; 32], steps_json(&proof));
    let res: Value = serde_json::from_slice(&call(&req)).expect("json response");
    // Well-formed fields, false verdict: refusal is a lawful total answer,
    // not a typed error (the ABI is total; only malformed FIELDS are errors).
    assert_eq!(res["ok"], json!(true), "verdict is total: {res}");
    assert_eq!(
        res["admitted"],
        json!(false),
        "tampered step refuses: {res}"
    );
}

#[test]
fn wrong_root_refuses_through_the_abi() {
    let (_, root, keys) = tree_with(2, [9u8; 32]);
    let mut root = root.expect("root");
    root[0] ^= 0x80;
    // Take the proof against the honest root from a pristine tree, verify
    // against the mutated root.
    let (mut honest_tree, honest_root, _) = tree_with(2, [9u8; 32]);
    let honest_root = honest_root.expect("root");
    let proof = honest_tree
        .get_merkle_proof(Some(&honest_root), &keys[0])
        .expect("proof")
        .expect("proof present");
    let req = request(&root, &keys[0], &[9u8; 32], steps_json(&proof));
    let res: Value = serde_json::from_slice(&call(&req)).expect("json response");
    assert_eq!(
        res["ok"],
        json!(true),
        "verdict is total, not an error: {res}"
    );
    assert_eq!(res["admitted"], json!(false), "wrong root refuses: {res}");
}

#[test]
fn malformed_request_is_a_typed_error_not_a_panic() {
    let req = serde_json::to_vec(&json!({
        "op": "smt_inclusion_verify",
        "root_hex": format!("{:0>64}", "0"),
        "leaf_key_hex": format!("{:0>64}", "0"),
        "leaf_value_hex": format!("{:0>65}", "0"),
        "proof_steps": []
    }))
    .expect("json");
    let res: Value = serde_json::from_slice(&call(&req)).expect("json response");
    assert_eq!(res["ok"], json!(false));
    assert_eq!(res["error"]["code"], json!("bad_field"));

    let unknown = serde_json::to_vec(&json!({ "op": "smt_inclusion_verify_noop" })).expect("json");
    let res: Value = serde_json::from_slice(&call(&unknown)).expect("json response");
    assert_eq!(res["error"]["code"], json!("unknown_op"));
}
