# `affi doctor` — Environment & Installation Health

> Status: design + prototype (not yet implemented)
> Scope: environment / install / config health. **Not** receipt-content health (owned elsewhere).
> Doctrine: **certify, don't decide.** `doctor` inspects your machine, reports facts, and *suggests* fixes. It never judges whether your provenance work is honest.

---

## 1. Vision — the concrete 1000x leap

Today, when `affi` misbehaves, the failure is *silent and displaced*. You run `affi receipt graph`, get `Error: discovery feature not enabled` (the literal string from `src/handlers.rs:891`), and now you are spelunking through `Cargo.toml`'s 30-feature matrix to guess which `--features` flag you forgot. You assemble a receipt, it won't verify on a teammate's box, and nobody realizes the binaries were built from different `GENESIS_SEED` releases. A stale `.affi/working.json` from last week silently prepends ghost events to today's chain. Every one of these is a 20–40 minute detour, and **none of them is your receipts being wrong** — it's your *environment* being wrong, which `affi` currently has no vocabulary to describe.

**The leap is not a prettier error. It is a single command that turns N invisible environment failure modes into one ranked, copy-pasteable checklist — and auto-repairs the safe ones.** `brew doctor` did this for Homebrew; `flutter doctor` made "is my SDK set up?" a 2-second question instead of a Stack Overflow thread.

A developer's **first 60 seconds** with `affi doctor`:

```text
$ affi doctor
affi doctor — environment & installation health   (affi 26.6.17)

  ok    binary           affi 26.6.17 on PATH (/usr/local/bin/affi)
  ok    toolchain        rustc 1.81.0 ≥ MSRV 1.78
  warn  features         built with [core]; `graph`/`model` need `discovery`
  fail  working-dir      .affi/working.json is 9 days old, 41 staged events
  warn  genesis          binary genesis tag v26.6.14 < pkg 26.6.17 (cross-box risk)
  ok    config           no AFFI_* overrides; using defaults
  warn  completions      shell completions not installed for bash
  ok    profile          default profile core/v1 recognized

  6 ok · 3 warn · 1 fail        run `affi doctor --fix` to apply 2 safe fixes
```

Ten seconds in, the developer *knows* the deploy that won't verify is the `genesis` mismatch, knows the ghost events come from a stale working file, and knows two of these are auto-fixable. That is the order-of-magnitude difference: **diagnosis time goes from "open the source and read feature flags" to "read four lines."**

---

## 2. Current state — what exists today

There is **no `doctor` verb** anywhere in the source (confirmed: `doctor` appears only in unrelated `thesis/` and `survey/` docs). The closest existing capability is `diagnose`, but it is *receipt-scoped*, not *environment-scoped*:

- **`src/verbs/diagnose.rs`** — `#[verb("diagnose", "receipt")] pub fn diagnose(receipt: String)` delegates to `crate::handlers::diagnose`.
- **`src/handlers.rs:973`** — `diagnose` runs `crate::cli::verify` and renders the resulting `Verdict` as LSP diagnostics. It requires a receipt path and, under `#[cfg(not(feature = "lsp"))]`, fails with `"lsp feature not enabled"` (`src/handlers.rs:996`). It tells you nothing about your *install*.
- **`src/verbs/inspect.rs`** / **`handlers.rs:724`** — structural analysis of a receipt's events/objects. Also receipt-scoped.

The verb-wrapper pattern this design must match is uniform and thin (see `diagnose.rs`, `inspect.rs`, `emit.rs`):

```rust
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

#[verb("inspect", "receipt")]
pub fn inspect(receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::inspect(receipt, format)   // delegates to handlers::*
}
```

Relevant existing building blocks I will reuse rather than reinvent:

