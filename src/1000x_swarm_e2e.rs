// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 1000X COMBINATORIAL MAXIMALISM: Global Swarm E2E Harness.
//!
//! SPECIFICATION:
//! 1. Scale: Spawn 10,000 parallel Tokio tasks.
//! 2. Pipeline: Each task executes a full (emit -> assemble -> verify) lifecycle.
//! 3. Determinism: Verify that concurrent execution does not drift hashes for identical inputs.
//! 4. Contention: Measure lock-wait duration on a shared resource to profile system-wide friction.
//! 5. Verification: Every verdict in the swarm must be ACCEPTED.

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

// Seam alignment: reuse handlers and core logic from affidavit
use crate::chain::ChainAssembler;
use crate::ocel::{build_event, object_ref, SeqCounter};
use crate::verifier::verify;
use crate::types::Blake3Hash;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const NUM_PIPELINES: usize = 10_000;
    const EVENTS_PER_PIPELINE: usize = 10;

    outln!("\x1b[1;35m--- GLOBAL SWARM E2E HARNESS ---\x1b[0m");
    outln!("Target Scale: {} parallel pipelines", NUM_PIPELINES);
    outln!("Load Depth:   {} events per pipeline", EVENTS_PER_PIPELINE);
    outln!("Total Events: {}", NUM_PIPELINES * EVENTS_PER_PIPELINE);
    outln!("----------------------------------");

    let swarm_start = Instant::now();
    let completed = Arc::new(AtomicUsize::new(0));
    
    // Shared resource to measure lock contention
    let contention_registry = Arc::new(Mutex::new(Vec::new()));
    let total_wait_micros = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(NUM_PIPELINES);

    for i in 0..NUM_PIPELINES {
        let completed = Arc::clone(&completed);
        let contention_registry = Arc::clone(&contention_registry);
        let total_wait_micros = Arc::clone(&total_wait_micros);
        
        let handle = tokio::spawn(async move {
            // PHASE 1: EMIT
            let mut counter = SeqCounter::new();
            let mut asm = ChainAssembler::new();
            
            for j in 0..EVENTS_PER_PIPELINE {
                let event = build_event(
                    format!("swarm.op.{}", j),
                    vec![object_ref(format!("artifact.{}", i), "artifact")],
                    format!("payload.data.{}.{}", i, j).as_bytes(),
                    &mut counter,
                ).expect("EMIT failure");
                
                // PHASE 2: ASSEMBLE
                asm.append(event).expect("ASSEMBLE failure");
            }
            
            let receipt = asm.finalize();
            let final_hash = receipt.chain_hash.clone();
            
            // PHASE 3: VERIFY
            let verdict = verify(&receipt);
            if !verdict.accepted {
                panic!("VERIFY failure in pipeline {}: {}", i, verdict.reason);
            }

            // MEASURE CONTENTION
            let wait_start = Instant::now();
            {
                let mut registry = contention_registry.lock().await;
                let wait_duration = wait_start.elapsed();
                total_wait_micros.fetch_add(wait_duration.as_micros() as usize, Ordering::Relaxed);
                
                // Minimal work under lock to simulate registry registration
                registry.push(final_hash);
            }
            
            completed.fetch_add(1, Ordering::SeqCst);
            receipt.chain_hash
        });
        handles.push(handle);
    }

    // Collect all results
    let mut swarm_hashes = Vec::with_capacity(NUM_PIPELINES);
    for handle in handles {
        swarm_hashes.push(handle.await?);
    }

    let duration = swarm_start.elapsed();
    let total_done = completed.load(Ordering::SeqCst);
    let avg_wait = total_wait_micros.load(Ordering::Relaxed) as f64 / total_done as f64;

    outln!("\x1b[1;32mSwarm Mission Accomplished.\x1b[0m");
    outln!("Wall Time:      {:?}", duration);
    outln!("Throughput:     {:.2} pipelines/sec", total_done as f64 / duration.as_secs_f64());
    outln!("Aggregate TP:   {:.2} events/sec", (total_done * EVENTS_PER_PIPELINE) as f64 / duration.as_secs_f64());
    outln!("Lock Contention: {:.2} µs (avg wait)", avg_wait);
    
    // DETERMINISM VALIDATION
    outln!("\nValidating cross-thread determinism...");
    let det_start = Instant::now();
    let mut det_handles = Vec::new();
    
    // Spawn tasks with EXACTLY identical inputs to ensure identical output hashes
    for _ in 0..100 {
        det_handles.push(tokio::spawn(async move {
            let mut counter = SeqCounter::new();
            let mut asm = ChainAssembler::new();
            let ev = build_event("const", vec![], b"fixed_payload", &mut counter).unwrap();
            asm.append(ev).unwrap();
            asm.finalize().chain_hash
        }));
    }
    
    let base_hash = det_handles.remove(0).await?;
    for (idx, h) in det_handles.into_iter().enumerate() {
        let h_val = h.await?;
        if h_val != base_hash {
            outln!("\x1b[1;31mDETERMINISM BREAK\x1b[0m at task {}: {} != {}", idx + 1, h_val, base_hash);
            std::process::exit(1);
        }
    }
    
    outln!("Determinism Check: \x1b[1;32mPASSED\x1b[0m (100/100 coherent) in {:?}", det_start.elapsed());
    outln!("----------------------------------");
    
    Ok(())
}
