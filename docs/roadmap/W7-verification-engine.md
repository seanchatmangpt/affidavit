# W7 — Verification Engine: Scale, Performance & Topology (2026 H2 → 2030)

**Workstream:** W7 (Verification Engine) · **Owner doctrine:** *certify, don't decide.*
**Status:** roadmap / design · **Date:** 2026-06-20 · **Caveat:** private `26.6` deps do not
resolve in a lone checkout, so **none of the Rust below was `cargo build`/`test`-verified**. All
sketches are *compilable-style* — pinned to real in-tree symbols (`verifier::verify`,
`chain::recompute_chain`, `ChainAssembler::{from_events,finalize}`, `Receipt::sealed`,
`canonical_bytes`), pending signature finalization against the sibling crates.

---

## 1. Mission & scope

W7 owns the **evolution of the verification engine in scale, performance, and topology** — the items
under CLAUDE.md *Roadmap & Future Work*. The current verifier is a pure, sequential, single-receipt
7-stage pipeline (`src/verifier.rs:43`). W7 grows it along five axes **without ever changing the
verdict a receipt receives**:

1. **Multi-profile** — multiple `core/vX` standards as first-class objects; profile negotiation,
   coexistence, and migration. Today there is exactly one (`ProfileId::CoreV1`, `src/types.rs:236`;
   `STANDARD_VERSION`, `src/verifier.rs:22`).
2. **Streaming / incremental** — append events and verify the *new* suffix without re-folding the
   whole chain from genesis (today `recompute_chain` always starts at genesis, `src/chain.rs:68`).
3. **Parallel verification** — run the independent per-event stages (4–6) and the chain fold (3)
   concurrently; stage 7 (`emit_verdict`) stays terminal (`src/verifier.rs:59`).
4. **GPU acceleration** — the `gpu` feature (`Cargo.toml:161`), for high-throughput *batch* verify
   of many receipts. A prototype exists (`src/1000x_gpu_verifier.rs`) but is **not yet verdict-exact**.
5. **Distributed verification** — Merkle-proof verification across shards / partial-receipt proofs.
   A sharding prototype exists (`src/1000x_distributed_sharding.rs`) but is **not wired in** and
   stitches by boundary hashes, not Merkle proofs.

### Boundary (what W7 does **not** own)

- **Cryptographic trust** — signing, attestation, PQC sealing, transparency logs — is **W8**. BLAKE3
  here is a content address, not a signature; W7 makes recomputing it fast/incremental/distributed, W8
  decides what it *means* to trust a recomputed root. (`src/1000x_post_quantum_sealing.rs` is W8.)
- **Receipt-store health / repair** — scanning a `.affi/` store, scoring, quarantine, re-finalize — is
  **W2** (`docs/innovation/02-doctor-receipts.md`). W7 supplies the *engine* W2 calls (`verifier::verify`
  verbatim; incremental + parallel variants); W2 supplies triage and fixes.
- **CLI ergonomics / output contract** — `--format`, exit-code catalog, `affi why` — is **W3**.
- **The doctrine** — "certify, don't decide" — is invariant for everyone; W7 must *preserve* it under
  every optimization, never relax it.

### The W7 prime directive (the bit-identity contract)

> **For every receipt `r` and every W7 path `P` (sequential, incremental, parallel, GPU, sharded):
> `P(r).accepted == verifier::verify(r).accepted`, and the failing-stage diagnosis agrees.**

The sequential CPU verifier (`src/verifier.rs:43`) is the **reference oracle**. Every faster or
more-distributed path is a *performance rewrite that must round-trip to the oracle bit-for-bit*. This
is the single acceptance gate that recurs in every phase below.

---

## 2. Current state (grounded) & the gap

### What exists and is load-bearing

| Capability | Symbol | Cite |
|---|---|---|
| 7-stage pipeline, pure & deterministic | `verifier::verify(&Receipt) -> Verdict` | `src/verifier.rs:43` |
| Stage 3 fold (the chain truth) | `chain::recompute_chain(&[OperationEvent])` | `src/chain.rs:68` |
| One-event fold step | `fold_event(prev, event)` (private) | `src/chain.rs:54` |
| Genesis seed | `GENESIS_SEED` / `genesis_hash()` (private) | `src/chain.rs:22`,`:48` |
| Sealed minting (the only `Receipt` constructor) | `Receipt::sealed` (`pub(crate)`), `ChainAssembler::finalize` | `src/types.rs:93`, `src/chain.rs:135` |
| Deser re-verifies the chain (anti-forgery) | `impl Deserialize for Receipt` | `src/types.rs:110`–`131` |
| Canonical bytes (deterministic JSON) | `canonical_bytes` + `sort_value` | `src/types.rs:545`,`:612` |
| Single profile | `enum ProfileId { CoreV1 }`; profile stage hardcoded | `src/types.rs:236`; `src/verifier.rs:191` |
| GPU batch prototype (gpu feature) | `gpu_verifier::GpuVerifier` | `src/1000x_gpu_verifier.rs`; wired `src/lib.rs:111` |
| Sharding prototype (orphaned) | `DistributedVerifier`, `ReceiptShard`, `ReceiptManifest`, `shard_receipt` | `src/1000x_distributed_sharding.rs` |
| Verify bench | `bench_verifier_pipeline`, `recompute_chain_100_events` | `benches/receipt_operations.rs:57`,`:79` |

