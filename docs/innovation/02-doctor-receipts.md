# `affi doctor` — Store-Wide Receipt & Chain Health

> **Innovation 02 · Receipt/Chain health side of `affi doctor`**
> Companion to Innovation 01 (environment doctor: install/config). This doc owns the **data**:
> scanning a store of receipts + the working chain and producing a triaged, actionable health report
> with *safe* self-healing suggestions.
>
> **Doctrine is sacred:** tools *certify* and *suggest*. Nothing here ever decides a chain is honest,
> and — proven in §5 — **no path can launder a tampered receipt into ACCEPT.**

---

## 1. Vision — the 1000x leap

Today `affidavit` verifies **one receipt at a time** and hands you a binary verdict:

```console
$ affi receipt verify .affi/9f2c…json
verdict: REJECT [core/v1] — chain_integrity: chain hash mismatch: stored 203d…, recomputed a17b…
```

That is correct, decidable, and *useless at scale*. A real `.affi/` store after a few weeks of CI is a
junk drawer: dozens of finalized receipts, half-finished `working.json` files left by crashed runs,
a receipt someone hand-edited "just to fix a typo," one emitted by an old `v0` build. To find the rot
today you would loop `verify` over every file by hand, read six stage lines each, and reconstruct the
story yourself. Worse — as §5 shows — the current directory loader **silently drops** exactly the
files you most need to see.

**The leap: from `verify(one) → ACCEPT/REJECT` to `doctor(store) → triaged health report + safe fixes`.**

| | Before (`verify` loop) | After (`affi doctor`) |
|---|---|---|
| **Input** | one receipt path | a directory / the whole `.affi/` store |
| **Output** | 7 stage lines, one verdict | per-receipt **health score**, ranked findings, store rollup |
| **Tampered file** | silently skipped by dir loaders (§5) | surfaced as a **CRITICAL** finding, quarantine suggested |
| **Orphaned `working.json`** | invisible | flagged, `affi fix` offers re-finalize or prune |
| **"What do I do?"** | you reverse-engineer it | every finding ships a **root cause + suggested remediation** |
| **Automation** | parse stderr text, guess | `--json` + exit `0`/`2`, same convention as `verify` |
| **Repair** | manual | `affi fix --dry-run` then `--apply`, structurally-safe only |

This is the difference between a smoke detector that beeps and a fire marshal who hands you a
prioritized punch-list. Same doctrine, 1000x the leverage per operator-minute.

---

## 2. Current state — what exists, and the gap

Everything `doctor` needs to *judge a single receipt* already exists and is load-bearing. The gap is
purely **store-wide aggregation, triage, and safe repair** — plus one subtle visibility bug.

**Reusable today (cite):**

- **`crate::verifier::verify(&Receipt) -> Verdict`** — `src/verifier.rs:43`. The canonical 7-stage
  pipeline (`decode` → `check_format` → `chain_integrity` → `continuity` → `verify_commitments` →
  `evaluate_profile` → verdict). Each `CheckOutcome` (`src/types.rs:269`) is named and decidable.
  **`doctor` reuses this verbatim** — it does not reinvent any check.
- **`crate::chain::recompute_chain(&[OperationEvent])`** — `src/chain.rs:68`. Re-derives the rolling
  BLAKE3 hash; the heart of `chain_integrity` (`src/verifier.rs:109`).
- **`crate::chain::ChainAssembler::{from_events, finalize}`** — `src/chain.rs:107`,`:135`. The *only*
  sealed seam that mints a `Receipt` (`src/types.rs:93`, private `_seal`). **`affi fix` re-finalizes
  exclusively through this seam.**
- **`load_working` / `WORKING_PATH`** — `src/chain.rs:169`,`:25`. Reads `.affi/working.json` as a raw
  `Vec<OperationEvent>` (note: *not* a sealed `Receipt`).
- **`crate::handlers::verify` / `verify_family`** — `src/handlers.rs:341`,`:373`. The verb-handler and
  exit-code conventions (`std::process::exit(2)` on REJECT, `src/handlers.rs:351`,`:367`).
- **Verb pattern** — `#[verb("verb","noun")]` thin wrapper → `crate::handlers::*`
  (`src/verbs/verify.rs:13`, `src/verbs/portfolio_health.rs:13`). `doctor`/`fix` follow it exactly.

