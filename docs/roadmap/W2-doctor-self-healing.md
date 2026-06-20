# W2 — Doctor & Self-Healing (2026 H2 → 2030)

> **Workstream:** W2 of the affidavit multi-year program · **Owner:** Doctor & Self-Healing
> **Status:** roadmap / design · **Date:** 2026-06-20 · **Doctrine:** *certify, don't decide.*
> **Build caveat:** private-registry `26.6` deps (`clap-noun-verb`, `clnrm-core`, `wasm4pm`,
> `lsp-max`) do not resolve in a lone checkout, so nothing here was `cargo build`/`test`-verified.
> All Rust below is **compilable-style** — pinned to real in-tree symbols, signatures to be
> finalized against the sibling crates during implementation.

This document is the implementation program for `affi doctor` and the separate `affi fix` verb:
the system that turns invisible environment failures and silent receipt rot into one ranked,
copy-pasteable, optionally self-healing checklist — and does so **without ever laundering a
tampered chain into ACCEPT**.

---

## 1. Mission & Scope

### Mission

Make "is my affidavit setup and store healthy?" a **two-second question** instead of a source-diving
expedition, and make the *safe* repairs one command — while the integrity verdict stays the exclusive
property of the existing 7-stage verifier (`src/verifier.rs:43`). Doctor **surfaces and suggests**;
`fix` **re-seals already-honest bytes or quarantines dishonest ones**; neither mints a verdict.

### In scope (W2 owns)

- **`affi doctor`** — ONE command (reconciled per synthesis Part C, `00-SYNTHESIS.md:88-94`):
  - **environment mode (default):** install / toolchain / features / genesis / working-dir / config /
    completions checks (`01-doctor-command.md`).
  - **receipt-store mode (when a path / `--receipts` is given, or run inside a `.affi/` store):**
    store-wide chain-health scan, deterministic 0–100 score, triaged findings (`02-doctor-receipts.md`).
- **`affi fix`** — separate verb for receipt repairs, **Finalize / Quarantine only**, `--dry-run`/`--apply`
  (`00-SYNTHESIS.md:96-100`). Destructive-looking data ops are never a flag on a diagnostic command.
- The shared **`DoctorCheck` / `Finding` framework** (mirroring `CheckOutcome`, `types.rs:269`) on a
  `linkme` registry (`Cargo.toml:42`, currently unused in `src/` — this is its first idiomatic use).
- The timeline extension: continuous/scheduled health → fleet/CI doctor → predictive health by 2030.

### Out of scope (W2 consumes, does not build)

- **W1 Foundations & Correctness** — owns the bug fixes B1–B4 (output streams, JSON escaping,
  silent-tamper-drop in `load_receipts_from_path` at `handlers.rs:84` = bug B3, genesis seed drift =
  bug B4) and the `output.rs`/`diag.rs` output/diag contract. W2 **consumes** that contract for its
  `--json`/exit-code envelope and the stable error-code catalog.
- **W4 Onboarding / Registry** — owns `src/registry.rs` (verb/feature source-of-truth). W2's `features`
  check **consumes** the verb→feature mapping from there rather than re-deriving the scattered `#[cfg]`.
- **W7 Verification Engine** — owns the verifier and any new profiles. W2 **calls** `verifier::verify`
  verbatim and never re-implements a check.
- **W8 Cryptography & Trust** — owns signatures / PQC. Doctor *reports* their presence/reachability only.

### Boundaries restated as a one-line contract

> Doctor reads facts and ranks them; `fix` only re-finalizes a chain that already chains, or moves a
> bad file aside. Honest/ACCEPT judgments come *exclusively* from calling the existing verifier.

---

## 2. Current State (cited) & Gap

### What exists and is reused (read-only)