### Determinism guarantees we must preserve (CLAUDE.md *Determinism*)

- **No wall-clock**: ordering is the monotonic `seq` (`src/types.rs:189`), never time.
- **Canonical JSON**: `canonical_bytes` recursively sorts keys (`src/types.rs:612`) so the same logical
  event always yields identical bytes — the precondition for *any* parallel/GPU/sharded recompute to
  agree with the CPU fold.
- **Same inputs → same receipt**: chain hash is reproducible; `verify` is pure (`verif_pure_deterministic`, `src/verifier.rs:255`).

### The gap (precise, cited)

1. **Profiles are not data, they are constants.** `STANDARD_VERSION` is a `const` equal to one string
   (`src/verifier.rs:22`); `stage_check_format` compares against it (`:92`); `stage_evaluate_profile`
   hardcodes the CoreV1 rule (`:191`). There is no registry, no `core/v2`, no negotiation/migration.
2. **No incrementality.** `recompute_chain` always folds from `genesis_hash()` over the *entire* slice
   (`src/chain.rs:69`); `verify` ingests a whole `&Receipt`. Appending one event re-verifies O(n).
   `ChainAssembler` already keeps a `running` hash (`src/chain.rs:85`) — the seed of incrementality is
   present but unexposed for *verification* (only assembly).
3. **No parallelism.** `verify` builds the outcomes `Vec` by calling stages in strict sequence
   (`src/verifier.rs:44`-`57`). `rayon` is an optional dep (`Cargo.toml:89`, behind `benchmarking`)
   but unused by the verifier.
4. **GPU prototype is not verdict-exact.** The WGSL `compress` is *"simplified to 1 round for
   prototype speed"* (`src/1000x_gpu_verifier.rs:115`) and the format-hash constant is a
   *"Placeholder"* (`:141`). So `GpuVerdict` ≠ `verifier::verify` today — it cannot be trusted as a
   certifier, only as a *prefilter*. Closing this is the heart of the GPU phase.
5. **Sharding prototype is orphaned and not Merkle-based.** `1000x_distributed_sharding.rs` is **not**
   declared in `src/lib.rs` (no `pub mod`), and it `use`s the **private** `chain::genesis_hash`
   (`:12`) — so it would not compile against the current `chain.rs`. It also re-implements stages
   locally (`verify_shard`, `:207`) instead of reusing `verifier::verify`, and stitches by boundary
   hashes (`:156`) rather than emitting verifiable **Merkle proofs**. It is a sketch to *supersede*,
   not adopt.

**Net gap:** the engine is correct but mono-profile, whole-chain, single-threaded, CPU-only, and
non-distributed. W7 closes all five while keeping the §1 bit-identity contract.

---

## 3. Phased plan

Each phase: **Objectives → Deliverables (compilable-style) → Acceptance (incl. determinism/equivalence)
→ Cross-workstream deps.** Phases are ordered so each rests on the last: profiles and incremental
recompute (the *correctness substrate*) precede parallel/GPU/distributed (the *performance/topology
multipliers*), because every multiplier must be checked against a profile-correct oracle.

---

### Phase 2026 H2 — Multi-profile foundation (make profiles data, not constants)

**Objective.** Replace the single hardcoded `core/v1` standard with a **profile registry** so the
verifier can certify a receipt under the profile it *declares* and refuse cleanly if that profile is
unknown — without changing the verdict for any existing `core/v1` receipt.

**Deliverables.**

- A `Profile` trait + a frozen `CoreV1` impl that reproduces today's stages 2 & 6 **verbatim** (so the
  refactor is verdict-preserving by construction).

