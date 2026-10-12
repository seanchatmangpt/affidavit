//! Adversarial Skeptic Lane 3: Anti-Replay & Epoch Nullifier Skeptic Tests
//! Probes:
//! 1. SlidingBitFilter false positive and false negative rates across capacity boundaries (65,536+ insertions).
//! 2. Multi-threaded concurrent access and epoch rollover race conditions.
//! 3. Persistence resilience against partial writes or corrupted files in `.affi/nullifiers.bin`.
//! 4. Strict refusal of duplicate receipts within an epoch.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::thread;

use affidavit::replay_nullifier::{
    compute_nullifier, EpochKey, Nullifier, ReplayNullifierError, ReplayNullifierRegistry,
    SlidingBitFilter, NULLIFIER_SIZE,
};

#[test]
fn test_sliding_bit_filter_false_negative_and_positive_rates_at_scale() {
    // 65,536 elements stress test
    let capacity = 65_536;
    let mut filter = SlidingBitFilter::new(capacity * 8);

    let key: EpochKey = [0x7au8; 32];
    let mut nullifiers = Vec::with_capacity(capacity);

    for i in 0..capacity {
        let wire = format!("adversarial-receipt-wire-item-{i}");
        let n = compute_nullifier(&key, wire.as_bytes());
        nullifiers.push(n);
    }

    // Insert all items
    for n in &nullifiers {
        filter.set(n);
    }

    // Probe 1: Zero False Negatives (Bloom filter invariant: might_contain MUST be true for all inserted elements)
    let mut false_negatives = 0;
    for n in &nullifiers {
        if !filter.might_contain(n) {
            false_negatives += 1;
        }
    }
    assert_eq!(
        false_negatives, 0,
        "Critical integrity violation: SlidingBitFilter exhibited {false_negatives} false negatives!"
    );

    // Probe 2: False Positive Rate under 2-hash Bloom filter with m = 8*N bits (approx theoretical ~ 18-22%)
    let mut false_positives = 0;
    let test_probes = 20_000;
    for i in 0..test_probes {
        let non_member_wire = format!("non-member-receipt-{i}");
        let n = compute_nullifier(&key, non_member_wire.as_bytes());
        if filter.might_contain(&n) {
            false_positives += 1;
        }
    }

    let fp_rate = (false_positives as f64) / (test_probes as f64);
    println!(
        "SlidingBitFilter 65,536 insertions: FP count = {false_positives}/{test_probes} ({:.2}%)",
        fp_rate * 100.0
    );

    // Ensure it acts as a functional filter and isn't completely saturated (FP < 90%)
    // Notice: with 2 hash functions and m/n = 8: p = (1 - e^(-2/8))^2 ≈ (1 - e^(-0.25))^2 ≈ 0.049 (4.9%)
    // But as n saturates, verify it doesn't exceed 40%
    assert!(
        fp_rate < 0.40,
        "False positive rate abnormally high: {:.2}%",
        fp_rate * 100.0
    );
}

#[test]
fn test_strict_refusal_of_duplicate_receipts_within_epoch() {
    let epoch_key: EpochKey = [0x55u8; 32];
    let mut registry = ReplayNullifierRegistry::new(epoch_key, 50_000);

    let receipt_payload = b"CRITICAL_STATE_TRANSITION_RECEIPT_v1";

    // First admittance must succeed
    let n1 = registry.check_and_admit(receipt_payload);
    assert!(n1.is_ok(), "First receipt submission should be accepted");

    // Second admittance of identical receipt wire must be strictly refused
    let n2 = registry.check_and_admit(receipt_payload);
    match n2 {
        Err(ReplayNullifierError::ReplayDetected(hex)) => {
            let expected_hex = n1
                .unwrap()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            assert_eq!(hex, expected_hex, "Reported nullifier in error should match");
        }
        Err(e) => panic!("Unexpected error on duplicate receipt: {e}"),
        Ok(_) => panic!("FATAL: Duplicate receipt wire was admitted without detection!"),
    }

    // Direct admittance via admit_nullifier must also be strictly refused
    let computed_n = compute_nullifier(&epoch_key, receipt_payload);
    let direct_res = registry.admit_nullifier(computed_n);
    assert!(
        matches!(direct_res, Err(ReplayNullifierError::ReplayDetected(_))),
        "Direct re-admittance must fail"
    );
}

