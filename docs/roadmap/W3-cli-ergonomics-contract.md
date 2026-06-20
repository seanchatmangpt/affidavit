# W3 — CLI Ergonomics & Public Contract (Roadmap → 2030)

**Workstream:** W3 of 10 · **Owner:** CLI Ergonomics & Public Contract
**Status:** program plan / proposal · **Date:** 2026-06-20
**Grounds on:** `docs/innovation/00-SYNTHESIS.md`, `docs/innovation/03-dx-cli-ergonomics.md`
**Doctrine guardrail:** *certify, don't decide.* Nothing in W3 mints, mutates, or
launders a verdict. Every feature here is a **presentation/contract seam** over the
existing `Verdict`/`CheckOutcome` (`src/types.rs:269,292`) and the existing pipeline
(`src/cli.rs:110`). We change how a human or a script *experiences* a verdict, never
the verdict.

> **Build caveat:** the five `26.6` sibling crates (`clap-noun-verb`,
> `clap-noun-verb-macros`, `clnrm-core`, `wasm4pm`, `lsp-max`) do not resolve in a lone
> checkout (`Cargo.toml:40-41`), so **nothing here is `cargo build`/`test`-verified.**
> All Rust is compilable-*style*: correct against in-tree patterns, pending signature
> finalization against the sibling crates. This doc creates no code and modifies no
> existing file.

---

## 1. Mission, scope & boundaries

### 1.1 Mission

Turn `affi` from a tool that *works* into a tool with a **stable, versioned, fully
scriptable public CLI contract** that lasts to 2030. The north star (from 03 §1): *a
first-time user fixes their own error without leaving the terminal, and a CI script
parses every verb the same way, forever.*

### 1.2 W3 owns (the deliverable surface)

1. **`affi --explain <CODE>`** — offline error/exit-code explainer (no receipt, no
   network; works in an air-gapped CI box).
2. **`affi why <RECEIPT>`** — plain-English verdict narrative derived from the existing
   `Verdict`/`CheckOutcome`.
3. **Uniform output flags rolled across ALL ~67 verbs** — `--format human|json|yaml`,
   `--json`, `--quiet`, `--verbose`, `--color auto|always|never`.
4. **Generated shell completions** — bash, zsh, fish, **and powershell**, from the verb
   registry (W4's source-of-truth), replacing today's stale hand-authored files.
5. **Machine-readable output JSON Schema per verb** — one published schema per verb's
   `--json` payload, dumpable via `affi --schema <name>`.
6. **Output-schema versioning / stability + deprecation policy** — so downstream scripts
   never break silently, with an explicit support window extending to 2030.

### 1.3 Boundaries (who builds what)

| Concern | Owner | W3's relationship |
|---|---|---|
| `src/diag.rs` (`Code`, `ExitCode`, `Diagnostic`) + `src/output.rs` (`Format`, `Out`) **primitives** | **W1** Foundations | W3 **consumes**; builds the *features* (`--explain`, `why`, completions, schemas) on top. |
| Bug fixes B1 (stream split), B2 (`format!`-JSON), B6 (exit codes) | **W1** | Prerequisites for W3's output contract; W3 does **not** re-fix them. |
| Verb **registry** source-of-truth (`src/registry.rs`) | **W4** Onboarding/Registry | W3 **consumes** it to enumerate verbs/args/value-choices for completions, `--explain`, and schema generation. |
| `affi doctor`/`fix`, watch, init/config, REPL/TUI, verification-engine internals, crypto, governance | W2/W5/W6/W7/W8/W10 | W3 supplies the *output contract* (`Out` discipline, schemas) those verbs emit through; their logic is out of scope here. |

W3 is the **cross-verb rollout driver** and the **schema/versioning program owner**. W1
hands us primitives and a fixed stream discipline; W4 hands us a registry; W3 turns both
into the user-facing contract and *keeps it stable through 2030*.

---

## 2. Current state (cited) & gap

### 2.1 The verb mechanism (what we are rolling across)

Each verb is a thin wrapper using the `clap_noun_verb_macros::verb` attribute, e.g.
`src/verbs/verify.rs:13`:

```rust
#[verb("verify", "receipt")]
pub fn verify(receipt: String, format: Option<String>, profile: Option<String>, strict: Option<bool>)
    -> Result<()> { crate::handlers::verify(receipt, format, profile, strict) }
```

`src/verbs/mod.rs:4-70` declares **67 verb modules** (the true count; README's "59" is
stale — synthesis B9). The single hand-written dispatch seam is `lib.rs:129-131`:

```rust
pub fn run() -> clap_noun_verb::Result<()> { clap_noun_verb::run() }
```

This is the one place W3's top-level intercepts (`--explain`, `--completions`,
`--schema`) hook in *before* noun-verb dispatch.