```rust
// src/profiles/mod.rs  (new) — profiles become first-class, versioned objects.
use crate::types::{CheckOutcome, OperationEvent};

/// A frozen certification standard. Each profile is immutable once published:
/// a published `Profile` impl is NEVER edited — a new standard is a NEW impl
/// (CoreV2, ...), so a receipt minted under core/v1 always re-verifies identically.
pub trait Profile: Send + Sync {
    /// Wire identifier, e.g. "core/v1". Matched against `Receipt.format_version`.
    fn id(&self) -> &'static str;

    /// Stage 2 (check_format): is `declared` exactly this profile's id?
    /// Default reproduces `stage_check_format` (src/verifier.rs:91) bit-for-bit.
    fn check_format(&self, declared: &str) -> CheckOutcome {
        let passed = declared == self.id();
        CheckOutcome { stage: "check_format".into(), passed,
            detail: if passed { format!("format_version == {}", self.id()) }
                    else { format!("expected format_version {}, found {declared}", self.id()) } }
    }

    /// Stage 6 (evaluate_profile): per-event structural rule for THIS standard.
    fn evaluate(&self, events: &[OperationEvent]) -> CheckOutcome;
}

/// core/v1 — the existing standard, frozen. Mirrors src/verifier.rs:191 exactly.
pub struct CoreV1;
impl Profile for CoreV1 {
    fn id(&self) -> &'static str { "core/v1" }
    fn evaluate(&self, events: &[OperationEvent]) -> CheckOutcome {
        for e in events {
            if e.event_type.trim().is_empty() {
                return CheckOutcome { stage: "evaluate_profile".into(), passed: false,
                    detail: format!("event {} has an empty event_type", e.id) };
            }
            if e.payload_commitment.as_hex().is_empty() {
                return CheckOutcome { stage: "evaluate_profile".into(), passed: false,
                    detail: format!("event {} is missing a commitment", e.id) };
            }
        }
        CheckOutcome { stage: "evaluate_profile".into(), passed: true,
            detail: "profile core/v1 satisfied".into() }
    }
}
```

- A **registry** (deterministic resolution; `linkme` is already a dep, `Cargo.toml:42`, declared but
  unused per `00-SYNTHESIS.md` B10 — this is its idiomatic first use) plus a `verify_with` entry that
  threads a resolved profile through the *same* pipeline shape as `verify`.

```rust
// src/profiles/registry.rs (new)
use std::collections::BTreeMap;          // BTreeMap => deterministic iteration order.

pub fn resolve(id: &str) -> Option<&'static dyn Profile> {
    // Built from a linkme slice of (&'static str, &'static dyn Profile); BTreeMap
    // keeps resolution order stable across builds (no HashMap nondeterminism).
    static TABLE: once_cell::sync::Lazy<BTreeMap<&'static str, &'static dyn Profile>> = /* ... */;
    TABLE.get(id).copied()
}

// src/verifier.rs (additive — `verify` keeps its exact signature & behaviour)
pub fn verify_with(receipt: &Receipt, profile: &dyn Profile) -> Verdict { /* stages, profile-driven */ }

/// Back-compat shim: today's `verify` == verify_with(CoreV1). Existing callers/tests unchanged.
pub fn verify(receipt: &Receipt) -> Verdict { verify_with(receipt, &crate::profiles::CoreV1) }
```

- **Profile negotiation & migration (design + types only this phase).** A receipt *declares* one
  profile; negotiation is choosing the verifier-side profile to apply (default: the declared one;
  `--profile` can request a *stricter superset* check, never a laxer one — certifying laxly would be a
  decision, forbidden). Migration is **re-emit/re-assemble under the new standard**, never an in-place
  edit (which would break the chain). Document that `core/vN` are *append-only* siblings.

**Acceptance.**
- **Equivalence:** `verify(r) == verify_with(r, &CoreV1)` for a battery of receipts (honest, tampered,
  seq-gap, dup-id, bad-commitment, wrong-format). Byte-equal `Verdict` (outcomes + reason).
- All existing verifier tests (`src/verifier.rs:246`-`316`) pass **unmodified**.
- `resolve("core/v1").is_some()` and `resolve("core/v99").is_none()`; an unknown declared profile
  fails at `check_format` with a clear reason, never silently ACCEPTs.
- Registry order is deterministic (BTreeMap), asserted by a test that resolves twice and compares.

**Cross-workstream deps.** **W9** (Ecosystem & Standards) co-authors the `core/v2` *spec* (W7 supplies
the trait/registry mechanics; W9 owns what v2 *means*). **W3** consumes the new "unknown profile"
reason via its error catalog. **W2**'s `ProfileMismatch` finding (`docs/innovation/02-doctor-receipts.md`
appendix) starts calling `resolve` instead of comparing a constant.

---

### Phase 2027 — Incremental / streaming verification (verify the suffix, not the whole)

**Objective.** Append events and re-verify in **O(new events)**, not O(n), by checkpointing the rolling
state — while guaranteeing the incremental result equals a full `verify` of the resulting receipt.

**Deliverables.**

- A **verification checkpoint** capturing exactly the state stage 3 + stage 4 need to resume: the
  running chain hash, the next expected `seq`, and a commitment to the id-set (for the uniqueness
  check). The checkpoint is itself content-addressed so it cannot silently drift from the prefix.