| Capability | Symbol / location | W2 use |
|---|---|---|
| 7-stage certify pipeline | `verifier::verify(&Receipt) -> Verdict` — `src/verifier.rs:43` | called **verbatim** for honest receipts |
| Rolling chain recompute | `chain::recompute_chain(&[OperationEvent])` — `src/chain.rs:68` | tamper detection in receipt mode |
| Sealed mint seam | `ChainAssembler::{from_events,finalize}` — `src/chain.rs:107`,`:135` | the ONLY path `fix` re-seals through |
| Receipt seal | `Receipt::sealed` (`pub(crate)`) + private `_seal` — `src/types.rs:93`,`:222` | makes forgery unconstructable (E0451) |
| Re-verifying deserialize | `impl Deserialize for Receipt` — `src/types.rs:110-143` (rejects tamper at `:131`) | why doctor needs a *shadow* parse |
| Working chain | `WORKING_PATH=".affi/working.json"` — `src/chain.rs:25`; `load_working` — `:169` | `working-dir` + `OrphanedWorking` checks |
| Genesis seed | `GENESIS_SEED=b"affidavit-v26.6.14-genesis"` — `src/chain.rs:22` | `genesis` cross-box drift check |
| Format version | `FORMAT_VERSION="core/v1"` — `src/chain.rs:19` | `format-version` / `VersionDrift` checks |
| Outcome shape | `CheckOutcome{stage,passed,detail}` — `src/types.rs:269` | `Finding` deliberately mirrors it |
| Verb wrapper pattern | `#[verb("diagnose","receipt")]` — `src/verbs/diagnose.rs:13` | `doctor`/`fix` follow it exactly |
| Handler plumbing | `adapt`/`io_err`/`print_json_or` — `src/handlers.rs:59-108` | error mapping + JSON envelope |
| Feature gates | `default=["core"]` `Cargo.toml:118`; `discovery` `:127`; `lsp` `:133`; `otel` `:143` | `features`/`registry` checks |
| Existing watcher | `FileWatcher` — `src/quality.rs:1147` (debounced, `notify`-backed) | drives 2027 continuous mode |
| Existing fleet stub | `handlers::portfolio_health` — `src/handlers.rs:2078` | aligns with 2028 fleet doctor |

### The gap (what does NOT exist)

1. **No `doctor` verb anywhere** — `doctor` appears only in unrelated `thesis/`/`survey/` docs
   (`01-doctor-command.md:39`). The nearest verb, `diagnose` (`handlers.rs:973`), is *receipt-scoped*,
   renders a `Verdict` as LSP diagnostics, and fails with `"lsp feature not enabled"` (`handlers.rs:996`)
   without the `lsp` feature. It tells you **nothing about your install**.
2. **Tampered receipts are invisible to every directory loader** — `load_receipts_from_path`
   (`handlers.rs:84`) does `if let Ok(r) = crate::cli::show(...)` and **silently swallows** the
   deserialize-time chain-hash error (`types.rs:131`). So `verify_family`/`query`/`portfolio_health`
   literally cannot see the files you most need to see. (Synthesis bug **B3**; W1 fixes the loader, W2
   additionally needs a *shadow* parse to report-not-drop.)
3. **No store-wide aggregation, triage, or score** — `verify_family` (`handlers.rs:373`) does a *weaker*
   check than the pipeline (`format_version=="core/v1" && events_len>0`, `handlers.rs:386`) and emits no
   findings, scores, or remediations.
4. **No notion of store *working* state** — orphaned/abandoned `working.json`, stale temp files, version
   drift across the set — is reported anywhere.
5. **No safe-repair primitive** — there is no `fix`; the only "repair" is manual loop-and-eyeball.
6. **`linkme` declared but unused** (`Cargo.toml:42`) — the check registry is its first real use.

---

## 3. Phased Plan (2026 H2 / 2027 / 2028 / 2029 / 2030)

Sizes: **S** ≤ ½ day · **M** ~1–2 days · **L** ~3–5 days (estimates, unverified against a real build).
Each phase is independently shippable; later phases extend, never rewrite, earlier ones.

---

### Phase 2026 H2 — The spine + the checks that pay rent (anchors synthesis P0/P1)

**Objective:** Ship the `DoctorCheck`/`Finding` framework, the single `affi doctor` command with
environment-default + receipt-when-pathed behaviour, and the separate `affi fix` (Finalize/Quarantine).
This is the synthesis P0 framework + P1 receipt mode and `fix`, scoped to W2.

**Deliverables**

1. **`src/doctor/mod.rs`** — `Status{Ok,Warn,Fail}`, `Finding`, `DoctorCtx`, `DoctorCheck` trait (mirrors
   `CheckOutcome`, `types.rs:269`). No "honest"/"dishonest" status exists by construction.
2. **`src/doctor/registry.rs`** — `linkme::distributed_slice` of checks + deterministic `ordered_checks()`
   (sort by name, not link order → same inputs, same output).
3. **Environment checks (P0 set):** `genesis`, `working-dir` (auto-fixable: archive), `features`, `config`,
   `completions`. (`binary`/`toolchain` follow as S add-ons.)
