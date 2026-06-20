# W1 — Foundations & Correctness

**Workstream:** W1 (the bedrock) · **Owner:** Foundations
**Horizon:** 2026 H2 → 2030 · **Status:** roadmap / proposal
**Doctrine:** *certify, don't decide.* Everything here is a **presentation, correctness, and
plumbing** layer. W1 never mints a verdict, never rewrites the rolling BLAKE3, never decides
whether a chain is honest. It makes the existing verifier's truth **legible, correct on the
wire, and machine-parseable**.

> **Build caveat.** The external deps are private-registry `26.6` crates (`clap-noun-verb`,
> `clnrm-core`, `wasm4pm`, `lsp-max`) that do not resolve in a lone checkout, so **nothing
> here was `cargo build`/`test`-verified.** All Rust below is *compilable-style* — correct
> against in-tree patterns, pending signature finalization against the sibling crates.

---

## 1. Mission & scope

### 1.1 Mission
W1 owns the two things every other workstream stands on:

1. **Correctness** — close the 10-item bug ledger (B1–B10) in
   [`docs/innovation/00-SYNTHESIS.md`](../innovation/00-SYNTHESIS.md) §Part A, the cited
   defects that grounding five agents surfaced. These break redirects, emit invalid JSON,
   silently drop tampered receipts, and pin chains to the wrong release.
2. **The output/diagnostics primitives** — `src/diag.rs` (stable error codes, exit-code
   catalog, rustc/miette-shaped `Diagnostic`) and `src/output.rs` (one `Out` handle:
   `--format human|json|yaml`, `--json`, `--quiet/--verbose`, `NO_COLOR/--color`, strict
   data→stdout / chatter→stderr), plus the **migration scaffolding** to retrofit the ~67
   verbs onto that contract.

The design source for the primitives is
[`docs/innovation/03-dx-cli-ergonomics.md`](../innovation/03-dx-cli-ergonomics.md) §3.1–3.2;
W1 turns that sketch into a sequenced, tested implementation.

### 1.2 In scope
- `src/diag.rs`, `src/output.rs` (new modules — the only new public surface W1 introduces).
- One method on the existing error type: `AffidavitError::diagnostic(&self) -> Diagnostic`.
- The single `main`/`run` exit seam that converts a `Diagnostic` into one `process::exit`.
- Per-verb retrofit scaffolding (a `migrate!` helper / recipe) and the batch migration of
  all ~67 handlers off `println!`/`eprintln!`/`format!`-JSON.
- Acceptance tests for every bug fix and the contract (golden stdout/stderr, exit-code
  matrix, schema snapshots).

### 1.3 Boundaries vs neighbors (what W1 does **not** build)
| W1 builds (primitive) | Consumer feature (NOT W1) | Owner |
|---|---|---|
| `Code`, `Code::explanation()` table | `affi --explain <CODE>` flag + offline doctor | **W3** |
| `Diagnostic`, stage→`Code` map, `Verdict` access | `affi why <RECEIPT>` plain-English verb | **W3** |
| The live arg/value-choice surface contract | generated completions (bash/zsh/fish/pwsh) | **W3** |
| `Out` + schema-stamped payloads | per-verb stable JSON **schemas** + `--schema` dump | **W3** |
| `Finding`-shaped diagnostic, stream split | `affi doctor` / `affi fix` self-healing | **W2** |
| (resolved verb count B7/B9) | `registry.rs` source-of-truth, grouping, "did you mean" | **W4** |

W1 ships the **mechanism**; W2/W3/W4 ship the **surfaces** that consume it. The line is
deliberate: if `--explain` and `why` lived here, W3 could not iterate on UX without touching
the correctness bedrock.

---

## 2. Current state (cited) + the gap

### 2.1 The bug ledger, re-confirmed against the tree at this commit
Every item below was re-verified by reading source, not the synthesis prose.

