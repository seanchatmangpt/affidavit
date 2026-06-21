# affidavit-core

A **zero-dependency**, `#![no_std]`, `#![forbid(unsafe_code)]` reference verifier
for affidavit-style receipt chains.

> **On "1000x".** This repo brands features as "1000x". That's marketing, not a
> measured number, and this crate doesn't pretend otherwise. What it *does* is
> take the one move a Rust engineer would actually defend as a step-change for a
> trust-critical verifier: **make the verifier depend on nothing, run anywhere,
> and make invalid receipts unrepresentable.** The honest punchline is in your
> build log — the main `affidavit` crate currently *cannot compile* because one
> published dependency (`wasm4pm-compat 26.6.13`) is broken. A verifier with an
> empty `[dependencies]` cannot fail that way, and this one builds and tests
> green right next to it.

## Why this shape

| Property | What it buys you |
|---|---|
| **Zero dependencies** | No supply-chain surface. Nothing upstream can break or backdoor the thing you trust to certify provenance. |
| **`no_std`, no `alloc` on the verify path** | [`verify`] runs over a borrowed `&[Event]` and allocates nothing — it runs in a WASM sandbox, an embedded HSM, or an on-chain runtime, not just on a server. |
| **`forbid(unsafe_code)`** | Memory safety is enforced crate-wide, by the compiler. |
| **Compile-time sealing** | A `Receipt` is only obtainable from `ChainBuilder::finalize`. A private witness field makes a hand-forged `Receipt` an `E0451` compile error — proven by a `compile_fail` doctest. |
| **Pluggable hash** | The chain folds through any `ChainHasher`. A zero-dep reference hasher (`Fnv256`) ships in-crate; drop in BLAKE3 for a tamper-*evident* chain. |

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

## Features

- `alloc` (default) — enables the owned `ChainBuilder` / `Receipt`. Turn it off
  (`default-features = false`) for the pure, allocation-free verify path.

## Caveats (honest ones)

- **`Fnv256` is non-cryptographic.** It's deterministic and diffuses well enough
  to detect tampering in tests/demos, but it is **not** collision-resistant.
  Implement `ChainHasher` over BLAKE3 (or another cryptographic hash) for any
  adversarial setting; nothing else changes.
- **Scope.** This is the structural chain core: events carry a `commitment`
  digest; it does not re-hash event *payloads* (that's the caller's `commitment`)
  nor parse JSON. It is the portable spec the full `affidavit` crate can build on.
- `event_id` uniqueness is checked in O(n²) — the right trade for an
  allocation-free verifier over typically small chains.

## Develop

```bash
cd affidavit-core
cargo test                       # 12 unit tests + 2 doctests (incl. the compile_fail seal)
cargo build --no-default-features # prove the verify path needs no allocator
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

## License

MIT OR Apache-2.0
