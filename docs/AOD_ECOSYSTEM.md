# AOD ecosystem absorption — crypto / git authority fence

**Spec:** AFFI-SPEC-26.11.19 (Zero-String Sealed AOD)  
**Lane:** PR-8 / embeddable micro-verifier + ecosystem absorption  
**Status:** normative fence (affidavit-side). Sibling repos are surveyed
read-only; fixes land there only when trivial and in-scope.

---

## Law

**Affidavit remains the cryptographic and git-identity authority** for the
AOD / CTRT / receipt plane.

| Plane | Owner | Consumers must |
|-------|-------|----------------|
| PQC (ML-DSA-65, SLH-DSA, ML-KEM), BLAKE3 chain / AOD seal | `affidavit` / `affidavit-core` | call `affi`, `affidavit-wasm`, or `affidavit-micro-verifier` |
| Historical Witness (Mode B) | `affidavit_core::aod::verify_witness` | use micro-verifier C ABI / documented NIF / CLI `affi aod witness` |
| Live Gate (Mode A) | `affidavit_core::aod::map_live` | stay in affidavit control plane — **not** exported to ecosystem NIFs |
| Protocol / Ash / product UX | `ash_a2a`, `fnox`, `strata` | certify via affidavit; never reimplement algorithms |

Doctrine preserved: **certify, don't decide.** Ecosystem crates may *display*
or *route* attestations; they do not mint a second trust plane.

Companion: [`CRYPTO_TRUST_PLANE.md`](CRYPTO_TRUST_PLANE.md),
[`WASM.md`](WASM.md), [`micro_verifier/README.md`](../micro_verifier/README.md).

---

## Embeddable micro-verifier

Scaffold path: **`micro_verifier/`** (crate `affidavit-micro-verifier`).

| Surface | Path | Notes |
|---------|------|-------|
| C ABI | `micro_verifier/include/affidavit_aod.h` | `af_aod_abi_version`, `af_aod_witness_verify` |
| Rust safe API | `micro_verifier/src/lib.rs` | `witness_verify` |
| FFI | `micro_verifier/src/ffi.rs` | only `unsafe` |
| BEAM NIF stub | `micro_verifier/nif/` | documented; not on the default Cargo path |

Wire format of `.aod` is **unchanged** by this PR. The ABI only witnesses
existing envelopes.

---

## Forbidden dependency growth (ecosystem)

Sibling Erlang/Elixir (and their native helpers) **must not grow**:

| Class | Examples | Typed stop |
|-------|----------|------------|
| Crypto algorithm crates | `blake3`, `ml-dsa`, `slh-dsa`, `ml-kem`, `ring`, `openssl`, `libsodium`, hand-rolled PQC | `REFUSED_ECOSYSTEM_CRYPTO_DEP` |
| Git / object-store identity crates used as trust plane | `git2`, `libgit2`, reinvented CAS signing | `REFUSED_ECOSYSTEM_GIT_TRUST` |
| Second receipt verifier | local reimplementation of AOD envelope / CTRT seal | `REFUSED_DUPLICATE_VERIFIER` |

**Allowed** (not authority):

* OTP `:crypto` for TLS / classical digest helpers that do **not** claim AOD/CTRT standing.
* Consuming `ash_affidavit`, Wasmex + pinned `affidavit-wasm`, or this micro-verifier.
* Build-only `publish = false` helper binaries that hash **artifact identity** of a vendored wasm (e.g. SHA-256 of bytes) without claiming seal authority.
* Documented deferred Ash tiers that explicitly refuse to sign.

---

## Survey receipt (2026-11-21, public GitHub read)

Exact subjects surveyed via GitHub contents API on default branches. Standing
uses `UNKNOWN | PARTIAL_ALIVE | ALIVE | BLOCKED | BUILD_BROKEN | UNSUPPORTED`
plus typed `REFUSED_*`. This survey is **read-only** — no pushes to siblings.

### `seanchatmangpt/ash_a2a` (Elixir, public)