```rust
// src/incremental.rs (new)
use crate::types::{Blake3Hash, OperationEvent, Receipt, Verdict};

/// Resumable verification state after verifying a prefix of `n` events.
/// `running` mirrors ChainAssembler.running (src/chain.rs:85): the fold up to event n-1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyCheckpoint {
    pub verified_len: usize,        // how many events are already certified
    pub running: Blake3Hash,        // rolling chain hash after `verified_len` events
    pub next_seq: u64,              // == verified_len (continuity invariant)
    pub idset_digest: Blake3Hash,   // order-independent commitment to seen event ids
}

impl VerifyCheckpoint {
    pub fn genesis() -> Self { /* running = blake3(GENESIS_SEED) (src/chain.rs:48), len 0, seq 0 */ }
}

/// Verify ONLY `new_events`, resuming from `cp`. Folds with the SAME `fold_event`
/// rule as recompute_chain (src/chain.rs:54) so the resulting hash is identical to
/// a from-genesis recompute over prefix++new_events.
pub fn verify_append(
    cp: &VerifyCheckpoint,
    new_events: &[OperationEvent],
    profile: &dyn crate::profiles::Profile,
) -> Result<(VerifyCheckpoint, Verdict), crate::chain::ChainError> {
    // continuity: each new event's seq must equal next_seq, next_seq+1, ...
    // chain: fold new_events into cp.running
    // commitments + profile.evaluate: run over new_events only
    // idset: fold each new id into idset_digest; reject on collision
    // returns advanced checkpoint + a Verdict scoped to the appended suffix
    todo!("incremental suffix verify; see equivalence theorem below")
}
```

- A **streaming assembler seam** so `affi emit` can persist `(working events, checkpoint)` and a future
  `affi verify --incremental` resumes from the checkpoint instead of re-folding. This generalizes
  `ChainAssembler::from_events` (`src/chain.rs:107`) — which today *always* recomputes from genesis —
  into a `resume(events, checkpoint)` that trusts the checkpoint *only after re-deriving it once*.

```rust
// src/chain.rs (additive, sketch) — checkpoint is verified, never blindly trusted.
impl ChainAssembler {
    /// Resume assembly from a prior checkpoint. SAFETY: re-folds the first event
    /// against `cp.running` and aborts if the published `running` disagrees with a
    /// fresh recompute of the prefix => a stale/forged checkpoint can never seed a chain.
    pub fn resume(prefix: &[OperationEvent], cp: &VerifyCheckpoint) -> Result<Self, ChainError> { todo!() }
}
```

- **Equivalence theorem (documented + property-tested).** For any split `events == prefix ++ suffix`:
  `verify_append(checkpoint_of(prefix), suffix, P).1.accepted == verify_with(seal(events), P).accepted`,
  and the final checkpoint's `running == recompute_chain(events)`. This is provable because
  `fold_event` is a left fold (associative resumption): folding `suffix` onto the prefix's running hash
  equals folding `prefix ++ suffix` from genesis — the exact identity `from_events` relies on today
  (`from_events_reconstructs_running_hash`, `src/chain.rs:293`).

**Acceptance.**
- **Equivalence (property test):** randomized `events` and split points; incremental verdict ≡ full
  verdict; final `running` ≡ `recompute_chain(events)`. Run single-threaded for determinism per CLAUDE.md.
- **Tamper still caught:** mutating any event in the *already-checkpointed prefix* and re-checkpointing
  must fail the `resume` re-derivation (forged checkpoint rejected). A tamper in the suffix fails
  `verify_append`'s chain step.
- **Anti-forgery preserved:** the sealed-construction invariant is untouched — `resume` ends at the
  existing `finalize` (`src/chain.rs:135`); `_seal` stays private; deser still re-verifies (`src/types.rs:127`).
- **Perf:** appending k events to an n-event chain is O(k), not O(n+k) — assert via a bench extending
  `recompute_chain_100_events` (`benches/receipt_operations.rs:79`) with an incremental variant.

**Cross-workstream deps.** **W5** (Workflow Automation) drives `affi watch`/incremental emit; it calls
`verify_append` per file-change so a CI loop re-certifies only the delta. **W2** uses checkpoints to
make store scans incremental (re-score only changed receipts). **W8** notes: a checkpoint is *not* a
trust anchor — signing the checkpoint is W8's call.

---

### Phase 2028 — Parallel verification (independent stages concurrent; verdict terminal)

**Objective.** Exploit the structure of `verify` — stage 3 is a *sequential fold*, but stages 4, 5, 6
are *independent per-event passes* — to verify large receipts across cores, yielding a `Verdict`
bit-identical to the sequential one. Stage 7 (`emit_verdict`) remains the single terminal reducer.

**Why this is safe to parallelize (grounded).** Looking at `src/verifier.rs:44`-`57`: stage 4
(continuity, `:136`), stage 5 (commitments, `:169`), stage 6 (profile, `:191`) each iterate events and
short-circuit on the *first* failure — but the *set* of failures is a pure function of the events, and
the verdict only needs the **first failing stage in pipeline order** (`first_failure`, `:60`). So we can
compute each stage's outcome independently and then reduce in fixed stage order — order of *computation*
is irrelevant; order of *reduction* is fixed. Stage 3's fold is inherently sequential **per chain**, but
parallelizable across *receipts* (the GPU/batch phase) and chunkable with a tree-fold *only if* the fold
were associative over event bytes — it is not (it threads `prev.as_hex()` into each step,
`src/chain.rs:57`), so within one chain stage 3 stays sequential; we parallelize 4/5/6 against it.