4. **Receipt-store mode:** `ShadowReceipt` defensive parse + `assess_receipt` reusing `verifier::verify`
   verbatim; deterministic 0–100 `score_from`; `StoreHealth` rollup.
5. **`src/fix_engine.rs` + `affi fix`** — `SafeRepair{FinalizeWorking, Quarantine}`, `--dry-run`/`--apply`.
6. **Verb wrappers** `src/verbs/doctor.rs`, `src/verbs/fix.rs`; `--json` envelope via W1's contract;
   exit `0` clean / `2` on Fail (mirrors `verify`, `handlers.rs:367`).

```rust
//! src/doctor/mod.rs (NEW) — compilable-style; mirrors types::CheckOutcome (types.rs:269)
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status { Ok, Warn, Fail } // NOTE: intentionally no "honest"/"dishonest" variant.

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub check: String,             // stable kebab-case key; also `--check <name>`/`--fix` selector
    pub status: Status,
    pub finding: String,           // one line: what is true now
    pub remediation: Option<String>, // one line: exact command/edit; None when Ok
    pub auto_fixable: bool,        // true iff a safe, idempotent fix exists
}

/// Ambient facts gathered once, shared by all checks (keeps checks pure + unit-testable).
pub struct DoctorCtx {
    pub pkg_version: &'static str,         // env!("CARGO_PKG_VERSION") => "26.6.17"
    pub genesis_tag: String,               // parsed from chain::GENESIS_SEED (chain.rs:22)
    pub working_path: std::path::PathBuf,  // --working-dir or chain::WORKING_PATH (chain.rs:25)
    pub compiled_features: &'static [&'static str],
    pub receipts: Option<std::path::PathBuf>, // Some => also run store mode
    pub apply_fixes: bool,                 // --fix (env fixes only; receipt repair lives in `affi fix`)
}

pub enum Fixed { Applied(String), Skipped(String) }

pub trait DoctorCheck: Sync {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &DoctorCtx) -> Finding;            // read-only, never mutates
    fn fix(&self, _ctx: &DoctorCtx) -> anyhow::Result<Fixed> {
        Ok(Fixed::Skipped("no automatic fix for this check".into()))
    }
}
```

```rust
//! src/doctor/registry.rs (NEW) — first `linkme` use in src/ (Cargo.toml:42)
use linkme::distributed_slice;
use super::DoctorCheck;

#[distributed_slice]
pub static DOCTOR_CHECKS: [&'static (dyn DoctorCheck)] = [..];

/// Deterministic order — doctrine: same inputs → same output.
pub fn ordered_checks() -> Vec<&'static dyn DoctorCheck> {
    let mut v: Vec<_> = DOCTOR_CHECKS.iter().copied().collect();
    v.sort_by_key(|c| c.name());
    v
}
```

```rust
//! src/doctor/store.rs (NEW) — receipt mode: SEE what the verifier rejects, then reuse it verbatim.
use serde::{Deserialize, Serialize};
use crate::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use crate::types::{Blake3Hash, OperationEvent};
use crate::verifier::verify;

/// On-disk shape with NO custom Deserialize — parsing this NEVER blesses anything.
/// (Receipt::deserialize re-chains and rejects tamper at types.rs:131; we must bypass that to *report* it.)
#[derive(Debug, Clone, Deserialize)]
struct ShadowReceipt {
    #[serde(default)] format_version: String,
    #[serde(default)] events: Vec<OperationEvent>,
    #[serde(default)] chain_hash: Blake3Hash,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReceiptHealth { pub file: String, pub score: u8, pub certifiable: bool, pub findings: Vec<Finding> }

fn score_from(findings: &[Finding]) -> u8 {            // CRITICAL alone floors to 0; deterministic
    let loss: u32 = findings.iter().map(|f| penalty(f.status) as u32).sum();
    100u8.saturating_sub(loss.min(100) as u8)
}

fn assess_receipt(file: &str, shadow: ShadowReceipt) -> ReceiptHealth {
    let mut findings = Vec::new();
    match recompute_chain(&shadow.events) {            // chain.rs:68 — the gate on everything
        Ok(c) if c == shadow.chain_hash => {
            // Sound chain: mint a REAL receipt (sealed seam) and run the canonical pipeline verbatim.
            match ChainAssembler::from_events(shadow.events.clone()).map(|a| a.finalize()) {
                Ok(real) => for o in verify(&real).outcomes.iter().filter(|o| !o.passed) {
                    findings.push(finding_from_stage(file, o)); // root_cause verbatim from verifier
                },
                Err(e) => findings.push(unparseable(file, format!("seal failed: {e}"))),
            }
        }
        Ok(_) => findings.push(chain_hash_mismatch(file)), // TAMPER: report; NEVER seal. Fix offered = Quarantine.
        Err(e) => findings.push(unparseable(file, format!("canonicalization failed: {e}"))),
    }
    let certifiable = findings.iter().all(|f| f.status != Status::Fail);
    ReceiptHealth { file: file.into(), score: score_from(&findings), certifiable, findings }
}
```

