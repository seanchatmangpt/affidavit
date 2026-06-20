# W10 — Compliance & Governance

**Workstream:** W10 (Compliance & Governance) · **Owner role:** compliance-evidence + governance layer
**Status:** design / proposal · **Horizon:** 2026 H2 → 2030
**Doctrine:** *certify, don't decide.* A compliance check certifies that receipts **match a named
framework's evidentiary format**; it never claims the organization is "truly compliant" or the work
"honest."

> **Build caveat:** private `26.6` deps (`clap-noun-verb`, `clnrm-core`, `wasm4pm`, `lsp-max`) do not
> resolve in a lone checkout. Nothing here was `cargo build`/`test`-verified. All Rust is
> compilable-*style* — correct against in-tree patterns, pending signature finalization.

---

## 1. Mission, Scope & Boundaries

### Mission
Turn provenance receipts (BLAKE3-chained, 7-stage-verified) into **audit-ready, attestable
compliance evidence**. W10 maps the *structural facts a verified receipt already proves* onto the
*evidentiary requirements of named control frameworks*, runs **policy-as-code** deterministically
over receipts/chains, governs the evidence (retention, redaction-with-proof, access/audit trails),
and exports **auditor-ready artifacts** — all without ever minting a legal or moral compliance
judgment.

### In scope (W10 owns)
- **Compliance evidence verbs:** `soc2_audit`, `hipaa`, `pci_dss`, `gdpr_proof`,
  `license_compliance`, `verify_compliance` (handlers.rs:1785, 1871, 1903, 1833, 1942, 471).
- **Policy-as-code engine:** `policy_enforce` (handlers.rs:2000) → declarative, deterministic rules
  over receipts and chains. Today hardcoded; W10 makes it a real rule language + evaluator.
- **Control/Evidence mapping model:** a registry that ties framework *controls* to *receipt
  predicates* (what structural fact, in which event, satisfies which evidentiary requirement).
- **Governance:** retention windows, redaction-with-proof (prove a payload existed and was removed
  without disclosing it), and access/audit trails over who ran which compliance export.
- **Audit-report generation:** export control-coverage matrices, evidence catalogs, and an
  **evidence pack** (machine + human format) suitable for handing to an auditor.

### Out of scope (boundaries — reference, do not build)
| Concern | Owner | W10 relationship |
|---|---|---|
| Cryptographic **signing** of reports/evidence packs | **W8** (Crypto & Trust) | W10 *consumes* a `Signer` trait; the `attest`/`sign`/`notarize` stubs (handlers.rs:567, 641, 612) and `assemble_with_signature` (handlers.rs:295) become W8-backed. W10 never rolls its own crypto. |
| Raw **standards ingestion** (SBOM CycloneDX/SPDX, OCEL) | **W9** (Ecosystem & Standards) | `sbom_compliance` (handlers.rs:3698 → `crate::sbom_compliance::assess_all`) belongs to W9. W10 *imports* its `FrameworkResult` as one evidence source, but owns the framework→control mapping. |
| The 7-stage **verifier** itself, profiles, conformance metrics | **W7** (Verification Engine) | W10 *runs* `crate::verifier::verify` (verifier.rs:43) and reads its `Verdict`; W10 never re-implements chain integrity. New "compliance profiles" are proposed to W7, not forked. |
| Output/diagnostics contract, exit-code catalog | **W3** (CLI Ergonomics) | W10 emits via the shared `Out` handle and stable codes once they exist (see §2 gap B2/B6). |

**The line:** W7 proves the receipt is *well-formed and untampered*; W9 ingests *external standards*;
W8 *signs*. W10 owns the **interpretation layer** — "given a verified receipt, what evidence does it
constitute for which control, and how is that evidence governed and exported."

---

## 2. Current State (grounded) + Gap

### What the compliance verbs actually do today

Every compliance verb is a thin `#[verb(...)]` wrapper (e.g. soc2_audit.rs:13, verify_compliance.rs:13,
policy_enforce.rs:13) delegating to `crate::handlers::*`. The real bodies:

- **`verify_compliance`** (handlers.rs:471): runs `crate::cli::verify` (the real pipeline), then
  attaches a **hardcoded `match framework`** table of `(check_name, bool, note)` tuples where the
  bool is almost always just `verdict.accepted` or `!verdict.outcomes.is_empty()` (handlers.rs:475-529).
  Unknown frameworks collapse to a single `generic-check` (handlers.rs:524). On failure it
  `std::process::exit(2)` (handlers.rs:562).
- **`soc2_audit`** (handlers.rs:1785): emits a JSON report with a fixed `trust_service_criteria`
  block and a per-receipt `"integrity_status": "chain-verified"` that is a **string literal, not a
  recomputed result** (handlers.rs:1798). `soc2_type` defaults to `"II"` (handlers.rs:1792).