| # | Sev | Defect | Confirmed at | Note |
|---|-----|--------|--------------|------|
| **B1** | High | Stream split: `emit` → **stdout**, `verify`/`show`/`inspect`/`stats` → **stderr** | `handlers.rs:127` (stdout) vs `:356,:364,:687,:754` (stderr) | `affi receipt show r.json > out.txt` captures nothing |
| **B2** | High | JSON hand-built with `format!` interpolation (un-escaped → invalid/injectable) | `handlers.rs:157,166,184,203,223,264` | also `emit_batch` literal at `:157` |
| **B3** | High | Tampered receipts silently dropped: `if let Ok(r) = crate::cli::show(...)` | `handlers.rs:84` | `load_receipts_from_path` swallows the decode-time chain check |
| **B4** | High | `GENESIS_SEED` version drift | `chain.rs:22` → `b"affidavit-v26.6.14-genesis"` | package is **26.6.17** (CLAUDE.md, synthesis B4) → cross-box ACCEPT/REJECT divergence |
| **B5** | Med | `monitor` is a stub though a real `FileWatcher` exists | `handlers.rs:2632` ("not yet implemented") vs `quality.rs:1147,1158` | W1 owns *retiring the stub cleanly*; the `watch` daemon is W5 |
| **B6** | Med | Exit codes scattered / collapsed to ad-hoc `1`/`2` | `handlers.rs:352,367,562` **and** an extra `:2068` the audit missed | no catalog a script can branch on |
| **B7** | Low | Stray duplicate module file | `src/verbs/receipt_throughput.rs` **and** `src/verbs/receipt-throughput.rs` | hyphenated name can't be a Rust module → dead file |
| **B8** | Low | Stale completions cover ~4 of 67 verbs, no PowerShell | `completions/` | generation is W3; W1 only deletes the stale artifacts once W3 ships |
| **B9** | Low | Doc drift: README "59 capabilities" vs 67 `mod` decls | `README.md:101` vs `verbs/mod.rs` (67 decls) | true count must be fixed before W4 grouping / W3 completions |
| **B10** | Info | `linkme` declared, **zero** uses in `src/` | `Cargo.toml:42` (`linkme = "0.3"`); `grep -rln linkme src/` → none | first idiomatic use is W2's `DoctorCheck` registry |

Supporting facts that shape the design:
- `src/error.rs:178-251` already defines a strong `AffidavitError` with 18 variants + sub-enums
  (`OcelError:13`, `ChainError:33`, `ShardingError:84`, `SloViolation:137`). The structure is
  **thrown away** at the boundary: `handlers.rs:18-57` `to_noun_verb()` maps every variant to a
  flat `execution_error(format!("…: {e}"))`, and `:59-61` `adapt()` collapses all `anyhow`
  errors into `AffidavitError::Execution(format!("{e:#}"))`. The typed distinction is gone before
  a user sees it.
- `src/types.rs:270` `CheckOutcome` and `:293` `Verdict` are `Serialize` and carry a per-stage
  `detail` string — the raw material a `Diagnostic` and (later, W3) `why` present.
- `src/handlers.rs:96-108` `print_json_or` is the inline `format.as_deref() == Some("json")`
  helper, repeated ~30× across handlers — the seam every verb will route through `Out` instead.
- `src/lib.rs:61` `#![deny(clippy::print_stdout)]` is already set. Today it is satisfied only
  because handlers lean on `eprintln!`; the **contract must keep it satisfied** by funneling all
  stdout through one allowlisted module (`Out::emit`).

### 2.2 The gap
The verifier's *primitives are excellent and its logic is correct* — the defect surface is
entirely at the **presentation/plumbing boundary**: data goes to the wrong stream (B1);
machine output is string-built, not guaranteed valid (B2); failures are swallowed (B3) or
collapsed to indistinguishable exit codes (B6); the chain is pinned to a stale release
constant (B4); and the typed error structure exists but is flattened to a string at the seam
(`handlers.rs:18-61`). W1 closes that gap with two additive modules and a mechanical retrofit,
touching **no verification logic** in `verifier.rs` or the `chain.rs` fold.

---

## 3. Phased plan (2026 H2 → 2030)

Each phase lists objectives, concrete deliverables, compilable-style sketches where load-bearing,
acceptance criteria, and cross-workstream dependencies.

---

### Phase 2026 H2 — Correctness PR + the contract core (anchors synthesis P0)

**Objective.** Land the self-contained correctness fixes and the two primitive modules. This is
exactly synthesis P0 §Part D: *"land B1–B4 as a small, self-contained correctness PR first;
B1/B2 are prerequisites for the output contract."*

**Deliverables.**

**PR-1 — Correctness (B3, B4): the highest-trust, smallest blast radius.**
- B3: in `load_receipts_from_path` (`handlers.rs:84`) stop swallowing the error. A tampered
  receipt that fails the decode-time chain check must be **reported, not skipped** — surfaced,
  never judged (doctrine: we report that it did not decode; we do not call it dishonest).