```rust
//! src/fix_engine.rs (NEW) — the ONLY two repairs; both provably non-laundering.
pub enum SafeRepair {
    FinalizeWorking { working_path: String, out_path: String }, // re-seal an already-chaining working set
    Quarantine { file: String, reason: String },                // move a bad file aside (never edits bytes)
}

impl FixEngine {
    pub fn apply(&self, repair: &SafeRepair) -> anyhow::Result<()> {
        match repair {
            SafeRepair::FinalizeWorking { working_path, out_path } => {
                let events = crate::chain::load_working_at(working_path)?; // generalizes load_working (chain.rs:169)
                let asm = ChainAssembler::from_events(events)?;            // chain.rs:107 — fails if it doesn't chain
                let receipt = asm.finalize();                             // chain.rs:135 — the sealed seam
                debug_assert!(verify(&receipt).accepted, "fix must NEVER emit a non-ACCEPT receipt");
                crate::chain::save_receipt(&receipt, std::path::Path::new(out_path))?;
                Ok(())
            }
            SafeRepair::Quarantine { file, reason } => quarantine_file(file, reason), // fs move + sidecar note
        }
    }
}
```

```rust
//! src/verbs/doctor.rs + src/verbs/fix.rs (NEW) — thin wrappers (diagnose.rs:13 pattern)
#[verb("doctor", "env")] // env default; adds store mode when `receipts` is Some / inside a .affi/ store
pub fn doctor(fix: Option<bool>, json: Option<bool>, check: Option<String>,
              receipts: Option<String>, working_dir: Option<String>) -> Result<()> {
    crate::handlers::doctor(fix, json, check, receipts, working_dir)
}

#[verb("fix", "receipt")]
pub fn fix(receipts: Option<String>, apply: Option<bool>,
           finalize_working: Option<String>, quarantine: Option<String>) -> Result<()> {
    crate::handlers::fix(receipts, apply, finalize_working, quarantine)
}
```

**Acceptance criteria**
- `affi doctor` with no path runs env checks; with a path / `--receipts` *additionally* runs store mode.
  One command, two modes (synthesis Part C).
- Honest store → every receipt scores 100 and `certifiable == verify(receipt).accepted` **bit-identically**.
- A tampered receipt is **surfaced** as a Fail/Critical finding (not dropped as `handlers.rs:84` would),
  with a `Quarantine` remediation; exit code `2`.
- `affi fix --dry-run` lists planned repairs and never writes; `--apply` finalizes orphaned-but-sound
  working sets and quarantines tampered files; a property test proves `fix` cannot raise any tampered
  file to ACCEPT (see §6).
- `--json` output validates against a schema and matches the human view field-for-field.
- Unit-testable via a `DoctorCtx` fixture; deterministic `ordered_checks()`.

**Cross-workstream deps**
- **W1:** consume `output.rs`/`diag.rs` for the JSON/exit-code envelope and stable codes; rely on W1's
  B3 loader fix (doctor adds shadow-parse on top). **W4:** consume verb/feature registry for `features`
  check (fall back to a local `const FEATURE_NEEDS` if registry not yet landed). **W7:** call `verify`.

---

### Phase 2027 — Continuous & scheduled health (`affi doctor watch` / hook integration)

**Objective:** Move from one-shot to *continuous*. A long-running doctor re-scans on change, and a
git/CI hook surface makes "doctor is green" a gate. Drive the **existing** `FileWatcher`
(`src/quality.rs:1147`) — do not write a second watch loop (the `monitor` stub at `handlers.rs:2632`
is "tokio-based watch loop not yet implemented" = synthesis bug B5; W2 either reuses W6's `watch` or
drives `FileWatcher` directly).

**Deliverables**
1. **`affi doctor watch [--receipts <dir>] [--interval <s>]`** — debounced re-scan over the store using
   `FileWatcher`, emitting a delta of findings (new/resolved) per cycle. No new watcher logic.
