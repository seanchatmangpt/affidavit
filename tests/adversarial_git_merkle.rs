//! Adversarial Skeptic Lane 4: Pure-Rust Git Merkle & Census Binding Skeptic tests.
//!
//! M2 disclosure: this target imports the optional `gix_*` dependencies and
//! `affidavit::git_merkle` (git-provenance surface) but was never feature-
//! gated, so ANY `cargo test --features <not-git-provenance>` run failed to
//! compile it — pre-existing, reproduced identically with `--features
//! inspection`. Gated here so the Phase D M2 gate (`cargo test --features
//! ctdd`) can run the full suite; content untouched.
#![cfg(feature = "git-provenance")]
//!
//! Stresses:
//! 1. Census binding uniqueness and determinism under adversarial mutations of GitTreeSHA and WasmBytecodeHash.
//! 2. Loose object database traversal against deep trees, empty commits/trees, and tampered trees.
//! 3. `refs/notes/strata-dossier` note attachment and retrieval under edge cases (tampered bytes, corrupted refs, multiple commits, missing objects).

use affidavit::git_merkle::{compute_census_binding, GitDossierNotes, GitTreeBuilder, STRATA_DOSSIER_REF};
use affidavit::trajectory_folding::CtrtReceipt;
use gix_hash::ObjectId;
use gix_object::bstr::BString;
use gix_object::tree::EntryKind;
use std::collections::HashSet;
use std::fs;
use tempfile::tempdir;

fn test_signature() -> gix_actor::Signature {
    gix_actor::Signature {
        name: BString::from("Adversary Skeptic"),
        email: BString::from("skeptic@strata.dossier"),
        time: gix_object::date::Time::new(1710000000, 0),
    }
}

fn sample_ctrt_receipt(name: &str, run_id: u64) -> CtrtReceipt {
    let mut receipt = CtrtReceipt {
        flags: 2, // FLAG_FAIL_CLOSED
        subject_hash: [0x5au8; 32],
        parent_receipt_hash: [0u8; 32],
        timestamp_ms: 1710000000000,
        run_id,
        total_ticks: 1000,
        wall_ms: 42,
        thermal: 1,
        outcome: 0,
        budget_met: true,
        positive_control: [0u8; 32],
        metadata_digest: [0xa1u8; 32],
        prev_self_hash: [0u8; 32],
        name: name.to_string(),
        chain_hash: [0u8; 32],
    };
    let encoded = receipt.encode().expect("receipt encode must succeed");
    receipt.chain_hash.copy_from_slice(&encoded[320..352]);
    receipt
}

#[test]
fn test_adversarial_census_binding_mutation_uniqueness_and_determinism() {
    let base_tree_sha = [0x42u8; 20];
    let base_wasm_hash = [0x99u8; 32];

    let base_digest = compute_census_binding(&base_tree_sha, &base_wasm_hash);

    // 1. Strict determinism check across iterations
    for _ in 0..100 {
        assert_eq!(
            compute_census_binding(&base_tree_sha, &base_wasm_hash),
            base_digest,
            "Census binding must be 100% deterministic"
        );
    }

    // 2. Single-bit / single-byte mutations of GitTreeSHA
    let mut seen_digests = HashSet::new();
    seen_digests.insert(base_digest);

    for byte_idx in 0..base_tree_sha.len() {
        for bit in 0..8 {
            let mut mutated_tree = base_tree_sha;
            mutated_tree[byte_idx] ^= 1 << bit;
            let digest = compute_census_binding(&mutated_tree, &base_wasm_hash);
            assert!(
                seen_digests.insert(digest),
                "Collision detected in mutated GitTreeSHA at byte {byte_idx} bit {bit}"
            );
        }
    }

    // 3. Single-bit / single-byte mutations of WasmBytecodeHash
    for byte_idx in 0..base_wasm_hash.len() {
        for bit in 0..8 {
            let mut mutated_wasm = base_wasm_hash;
            mutated_wasm[byte_idx] ^= 1 << bit;
            let digest = compute_census_binding(&base_tree_sha, &mutated_wasm);
            assert!(
                seen_digests.insert(digest),
                "Collision detected in mutated WasmBytecodeHash at byte {byte_idx} bit {bit}"
            );
        }
    }

    // 4. Fixed domain representation check:
    // In protocol spec, GitTreeSHA is 20-byte SHA-1 (or 32-byte SHA-256) and WasmBytecodeHash is 32-byte BLAKE3.
    // Notice that BLAKE3(A || B) without length prefixing is vulnerable to boundary shifts if variable-length,
    // which proves why GitTreeSHA and WasmBytecodeHash MUST be fixed-size buffers in practice.
    // Test that distinct fixed-length slices never collide:
    let d_split1 = compute_census_binding(&[0xaa; 20], &[0xbb; 32]);
    let d_split2 = compute_census_binding(&[0xaa; 20], &[0xcc; 32]);
    let d_split3 = compute_census_binding(&[0xbb; 20], &[0xbb; 32]);
    assert_ne!(d_split1, d_split2);
    assert_ne!(d_split1, d_split3);
    assert_ne!(d_split2, d_split3);

    // 5. Empty inputs
    let d_empty_both = compute_census_binding(b"", b"");
    let d_empty_tree = compute_census_binding(b"", &base_wasm_hash);
    let d_empty_wasm = compute_census_binding(&base_tree_sha, b"");
    assert_ne!(d_empty_both, d_empty_tree);
    assert_ne!(d_empty_both, d_empty_wasm);
    assert_ne!(d_empty_tree, d_empty_wasm);
}

