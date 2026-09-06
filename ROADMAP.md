# affidavit — Project Roadmap

**Version:** 26.9.6 → beyond
**Date:** 2026-09-06
**Status:** living document — re-verified against the code each release

---

## Current State

affidavit ships **79 canonical CLI verbs** across 11 groups backed by a
compile-time static registry (`src/registry.rs`). The 7-stage certify pipeline is
production-ready. The BLAKE3 chain, sealed receipts, and determinism guarantees
are stable. As of v26.9.6 the evidence federation kernel has an operator surface.

### What works
- Full emit → assemble → verify lifecycle
- 7-stage verifier: decode, check_format, chain_integrity, continuity,
  verify_commitments, evaluate_profile, emit_verdict
- Sealed receipts (private `_seal` field, E0451 unconstructable bypass)
- Compile-time genesis seed (`concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`)
  and `affi --version` reporting that same version
- Stable exit-code catalog (`src/diag.rs`)
- Unified output handle (`src/output.rs`) plus the crate-internal `outln!`/`out!`
  macros keeping `#![deny(clippy::print_stdout)]` honest
- `affi doctor` (with `--fix`) over a `linkme`-discovered `DoctorCheck` registry
- **Federation courts** — `affi standing`, `affi ecosystem`, `affi errc` over the
  `standing` / `ecosystem` / `errc` / `errc_claim_assurance` certifiers
  (see `docs/FEDERATION.md`)
- Registry ↔ ontology ↔ projection parity, enforced by tests
- OCEL, SBOM, conformance, quality-monitor features behind feature flags

### Build state
The root crate builds, tests, and lints clean. The real published
`wasm4pm-compat 26.8.7` is the admitted structural dependency; `wasm4pm` and
`clnrm-core` remain fenced by local stubs via `[patch.crates-io]`. A stub is a
named capability boundary, not proof upstream behaviour executed — do not
silently broaden one. Run the full AGENTS.md §6 ladder with `just validate`.

---

## Bug Ledger

Re-verified against the code at 26.9.6. Several items previously listed as open
had already shipped; the statuses below carry the evidence.

| # | Sev | Status | Defect | Evidence |
|---|-----|--------|--------|----------|
| B1 | High | **FIXED** | Output stream split (stdout vs stderr) | `src/output.rs`, `src/macros.rs` |
| B2 | High | **FIXED** | Hand-built JSON via `format!` (injection risk) | `handlers.rs` uses `serde_json::json!` |
| B3 | High | **FIXED** | `load_receipts_from_path` swallowed load failures | `handlers.rs:429` emits a REJECT entry per failure |
| B4 | High | **FIXED** | Genesis seed version drift | `chain.rs:25`; witnessed by `tests/release_identity.rs` |
| B5 | Med  | **FIXED** | `monitor` was a stub; `FileWatcher` not wired | `handlers.rs:3052` behind the `file-watch` feature |
| B6 | Med  | **FIXED** | REJECT used a generic exit code | `handlers.rs` uses `exit_codes::REJECT` (2) |
| B7 | Low  | **FIXED** | Duplicate `receipt-throughput.rs` | removed from `src/verbs/` |
| B8 | Low  | **Partial** | Completions covered ~4 of 69 verbs | v26.9.6 covers all 79 verbs and all 6 nouns in bash/zsh/fish; **no PowerShell**, and they are still hand-maintained (see P1-5) |
| B9 | Low  | **FIXED** | README verb count wrong | witnessed by `tests/release_identity.rs` |
| B10 | Info | **FIXED** | `linkme` declared but unused | `src/doctor_check.rs` distributed slice, consumed at `handlers.rs:4456` |
| B11 | Med | **FIXED** | `affi --version` reported the clap-noun-verb version (`cli 26.6.2`) | `src/bin/affi.rs` answers the bare top-level flag |
| B12 | Med | **FIXED** | `why`, `fix`, `install-git-hook`, `monitor` shipped undeclared in the ontology | declared in `ontology/affi-cli.ttl`; blocked by `registry.rs` parity test |
| B13 | Med | **Partial** | `clap-noun-verb` appends its rendering of each verb's return value to stdout, so `--format json` output is not parseable | fixed for the 8 federation verbs (they exit with their own code before the runtime renders); the other 71 verbs are still affected — see P1-6 |