**Deliverables.**

- A **stage scheduler** that runs stage 3 on one task and stages 4–6 on a `rayon` pool (`rayon` already
  a dep, `Cargo.toml:89`; promote it out from under `benchmarking` into a new `parallel` feature), then
  reduces deterministically. The per-stage "first failure" must be the *lowest-index* event, found via
  a deterministic reduction (min by event index), **not** "whichever thread reported first."

```rust
// src/parallel.rs (new) — behind feature = "parallel" (rayon). Determinism by construction.
use crate::types::{CheckOutcome, Receipt, Verdict};

/// Parallel verify: identical Verdict to verifier::verify, computed concurrently.
/// Invariant: each stage's outcome is a PURE function of `receipt`; we only change
/// the ORDER OF COMPUTATION, never the order of REDUCTION (fixed pipeline order).
#[cfg(feature = "parallel")]
pub fn verify_parallel(receipt: &Receipt, profile: &dyn crate::profiles::Profile) -> Verdict {
    use rayon::prelude::*;
    // Stage 3 (sequential fold) on its own task:
    let s3 = std::thread::scope(|s| {
        let h = s.spawn(|| stage_chain_integrity(receipt)); // == src/verifier.rs:109
        // Stages 4,5,6: each scans events; for 4/5/6 the "first failure" is the MIN
        // event index that fails — computed with a deterministic parallel reduce:
        let s4 = continuity_first_failure(receipt);   // par map -> reduce(min-by-index)
        let s5 = commitments_first_failure(receipt);
        let s6 = profile.evaluate(receipt.events());  // profile-driven (Phase 2026 H2)
        (h.join().unwrap(), s4, s5, s6)
    });
    // Reduce in FIXED pipeline order — identical to verifier::verify's vec! order (src/verifier.rs:44):
    let outcomes: Vec<CheckOutcome> =
        vec![stage_decode(receipt), stage_check_format(receipt, profile), s3.0, s3.1, s3.2, s3.3];
    emit_verdict(outcomes)   // == src/verifier.rs:59-72 (the terminal reducer, unchanged)
}
```

- A **batch facade** `verify_many(&[Receipt])` that parallelizes *across receipts* (embarrassingly
  parallel — each receipt is independent), the natural CPU analog of the GPU batch path and the input
  to W2's store scan and W10's audit sweeps.
- **Determinism harness:** a differential test that runs `verify` and `verify_parallel` over the same
  corpus under varying thread counts (`RAYON_NUM_THREADS` ∈ {1,2,8,…}) and asserts byte-equal verdicts.

**Acceptance.**
- **Equivalence under nondeterministic scheduling:** `verify_parallel(r) == verify(r)` for the full
  corpus, **for every thread count** — the core anti-flake guarantee. Includes receipts with multiple
  simultaneous defects (e.g., seq-gap *and* bad commitment) to prove the *first-failing-stage* reduction
  matches sequential order, not race order.
- No data races (single `&Receipt`, read-only; outputs are owned `CheckOutcome`s reduced after join).
- `_seal` invariant untouched (verifier never constructs receipts).
- **Perf:** `verify_many` of N receipts scales ~linearly to core count; document that *within* one chain,
  stage 3 is the serial floor (Amdahl) — large *batches* win, single huge chains are bounded by the fold.

**Cross-workstream deps.** **W2** store scan and **W10** compliance sweeps consume `verify_many`. **W6**
(Interactive Surfaces) gets responsive bulk-verify for TUI/LSP. **W3** owns how parallel progress is
rendered (W7 emits structured per-receipt results; W3 formats).

---

### Phase 2029 — GPU acceleration (verdict-exact batch verification)

**Objective.** Turn the `gpu` prototype (`src/1000x_gpu_verifier.rs`) from a *fast approximate
prefilter* into a **verdict-exact batch certifier**: a GPU pass whose ACCEPT set is provably identical
to running `verifier::verify` on each receipt — or, where exactness on-GPU is impractical, a sound
**two-tier** design (GPU rejects fast; CPU oracle confirms every GPU-ACCEPT before any receipt is
treated as certified).

**The exactness gap to close (grounded).** Three things make today's `GpuVerifier` non-exact:
1. WGSL `compress` is *"simplified to 1 round"* (`src/1000x_gpu_verifier.rs:115`) — not real BLAKE3, so
   the GPU chain hash ≠ `recompute_chain`.
2. The format-version check uses a *"Placeholder"* constant (`:141`).
3. The GPU hashes `event_type`/`id` into fixed `[u32;8]` lanes (`prepare_batch`, `:289`) rather than
   folding the **canonical event bytes** (`canonical_bytes`, `src/types.rs:545`) that the CPU fold uses
   (`src/chain.rs:55`) — so even with full BLAKE3 the *preimage* differs.