### 2.2 Output format — inconsistent, three defects (all W1-owned to fix)

The only format check is the literal string `"json"`, repeated inline ~30×:

- `src/handlers.rs:101` `print_json_or` → `if format.as_deref() == Some("json")`; copied
  at `emit` L122, `assemble` L284, `verify` L348, `inspect` L734, `stats` L832,
  `dora_metrics` L1383, and ~24 more.
- **No `human`/`yaml` literal** — `--format yaml` silently falls back to human text.
- **Stream split (B1):** `emit` writes to **stdout** (`handlers.rs:127`), but
  `verify`/`show`/`inspect`/`stats`/`diff` write to **stderr**
  (`handlers.rs:356-365,687-719,754,844,780`). `affi receipt show r.json > out.txt`
  captures nothing.
- **Hand-built JSON via `format!` (B2):** `emit_batch` (`handlers.rs:157`),
  `assemble_with_signature` (L303), `assemble_and_notarize` (L324) interpolate values
  into JSON strings → invalid/injectable JSON when a value contains `"` or `\`.
- **`--format` is per-verb, not global:** `replay` (`handlers.rs:897`), `model` (L925),
  `conformance` (L948), `diagnose` (L974) have **no `format` parameter at all**;
  `visualize` (L1001) takes `format: String` **required** — the only verb where
  `--format` is mandatory.
- **No `--json`/`--quiet`/`--verbose`/`--color`** anywhere.

### 2.3 Errors — rich type flattened to a string (W1 owns `diag.rs`)

`src/error.rs:178-251` defines a strong `AffidavitError` (18 variants + sub-enums
`OcelError`, `ChainError`, `ShardingError`, …). The boundary throws the structure away:
`handlers.rs:18-57` `to_noun_verb()` maps **every** variant to
`NounVerbError::execution_error(format!("…: {e}"))`, and `handlers.rs:59-61` `adapt()`
collapses all `anyhow` errors into `Execution(format!("{e:#}"))` first. No variant carries
a **code**, **hint**, or **span**. `OcelError::MalformedObjectRef(String)` (`error.rs:27`)
knows the bad input but renders it with no caret and no fix.

### 2.4 Exit codes — two values, scattered (B6, W1 owns the catalog)

`src/cli.rs:110` `verify()` returns `(i32, Verdict)` (0 or 2). The process exit is forced
by scattered `std::process::exit(code)` **inside handlers**: `handlers.rs:352,367`
(verify), `562` (verify-compliance), `2068`. `verify_sla` failure returns a generic
`execution_error` (`handlers.rs:463`) → exit `1`, indistinguishable from an IO error. No
catalog: usage/IO/REJECT all collapse onto `1`/`2`.

### 2.5 Color — none today, will leak when added

`colored`/`indicatif` exist behind the `ui` feature (`Cargo.toml:80-81,153`) but no
handler consults `NO_COLOR`, `--color`, or `isatty`. The moment color is added without a
gate, ANSI leaks into pipes.

### 2.6 Completions — stale, partial, no PowerShell (B8) — **W3 owns the fix**

`completions/affi.bash:1-13` header literally says *"AUTHORED, NOT GENERATED … Keep it in
sync."* It enumerates `receipt_verbs='emit assemble verify show'` (`affi.bash:30`) — **4
of 67 verbs**. `--format` completes only `json` (`affi.bash:35`), matching the broken
contract. `completions/` contains only `affi.bash`, `affi.zsh`, `affi.fish` — **no
PowerShell**. Three files, ~94% of verbs missing.

### 2.7 Discoverability — no `--explain`, no `why`, no schemas — **W3 owns**

No `--explain` flag, no `why` verb, no `--schema` dump, and `docs/schemas/` does not
exist. `diagnose` (`handlers.rs:974`) is the nearest analog but requires the `lsp` feature
and emits LSP `line:character` diagnostics (a hard error `"lsp feature not enabled"`,
L996, without it) — not plain English.

### 2.8 The gap, stated as a contract

The **primitives are excellent** (typed errors, typed `Verdict`/`CheckOutcome` with
per-stage `detail`); they are simply **not surfaced** as a stable, versioned, scriptable
contract. The 1000x win is almost entirely a presentation/contract seam — *plus* the
discipline to **version and never break it** for the rest of the decade. That second half
— the multi-year stability program — is what this roadmap adds beyond 03.

---

## 3. Phased plan (2026 H2 → 2030)

Each phase: **objectives → deliverables → compilable-style sketch → acceptance criteria →
cross-workstream deps.** 2026 H2 is anchored to synthesis **P0/P1**; later years extend
the schema/versioning program 03 only gestures at.

Sizes (synthesis convention, unverified against a real build): **S** ≤ ½ day · **M** ~1–2
days · **L** ~3–5 days.

---

### Phase 2026 H2 — Surface the contract (anchored to synthesis P0/P1)

**Objectives.** Ship the three marquee W3 features on top of W1's just-landed
`diag.rs`/`output.rs`, and prove the cross-verb rollout on the 4 core verbs. Establish
schema v1 for those verbs as the contract anchor.

**Deliverables.**

1. **`affi --explain <CODE>`** (synthesis P0, 03 §3.4) — top-level flag intercepted in
   `lib.rs:129` *before* dispatch. Resolves a `Code` (W1's `diag::Code`) by `id()` or a
   bare exit code, prints `title()` + `explanation()`; `--json` returns the catalog entry
   as data. The **code/exit-code catalog table itself is a W3 deliverable** (the canonical
   list users grep), authored against W1's `Code`/`ExitCode` enums.
2. **`affi why <RECEIPT>`** (synthesis P1, 03 §3.5) — new verb, pure presentation over the
   *existing* pipeline (`cli.rs:110`) and per-stage `CheckOutcome.detail`. No new
   verification logic.
3. **Core-verb rollout to `Out`** — migrate `emit`/`assemble`/`verify`/`show` off
   `format.as_deref() == Some("json")` (`handlers.rs:122,284,348,682`) to W1's `Out`.
   This proves the retrofit recipe (03 §5) and removes the highest-traffic stream split.
4. **`--json` shorthand + `--format yaml`** on the 4 core verbs (completes the output
   contract there; `serde_yaml` dep added by W1).
5. **Schema v1 for `verdict`, `emit`, `assemble`, `show`** — published under
   `docs/schemas/` with `"schema": "affidavit.<name>/v1"` stamped on each payload.

**Sketch — the explain table (W3-authored, over W1's `Code`):**

```rust
//! W3 authors the canonical catalog rows; W1 owns the Code/ExitCode enums.
//! `affi --explain <CODE>` and `affi --help` footer both read this one table.
pub struct ExplainRow {
    pub id: &'static str,        // STABLE contract string, e.g. "AFFI-E031"
    pub exit: crate::diag::ExitCode,
    pub title: &'static str,     // one-line headline
    pub why: &'static str,       // multi-line cause
    pub fix: &'static str,       // the actionable next step
    pub confirm: &'static str,   // a command to verify the fix
}