| B14 | High | **FIXED** | `verify` could not reach stage 3: a tampered receipt exited 1 with a parse error, not the documented REJECT (2) | `chain.rs::deserialize_receipt_unchecked`; `tests/e2e.rs`, `tests/golden_run.rs` |
| B15 | Med | **FIXED** | `verify-family` exited 0 while reporting REJECTs — CI stayed green over a tampered store | `handlers.rs::family_exit` |
| B16 | Med | **FIXED** | 41 REGISTRY rows used snake_case tokens the CLI cannot dispatch | `no_registry_verb_token_uses_snake_case` |
| B17 | Med | **FIXED** | `receipt-throughput` had a registry row, a projection and a handler but no `pub mod` line, so it was never compiled | `src/verbs/mod.rs`; parity test now walks the module list |
| B18 | Med | **FIXED** | `affi doctor` and `guide search` were dispatchable but absent from REGISTRY and the ontology | both registered and declared; parity tests compare `(verb, noun)` pairs |
| B19 | High | **FIXED** | `web/` genesis seed pinned to `affidavit-v26.6.17-genesis` — the browser verifier rejected every real receipt for three releases | `tests/release_identity.rs` |
| B20 | Med | **FIXED** | `examples/golden_run.sh` exited 101 and no test or workflow ran it | build-from-root fix; `tests/golden_run.rs` |

| B21 | High | **FIXED** | Module docs claimed quantum-resistant unforgeability over `mock_*` functions that compute unkeyed BLAKE3, and in-shader BLAKE3 over a shader doing one mix round against a `// Placeholder` constant | `src/1000x_post_quantum_sealing.rs`, `src/1000x_gpu_verifier.rs` module headers rewritten to state what the code does |
| B22 | Med | **FIXED** | `wasm-encoder` was a mandatory dependency linked into every build and published, with no compiled consumer | removed from `[dependencies]` |
| B23 | Med | **FIXED** | The `remediation` feature omitted `tracing`, which its own module imports, so it could never build | `Cargo.toml` feature list |
| B24 | Med | **FIXED** | README told users to run `cargo build --release --all-features`, which fails, and claimed "65+ canonical verbs" three lines from its own "79" | README install block + feature-status table |
| B25 | Med | **Open** | Seven targets with `required-features` are silently skipped by CI, and `--features gpu`/`remediation` fail clippy | see P1-8 |

| B26 | High | **FIXED** | Shell completions offered 39 snake_case verb names the binary rejects — a completion that types a refused command | all three files regenerated from REGISTRY; `shell_completions_offer_only_dispatchable_verbs` |
| B27 | High | **FIXED** | The three checks that make the kernel unforgeable (`CoverageMismatch`, `StandingMismatch`, `ClaimSetMismatch`) had zero tests | 5 anti-forgery tests in `src/ecosystem.rs` and `src/errc_claim_assurance.rs` |

---

## Shipped in 26.9.6

| Item | What landed |
|------|-------------|
| **Federation CLI surface** | `src/federation.rs` + 8 verbs across 3 new nouns; the kernel is reachable |
| **Ontology parity** | `every_registry_verb_is_declared_in_the_ontology`, `every_registry_entry_has_a_verb_projection` |
| **Release identity** | version bump to 26.9.6, `affi --version` fixed, `tests/release_identity.rs` |
| **`just validate`** | the AGENTS.md §6 ladder as one recipe |
| **Completions** | all 79 verbs, all 6 nouns, bash/zsh/fish |
| **`docs/FEDERATION.md`** | operator guide, exit-code contract, executed worked example |
| **Stage 3 reachable** | `chain::deserialize_receipt_unchecked` forensic seam; `verify`/`why`/`fix` all repaired |
| **(verb, noun) parity, both directions** | caught `receipt-throughput` uncompiled, `affi doctor`/`guide search` unregistered, 41 snake_case tokens, 4 wrong-noun declarations |
| **Web seed guard** | `tests/release_identity.rs` pins the TypeScript verifier's genesis seed to the crate version |
| **Golden example runs** | `examples/golden_run.sh` fixed; `tests/golden_run.rs` executes it |
| **Completions dispatchable** | kebab-case throughout, held to REGISTRY by a test |
| **Kernel unforgeability tested** | derived-field forgeries with matching hashes are refused by name |