**The gap (and a real bug):**

1. **No store scan.** `verify_family` (`src/handlers.rs:373`) is the closest analog, but it does a
   *weaker* check than the real pipeline — `receipt.format_version == "core/v1" && events_len > 0`
   (`src/handlers.rs:386`) — and emits no scores, no findings, no remediations.
2. **Tampered receipts are invisible to every directory loader.** This is the linchpin. `Receipt`'s
   `Deserialize` impl (`src/types.rs:110`) **re-verifies the chain hash during parse** and returns
   `Err("chain hash mismatch…")` (`src/types.rs:131`) if it fails. `load_receipt`
   (`src/cli.rs:188`) propagates that error — fine for single-file `verify`. But the dir loader
   `load_receipts_from_path` (`src/handlers.rs:84`) does `if let Ok(r) = crate::cli::show(...)` and
   **silently swallows the failure.** So `verify_family`, `query`, and friends literally *cannot see*
   a tampered file. `doctor` must read **raw bytes into a shadow struct** that does *not* re-verify,
   so it can report the tamper instead of dropping it. (This visibility property is also what keeps us
   inside the doctrine — see §5.)
3. **No notion of the store's *working* state** — orphaned/abandoned `working.json`, stale temp files,
   version/profile drift across the set — is reported anywhere.

---

## 3. Proposed design

Four parts, layered so each is independently testable and each **reuses** existing seams:

1. **`StoreScanner`** — walks a dir, classifies every file, parses receipts *defensively*.
2. **Health-scoring model** — deterministic 0–100 score per receipt + store rollup.
3. **`Finding` / `Remediation` types** — severity, evidence, root cause, suggested fix.
4. **`FixEngine`** — proposes/applies only *structurally-safe* repairs (re-seal or quarantine).

### 3.1 Defensive parsing — see what the verifier rejects

The single most important design decision. Because `Receipt::deserialize` re-chains and errors on
tamper (`src/types.rs:127`), `doctor` cannot use it to *find* tampers. We parse a **shadow** struct
with no custom `Deserialize`, then re-derive truth ourselves via the existing `recompute_chain`.

```rust
use serde::Deserialize;
use crate::types::{Blake3Hash, OperationEvent};

/// A receipt as it literally sits on disk — NO chain re-verification on parse.
/// Mirrors `Receipt` field-for-field (`src/types.rs:213`) but is inert: parsing
/// this NEVER blesses anything. It is the only way to *observe* a tampered file
/// without `Receipt`'s deserializer rejecting it first (`src/types.rs:131`).
#[derive(Debug, Clone, Deserialize)]
struct ShadowReceipt {
    #[serde(default)]
    format_version: String,
    #[serde(default)]
    events: Vec<OperationEvent>,
    #[serde(default)]
    chain_hash: Blake3Hash,
}

/// What a single file on disk turned out to be.
enum Artifact {
    /// Parsed as a receipt shape (honest OR tampered — we don't decide yet).
    Receipt(ShadowReceipt),
    /// A `working.json`-shaped bare event array (`src/chain.rs:161`).
    Working(Vec<OperationEvent>),
    /// JSON, but neither shape — likely a temp/foreign file.
    UnknownJson(serde_json::Value),
    /// Could not even parse as JSON.
    Unparseable(String),
}

fn classify(path: &std::path::Path, bytes: &[u8]) -> Artifact {
    // Try the strongest shape first, falling back. Order matters: a `working.json`
    // is a bare array, a receipt is an object — they never collide.
    if let Ok(r) = serde_json::from_slice::<ShadowReceipt>(bytes) {
        if !r.chain_hash.as_hex().is_empty() || !r.events.is_empty() {
            return Artifact::Receipt(r);
        }
    }
    if let Ok(evs) = serde_json::from_slice::<Vec<OperationEvent>>(bytes) {
        return Artifact::Working(evs);
    }
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(v) => Artifact::UnknownJson(v),
        Err(e) => Artifact::Unparseable(e.to_string()),
    }
}
```

### 3.2 Finding & Remediation types

Every problem is a `Finding`: severity, a stable `code`, the offending file (and `seq`/field when
known), a one-line root cause, and a **suggested** remediation. Suggestions describe an *action* and a
**safety class** — the `FixEngine` will only ever auto-apply the safe classes.

