# 03 — DX & CLI Ergonomics: The Self-Explaining `affi`

> **Scope:** the moment-to-moment ergonomics every user hits on *every* command —
> errors, exit codes, output format, color, completions, and the gap between a
> bare `REJECT` and *understanding why*.
>
> **Doctrine alignment:** "certify, don't decide." None of this changes the
> verdict. It changes how a human or a script *experiences* the verdict.
>
> **Caveat:** code below is **compilable-style prototype**, not built here. `affi`
> depends on five `26.6` sibling crates (`clap-noun-verb`, `clnrm-core`,
> `wasm4pm`, `lsp-max`, `wasm4pm-compat`) absent from a lone checkout, so
> `cargo build`/`test` will not resolve. Nothing in this doc modifies existing
> files; it specifies one new module surface plus an ontology delta.

---

## 1. Vision — the 1000x leap

Today `affi` *works* but it does not *teach*. A failure prints
`Execution error: <stringified anyhow chain>` to stderr and exits `1`. A
`REJECT` prints to **stderr** (where pipes can't see it), exits `2`, and leaves
the operator to read seven stage lines and guess which one mattered. JSON exists
on *some* verbs as the magic string `"json"`. There is no `yaml`, no `--json`
flag, no exit-code catalog, no `--explain`, no `why`, and the shell completions
know about 4 of 59+ verbs.

The 1000x version treats the CLI as a **product with an API contract**:

1. **Every error is a teachable diagnostic** — stable code (`AFFI-E031`),
   one-line cause, the offending input with a caret span, and an *actionable*
   fix. `rustc`/`miette` grade, achieved with ~200 lines and zero new deps.
2. **Every exit code is documented and queryable** — `affi --explain AFFI-E031`
   prints the full story offline; CI greps the catalog, not our changelog.
3. **Every verb honors one output contract** — `--format human|json|yaml`,
   `--json` shorthand, `--quiet`, `--verbose`, `NO_COLOR`/`--color`, with
   **stable, versioned schemas** on a single stream discipline (data→stdout,
   chatter→stderr).
4. **`affi why <receipt>` bridges verdict→understanding** — plain-English "this
   was rejected at *chain_integrity* because event `evt-3` was edited; here is
   the byte that broke and the command to confirm."
5. **Help that teaches** — `affi receipt verify --help` ends with *runnable*
   examples and a "See also" list, not just a flag table.

The north star: **a first-time user fixes their own error without leaving the
terminal, and a CI script parses every verb the same way, forever.**

---

## 2. Current state — audit (cited, specific)

### 2.1 Output format: three bugs in one contract

The only format check in the codebase is the literal string `"json"`:

- `src/handlers.rs:101` — `if format.as_deref() == Some("json")` (the
  `print_json_or` helper), repeated **inline ~30 times** across handlers
  (`emit` L122, `assemble` L284, `verify` L348, `inspect` L734, `stats` L832,
  `dora_metrics` L1383, …).

Concrete inconsistencies:

| Problem | Evidence | Impact |
|---|---|---|
| No `human` / `yaml` literal — only `json` or "else" | `handlers.rs:101,122,348` | `--format yaml` silently falls back to human text; `--format=jsonl` does nothing |
| Human output streams are **mixed** | `emit` prints to **stdout** (`handlers.rs:127`); `verify`/`show`/`inspect`/`stats`/`diff` print to **stderr** (`handlers.rs:356-365, 687-719, 754, 844`) | `affi receipt show r.json > out.txt` captures nothing; pipelines break unpredictably |
| `--format` is **per-verb**, not global | ontology `affi:FormatArg` attached individually (`ontology/affi-cli.ttl:141,168,183,213,229,267,282`) and **missing** on `replay` (L297), `model` (L312), `conformance`, `diagnose` (L14 wrapper has no `format`) | `affi receipt diagnose r.json --json` is a usage error on some verbs, accepted-then-ignored on others |
| Hand-rolled JSON via `format!` | `assemble_with_signature` (`handlers.rs:303`), `assemble_and_notarize` (L324), `emit_batch` (L157) build JSON by string interpolation | un-escaped values → invalid JSON when a path contains `"` |
| `visualize` takes `format: String` (required) | `handlers.rs:1001`, wrapper differs from the `Option<String>` everywhere else | inconsistent surface; `--format` is mandatory for one verb only |
| No `--json` shorthand, no `--quiet`/`--verbose` | nowhere in `cli.rs`/`handlers.rs`/ontology | every script must spell `--format json` and there is no way to silence chatter |

### 2.2 Errors: rich type, flattened to mush

`src/error.rs` already defines a strong `AffidavitError` (L178-251) with 18
variants plus sub-enums (`OcelError`, `ChainError`, `ShardingError`, …). But the
boundary throws the structure away:

- `src/handlers.rs:18-57` `to_noun_verb()` maps **every** variant to
  `NounVerbError::execution_error(format!("…: {e}"))` — a flat string.
- `src/handlers.rs:59-61` `adapt()` collapses *all* `anyhow` errors into
  `AffidavitError::Execution(format!("{e:#}"))` before that. So the carefully
  typed `Io` / `Json` / `AdmissionRefused` distinction is **gone** by the time a
  user sees it.
- No variant carries a **code**, a **hint**, or the **offending span**. The
  `OcelError::MalformedObjectRef(String)` (`error.rs:27`) knows the bad input but
  presents it as `object ref 'x' is not in 'id:type'…` with no caret, no fix.

### 2.3 Exit codes: two values, ad-hoc

- `src/cli.rs:110-138` `verify()` returns `(i32, Verdict)` — `0` or `2`.
- The exit is forced by scattered `std::process::exit(code)` /
  `std::process::exit(2)` (`handlers.rs:352,366,562`) **inside** handlers.
- `verify_sla` failure returns a generic `execution_error` (`handlers.rs:463`)
  → process exit `1` with no code, undistinguishable from an IO error.
- There is **no catalog**: usage errors, IO errors, and REJECT all collapse onto
  `1`/`2` with no stable mapping a script can branch on.

### 2.4 Color & accessibility

- `colored` and `indicatif` exist behind the `ui` feature (`Cargo.toml:80-81,153`),
  but no handler consults `NO_COLOR`, a `--color` flag, or `isatty`. Output is
  plain today; the moment color is added it will leak ANSI into pipes.

### 2.5 Completions: stale and partial

- Hand-authored in `completions/` (`affi.bash`, `affi.zsh`, `affi.fish`) — headers
  explicitly say *"AUTHORED, NOT GENERATED … Keep it in sync"* (`completions/affi.bash:1-13`).
- They list **`receipt_verbs='emit assemble verify show'`** (`affi.bash:35`) — 4
  of 59+ verbs in `src/verbs/mod.rs`.
- **No PowerShell** completion file at all.
- `--format` completes only `json` (`affi.bash:38`), matching the broken contract.
- `CONTRIBUTING.md:199-224` documents the manual install/sync burden.

### 2.6 Discoverability: no `--explain`, no `why`

- No `--explain` flag and no `why` verb exist. `explain-incident`
  (`ontology/affi-cli.ttl:954`) is unrelated (root-cause tracing across receipts).
- `diagnose` (`handlers.rs:974`) is the closest thing, but it requires the `lsp`
  feature and emits LSP `line:character` diagnostics — not plain English, and a
  hard error (`"lsp feature not enabled"`, L996) without it.

**Verdict on current state:** the *primitives* are excellent (typed errors,
typed `Verdict`/`CheckOutcome`, per-stage `detail` strings). They are simply not
surfaced. The 1000x win is almost entirely a **presentation seam**, not new
verification logic.

---

## 3. Proposed design

### 3.1 The diagnostic core (`src/diag.rs`, new)

One module owns codes, exit codes, and rendering. No new dependency (miette is
the *aspiration*; we ship its *shape*).

```rust
//! src/diag.rs — stable error codes, exit-code catalog, teachable diagnostics.
//! Zero new deps; `miette`-grade output via std formatting + optional `colored`.

use std::fmt;

/// Stable, documented process exit codes. The CLI converts every terminal
/// outcome into exactly one of these — no scattered `process::exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Verdict ACCEPT, or any verb that completed successfully.
    Ok = 0,
    /// Generic / uncategorized runtime failure (last resort).
    Failure = 1,
    /// Verdict REJECT — the receipt did not certify. NOT an error: a result.
    Reject = 2,
    /// Bad invocation: unknown verb/flag, missing arg, invalid `--format`.
    Usage = 64,        // sysexits.h EX_USAGE
    /// Input file unreadable / parse failed (the receipt JSON is broken).
    DataErr = 65,      // EX_DATAERR
    /// I/O failure: file not found, permission denied, disk full.
    IoErr = 74,        // EX_IOERR
    /// A required cargo feature was not compiled in (e.g. `lsp`, `discovery`).
    Unavailable = 69,  // EX_UNAVAILABLE
}