**Deliverables.**

- A **full BLAKE3-256 WGSL** implementation (all 7 rounds, correct message schedule, length/flags) that
  reproduces `blake3::hash` exactly, plus a host-side packer that feeds the GPU the **same canonical
  bytes per event** the CPU folds — so the GPU recomputes the *identical* rolling chain hash. This is
  the linchpin: same preimage + same hash ⇒ same `chain_integrity`.

```rust
// src/gpu/exact.rs (new, feature = "gpu") — exactness contract over the prototype.
/// Pack a receipt for the GPU using the SAME bytes the CPU folds.
/// Each event contributes `canonical_bytes(event)` (src/types.rs:545); the shader
/// folds blake3(prev_hex || event_bytes) (mirroring fold_event, src/chain.rs:54),
/// so the GPU chain hash is bit-identical to recompute_chain (src/chain.rs:68).
pub fn pack_exact(receipts: &[crate::types::Receipt]) -> GpuBatch { todo!() }

/// Two-tier verdict: GPU is allowed to REJECT autonomously (a fast filter), but a
/// GPU-ACCEPT is provisional until the CPU oracle confirms. This makes a shader bug
/// a (rare) false-REJECT at worst — NEVER a false-ACCEPT laundered into certification.
pub enum GpuClass { RejectedByGpu(u32 /*stage bitmask*/), AcceptedPendingOracle }

#[cfg(feature = "gpu")]
pub async fn verify_batch_exact(receipts: &[crate::types::Receipt]) -> Vec<crate::types::Verdict> {
    let classes = run_gpu(pack_exact(receipts)).await;            // massively parallel
    receipts.iter().zip(classes).map(|(r, c)| match c {
        GpuClass::RejectedByGpu(_)        => crate::verifier::verify(r), // CPU produces the exact reason
        GpuClass::AcceptedPendingOracle   => crate::verifier::verify(r), // CONFIRM on the oracle
    }).collect()
}
```

- A **GPU↔CPU differential conformance suite** run in CI on every shader change: for a large randomized
  corpus, `verify_batch_exact(corpus)` must equal `corpus.map(verify)` elementwise. A KAT (known-answer
  test) pins the WGSL BLAKE3 against `blake3::hash` on fixed vectors so a shader regression fails loudly.
- A **fallback contract:** no GPU adapter (`GpuVerifier::new` errors today, `src/1000x_gpu_verifier.rs:208`)
  ⇒ transparently fall back to `verify_many` (Phase 2028). GPU is an *accelerator*, never a *requirement*;
  the verdict is identical with or without it.

**Acceptance.**
- **Bit-exactness:** GPU verdict set ≡ CPU oracle verdict set on ≥10⁶ receipts incl. adversarial tampers
  (flipped commitments, seq gaps, wrong format). **Zero false-ACCEPTs is a hard gate** (the two-tier
  oracle confirm makes this structural, not merely tested).
- **KAT:** WGSL BLAKE3 matches `blake3::hash` on the standard test vectors.
- **Doctrine:** the GPU never *mints* a `Receipt` (no `_seal` access from the shader path); it only
  classifies. Certification still flows through `verifier::verify`. A reviewer can grep: the only
  ACCEPT-producing call in the GPU path is `verifier::verify`.
- **Determinism:** GPU floating-point is irrelevant (all integer ops); workgroup reduction order does not
  affect per-receipt results (each receipt is one invocation, `:131`). Document that batch *ordering* of
  outputs is index-stable (`verdicts[idx]`, `:191`).

**Cross-workstream deps.** **W8** owns whether a GPU-recomputed root may anchor a *signature/transparency*
claim (W7 guarantees the root is correct; W8 decides its trust weight). **W10** (Compliance) uses
high-throughput batch verify for portfolio-scale audit. **W1** (Foundations) owns CI GPU-runner provisioning.

---

### Phase 2030 — Distributed verification (Merkle proofs across shards; partial-receipt proofs)

**Objective.** Verify receipts too large for one machine by **sharding** them and exchanging **Merkle
proofs** instead of whole chains — enabling (a) parallel verification across nodes and (b)
*partial-receipt* proofs (prove event *e* is in receipt *R* without shipping all of *R*) — with a
**globally stitched verdict bit-identical to a single-machine `verify`** of the reassembled receipt.

**Supersede the prototype (grounded).** `1000x_distributed_sharding.rs` is the right *shape* (shards +
manifest + parallel coordinator, `:96`-`:190`) but: it is **orphaned** (not in `src/lib.rs`), imports the
**private** `genesis_hash` (`:12`, would not compile), re-implements stages locally (`verify_shard`,
`:207`) instead of reusing `verifier::verify`, and **stitches by boundary chain hashes** (`:156`) rather
than Merkle proofs (so it cannot do partial-receipt proofs). The 2030 design *replaces* the stitching
layer with a Merkle commitment over shard roots while **keeping** the sound boundary-continuity checks.