pub static CATALOG: &[ExplainRow] = &[
    ExplainRow {
        id: "AFFI-E030", exit: crate::diag::ExitCode::Reject,
        title: "chain_integrity: recomputed chain hash != stored chain_hash",
        why:  "Stage 3 recomputes the rolling BLAKE3 over events in order; a mismatch \
               means at least one event's bytes changed after sealing.",
        fix:  "You cannot patch the hash by hand. Re-assemble from a correct working set.",
        confirm: "affi receipt why <receipt>",
    },
    // … one row per Code; AFFI-E031 SeqGap, E033 BadCommitment, E001 BadFormatFlag, …
];

/// Intercepted in lib.rs::run() before clap_noun_verb::run().
pub fn explain(code: &str) -> i32 {                 // returns the process exit code
    // bare exit codes ("2") and ids ("AFFI-E030") both resolve
    if let Some(row) = CATALOG.iter().find(|r| r.id == code || r.exit.code().to_string() == code) {
        println!("{}  {}\nexit code: {} ({:?})\n\nWHY: {}\n\nFIX: {}\n\nCONFIRM: {}",
                 row.id, row.title, row.exit.code(), row.exit, row.why, row.fix, row.confirm);
        return crate::diag::ExitCode::Ok.code();
    }
    eprintln!("unknown code {code:?}; see `affi --help` for the exit-code catalog");
    crate::diag::ExitCode::Usage.code()
}
```

**Sketch — `affi why` (presentation over the existing pipeline):**

```rust
//! src/verbs/why.rs (thin wrapper) -> crate::handlers::why
#[verb("why", "receipt")]
pub fn why(receipt: String, format: Option<String>, json: Option<bool>) -> Result<()> {
    crate::handlers::why(receipt, format, json.unwrap_or(false))
}