- **`hipaa`** (handlers.rs:1871) / **`pci_dss`** (handlers.rs:1903) / **`gdpr_proof`**
  (handlers.rs:1833): emit fixed `safeguards` / `requirements` / `evidence` blocks. `pci_dss` does
  one real derivation — counting `deploy`-typed events (handlers.rs:1906-1910). The rest are static
  prose keyed only on `receipts.len()` and `events.len()`.
- **`license_compliance`** (handlers.rs:1942): loads an `allowed_licenses` policy, extracts events
  whose `event_type.contains("license")` (handlers.rs:1963), but **never compares** found licenses
  against the allow-list — `status` is just `"policy loaded — license events extracted"`
  (handlers.rs:1980).
- **`policy_enforce`** (handlers.rs:2000): the closest to real policy-as-code. Reads `min_approvals`
  and `require_security_scan` from a JSON file (handlers.rs:2010-2011), counts events whose type
  `contains("approve"|"review")` and any `contains("security"|"scan")` (handlers.rs:2019-2024),
  records violations, `exit(2)` on any (handlers.rs:2068). The rule vocabulary is **two hardcoded
  keys**; there is no rule language.

The honest workhorse underneath is **W7's verifier** (verifier.rs:43-213): seven decidable stages,
pure over receipt bytes, explicitly *"never decides whether the underlying code was honest"*
(verifier.rs:8-11). Its output type is `Verdict { accepted, profile, outcomes: Vec<CheckOutcome>,
reason }` (types.rs:293-302).

### The gap (and the doctrine hazards)

| # | Gap | Evidence | Severity |
|---|---|---|---|
| **G1** | **Overclaiming language.** `soc2_audit` emits `"opinion": "These receipts constitute a sufficient audit trail for SOC 2 certification."` — an opinion the tool must not render. | handlers.rs:1814 | **Doctrine-critical** |
| **G2** | **"COMPLIANT" verdict.** `verify_compliance` prints `COMPLIANT` / `NON-COMPLIANT` and exits 2, framing a structural match as an org-level compliance judgment. | handlers.rs:550-557, 562 | **Doctrine-critical** |
| **G3** | **No Control/Evidence model.** Frameworks are inline `match` arms; controls aren't first-class, aren't versioned, and can't be added without editing Rust. | handlers.rs:475-529 | High |
| **G4** | **Policy-as-code is two hardcoded keys.** No rule type, no operators, no evaluator; can't express "every `deploy` event must be preceded by an `approve` event by a different actor." | handlers.rs:2010-2038 | High |
| **G5** | **Evidence is asserted, not derived.** `"integrity_status":"chain-verified"` and the safeguard blocks are string literals not tied to a fresh `verify()` call. | handlers.rs:1798, 1877-1881 | High |
| **G6** | **`license_compliance` never enforces.** Allow-list loaded but unused. | handlers.rs:1952-1980 | Med |
| **G7** | **No governance.** No retention, no redaction-with-proof, no access/audit trail over who exported what. | (absent) | High |
| **G8** | **Reports are unsigned & unattested.** Exports are bare JSON; no W8 signature, no provenance receipt *of the audit itself*. | handlers.rs:1817-1827 | Med |
| **G9** | **Hand-built JSON.** Several outputs use `format!`-interpolated JSON (e.g. handlers.rs:304, 325) → injectable/invalid for values with quotes. Inherits synthesis bug **B2**. | handlers.rs:304, 325 | Med |
| **G10** | **No continuous compliance.** Everything is one-shot CLI; no drift detection, no scheduled re-evaluation, no posture history. | (absent) | Med (2028+) |

---

## 3. Phased Plan (2026 H2 → 2030)

Design rule for every phase: **certify the receipt-to-framework structural match; report evidence
presence/absence; never emit an org-level verdict.** Output nouns are always *"evidence present for
control X"*, *"requirement R has no witnessing event"* — never *"you are compliant."*

### 2026 H2 — Doctrine correction + Control/Evidence model (foundation)

**Objectives**
1. Stop overclaiming **now** (G1, G2) — a small, self-contained language PR.
2. Introduce a first-class **Control/Evidence mapping model** so frameworks become data, not `match`
   arms (G3), with evidence **derived from a fresh `verify()`** (G5).
3. Establish W10's vocabulary in `src/compliance/mod.rs` (new module; does **not** modify existing
   verb bodies — the verbs will *opt in* by re-pointing their handlers in a later, owner-coordinated
   step, but the model and evaluator land standalone first).