#[test]
fn test_loose_odb_stress_deep_trees_and_empty_structures() {
    let dir = tempdir().expect("tempdir");
    let objects_dir = dir.path().join(".git").join("objects");
    fs::create_dir_all(&objects_dir).expect("create objects dir");

    let builder = GitTreeBuilder::new(&objects_dir);

    // 1. Empty tree handling
    let empty_tree_oid = builder.write_tree(vec![]).expect("empty tree must write successfully");
    let nodes = builder.traverse_tree(&empty_tree_oid).expect("empty tree traversal succeeds");
    assert!(nodes.is_empty(), "Empty tree should have zero descendant nodes");

    // 2. Empty commit referencing empty tree
    let empty_commit_oid = builder.create_signed_commit(
        empty_tree_oid,
        vec![],
        test_signature(),
        test_signature(),
        "Empty root commit",
        vec![],
    ).expect("commit on empty tree should succeed");
    assert!(!empty_commit_oid.to_hex().to_string().is_empty());

    // 3. Deeply nested directory hierarchy (50 levels deep)
    let mut current_oid = builder.write_blob(b"leaf content at depth 50").expect("write leaf blob");
    for depth in (1..=50).rev() {
        let entry = gix_object::tree::Entry {
            mode: if depth == 50 { EntryKind::Blob.into() } else { EntryKind::Tree.into() },
            filename: BString::from(format!("level_{depth}")),
            oid: current_oid,
        };
        current_oid = builder.write_tree(vec![entry]).expect("write subtree");
    }

    let deep_root_oid = current_oid;
    let deep_traversed = builder.traverse_tree(&deep_root_oid).expect("deep tree traversal should succeed without stack overflow");
    assert_eq!(deep_traversed.len(), 50, "Should traverse exactly 50 levels");
    assert_eq!(deep_traversed.last().unwrap().path, (1..=50).map(|d| format!("level_{d}")).collect::<Vec<_>>().join("/"));

    // 4. Tampered tree: reference an ObjectId that does not exist in the loose store
    let phantom_oid = ObjectId::from_hex(b"baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let tampered_entry = gix_object::tree::Entry {
        mode: EntryKind::Tree.into(),
        filename: BString::from("corrupted_subtree"),
        oid: phantom_oid,
    };
    let tampered_tree_oid = builder.write_tree(vec![tampered_entry]).expect("tampered tree write");
    
    // Traversal must fail with an ODB error when attempting to fetch missing subtree
    let traverse_err = builder.traverse_tree(&tampered_tree_oid);
    assert!(traverse_err.is_err(), "Traversing into missing subtree must fail");
    let err_msg = format!("{}", traverse_err.err().unwrap());
    assert!(err_msg.contains("Object") && err_msg.contains("not found"), "Error should clearly report missing object: {err_msg}");
}