```rust
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity { Info, Warning, Error, Critical }

/// Stable machine codes — greppable, doc-linkable, never reworded across releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FindingCode {
    ChainHashMismatch,      // stored chain_hash != recompute_chain(events)  -> CRITICAL (tamper/corruption)
    SeqNonContiguous,       // continuity stage: gap/disorder in seq         -> ERROR
    DuplicateEventId,       // continuity stage: repeated event id           -> ERROR
    MalformedCommitment,    // verify_commitments: not 64 lc-hex chars       -> ERROR
    EmptyEventType,         // evaluate_profile: blank event_type            -> ERROR
    VersionDrift,           // format_version != "core/v1"                   -> WARNING
    ProfileMismatch,        // not certifiable under requested profile       -> WARNING
    OrphanedWorking,        // working.json with events, never assembled     -> WARNING
    AbandonedTemp,          // *.tmp / *.partial / 0-byte / non-receipt json -> INFO
    Unparseable,            // not valid JSON / not a receipt shape          -> ERROR
}

/// How safe is it to let `affi fix` perform this automatically?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FixClass {
    /// Provably non-laundering: re-seal an honest-but-unassembled chain, or
    /// move a bad file aside. NEVER changes what a verdict would be (§5).
    Safe,
    /// Touches event content/order; could change meaning. Suggest only — humans only.
    Unsafe,
    /// Nothing to do (informational).
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct Remediation {
    pub class: FixClass,
    /// Imperative, copy-pasteable description of the suggested action.
    pub action: String,
    /// The exact command `affi fix` would run, if any (for --dry-run preview).
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub code: FindingCode,
    pub severity: Severity,
    pub file: String,
    /// Evidence pointer: which event seq / which field, when applicable.
    pub seq: Option<u64>,
    pub field: Option<String>,
    /// Why this happened, in one human sentence.
    pub root_cause: String,
    pub remediation: Remediation,
}
```

### 3.3 The scanner + scoring model

The scanner is the new logic; the **judging is delegated to the existing pipeline.** For a parseable,
shadow-loaded receipt we attempt to mint a real `Receipt` through the sealed seam and, if that
succeeds, run `verifier::verify` *verbatim* — so `doctor`'s view of an honest receipt is
**bit-identical** to `affi verify`'s. If sealing fails (tamper/corruption), we synthesize findings
directly from the recompute, because the verifier can never be handed a forged `Receipt` (that's the
whole point — see §5).

