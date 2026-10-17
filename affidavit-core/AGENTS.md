# AGENTS.md — `affidavit-core`

This crate is the one place in the repo where you can write Rust, **compile it,
and test it**. It is held to a deliberately strict bar. If you change anything
here, preserve the invariants below or the build will reject you.

## Invariants you MUST preserve

1. **No cryptographic algorithm is implemented in this crate.** All algorithms
   are external dependencies: **ML-DSA-65** via the RustCrypto `ml-dsa` crate
   (FIPS 204) and **BLAKE3** via the `blake3` crate — both unconditional
   `[dependencies]` with `default-features = false`, preserving `no_std`.
   Do not add another crypto dependency, and do not hand-roll arithmetic:
   if you think you need a new algorithm, the answer is the existing surface.
   In-crate hashers are limited to `Fnv256`, which is explicitly
   **non-cryptographic**.
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
7. **The algorithm surface is ML-DSA-65 + BLAKE3, unconditionally.** Signatures:
   FIPS 204 ML-DSA-65 only, via `pqc::ExternalMlDsa65` over the external
   `ml-dsa` crate — it ships with no feature flag so `PqcCourt` reaches the
   `affidavit-wasm` artifact. Hashing: BLAKE3 via `digest::Blake3Hasher` over
   the external `blake3` crate. No SLH-DSA, no SHA-256, no classical signature
   algorithms, no crypto feature flags — the only features are
   `default`/`alloc`.

## The validate loop (run ALL before pushing)

```bash
cd affidavit-core
cargo test                                       # unit tests + doctests (incl. a compile_fail seal proof)
cargo test --test pqc_external_validation        # external-validation court (ml-dsa)
cargo clippy --all-targets -- -D warnings        # default features
cargo clippy --no-default-features -- -D warnings # the no-alloc verify path
cargo build --no-default-features                # proves verify works without an allocator
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps   # doc build must be warning-free
cargo test --doc                                 # doctests
cargo fmt -- --check                             # formatting
```

## Module map

| File | Role |
|---|---|
| `src/digest.rs` | `Digest` (32-byte), the `ChainHasher` trait, `Fnv256` (**non-cryptographic** reference hasher — the only in-crate hasher, retained as a non-crypto baseline), and `Blake3Hasher` (delegates to the external `blake3` crate for tamper-evidence). |
| `src/pqc.rs` | The post-quantum **court**: `MlDsa65Verifier` trait, `ExternalMlDsa65` (the unconditional adapter over the external `ml-dsa` crate — ships with no feature flag, so it reaches `affidavit-wasm`), `PqcProfile`, and `PqcCourt` — the fail-fast court returning typed `PqcVerdict`/`PqcRefusal`. External validation: `tests/pqc_external_validation.rs`. |
| `src/chain.rs` | `Event` (borrowed, zero-alloc), `compute_chain_hash` (per-link rolling fold), and behind `alloc`: `OwnedEvent`, the sealed `Receipt`, and `ChainBuilder` (the only constructor — a private `_seal` field makes a forged `Receipt` an `E0451` compile error). |
| `src/verifier.rs` | `verify` — total, fail-fast, no-alloc; `Verdict` / `RejectReason`. Mirrors affidavit's pipeline stages (format → chain integrity → continuity → commitments → profile). |
| `src/mining/` | Process mining (`alloc`-gated): `Trace`, `DirectlyFollowsGraph` (discovery), `footprint` (α-relations), `conformance` (token replay), `stats` (log statistics), and `ocel` (object-centric extension: `OcelEvent`, per-object-type trace projection, per-type DFG discovery and conformance). |

## How to extend (recipes)

- **Add a verifier stage:** add a `RejectReason` variant (documented), insert the
  check in `verify` in pipeline order (fail-fast), and add unit tests for both the
  pass and the new reject. Keep it allocation-free.
- **Add a real hasher:** implement `ChainHasher` for a new zero-size type that
  delegates to an external crate, as `Blake3Hasher` does. Don't add a new
  *algorithm* to the lawful surface (invariant 7: ML-DSA-65 + BLAKE3 only,
  always external, never hand-rolled in `src/`).
- **Add a mining primitive:** new file under `src/mining/`, `pub mod` it in
  `mining/mod.rs`, import only `crate::mining::{Trace, DirectlyFollowsGraph}` plus
  `core`/`alloc`, add inline `#[cfg(test)]` tests, and re-export the public types
  from `mining/mod.rs`. The OCEL extension (`mining/ocel`) follows this pattern.

## Don't

- …add crypto dependencies beyond the lawful surface (`ml-dsa`, `blake3`),
  `unsafe`, or `std::` (non-test).
- …allocate on the verify path.
- …commit `target/` or `Cargo.lock` (both git-ignored; this is a library).
- …weaken `Fnv256`'s docstring claim — it is explicitly *non-cryptographic*.
- …implement a cryptographic algorithm in `src/` (invariant 1) or widen the
  algorithm surface beyond ML-DSA-65 + BLAKE3 (invariant 7).