**Deliverables.**

- A **shard Merkle tree** over per-shard chain roots, with an inclusion-proof type. The manifest commits
  to the Merkle root; a node can verify *its* shard + a logarithmic proof that the shard belongs to the
  whole, without holding sibling shards.

```rust
// src/distributed/merkle.rs (new) — proofs replace whole-chain shipping.
use crate::types::Blake3Hash;

/// Inclusion proof that a shard's root is the `index`-th leaf under `root`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleProof {
    pub leaf: Blake3Hash,            // this shard's chain root
    pub index: usize,
    pub siblings: Vec<Blake3Hash>,   // O(log shard_count) audit path
}

impl MerkleProof {
    /// Recompute the root from leaf+siblings using blake3 (same hash family as the chain).
    /// Deterministic: fixed left/right ordering by bit of `index`, canonical concat.
    pub fn recompute_root(&self) -> Blake3Hash { todo!() }
    pub fn verify(&self, expected_root: &Blake3Hash) -> bool { &self.recompute_root() == expected_root }
}

/// A shard verified in isolation + its membership proof. Crucially, local checks
/// REUSE the canonical pipeline — not a re-implementation — to stay verdict-exact.
pub struct ShardVerification {
    pub shard_root: Blake3Hash,
    pub prev_boundary: Blake3Hash,   // running hash entering the shard (continuity stitch)
    pub local: crate::types::Verdict, // from verifier::verify over a sealed sub-receipt
    pub proof: MerkleProof,
}
```

- A **distributed coordinator** that fetches shards + proofs in parallel, verifies each shard's *local*
  verdict **through `verifier::verify`** (sealing each shard's events into a real sub-`Receipt` so the
  sealed invariant holds end-to-end), checks every Merkle proof against the manifest root, and stitches
  boundary continuity (each shard's `prev_boundary` equals the prior shard's root). The global verdict is
  ACCEPT iff: all shards locally ACCEPT ∧ all proofs verify ∧ all boundaries chain ∧ stitched root ==
  manifest root.

```rust
// src/distributed/coordinator.rs (new)
/// Global verdict == single-machine verify of the reassembled receipt. Proven by:
///  (1) each shard's events seal+verify via the SAME verifier::verify (no local re-impl);
///  (2) Merkle proofs bind each shard root into one manifest root (tamper-evident membership);
///  (3) boundary continuity makes the concatenation equal one from-genesis fold (fold_event assoc).
pub fn verify_distributed(manifest: &ShardManifest, fetch: &dyn ShardSource) -> Verdict { todo!() }
```

- **Partial-receipt proofs:** a `prove_event(receipt, seq) -> (OperationEvent, MerkleProof, BoundaryWitness)`
  and `verify_event_inclusion(...)` so a third party can confirm one event is in a receipt against the
  published manifest root **without** the full chain — the distributed analog of `affi show <one event>`,
  and the substrate W10 needs for selective-disclosure audit.