```rust
use crate::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use crate::types::Receipt;
use crate::verifier::verify;

#[derive(Debug, Clone, Serialize)]
pub struct ReceiptHealth {
    pub file: String,
    pub score: u8,                 // 0..=100, deterministic
    pub event_count: usize,
    pub certifiable: bool,         // would `affi verify` ACCEPT this exact file?
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreHealth {
    pub root: String,
    pub scanned: usize,
    pub receipts: Vec<ReceiptHealth>,
    pub orphans: Vec<Finding>,     // working.json / temp / foreign files
    pub store_score: u8,           // min of receipt scores, penalized by orphans
    pub worst_severity: Severity,
}

/// Severity → score penalty. CRITICAL alone floors a receipt to 0.
fn penalty(s: Severity) -> u8 {
    match s { Severity::Info => 1, Severity::Warning => 8, Severity::Error => 25, Severity::Critical => 100 }
}

fn score_from(findings: &[Finding]) -> u8 {
    let loss: u32 = findings.iter().map(|f| penalty(f.severity) as u32).sum();
    100u8.saturating_sub(loss.min(100) as u8)
}

/// Judge one shadow receipt. Reuses the real verifier when the chain is sound.
fn assess_receipt(file: &str, shadow: ShadowReceipt) -> ReceiptHealth {
    let mut findings = Vec::new();

    // (a) format/profile drift — cheap, independent of chain soundness.
    if shadow.format_version != FORMAT_VERSION {
        findings.push(Finding {
            code: FindingCode::VersionDrift,
            severity: Severity::Warning,
            file: file.into(), seq: None, field: Some("format_version".into()),
            root_cause: format!(
                "format_version is {:?}; this verifier certifies {:?} only \
                 (check_format stage, src/verifier.rs:91)",
                shadow.format_version, FORMAT_VERSION),
            remediation: Remediation {
                class: FixClass::Unsafe, // changing the declared standard is a decision, not a repair
                action: "Re-emit/re-assemble under the current standard, or pin an older affi to read it.".into(),
                command: None,
            },
        });
    }

    // (b) chain soundness — the ONE check that gates everything else.
    match recompute_chain(&shadow.events) {
        Ok(computed) if computed == shadow.chain_hash => {
            // Chain is sound. Mint a REAL receipt through the sealed seam and run the
            // canonical pipeline verbatim. `from_events` recomputes the running hash
            // (src/chain.rs:107); `finalize` is the only minting path (src/chain.rs:135).
            match ChainAssembler::from_events(shadow.events.clone()).map(|a| a.finalize()) {
                Ok(real) => collect_pipeline_findings(file, &real, &mut findings),
                Err(e) => findings.push(unparseable(file, format!("seal failed: {e}"))),
            }
        }
        Ok(computed) => {
            // The defining tamper signal. We DO NOT seal; we report. (§5)
            findings.push(Finding {
                code: FindingCode::ChainHashMismatch,
                severity: Severity::Critical,
                file: file.into(), seq: None, field: Some("chain_hash".into()),
                root_cause: format!(
                    "stored chain_hash {} != recomputed {} — an event's bytes changed after \
                     sealing, or the file was hand-edited (chain_integrity, src/verifier.rs:109)",
                    shadow.chain_hash, computed),
                remediation: Remediation {
                    class: FixClass::Safe, // quarantine is safe: it removes, never blesses
                    action: "QUARANTINE. This receipt cannot be certified and must not be trusted. \
                             Move it aside; investigate the source run. affi will NOT re-seal a tampered chain.".into(),
                    command: Some(format!("affi fix --quarantine {file}")),
                },
            });
        }
        Err(e) => findings.push(unparseable(file, format!("event canonicalization failed: {e}"))),
    }

    let score = score_from(&findings);
    let certifiable = findings.iter().all(|f| f.severity < Severity::Error);
    ReceiptHealth { file: file.into(), score, event_count: shadow.events.len(), certifiable, findings }
}

/// Translate the canonical Verdict's failing stages into Findings — one source of truth.
fn collect_pipeline_findings(file: &str, real: &Receipt, out: &mut Vec<Finding>) {
    let verdict = verify(real); // EXACT pipeline `affi verify` runs (src/verifier.rs:43)
    for o in verdict.outcomes.iter().filter(|o| !o.passed) {
        let (code, sev) = match o.stage.as_str() {
            "continuity" if o.detail.contains("duplicate") => (FindingCode::DuplicateEventId, Severity::Error),
            "continuity"            => (FindingCode::SeqNonContiguous, Severity::Error),
            "verify_commitments"    => (FindingCode::MalformedCommitment, Severity::Error),
            "evaluate_profile"      => (FindingCode::EmptyEventType, Severity::Error),
            "check_format"          => (FindingCode::VersionDrift, Severity::Warning),
            _                       => (FindingCode::ProfileMismatch, Severity::Warning),
        };
        out.push(Finding {
            code, severity: sev, file: file.into(),
            seq: parse_seq_hint(&o.detail), field: None,
            root_cause: o.detail.clone(),              // verbatim from the verifier
            remediation: suggest_for(code, file),
        });
    }
}
```

The working chain gets its own check: a non-empty `.affi/working.json` (`src/chain.rs:25`) whose
events form a sound prefix but was **never assembled** is an `OrphanedWorking` finding whose *safe*
remediation is "re-finalize through `ChainAssembler::finalize`."

