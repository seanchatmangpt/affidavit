# affidavit-core

A `#![no_std]`, `#![forbid(unsafe_code)]` reference verifier for affidavit-style
receipt chains. It implements **zero cryptographic algorithms** — every
algorithm is an external dependency.

> **On "1000x".** This repo brands features as "1000x". That's marketing, not a
> measured number, and this crate doesn't pretend otherwise. What it *does* is
> take the one move a Rust engineer would actually defend as a step-change for a
> trust-critical verifier: **implement no crypto of its own, run anywhere, and
> make invalid receipts unrepresentable.** The sibling `affidavit` crate
> builds and tests green as of v26.9.6; this core stays the
> no-implemented-crypto substrate it compiles against.

## Why this shape

| Property | What it buys you |
|---|---|
| **No crypto implemented in-crate** | Every algorithm is external: **BLAKE3** (`blake3` crate) for chain hashing and **ML-DSA-65** (FIPS 204, RustCrypto `ml-dsa` crate) for signatures — both unconditional `[dependencies]` with `default-features = false`, so `no_std` is preserved. No feature flag can compile this crate without them; none needs to. The supply-chain surface is two externally audited crates, not in-crate hand-rolled arithmetic. |
| **`no_std`, no `alloc` on the verify path** | [`verify`] runs over a borrowed `&[Event]` and allocates nothing — it runs in a WASM sandbox, an embedded HSM, or an on-chain runtime, not just on a server. |
| **`forbid(unsafe_code)`** | Memory safety is enforced crate-wide, by the compiler. |
| **Compile-time sealing** | A `Receipt` is only obtainable from `ChainBuilder::finalize`. A private witness field makes a hand-forged `Receipt` an `E0451` compile error — proven by a `compile_fail` doctest. |
| **Pluggable hash** | The chain folds through any `ChainHasher`. `Fnv256` (**non-cryptographic**) is the in-crate reference hasher; [`Blake3Hasher`](#no-in-crate-crypto-law) delegates to the external `blake3` crate for tamper-*evident* chains. |

## Use it

```rust
use affidavit_core::{ChainBuilder, Digest, Fnv256, Verdict};

let receipt = ChainBuilder::<Fnv256>::new()
    .event("build", "evt-0", Digest([1u8; 32]))
    .event("test",  "evt-1", Digest([2u8; 32]))
    .finalize();

assert_eq!(receipt.verify::<Fnv256>(), Verdict::Accept);
```

The allocation-free verify path (no builder, no `alloc`) works over a borrowed
slice you own:

```rust
use affidavit_core::{compute_chain_hash, verify, Event, Digest, Fnv256, Verdict, PROFILE};

let events = [
    Event { seq: 0, event_id: "evt-0", event_type: "build", commitment: Digest([1u8; 32]) },
    Event { seq: 1, event_id: "evt-1", event_type: "test",  commitment: Digest([2u8; 32]) },
];
let chain_hash = compute_chain_hash::<Fnv256>(&events);
assert_eq!(verify::<Fnv256>(&events, &chain_hash, PROFILE), Verdict::Accept);
```

## The verifier

A total, fail-fast function from a borrowed chain to a `Verdict`. It condenses
affidavit's 7-stage pipeline to the structural stages meaningful in a pure core,
in the same order:

| affidavit stage | here |
|---|---|
| 2 `check_format` | profile tag equals `PROFILE` |
| 3 `chain_integrity` | recomputed rolling hash equals the stored one |
| 4 `continuity` | `seq` is `0..n`; `event_id`s are unique |
| 5 `verify_commitments` | every commitment is non-zero (well-formed) |
| 6 `evaluate_profile` | every `event_type` is non-empty |

Rejection carries the **first** failing reason (a `RejectReason` enum — no
allocation, no strings). *Certify, don't decide:* it reports conformance to the
format and never judges whether the recorded work was honest.

### Chain construction

Per-link rolling fold: `acc₀ = H(DOMAIN)`, then `accᵢ = H(accᵢ₋₁ ‖ encode(eventᵢ))`,
with each event's fields length-prefixed so field boundaries can't alias. Any
edit to any event propagates through every later link.

## Process mining (van der Aalst)

A receipt chain **is an event log**: each event's `event_type` is an *activity*
and `seq` gives the order, so one receipt is one *trace*. The `affidavit_core::mining`
module (behind the default `alloc` feature) puts the foundational process-mining
lens of Wil van der Aalst's lineage onto that log: **directly-follows graphs** for
discovery, the **α-algorithm footprint** for control-flow relations, and
**conformance checking** by **token replay**. It holds the crate's discipline —
no implemented crypto, no `unsafe` — and is deterministic throughout
(`BTree`-backed),
so a discovered model is reproducible.

| Piece | What it gives you |
|---|---|
| **`Trace`** | A receipt's activity sequence. `Trace::from_events(&[Event])` projects a chain; `Trace::from_activities(&["a","b","c"])` builds one directly. |
| **`DirectlyFollowsGraph`** (discovery) | `DirectlyFollowsGraph::discover(&[Trace])` builds the directly-follows relation: `.directly_follows(a,b) -> u64`, `.follows(a,b) -> bool`, `start`/`end` activity multisets, and `.activity_list()`. |
| **`footprint::Footprint`** (α-algorithm) | `Footprint::from_dfg(&dfg)`, then `.relation(a,b) -> AlphaRelation` — one of `Causality` (a→b), `ReverseCausality` (a←b), `Parallel` (a‖b), or `Choice` (a#b). |
| **`conformance::replay`** (token replay) | `replay(&model, &trace) -> ConformanceResult` with `.fitness() -> f64`, `.is_conformant() -> bool`, `.verdict() -> ConformanceVerdict` (`Conformant`/`NonConformant`), and `first_violation: Option<usize>`. A move is legal iff it's a directly-follows edge in the model; the first activity must be a model start and the last a model end. |
| **`stats::LogStatistics`** | `LogStatistics::from_traces(&[Trace])`: `event_count`, `trace_count`, `activity_frequency`, `variants`, plus `.distinct_activities()`, `.distinct_variants()`, `.most_frequent_variant()`. |

```rust
use affidavit_core::mining::{Trace, DirectlyFollowsGraph};
use affidavit_core::mining::footprint::{Footprint, AlphaRelation};
use affidavit_core::mining::conformance::{replay, ConformanceVerdict};

// Two traces over the same activities — `build → test → deploy`.
let traces = [
    Trace::from_activities(&["build", "test", "deploy"]),
    Trace::from_activities(&["build", "test", "deploy"]),
];

// Discover a model: the directly-follows graph.
let model = DirectlyFollowsGraph::discover(&traces);
assert_eq!(model.directly_follows("build", "test"), 2);
assert!(model.follows("test", "deploy"));

// Read an α-relation off the footprint.
let footprint = Footprint::from_dfg(&model);
assert_eq!(footprint.relation("build", "test"), AlphaRelation::Causality);

// A conforming trace replays fully (fitness 1.0).
let ok = replay(&model, &Trace::from_activities(&["build", "test", "deploy"]));
assert_eq!(ok.verdict(), ConformanceVerdict::Conformant);
assert!(ok.is_conformant());

// `build → deploy` is not a directly-follows edge in the model.
let bad = replay(&model, &Trace::from_activities(&["build", "deploy"]));
assert_eq!(bad.verdict(), ConformanceVerdict::NonConformant);
assert_eq!(bad.first_violation, Some(0));
```

*Certify, don't decide* holds here too: conformance **certifies** a trace against
a discovered model — it does not judge whether the recorded work was honest.

This is classic control-flow mining. **Object-centric** event logs (OCEL — also
van der Aalst's lineage) extend this from one sequence-per-receipt to activities
shared across multiple objects.

## OCEL mining (object-centric extension)

`affidavit_core::mining::ocel` (behind the default `alloc` feature) lifts the
directly-follows lens from single *traces* to events that carry **object
references**: one activity may touch many objects, and the model is mined
*per object type* rather than per receipt.

| Piece | What it gives you |
|---|---|
| **`OcelEvent`** | An activity plus its object references (`(object_id, object_type)` pairs) — the object-centric analogue of `Event`. |
| **`ocel::object_traces`** | Projects a slice of `OcelEvent`s into per-object-type `Trace`s, preserving `seq` order within each object. |
| **`ocel::discover_per_object`** | Discovers one `DirectlyFollowsGraph` per object type from an event log. |
| **`ocel::conformance_per_object`** | Token-replays each object's trace against its type's model, returning per-type `ConformanceResult`s. |

```rust
use affidavit_core::mining::ocel::{OcelEvent, object_traces, discover_per_object};

let log = [
    OcelEvent::new("build", [("repo", "repository")]),
    OcelEvent::new("test",  [("repo", "repository"), ("bin", "artifact")]),
    OcelEvent::new("ship",  [("bin", "artifact")]),
];

// One trace per object type: `repository` sees build→test, `artifact` sees test→ship.
let traces = object_traces(&log);
let models = discover_per_object(&log);
assert!(models["repository"].follows("build", "test"));
assert!(models["artifact"].follows("test", "ship"));
```

Doctrine holds: OCEL mining certifies per-object control flow against discovered
models — it never judges whether the recorded work was honest.

## Post-quantum verification court

Operator law: **no cryptographic algorithm is implemented anywhere in this
project — all algorithms are external.** The signature algorithm surface is
fixed to **ML-DSA-65 only** (FIPS 204, via the external RustCrypto `ml-dsa`
crate, an unconditional dependency — no feature flag). `affidavit_core::pqc`
defines the seam over it:

- **`MlDsa65Verifier`** — the FIPS 204 (ML-DSA-65) verification contract,
  implemented by the external `ml-dsa` crate or a consumer adapter. Total,
  boolean-collapsed: any malformed input returns `false`, never panics.
- **`ExternalMlDsa65`** — the shipped zero-size adapter over `ml-dsa`.
  Unconditional by operator law: no `pqc` feature flag exists; it compiles
  always so the court reaches the `affidavit-wasm` artifact.
- **`PqcProfile`** — `Classical` / `Hybrid` / `Pqc`, the declared assurance
  level of an envelope.
- **`PqcCourt`** — the fail-fast certification court: registers a verifier,
  runs only the checks the declared profile demands in order, and stops at
  the first failure. A profile demanding an algorithm with no verifier
  supplied is a typed `PqcRefusal` (e.g. `MissingMlDsa65Verifier`), never a
  skip.

*Certify, don't decide* here means the court reports what the external
verifier said; it never manufactures a cryptographic judgment of its own.
External validation lives in `tests/pqc_external_validation.rs` — real
collaborators, not mocks, and not in-crate reimplementations.

## No in-crate crypto (law)

The crate implements **zero cryptographic algorithms**. The algorithm surface
is fixed by operator law:

| Purpose | Algorithm | External dependency (unconditional) |
|---|---|---|
| Chain hashing | BLAKE3 | `blake3` (`default-features = false`, `no_std`) |
| Signatures | ML-DSA-65 (FIPS 204) | `ml-dsa` (RustCrypto, `default-features = false`, `no_std`) |

Nothing else is lawful: no SLH-DSA, no SHA-256, no classical signature
algorithms, no crypto feature flags. In-crate you get only structure:
`Fnv256` (an explicitly **non-cryptographic** reference hasher, retained as
such) and the trait seams themselves.

## Features

- `alloc` (default) — enables the owned `ChainBuilder` / `Receipt` and the
  whole `mining` module (including OCEL). Turn it off
  (`default-features = false`) for the pure, allocation-free verify path.
  These are the *only* features (`default`, `alloc`); there are no crypto
  feature flags — BLAKE3 and ML-DSA-65 ship unconditionally.

## Caveats (honest ones)

- **No crypto algorithms in-crate, and `Fnv256` is non-cryptographic.** By
  operator law this crate implements **zero cryptographic algorithms** — no
  hand-rolled hash or signature arithmetic. `Fnv256` is the in-crate
  reference hasher: deterministic, good diffusion for tests/demos, **not**
  collision-resistant, and retained as a non-cryptographic baseline only.
  For any adversarial setting use `Blake3Hasher` (the external `blake3`
  crate) — nothing else changes; for post-quantum verification use
  `ExternalMlDsa65` (the external `ml-dsa` crate) or inject your own
  `MlDsa65Verifier`.
- **Scope.** This is the structural chain core: events carry a `commitment`
  digest; it does not re-hash event *payloads* (that's the caller's `commitment`)
  nor parse JSON. It is the portable spec the full `affidavit` crate can build on.
- `event_id` uniqueness is checked in O(n²) — the right trade for an
  allocation-free verifier over typically small chains.

## Develop

```bash
cd affidavit-core
cargo test                                    # unit tests + doctests (incl. the compile_fail seal)
cargo test --test pqc_external_validation     # external-validation court (ml-dsa)
cargo test mining                             # mining + OCEL tests (alloc-gated)
cargo build --no-default-features             # prove the verify path needs no allocator
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps # doc build must be warning-free
cargo test --doc                              # doctests (README-adjacent examples)
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

## License

MIT OR Apache-2.0