- **Genesis-seed hygiene:** expose a *public, stable* genesis accessor for the distributed crate (the
  prototype's illegal `use crate::chain::genesis_hash` at `:12` is replaced by a sanctioned API), and
  pin it to the package version to avoid the `GENESIS_SEED` drift flagged as bug **B4** in
  `00-SYNTHESIS.md` (seed says `v26.6.14`, package is `26.6.17`, `src/chain.rs:22`). *(W1 owns the actual
  seed-version fix; W7 only consumes a stable accessor.)*

**Acceptance.**
- **Equivalence:** for any receipt and any shard size, `verify_distributed(shard(r))` ≡ `verify(r)`
  (accepted + first-failing stage). Mirrors and *replaces* the prototype's flow test
  (`test_distributed_verification_flow`, `src/1000x_distributed_sharding.rs:321`), now asserting verdict
  *exactness* against the oracle, not just `accepted == true`.
- **Proof soundness:** a forged shard (tampered event) fails its *local* `verify` (chain mismatch); a
  forged *membership* (right shard, wrong position/root) fails `MerkleProof::verify`; a forged *boundary*
  fails stitching. Property tests fuzz each independently.
- **Partial-proof soundness:** `verify_event_inclusion` accepts iff the event is genuinely in the receipt
  under the manifest root; flipping any event byte or proof sibling makes it reject.
- **Determinism:** Merkle tree shape is a pure function of shard roots (fixed fan-in, fixed L/R ordering);
  coordinator reduces shard results in shard-index order (BTreeMap, as the prototype already does at
  `:116`), so the global verdict is independent of fetch/arrival order.
- **Doctrine & seal:** every ACCEPT still terminates in `verifier::verify` over sealed (sub-)receipts;
  no distributed path constructs a `Receipt` outside `finalize`; W7 distributes *recomputation*, never
  *trust*.

**Cross-workstream deps.** **W8** owns signing the manifest Merkle root and any transparency-log
anchoring of it (W7 makes the root correct & provable; W8 makes it *attestable*). **W9** standardizes the
shard/manifest/proof **wire format** (a `dist/v1` profile sibling to `core/vN`). **W10** consumes
partial-receipt proofs for selective-disclosure compliance. **W1** provides the public genesis accessor
and fixes B4.

---

## 4. Definition of done @ 2030

The verification engine is, end to end:

1. **Multi-profile.** `core/v1` and ≥1 successor (`core/v2`) coexist as frozen `Profile` impls behind a
   deterministic registry; receipts certify under their declared profile; unknown profiles fail cleanly;
   migration is re-emit, never in-place edit. `verify(r) == verify_with(r, declared_profile)` for all
   `core/v1` receipts (back-compat proven).
2. **Incremental.** Appending k events re-verifies in O(k) via verified checkpoints; the incremental
   verdict equals a full `verify` of the result; stale/forged checkpoints are rejected on resume.
3. **Parallel.** `verify_parallel` and `verify_many` produce byte-identical verdicts to the sequential
   oracle *under any thread count*; batches scale ~linearly; the stage-3 fold is the documented serial floor.
4. **GPU-accelerated.** `verify_batch_exact` is verdict-exact (full-BLAKE3 WGSL + canonical-byte packing +
   two-tier oracle confirm) with **zero false-ACCEPTs by construction**; absent a GPU it transparently
   falls back to CPU batch with identical results.
5. **Distributed.** Receipts shard, exchange Merkle proofs, and stitch to a global verdict bit-identical
   to single-machine `verify`; partial-receipt (selective-disclosure) proofs are sound; the manifest root
   is the W8 attestation surface.

**Across all five, the §1 prime directive holds:** every path round-trips to `verifier::verify` bit-for-bit,
and **no path constructs a `Receipt` outside `ChainAssembler::finalize`** (`_seal` stays private,
`src/types.rs:222`; deser keeps re-verifying, `src/types.rs:127`). The engine got faster, incremental,
parallel, GPU-accelerated, and distributed — and the *verdict never changed*, and "is this work honest?"
was *never* decided. Determinism guarantees from CLAUDE.md (no wall-clock, canonical JSON, reproducible
hashes) are preserved at every layer.

### How determinism is preserved under each optimization (summary)

| Optimization | Nondeterminism risk | How W7 neutralizes it |
|---|---|---|
| Multi-profile | profile resolution order | `BTreeMap` registry; profiles frozen (new standard = new impl, never an edit) |
| Incremental | checkpoint drift | checkpoint is content-addressed + re-derived on `resume` before use; `fold_event` left-fold associativity = identical hash |
| Parallel | race in "first failure" | compute order ≠ reduce order; reduce in *fixed* pipeline order; first failure = **min event index**, not first thread |
| GPU | shader bug / FP / preimage | full-BLAKE3 KAT + canonical-byte packing; integer-only; **two-tier oracle confirm** ⇒ no false-ACCEPT |
| Distributed | fetch/arrival order, forged membership | shard-index `BTreeMap` reduction; Merkle proofs (fixed L/R); each shard sealed + `verify`'d; boundary stitch = single fold |

---

## 5. Cross-workstream dependencies (consolidated)

| Phase | W7 provides | W7 depends on |
|---|---|---|
| 2026 H2 Multi-profile | `Profile` trait + registry mechanics; "unknown profile" reason; `verify_with` | **W9** co-authors `core/v2` spec; **W3** error catalog entry; **W2** `resolve` for `ProfileMismatch` |
| 2027 Incremental | `VerifyCheckpoint`, `verify_append`, `ChainAssembler::resume` | **W5** drives incremental emit/watch; **W2** incremental store re-score; **W8** (checkpoint ≠ trust anchor) |
| 2028 Parallel | `verify_parallel`, `verify_many`, determinism harness | **W2**/**W10** consume `verify_many`; **W6** responsive bulk verify; **W3** progress rendering; **W1** `parallel` feature gate |
| 2029 GPU | `verify_batch_exact`, GPU↔CPU differential suite, CPU fallback | **W8** trust weight of GPU-recomputed root; **W10** portfolio audit throughput; **W1** CI GPU runners |
| 2030 Distributed | Merkle proof types, distributed coordinator, partial-receipt proofs | **W8** signs/anchors manifest root; **W9** standardizes `dist/v1` wire format; **W10** selective-disclosure; **W1** public genesis accessor + B4 fix |

**Hard boundary reminders:** signing/attestation/PQC/transparency logs → **W8** (W7 makes roots correct &
provable, never *trusted*). Store health/quarantine/repair → **W2** (W7 supplies the engine it calls).
Output formatting/exit codes → **W3**. Standard *semantics* of `core/vN` and wire formats → **W9**. W7's
lane is strictly verification *mechanics, performance, and distribution topology* — and the bit-identity
contract with the sequential oracle is the law that governs all of it.