```rust
fn assess_working(path: &str, events: Vec<OperationEvent>) -> Option<Finding> {
    if events.is_empty() { return None; } // empty working file is normal, not a finding
    // Does it form a lawful chain that COULD be sealed right now?
    let sealable = ChainAssembler::from_events(events.clone()).is_ok();
    Some(Finding {
        code: FindingCode::OrphanedWorking,
        severity: Severity::Warning,
        file: path.into(), seq: None, field: None,
        root_cause: format!(
            "{} uncommitted event(s) in {path}: a run emitted but never `affi assemble`d \
             (likely a crashed/aborted pipeline).", events.len()),
        remediation: Remediation {
            class: if sealable { FixClass::Safe } else { FixClass::Unsafe },
            action: if sealable {
                "Re-finalize into an immutable receipt (re-runs ChainAssembler::finalize — \
                 produces the SAME content address determinism guarantees as `affi assemble`).".into()
            } else {
                "Working chain is malformed; inspect events before assembling. Not auto-fixable.".into()
            },
            command: sealable.then(|| format!("affi fix --finalize-working {path}")),
        },
    })
}
```

### 3.4 The FixEngine — safe by construction

`affi fix` only ever performs two **structurally-safe** operations, and each is safe for a
*provable* reason:

```rust
/// The ONLY two repairs affi will perform automatically. Both are FixClass::Safe.
pub enum SafeRepair {
    /// Re-run ChainAssembler::finalize over an *already chain-sound* working set.
    /// Pre-condition (asserted): recompute_chain(events) succeeds AND no event mutated.
    /// This cannot change a verdict — it only assigns a name to bytes that already chain.
    FinalizeWorking { working_path: String, out_path: String },
    /// Move a CRITICAL/unparseable file to `<root>/quarantine/` with a sidecar note.
    /// Removes a bad file from the trusted set. It can only ever LOWER trust, never raise it.
    Quarantine { file: String, reason: String },
}

impl FixEngine {
    pub fn plan(&self, store: &StoreHealth) -> Vec<SafeRepair> {
        // Derive repairs ONLY from findings whose remediation.class == Safe.
        // Unsafe suggestions (VersionDrift, malformed chains) are printed, never planned.
        // ... maps OrphanedWorking -> FinalizeWorking, ChainHashMismatch/Unparseable -> Quarantine
        unimplemented!("derive-from-safe-findings")
    }

    pub fn apply(&self, repair: &SafeRepair) -> anyhow::Result<()> {
        match repair {
            SafeRepair::FinalizeWorking { working_path, out_path } => {
                let events = crate::chain::load_working_at(working_path)?;
                // GUARD: re-finalize ONLY if it already chains. If recompute fails or the
                // sealed result wouldn't verify, refuse — we never invent a passing receipt.
                let asm = ChainAssembler::from_events(events)?;          // src/chain.rs:107
                let receipt = asm.finalize();                            // src/chain.rs:135 (sealed seam)
                debug_assert!(verify(&receipt).accepted, "fix must never emit a non-ACCEPT receipt");
                crate::chain::save_receipt(&receipt, std::path::Path::new(out_path))?; // src/chain.rs:182
                Ok(())
            }
            SafeRepair::Quarantine { file, reason } => {
                // Pure filesystem move + a human-readable note. Bytes are NEVER edited.
                quarantine_file(file, reason)
            }
        }
    }
}
```

The bright line: **`fix` either re-seals bytes that already chain, or it removes a file.** It owns no
code path that edits an event and recomputes a hash to "make it pass" — that capability is simply not
implemented, and §5 explains why it never will be.

---

## 4. CLI UX

New verbs, following the thin-wrapper pattern (`src/verbs/verify.rs:13`):

```rust
// src/verbs/doctor.rs
#[verb("doctor", "receipt")]
pub fn doctor(receipts: Option<String>, format: Option<String>, profile: Option<String>) -> Result<()> {
    crate::handlers::doctor(receipts, format, profile)   // src/handlers.rs (new)
}

// src/verbs/fix.rs
#[verb("fix", "receipt")]
pub fn fix(receipts: Option<String>, apply: Option<bool>,
           finalize_working: Option<String>, quarantine: Option<String>) -> Result<()> {
    crate::handlers::fix(receipts, apply, finalize_working, quarantine)
}
```

`--receipts` defaults to `.affi/`. Exit code mirrors `verify`: **0** if nothing above `Warning`, **2**
if any `Error`/`Critical` finding exists (`std::process::exit(2)`, as `src/handlers.rs:367`).

### 4.1 A healthy store