```rust
// handlers.rs — B3 fix sketch (compilable-style)
for entry in entries {
    let entry = entry.map_err(io_err)?;
    let ep = entry.path();
    if ep.extension().and_then(|s| s.to_str()) == Some("json") {
        let path = ep.to_str().unwrap_or("");
        match crate::cli::show(path) {
            Ok(r) => receipts.push(r),
            // Surface, don't drop. A directory scan must not hide a receipt
            // that failed to decode/verify — that is the very evidence we exist
            // to preserve. We REPORT the failure; we do not decide honesty.
            Err(e) => out.note(format!(
                "skipping {path}: {}", AffidavitError::from(e).diagnostic().cause
            )),
        }
    }
}
```
- B4: replace the literal in `chain.rs:22` with a value derived from the crate version so the
  seed can never drift from the package again.

```rust
// chain.rs — B4 fix sketch
/// Genesis seed for the rolling chain hash. Bound to the build's crate version
/// via env! so it can NEVER drift from Cargo.toml again (was the hard-coded
/// "affidavit-v26.6.14-genesis" while the package shipped 26.6.17).
pub const GENESIS_SEED: &[u8] =
    concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis").as_bytes();
```
> **Compatibility note (doctrine-critical).** Changing `GENESIS_SEED` changes `chain_hash_0`
> (`chain.rs:48` `genesis_hash`), so old-seed receipts REJECT under the new seed. That is
> *correct* — it makes drift visible — but it is a **format break**, so we gate it: make the
> verifier seed-version-aware, OR ship a one-time `core/v1`→`core/v1.1` migration with a
> `affi receipt migrate-seed` re-assembly path. The PR lands the version-bound constant and the
> migration note together. **W1 never silently relaxes verification to launder old receipts into
> ACCEPT.**

**PR-2 — `src/diag.rs`: stable codes, exit-code catalog, `Diagnostic`.**
Mirrors [`03-dx-cli-ergonomics.md`](../innovation/03-dx-cli-ergonomics.md) §3.1. The `Code`
*string* is the contract (greppable in CI, linkable in docs); the discriminant is incidental.

```rust
//! src/diag.rs — stable error codes, exit-code catalog, teachable diagnostics.
//! Zero new deps; miette-grade output via std formatting (+ optional `colored` later).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {                 // documented + queryable; replaces ad-hoc exits
    Ok = 0,            // ACCEPT or any verb that succeeded
    Failure = 1,       // uncategorized last resort
    Reject = 2,        // verdict REJECT — a RESULT, not a crash
    Usage = 64,        // EX_USAGE: bad invocation / bad --format
    DataErr = 65,      // EX_DATAERR: receipt JSON unparseable
    Unavailable = 69,  // EX_UNAVAILABLE: feature not compiled in (e.g. lsp)
    IoErr = 74,        // EX_IOERR: file not found / unreadable
}
impl ExitCode { pub fn code(self) -> i32 { self as i32 } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    MalformedObjectRef, EmptyEventType,            // admission (maps OcelError:15,27)
    ReceiptNotFound, ReceiptParse,                 // io / decode
    ChainMismatch, SeqGap, DuplicateEventId,       // verify stages 3–4
    BadCommitment, FormatMismatch,                 // verify stages 5,2
    AdmissionRefused, FeatureDisabled, BadFormatFlag,
}
struct Row { code: Code, id: &'static str, title: &'static str,
             explanation: &'static str, exit: ExitCode }
// One static table row per Code keeps id/title/explanation/exit in lockstep.
static CODES: &[Row] = &[ /* … one row per arm … */ ];

impl Code {
    pub fn id(self)          -> &'static str { row(self).id }          // STABLE string
    pub fn title(self)       -> &'static str { row(self).title }       // --explain headline (W3)
    pub fn explanation(self) -> &'static str { row(self).explanation } // WHY/FIX/CONFIRM (W3 prints)
    pub fn exit(self)        -> ExitCode     { row(self).exit }
}
fn row(c: Code) -> &'static Row { CODES.iter().find(|r| r.code == c).expect("every Code has a row") }

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: Code,
    pub cause: String,              // what happened, concretely
    pub input: Option<SourceSpan>,  // offending text + caret (rustc-style)
    pub hint: Option<String>,       // single most actionable next step
}
#[derive(Debug, Clone)]
pub struct SourceSpan { pub label: String, pub text: String, pub start: usize, pub len: usize }

impl Diagnostic {
    pub fn new(code: Code, cause: impl Into<String>) -> Self {
        Self { code, cause: cause.into(), input: None, hint: None }
    }
    pub fn with_span(mut self, s: SourceSpan) -> Self { self.input = Some(s); self }
    pub fn with_hint(mut self, h: impl Into<String>) -> Self { self.hint = Some(h.into()); self }
    pub fn exit(&self) -> ExitCode { self.code.exit() }
    /// Machine form, stable schema "affidavit.diagnostic/v1".
    pub fn to_json(&self) -> serde_json::Value { /* serde_json::json!({ schema, code, … }) */ todo!() }
}
impl std::fmt::Display for Diagnostic { /* error[ID]: title / cause / --> span^^^ / hint / help */ }
```
The **bridge** is one method on the existing error type — W1 enriches the boundary that
`handlers.rs:18-61` flattens, without changing any verification:
```rust
// error.rs — additive; maps each existing variant to a Code + hint.
impl AffidavitError {
    pub fn diagnostic(&self) -> crate::diag::Diagnostic {
        use crate::diag::{Code, Diagnostic, SourceSpan};
        match self {
            AffidavitError::Ocel(OcelError::MalformedObjectRef(s)) =>
                Diagnostic::new(Code::MalformedObjectRef, "missing ':' separator between id and type")
                    .with_span(SourceSpan { label: "object spec".into(), text: s.clone(),
                                            start: 0, len: s.len() })
                    .with_hint("write it as `repo:main` — or add a qualifier: `repo:main:fast`"),
            AffidavitError::Io(e)   => Diagnostic::new(Code::ReceiptNotFound, e.to_string()),
            AffidavitError::Json(e) => Diagnostic::new(Code::ReceiptParse,   e.to_string()),
            AffidavitError::AdmissionRefused(s) => Diagnostic::new(Code::AdmissionRefused, s.clone()),
            _ => Diagnostic::new(Code::ReceiptParse, self.to_string()), // fallback; refined over time
        }
    }
}
```