| Observation | Standing / fence |
|-------------|------------------|
| Depends on `{:ash_affidavit, "~> 26.10"}` | **ALIVE** consumer path — keep |
| `sa2a_crypto` Mix nested project: deps `jcs`, `jason` only; describes itself as “Affidavit-shaped … certify, don't decide” | **PARTIAL_ALIVE** — OK substrate; must not grow PQC crates |
| `native/graphlaw_host`: `wasmtime` + `sha2` for **wasm file identity** only; docs forbid GraphLaw/crypto logic in-crate | **ALIVE** fence — do not add `blake3`/`ml-dsa` |
| `native/hddl_cli`: **git** deps on `ferroplan` (`git = "https://github.com/seanchatmangpt/ferroplan.git"`) | **REFUSED_ECOSYSTEM_GIT_TRUST** (recommendation): pin crates.io / path vendor for manufacturing helpers, or isolate under explicit `build-only` + never use git rev as runtime trust |
| `wasmex` for graphlaw wasm | Correct absorption pattern — mirror for AOD via Wasmex or micro-verifier NIF |
| OTP `extra_applications: [:crypto]` | Allowed classical; not AOD authority |

**Recommendation:** mount Historical Witness through `affidavit-micro-verifier`
or Wasmex; do **not** extend `sa2a_crypto` with ML-DSA/BLAKE3.

### `seanchatmangpt/fnox` (private monorepo)

| Observation | Standing / fence |
|-------------|------------------|
| Root is polyglot (Python/Rust/…); no root `mix.exs` | Survey limited to visible crates |
| `crates/fnox-doctor` Cargo.toml includes `blake3 = "1.8"` | **REFUSED_ECOSYSTEM_CRYPTO_DEP** — doctor CLIs must call `affi` / core, not grow hash authority |
| `ecosystem/capability-owner.toml` present | Align owner id with affidavit provenance role |
| No `git2` hit in indexed search | No git-trust finding this pass |

**Recommendation:** replace in-crate `blake3` with affidavit CLI/library calls;
record a permanent refusal test if the dep reappears.

### `seanchatmangpt/strata` (Rust + Elixir `stratus`, public)

| Observation | Standing / fence |
|-------------|------------------|
| `strata_protocol` depends on `blake3`, `ml-dsa`, `sha1`, `zeroize` | **REFUSED_ECOSYSTEM_CRYPTO_DEP** / **REFUSED_DUPLICATE_VERIFIER** — parallel PQC/hash plane |
| Docs (`KNOWN_GAPS.md` KG-ARD21): valve does **not** sign; signatures Rust-side | Directionally correct split — **absorb** Rust-side verify/sign into affidavit rather than widening strata |
| Elixir `stratus` Ash tier DEFERRED (KG-FR4) | Do not add crypto while deferred |
| No `git2` in indexed search; CAS uses git-pack *format* as storage | Storage encoding ≠ trust authority; keep seal/verify in affidavit |

**Recommendation:** long-term absorption — strata keeps wire/CAS/PDU; affidavit
owns ML-DSA detach verify and AOD/CTRT witness. Short-term: freeze new crypto
deps; document the duplicate plane as a typed debt.

---

## CI checklist (affidavit-side)

Run from repo root (also scripted: `scripts/check_aod_ecosystem_fence.sh`):

1. `cd micro_verifier && cargo test && cargo clippy --all-targets -- -D warnings`
2. Confirm `micro_verifier/Cargo.toml` has **no** crypto/git dependencies other
   than the `affidavit-core` path pin.
3. Confirm `include/affidavit_aod.h` `AF_AOD_ABI_VERSION` equals
   `micro_verifier::ABI_VERSION`.
4. On sibling PRs (manual / scheduled): refuse merge if `mix.exs` /
   `Cargo.toml` gains `blake3`, `ml-dsa`, `git2`, or equivalent without an
   affidavit absorption exception filed in this document.

---

## Falsifiers

* Ecosystem crate ships a second ML-DSA verify path that accepts what
  `affi aod witness` rejects (or vice versa) → fence broken.
* Micro-verifier exports Live Gate / Region 3 pointers → Mode B isolation broken.
* `.aod` magic / shadow layout changed by this scaffold → wire-format regression
  (out of scope; refuse).

---

## See also

* [`micro_verifier/`](../micro_verifier/) — scaffold
* [`tutorials/02_wasm_beam_mounting.md`](tutorials/02_wasm_beam_mounting.md) — BEAM/Wasmex
* [`reference/verbs/aod/witness.md`](reference/verbs/aod/witness.md) — CLI verb
* [`FEDERATION.md`](FEDERATION.md) — cross-repo evidence courts