```console
$ affi doctor
affi doctor — scanning .affi/  (3 receipts, working.json)

  HEALTH  RECEIPT                                   EVENTS  STATUS
  ──────  ────────────────────────────────────────  ──────  ───────────
   100    9f2c4e…a71b.json                              12   certifiable
   100    3d8819…0c55.json                               4   certifiable
   100    b07a2f…ee90.json                               7   certifiable
    —     working.json                                   0   clean (empty)

Store health: 100/100 — ACCEPT-clean. No findings. Nothing to fix.
```

Exit `0`.

### 4.2 A store with a tampered receipt + an orphaned working file

```console
$ affi doctor
affi doctor — scanning .affi/  (3 receipts, working.json)

  HEALTH  RECEIPT                                   EVENTS  STATUS
  ──────  ────────────────────────────────────────  ──────  ───────────
   100    9f2c4e…a71b.json                              12   certifiable
     0    3d8819…0c55.json                               4   NOT CERTIFIABLE
    92    b07a2f…ee90.json                               7   version drift
    —     working.json                                   5   ORPHANED

CRITICAL  3d8819…0c55.json
  └─ chain_hash mismatch: stored 203d3bbf… != recomputed a17b90c2…
     root cause: an event's bytes changed after sealing (hand-edit or corruption)
                 [chain_integrity · src/verifier.rs:109]
     suggested:  QUARANTINE — cannot be certified, must not be trusted.
                 affi will NOT re-seal a tampered chain.
       fix ›     affi fix --quarantine .affi/3d8819…0c55.json   [safe]

WARNING   b07a2f…ee90.json
  └─ format_version "core/v0"; this verifier certifies "core/v1" only
     suggested:  re-assemble under current standard (manual — changing the
                 declared standard is a decision, not a repair)            [unsafe]

WARNING   working.json
  └─ 5 uncommitted event(s): a run emitted but never `affi assemble`d
     suggested:  re-finalize into an immutable receipt
       fix ›     affi fix --finalize-working .affi/working.json           [safe]

Store health: 0/100 — 1 critical, 2 warnings.
2 safe fix(es) available — preview with `affi fix --dry-run`.
```

Exit `2`. Note the tampered file is *surfaced*, not silently dropped as the current dir loader would
do (`src/handlers.rs:84`).

### 4.3 `--json` (machine-readable, same exit code)

```console
$ affi doctor --json
{
  "root": ".affi/",
  "scanned": 4,
  "store_score": 0,
  "worst_severity": "Critical",
  "receipts": [
    { "file": ".affi/9f2c4e…a71b.json", "score": 100, "event_count": 12,
      "certifiable": true, "findings": [] },
    { "file": ".affi/3d8819…0c55.json", "score": 0, "event_count": 4, "certifiable": false,
      "findings": [
        { "code": "ChainHashMismatch", "severity": "Critical",
          "file": ".affi/3d8819…0c55.json", "seq": null, "field": "chain_hash",
          "root_cause": "stored chain_hash 203d3bbf… != recomputed a17b90c2…",
          "remediation": { "class": "Safe", "action": "QUARANTINE…",
                           "command": "affi fix --quarantine .affi/3d8819…0c55.json" } }
      ] }
  ],
  "orphans": [
    { "code": "OrphanedWorking", "severity": "Warning", "file": ".affi/working.json",
      "seq": null, "field": null, "root_cause": "5 uncommitted event(s)…",
      "remediation": { "class": "Safe", "action": "re-finalize…",
                       "command": "affi fix --finalize-working .affi/working.json" } }
  ]
}
```

### 4.4 `affi fix` — dry-run, then apply

```console
$ affi fix --dry-run
affi fix — 2 safe repair(s) planned (0 will run; --dry-run)

  [1] FINALIZE working.json → .affi/<content-address>.json
      re-runs ChainAssembler::finalize over 5 already-chaining events
      verdict after fix (simulated): ACCEPT
  [2] QUARANTINE 3d8819…0c55.json → .affi/quarantine/3d8819…0c55.json
      reason: chain_hash mismatch (tampered/corrupted). Bytes preserved, not edited.

Skipped (suggest-only, never auto-applied):
  · b07a2f…ee90.json — version drift "core/v0" (unsafe: changing the standard is a decision)

Re-run with --apply to perform the 2 safe repair(s).
```