- **`src/chain.rs`** constants: `WORKING_PATH = ".affi/working.json"` (`chain.rs:25`), `FORMAT_VERSION = "core/v1"` (`chain.rs:19`), `GENESIS_SEED = b"affidavit-v26.6.14-genesis"` (`chain.rs:22`), plus `load_working()` / `save_working()`.
- **`src/types.rs`**: `CheckOutcome { stage, passed, detail }` (`types.rs:270`) and `Verdict { accepted, profile, outcomes, reason }` (`types.rs:292`). `doctor`'s finding type is deliberately modeled on `CheckOutcome` so output feels native.
- **`Cargo.toml`** feature matrix (`default = ["core"]`, `core = ["wasm4pm-compat"]`, and `lsp`, `discovery`, `otel`, `metrics`, `webhook`, `gpu`, …). The handlers already gate behavior on these (e.g. `#[cfg(feature = "discovery")]` in `handlers.rs:828`/`866`).
- **`linkme = "0.3"`** is declared in `Cargo.toml:42` but **not used anywhere in `src/`** yet. `doctor` introduces the first `distributed_slice` registry — exactly the plugin-discovery use the dependency was added for.

---

## 3. Proposed design

### 3.1 The `DoctorCheck` trait

Each check is a small, self-contained unit yielding a `Finding`. Mirroring `CheckOutcome`'s field names keeps the mental model identical to the verifier.

```rust
//! src/doctor/mod.rs  (NEW — not created by this design doc)

use serde::Serialize;

/// Tri-state health status. NOTE: there is intentionally no "honest"/"dishonest"
/// status — doctor certifies environment facts and suggests fixes; it never
/// adjudicates the integrity of the user's work (doctrine: certify, don't decide).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status { Ok, Warn, Fail }

/// One diagnosis. Field layout deliberately echoes types::CheckOutcome so the
/// human/JSON renderers feel native to existing `verify`/`inspect` output.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable check name; also the key for `--check <name>` and `--fix`.
    pub check: String,
    pub status: Status,
    /// One line: what is true right now.
    pub finding: String,
    /// One line: the exact command or edit that resolves it. None when ok.
    pub remediation: Option<String>,
    /// True iff `--fix` can resolve this safely and idempotently.
    pub auto_fixable: bool,
}

impl Finding {
    pub fn ok(check: &str, finding: impl Into<String>) -> Self {
        Finding { check: check.into(), status: Status::Ok,
                  finding: finding.into(), remediation: None, auto_fixable: false }
    }
    pub fn warn(check: &str, finding: impl Into<String>, fix: impl Into<String>) -> Self {
        Finding { check: check.into(), status: Status::Warn,
                  finding: finding.into(), remediation: Some(fix.into()), auto_fixable: false }
    }
    pub fn fail(check: &str, finding: impl Into<String>, fix: impl Into<String>) -> Self {
        Finding { check: check.into(), status: Status::Fail,
                  finding: finding.into(), remediation: Some(fix.into()), auto_fixable: false }
    }
    pub fn fixable(mut self) -> Self { self.auto_fixable = true; self }
}

/// Outcome of an attempted auto-fix.
pub enum Fixed { Applied(String), Skipped(String) }

/// A single environment health check.
pub trait DoctorCheck: Sync {
    /// Stable identifier (kebab-case): "binary", "working-dir", "features", …
    fn name(&self) -> &'static str;
    /// Run read-only; never mutates the environment.
    fn run(&self, ctx: &DoctorCtx) -> Finding;
    /// Apply the safe fix. Default: nothing is auto-fixable.
    fn fix(&self, _ctx: &DoctorCtx) -> anyhow::Result<Fixed> {
        Ok(Fixed::Skipped("no automatic fix for this check".into()))
    }
}

/// Ambient facts gathered once and shared by all checks (keeps checks pure-ish
/// and cheap to unit-test by constructing a DoctorCtx fixture).
pub struct DoctorCtx {
    pub pkg_version: &'static str,         // env!("CARGO_PKG_VERSION") => "26.6.17"
    pub genesis_tag: String,               // parsed from chain::GENESIS_SEED
    pub working_path: std::path::PathBuf,  // resolves --working-dir or chain::WORKING_PATH
    pub compiled_features: &'static [&'static str],
    pub env: std::collections::BTreeMap<String, String>, // AFFI_* snapshot
    pub apply_fixes: bool,                 // --fix
}
```

### 3.2 The registry — `linkme::distributed_slice`

`linkme` is already a dependency and currently unused in `src/`. A distributed slice lets each check register itself at link time, so adding a check is a one-file change with no central match arm to edit — the same plugin pattern CLAUDE.md describes for custom handlers.