**Deliverables**
- `src/compliance/model.rs` — `Framework`, `Control`, `EvidenceRequirement`, `ReceiptPredicate`.
- `src/compliance/evidence.rs` — the evaluator that maps a verified receipt to per-control
  `EvidenceStatus` (`Present` / `Absent` / `NotApplicable` / `Indeterminate`).
- A **doctrine linter** for compliance output (`forbidden_claims()`), unit-tested against the strings
  in handlers.rs:1814 and handlers.rs:551.

```rust
// src/compliance/model.rs  — Control/Evidence mapping (compilable-style)
use serde::{Deserialize, Serialize};

/// A named control framework *as an evidentiary format*, never as a legal status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    pub id: String,            // "soc2", "hipaa", "pci-dss", "gdpr"
    pub version: String,       // "2017-TSC", "2022-v4.0" — frameworks are versioned data
    pub controls: Vec<Control>,
}

/// One control / requirement within a framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: String,            // "CC6.1", "Req-10", "Art.30"
    pub title: String,
    /// What receipt fact, if present, *witnesses* evidence for this control.
    pub requires: Vec<EvidenceRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub label: String,         // "change events recorded", "access log present"
    pub predicate: ReceiptPredicate,
}

/// A *decidable* predicate over a verified receipt. No predicate may encode intent,
/// honesty, or a legal conclusion — only structural facts the chain already proves.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReceiptPredicate {
    /// The W7 verifier accepted the receipt (chain integrity holds).
    ChainAccepted,
    /// At least `min` events whose type matches `pattern` exist.
    EventTypePresent { pattern: String, min: usize },
    /// Every event matching `cause` is preceded (lower seq) by one matching `effect`.
    OrderedBefore { cause: String, effect: String },
    /// An object of `obj_type` is referenced by some event.
    ObjectReferenced { obj_type: String },
}

/// Tri-/quad-state evidence status. NOTE: there is deliberately **no** `Compliant`
/// variant — the tool reports evidence, the auditor renders compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Present,        // a witnessing receipt fact exists
    Absent,         // requirement has no witness in these receipts
    NotApplicable,  // requirement does not apply to this receipt set
    Indeterminate,  // cannot be decided from receipts alone (e.g. needs out-of-band proof)
}
```

```rust
// src/compliance/evidence.rs  — derive evidence, never decide compliance
use crate::compliance::model::*;
use crate::types::Receipt;
use crate::verifier;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ControlEvidence {
    pub control_id: String,
    pub status: EvidenceStatus,
    pub witnesses: Vec<String>,   // event ids / chain hashes that witness it
    pub note: String,             // evidentiary language only — see doctrine_guard()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EvidenceReport {
    pub framework_id: String,
    pub framework_version: String,
    pub receipts_examined: usize,
    pub controls: Vec<ControlEvidence>,
    /// Mandatory disclaimer embedded in every report (see §4 doctrine).
    pub disclaimer: &'static str,
}

const DISCLAIMER: &str =
    "This report certifies whether the supplied receipts contain evidence matching the named \
     framework's evidentiary format. It is NOT a determination of legal or regulatory compliance, \
     nor a judgment that the underlying work is correct or honest. Compliance determinations are \
     made by a qualified auditor.";

pub fn evaluate(fw: &Framework, receipts: &[Receipt]) -> EvidenceReport {
    let controls = fw
        .controls
        .iter()
        .map(|c| evaluate_control(c, receipts))
        .collect();
    EvidenceReport {
        framework_id: fw.id.clone(),
        framework_version: fw.version.clone(),
        receipts_examined: receipts.len(),
        controls,
        disclaimer: DISCLAIMER,
    }
}

fn evaluate_control(control: &Control, receipts: &[Receipt]) -> ControlEvidence {
    let mut witnesses = Vec::new();
    let mut all_present = true;
    for req in &control.requires {
        match witness(&req.predicate, receipts) {
            Some(mut w) => witnesses.append(&mut w),
            None => all_present = false,
        }
    }
    ControlEvidence {
        control_id: control.id.clone(),
        status: if all_present { EvidenceStatus::Present } else { EvidenceStatus::Absent },
        witnesses,
        note: format!("evidence {} for control {}",
            if all_present { "present" } else { "not found" }, control.id),
    }
}

/// Returns the witnessing event ids if the predicate holds; None otherwise.
/// Every arm reduces to a decidable structural fact — no arm judges honesty.
fn witness(p: &ReceiptPredicate, receipts: &[Receipt]) -> Option<Vec<String>> {
    match p {
        ReceiptPredicate::ChainAccepted => {
            let all_ok = receipts.iter().all(|r| verifier::verify(r).accepted);
            all_ok.then(|| receipts.iter().map(|r| r.chain_hash.0.clone()).collect())
        }
        ReceiptPredicate::EventTypePresent { pattern, min } => {
            let hits: Vec<String> = receipts.iter().flat_map(|r| &r.events)
                .filter(|e| e.event_type.contains(pattern.as_str()))
                .map(|e| e.id.clone()).collect();
            (hits.len() >= *min).then_some(hits)
        }
        ReceiptPredicate::OrderedBefore { cause, effect } => {
            // every `cause` event has some `effect` at strictly lower seq in its receipt
            let ok = receipts.iter().all(|r| r.events.iter()
                .filter(|e| e.event_type.contains(cause.as_str()))
                .all(|c| r.events.iter()
                    .any(|e| e.event_type.contains(effect.as_str()) && e.seq < c.seq)));
            ok.then(Vec::new)
        }
        ReceiptPredicate::ObjectReferenced { obj_type } => {
            let hits: Vec<String> = receipts.iter().flat_map(|r| &r.events)
                .filter(|e| e.objects.iter().any(|o| &o.obj_type == obj_type))
                .map(|e| e.id.clone()).collect();
            (!hits.is_empty()).then_some(hits)
        }
    }
}

/// Doctrine guard: reject any output string that renders a legal/honesty verdict.
pub fn doctrine_guard(s: &str) -> Result<(), &'static str> {
    const FORBIDDEN: &[&str] = &[
        "is compliant", "are compliant", "fully compliant", "sufficient audit trail",
        "certification.", "guarantees compliance", "is honest", "passes audit",
    ];
    let lc = s.to_lowercase();
    if FORBIDDEN.iter().any(|f| lc.contains(f)) {
        return Err("output renders a compliance/honesty verdict — forbidden by doctrine");
    }
    Ok(())
}
```