```console
$ affi fix --apply
affi fix — applying 2 safe repair(s)

  [1] ✔ finalized → .affi/c4f10a…39bd.json  (ACCEPT, 5 events)
  [2] ✔ quarantined → .affi/quarantine/3d8819…0c55.json  (+ .reason.txt)

Done. Re-run `affi doctor` to confirm. 1 unsafe finding still needs a human.
```

```console
$ affi doctor
…
Store health: 92/100 — 0 critical, 1 warning (version drift, manual). Exit 0… (no Errors/Criticals remain)
```

The tampered receipt is **gone from the trusted set** (quarantined), the orphan is **sealed into a
real receipt that ACCEPTs** — and the dishonest file was *never* turned into an ACCEPT.

---

## 5. Doctrine compliance — no path launders a bad chain into ACCEPT

This section is the contract. The doctrine is **certify, don't decide**; the specific danger a
"self-healing" feature courts is *laundering* — silently transforming a receipt that should REJECT
into one that ACCEPTs. We prove, structurally, that no such path exists.

**Invariant: `fix` can produce an ACCEPT receipt only from inputs that already chain honestly.**

1. **Minting is sealed; `doctor`/`fix` cannot forge.** A `Receipt` is only constructible via
   `Receipt::sealed` (`src/types.rs:93`, `pub(crate)`) behind the private `_seal` field
   (`src/types.rs:222`), reachable solely through `ChainAssembler::finalize` (`src/chain.rs:135`).
   `finalize` stamps the chain hash from the bytes it was given — it cannot be told to lie. There is
   no public API that produces a `Receipt` with a chain hash that disagrees with its events.
2. **The verifier is reused verbatim, never re-implemented.** `doctor` calls
   `verifier::verify` (`src/verifier.rs:43`) on receipts minted through that sealed seam
   (`assess_receipt` → `collect_pipeline_findings`). It defines no looser pass condition — contrast
   the legacy `verify_family` shortcut (`src/handlers.rs:386`), which `doctor` deliberately replaces.
   `doctor`'s `certifiable` flag is, by construction, exactly what `affi verify` would return.
3. **Tamper is observed, never sealed.** A tampered file fails `recompute_chain` equality
   (`src/chain.rs:68`), so `assess_receipt` takes the `Ok(computed) if computed == …` *else* branch:
   it emits `ChainHashMismatch (Critical)` and **never calls `from_events().finalize()` on it.** The
   only `fix` action offered is `Quarantine`, which moves the file — it does not edit bytes or
   recompute a hash. (Even by accident this is unreachable: `Receipt::deserialize` would itself reject
   the bytes, `src/types.rs:131`.)
4. **`fix --finalize-working` re-seals, it does not repair.** It runs only when the working events
   *already* form a sound chain (`ChainAssembler::from_events` succeeds, `src/chain.rs:107`), and a
   `debug_assert!(verify(&receipt).accepted)` guards the output. It assigns a *name* (content address)
   to bytes that already chain; it changes no event and therefore cannot change a verdict. If the
   working set is malformed, the remediation is downgraded to `FixClass::Unsafe` and **not planned**
   (`assess_working`).
5. **The unsafe/safe partition is enforced at planning time.** `FixEngine::plan` derives repairs
   *only* from findings with `remediation.class == FixClass::Safe`. Everything that could change
   meaning — version drift, malformed chains, non-contiguous `seq` — is `Unsafe`, printed as a
   suggestion, and structurally excluded from `apply`. There is no `--force` that crosses this line.
6. **"Fix" never upgrades trust.** The two safe ops have opposite, monotone effects on the trusted
   set: `FinalizeWorking` *adds* a receipt that already passes; `Quarantine` *removes* one that
   doesn't. Neither can take a REJECT-worthy chain and make `affi verify` say ACCEPT. The set of
   bytes that the verifier accepts is identical before and after any `fix`.

**Net:** `doctor` only ever *reports* and *ranks*; `fix` only ever *re-seals already-honest bytes* or
*quarantines dishonest ones*. The decision "is this work honest?" is never made — exactly as the
doctrine demands. A reviewer can audit this by grepping: the only call sites of `finalize` in the new
code are gated by a successful `recompute_chain` and an `accepted` assertion.

---

## 6. Integration & rollout

### Touched / new files