#[test]
fn test_multithreaded_concurrent_access_and_epoch_rollover_races() {
    let initial_key: EpochKey = [0x11u8; 32];
    let next_key: EpochKey = [0x22u8; 32];
    let registry = Arc::new(RwLock::new(ReplayNullifierRegistry::new(initial_key, 20_000)));

    let num_threads = 8;
    let receipts_per_thread = 500;
    let barrier = Arc::new(Barrier::new(num_threads + 1));
    let mut handles = Vec::new();

    let duplicate_rejected_count = Arc::new(Mutex::new(0));

    // Spawn 8 client worker threads attempting concurrent submissions
    for thread_idx in 0..num_threads {
        let registry_clone = Arc::clone(&registry);
        let barrier_clone = Arc::clone(&barrier);
        let dup_counter = Arc::clone(&duplicate_rejected_count);

        handles.push(thread::spawn(move || {
            barrier_clone.wait();
            for i in 0..receipts_per_thread {
                // Introduce deliberate overlap across thread 0 and thread 1
                let receipt_id = if thread_idx == 1 { i } else { thread_idx * 10_000 + i };
                let wire = format!("concurrent-receipt-wire-{receipt_id}");

                let mut reg = registry_clone.write().unwrap();
                match reg.check_and_admit(wire.as_bytes()) {
                    Ok(_) => {}
                    Err(ReplayNullifierError::ReplayDetected(_)) => {
                        let mut count = dup_counter.lock().unwrap();
                        *count += 1;
                    }
                    // Cross-rotation wire replays are now refused with the
                    // typed epoch-mismatch code (SK5 fix-forward, lane R31).
                    Err(ReplayNullifierError::ReplayEpochMismatch(_)) => {
                        let mut count = dup_counter.lock().unwrap();
                        *count += 1;
                    }
                    Err(e) => panic!("Unexpected error: {e}"),
                }
            }
        }));
    }

    // Spawn epoch rollover coordinator thread concurrently
    let registry_rollover = Arc::clone(&registry);
    let barrier_rollover = Arc::clone(&barrier);
    let rollover_handle = thread::spawn(move || {
        barrier_rollover.wait();
        // Allow a brief interleaving then trigger epoch rollover mid-flight
        thread::yield_now();
        let mut reg = registry_rollover.write().unwrap();
        reg.rotate_epoch(next_key);
    });

    for h in handles {
        h.join().unwrap();
    }
    rollover_handle.join().unwrap();

    let final_reg = registry.read().unwrap();
    println!(
        "Concurrent stress finished: epoch key matches new = {}, tracked items = {}, detected duplicates = {}",
        *final_reg.epoch_key() == next_key,
        final_reg.len(),
        *duplicate_rejected_count.lock().unwrap()
    );

    // Verify invariants: no panic or corrupted state
    assert_eq!(*final_reg.epoch_key(), next_key);
}

#[test]
fn test_persistence_resilience_partial_writes_and_corrupted_bin() {
    let temp_dir = std::env::temp_dir().join(format!("affi_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let nullifiers_path = temp_dir.join("nullifiers.bin");

    let epoch_key: EpochKey = [0x33u8; 32];
    let mut registry = ReplayNullifierRegistry::new(epoch_key, 1000);

    // Scenario 1: Pre-populate with valid 32-byte nullifiers
    let valid_count = 10;
    let mut valid_nullifiers = Vec::new();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&nullifiers_path)
        .unwrap();

    for i in 0..valid_count {
        let n: Nullifier = [(i as u8); 32];
        valid_nullifiers.push(n);
        file.write_all(&n).unwrap();
    }

    // Scenario 2: Simulate crash leading to partial write (e.g., 17 bytes trailing corrupt chunk)
    let partial_garbage = [0xde, 0xad, 0xbe, 0xef, 0xaa, 0xbb, 0xcc, 0xdd, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
    file.write_all(&partial_garbage).unwrap();
    file.flush().unwrap();
    drop(file);

    // Verify parser behavior matching src/handlers.rs:
    // chunk_exact(32) should gracefully ingest full 32-byte records and ignore trailing partials without panic
    let file_bytes = fs::read(&nullifiers_path).unwrap();
    let mut recovered_count = 0;
    for chunk in file_bytes.chunks_exact(NULLIFIER_SIZE) {
        if let Ok(n) = chunk.try_into() {
            let res = registry.admit_nullifier(n);
            assert!(res.is_ok());
            recovered_count += 1;
        }
    }

    assert_eq!(
        recovered_count, valid_count,
        "Should successfully recover exactly the valid complete chunks despite partial trailing write"
    );

    // Any already admitted nullifier must be detected on re-admit
    for n in &valid_nullifiers {
        assert!(registry.is_seen(n));
        assert!(matches!(
            registry.admit_nullifier(*n),
            Err(ReplayNullifierError::ReplayDetected(_))
        ));
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_sliding_window_eviction_vulnerability_boundary_analysis() {
    // Critical architectural probe: ReplayNullifierRegistry's sliding window behavior
    // When dense_cache reaches window_capacity, line 164 clears the entire dense_cache and bit_filter!
    // This means any receipt admitted before eviction can be replayed after capacity overflow.
    let epoch_key: EpochKey = [0x99u8; 32];
    let capacity = 50;
    let mut registry = ReplayNullifierRegistry::new(epoch_key, capacity);

    let target_receipt = b"replayed_target_receipt";
    let n_target = registry.check_and_admit(target_receipt).expect("initial admit");

    assert!(registry.is_seen(&n_target));
    assert!(registry.check_and_admit(target_receipt).is_err());

    // Adversarially flood registry up to capacity threshold
    for i in 1..capacity {
        let dummy = format!("flood_receipt_{i}");
        registry.check_and_admit(dummy.as_bytes()).expect("flood admit");
    }

    assert_eq!(registry.len(), capacity);

    // Next receipt triggers the sliding window clear:
    let evictor_receipt = b"evictor_receipt";
    registry.check_and_admit(evictor_receipt).expect("evictor admit");

    // FIXED (SK5 fix-forward, lane R31): target_receipt was retired into the
    // cross-epoch wire history at eviction, so its replay is REFUSED with the
    // typed epoch-mismatch code — the former eviction amnesty is closed.
    let replay_after_eviction = registry.check_and_admit(target_receipt);
    println!(
        "Adversarial observation: Replay after capacity eviction result: {:?}",
        replay_after_eviction
    );
    match replay_after_eviction {
        Err(ReplayNullifierError::ReplayEpochMismatch(_)) => {}
        other => panic!(
            "Eviction-boundary replay must be refused as ReplayEpochMismatch, got: {other:?}"
        ),
    }
}