---

## P1 — High-Leverage, Still Open

### [P1-5] Generate shell completions from the registry
**Status:** Open
**What:** `completions/affi.{bash,zsh,fish}` are hand-maintained. v26.9.6 brought
them back into sync, but nothing prevents the next verb from desynchronising
them again — the registry parity tests cover the ontology and the projection,
not the completions. Generate them from `REGISTRY` instead, and add PowerShell
(closes B8 fully).
**New file:** `src/bin/gen_completions.rs` or a build script
**Done when:** adding a `VerbEntry` and regenerating is the only step, and a test
fails if the checked-in completions differ from the generated ones.

### [P1-7] Make `--all-features` build, or stop offering it
**Status:** Open
**What:** `cargo build --all-features` — which README told users to run until
v26.9.6 — fails. `discovery`/`conformance`/`predictive` need `wasm4pm` APIs
(`ilp_discovery`, `process_tree`, `models::EventLog`) that `stubs/wasm4pm` does
not expose; `mutation` needs `clnrm-core` APIs (`determinism::rng`) the stub
does not expose. CI never noticed because `rust.yml` builds default features
only. v26.9.6 fixed the `remediation` feature (a missing `tracing` dep — it now
builds) and replaced the README command with one that works plus a
feature-status table verified row by row with `cargo check --lib --features
<name>`. Four features remain broken: `discovery`, `conformance`, `predictive`,
`mutation`.
**The constraint:** AGENTS.md §1 forbids broadening a stub without an observed
integration proof, so this is *not* "add the missing functions to the stub". It
is either a real upstream integration or an explicit removal of the features.
**Done when:** every feature in the `all` list either builds under a CI job or
is deleted from `Cargo.toml`, and a workflow step exercises the combination.

### [P1-8] Feature combinations are unguarded by CI
**Status:** Open
**What:** `rust.yml` runs `cargo build/test --all-targets` with default features
only, so seven targets carrying `required-features` (the `quality-monitor` and
`discovery` benches, the `lsp` and `discovery` examples) are silently skipped,
and `--features gpu` / `--features remediation` fail
`clippy -- -D warnings` (a `manual_div_ceil` in
`src/1000x_gpu_verifier.rs:366` among others) without anyone learning.
**Done when:** CI builds and lints at least `lsp,shell,quality-monitor,gpu,pqc`
in addition to default, and the currently-broken features are excluded by name
rather than by accident.

### [P1-6] Return-value rendering contract (B13)
**Status:** Open for 71 of 79 verbs
**What:** `clap-noun-verb` renders each verb's return value to stdout after the
handler returns, appending a bare `null`, so `affi receipt verify --format json |
jq` fails. The 8 federation verbs fix this by exiting with their own code before
the runtime renders (`emit_court` in `src/handlers.rs`); the same seam has not
been applied to the rest. The general fix is either that seam applied uniformly,
a `--quiet`-by-default data path, or an upstream change.
**Done when:** `affi <any verb> --format json` produces a single JSON document on
stdout, witnessed by a test per verb group.

---

## P2 — Depth & Polish

### [P2-2] Content-addressed verdict cache
**Status:** Open
**What:** Cache `(receipt_content_address, verifier_version) → Verdict`. Skip
re-verification when neither the receipt nor the binary has changed. Critical for
large receipt stores. The genesis-seed/version binding makes the cache key
naturally correct across releases.

### [P2-4] REPL upgrade: registry-driven dispatch
**Status:** Open
**What:** `src/bin/affi-shell.rs` hand-maintains a 15-arm dispatch against a
79-verb registry, and does not reference `registry.rs` at all. Drive it from the
registry, add tab completion, and add a `Session` that tracks the active
working-chain.

### [P2-5] Generated man pages
**Status:** Open
**What:** Auto-generate `man/affi-<noun>-<verb>.1` from registry entries.
**Dependency:** P1-5 (same generation seam)