// handlers.rs — NO new verification; reuses cli::verify (the existing pipeline):
pub fn why(receipt: String, format: Option<String>, json: bool) -> Result<()> {
    let out = crate::output::Out::from_flags(format.as_deref(), json)?;     // W1 type
    let (_code, verdict) = adapt(crate::cli::verify(&receipt))?;            // existing
    let story = WhyReport::from_verdict(&receipt, &verdict);                // maps stage->Code
    out.emit(&story, |s| story.render_human(s));
    if !verdict.accepted { return Err(/* W1 Diagnostic -> exit via main */ todo!()); }
    Ok(())
}

/// #[derive(Serialize)], schema "affidavit.why/v1":
/// { schema, receipt, accepted, failing_stage, code, plain_english, next_command }
pub struct WhyReport { /* … */ }
```

The stage→Code map is total and tiny (03 §3.5): `chain_integrity → ChainMismatch`,
`continuity → SeqGap | DuplicateEventId` (disambiguated by `detail`),
`verify_commitments → BadCommitment`, `check_format → FormatMismatch`. Every arm already
has a human `detail` from `verifier.rs`.

**Acceptance criteria.**
- `affi --explain AFFI-E030` and `affi --explain 2` both print a non-empty explanation and
  exit `0`; an unknown code exits `64` (USAGE).
- `affi --explain <CODE> --json` emits valid JSON validating against
  `docs/schemas/explain.v1.json`.
- `affi receipt why ok.json` prints `ACCEPTED … all 7 stages`; `affi receipt why
  tampered.json` names the failing stage and exits `2`.
- `affi receipt verify ok.json --json | jq .accepted` works **and** `affi receipt show
  r.json > out.txt` now captures the dump (stream split gone for the 4 core verbs).
- Every core-verb `--json` payload carries `"schema": "affidavit.<name>/v1"`.
- A test asserts each shipped `Code::id()` string and each `affidavit.*/vN` schema string
  is byte-stable (drift = failing test).

**Cross-workstream deps.** **W1** must land `diag.rs`/`output.rs` and bug fixes B1/B2/B6
*first* (`Out`, `Code`, `ExitCode`, fixed streams). **W4**'s registry is *not* strictly
required for this phase (the 4 verbs are hand-listed), but `--explain` titles should reuse
W7's stage names where they exist.

---

### Phase 2027 — Roll the contract across all 67 verbs

**Objectives.** Take the proven core-verb pattern to the **full verb surface**, give every
verb the global-arg group, and ship generated completions for all four shells. End-state:
no verb is special; every verb honors one output contract.

**Deliverables.**

1. **Global-arg group** (03 §3.3) — define `--format/--json/--quiet/--verbose/--color`
   **once** and attach to the noun so all 67 verbs inherit it. The verbs that lack
   `format` today — `replay`/`model`/`conformance`/`diagnose` (`handlers.rs:897,925,948,974`)
   — get it for free; `visualize`'s required `format: String` (L1001) is normalized to the
   optional contract.
2. **Mechanical migration of the remaining ~63 verbs** to `Out` (03 §5 recipe), in batches
   by cluster (emission, assembly, verification, analytics, sbom, compliance). Deletes
   every remaining `format!`-built JSON (e.g. `handlers.rs:157,166,304,325`) and every
   in-handler `process::exit` (`handlers.rs:352,367,562,2068`).
3. **`src/completions.rs` + `affi --completions {bash,zsh,fish,powershell}`** (synthesis
   P1, 03 §3.6) — generated from **W4's registry**, covering all 67 verbs and value-choices.
   Adds PowerShell (the gap in §2.6). Replaces the hand-authored `completions/*`.
4. **`--color auto|always|never` honoring `NO_COLOR` + isatty** (03 §2.4/P2) — gated on the
   `ui` feature; satisfies `#![deny(clippy::print_stdout)]` (`lib.rs:61`) because all stdout
   routes through `Out`.

**Sketch — completions over W4's registry:**

```rust
//! src/completions.rs — render from the live noun/verb/arg registry (W4-owned).
pub fn generate(shell: &str, bin: &str, mut w: impl std::io::Write) -> std::io::Result<()> {
    let model = crate::registry::all();       // W4 source-of-truth: nouns,verbs,args,choices
    match shell {
        "bash"       => render_bash(&model, bin, &mut w),
        "zsh"        => render_zsh(&model, bin, &mut w),
        "fish"       => render_fish(&model, bin, &mut w),
        "powershell" => render_powershell(&model, bin, &mut w),   // the new shell
        other => writeln!(w, "# unknown shell: {other}"),
    }
}
// Intercepted in lib.rs::run() before dispatch, mirroring --explain.
```

**Sketch — the per-verb migration shape (mechanical, identical across the cluster):**

```rust
// BEFORE (handlers.rs:284, repeated ~30x): stream split + magic string
pub fn assemble(out: Option<String>, format: Option<String>) -> Result<()> {
    let output = adapt(crate::cli::assemble(out.as_deref()))?;
    if format.as_deref() == Some("json") { println!("{}", to_pretty(&output)?); return Ok(()); }
    println!("assembled receipt -> {}", output.receipt_path); Ok(())
}
// AFTER: one contract; data->stdout; schema-stamped; yaml for free
pub fn assemble(out: Option<String>, o: crate::output::Out) -> Result<()> {
    let payload = adapt(crate::cli::assemble(out.as_deref()))?;   // already a Serialize struct
    o.emit(&payload, |s| { use std::fmt::Write;
        let _ = writeln!(s, "assembled receipt -> {}", payload.receipt_path);
        let _ = writeln!(s, "content address: {}", payload.content_address);
    });
    Ok(())
}
```

**Acceptance criteria.**
- All 67 verbs accept `--format human|json|yaml`, `--json`, `--quiet`, `--verbose`,
  `--color`; an unknown `--format` value is a USAGE error (exit 64) on **every** verb, not
  silently ignored.
- Zero `format!`-built JSON and zero in-handler `std::process::exit` remain in
  `handlers.rs` (enforced by a grep test in CI).
- `affi --completions powershell` emits a non-empty script; `affi receipt <TAB>` offers all
  67 verbs in each shell; `--format <TAB>` offers `human json yaml`.
- `NO_COLOR=1 affi receipt show r.json` and `affi … | cat` emit zero ANSI.
- A "stream discipline" test: every verb's machine payload appears on **stdout** and chatter
  on **stderr** under `--json`.

**Cross-workstream deps.** **W4** registry must be the single enumerator (completions +
arg-group); W3 consumes, never forks it. **W1** `Out`/`Diagnostic` finalized. Coordinate
the global-arg attach with **W4** so `affi` help grouping and completions stay aligned.

---

### Phase 2028 — Publish & freeze the machine contract (schemas + v1.0 stability)

**Objectives.** Promote the output JSON Schemas from "stamped strings" to a **published,
validated, frozen v1.0 public contract** with a written stability + deprecation policy.
This is the year scripts can depend on `affi` output and stop pinning binary versions.

**Deliverables.**

1. **Output JSON Schema per verb** — one `docs/schemas/<verb>.v1.json` for every verb that
   emits `--json`, generated from the `Serialize` payload types in `src/types.rs`
   (`EmitOutput`, `AssembleOutput`, `InspectionReport`, `Verdict`, …) and hand-finished for
   the ad-hoc `serde_json::json!` payloads (`verify_family` L400, `dora_metrics` L1356,
   etc.), which are first promoted to named structs.
2. **`affi --schema <name>` / `affi schema list`** — dump any schema offline (mirrors
   `--explain`/`--completions` intercept in `lib.rs:129`); CI validates fixtures against it.
3. **The Output-Schema Stability & Deprecation Policy (W3 charter doc)** — written, tested
   rules (full text in §3a below): every payload carries `"schema": "<name>/vN"`; additive
   changes never bump `N`; breaking changes bump `N` and keep `N-1` for a **≥18-month / two
   minor-release** window; a `golden-schemas/` corpus + a conformance test enforce it.
4. **Schema conformance gate in CI** — `affi <verb> … --json` output for a fixture corpus
   must validate against the published schema *and* against the previous version for the
   duration of its window.

**Sketch — schema emit + the versioning invariant as a test:**

```rust
//! src/schema.rs — one published schema per verb payload; offline dump.
pub fn dump(name: &str, mut w: impl std::io::Write) -> std::io::Result<()> {
    let v = match name {
        "verdict"  => schema_for::<crate::types::Verdict>("affidavit.verdict/v1"),
        "emit"     => schema_for::<crate::types::EmitOutput>("affidavit.emit/v1"),
        "assemble" => schema_for::<crate::types::AssembleOutput>("affidavit.assemble/v1"),
        "inspect"  => schema_for::<crate::types::InspectionReport>("affidavit.inspect/v1"),
        other => { return writeln!(w, "{{\"error\":\"unknown schema {other}\"}}"); }
    };
    writeln!(w, "{}", serde_json::to_string_pretty(&v).expect("schema serialize"))
}

#[cfg(test)]
mod contract {
    /// THE load-bearing stability test: schema id strings are frozen forever.
    #[test] fn schema_ids_are_stable() {
        for (verb, id) in REGISTERED_SCHEMAS {                  // (name, "affidavit.x/vN")
            assert_eq!(super::id_of(verb), id, "schema id for {verb} changed — bump vN + keep old");
        }
    }
    /// Every fixture's --json output validates against its published schema.
    #[test] fn fixtures_validate() { /* jsonschema::validate(payload, schema) */ }
}
```

**Acceptance criteria.**
- `docs/schemas/` contains one `*.vN.json` per `--json`-emitting verb; `affi schema list`
  enumerates them; `affi --schema verdict` prints valid JSON Schema offline.
- Every `--json` payload across all verbs validates against its published schema (CI gate).
- `schema_ids_are_stable` test passes; a deliberate breaking field change fails CI unless a
  new `vN` is added and the old retained.
- The policy doc (§3a) is checked in and referenced from `--help`.

**Cross-workstream deps.** **W9** Ecosystem & Standards is the natural consumer/co-author of
the published schemas (OCEL/SLSA/in-toto downstreams) — align schema field names with W9's
external mappings. **W10** Compliance consumes frozen schemas for audit evidence.

---

### Phase 2029 — Backward-compatibility tooling & the support matrix

**Objectives.** Make the now-frozen contract *survive evolution*. Build the tooling that
lets `affi` add features without breaking 2028's scripts, and publish a machine-readable
support/deprecation matrix.

**Deliverables.**

1. **`--output-version <vN>` opt-in pinning** — a caller can request a prior schema version
   for any verb; `affi` emits the old shape from the retained encoder for the duration of its
   window. Default remains the latest stable.
2. **Deprecation surfacing** — when a verb/flag/schema enters deprecation, a one-line
   `note()` to **stderr** (never stdout, so machine output stays clean) names the
   replacement and the removal date; suppressible via `--quiet` and `AFFI_NO_DEPRECATION`.
3. **`affi schema diff <name> vA vB`** — show the field-level delta between two schema
   versions (added/removed/typed-changed), so downstream owners see exactly what to migrate.
4. **Machine-readable support matrix** — `docs/schemas/SUPPORT.json` (and an `affi schema
   support` dump): per schema/verb → `introduced`, `deprecated`, `removed`, `replacement`.
   This is the contract's own changelog, parseable by downstream CI.
5. **Stable-codes audit** — a generated report proving no `AFFI-Exxx` was ever renumbered or
   reused since 2026 (the 03 §5 invariant, now mechanically verified across releases).

**Sketch — version pinning + the support matrix:**

```rust
//! Opt-in output pinning, honored uniformly by Out across all verbs.
pub struct Out { pub format: Format, pub output_version: Option<u32>, /* +color/quiet/verbose */ }
impl Out {
    pub fn emit<T: VersionedPayload>(&self, v: &T, human: impl FnOnce(&mut String)) {
        let want = self.output_version.unwrap_or(T::LATEST);
        match self.format {
            Format::Json => println!("{}", v.encode_as(want)),   // retained encoders per vN
            Format::Yaml => println!("{}", v.encode_yaml(want)),
            Format::Human => { let mut s = String::new(); human(&mut s); print!("{s}"); }
        }
    }
}

