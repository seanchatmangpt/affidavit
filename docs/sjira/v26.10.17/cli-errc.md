# cli-errc — CLI vacuity audit + ERRC — v26.10.17 wave (2026-10-17)

| field | value |
|---|---|
| repo | `/Users/sac/affidavit` |
| branch | `v26.10.16` (HEAD `b0325d4` at receipt time) |
| lane ownership | `docs/sjira/v26.10.17/cli-errc.md` ONLY |
| standing | PARTIAL_ALIVE — all actions landed and receipted; current-tree re-run of the tripwire court BLOCKED[COMPILE-FREEZE] (see court-exit notes) |

Audit register: `docs/reference/CLI_VACUITY_AUDIT_26_10_17.md` (commit `abcea17`).
Method: every registered verb executed against the built `affi` binary across
7 lanes (receipt batches 1–6 + non-receipt batch 7), real invocations, real
fixtures. 100 verbs stood at wave close (101 at audit time; `receipt test`
deleted by the ERRC below). Verdict distribution: 62 EXECUTES / 17
REFUSED-TYPED / 9 THIN / 6 VACUOUS / 6 UNWIRED / 4 FEATURE-GATED
(CTDD-gated surfaces counted within the sealed classes; the batch-6
"unwired" reading of validate/chain-verify was corrected by the fresh-binary
re-check into the FEATURE-GATED class).

## Work-order rows

| # | action | evidence commit | court exit |
|---|---|---|---|
| R0 | **Audit executed** — 100 verbs (101 at audit time), 7 lanes; verdicts 62 EXECUTES / 17 REFUSED-TYPED / 9 THIN / 6 VACUOUS / 6 UNWIRED / 4 FEATURE-GATED | `abcea17` (docs/reference/CLI_VACUITY_AUDIT_26_10_17.md) | register-only; falsified per-verb in lanes; no gate at audit time |
| R1 | VACUOUS-1 `legal-hold status`: certifies FRCP 37(e) over ZERO chains, fabricates `events_preserved: 1` → now refuses `REFUSED_NO_CHAINS` on empty input | `127b2c0` | court: empty-input refusal cell flips (tamper-sensitive; see R14 cell 1) |
| R2 | VACUOUS-2 `receipt license-compliance`: never compared licenses vs `allowed_licenses` → real allowed/disallowed comparison; verdict flips on license content | `127b2c0` | court: allowed vs disallowed flips (R14 cell 2) |
| R3 | VACUOUS-3 `receipt audit`: stdout was literally `null` + canned stderr progress → real receipt-store scan and report (audit made real, not deleted) | `3aa41ea` | audit command executes against real store state |
| R4 | VACUOUS-4 `receipt test` DELETED from registry + ontology (2 sites); registry count 101 → 100; readme sites swept (`38a9ed4`, `2c3a6a0`) | `3aa41ea` | registry parity tests fail if wiring count regresses; readme count courts green at landing |
| R5 | VACUOUS-5 `certificate fre-902`: canned affidavit, zero world-state input → recomputes the supplied CTRT chain before certifying | in-tree, uncommitted at receipt time (`src/verbs/certificate_fre_902.rs`, `src/verbs/dossier_seal.rs`, `src/verbs/dossier_verify.rs` dirty) | BLOCKED[COMPILE-FREEZE] at receipt time — see freeze note |
| R6 | VACUOUS-6 `restitution calculate`: identical $1.25M/$625K for every tariff schedule → figures derived from a supplied schedule (`tariff_schedule: Option<String>`) | in-tree, uncommitted at receipt time (`src/handlers.rs:8942`, `src/verbs/restitution_calculate.rs` dirty) | BLOCKED[COMPILE-FREEZE] at receipt time — see freeze note |
| R7 | UNWIRED 6/6 `emit-from-*` verbs: payload-as-path defect → payload staging via `emit_with_payload`; UNWIRED class eliminated | `0dd25cb` | emit-from verbs execute with real staged payloads |
| R8 | B13 trailing-`null` output-layer defect on every success path (~40 verbs) — one fix in the output layer (upstream clap-noun-verb B13) | `05461c3` | output layer byte-clean; federation courts exempt per audit |
| R9 | Typed refusals on unknown input across 5 analytics verbs (predict, causality-chain, find-blast-radius, explain-incident, monitor --rules): silent-empty exit 0 → non-zero typed refusal | `a48a5a7` | non-zero exit witnessed per verb |
| R10 | Verdict-determined exits (format-independent; `verify-compliance` human=2/json=0 split closed) + honest SLA/attest help claims | `1b256eb` | verdict decides exit, not renderer |
| R11 | Feature-aware help prefixes — `[crypto-trust]`/`[ctdd]` annotations on the 17 REFUSED-TYPED / gated surfaces at `--help` time | `d7f4443` | help-time visibility; registry parity intact |
| R12 | Stable exit-code diag catalog 1500–1599 (CLI framework surface errors) + error-case courts aligned to the catalog | `8153451` (catalog), `8624548` (court alignment) | `src/diag.rs:72,118-122`; `CliCommandNotFound = 1500` witnessed |
| R13 | gdpr-proof + hipaa made falsifiable (last THIN pair closed) | `e9a89be` | proof output now bound to real input state |
| R14 | **Permanent court**: `tests/cli_vacuity_tripwire.rs` — 19 live cells + byte-identity detector (runs every registered verb with minimal fixtures; asserts NO verb returns success whose output is byte-identical across two different inputs — the mechanical VACUOUS-class detector, ERRC-CREATE) | `5b8589b` (+ serialization `63055dd`, SuiteGuard `580c4dd`) | 3/3 cells passing at landing (cells 1 legal-hold, 2 license-compliance, + detector); 16 cells ignored-loud (owned by other lanes); current-tree re-run BLOCKED[COMPILE-FREEZE] |

## Court-exit freeze note (as-of-receipt)

Current-tree rerun of `cargo test --test cli_vacuity_tripwire` aborts at
compile: 6 errors in the lib — 4× E0425 (`handlers::dossier_seal`,
`dossier_verify`, `receipt_chain_verify`, `receipt_validate` not found in
`crate::handlers`) and 2× E0061 (`handlers::restitution_calculate` now takes
`Option<String>`; wrappers in `src/verbs/restitution_calculate.rs:21` not yet
updated). These are in-flight edits by other lanes on files outside this
lane's ownership; per the compile-freeze SLA the owning lanes must fix or
revert. Receipt re-verifies once the boundary compiles.

## Falsifiers

- Any verb regressing to byte-identical output across two different inputs
  must fail the tripwire detector — if it passes, the court is vacuous.
- Registry count != 100 → parity test fails (R4).
- `legal-hold status` succeeding on empty input → R1 regression.
- `emit-from-*` succeeding without a staged payload file → R7 regression.