2. **Snapshot/diff** — persist the last `StoreHealth` to `.affi/.doctor-state.json`; report only *changes*
   so CI logs stay quiet when healthy.
3. **`affi doctor --since <snapshot>`** — compare current health against a recorded baseline (regression
   gate: "no receipt got *worse*").
4. **CI/hook recipe** — a `doctor --json` pre-merge gate; exit `2` blocks. (W5 owns the hook *suite*; W2
   provides the doctor entrypoint + recommended config.)
5. **Check budget / fast mode** — `--fast` skips network-backed `registry` probes so the watch loop never
   stalls a build.

```rust
//! src/doctor/watch.rs (NEW) — reuse, don't reinvent (quality.rs:1147; retire stub handlers.rs:2632)
pub fn doctor_watch(ctx: DoctorCtx, interval_s: u64) -> anyhow::Result<()> {
    let mut prev = run_store_scan(&ctx)?;                      // initial baseline
    let mut watcher = crate::quality::FileWatcher::new(       // existing, debounced, notify-backed
        ctx.receipts.as_ref().unwrap().to_str().unwrap(), 500)?;
    loop {
        watcher.wait_for_change_or_interval(interval_s)?;     // FileWatcher's own debounce
        let next = run_store_scan(&ctx)?;
        let delta = StoreHealthDelta::between(&prev, &next);  // new vs resolved findings
        if !delta.is_empty() { render_delta(&delta); }        // chatter→stderr per W1 contract
        prev = next;                                          // verdicts still come from verify() each scan
    }
}
```

**Acceptance criteria**
- `doctor watch` reports a newly-tampered file within one debounce interval and a *resolved* finding when
  the file is quarantined — driven entirely by `FileWatcher`, with the `monitor` stub retired/aliased.
- `--since` exits `2` iff any receipt's score decreased vs the baseline; never on a *new healthy* receipt.
- `--fast` performs zero network calls.
- Each scan re-runs `verifier::verify`; no cached verdict is ever trusted across edits (cache, if any,
  is keyed on content address so an edit invalidates it).

**Cross-workstream deps**
- **W6 Interactive Surfaces** owns `affi watch`; W2 either composes it or shares `FileWatcher`. **W5
  Workflow Automation** owns the git-hook suite; W2 supplies the doctor gate. **W1** contract for delta
  rendering (data→stdout / chatter→stderr).

---

### Phase 2028 — Fleet & CI doctor (many stores, one rollup)

**Objective:** Scale doctor from one `.affi/` to a *fleet* — many repos/stores rolled up into one
prioritized board, suitable for a platform team. Align with the existing `portfolio_health` surface
(`handlers.rs:2078`, verb `portfolio-health` `src/verbs/portfolio_health.rs:13`) without inheriting its
weak check (it currently never calls the verifier).

**Deliverables**
1. **`affi doctor fleet <root> [--json]`** — discover every `.affi/` under `<root>`, run store mode on
   each, and emit a `FleetHealth` rollup (worst-first), each store reusing `assess_receipt` (so every
   store-level verdict is still `verify`-exact).
2. **Stable machine output for CI** — `FleetHealth` JSON with per-store `store_score`, `worst_severity`,
   and counts; a single fleet exit code (`2` if any store has an Error/Critical).
3. **Quarantine policy at scale** — `affi fix --fleet <root> --dry-run` plans per-store safe repairs;
   `--apply` still performs only Finalize/Quarantine, per store, with an aggregate report.
4. **CI annotations** — optional GitHub-style `::error file=…` lines (gated, opt-in) so findings surface
   inline in CI without coupling doctor to any CI vendor.
5. **OTel span per scan** — under the `otel` feature (`Cargo.toml:143`), one span per store + a fleet
   parent, for platform observability. No-op without the feature.

```rust
//! src/doctor/fleet.rs (NEW) — align with portfolio_health (handlers.rs:2078) but verify-exact per store
#[derive(serde::Serialize)]
pub struct FleetHealth { pub root: String, pub stores: Vec<StoreHealth>, pub worst: Severity }

pub fn doctor_fleet(root: &std::path::Path) -> anyhow::Result<FleetHealth> {
    let mut stores = Vec::new();
    for affi_dir in discover_affi_stores(root)? {            // walk for `.affi/` dirs
        stores.push(run_store_scan_at(&affi_dir)?);          // each calls assess_receipt → verify() verbatim
    }
    stores.sort_by_key(|s| s.store_score);                   // worst-first
    let worst = stores.iter().map(|s| s.worst_severity).max().unwrap_or(Severity::Info);
    Ok(FleetHealth { root: root.display().to_string(), stores, worst })
}
```