impl ExitCode {
    pub fn code(self) -> i32 { self as i32 }
}

/// Stable error-code identifiers. The STRING is the contract (greppable in CI,
/// linkable in docs); the discriminant is incidental. Never renumber a shipped
/// code — add a new one and deprecate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    MalformedObjectRef, // AFFI-E010
    EmptyEventType,     // AFFI-E011
    ReceiptNotFound,    // AFFI-E020
    ReceiptParse,       // AFFI-E021
    ChainMismatch,      // AFFI-E030  (verify stage: chain_integrity)
    SeqGap,             // AFFI-E031  (verify stage: continuity)
    DuplicateEventId,   // AFFI-E032  (verify stage: continuity)
    BadCommitment,      // AFFI-E033  (verify stage: verify_commitments)
    FormatMismatch,     // AFFI-E034  (verify stage: check_format)
    AdmissionRefused,   // AFFI-E040
    FeatureDisabled,    // AFFI-E090
    BadFormatFlag,      // AFFI-E001
}

impl Code {
    // Backed by one static table row per code, so id/title/explanation/exit
    // stay in lockstep. Shown as representative arms; every code has a row.
    pub fn id(self) -> &'static str {            // STABLE contract string
        match self {
            Code::SeqGap             => "AFFI-E031",
            Code::ChainMismatch      => "AFFI-E030",
            Code::MalformedObjectRef => "AFFI-E010",
            Code::FeatureDisabled    => "AFFI-E090",
            _ => CODES.iter().find(|r| r.code == self).unwrap().id, // table lookup
        }
    }
    pub fn title(self) -> &'static str {         // `--explain` headline
        match self {
            Code::SeqGap        => "event sequence numbers are not contiguous from 0",
            Code::ChainMismatch => "recomputed chain hash does not match the stored chain_hash",
            _ => CODES.iter().find(|r| r.code == self).unwrap().title,
        }
    }
    pub fn explanation(self) -> &'static str { /* multi-paragraph WHY + FIX + CONFIRM, per row */ todo!() }
    pub fn exit(self) -> ExitCode {              // → exit-code catalog
        match self {
            Code::BadFormatFlag   => ExitCode::Usage,
            Code::ReceiptNotFound => ExitCode::IoErr,
            Code::ReceiptParse    => ExitCode::DataErr,
            Code::FeatureDisabled => ExitCode::Unavailable,
            // verify-stage codes are *results*, not crashes:
            Code::ChainMismatch | Code::SeqGap | Code::DuplicateEventId
            | Code::BadCommitment | Code::FormatMismatch => ExitCode::Reject,
            _ => ExitCode::Failure,
        }
    }
}