**PR-3 — `src/output.rs`: the `Out` handle (fixes B1, B2, B6 at the seam).**
Mirrors §3.2 of the DX doc. One abstraction resolves format/color/stream once and **guarantees
valid machine output** (typed `Serialize`, never `format!`).

```rust
//! src/output.rs — one output contract for every verb. data→stdout, chatter→stderr.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format { Human, Json, Yaml }
impl Format {
    /// Unknown values are a USAGE error (AFFI-E001), never a silent fallback (B-contract).
    pub fn resolve(flag: Option<&str>, json: bool) -> Result<Self, crate::diag::Diagnostic> {
        if json { return Ok(Format::Json); }
        match flag {
            None | Some("human") => Ok(Format::Human),
            Some("json") => Ok(Format::Json),
            Some("yaml") => Ok(Format::Yaml),
            Some(other) => Err(crate::diag::Diagnostic::new(
                crate::diag::Code::BadFormatFlag, format!("unknown --format value {other:?}"))
                .with_hint("valid values: human, json, yaml")),
        }
    }
}
pub struct Out { pub format: Format, pub color: bool, pub quiet: bool, pub verbose: bool }
impl Out {
    pub fn from_flags(format: Option<&str>, json: bool) -> Result<Self, crate::diag::Diagnostic> {
        Ok(Self { format: Format::resolve(format, json)?,
                  color: false, quiet: false, verbose: false }) // color/quiet/verbose wired in 2027
    }
    /// DATA → stdout. One typed value renders as human/json/yaml — no string-built JSON (kills B2).
    pub fn emit<T: serde::Serialize>(&self, value: &T, human: impl FnOnce(&mut String)) {
        match self.format {
            Format::Json => println!("{}", serde_json::to_string_pretty(value).expect("Serialize")),
            Format::Yaml => println!("{}", serde_yaml::to_string(value).expect("Serialize")),
            Format::Human => { let mut s = String::new(); human(&mut s); print!("{s}"); }
        }
    }
    /// CHATTER → stderr; suppressed under --quiet; never machine-parsed (kills B1).
    pub fn note(&self, msg: impl AsRef<str>) { if !self.quiet { eprintln!("{}", msg.as_ref()); } }
    /// ERROR → stderr always; returns the exit code so MAIN does the single exit (kills B6).
    pub fn fail(&self, d: &crate::diag::Diagnostic) -> i32 {
        match self.format {
            Format::Human => eprintln!("{d}"),
            _ => eprintln!("{}", serde_json::to_string_pretty(&d.to_json()).expect("Serialize")),
        }
        d.exit().code()
    }
}
```
> `Out::emit` is the **single** allowlisted `println!` site, which is how `#![deny(clippy::print_stdout)]`
> (`lib.rs:61`) keeps holding once handlers stop printing directly.