**Acceptance criteria**
- `doctor fleet` over N stores reports each store's verify-exact health and a deterministic worst-first
  rollup; a tampered receipt in *any* store drives fleet exit `2`.
- `fix --fleet --apply` performs only Finalize/Quarantine per store; the doctrine property test extends to
  the fleet path (no store's tampered file is ever raised to ACCEPT).
- `otel` spans appear only when the feature is compiled; a `--features core` build makes zero spans/network
  calls.

**Cross-workstream deps**
- **W10 Compliance & Governance** consumes `FleetHealth` for org-wide attestation/audit dashboards
  (W2 provides the data feed; W10 owns policy/governance semantics). **W9 Ecosystem & Standards** may
  consume the rollup for OCEL/standards export. **W1** envelope + exit-code catalog.

---

### Phase 2029 — Extensible & policy-aware doctor (third-party checks, profile-aware health)

**Objective:** Open the check registry to *plugins* and make health **profile-aware**, so doctor scales
to organization-specific environment and receipt expectations — still suggest-only, still verify-exact.

**Deliverables**
1. **Third-party `DoctorCheck`s** — document and stabilize the `#[distributed_slice(DOCTOR_CHECKS)]`
   extension path (the CLAUDE.md plugin pattern). A plugin check is a one-file, no-central-edit addition.
2. **Profile-aware scoring** — when W7 ships multi-profile validation, `assess_receipt` runs `verify`
   under the requested profile and scores against it; an unknown profile is a `ProfileMismatch` *Warning*
   (never a silent pass). W2 consumes profiles; it does not define them.
3. **Policy bundles** — `affi doctor --policy <file>` selects which checks are required/advisory and which
   findings gate CI (e.g. "VersionDrift is Error in prod"). Policy can only *tighten* severity or
   *select* checks — **it can never reclassify a tampered chain as acceptable** (enforced: chain integrity
   is non-negotiable; see §6).
4. **Remediation library** — structured, versioned remediation strings keyed by stable `FindingCode`
   (`02-doctor-receipts.md` catalog), so messages are greppable and never reworded across releases.
5. **LSP surfacing** — emit `Finding`s as workspace diagnostics through `crate::lsp` (reuse the `diagnose`
   shape, `handlers.rs:973`) under the `lsp` feature.

```rust
//! Policy can SELECT/TIGHTEN, never launder. Chain integrity is structurally exempt from override.
pub struct DoctorPolicy { required: Vec<String>, gate_at: Severity, /* ... */ }

impl DoctorPolicy {
    /// Apply policy to a finding's severity. ChainHashMismatch is floored at Critical, always.
    fn effective_severity(&self, code: FindingCode, base: Severity) -> Severity {
        if code == FindingCode::ChainHashMismatch { return Severity::Critical; } // non-overridable
        base.max(self.tighten_for(code)) // policy may only raise, never lower
    }
}
```

**Acceptance criteria**
- A third-party crate can register a check with no edit to W2 source; `ordered_checks()` includes it
  deterministically.
- Profile-aware scoring matches `verify(receipt, profile)` exactly for that profile.
- A property test proves: for *any* policy file, a tampered receipt's effective severity is still
  Critical and `certifiable` is still false (policy cannot launder).
- LSP diagnostics mirror the `Finding` set under `lsp`; without the feature, doctor still runs (no
  `"lsp feature not enabled"` dead-end for the core path).

**Cross-workstream deps**
- **W7** (multi-profile), **W4** (registry for plugin discovery surfacing), **W8** (if signature checks
  become plugin checks, doctor reports presence/validity-of-format only), **W10** (policy semantics align
  with governance).

---

### Phase 2030 — Predictive & continuous fleet health (the steady state)

**Objective:** Close the loop: doctor not only reports *current* health but flags *trends* — stores
drifting toward failure (rising orphan rate, growing version-drift, repeated near-tamper) — as advisory
**predictions**. Doctrine holds absolutely: a prediction is a *warning about the future*, never a verdict
about the present, and never a basis for accepting anything.

**Deliverables**
1. **Health timeseries** — persist per-store `StoreHealth` snapshots over time (`.affi/.doctor-history/`);
   `affi doctor trend <store|fleet>` renders the trajectory and a deterministic, explainable risk score.
2. **Predictive findings (advisory only)** — e.g. "orphaned-working rate up 4× over 30 days → a crashed
   pipeline is recurring." Severity capped at **Warning**; predictions are `FixClass::None` (never
   auto-actioned) and carry an explicit "this is a forecast, not a verdict" banner.