// Example explanation row (one per Code) — what `affi --explain AFFI-E031` prints:
//   WHY:    Stage 4 (continuity) requires each event's `seq` to equal its
//           position, from 0 with no gaps — a gap means an event was dropped,
//           reordered, or inserted out of band.
//   FIX:    Re-`assemble` from a correct working set; you cannot patch `seq`
//           alone because the chain hash is bound to event bytes.
//   CONFIRM: affi receipt why <receipt>

/// A teachable diagnostic: code + cause + (optional) offending input span + hint.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: Code,
    pub cause: String,                 // what happened, concretely
    pub input: Option<SourceSpan>,     // the offending text + caret range
    pub hint: Option<String>,          // the single most actionable next step
}

/// The offending input with a byte range to underline (rustc-style caret).
#[derive(Debug, Clone)]
pub struct SourceSpan {
    pub label: String,   // e.g. "object spec" / "receipt path"
    pub text: String,    // the literal the user passed
    pub start: usize,    // caret start (byte offset into `text`)
    pub len: usize,      // caret length
}

impl Diagnostic {
    pub fn new(code: Code, cause: impl Into<String>) -> Self {
        Self { code, cause: cause.into(), input: None, hint: None }
    }
    pub fn with_span(mut self, s: SourceSpan) -> Self { self.input = Some(s); self }
    pub fn with_hint(mut self, h: impl Into<String>) -> Self { self.hint = Some(h.into()); self }
    pub fn exit(&self) -> ExitCode { self.code.exit() }