```rust
//! src/doctor/registry.rs (NEW)

use linkme::distributed_slice;
use super::DoctorCheck;

/// Every check appends a &'static dyn DoctorCheck here at link time.
#[distributed_slice]
pub static DOCTOR_CHECKS: [&'static (dyn DoctorCheck)] = [..];

/// Stable, deterministic order (doctrine: same inputs → same output). We sort by
/// a fixed severity-class then name rather than rely on link order.
pub fn ordered_checks() -> Vec<&'static dyn DoctorCheck> {
    let mut v: Vec<_> = DOCTOR_CHECKS.iter().copied().collect();
    v.sort_by_key(|c| c.name());
    v
}
```

A concrete check registers itself inline (example: stale working file — the ghost-events foot-gun):

```rust
//! src/doctor/checks/working_dir.rs (NEW)

use crate::doctor::{DoctorCheck, DoctorCtx, Finding, Fixed};
use crate::doctor::registry::DOCTOR_CHECKS;
use linkme::distributed_slice;

pub struct WorkingDirCheck;

#[distributed_slice(DOCTOR_CHECKS)]
static REGISTER: &dyn DoctorCheck = &WorkingDirCheck;

const STALE_DAYS: u64 = 7;

impl DoctorCheck for WorkingDirCheck {
    fn name(&self) -> &'static str { "working-dir" }

    fn run(&self, ctx: &DoctorCtx) -> Finding {
        let p = &ctx.working_path; // chain::WORKING_PATH = ".affi/working.json"
        let meta = match std::fs::metadata(p) {
            Ok(m) => m,
            // Absence is fine: `affi emit` creates it. Not a failure.
            Err(_) => return Finding::ok("working-dir", format!("{} absent (clean slate)", p.display())),
        };
        let age_days = meta.modified().ok()
            .and_then(|t| t.elapsed().ok())
            .map(|d| d.as_secs() / 86_400)
            .unwrap_or(0);
        let staged = crate::chain::load_working().map(|e| e.len()).unwrap_or(0);
        if age_days >= STALE_DAYS && staged > 0 {
            return Finding::fail(
                "working-dir",
                format!("{} is {age_days}d old with {staged} staged events", p.display()),
                "run `affi doctor --fix` to archive it, or `affi receipt assemble` to finalize",
            ).fixable();
        }
        Finding::ok("working-dir", format!("{staged} staged event(s), fresh"))
    }

    fn fix(&self, ctx: &DoctorCtx) -> anyhow::Result<Fixed> {
        // SAFE fix = never destructive. We *archive* (rename), never delete.
        let p = &ctx.working_path;
        let stamp = format!("{}.archived", p.display());
        std::fs::rename(p, &stamp)?;
        Ok(Fixed::Applied(format!("archived stale working file to {stamp}")))
    }
}
```

### 3.3 The handler — `crate::handlers::doctor`

Lives next to the existing handlers (`src/handlers.rs`), reusing `print_json_or` and the `adapt`/`Result` adaptation already in that file.