3. **Continuous fleet daemon** — `affi doctor serve` runs the 2028 fleet scan on a schedule, exposes a
   read-only health endpoint/JSON, and (under `otel`/`metrics`, `Cargo.toml:143-145`) exports gauges.
   Read-only: it never mutates a store and never calls `fix`.
4. **Self-test of the doctor** — `affi doctor --self-check` runs the doctrine property suite at runtime
   (or asserts it was run at build) so a deployed binary can attest "my fix engine cannot launder."
5. **Definition-of-done audit hooks** — emit `Finding`/trend data in a stable shape W10 can attest over.

```rust
//! src/doctor/predict.rs (NEW) — forecasts are advisory; capped at Warning; never gate ACCEPT.
#[derive(serde::Serialize)]
pub struct Prediction {
    pub subject: String,
    pub signal: String,        // e.g. "orphaned_working_rate"
    pub trend: f64,            // deterministic slope over the recorded window
    pub note: String,          // "forecast, not a verdict"
}

impl Prediction {
    pub fn into_finding(self) -> Finding {
        // ALWAYS advisory: a prediction can warn, never fail, never feed `fix`.
        Finding { check: format!("trend:{}", self.signal), status: Status::Warn,
                  finding: self.note, remediation: None, auto_fixable: false }
    }
}
```

**Acceptance criteria**
- Trend scores are deterministic over a fixed history (same snapshots → same score) and explainable
  (every prediction cites the signal + window it derived from).
- No prediction can change a receipt's `certifiable` flag or a store's pass/fail verdict; a test asserts
  predictions are emitted only as `Status::Warn` with `FixClass::None`.
- `doctor serve` is provably read-only (never imports `fix_engine::apply`); a test/grep gate enforces it.
- `--self-check` passes the §6 property suite on the shipped binary.

**Cross-workstream deps**
- **W10** consumes trend/attestation feeds; **W7** for any new verdict semantics (W2 still never invents
  one); **W1** for the stable serialization contract the history relies on.

---

## 4. Definition of Done @ 2030

By end of 2030, W2 delivers a doctor/self-healing system where:

1. **One `affi doctor`** answers environment health by default and store health when pathed — never two
   commands the user must discover (synthesis Part C honored).
2. **`affi fix`** is the *only* mutating verb, and it does *only* Finalize (re-seal an already-chaining
   working set) and Quarantine (move a bad file aside). No code path edits an event and recomputes a hash.
3. **Every honest-receipt verdict is bit-identical to `affi verify`** because doctor calls
   `verifier::verify` (`src/verifier.rs:43`) verbatim through the sealed seam.
4. Health is **continuous** (watch), **fleet-scale** (rollup), **extensible** (plugin checks),
   **policy-aware** (tighten-only), and **predictive** (advisory-only) — each layer additive over the
   2026 H2 spine.
5. A **machine-checkable doctrine property suite** (§6) ships with the binary (`--self-check`) and proves
   no doctor/fix/watch/fleet/policy/predict path can launder a tampered chain into ACCEPT.
6. Output is **stable and dual** (human + `--json`) on the W1 contract, with a stable `FindingCode`/exit-code
   catalog suitable for CI gates and W10 governance attestation.

**Done means:** a platform engineer runs `affi doctor fleet ~/repos`, sees a worst-first board, runs
`affi fix --fleet --apply`, watches the quarantined-tamper count and the sealed-orphan count change — and
can *prove*, by audit, that no dishonest receipt was ever turned into an ACCEPT.

---

## 5. Cross-Workstream Dependencies