    /// Machine form: stable schema `affidavit.diagnostic/v1` with fields
    /// { schema, code, title, cause, hint, exit_code, span:{label,text,start,len} }.
    pub fn to_json(&self) -> serde_json::Value { /* serde_json::json!({ … }) */ todo!() }
}

/// rustc/miette-style human render (color decided once, upstream):
///   error[AFFI-E010]: <title>
///     cause: <cause>
///      --> <span.label>: <span.text>
///                        ^^^^^^         (caret at span.start, len span.len)
///     hint: <hint>
///     help: run `affi --explain AFFI-E010` for the full explanation
impl fmt::Display for Diagnostic { /* writeln! the block above */ }
```

**Where it plugs in.** `AffidavitError` (`src/error.rs`) gains one method —
`fn diagnostic(&self) -> Diagnostic` — mapping each existing variant to a `Code`
+ hint. `OcelError::MalformedObjectRef(s)` (`error.rs:27`) builds a
`SourceSpan { label: "object spec", text: s.clone(), start: caret_of(&s), .. }`.
Nothing about verification logic changes; we only *enrich the boundary* that
`handlers.rs:18-61` currently flattens.

### 3.2 The output contract (`src/output.rs`, new)

A single abstraction every verb funnels through. Resolves format/color/stream
once, enforces the data→stdout / chatter→stderr split, and guarantees valid,
schema-stamped machine output.

```rust
//! src/output.rs — one output contract for every verb.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format { Human, Json, Yaml }