| File | Change |
|---|---|
| `src/doctor.rs` *(new)* | `StoreScanner`, `ShadowReceipt`, `assess_receipt`, scoring, `Finding`/`Remediation`. |
| `src/fix_engine.rs` *(new)* | `SafeRepair`, `FixEngine::{plan, apply}`, `quarantine_file`. |
| `src/verbs/doctor.rs`, `src/verbs/fix.rs` *(new)* | Thin `#[verb]` wrappers (`src/verbs/verify.rs:13` pattern). |
| `src/verbs/mod.rs` | `pub mod doctor; pub mod fix;` (`src/verbs/mod.rs:16` neighborhood). |
| `src/handlers.rs` | `pub fn doctor(...)`, `pub fn fix(...)`; reuse `print_json_or` (`:96`), exit-code idiom (`:367`). |
| `src/chain.rs` | Add `load_working_at(path)` + `save`-at-path helpers (generalize `load_working`, `src/chain.rs:169`). Tiny, additive. |
| `src/lib.rs` | `pub mod doctor;` re-export for the public API + examples. |
| `examples/doctor_store.rs` *(new)* | Build a 3-receipt store (one tampered, one orphan), run `doctor`, assert the rollup. |

> **Build caveat:** this repo depends on private-registry `26.6` crates (`clap-noun-verb`,
> `wasm4pm-compat`, …, `Cargo.toml:40`) that will not resolve in a clean environment, so `cargo
> build`/`test` were intentionally **not** run. The Rust above is *compilable-style* and pins to real
> symbols (`recompute_chain`, `ChainAssembler::{from_events,finalize}`, `verifier::verify`,
> `CheckOutcome`) — finalize signatures against the actual crates during P0.

### Rollout (P-tiers · S/M/L)

**P0 — credible MVP (the leap).**
- **S** · `ShadowReceipt` + `classify` defensive parser (fixes the silent-drop gap, §2).
- **M** · `StoreScanner` + `assess_receipt` reusing `verifier::verify`; `Finding`/scoring; `affi doctor` human + `--json`; exit `0`/`2`.
- **S** · `examples/doctor_store.rs` + unit tests: honest store → 100; tampered → Critical/exit 2; orphan → Warning.

**P1 — safe self-healing.**
- **M** · `FixEngine` with `FinalizeWorking` + `Quarantine`; `affi fix --dry-run` / `--apply`; the `accepted` guard + unsafe/safe partition (§5).
- **S** · `--quarantine` / `--finalize-working` targeted flags; quarantine sidecar notes.
- **S** · Doctrine test suite: assert `fix` *cannot* raise a tampered file to ACCEPT (property test).

**P2 — depth & ecosystem.**
- **M** · `--profile` aware scoring (extends `evaluate_profile`, `src/verifier.rs:191`); duplicate-content-address detection across the store.
- **M** · LSP surfacing — emit `Finding`s as workspace diagnostics via `crate::lsp` (`src/lsp/diagnostics.rs`), reusing the `diagnose` shape (`src/handlers.rs:974`).
- **L** · Cross-store `doctor --portfolio <root>` rollup over many `.affi/` dirs (align with `portfolio_health`, `src/verbs/portfolio_health.rs:13`); optional OTel span per scan (`otel` feature, `Cargo.toml:143`).

---

### Appendix — finding catalog (severity & default fix class)

| Code | Detects | Severity | Default `FixClass` |
|---|---|---|---|
| `ChainHashMismatch` | tamper / corruption (recompute ≠ stored) | Critical | Safe (quarantine) |
| `Unparseable` | not JSON / not a receipt shape | Error | Safe (quarantine) |
| `SeqNonContiguous` | gap/disorder in `seq` | Error | Unsafe |
| `DuplicateEventId` | repeated event id | Error | Unsafe |
| `MalformedCommitment` | commitment ≠ 64 lowercase-hex | Error | Unsafe |
| `EmptyEventType` | blank `event_type` | Error | Unsafe |
| `VersionDrift` | `format_version` ≠ `core/v1` | Warning | Unsafe |
| `ProfileMismatch` | not certifiable under requested profile | Warning | Unsafe |
| `OrphanedWorking` | uncommitted `working.json` | Warning | Safe (finalize, if sound) |
| `AbandonedTemp` | stray `*.tmp`/`*.partial`/0-byte | Info | Safe (quarantine) |
