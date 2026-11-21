//! AOD zero-allocation read-path court (AFFI-SPEC-26.11.19 DoD).
//!
//! # Method
//!
//! Install a counting [`GlobalAlloc`] that delegates to the system allocator
//! while tallying every `alloc` / `alloc_zeroed` / `realloc` **on the measuring
//! thread only**. The honest fixture is sealed **before** the counter is armed
//! (ML-DSA-65 signing may allocate). Then:
//!
//! * [`verify_witness`]
//! * [`shared_envelope_check`]
//! * [`map_live`] with a stack-only [`LeaseRegistry`]
//!
//! must leave the allocation counter at **zero**.
//!
//! Companion compile court: `cargo test --no-default-features` still exercises
//! `aod::envelope` / `aod::witness` unit tests — those modules never touch
//! `alloc`. This file is the runtime counter proof under the `std` harness.
//!
//! `affidavit-core` itself `forbid(unsafe_code)`; this integration binary is a
//! separate crate so the counting allocator may use `unsafe` solely for
//! `GlobalAlloc`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use affidavit_core::{
    map_live, seal_aod, shared_envelope_check, verify_witness, AodRefusal, CompiledOntology,
    LeaseRegistry, SealParams,
};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// Only the measuring thread's heap traffic is tallied (harness threads ignore).
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

struct CountingAllocator;

#[inline]
fn tally() {
    // TLS already warmed before COUNTING is set true — `.with` must not allocate.
    let counting = COUNTING.try_with(|c| c.get()).unwrap_or(false);
    if counting {
        ALLOC_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        tally();
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        tally();
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        tally();
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn reset_allocs() {
    ALLOC_COUNT.store(0, Ordering::SeqCst);
}

fn allocs() -> usize {
    ALLOC_COUNT.load(Ordering::SeqCst)
}

/// Run `f` with a zeroed per-thread counter; return `(result, allocs)`.
fn measure_allocs<R>(f: impl FnOnce() -> R) -> (R, usize) {
    // Warm TLS before enabling so the first tally does not allocate the cell.
    COUNTING.with(|c| c.set(false));
    reset_allocs();
    COUNTING.with(|c| c.set(true));
    let out = f();
    COUNTING.with(|c| c.set(false));
    (out, allocs())
}

/// Stack-only lease registry — no heap, one burned nullifier slot.
struct StackLease {
    burned: Option<[u8; 32]>,
}

impl LeaseRegistry for StackLease {
    fn burn(&mut self, _lease_epoch: u64, nullifier: &[u8; 32]) -> Result<(), AodRefusal> {
        if self.burned.as_ref() == Some(nullifier) {
            return Err(AodRefusal::LeaseSpentOrExpired);
        }
        self.burned = Some(*nullifier);
        Ok(())
    }
}

/// Honest empty AOD — sealed outside the measuring window (ML-DSA may allocate).
fn minimal_empty_aod() -> Vec<u8> {
    let ontology = [0x44u8; 32];
    seal_aod(
        &[],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 0,
            lease_epoch: 9,
            lease_nullifier: &[0x55; 32],
            folded_receipt: &[0x66; 32],
        },
    )
    .expect("seal empty")
}

/// Single-threaded court body: vacuity + verify_witness + envelope + map_live.
#[test]
fn aod_zero_alloc_read_path_court() {
    // Vacuity: the counter must observe a real heap allocation.
    let (n, _) = measure_allocs(|| {
        let _heap = Box::new([0u8; 64]);
        allocs()
    });
    assert!(n > 0, "counting GlobalAlloc must observe Box allocation");

    let raw = minimal_empty_aod();

    let (att, n) = measure_allocs(|| verify_witness(&raw).expect("witness"));
    assert_eq!(n, 0, "verify_witness must not allocate");
    assert_eq!(att.ontology_hash, [0x44; 32]);
    assert_eq!(att.lease_epoch, 9);

    let (ok, n) = measure_allocs(|| shared_envelope_check(&raw).expect("envelope"));
    assert_eq!(n, 0, "shared_envelope_check must not allocate");
    assert_eq!(ok.header.buffer_count, 0);

    let ontology = [0x44u8; 32];
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[],
    };
    let mut leases = StackLease { burned: None };
    let (view, n) = measure_allocs(|| map_live(&raw, &compiled, &mut leases).expect("live"));
    assert_eq!(n, 0, "map_live must not allocate on the read path");
    assert_eq!(view.row_count, 0);
    assert_eq!(view.buffer_count(), 0);
}