// docs/schemas/SUPPORT.json — the contract's machine-readable changelog:
// { "affidavit.verdict": { "v1": {"introduced":"2026H2","deprecated":null,"removed":null},
//                          "v2": {"introduced":"2029","deprecated":null,"removed":null,
//                                 "replaces":"v1"} } }
```

**Acceptance criteria.**
- `affi receipt verify r.json --json --output-version 1` reproduces the 2028 v1 payload
  byte-for-byte (golden-corpus test) even after a v2 ships.
- Deprecation notes appear only on stderr, name a replacement + removal date, and vanish
  under `--quiet`.
- `affi schema diff verdict v1 v2` lists the exact field delta; `affi schema support` emits
  `SUPPORT.json`.
- A CI report asserts the full `AFFI-Exxx` set is append-only since 2026.

**Cross-workstream deps.** **W4** registry feeds the support matrix (verb→schema mapping).
**W9/W10** consume `SUPPORT.json` for their own downstream pinning and audit trails.

---

### Phase 2030 — Lock v2.0, contract-as-test, long-term guarantee

**Objectives.** Declare a **stable, versioned, fully-scriptable public CLI contract** as a
first-class, test-enforced artifact with a published long-term support guarantee. The
contract is now something you can build a business on.

**Deliverables.**

1. **Contract v2.0 freeze** — consolidate any v2 schemas, retire windows that have fully
   expired (with the matrix proving the window was honored), and publish the v2.0 baseline.
2. **`affi contract` verb** — one command that emits the *entire* public contract as data:
   every verb, its args, its exit codes, its schema id + version, deprecation status. The
   single artifact a downstream integrator or auditor consumes instead of reading source.
3. **Contract conformance suite (golden CLI corpus)** — a frozen set of
   invocation→(stdout-schema, exit-code) expectations covering every verb in every format;
   any drift from the published contract fails CI. This is the contract *as executable
   spec*.
4. **Long-term support policy** — written guarantee: stable codes are permanent; schema
   `vN` support window ≥18 months; exit codes `0/2` (ACCEPT/REJECT) never change meaning;
   `64/65/69/74` only refine. Published in `docs/schemas/POLICY.md` and surfaced in
   `affi --help`.

**Sketch — the whole contract as one emittable artifact + the conformance gate:**

```rust
//! `affi contract` — the entire public CLI contract as one Serialize value.
#[derive(serde::Serialize)]
pub struct PublicContract {
    pub affi_version: &'static str,
    pub verbs: Vec<VerbContract>,        // from W4 registry
    pub exit_codes: Vec<ExitCodeRow>,    // from the §3a catalog
    pub schemas: Vec<SchemaRef>,         // name, version, status (active|deprecated)
}
#[derive(serde::Serialize)]
pub struct VerbContract { pub noun: &'static str, pub verb: &'static str,
    pub args: Vec<ArgSpec>, pub output_schema: &'static str, pub exit_codes: Vec<i32> }

#[cfg(test)]
mod conformance {
    /// Frozen invocation -> (schema id, exit code). Drift = broken public contract.
    const GOLDEN: &[(&str, &str, i32)] = &[
        ("receipt verify ok.json --json",       "affidavit.verdict/v1", 0),
        ("receipt verify tampered.json --json",  "affidavit.verdict/v1", 2),
        ("receipt emit --type build … --json",   "affidavit.emit/v1",    0),
        // … one row per (verb, format) across all 67 verbs.
    ];
    #[test] fn cli_matches_published_contract() { /* run, parse .schema, assert exit */ }
}
```

**Acceptance criteria.**
- `affi contract --json` validates against `affidavit.contract/v1` and lists all 67 verbs
  with args, exit codes, and schema versions.
- The golden CLI conformance corpus passes for every verb in every format; any unannounced
  change to output shape or exit code fails CI.
- `docs/schemas/POLICY.md` states the LTS guarantee and is linked from `--help`.
- No `AFFI-Exxx` renumbered, no `vN` window violated, since 2026 (mechanically proven).

**Cross-workstream deps.** `affi contract` is the integration point every other workstream
publishes through — **W4** (verb/arg registry), **W7** (verify exit codes/stage names),
**W8** (signing-verb output schemas), **W9/W10** (external-standard + compliance schema
refs). W3 owns the *envelope*; each workstream owns the *contents* of its verbs.

---

### 3a. The Output-Schema Stability & Deprecation Policy (W3 charter, drafted 2028, enforced through 2030)

The normative core of W3's multi-year value. Tested, not just documented.

1. **Identity.** Every machine payload carries `"schema": "affidavit.<verb>/vN"`. The id
   string is the contract; the in-code discriminant is incidental (mirrors 03's stable-code
   rule).
2. **Additive is free.** Adding an *optional* field, a new enum case in a non-exhaustive
   position, or a new verb does **not** bump `N`. Consumers must ignore unknown fields.
3. **Breaking bumps `N` and retains `N-1`.** Removing/renaming a field, changing a type, or
   changing semantics ships a new `vN` *and* keeps the `vN-1` encoder for a **≥18-month /
   two-minor-release** support window, opt-in via `--output-version` (2029).
4. **Exit codes are permanent.** `0`=OK/ACCEPT and `2`=REJECT never change meaning
   (`golden_run.sh`/CI keep working); `64/65/69/74` only *refine* the legacy `1`. No exit
   code is ever reused for a new meaning.
5. **Stable error codes are forever.** A shipped `AFFI-Exxx` is never renumbered or reused;
   deprecate and add. Enforced by `schema_ids_are_stable` + the stable-codes audit (2029).
6. **Deprecation is loud on stderr, silent on stdout.** Notices go to `note()` (stderr),
   name the replacement and removal date, and never pollute machine output; suppressible.
7. **The corpus is the proof.** A `golden-schemas/` + golden-CLI corpus turns every rule
   above into a failing test on violation. The policy is enforced by CI, not goodwill.

---

## 4. Definition of done @2030

W3 is done when `affi` ships a **stable, versioned, fully-scriptable public CLI contract**:

- [ ] **Self-explaining.** `affi --explain <CODE>` resolves every shipped error and exit
  code offline; `affi why <RECEIPT>` narrates any verdict in plain English — both with a
  `--json` form. A first-time user fixes their own error in-terminal.
- [ ] **Uniform.** All 67 verbs honor one output contract — `--format human|json|yaml`,
  `--json`, `--quiet`, `--verbose`, `--color` — with data→stdout / chatter→stderr
  discipline. No verb is special; no `--format` is silently ignored; no `format!`-built
  JSON or in-handler `process::exit` remains.
- [ ] **Discoverable.** Generated completions for bash, zsh, fish, **and powershell** cover
  every verb and value-choice, regenerated from W4's registry — zero hand-maintained
  completion debt.
- [ ] **Machine-contracted.** A published JSON Schema per verb (`docs/schemas/`), dumpable
  via `affi --schema`/`affi contract`, validated against a golden corpus in CI.
- [ ] **Versioned & guaranteed.** Schema versioning + the §3a deprecation policy +
  `--output-version` pinning + `SUPPORT.json` matrix give downstream scripts an ≥18-month
  window on every breaking change and a written LTS guarantee through and beyond 2030.
- [ ] **Enforced.** The contract is *executable*: stable-code, schema-id, stream-discipline,
  and golden-CLI conformance tests fail CI on any unannounced drift.

The test that it worked: **a CI script written in 2028 against `affi … --json` still parses
in 2030**, and an error a user hits in 2030 is one `--explain` away from a fix — without
ever consulting our changelog.

---

## 5. Cross-workstream dependencies (consolidated)

| W3 needs / provides | Partner | Direction | Detail |
|---|---|---|---|
| `diag.rs` (`Code`/`ExitCode`/`Diagnostic`), `output.rs` (`Format`/`Out`), bug fixes B1/B2/B6 | **W1** Foundations | **W3 ← W1** (hard dep, 2026 H2) | W3 builds `--explain`/`why`/schemas *on* these primitives; W3 authors the catalog rows, W1 owns the enums. |
| Verb **registry** (nouns/verbs/args/value-choices) | **W4** Onboarding/Registry | **W3 ← W4** | Single enumerator for completions (2027), `--explain` titles, schema generation, `affi contract`. W3 consumes, never forks. |
| Output contract / `Out` discipline + schemas | **W2** Doctor, **W5** Automation, **W6** Interactive | **W3 → them** | Their verbs (`doctor`, `watch`, hooks, REPL) emit through W3's `Out` and publish W3-versioned schemas; their logic is theirs. |
| Verify exit codes, stage names, `Verdict`/`CheckOutcome` shapes | **W7** Verification Engine | **W3 ↔ W7** | `why`'s stage→Code map and the `--explain` verify rows track W7's stages; W7 must not silently rename stages (would break `why` + golden corpus). |
| Signing/attestation verb output schemas | **W8** Cryptography & Trust | **W3 ↔ W8** | `assemble-with-signature`/`attest`/`notarize`/`sign` payloads get named structs + published schemas under W3's versioning. |
| Published JSON Schemas + external-standard field mapping | **W9** Ecosystem & Standards | **W3 ↔ W9** | Co-author 2028 schemas so field names align with OCEL/SLSA/in-toto; W9 consumes `SUPPORT.json`. |
| Frozen schemas + `SUPPORT.json` + exit-code catalog as audit evidence | **W10** Compliance & Governance | **W3 → W10** | W10 cites the versioned contract and deprecation windows as governance artifacts. |

**Doctrine check (final).** Every item above is presentation, contract, or tooling over the
*existing* verdict. `--explain` reads a static table; `why` re-runs `cli::verify`
verbatim; completions/schemas/`contract` enumerate metadata. None mints a verdict, mutates
a chain, or lets a tool decide whether work is honest. **Certify, don't decide** holds end
to end.