```rust
//! addition to src/handlers.rs

use crate::doctor::{self, registry, Status, Fixed};

/// `affi env doctor` — diagnose environment & installation health.
pub fn doctor(
    fix: Option<bool>,
    json: Option<bool>,
    check: Option<String>,        // run only this one check
    working_dir: Option<String>,
) -> Result<()> {
    let ctx = doctor::DoctorCtx::gather(working_dir, fix.unwrap_or(false));

    let mut checks = registry::ordered_checks();
    if let Some(only) = &check {
        checks.retain(|c| c.name() == only);
        if checks.is_empty() {
            return Err(clap_noun_verb::error::NounVerbError::execution_error(
                format!("unknown check '{only}'; see `affi doctor --help`"),
            ));
        }
    }

    let mut findings = Vec::with_capacity(checks.len());
    for c in &checks {
        let mut f = c.run(&ctx);
        if ctx.apply_fixes && f.auto_fixable && f.status != Status::Ok {
            match c.fix(&ctx) {
                Ok(Fixed::Applied(msg)) => { f.finding = format!("FIXED: {msg}"); f.status = Status::Ok; }
                Ok(Fixed::Skipped(msg)) => f.finding = format!("{} (skipped: {msg})", f.finding),
                Err(e)                  => f.finding = format!("{} (fix failed: {e})", f.finding),
            }
        }
        findings.push(f);
    }

    let fails = findings.iter().filter(|f| f.status == Status::Fail).count();

    if json.unwrap_or(false) {
        let body = serde_json::json!({
            "affi_version": ctx.pkg_version,
            "summary": summarize(&findings),   // {ok, warn, fail}
            "findings": findings,
        });
        println!("{}", adapt(serde_json::to_string_pretty(&body).map_err(anyhow::Error::from))?);
    } else {
        render_human(&ctx, &findings);         // the boxed table from §1 / §4
    }

    // EXIT-CODE DOCTRINE: warnings never fail the process; only `fail` does, and
    // only without --fix. This keeps `affi doctor` safe in non-blocking CI gates.
    if fails > 0 && !ctx.apply_fixes {
        std::process::exit(2);   // mirrors verify's REJECT code (handlers.rs:562)
    }
    Ok(())
}
```

### 3.4 The verb wrapper — matching the macro pattern exactly

Filed under a new `env` noun (keeps it out of the `receipt`-scoped namespace, signalling "this is about your machine, not a receipt"). Identical shape to `diagnose.rs`/`inspect.rs`:

```rust
//! src/verbs/doctor.rs (NEW)
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Diagnose environment & installation health; suggest (and optionally apply) fixes
#[verb("doctor", "env")]
pub fn doctor(
    fix: Option<bool>,
    json: Option<bool>,
    check: Option<String>,
    working_dir: Option<String>,
) -> Result<()> {
    crate::handlers::doctor(fix, json, check, working_dir)
}
```