**Acceptance criteria (2026 H2)**
- `doctrine_guard` rejects the exact current strings at handlers.rs:1814 ("sufficient audit trail",
  "certification.") and the `COMPLIANT` framing at handlers.rs:551 — proven by unit test.
- `evaluate()` produces `EvidenceStatus::{Present,Absent}` derived from a live `verifier::verify`
  call (closes G5 for the new path) — no string-literal `"chain-verified"`.
- A builtin `soc2` / `hipaa` / `pci-dss` / `gdpr` `Framework` is expressible **as data** (TOML/JSON
  fixtures under `fixtures/frameworks/`), not Rust `match` arms.
- New module compiles in isolation (compilable-style); zero edits to existing files.

**Cross-workstream deps:** W3 (output contract for the report; until then, emit via existing
`print_json_or`). W7 (consumes `Verdict`; no changes requested yet).

---

### 2027 — Policy-as-code engine + enforced license/governance v0

**Objectives**
1. Replace `policy_enforce`'s two hardcoded keys (G4) with a real **declarative rule type +
   deterministic evaluator** reusing `ReceiptPredicate`.
2. Make `license_compliance` actually enforce the allow-list (G6).
3. Governance **v0:** retention windows + an access/audit trail of who ran which export (G7, start).

**Deliverables**
- `src/compliance/policy.rs` — `PolicyDocument`, `Rule`, `RuleOutcome`, `evaluate_policy`.
- License allow-list enforcement folded into the evidence model as a predicate.
- `src/compliance/governance.rs` (v0) — `RetentionPolicy`, `AccessLogEntry` (emitted as ordinary
  affidavit events so the audit trail is itself a verifiable chain).

```rust
// src/compliance/policy.rs  — declarative policy-as-code (compilable-style)
use crate::compliance::model::ReceiptPredicate;
use crate::compliance::evidence::witness;
use crate::types::Receipt;
use serde::{Deserialize, Serialize};

/// A declarative, deterministic policy. Evaluating it NEVER adjudicates intent —
/// it reports which rules are satisfied by the receipt facts and which are not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDocument {
    pub id: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub description: String,
    /// Reuse the evidence predicates; a rule "holds" iff its predicate witnesses.
    pub when: ReceiptPredicate,
    /// Severity is advisory metadata for the *report*, not a decision.
    #[serde(default)]
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity { #[default] Info, Low, Medium, High }

#[derive(Debug, Clone, Serialize)]
pub struct RuleOutcome {
    pub rule_id: String,
    pub satisfied: bool,
    pub severity: Severity,
    pub witnesses: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyReport {
    pub policy_id: String,
    pub receipts_examined: usize,
    pub outcomes: Vec<RuleOutcome>,
    /// Count of unsatisfied rules — a *fact*, surfaced for the operator/auditor,
    /// not a "non-compliant" verdict.
    pub unsatisfied: usize,
    pub disclaimer: &'static str,
}

const POLICY_DISCLAIMER: &str =
    "Each rule outcome is a deterministic structural check over receipt evidence. \
     An unsatisfied rule means 'no witnessing event was found in these receipts', \
     not that a policy was wilfully violated or that the org is non-compliant.";

pub fn evaluate_policy(policy: &PolicyDocument, receipts: &[Receipt]) -> PolicyReport {
    let outcomes: Vec<RuleOutcome> = policy.rules.iter().map(|rule| {
        let w = witness(&rule.when, receipts);
        let satisfied = w.is_some();
        RuleOutcome {
            rule_id: rule.id.clone(),
            satisfied,
            severity: rule.severity,
            witnesses: w.unwrap_or_default(),
            detail: if satisfied { "witnessing evidence found".into() }
                    else { "no witnessing evidence in supplied receipts".into() },
        }
    }).collect();
    let unsatisfied = outcomes.iter().filter(|o| !o.satisfied).count();
    PolicyReport {
        policy_id: policy.id.clone(),
        receipts_examined: receipts.len(),
        outcomes,
        unsatisfied,
        disclaimer: POLICY_DISCLAIMER,
    }
}
```
This expresses the segregation-of-duties intent the verb's docstring promises (policy_enforce.rs:12,
"2-person approval, segregation of duties") via `ReceiptPredicate::OrderedBefore { cause: "deploy",
effect: "approve" }` plus `EventTypePresent { pattern: "approve", min: 2 }` — declaratively, not as
two inline `if` blocks (handlers.rs:2026-2038).

