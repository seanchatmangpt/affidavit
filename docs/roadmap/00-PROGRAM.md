# affidavit 2030 Program — Master Plan

**Status:** program design / proposal · **Branch:** `claude/jolly-turing-t488iq` · **Date:** 2026-06-20
**Companion:** near-term feature designs in [`../innovation/00-SYNTHESIS.md`](../innovation/00-SYNTHESIS.md)

This is the master plan for taking `affidavit` from its current state to a 2030 end-state,
produced by a 10-agent fan-out. Each agent owned one **workstream**, grounded itself in the
real source, and plotted its area across **2026 H2 → 2027 → 2028 → 2029 → 2030**. This file
unifies them: the workstream index, a single release calendar, the cross-workstream
dependency graph, the two ledgers that grounding exposed, and a program-level definition of
done.

> **Doctrine guardrail (whole program):** *certify, don't decide.* Every workstream re-runs
> the existing verifier verbatim and **surfaces/suggests**; none judges honesty or mints a
> verdict. The sharpest violations of this today are real and cited in §5.

> **Verification caveat:** external deps are private-registry `26.6` crates that do not
> resolve in a lone checkout, so **nothing here was `cargo build`/`test`-verified.** All Rust
> in these docs is compilable-*style*. Execute each phase in an environment where the sibling
> crates resolve; start with the P0 correctness PR (§4a).

---

## 1. The ten workstreams

| # | Workstream | Mission (one line) | Upstream deps |
|---|-----------|--------------------|---------------|
| [W1](W1-foundations-correctness.md) | Foundations & Correctness | `diag.rs`+`output.rs` contract; close the bug ledger | — (keystone) |
| [W4](W4-onboarding-registry.md) | Onboarding & Registry | `registry.rs` single source of truth; `guide` noun; group 67 verbs | — (keystone) |
| [W2](W2-doctor-self-healing.md) | Doctor & Self-Healing | one `affi doctor` (env+receipt) + safe `affi fix` | W1, W4, W7 |
| [W3](W3-cli-ergonomics-contract.md) | CLI Ergonomics & Contract | `--explain`, `affi why`, uniform `--json`, versioned schemas | W1, W4 |
| [W5](W5-workflow-automation.md) | Workflow Automation | `init`/`watch`/`config`/hooks; verdict cache | W1, W2, W7 |
| [W6](W6-interactive-surfaces.md) | Interactive Surfaces | REPL parity, TUI dashboard, LSP/IDE | W1, W4, W2 |
| [W7](W7-verification-engine.md) | Verification Engine | multi-profile, streaming, parallel, GPU, distributed | W1 |
| [W8](W8-cryptography-trust.md) | Cryptography & Trust | real Ed25519→PQC, transparency log, signing | W7 |
| [W9](W9-ecosystem-standards.md) | Ecosystem & Standards | OCEL/OTel/SBOM interop; `SourceAdapter` framework | W1, W7 |
| [W10](W10-compliance-governance.md) | Compliance & Governance | evidence model, policy-as-code, audit packs | W7, W8, W9 |

---

## 2. Unified release calendar

CalVer continues (`YY.M.patch`). Headlines only — full detail lives in each Wn doc.

| Year | Theme | Headline deliverables (workstream) |
|------|-------|------------------------------------|
| **2026 H2** `v26.7–26.12` | **Foundations & "make it real"** | `diag.rs`+`output.rs` and the P0 bug PR (W1); `registry.rs` + verb taxonomy (W4); unified `affi doctor`+`fix` (W2); `--explain`/`why` (W3); `Profile` trait (W7); **real Ed25519 signing** (W8); `SourceAdapter` framework (W9); **doctrine string correction** (W10); `init`+layered `config` (W5); REPL parity + LSP loop (W6) |
| **2027** `v27.x` | **Close the loops** | `--json` across all 67 verbs + completions (W3); `affi watch` on the real `FileWatcher`, hook suite, verdict cache (W5); incremental/streaming verify (W7); **Merkle transparency log** (W8); OCEL round-trip + OTel wired into verify (W9); policy-as-code engine (W10); generated man/help/search (W4); TUI v1 (W6) |
| **2028** `v28.x` | **Harden + post-quantum** | versioned schemas + CI conformance (W1/W3); parallel verification (W7); **PQC lands — real ML-DSA, default-hybrid** (W8); exporters + SBOM breadth (W9); fleet doctor (W2); auditor-ready signed evidence packs + redaction-with-proof (W10) |
| **2029** `v29.x` | **Scale & distribution** | GPU verdict-exact (W7); output determinism + NDJSON (W1); `--output-version` pinning (W3); multi-repo daemon + remote cache (W5); key rotation/revocation, threshold signing (W8); streaming ingest + OCEL 2.0 (W9); continuous compliance (W10) |
| **2030** `v30.x` | **Platform / steady state** | **distributed Merkle-proof + partial-receipt (selective-disclosure) verification** (W7); freeze `affidavit-contract/1.0` (W1) + CLI LTS (W3); PQC-primary + HSM/keyless (W8); editor-native provenance (W6); interop manifest (W9); attestable compliance platform (W10); predictive doctor (W2); discoverability CI gate (W4) |