Wiring (for the implementer; **not done in this doc**): add `pub mod doctor;` to `src/verbs/mod.rs`, add `pub mod doctor;` + `pub mod checks;` under a new `src/doctor/`, and re-export from `src/lib.rs`. The `#[verb]` macro registers it with the noun-verb app automatically, exactly as every other verb is registered (`src/bin/affi.rs` just calls the library's pre-wired `run`).

### 3.5 The check catalog (the headline content)

| check | class | what it inspects | auto-fix |
|---|---|---|---|
| `binary` | install | `affi` resolvable on `PATH`; reported path vs. the running exe | no |
| `toolchain` | install | `rustc`/`cargo` present; version ≥ MSRV `1.78` (`Cargo.toml:5`) | no |
| `features` | install | features compiled in (`core`, `lsp`, …) vs. what verbs need; explains the `"… feature not enabled"` errors before you hit them | no |
| `genesis` | foot-gun | binary `GENESIS_SEED` tag (`v26.6.14`) vs. pkg version (`26.6.17`) — cross-box verify mismatch | no |
| `working-dir` | foot-gun | `.affi/working.json` presence, age, permissions, staged-event count | **yes** (archive) |
| `config` | config | `AFFI_*` env vars, config file discovery & precedence (env > file > default) | no |
| `format-version` | foot-gun | default profile resolves to `core/v1` (`FORMAT_VERSION`); flags unknown `--profile` | no |
| `completions` | DX | shell completion scripts installed for the active shell | **yes** (generate) |
| `man-pages` | DX | man pages present on `MANPATH` | **yes** (install) |
| `registry` | optional-dep | reachability of feature-gated endpoints (otel collector, lsp, webhook sink, wasm4pm registry) — **only for features actually compiled in** | no |

The `registry` check is where doctrine bites hardest: it reports *reachability* ("otel collector at `$OTEL_EXPORTER` not responding") and suggests a fix, but it **never** concludes your telemetry is "wrong" — unreachable ≠ misconfigured ≠ dishonest.

---

## 4. CLI UX

### 4.1 Healthy machine (human)

```text
$ affi doctor
affi doctor — environment & installation health   (affi 26.6.17)

  ok    binary           affi 26.6.17 on PATH (/usr/local/bin/affi)
  ok    toolchain        rustc 1.81.0 ≥ MSRV 1.78
  ok    features         built with [core, discovery, lsp, otel]
  ok    genesis          chain genesis tag matches pkg (26.6.x)
  ok    working-dir      3 staged event(s), fresh
  ok    config           no AFFI_* overrides; using defaults
  ok    format-version   default profile core/v1 recognized
  ok    completions      bash completions installed
  ok    registry         otel collector reachable (localhost:4317)

  9 ok · 0 warn · 0 fail        your environment is healthy 🎉
```

### 4.2 Broken machine (human) — the case that earns the 1000x

```text
$ affi doctor
affi doctor — environment & installation health   (affi 26.6.17)

  ok    binary           affi 26.6.17 on PATH (/usr/local/bin/affi)
  ok    toolchain        rustc 1.81.0 ≥ MSRV 1.78
  warn  features         built with [core]; `graph`,`model`,`conformance` need `discovery`
                         → cargo install affidavit --features discovery
  fail  working-dir      .affi/working.json is 9d old with 41 staged events
                         → affi doctor --fix   (archives it; or `affi receipt assemble` to finalize)
  warn  genesis          binary genesis tag v26.6.14 < pkg 26.6.17
                         → receipts assembled here may REJECT on 26.6.17 peers; rebuild from matching source
  fail  config           AFFI_PROFILE=core/v2 set, but only core/v1 is recognized
                         → unset AFFI_PROFILE  (or export AFFI_PROFILE=core/v1)
  warn  completions      no bash completions installed
                         → affi doctor --fix   (writes ~/.local/share/bash-completion/completions/affi)
  ok    registry         no network-backed features compiled; nothing to probe

  3 ok · 3 warn · 2 fail        run `affi doctor --fix` to apply 2 safe fixes
$ echo $?
2
```

Two failures, one is the *exact* reason a teammate's receipts won't verify (`genesis`), one explains a class of confusing `feature not enabled` errors (`features`) — all on one screen, all with a remediation line. Exit code `2` (matching `verify`'s REJECT in `handlers.rs:562`) lets CI gate on it.

### 4.3 `--fix` run

```text
$ affi doctor --fix
affi doctor — applying safe fixes…

  ✔ working-dir   FIXED: archived stale working file to .affi/working.json.archived
  ✔ completions   FIXED: wrote bash completions to ~/.local/share/bash-completion/completions/affi
  •  features     skipped: rebuild required (cargo install affidavit --features discovery)
  •  genesis      skipped: not auto-fixable (rebuild from source matching peer version)
  •  config       skipped: refusing to mutate your shell env; unset AFFI_PROFILE yourself

  2 fixed · 3 require manual action
```

`--fix` is conservative on purpose: it only ever performs additive/reversible filesystem actions (rename, write a completion script). It will **not** edit your shell config, install toolchains, or delete data — the doctrine "suggest, don't impose" extended to repair.

### 4.4 `--json` (machine / CI)

```text
$ affi doctor --json --check working-dir
```
```json
{
  "affi_version": "26.6.17",
  "summary": { "ok": 0, "warn": 0, "fail": 1 },
  "findings": [
    {
      "check": "working-dir",
      "status": "fail",
      "finding": ".affi/working.json is 9d old with 41 staged events",
      "remediation": "run `affi doctor --fix` to archive it, or `affi receipt assemble` to finalize",
      "auto_fixable": true
    }
  ]
}
```

The JSON envelope reuses the `Finding` derive(Serialize) directly, so it stays in lockstep with the human view — same discipline `verify`/`inspect` use via `print_json_or` (`handlers.rs:96`).

---

## 5. Integration — what it touches and how it composes

**Reuses (read-only) existing code:**

- `crate::chain` — `WORKING_PATH`, `GENESIS_SEED`, `FORMAT_VERSION`, `load_working()` for the `working-dir`, `genesis`, and `format-version` checks. No new chain logic.
- `crate::handlers` plumbing — `adapt(...)`, `io_err(...)`, and the `print_json_or` convention (`handlers.rs:59`–`108`) so error mapping and JSON output match every other verb.
- `env!("CARGO_PKG_VERSION")` → `26.6.17` and `cfg!(feature = "…")` to discover compiled features — the same gates handlers already branch on (`handlers.rs:828`, `:866`, `:977`).
- `linkme` (`Cargo.toml:42`) — first real use in `src/`, for the check registry.

**Composition with `diagnose` / `inspect` (clear division of labor — doctrine-aligned):**

- `affi doctor` answers **"is my machine/install OK?"** (environment).
- `affi receipt diagnose <receipt>` answers **"why did *this receipt* fail to verify?"** (content) — unchanged.
- `affi receipt inspect <receipt>` answers **"what's *in* this receipt?"** (structure) — unchanged.

`doctor` can *hand off* without overstepping: when a `diagnose`/`verify` run can't even start because of an environment problem (e.g. `lsp feature not enabled` at `handlers.rs:996`, or a missing `.affi/`), those handlers can append a single hint line — `note: run \`affi doctor\` to check your environment` — turning a dead-end error into a next step. `doctor` never imports the verifier or renders a `Verdict`; it stays strictly on the environment side of the line. The boundary is the whole point: **`doctor` certifies the environment and suggests; the verifier certifies receipts and decides ACCEPT/REJECT; neither decides honesty.**

**A small shared upgrade (optional):** the `features` check's mapping of *verb → required feature* is exactly the knowledge currently scattered across `#[cfg(...)]` arms. Centralizing it as a `const FEATURE_NEEDS: &[(&str, &str)]` in `src/doctor/` lets both `doctor` and the existing handlers point users at the same remediation string, eliminating the bare `"discovery feature not enabled"` dead-ends.

---

## 6. Effort & rollout

Sizing: **S** ≈ ½ day, **M** ≈ 1–2 days, **L** ≈ 3–5 days.

### P0 — the spine and the four checks that pay rent
| item | size | notes |
|---|---|---|
| `DoctorCheck` trait, `Finding`, `DoctorCtx`, `Status` | S | pure data + trait; unit-testable with a `DoctorCtx` fixture |
| `linkme` registry + `ordered_checks()` | S | first `distributed_slice` in `src/` |
| verb wrapper `#[verb("doctor","env")]` + handler + human/JSON renderers | M | mirrors `diagnose.rs`; reuses `print_json_or` |
| checks: `binary`, `toolchain`, `features`, `working-dir` | M | highest pain-to-effort ratio |
| `--json`, `--check <name>`, exit-code policy | S | envelope + filtering |

### P1 — close the foot-guns and ship the repair
| item | size | notes |
|---|---|---|
| `--fix` engine (`fix()` dispatch, additive-only guarantee) | M | rename/write only; never destructive |
| checks: `genesis`, `format-version`, `config` (`AFFI_*` precedence) | M | `genesis` is the cross-box "won't verify" bug |
| checks: `completions`, `man-pages` (auto-fixable) | M | `--fix` writes the scripts |
| handoff hints from `verify`/`diagnose` → `affi doctor` | S | one line each; touches existing handlers |

### P2 — depth for power users & CI
| item | size | notes |
|---|---|---|
| `registry` reachability for compiled features (otel/lsp/webhook/wasm4pm) | L | network probes, timeouts, **only for enabled features** |
| `--fix --yes` / `--dry-run` and per-check `--fix <name>` | S | granular control |
| third-party checks via the same `distributed_slice` (plugin DX) | M | documents the extension path from CLAUDE.md |
| optional `colored`/`indicatif` polish behind the `ui` feature | S | graceful no-color fallback |

**Rollout:** P0 ships behind no new feature flag (depends only on `linkme`, already core). `registry` probes land under P2 gated to their owning features so a `--features core` build never makes a network call. Each check is independently testable, so the surface can grow check-by-check without destabilizing the command.

---

### Caveat on building/verifying this design
This repository's external dependencies (`clap-noun-verb`, `wasm4pm`, `lsp-max`, etc. at version `26.6`) resolve only against a private registry, so `cargo build`/`cargo test` will not run in a clean environment. The prototype above is written to match the codebase's real types and the `#[verb]` macro contract (verified against `src/verbs/diagnose.rs`, `src/handlers.rs`, `src/types.rs`, `src/chain.rs`, `Cargo.toml`) but has not been compiled here.