**PR-4 — Prove the seam: migrate the 4 core verbs + centralize the exit.**
- Migrate `emit` (`handlers.rs:115`), `assemble`, `verify`, `show` to `Out`.
- Replace the 4 scattered `std::process::exit` (`handlers.rs:352,367,562,2068`) with a **single**
  exit in the `run`/`main` seam that consumes the code returned by `Out::fail`. Handlers return
  `Diagnostic`/code; they never call `process::exit`.

**Acceptance criteria (2026 H2).**
- `affi receipt show r.json > out.txt` produces non-empty `out.txt` (B1 regression test:
  capture stdout, assert the chain dump is present; assert stderr holds only chatter).
- `affi receipt emit --object 'a"b:t' …` (a quote in a value) emits JSON that `jq .` parses
  (B2 fuzz test over values containing `"`, `\`, newline).
- A directory containing one tampered receipt makes `verify-family`/scan **report** it on stderr
  with a `Code`, and the receipt is *not* silently absent from the count (B3).
- `GENESIS_SEED` equals `concat!("affidavit-v", CARGO_PKG_VERSION, "-genesis")`; a unit test
  asserts seed-vs-version equality so drift fails CI forever (B4).
- An exit-code matrix test: ACCEPT→0, REJECT→2, bad `--format`→64, missing file→74, bad JSON→65.
- `Code::id()` strings are snapshotted; a test fails if any shipped id changes (stability lock).
- The 4 core verbs round-trip `--format human|json|yaml`; `--json` == `--format json`.

**Dependencies.** None inbound (this is the floor). **Outbound:** unblocks W2 `doctor --json`,
W3 `--explain`/`why`/completions/schemas, W4 registry, W5 `watch` output.

---

### Phase 2027 — Full retrofit, the diagnostic catalog filled, color/quiet/verbose

**Objective.** Make the contract *universal* (all ~67 verbs) and *complete* (every error path
yields a real `Code`, color/quiet/verbose honored), so downstream workstreams build against a
finished primitive rather than a moving one.

**Deliverables.**
- **Migration scaffolding** so 67 verbs convert mechanically (the synthesis "retrofit recipe",
  §03 P-rollout, made a macro). A `migrate!` helper standardizes the wrapper body:

```rust
// macro sketch — each handler becomes "build Out, do work, emit typed payload, map err→Diagnostic".
macro_rules! with_out {
    ($format:expr, $json:expr, |$out:ident| $body:block) => {{
        let $out = match crate::output::Out::from_flags($format.as_deref(), $json) {
            Ok(o) => o,
            Err(d) => return Err(crate::handlers::diag_to_noun_verb(d)), // USAGE path
        };
        $body
    }};
}
```
- **Retire `print_json_or`** (`handlers.rs:96`): delete the helper once every caller routes
  through `Out`; the inline `format.as_deref() == Some("json")` checks vanish with it.
- **Fill the `Code` table**: every variant of `AffidavitError` (`error.rs:178`) and every
  sub-enum (`OcelError`, `ChainError`, `ShardingError`, `SloViolation`) maps to a `Code` with a
  caret span where the offending input is known (e.g. `OcelError::MalformedObjectRef` →
  `SourceSpan` at the missing-colon offset). Stage→`Code` total map provided for W3's `why`:
  `chain_integrity→ChainMismatch`, `continuity→SeqGap|DuplicateEventId` (disambiguated by
  `CheckOutcome.detail`, `types.rs:270`), `verify_commitments→BadCommitment`,
  `check_format→FormatMismatch`.
- **Color & accessibility**: wire `ColorChoice::{Auto,Always,Never}` honoring `NO_COLOR` (any
  value) and stdout-isatty. `colored` already exists behind the `ui` feature (`Cargo.toml:80-81`);
  W1 ensures color is decided once in `Out` and **never leaks ANSI into a pipe**.
- **`--quiet`/`--verbose`** semantics finalized: quiet suppresses `note()` chatter but never data
  or errors; verbose adds per-stage timing to stderr.
- **B5/B7/B9 cleanup**: delete the dead hyphenated `src/verbs/receipt-throughput.rs` (B7);
  reconcile the verb count and the README "59 capabilities" claim (`README.md:101`) against the
  67 `verbs/mod.rs` decls — publish the authoritative number W4 will index (B9). For B5, mark the
  `monitor` stub (`handlers.rs:2632`) **deprecated** and route it through `Out::note` cleanly; the
  real `watch` daemon over `FileWatcher` (`quality.rs:1147`) is W5's to build (W1 only stops the
  stub from lying via stray `eprintln!`).

**Acceptance criteria (2027).**
- 100% of handlers route stdout through `Out::emit`; CI `grep` asserts **zero** `println!` and
  zero `format!`-built JSON outside `src/output.rs` (machine-checked, not vibes).
- Every `AffidavitError` variant returns a `Diagnostic` whose `Code::id()` is in the catalog;
  a test enumerates variants and asserts no `Code::ReceiptParse` fallback leaks for a known case.
- `NO_COLOR=1 affi … --color always | cat` contains zero ANSI escape bytes (pipe = no color).
- `affi … --quiet` produces byte-identical stdout to non-quiet for the same verb (chatter only
  differs on stderr).
- The verb count is single-sourced; README and any count display agree (B9 closed).

**Dependencies.** **Inbound:** W4 publishes the registry's authoritative verb enumeration (helps
finalize B9 count); if it slips, W1 ships an interim const. **Outbound:** W3 can now generate
completions from a stable arg surface and ship `--explain` against the filled table; W2's doctor
`Finding` reuses the `Diagnostic` shape.

---

### Phase 2028 — Contract hardening: schema stability, conformance gate, fuzzing

**Objective.** Promote the output contract from "consistent" to **guaranteed and versioned** —
the point at which external tools (W9 interop, W10 compliance) can depend on `affi`'s JSON the way
they depend on a published API.

**Deliverables.**
- **Schema registry for payloads.** Every `Out::emit` payload carries `"schema": "affidavit.<name>/vN"`.
  W1 centralizes the schema constants and adds a snapshot test per payload type (`EmitOutput`,
  `AssembleOutput`, `InspectionReport`, `Verdict` at `types.rs:293`, the new `Diagnostic`/v1). A
  breaking field change must bump `N` and keep the prior encoder for one release. (The *public
  `--schema` dump* surface is W3; W1 owns the **invariant** and its enforcement.)
- **A contract conformance test harness.** A table-driven suite runs every verb in
  `human|json|yaml`, asserting: data only on stdout, chatter only on stderr, JSON parses, the
  `schema` field is present and matches the registry, and the exit code matches the catalog. This
  is W1's standing guard so neighbor PRs cannot silently regress the contract.
- **Diagnostic fuzzing.** Property tests feed adversarial inputs (paths/object refs/payloads with
  quotes, backslashes, control chars, non-UTF-8-ish escapes) through emit/verify and assert output
  is always valid JSON and the caret offsets in `SourceSpan` stay in-bounds.
- **Exit-code & code-string golden lock.** Expand the stability snapshot to the full catalog;
  CI diff-fails on any renumber/rename. Begin a deprecation registry (`AFFI-Exxx → superseded by`)
  so codes are append-only forever.

**Acceptance criteria (2028).**
- Conformance harness covers ≥95% of verbs and runs in CI on every PR; a deliberately-regressed
  verb (wrong stream) fails it.
- Every machine payload has a `schema` field and a snapshot; bumping a field without bumping `vN`
  fails the snapshot test.
- Fuzz suite runs ≥1e6 cases with zero invalid-JSON / out-of-bounds-span findings.
- A documented, machine-readable exit-code + error-code catalog file exists and is asserted in
  sync with `diag.rs` (the source W3's `--explain` and W10's audit tooling both read).

**Dependencies.** **Outbound:** W9 (ecosystem interop) and W10 (compliance/governance) consume the
versioned schema registry and the catalog file as their stable contract. **Inbound:** none blocking.

---

### Phase 2029 — Structured event stream + scale-safe output

**Objective.** Extend the contract from "one payload per command" to a **structured, streamable
diagnostic event model**, so W7's large/streaming/parallel verification and W6's interactive
surfaces have a uniform, ordered way to surface progress and per-item results without abandoning
the data/stdout discipline.

**Deliverables.**
- **NDJSON / event-stream mode.** `--format json-stream` (or `--ndjson`) emits one schema-stamped
  JSON object per line for long scans, so a 100k-receipt store scan (W7 territory) streams results
  instead of buffering. W1 owns the *envelope* (`{schema:"affidavit.event/v1", kind, seq, payload}`)
  and ordering guarantee; the producers (parallel verify, store scan) are W7/W2.
- **Backpressure-safe `Out`.** Replace `println!` internals with a buffered, flush-disciplined
  writer so high-throughput producers (W7 parallel stages) don't interleave partial lines across
  threads. The stream stays append-only and per-line atomic.
- **Diagnostic aggregation type.** A `DiagnosticSet` (ordered, dedup-aware) so a multi-item
  operation can return *all* findings with stable ordering, which W2 doctor and W7 batch verify
  both need. Still pure presentation — it aggregates `Diagnostic`s; it never re-decides a verdict.
- **Determinism guarantee on output.** Lock byte-for-byte identical JSON for identical inputs
  (field order, no wall-clock in payloads) — extending the project's existing determinism doctrine
  (CLAUDE.md §4) from receipts to *output*, which W10 reproducibility audits rely on.

```rust
// output.rs — streaming envelope sketch (compilable-style)
#[derive(serde::Serialize)]
pub struct Event<'a, T: serde::Serialize> {
    pub schema: &'static str,   // "affidavit.event/v1"
    pub kind: &'a str,          // "progress" | "item" | "summary"
    pub seq: u64,               // monotonic, deterministic (no timestamps)
    pub payload: T,
}
impl Out {
    pub fn stream_item<T: serde::Serialize>(&self, w: &mut impl std::io::Write, seq: u64, kind: &str, p: T) {
        if let Format::Json = self.format {
            let _ = writeln!(w, "{}", serde_json::to_string(
                &Event { schema: "affidavit.event/v1", kind, seq, payload: p }).expect("Serialize"));
        }
    }
}
```

**Acceptance criteria (2029).**
- A 100k-item scan in `--ndjson` streams the first line before the last item is computed (memory
  stays flat; tested with a synthetic producer).
- Under a parallel producer (N threads), no line is ever interleaved/torn (concurrency stress test).
- Identical inputs produce byte-identical stream output across two runs and two machines
  (determinism test, `--test-threads=1`).
- `DiagnosticSet` preserves insertion order and is snapshot-stable.

**Dependencies.** **Inbound:** W7 (scale/streaming/parallel engine) is the primary producer and
co-designs the envelope cadence; W6 (REPL/TUI) consumes the stream for live views. **Outbound:**
W2/W7 batch operations emit through this without rolling their own framing.

---

### Phase 2030 — Stabilization, contract v1.0, long-term guarantees

**Objective.** Declare the W1 contract **1.0 / frozen**: a fully documented, fuzz-hardened,
schema-versioned, deterministic output + diagnostics layer with formal stability guarantees, so
the entire program (W2–W10) and external integrators build on a surface that will not move.

**Deliverables.**
- **`affidavit-contract/1.0`**: a frozen specification doc + machine-readable catalog (codes, exit
  codes, schemas, stream envelope) with a written compatibility policy (codes append-only; schemas
  bump `vN` and keep N-1 for one release; exit codes `0/2` semantics permanent).
- **A `#[diagnostic]` derive / macro** (or equivalent) so new verbs and new error variants get a
  `Code` + `Diagnostic` by construction — making it *structurally impossible* to ship a verb that
  bypasses the contract (closes the class of bug B1/B2 were instances of, permanently).