---

## 3. Cross-workstream dependency graph

Two keystones gate almost everything; build them first.

```
        ┌──────────────────────── KEYSTONES ────────────────────────┐
        │  W1 Foundations (diag/output contract, bug fixes)          │
        │  W4 Registry (one source of truth for the verb surface)    │
        └───────┬───────────────────────────────────┬───────────────┘
                │ output/errors/schemas              │ verb metadata
        ┌───────▼───────┐  ┌──────────────┐  ┌───────▼───────┐
        │ W3 Ergonomics │  │ W2 Doctor    │  │ W6 Interactive│
        │ (explain/why) │  │ /Fix         │  │ (REPL/TUI/LSP)│
        └───────────────┘  └───────┬──────┘  └───────────────┘
                                   │ verify verbatim
                           ┌───────▼───────────────────────────┐
                           │ W7 Verification Engine (core)      │
                           │ Profile · streaming · parallel ·   │
                           │ GPU · distributed Merkle proofs    │
                           └───┬───────────────┬───────────┬────┘
              signed receipts  │   profiles    │  proofs   │
                       ┌───────▼──────┐  ┌──────▼─────┐ ┌───▼──────────┐
                       │ W8 Crypto &  │  │ W9 Eco &   │ │ (W5 watch /  │
                       │ Trust        │─▶│ Standards  │ │  cache reuse │
                       └───────┬──────┘  └──────┬─────┘ │  verify)     │
              signed packs     │   evidence     │       └──────────────┘
                          ┌────▼────────────────▼────┐
                          │ W10 Compliance & Governance│
                          └────────────────────────────┘
```

**Critical path:** W1 + W4 → W7 → {W8, W9} → W10. W2/W3/W5/W6 ride the keystones and can
proceed in parallel once W1/W4 land. `linkme` (declared but unused, `Cargo.toml:42`) becomes
the shared plugin substrate for W2 checks, W7 profiles, W9 adapters, and W10 frameworks.

---

## 4. What grounding revealed

Ten agents reading the source independently converged on two ledgers. **These are the real
starting line** — the program's first job is to close them.

### 4a. Correctness bug ledger (close in the P0 PR)
Carried from the innovation synthesis and reconfirmed with fresh citations:

| ID | Defect | Where |
|----|--------|-------|
| B1 | Output stream split — `emit`→stdout, `verify`/`show`/`inspect`/`stats`→stderr (redirects capture nothing) | `handlers.rs:127` vs `:356-365,687,754` |
| B2 | JSON built by `format!` → invalid/injectable on quotes | `handlers.rs:157,166,184,203,223,264,304,325` |
| B3 | Tampered receipts silently dropped (`if let Ok(r)`) | `handlers.rs:84` |
| B4 | `GENESIS_SEED` pinned `v26.6.14` while pkg is `26.6.17` → cross-machine verify fail | `chain.rs:22` |
| B6 | Scattered `process::exit` (4 sites); `verify_sla`→generic `1` | `handlers.rs:352,367,562,2068,463` |
| B7 | Dead duplicate module file (hyphen can't be a Rust module) | `verbs/receipt-throughput.rs` |
| B9 | README "59 capabilities" vs 67 live verbs (68 `#[verb]` attrs) | `README.md:101`, `verbs/mod.rs` |
| B10 | `linkme` declared but unused in `src/` | `Cargo.toml:42` |

### 4b. Aspirational-debt ledger — the "1000x" surface that is scaffolding
The headline finding of the whole exercise: several flagship capabilities **exist in name but
not in function.** The program makes each real (or supersedes it). Until then, they should not
be presented as working.

| Area | Reality on disk | Where | Owner |
|------|-----------------|-------|-------|
| **Signing** | `sign` reads no key and signs nothing (emits a JSON note "Production: sign…"); `notarize` is a fake RFC-3161 token; `attest` builds a real in-toto statement but **unsigned** | `handlers.rs:641-673,611-638,568-609` | W8 |
| **Post-quantum** | `1000x_post_quantum_sealing.rs` uses **mock BLAKE3**; `pqc` feature is **empty**; module **not** `cfg`-gated → the flag toggles nothing | `…:157-184`, `Cargo.toml:167`, `lib.rs:120-121` | W8 |
| **GPU verify** | shader is a **1-round** BLAKE3 + placeholder format hash → not verdict-exact | `1000x_gpu_verifier.rs:115` | W7 |
| **Distributed** | `1000x_distributed_sharding.rs` **not wired into `lib.rs`** and imports a **private** symbol → won't compile as-is | `…sharding.rs:12` | W7 |
| **Observability** | `metrics.rs` + `1000x_otel_hyper_spec.rs` **not declared in `lib.rs`**; `verifier.rs` has **zero** trace/metric refs; metrics use wall-clock `SystemTime::now` (breaks determinism) | `metrics.rs:248`, `lib.rs` | W9 |
| **Watch** | `monitor` prints *"tokio-based watch loop not yet implemented"* though a real `FileWatcher` exists (used only for quality) | `handlers.rs:2632` vs `quality.rs:1147` | W5 |
| **Ingestion** | 7 near-identical `emit_from_*` handlers fabricate stub payloads instead of ingesting real evidence; `format!`-JSON | `handlers.rs:166-263` | W9 |
| **Policy** | `policy_enforce` evaluates only two hardcoded keys | `handlers.rs:2010-2038` | W10 |

---

## 5. Doctrine watch — the sharpest hazards

"Certify, don't decide" is violated in exactly two ways today, both cited and both scheduled
for correction in **2026 H2**:

1. **Compliance overclaim (W10).** `verify_compliance` prints `COMPLIANT` / `NON-COMPLIANT`
   (`handlers.rs:550-557`) and `soc2_audit` asserts a *"sufficient audit trail … certification"*
   (`handlers.rs:1814`). W10's first deliverable is a `doctrine_guard` output linter + an
   `EvidenceStatus` type with **no `Compliant` variant**: output becomes "evidence present/absent
   for control X," with the legal determination explicitly left to the auditor.
2. **Trust theater (W8).** Verbs named `sign`/`notarize` that don't actually sign imply an
   integrity/authorship guarantee they don't provide — worse than absent. W8 replaces the stubs
   with real Ed25519 wrapping a *finalized* receipt (never bypassing `ChainAssembler::finalize`;
   `_seal` stays private), so a signature certifies authorship+integrity — never "honest."

Everywhere else the doctrine is structurally safe: W2/W5 re-run `verifier::verify` verbatim and
cache only on byte-identical content address; W7 keeps every optimization bit-identical to the
sequential oracle; W9 ingestion translates formats into events to be certified, not verdicts.

---

## 6. Program definition of done — 2030

- **Contract:** `affidavit-contract/1.0` frozen — stable error/exit-code catalog, one JSON schema
  per verb, `--output-version` pinning, CI conformance gate (W1/W3).
- **Doctor:** `affi doctor` is the default first-run experience; env+receipt+fleet health with
  safe `fix`; predictive/advisory health (W2).
- **Engine:** multi-profile, incremental, parallel, GPU-accelerated, and **distributed with
  Merkle + partial-receipt proofs** — all verdict-identical to the 2026 sequential verifier (W7).
- **Trust:** real signing, append-only transparency log, **PQC-primary hybrid** by default (W8).
- **Interop:** lossless OCEL round-trip, live OTel/metrics over the real pipeline, broad SBOM
  ingest/export under a versioned interop manifest (W9).
- **Compliance:** auditor-ready signed evidence packs across frameworks, policy-as-code,
  governance — with doctrine-clean language guaranteed by a shipped conformance suite (W10).
- **Surfaces:** registry-driven help/search/man, a navigable TUI, and editor-native LSP
  provenance (W3/W4/W6).
- **Hygiene:** every item in §4a and §4b closed, with standing regression tests.

---

## 7. How to execute

1. **P0 correctness PR first** (§4a B1–B4) — small, high-value, and a prerequisite for W3's
   output contract. Do it where the `26.6` deps resolve so it actually compiles/tests.
2. **Stand up the two keystones** (W1 contract, W4 registry) — they unblock the widest fan-out.
3. **Then parallelize** along the §3 graph. Each Wn doc carries acceptance criteria per phase.
4. Treat every §4b item as **"make real or clearly mark prototype"** — do not ship the name
   without the function.

## 8. File index

`00-PROGRAM.md` (this file) · `W1-foundations-correctness.md` · `W2-doctor-self-healing.md` ·
`W3-cli-ergonomics-contract.md` · `W4-onboarding-registry.md` · `W5-workflow-automation.md` ·
`W6-interactive-surfaces.md` · `W7-verification-engine.md` · `W8-cryptography-trust.md` ·
`W9-ecosystem-standards.md` · `W10-compliance-governance.md`