**Governance v0 — the audit trail is itself a chain:**
```rust
// src/compliance/governance.rs (v0 sketch)
#[derive(Debug, Clone, serde::Serialize)]
pub struct RetentionPolicy { pub min_events_retained: usize, pub label: String }

/// Recorded by emitting a normal affidavit event ("audit.export") so that *who
/// exported which evidence* is provable by the same verifier as everything else.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AccessLogEntry { pub actor: String, pub action: String, pub target_chain_hash: String }
```

**Acceptance criteria (2027)**
- `evaluate_policy` reproduces today's `min_approvals` / `require_security_scan` behavior
  (handlers.rs:2010-2038) from a *declarative* `PolicyDocument`, plus at least one ordering rule the
  old code could not express.
- No `std::process::exit(2)` inside the evaluator; exit handling is the verb layer's concern via the
  W3 exit-code catalog (decouples decision from process control — see G2).
- `license_compliance` flags an event referencing a license **not** in `allowed_licenses` (closes G6).
- Every `affi`-driven compliance export appends an `audit.export` event (governance v0).
- `PolicyReport`/`EvidenceReport` JSON is built with `serde_json` value construction, never `format!`
  (closes G9 for W10 outputs).

**Cross-workstream deps:** W3 (exit-code catalog, `Out`). W9 (`license_compliance` should consume
W9's SBOM license facts where an SBOM is present, rather than only `event_type.contains("license")`).

---

### 2028 — Audit-report exporter + redaction-with-proof + signed evidence packs

**Objectives**
1. **Auditor-ready exporter:** a control-coverage matrix + evidence catalog in machine (JSON) and
   human (Markdown) form — the "evidence pack" (closes G3 end-to-end, G8).
2. **Redaction-with-proof** (G7): prove a payload existed and was redacted *without disclosing it*,
   using the existing BLAKE3 commitment as the proof anchor.
3. **Sign** the evidence pack via **W8** (G8).

**Deliverables**
- `src/compliance/report.rs` — `AuditReportExporter` with pluggable `ReportFormat`.
- `src/compliance/redaction.rs` — `Redaction`, `RedactionProof` (commitment-preserving).
- W8 integration point: `sign_pack(pack, &dyn Signer)` consuming W8's signer (no crypto in W10).

```rust
// src/compliance/report.rs  — auditor-ready exporter (compilable-style)
use crate::compliance::evidence::EvidenceReport;
use crate::compliance::policy::PolicyReport;

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat { Json, Markdown }

/// A bundle handed to an auditor: framework evidence + policy outcomes + the
/// receipts' own chain hashes as provenance anchors. Carries the disclaimer.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvidencePack {
    pub generated_for: String,            // framework id(s)
    pub evidence: Vec<EvidenceReport>,
    pub policy: Vec<PolicyReport>,
    pub source_chain_hashes: Vec<String>, // provenance of the evidence itself
    pub disclaimer: &'static str,
    /// Filled by W8 after signing; None until then. W10 never signs.
    pub signature: Option<String>,
}

pub struct AuditReportExporter;

impl AuditReportExporter {
    /// Render the pack. The exporter is *format-only*: it never re-derives a
    /// pass/fail beyond echoing per-control evidence presence.
    pub fn render(pack: &EvidencePack, fmt: ReportFormat) -> Result<String, serde_json::Error> {
        match fmt {
            ReportFormat::Json => serde_json::to_string_pretty(pack),
            ReportFormat::Markdown => Ok(Self::to_markdown(pack)),
        }
    }

    fn to_markdown(pack: &EvidencePack) -> String {
        // Renders: disclaimer header (structurally unavoidable), then a
        // "Control Coverage Matrix" table | Control | Evidence | Witnesses |.
        // Cell language is evidentiary ("present"/"absent"), never "compliant".
        // ...elided...
        String::new()
    }
}
```

```rust
// src/compliance/redaction.rs  — redaction-with-proof (commitment-preserving)
use crate::types::Blake3Hash;

/// A redacted payload: the *commitment is preserved* so the verifier still proves
/// "an event with this exact payload existed", while the bytes are gone. This lets
/// GDPR erasure (gdpr_proof, handlers.rs:1833) be demonstrated without re-exposing PII.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RedactionProof {
    pub event_id: String,
    pub preserved_commitment: Blake3Hash, // == original event.payload_commitment
    pub redaction_reason: String,         // "gdpr-art17-erasure", "pci-pan-truncation"
    pub redacted_at_seq: u64,             // ordering anchor, not wall-clock
}

impl RedactionProof {
    /// The proof is valid iff the preserved commitment still matches the chain's
    /// recorded commitment for that event — i.e. redaction did not alter history.
    pub fn verifies_against(&self, recorded: &Blake3Hash) -> bool {
        &self.preserved_commitment == recorded
    }
}
```

**Acceptance criteria (2028)**
- `affi receipt audit-report --framework soc2 --format markdown` (new verb, W10-owned) emits a
  control-coverage matrix whose every cell says `Present`/`Absent`/`NotApplicable` — **no cell and no
  header says "compliant."** Enforced by routing all rendered strings through `doctrine_guard`.
- A `RedactionProof` round-trips: redact a payload, then `verifies_against` the chain's recorded
  commitment returns `true`; the redacted bytes are unrecoverable from the pack.
- An `EvidencePack` can be signed by a W8 `Signer` and the signature field populated; W10 contains
  **zero** signing primitives (boundary held).
- Pack `disclaimer` is present in **every** rendered format (JSON + Markdown), structurally
  (compile-time field, not optional).

**Cross-workstream deps:** **W8** (signer trait — the `sign`/`attest`/`notarize` stubs at
handlers.rs:641/567/612 become W8-backed; W10 calls them). W9 (pull SBOM/OCEL-derived control facts
into the pack as additional evidence sources).

---

### 2029 — Continuous compliance + multi-framework + posture history

**Objectives**
1. **Continuous compliance** (G10): drift detection — re-evaluate evidence when the receipt store
   changes, record posture-over-time as its own receipt chain.
2. **Multi-framework coverage:** one pass over a receipt set evaluates N frameworks and reports
   **shared-control** reuse (e.g. SOC2 CC7.2 ≈ HIPAA audit-controls ≈ PCI Req-10).
3. **Crosswalk model:** map equivalent controls across frameworks so one witnessing event satisfies
   many requirements without duplicate work.

**Deliverables**
- `src/compliance/continuous.rs` — `PostureSnapshot`, `drift(prev, next)`; integrates with W5's
  watch/automation to trigger re-evaluation.
- `src/compliance/crosswalk.rs` — `ControlCrosswalk { left, right, equivalence }`.
- Multi-framework `EvidencePack` (Vec of `EvidenceReport`, already shaped for it in §2028).

```rust
// src/compliance/continuous.rs  — drift as a fact, not an alarm (compilable-style)
use crate::compliance::evidence::{EvidenceReport, EvidenceStatus};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PostureSnapshot {
    pub framework_id: String,
    pub present: usize,
    pub absent: usize,
    pub source_chain_hashes: Vec<String>, // anchors this snapshot to provenance
}

impl PostureSnapshot {
    pub fn of(report: &EvidenceReport) -> Self {
        let present = report.controls.iter()
            .filter(|c| c.status == EvidenceStatus::Present).count();
        PostureSnapshot {
            framework_id: report.framework_id.clone(),
            present,
            absent: report.controls.len() - present,
            source_chain_hashes: vec![], // filled from the evaluated receipts
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PostureDrift {
    pub framework_id: String,
    pub newly_absent: Vec<String>,  // controls that lost their witness
    pub newly_present: Vec<String>, // controls that gained one
}

/// Drift reports *which control evidence appeared/disappeared* between snapshots.
/// It does not say posture "improved" or "degraded" — that is the auditor's read.
pub fn drift(prev: &EvidenceReport, next: &EvidenceReport) -> PostureDrift {
    let was_present = |r: &EvidenceReport, id: &str| r.controls.iter()
        .any(|c| c.control_id == id && c.status == EvidenceStatus::Present);
    let newly_absent = next.controls.iter()
        .filter(|c| was_present(prev, &c.control_id) && c.status != EvidenceStatus::Present)
        .map(|c| c.control_id.clone()).collect();
    let newly_present = next.controls.iter()
        .filter(|c| !was_present(prev, &c.control_id) && c.status == EvidenceStatus::Present)
        .map(|c| c.control_id.clone()).collect();
    PostureDrift { framework_id: next.framework_id.clone(), newly_absent, newly_present }
}
```

**Acceptance criteria (2029)**
- Given two receipt-store states, `drift` reports the exact set of controls whose *evidence presence*
  changed; output uses `newly_present` / `newly_absent`, never "improved"/"regressed."
- A single `affi receipt audit-report --framework soc2,hipaa,pci-dss` evaluates all three in one pass;
  a `ControlCrosswalk` lets one `audit-log`-type witness satisfy the equivalent control in each.
- Posture snapshots are themselves emitted as a receipt chain (continuous trail is verifiable by W7).
- Drift re-evaluation can be triggered by W5's watcher without W10 owning the scheduler.

**Cross-workstream deps:** **W5** (Workflow Automation — watch/trigger re-evaluation). W9 (crosswalk
benefits from W9's canonical control catalogs / standard mappings).

---

### 2030 — Attestable continuous-compliance platform (DoD horizon)

**Objectives**
1. **End-to-end attestable evidence:** every evidence pack is signed (W8), provenance-anchored, and
   itself re-emitted as a receipt — *the audit of the audit is verifiable.*
2. **Framework registry as data + plugin:** new frameworks/controls added via `linkme`-discovered
   registrations (the synthesis flags `linkme` as declared-but-unused, Cargo.toml:42 / B10) — no Rust
   edits to onboard a framework.
3. **Governance complete:** retention enforcement, redaction-with-proof, and full access/audit trails
   are first-class and exportable.

**Deliverables**
- `src/compliance/registry.rs` — `#[linkme::distributed_slice(FRAMEWORKS)]` discovery for frameworks
  and crosswalks.
- Hardened `audit-report` / `policy_enforce` / `verify_compliance` verbs all routing through the
  evidence model + `doctrine_guard`, emitting signed packs.
- A conformance suite proving the doctrine invariant across **all** compliance outputs.

```rust
// src/compliance/registry.rs  — frameworks as discoverable data (compilable-style)
use crate::compliance::model::Framework;

#[linkme::distributed_slice]
pub static FRAMEWORKS: [fn() -> Framework] = [..];

/// Resolve a framework by id from the registry (data-driven, no `match` arms).
pub fn resolve(id: &str) -> Option<Framework> {
    FRAMEWORKS.iter().map(|f| f()).find(|fw| fw.id == id)
}

// A framework definition registers itself; onboarding ISO 27001 etc. is data, not code edits.
#[linkme::distributed_slice(FRAMEWORKS)]
static SOC2_2017: fn() -> Framework = || Framework {
    id: "soc2".into(), version: "2017-TSC".into(),
    controls: crate::compliance::builtin::soc2_controls(),
};
```

**Definition of Done @2030 (see §4).**

**Cross-workstream deps:** **W8** (mandatory signing of every pack), **W9** (standard ingestion +
canonical control catalogs), **W7** (any new "compliance profile" lands in the verifier with W10
consuming it), W3/W4 (registry & ergonomics), W5 (continuous triggers).

---

## 4. Definition of Done @2030

W10 is **done** when all of the following hold:

1. **No overclaiming, structurally guaranteed.** Every compliance output (`soc2_audit`, `hipaa`,
   `pci_dss`, `gdpr_proof`, `license_compliance`, `verify_compliance`, `policy_enforce`,
   `audit-report`) passes `doctrine_guard`; the strings at handlers.rs:1814 and handlers.rs:551 are
   gone. Reports speak only of **evidence present/absent/indeterminate** and carry a mandatory
   disclaimer that the legal determination is the auditor's.
2. **Frameworks are data, not code.** Controls live in a `linkme`-discovered registry +
   fixtures; onboarding a new framework (ISO 27001, NIST 800-53, FedRAMP) requires **no edit to
   handler `match` arms**.
3. **Policy-as-code is a real, deterministic language.** `policy_enforce` evaluates a declarative
   `PolicyDocument` (predicates incl. ordering/segregation-of-duties), reproducibly, with no
   in-evaluator `process::exit` and no intent adjudication.
4. **Evidence is derived, not asserted.** Every control status traces to a live `verifier::verify`
   result and named witnessing events — no string-literal "chain-verified."
5. **Governance is first-class.** Retention windows, redaction-with-proof (commitment-preserving),
   and access/audit trails exist; every export emits an `audit.export` receipt event.
6. **Packs are attestable.** Evidence packs are signed via W8 and re-emitted as receipts — the audit
   trail of the audit is itself W7-verifiable. W10 holds **zero** crypto primitives.
7. **Continuous + multi-framework.** One pass evaluates N frameworks with crosswalk reuse; drift
   detection records posture history as a verifiable chain, triggerable by W5.
8. **Deterministic & pure.** Same receipts + same framework/policy ⇒ byte-identical report
   (matches the project's determinism doctrine; no wall-clock in evidence logic — ordering uses
   `seq` per types.rs:189).

---

## 5. Cross-Workstream Dependencies (summary)

| Dep | Direction | What W10 needs / gives |
|---|---|---|
| **W7** Verification Engine | W10 **consumes** | Runs `verifier::verify` (verifier.rs:43); reads `Verdict`/`CheckOutcome` (types.rs:270-302). New "compliance profiles" are *proposed to* W7, never forked. |
| **W8** Crypto & Trust | W10 **consumes** | A `Signer` to sign evidence packs / attestations. The `sign`/`attest`/`notarize`/`assemble_with_signature` stubs (handlers.rs:641/567/612/295) become W8-backed. W10 never signs. |
| **W9** Ecosystem & Standards | W10 **consumes** | SBOM/OCEL ingestion + `sbom_compliance::assess_all` (handlers.rs:3698); W10 imports `FrameworkResult` as one evidence source and owns the framework→control mapping + canonical control catalogs/crosswalks. |
| **W3** CLI Ergonomics | W10 **consumes** | Shared `Out` handle + stable exit-code catalog; removes ad-hoc `process::exit(2)` (handlers.rs:562, 2068) and `format!`-JSON (handlers.rs:304, 325 → synthesis B2/B6). |
| **W5** Workflow Automation | W10 **consumes** | Watch/scheduler to trigger continuous re-evaluation (2029+); W10 owns the evidence logic, not the trigger. |
| **W4** Onboarding/Registry | bidirectional | W10's framework registry aligns with the verb/source-of-truth registry; compliance verbs grouped/discoverable. |
| **W1** Foundations | W10 **consumes** | Correctness PR (synthesis B1-B4) underpins trustworthy evidence; `load_receipts_from_path` silent tamper-drop (handlers.rs:84, B3) must be fixed before evidence is trustworthy. |

---

## Doctrine — *certify, don't decide* (the sharpest tension in the program)

Compliance is where over-claiming is most tempting and most dangerous. W10's non-negotiable stance:

- **A compliance check certifies a *structural match to a framework's evidentiary format* — nothing
  more.** It answers "do these receipts contain evidence shaped like what control X requires?" It
  does **not** answer "is the organization compliant?" or "is the work honest?" Those are
  legal/auditor judgments the tool must refuse to render. This mirrors W7's own self-description
  (verifier.rs:8-11: the pipeline *"never decides whether the underlying code was honest"*).

- **Output language is evidentiary, never conclusive.** "evidence present for control CC6.1",
  "requirement Req-10 has no witnessing event", "indeterminate from receipts alone" — **never** "you
  are SOC 2 compliant", "fully compliant", or the current `soc2_audit` opinion that receipts
  *"constitute a sufficient audit trail for SOC 2 certification"* (handlers.rs:1814) or
  `verify_compliance`'s `COMPLIANT`/`NON-COMPLIANT` (handlers.rs:550-557). The `EvidenceStatus` enum
  deliberately has **no `Compliant` variant**, and `doctrine_guard` (§3, 2026 H2) blocks the forbidden
  phrases at the output boundary, enforced by the 2030 conformance suite.

- **Policy-as-code evaluates declaratively; it does not adjudicate intent.** An unsatisfied rule means
  *"no witnessing event was found in these receipts"* — not that a policy was wilfully violated. The
  evaluator is pure and deterministic; severity is advisory report metadata, not a decision, and the
  *decision to fail a build* belongs to the operator via the verb/exit-code layer, never to the
  evaluator (which is why `process::exit` is removed from the engine in 2027).

- **Every report carries a structural disclaimer** (`EvidenceReport.disclaimer`,
  `PolicyReport.disclaimer`, `EvidencePack.disclaimer` are non-optional fields, present in JSON *and*
  Markdown) stating the artifact is evidence for an auditor, not a compliance determination.

- **Redaction proves history without disclosing it.** Redaction-with-proof preserves the BLAKE3
  commitment so the chain still certifies "an event with this payload existed and was removed",
  satisfying erasure obligations **without** the tool re-exposing or judging the data.

In short: W10 hands an auditor a clean, signed, provenance-anchored **evidence pack** and gets out of
the way of the verdict. The receipt certifies; the auditor decides.