### [P2-8] Resolve the 17 never-compiled sources under `src/`
**Status:** Open
**What:** 164 KB across 17 files (13 `1000x_*.rs` drafts plus `generation.rs`,
`metrics.rs`, `mining.rs`, `mutation.rs`) sit in `src/` declared by no `mod` and
mapped by no `#[path]`, so the compiler never sees them — never type-checked,
never linted, never tested. v26.9.6 named them in Cargo.toml's `exclude` so the
published crate stops shipping them, and
`orphaned_sources_are_declared_or_excluded` in `tests/release_identity.rs`
prevents the list growing silently. That bounds the problem; it does not solve
it.
Nine of the seventeen no longer build, and two
(`1000x_semantic_isomorphism_e2e.rs`, `1000x_time_travel_dx.rs`) do not parse —
proof they have not been compiled since they were written. Removing them also
retired `wasm-encoder`, a **mandatory** dependency compiled into every build
whose only consumer was one of these orphans.
**Done when:** each file is either wired behind a feature gate (as
`1000x_gpu_verifier.rs`, `1000x_auto_remediate_dx.rs`, and
`1000x_post_quantum_sealing.rs` already are) with tests that compile it, or
deleted. Deciding per file needs the author, not an agent.

### [P2-7] Regroup benchmark and governance verbs under their own nouns
**Status:** Open
**What:** `ontology/affi-cli.ttl` declares `bench` and `governance` nouns, but
`receipt-throughput`, `variance`, `profile`, and `audit` all ship under the
`receipt` noun. v26.9.6 marked those two nouns RESERVED and pointed the four
verbs at `ReceiptNoun` so the ontology describes the binary that exists rather
than one that does not. Moving them is a breaking CLI change and needs its own
release: projection, handlers, registry, completions, and a deprecation period
for `affi receipt <verb>`.
**Done when:** `affi bench variance` and `affi governance audit` work, the
ontology's RESERVED comments are removed, and the old spellings warn.

### [P2-6] Federation receipt DAG
**Status:** Open
**What:** Every federation profile already carries `previous_receipt` for
receipt-DAG lineage, but nothing walks the chain. Add
`affi standing lineage <RECEIPT>` / `affi ecosystem lineage` to resolve and
verify a whole ancestry, not one link.
**Dependency:** v26.9.6 federation courts

---

## Workstream Map

```
P1-5 (generated completions) → P2-5 (man pages)
P1-6 (stdout contract)       → clean machine consumption for all 79 verbs
P2-4 (REPL from registry)    → one dispatch surface instead of two
P2-6 (federation DAG)        → multi-release standing histories
```

---

## Release Milestones

| Release | Theme | Items |
|---------|-------|-------|
| 26.6.22 | Correctness sprint | B1–B7, output routing, clippy gate |
| 26.9.1  | Federation kernel (library) | standing, ecosystem, errc, claim assurance |
| **26.9.6** | **Federation surface + release identity** | **the kernel becomes reachable; B11, B12** |
| next    | Generation sprint | P1-5, P2-5 |
| later   | Contract sprint | P1-6, P2-2 |
| later   | Lineage sprint | P2-4, P2-6 |

---

## ADR Status (all in force)

| ADR | Decision | Status |
|-----|----------|--------|
| ADR-1 | BLAKE3 for content addressing | ✓ stable |
| ADR-2 | Private `_seal` field (E0451 unconstructable bypass) | ✓ stable |
| ADR-3 | `ChainAssembler::finalize` as only canonical seam | ✓ stable |
| ADR-4 | No wall-clock in events (`seq` ordering only) | ✓ stable |
| ADR-5 | Canonical/sorted JSON for deterministic hashing | ✓ stable |
| ADR-6 | Date-pinned nightly toolchain | ✓ stable (`rust-toolchain.toml`) |
| ADR-7 | `#![deny(clippy::print_stdout)]` at library root | ✓ stable |
| ADR-8 | The CLI reaches kernel laws; it never adds one | ✓ stable (`src/federation.rs`) |

---

*See `AGENTS.md` for execution doctrine, `docs/FEDERATION.md` for the federation
courts, and `docs/roadmap/` for the 10-workstream 2030 program plan.*