| Direction | Workstream | What W2 needs / provides |
|---|---|---|
| **W2 ← W1** Foundations & Correctness | consume | `output.rs`/`diag.rs` (JSON envelope, exit-code catalog, data→stdout/chatter→stderr), stable error codes; B3 loader fix (silent-tamper-drop at `handlers.rs:84`); B4 genesis seed fix feeds the `genesis` check; B5 `monitor`-stub retirement feeds the 2027 watch. **W2 does not build these.** |
| **W2 ← W4** Onboarding/Registry | consume | `src/registry.rs` verb→feature mapping for the `features` check and plugin-check surfacing. Fallback: local `const FEATURE_NEEDS` until registry lands. |
| **W2 ← W7** Verification Engine | consume | `verifier::verify` (called verbatim) and multi-profile validation (2029 profile-aware scoring). W2 never re-implements a check. |
| **W2 ← W8** Cryptography & Trust | consume | Signature/PQC presence + format-validity to *report* (never to *decide* honesty). Possible 2029 plugin checks. |
| **W2 → W5** Workflow Automation | provide | `doctor --json` gate + recommended config for the git-hook suite (W5 owns the hooks). |
| **W2 ↔ W6** Interactive Surfaces | share | `FileWatcher` (`quality.rs:1147`) for `doctor watch`; compose or share W6's `affi watch`. |
| **W2 → W10** Compliance & Governance | provide | `Finding`/`StoreHealth`/`FleetHealth`/trend feeds for org attestation; W10 owns policy/governance semantics, W2 owns the health data. |
| **W2 → W9** Ecosystem & Standards | provide | Health rollups available for OCEL/standards export. |

---

## 6. Doctrine Compliance — no doctor/fix path launders a tampered chain into ACCEPT

The doctrine is **certify, don't decide**; the danger a self-healing feature courts is *laundering* —
silently turning a REJECT-worthy receipt into an ACCEPT. This holds **across every phase** by construction:

1. **Minting is sealed; doctor/fix cannot forge.** A `Receipt` exists only via `Receipt::sealed`
   (`pub(crate)`, `types.rs:93`) behind the private `_seal` field (`types.rs:222`), reachable solely
   through `ChainAssembler::finalize` (`chain.rs:135`). `finalize` stamps the chain hash from the bytes
   it is given — it cannot be told to lie. There is no public API producing a `Receipt` whose chain hash
   disagrees with its events.
2. **The verifier is reused verbatim, never re-implemented.** `assess_receipt` runs `verifier::verify`
   (`verifier.rs:43`) on receipts minted through that sealed seam. `doctor`'s `certifiable` flag is, by
   construction, exactly `verify(receipt).accepted` — it defines no looser pass condition (contrast the
   legacy `verify_family` shortcut, `handlers.rs:386`, which W2 replaces).
3. **Tamper is observed, never sealed.** A tampered file fails `recompute_chain` equality (`chain.rs:68`),
   so `assess_receipt` takes the mismatch branch, emits `ChainHashMismatch (Critical)`, and **never** calls
   `from_events().finalize()` on it. The only repair offered is `Quarantine` (an fs move; bytes never
   edited). The *shadow* parse exists precisely so this tamper is *reported* rather than silently dropped.
4. **`fix --finalize-working` re-seals, it does not repair.** It runs only when the working events
   *already* chain (`ChainAssembler::from_events` succeeds, `chain.rs:107`), guarded by
   `debug_assert!(verify(&receipt).accepted)`. It assigns a *name* (content address) to bytes that already
   chain; it changes no event, so it cannot change a verdict.
5. **Safe/Unsafe is partitioned at planning time.** `FixEngine::plan` derives repairs *only* from
   `FixClass::Safe` findings. Everything that could change meaning — version drift, malformed chain,
   non-contiguous `seq` — is suggest-only and structurally excluded from `apply`. There is no `--force`
   that crosses this line.
6. **Later phases cannot reopen the door.** *watch/fleet/serve* re-run `verify` each scan and never cache
   a verdict across an edit (any cache is content-address-keyed). *policy* may only *tighten* severity or
   *select* checks — `ChainHashMismatch` is floored at Critical and `certifiable` cannot be overridden
   (§Phase 2029 sketch). *predictions* are advisory `Status::Warn`/`FixClass::None` and never feed `fix`
   or a verdict. *serve* is read-only and never imports `fix_engine::apply`.
7. **"Fix" is monotone on trust.** `FinalizeWorking` *adds* a receipt that already passes; `Quarantine`
   *removes* one that doesn't. The set of bytes the verifier accepts is **identical before and after any
   fix** — auditable by grep: the only `finalize` call sites in W2 code are gated by a successful
   `recompute_chain` and an `accepted` assertion.

**Net:** doctor only *reports and ranks*; fix only *re-seals already-honest bytes or quarantines dishonest
ones*; predictions only *warn about the future*. The question "is this work honest?" is never answered by
W2 — exactly as the doctrine demands. The `--self-check` property suite (Phase 2030) makes this checkable
on the shipped binary.
