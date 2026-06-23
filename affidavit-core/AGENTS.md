# AGENTS.md — `affidavit-core`

This crate is the one place in the repo where you can write Rust, **compile it,
and test it**. It is held to a deliberately strict bar. If you change anything
here, preserve the invariants below or the build will reject you.

## Invariants you MUST preserve

1. **Zero dependencies.** `[dependencies]` is intentionally empty — that's the
   crate's whole point (a trust-critical verifier with no supply-chain surface).
   **Do not add a dependency.** If you think you need one, you almost certainly
   don't; reach for `core` / `alloc` instead, or reconsider the design.
2. **No `unsafe`.** `#![forbid(unsafe_code)]` is set crate-wide. Don't add `unsafe`.
3. **`no_std`.** The crate is `#![cfg_attr(not(test), no_std)]`. In non-test code
   use `core::…` and (behind the `alloc` feature) `alloc::…` — **never `std::`**.
   `std` is only available under `#[cfg(test)]`.
4. **The verify path stays allocation-free.** `verifier::verify` and
   `chain::compute_chain_hash` operate over borrowed slices and must not allocate.
   Anything needing `Vec`/`String`/`BTree*` goes behind the `alloc` feature
   (e.g. the owned `ChainBuilder`/`Receipt` and the whole `mining` module).
5. **Document every public item.** `#![warn(missing_docs)]` + `clippy -D warnings`
   means an undocumented `pub` item — **including every public struct field and
   enum variant** — fails the build. Document as you go.
6. **Determinism.** No wall-clock, no `HashMap` iteration-order dependence; the
   `mining` module uses `BTreeMap`/`BTreeSet` precisely so results are reproducible.

## The validate loop (run ALL before pushing)

```bash
cd affidavit-core
cargo test                                       # unit tests + doctests (incl. a compile_fail seal proof)
cargo clippy --all-targets -- -D warnings        # default features
cargo clippy --no-default-features -- -D warnings # the no-alloc verify path
cargo build --no-default-features                # proves verify works without an allocator
cargo fmt -- --check                             # formatting
```

`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` should also be clean.

## Module map

| File | Role |
|---|---|
| `src/digest.rs` | `Digest` (32-byte), the `ChainHasher` trait, and `Fnv256` (zero-dep **non-cryptographic** reference hasher — swap a BLAKE3 impl in for real tamper-evidence). |
| `src/chain.rs` | `Event` (borrowed, zero-alloc), `compute_chain_hash` (per-link rolling fold), and behind `alloc`: `OwnedEvent`, the sealed `Receipt`, and `ChainBuilder` (the only constructor — a private `_seal` field makes a forged `Receipt` an `E0451` compile error). |
| `src/verifier.rs` | `verify` — total, fail-fast, no-alloc; `Verdict` / `RejectReason`. Mirrors affidavit's pipeline stages (format → chain integrity → continuity → commitments → profile). |
| `src/mining/` | Process mining (`alloc`-gated): `Trace`, `DirectlyFollowsGraph` (discovery), `footprint` (α-relations), `conformance` (token replay), `stats` (log statistics). |

## How to extend (recipes)

- **Add a verifier stage:** add a `RejectReason` variant (documented), insert the
  check in `verify` in pipeline order (fail-fast), and add unit tests for both the
  pass and the new reject. Keep it allocation-free.
- **Add a real hasher:** implement `ChainHasher` for a new zero-size type (e.g. a
  `blake3` adapter). Don't add the dep to *this* crate — put adapters behind an
  optional feature, or in a separate crate, so the core stays dependency-free.
- **Add a mining primitive:** new file under `src/mining/`, `pub mod` it in
  `mining/mod.rs`, import only `crate::mining::{Trace, DirectlyFollowsGraph}` plus
  `core`/`alloc`, add inline `#[cfg(test)]` tests, and re-export the public types
  from `mining/mod.rs`.
- **Next big extension (unbuilt):** object-centric event logs (OCEL) — needs
  `Event` to carry object references and an object-centric DFG. Self-contained and
  on-theme (van der Aalst lineage) if asked.

## Don't

- …add dependencies, `unsafe`, or `std::` (non-test).
- …allocate on the verify path.
- …commit `target/` or `Cargo.lock` (both git-ignored; this is a library).
- …weaken `Fnv256`'s docstring claim — it is explicitly *non-cryptographic*.