impl Format {
    /// Resolve from `--format`, the `--json` shorthand, and a stdout-isatty
    /// default. Unknown values are a USAGE error (AFFI-E001), never silent.
    pub fn resolve(flag: Option<&str>, json_shorthand: bool) -> Result<Self, crate::diag::Diagnostic> {
        if json_shorthand { return Ok(Format::Json); }
        match flag {
            None | Some("human") => Ok(Format::Human),
            Some("json") => Ok(Format::Json),
            Some("yaml") => Ok(Format::Yaml),
            Some(other) => Err(crate::diag::Diagnostic::new(
                crate::diag::Code::BadFormatFlag,
                format!("unknown --format value {other:?}"),
            ).with_hint("valid values: human, json, yaml")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChoice { Auto, Always, Never }

impl ColorChoice {
    /// `NO_COLOR` (any value) AND a non-tty stdout both force color OFF.
    pub fn enabled(self) -> bool {
        match self {
            ColorChoice::Never  => false,
            ColorChoice::Always => true,
            ColorChoice::Auto   => std::env::var_os("NO_COLOR").is_none() && atty_stdout(),
        }
    }
}

/// The handle every verb receives. Carries the resolved contract so handlers
/// never touch `println!`/`eprintln!` directly again.
pub struct Out {
    pub format: Format,
    pub color: bool,
    pub quiet: bool,    // suppress non-essential chatter (still emit data + errors)
    pub verbose: bool,  // extra stage/timing detail to stderr
}

impl Out {
    /// DATA → stdout. The payload is rendered per the resolved format.
    /// Every machine payload is a `serde::Serialize` with a `schema` field, so
    /// JSON and YAML are two encodings of ONE typed value (no string-built JSON).
    pub fn emit<T: serde::Serialize>(&self, value: &T, human: impl FnOnce(&mut String)) {
        match self.format {
            Format::Json => println!("{}", serde_json::to_string_pretty(value).unwrap()),
            Format::Yaml => println!("{}", serde_yaml::to_string(value).unwrap()),
            Format::Human => {
                let mut s = String::new();
                human(&mut s);
                print!("{s}");
            }
        }
    }
    /// CHATTER → stderr. Suppressed under `--quiet`. Never machine-parsed.
    pub fn note(&self, msg: impl AsRef<str>) {
        if !self.quiet { eprintln!("{}", msg.as_ref()); }
    }
    /// ERROR → stderr, always (even under --quiet). Returns the exit code so
    /// `main` does the single `process::exit`, not the handler.
    pub fn fail(&self, d: &crate::diag::Diagnostic) -> i32 {
        match self.format {
            Format::Human => eprintln!("{d}"),
            _ => eprintln!("{}", serde_json::to_string_pretty(&d.to_json()).unwrap()),
        }
        d.exit().code()
    }
}
```

This is the single fix for §2.1: `emit`'s stdout (`handlers.rs:127`) and
`verify`'s stderr (`handlers.rs:356`) both become `out.emit(...)` /
`out.note(...)`. JSON-by-`format!` (`handlers.rs:303,324,157`) is deleted in
favor of typed `Serialize` payloads — escaping bugs vanish.

### 3.3 Global flags via the ontology (one source of truth)

The arg surface is generated from `ontology/affi-cli.ttl` (L141…). Rather than
hand-editing 59 wrappers, define a **shared global-arg group** once and attach it
to the noun, so *every* verb inherits the contract:

```turtle
# ontology/affi-cli.ttl  (delta — illustrative cnv vocabulary)
affi:GlobalArgs a cnv:ArgGroup ;
    cnv:hasArguments
        affi:FormatArg ,   # --format human|json|yaml   (existing arg, now global)
        affi:JsonArg ,     # --json    (shorthand for --format json)
        affi:QuietArg ,    # --quiet   -q
        affi:VerboseArg ,  # --verbose -v
        affi:ColorArg .    # --color auto|always|never

affi:FormatArg a cnv:Argument ;
    cnv:hasArgumentName "format" ;
    cnv:hasValueChoices ( "human" "json" "yaml" ) ;   # <- enables completion
    cnv:hasDefault "human" .

affi:ReceiptNoun cnv:inheritsArgs affi:GlobalArgs .   # all verbs get the group
```

Result: `replay`/`model`/`conformance`/`diagnose` (which lack `FormatArg` today
— `ttl:297,312`) get `--format` for free, and the wrapper bodies just forward an
`Out` built from the inherited flags. **Re-run `ggen sync`; no hand-edits.**

### 3.4 `affi --explain <CODE>` (offline doctor)

A top-level flag intercepted before noun-verb dispatch (in `lib.rs::run`, the
single hand-written seam, `lib.rs:129`). Looks up `Code` by `id()` and prints
`title()` + `explanation()`. No receipt, no network, works in an air-gapped CI
box. `--format json` returns the catalog entry as data.

```rust
// in run(): intercept the two teaching paths before clap_noun_verb::run()
pub fn run() -> clap_noun_verb::Result<()> {
    let argv: Vec<String> = std::env::args().collect();
    if let Some(i) = argv.iter().position(|a| a == "--explain") {
        let code = argv.get(i + 1).map(String::as_str).unwrap_or("");
        std::process::exit(crate::diag::explain(code)); // prints + returns exit
    }
    clap_noun_verb::run()
}
```

### 3.5 `affi why <RECEIPT>` (verdict → plain English)

A new verb that re-uses the *existing* pipeline (`verifier::verify`,
`verifier.rs:43`) and the per-stage `CheckOutcome.detail` strings, then maps the
first failing stage to a `Code` and renders a narrative. **No new verification
code** — it is a presentation over `Verdict` (`types.rs:292`).

```rust
//! src/verbs/why.rs (thin wrapper) -> crate::handlers::why
/// Explain a receipt's verdict in plain English (which stage, why, how to fix).
#[verb("why", "receipt")]
pub fn why(receipt: String, format: Option<String>, json: Option<bool>) -> Result<()> {
    crate::handlers::why(receipt, format, json.unwrap_or(false))
}

// handlers.rs — pure presentation over the EXISTING pipeline:
pub fn why(receipt: String, format: Option<String>, json: bool) -> Result<()> {
    let out = Out::from_flags(format.as_deref(), json)?;
    let parsed  = crate::cli::show(&receipt)?;          // existing loader
    let verdict = crate::verifier::verify(&parsed);     // existing pipeline
    let story   = WhyReport::from_verdict(&receipt, &verdict); // maps stage -> Code
    out.emit(&story, |s| story.render_human(s));
    if !verdict.accepted { std::process::exit(story.code.exit().code()); }
    Ok(())
}

/// `#[derive(Serialize)]`, schema "affidavit.why/v1":
/// { receipt, accepted, failing_stage, code (→id()), plain_english, next_command }
pub struct WhyReport { /* fields above */ }
```

The stage→Code map is total and tiny: `chain_integrity → ChainMismatch`,
`continuity → SeqGap | DuplicateEventId` (disambiguated by `detail`),
`verify_commitments → BadCommitment`, `check_format → FormatMismatch`. Every
arm already has a human `detail` from `verifier.rs:116,145,177,197`.

### 3.6 Completions: generate, don't hand-author

The ontology already enumerates nouns, verbs, args, and (with §3.3) value
choices. A `--completions <shell>` hidden flag walks the same registry
`clap-noun-verb` builds and prints a script for **bash, zsh, fish, AND
powershell** to stdout — making `completions/*` (`completions/affi.bash:35`,
stale at 4 verbs) generated artifacts, not maintenance debt.

```rust
// in run(), before dispatch:
if let Some(i) = argv.iter().position(|a| a == "--completions") {
    let shell = argv.get(i + 1).map(String::as_str).unwrap_or("bash");
    crate::completions::generate(shell, "affi", std::io::stdout());
    return Ok(());
}
```

```rust
//! src/completions.rs — render from the live noun/verb/arg registry.
pub fn generate(shell: &str, bin: &str, mut w: impl std::io::Write) {
    let model = clap_noun_verb::registry();        // nouns, verbs, args, choices
    match shell {
        "bash"       => render_bash(&model, bin, &mut w),
        "zsh"        => render_zsh(&model, bin, &mut w),
        "fish"       => render_fish(&model, bin, &mut w),
        "powershell" => render_powershell(&model, bin, &mut w),
        other => { let _ = writeln!(w, "# unknown shell: {other}"); }
    }
}
```

Install becomes one line per shell (replacing the manual `cp` matrix in
`CONTRIBUTING.md:209-221`):

```bash
affi --completions bash > ~/.local/share/bash-completion/completions/affi
affi --completions zsh  > ~/.zsh/completions/_affi
affi --completions fish > ~/.config/fish/completions/affi.fish
affi --completions powershell >> $PROFILE
```

### 3.7 Help that teaches

Each verb's doc comment (the `#[verb("verify","receipt")]` summary, e.g.
`src/verbs/verify.rs:12`) is extended with an `EXAMPLES` + `SEE ALSO` block. The
existing `format_help_markdown` renderer (`src/cli.rs:206`) already converts
Markdown headings/code-fences to terminal text — so help authored as Markdown in
the ontology `cnv:verbAbout` renders for free:

```
EXAMPLES
  # Certify a receipt, machine-readable, branch on the verdict in CI:
  affi receipt verify build.json --json || echo "rejected: $?"

  # Understand a REJECT in plain English:
  affi receipt why build.json

SEE ALSO
  affi receipt why · affi receipt diagnose · affi --explain AFFI-E030
```

---

## 4. CLI UX — before / after

### 4.1 An error case (malformed object ref)

**Before** (`handlers.rs:18-61` flattens `OcelError::MalformedObjectRef`):
```
$ affi receipt emit --type build --object "repo-main" --payload p.txt
Execution error: parsing object specs (expected id:type or id:type:qualifier): object ref 'repo-main' is not in 'id:type' or 'id:type:qualifier' form
$ echo $?
1
```

**After**:
```
$ affi receipt emit --type build --object "repo-main" --payload p.txt
error[AFFI-E010]: object ref is not in id:type[:qualifier] form
  cause: missing ':' separator between id and type
   --> object spec: repo-main
                    ^^^^^^^^^
  hint: write it as `repo:main` (id `repo`, type `main`) — or add a qualifier: `repo:main:fast`
  help: run `affi --explain AFFI-E010` for the full explanation
$ echo $?
64
```

### 4.2 A `--json` case (stable schema, one stream)

**Before** (`verify` prints human verdict to **stderr**, JSON only on the magic
string; stdout is empty for human mode — `handlers.rs:348-368`):
```
$ affi receipt verify ok.json > out.json ; cat out.json
            # empty: verdict went to stderr
$ affi receipt verify ok.json --format json | jq .accepted
true        # works, but only because someone knew the literal "json"
```

**After** (`--json` shorthand; data on stdout; schema-stamped):
```
$ affi receipt verify ok.json --json | jq '{ok: .accepted, schema}'
{
  "ok": true,
  "schema": "affidavit.verdict/v1"
}
$ affi receipt verify bad.json --json > v.json ; echo $?
2
$ jq -r '.outcomes[] | select(.passed==false) | .stage' v.json
chain_integrity
```

### 4.3 Completion install

**Before** (copy the stale, 4-verb hand-authored file — `CONTRIBUTING.md:209`):
```
$ cp completions/affi.bash ~/.local/share/bash-completion/completions/affi
$ affi receipt <TAB>
assemble  emit  show  verify          # missing 55+ verbs
```

**After** (generated from the live registry, all verbs + value choices):
```
$ affi --completions bash > ~/.local/share/bash-completion/completions/affi
$ affi receipt <TAB>
assemble   diagnose   emit       inspect    notarize   show       verify     why  …
$ affi receipt verify ok.json --format <TAB>
human  json  yaml                  # choices come from the ontology
```

### 4.4 `affi --explain 2` and `affi --explain AFFI-E030`

```
$ affi --explain AFFI-E030
AFFI-E030  chain_integrity: recomputed chain hash does not match stored chain_hash
exit code: 2 (REJECT)

WHY: Stage 3 recomputes the rolling BLAKE3 over the events in order and compares
it to the receipt's stored `chain_hash`. A mismatch means at least one event's
bytes changed after sealing — a single flipped field propagates to every later
link. This is exactly the tamper-evidence the chain is built to provide.

FIX:
  * You cannot patch the hash by hand; it is bound to event bytes.
  * Re-`assemble` from a correct working set (`affi receipt assemble`).
  * If you expected ACCEPT, the file was modified in transit — re-fetch it.

CONFIRM: affi receipt why <receipt>

$ affi --explain 2          # bare exit code also resolves
exit code 2 = REJECT — a receipt failed certification. This is a *result*, not a
crash. Stage-specific codes: AFFI-E030 (chain), AFFI-E031 (continuity), … .
See `affi --explain AFFI-E0xx`.
```

### 4.5 `affi why receipt.json`

```
$ affi receipt why tampered.json
This receipt was REJECTED.

  stage:  chain_integrity  (3 of 7)
  why:    the chain hash stored in the file is
            203d3bbf…  (claimed)
          but recomputing it from the events yields
            a91f0c47…  (actual)
          → at least one event was edited after the receipt was sealed.

  the break is at:  event evt-3 (seq 3, type "test")
  most likely:      a field in evt-3 was changed by hand, or the file was
                    corrupted/modified in transit.

  fix:   re-assemble from the source events:  affi receipt assemble
  learn: affi --explain AFFI-E030

$ echo $?
2
$ affi receipt why ok.json
This receipt was ACCEPTED. All 7 stages passed under profile core/v1.
```

---

## 5. Integration & rollout

**Guiding principle:** the diagnostic/output seam is *additive* — verbs migrate
one at a time behind a shared helper, and the verdict logic in `verifier.rs` is
never touched. Existing tests keep passing because `Verdict`/`CheckOutcome`
shapes (`types.rs:269,292`) are unchanged; we add *new* schema-stamped wrappers.

### Retrofit recipe (per verb, ~10 min each)
1. Replace inline `format.as_deref() == Some("json")` (e.g. `handlers.rs:122`)
   with `let out = Out::from_flags(...)?;`.
2. Replace `println!`/`eprintln!` with `out.emit(&payload, |s| …)` (data) and
   `out.note(…)` (chatter). Delete `format!`-built JSON (`handlers.rs:157,303,324`).
3. Define/confirm the payload is a `Serialize` struct in `types.rs` (most already
   are: `EmitOutput`, `AssembleOutput`, `InspectionReport`, `Verdict`).
4. Stop calling `std::process::exit` in the handler (`handlers.rs:352,366,562`);
   return the `Diagnostic`/code and let `main` exit once.

### Phasing

| Phase | Item | Size | Why first |
|---|---|---|---|
| **P0** | `src/diag.rs`: `Code`, `ExitCode`, `Diagnostic`, `AffidavitError::diagnostic()` | **M** | Unblocks every other item; pure addition |
| **P0** | `src/output.rs`: `Format`/`ColorChoice`/`Out`; centralize stream split | **M** | Fixes the stdout/stderr bug (§2.1) that breaks pipes today |
| **P0** | `affi --explain <CODE>` + bare exit codes; **exit-code catalog in `--help`** | **S** | Cheap, high-trust; works offline; depends only on `diag.rs` |
| **P0** | Migrate the 4 core verbs (`emit`/`assemble`/`verify`/`show`) to `Out` | **M** | Highest-traffic; proves the seam |
| **P1** | `affi receipt why` verb (ontology + wrapper + `WhyReport`) | **M** | The marquee DX win; reuses existing pipeline |
| **P1** | Global args via ontology `GlobalArgs` group + `ggen sync` | **M** | Gives `--format/--json/--quiet/--verbose/--color` to all 59 verbs at once |
| **P1** | `--format yaml` (add `serde_yaml` dep) + `--json` shorthand everywhere | **S** | Completes the output contract |
| **P1** | `src/completions.rs` + `--completions {bash,zsh,fish,powershell}` | **L** | Replaces stale hand-authored files; adds PowerShell |
| **P2** | Color via `colored` honoring `NO_COLOR`/`--color`/isatty | **S** | Accessibility; gated on `ui` feature |
| **P2** | `EXAMPLES`/`SEE ALSO` in every verb's help (ontology `verbAbout` Markdown) | **M** | Teaching help; renders via existing `format_help_markdown` (`cli.rs:206`) |
| **P2** | Migrate remaining ~45 analytics verbs to `Out` | **L** | Mechanical; do in batches by cluster |
| **P2** | JSON schema doc (`docs/schemas/`) + a `--schema <name>` dump | **S** | Locks the machine contract for downstream tools |

### Compatibility & guardrails
- **Stable codes are forever.** A shipped `AFFI-Exxx` is never renumbered or
  reused; deprecate and add. A test asserts `Code::id()` strings never change.
- **Schema versioning.** Every machine payload carries `"schema": ".../vN"`. A
  breaking field change bumps `N`; the old encoder stays for one release.
- **Exit-code stability.** `0`/`2` keep their meaning (ACCEPT/REJECT) — existing
  `golden_run.sh` and CI keep working; `64/65/69/74` only *refine* the old `1`.
- **`#![deny(clippy::print_stdout)]`** (`lib.rs:61`) is satisfied because all
  stdout goes through `Out::emit` (one allowlisted module), not scattered prints.

---

### Appendix — exit-code catalog (also printed by `affi --help` footer)

| Code | Name | Meaning | Typical trigger |
|---:|---|---|---|
| `0` | OK | success / verdict ACCEPT | verify passed; any verb completed |
| `1` | FAILURE | uncategorized runtime failure | last-resort fallback |
| `2` | REJECT | receipt did not certify (a *result*) | any failing verify stage |
| `64` | USAGE | bad invocation | unknown verb/flag, bad `--format` |
| `65` | DATAERR | input could not be parsed | corrupt receipt JSON |
| `69` | UNAVAILABLE | required feature not compiled in | `diagnose` without `lsp` |
| `74` | IOERR | I/O failure | receipt file not found / unreadable |