#[test]
fn test_dossier_notes_adversarial_edge_cases() {
    let dir = tempdir().expect("tempdir");
    let git_dir = dir.path().join(".git");
    let objects_dir = git_dir.join("objects");
    fs::create_dir_all(&objects_dir).expect("create objects dir");

    let notes = GitDossierNotes::new(dir.path());

    // Edge Case 1: Read receipt when notes ref does not exist at all
    let target1 = ObjectId::from_hex(b"1111111111111111111111111111111111111111").unwrap();
    let res = notes.read_receipt(&target1).expect("read should return Ok(None) if ref does not exist");
    assert!(res.is_none());

    // Edge Case 2: Attach multiple receipts to different targets in sequence
    let target2 = ObjectId::from_hex(b"2222222222222222222222222222222222222222").unwrap();
    let target3 = ObjectId::from_hex(b"3333333333333333333333333333333333333333").unwrap();

    let receipt1 = sample_ctrt_receipt("receipt_alpha", 1);
    let receipt2 = sample_ctrt_receipt("receipt_beta", 2);
    let receipt3 = sample_ctrt_receipt("receipt_gamma", 3);

    notes.attach_receipt(&target1, &receipt1, test_signature()).expect("attach 1");
    notes.attach_receipt(&target2, &receipt2, test_signature()).expect("attach 2");
    notes.attach_receipt(&target3, &receipt3, test_signature()).expect("attach 3");

    // Verify all 3 receipts are independently recoverable
    assert_eq!(notes.read_receipt(&target1).unwrap().unwrap(), receipt1);
    assert_eq!(notes.read_receipt(&target2).unwrap().unwrap(), receipt2);
    assert_eq!(notes.read_receipt(&target3).unwrap().unwrap(), receipt3);

    // Edge Case 3: Overwrite receipt for an existing target (mutation)
    let receipt1_updated = sample_ctrt_receipt("receipt_alpha_v2", 999);
    notes.attach_receipt(&target1, &receipt1_updated, test_signature()).expect("update receipt 1");
    assert_eq!(notes.read_receipt(&target1).unwrap().unwrap(), receipt1_updated);

    // Edge Case 4: Non-existent target on populated notes tree
    let non_existent = ObjectId::from_hex(b"9999999999999999999999999999999999999999").unwrap();
    assert!(notes.read_receipt(&non_existent).unwrap().is_none());

    // Edge Case 5: Corrupt the `refs/notes/strata-dossier` ref content (invalid hex)
    let ref_file = git_dir.join(STRATA_DOSSIER_REF);
    fs::write(&ref_file, b"corrupted-non-hex-ref-data\n").expect("write corrupt ref");
    let corrupt_read = notes.read_receipt(&target1);
    assert!(corrupt_read.is_err(), "Reading from corrupted ref hex must return error");

    // Edge Case 6: Ref points to non-existent commit OID
    let phantom_commit_oid = "ffffffffffffffffffffffffffffffffffffffff";
    fs::write(&ref_file, format!("{phantom_commit_oid}\n")).expect("write phantom commit hex");
    let phantom_read = notes.read_receipt(&target1);
    assert!(phantom_read.is_err(), "Reading when commit OID missing from ODB must error");

    // Edge Case 7: Note blob bytes are tampered / invalid CTRT length
    // Recreate a clean notes tree pointing to a corrupted blob (e.g. 50 bytes instead of 352)
    let builder = GitTreeBuilder::new(&objects_dir);
    let bad_blob_oid = builder.write_blob(b"short corrupted receipt data").expect("write bad blob");
    let target_hex = target1.to_hex().to_string();
    let bad_tree = builder.write_tree(vec![gix_object::tree::Entry {
        mode: EntryKind::Blob.into(),
        filename: BString::from(target_hex),
        oid: bad_blob_oid,
    }]).expect("write bad tree");
    let bad_commit = builder.create_signed_commit(
        bad_tree,
        vec![],
        test_signature(),
        test_signature(),
        "Bad receipt note commit",
        vec![],
    ).expect("commit bad note");

    notes.update_notes_ref(&bad_commit).expect("update ref to bad commit");
    let tampered_res = notes.read_receipt(&target1);
    assert!(tampered_res.is_err(), "Deserializing invalid/short CTRT receipt must return decode error");
}