- **Contract regression CI as a release gate**: the 2028 conformance harness + 2029 determinism +
  fuzz suite are required, blocking checks on `main`; a release cannot ship a contract regression.
- **Backfill audit**: confirm zero remaining `format!`-JSON, zero stray `process::exit`, zero
  ledger items open; the bug ledger (B1–B10) is closed and *kept* closed by machine checks.

**Acceptance criteria (2030).**
- The contract spec + catalog file are published and versioned `1.0`; a test asserts `diag.rs`,
  `output.rs`, and the spec agree.
- It is impossible to add a verb that prints to stdout outside `Out` (lint + macro enforced;
  demonstrated by a should-not-compile / CI-rejected fixture).
- All of B1–B10 are closed with standing regression tests; the ledger table links each to its test.
- Determinism, fuzz, and conformance suites are required release gates and green.

**Dependencies.** **Inbound:** W10 (governance) ratifies the stability/compatibility policy and
consumes the catalog for compliance evidence. **Outbound:** every workstream now depends on a frozen
W1 1.0.

---

## 4. Definition of done @2030

W1 is done when:

1. **The bug ledger is closed and stays closed.** B1–B10 each have a standing regression test
   (stream split, JSON validity, tamper-report, seed/version equality, exit-code matrix, dead-file
   absence, verb-count single-source, no-`linkme`-unused once W2 lands its registry). The ledger
   table in [`00-SYNTHESIS.md`](../innovation/00-SYNTHESIS.md) is fully struck through.
2. **One output contract, universally applied.** All ~67 verbs funnel data through `Out::emit`
   (stdout) and chatter through `Out::note` (stderr); `--format human|json|yaml`, `--json`,
   `--quiet`, `--verbose`, `NO_COLOR`/`--color` behave identically everywhere; `#![deny(clippy::print_stdout)]`
   (`lib.rs:61`) holds with `Out` as the lone allowlisted stdout site.
3. **One diagnostic system.** Every error path yields a `Diagnostic` with a stable `AFFI-Exxx`
   code, an exit code from the catalog, and (where input is known) a rustc-style caret. Codes are
   append-only and snapshot-locked.
4. **A frozen, versioned, machine-readable contract** (`affidavit-contract/1.0`): codes, exit
   codes, payload schemas (`…/vN`), and the streaming envelope — fuzz-hardened, deterministic, and
   enforced by required CI gates.
5. **Doctrine intact.** Across every phase, W1 added zero verdict logic. It never relaxed a
   verification stage, never laundered a stale-seed receipt into ACCEPT, never decided honesty —
   it only made the verifier's existing truth correct on the wire and legible to humans and scripts.

---

## 5. Cross-workstream dependencies — what W1 provides to W2–W10

W1 is the supplier; the table is the interface contract.

| WS | What W1 hands them | Where it lands |
|----|--------------------|----------------|
| **W2** Doctor & Self-Healing | `Diagnostic`/`Finding`-compatible shape; `Out` for `--json`; the stream split; the *first idiomatic `linkme` use* slot (B10) for the `DoctorCheck` registry. W1 retires the `monitor` stub's lying `eprintln!` (B5) cleanly. | 2026 H2 `diag.rs`/`output.rs`; 2027 catalog |
| **W3** CLI Ergonomics & Public Contract | `Code::explanation()` table → `affi --explain`; stage→`Code` map + `Verdict` access → `affi why`; stable arg/value surface → completions; schema invariant → `--schema` dump. **W3 builds the surfaces; W1 builds the mechanism.** | 2026 H2 → 2028 |
| **W4** Onboarding/Discoverability/Registry | The single authoritative verb count (B7/B9 resolved); typed `Serialize` payloads to index; `Diagnostic` for "did you mean" usage errors. | 2027 (count); ongoing |
| **W5** Workflow Automation & Config | `Out` so `watch`/`config` output obeys the data/stdout split; config-resolution errors as `Diagnostic`s. W5 builds the real `watch` daemon over `FileWatcher` (`quality.rs:1147`); W1 cleared the stub. | 2027 → 2029 |
| **W6** Interactive Surfaces (REPL/TUI/LSP) | The 2029 structured **event stream** for live views; `Diagnostic` for LSP diagnostics (the typed shape `lsp.rs` wants instead of a flat string). | 2029 |
| **W7** Verification Engine (scale/stream/parallel/GPU/distributed) | NDJSON streaming envelope + backpressure-safe `Out` + `DiagnosticSet` for batch/parallel result framing; determinism guarantee on output. W7 produces; W1 frames. | 2029 |
| **W8** Cryptography & Trust | `Diagnostic`/`Code` for signature/notarization failure paths (replacing the `format!`-JSON in `assemble_with_signature`/`assemble_and_notarize`, `handlers.rs:303,324` per §03 §2.1); stable schema for signed-receipt payloads. | 2026 H2 (JSON fix) → 2028 (schema) |
| **W9** Ecosystem & Standards Interop | The versioned payload **schema registry** (`…/vN`) and the machine-readable catalog as the stable contract external tools integrate against. | 2028 |
| **W10** Compliance & Governance | The frozen `affidavit-contract/1.0` catalog (codes/exit codes/schemas) as compliance evidence; the determinism + conformance CI gates as governance controls; ratifies the append-only stability policy. | 2028 → 2030 |

**Anti-dependency (explicit non-goals, to prevent scope creep):** W1 does **not** ship
`--explain`, `why`, completions, `--schema`, the registry, the doctor, the watch daemon, or any
verdict/honesty logic. It ships the **primitives those consume** and the **correctness floor they
stand on** — and it guards both with standing tests so neighbor PRs cannot regress them.
