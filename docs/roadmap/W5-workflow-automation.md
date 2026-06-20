# W5 — Workflow Automation & Config

**Workstream:** W5 of 10 · **Owner:** Workflow Automation & Config
**Status:** roadmap / design · **Horizon:** 2026 H2 → 2030
**Grounds on:** `docs/innovation/00-SYNTHESIS.md`, `docs/innovation/04-qol-workflow.md`

> **Doctrine guard (non-negotiable, applies to every line below).** *Certify, don't decide.*
> Automation in W5 only **drives** the canonical seams `crate::cli::{emit, assemble, verify}`
> → `crate::chain::ChainAssembler::finalize` → `crate::verifier::verify` →
> `crate::admission::admit`. It never constructs a `Receipt`, never mints `Admitted`, never
> edits a verdict. The verdict cache returns a stored verdict **only** on a content-address
> (BLAKE3 byte-identity) hit — it can return ACCEPT only for bytes the pipeline already
> certified ACCEPT; it can never fabricate one. `watch`/hooks/CI **surface** the verifier's
> own exit code (`0` ACCEPT / `2` REJECT) verbatim.

> **Build caveat.** The private-registry `26.6` deps (`clap-noun-verb`, `wasm4pm-compat`,
> `lsp-max`, …) do not resolve in a lone checkout, so nothing here is `cargo build`/`test`
> verified. All Rust/TOML/YAML below is **compilable-style** — correct against in-tree
> patterns and real symbols (cited `file:line`), pending signature finalization against the
> sibling crates.

---

## 1. Mission, scope & boundaries

### Mission
Drive the friction of producing one certified receipt from **~4 hand-typed commands +
grammar recall** to **one keystroke (save) and zero recall**, then scale that loop from a
single dev's working tree to an org-wide provenance service by 2030 — without ever letting
automation decide honesty.

### In scope (W5 owns)
1. **`affi init`** — scaffold `.affi/` (`profiles/`, `receipts/`, `cache/`), write a
   self-documenting `config.toml`, seed a starter profile, *offer* git hooks.
2. **Layered `affi config`** — precedence `flag > env > project > user > default`, with
   `affi config explain` showing each value's resolved `Source`.
3. **`affi watch`** — drive the **real** `FileWatcher` (`quality.rs:1147`) for the provenance
   loop (debounce → emit → assemble → verify), replacing the `monitor` stub that prints
   *"tokio-based watch loop not yet implemented"* (`handlers.rs:2632`).
4. **Supercharged git-hook suite** — `pre-commit` verify (block on REJECT), `commit-msg`
   receipt-address stamp, CI `doctor` gate; superset of the existing post-commit installer
   (`handlers.rs:2977`).
5. **Profiles / macros** — named, version-controlled emit sequences (`affi run <profile>`,
   `--staged` from the git index).
6. **`--dry-run` everywhere** — plan-only branch on every mutating verb and hook.
7. **Content-addressed verdict cache** — `address → verdict` under `.affi/cache/`,
   byte-identity hits only.
8. **Toward 2030:** a provenance **daemon/service** (long-lived watcher, socket/HTTP control
   plane), team profile registries, and org-wide automation.

### Boundaries (explicitly NOT W5)
| Concern | Owner | W5 relationship |
|---|---|---|
| Interactive REPL / TUI | **W6** | W5 exposes `watch`/`run` as non-interactive drivers; W6 may embed them |
| The doctor's checks themselves | **W2** | W5 **wires** `doctor` into hooks/CI; it does not author `DoctorCheck`s |
| Output/JSON & exit-code contract | **W1 / W3** | W5 **consumes** `Out`/error codes; never hand-formats JSON (avoids bug B2) |
| The 7-stage verifier internals | **W7** | W5 calls `verify`; never reaches inside the pipeline |
| Crypto/signing of receipts | **W8** | W5 caches verdicts by address; signing of cache entries is a W8 hand-off |
| Conformance/compliance policy | **W9 / W10** | W5 runs profiles; policy of *what must be emitted* is W10 |

---

## 2. Current state (cited) & the gap

| Capability | Today | Evidence | Gap |
|---|---|---|---|
| First-run scaffold | none | no `init` verb in `src/verbs/mod.rs`; users reverse-engineer `examples/golden_run.sh` (4 cmds, lines 35–48) | build `affi init` |
| Config / precedence | none | every invocation re-specifies flags; no `config.toml`, no resolver | build `src/config.rs` |
| Watch loop | **stub** | `handlers.rs:2632` prints *"tokio-based watch loop not yet implemented"*; `monitor` handler at `handlers.rs:2532`, verb `#[verb("monitor","receipt")]` at `src/verbs/monitor.rs:13` | drive the real watcher |
| Real file watcher | **exists, unused for provenance** | `crate::quality::file_watcher::FileWatcher` struct `quality.rs:1147`; `::new` `quality.rs:1168` (notify + `RecursiveMode::Recursive` + mpsc); `run_watch_loop` `quality.rs:1222` with `Instant` debounce; emits `Notification::{FileChanged, IntervalElapsed, Error}` (`quality.rs:1137`) — but only **measures quality**, never certifies | re-target to provenance |
| Git hook | **wrong hook** | `install_git_hook` installs a **post-commit** quality monitor (`handlers.rs:2977`), shells `affi receipt monitor` (`handlers.rs:3019/3054`), runs *after* the commit, never verifies a receipt | add pre-commit + commit-msg |
| Profiles/macros | none | emit sequences retyped; teams drift | `affi run <profile>` |
| `--dry-run` | none | mutating verbs always execute | plan-only branch |
| Verdict cache | none | identical bytes re-run full pipeline | content-addressed cache |

### Load-bearing seams W5 builds on (do not re-implement)
- `crate::cli::emit(event_type: &str, objects: &[String], payload: &str) -> Result<EmitOutput>` (`cli.rs:29`).
- `crate::cli::assemble(out: Option<&str>) -> Result<AssembleOutput>` (`cli.rs:78`) →
  `AssembleOutput { receipt_path, content_address, event_count }` (`types.rs:353`).
- `crate::cli::verify(receipt: &str) -> Result<(i32, Verdict)>` (`cli.rs:110`); runs the
  pipeline **and** `admission::admit` (`cli.rs:122-123`). `Verdict { accepted, profile,
  outcomes, reason }` (`types.rs:293`). Exit `0`/`2`.
- `chain::WORKING_PATH = ".affi/working.json"` (`chain.rs:25`).
- Handler helpers reused verbatim: `adapt` (`handlers.rs:59`), `io_err` (`handlers.rs:63`),
  `print_json_or` (`handlers.rs:96`); git helpers `determine_git_dir` (`handlers.rs:3117`),
  `generate_post_commit_hook` (`handlers.rs:3031`).
- Verb wrapper shape: `#[verb("assemble","receipt")]` → `crate::handlers::assemble(...)`
  (`src/verbs/assemble.rs:13`).
- `WorkspaceState` heuristics already model `has_working_receipt`, `src_modified`,
  `recent_receipts` (`src/1000x_cli_telepathy_qol.rs:43`) — W5 turns these **suggestions**
  into **actions**.

### Defect W5 retires
- **B5 (Synthesis Part A):** `monitor` is a stub though `FileWatcher` exists.
  `handlers.rs:2632` vs `quality.rs:1147`. W5's `affi watch` is the fix.
- **B4 prerequisite:** `GENESIS_SEED = b"affidavit-v26.6.14-genesis"` (`chain.rs:22`) vs
  package `26.6.17` (`Cargo.toml:3`). W5 does **not** own the fix (W1/W7), but the verdict
  cache MUST key on the *current* genesis so cache entries can never outlive a seed bump —
  see the cache key rule in §3 (2026 H2).

---

## 3. Phased plan (2026 H2 / 2027 / 2028 / 2029 / 2030)

New surfaces W5 adds (all additive — **no existing file modified**):
```
src/verbs/{init,watch,run,config_cmd}.rs   # thin #[verb] wrappers
src/handlers.rs additions: init, watch, run, config_explain, install_git_hook_v2
src/config.rs        (new) — layered Config + resolver + provenance Source trail
src/automation.rs    (new) — provenance watch loop, debounce, profile runner, verdict cache
```
New dep: `toml = "0.8"` behind a `config` feature folded into `core` (so `init`/`config`
work out of the box). `watch` reuses the existing `file-watch = ["notify","shell","tokio"]`
feature (`Cargo.toml:135`).

---

### 2026 H2 — Foundations: make the common path effortless (anchors Synthesis P1)

**Objective.** Ship `affi init`, the layered config resolver with `config explain`, the
profile/macro runner, and `--dry-run` on the mutating verbs. These are exactly Synthesis
**P1** rows ("`affi init` + layered `affi config` … `config explain`"). Watch and hooks land
in 2027 (Synthesis P2), but their config *shape* is written now so the file teaches itself.

**Deliverables.**
- `src/config.rs`: `Source { Flag, Env, ProjectConfig, UserConfig, Default }`,
  `Resolved<T> { value, source }`, `FileConfig` (all-optional fields), `Config::resolve`.
- `src/handlers.rs::init` + `src/verbs/init.rs` (`#[verb("init","receipt")]`).
- `affi config explain` (table of value/source/overridable-by).
- `affi run <profile>` + `STARTER_PROFILE` macro format; `src/automation.rs` skeleton.
- `--dry-run` plan branch on `emit`, `assemble`, `run`.

```rust
// src/config.rs (new) — layered resolver; each field remembers its Source.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Source { Flag, Env, ProjectConfig, UserConfig, Default }

#[derive(Debug, Clone, Serialize)]
pub struct Resolved<T> { pub value: T, pub source: Source }

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FileConfig {            // every field optional → partial files merge cleanly
    pub format: Option<String>,        // "text" | "json"
    pub profile: Option<String>,       // verifier profile, e.g. "core/v1"
    pub working_dir: Option<String>,   // parent of chain::WORKING_PATH (chain.rs:25)
    pub receipts_dir: Option<String>,  // default `assemble` output dir
    pub strict: Option<bool>,
    pub watch: Option<WatchConfig>,    // shape written now; consumed in 2027
    pub cache: Option<CacheConfig>,
}

#[derive(Debug, Default)]
pub struct FlagOverrides {             // parsed CLI overrides; None = unset
    pub format: Option<String>, pub profile: Option<String>,
    pub working_dir: Option<String>, pub strict: Option<bool>,
}

impl FileConfig {
    /// flag > env (AFFI_*) > project (./.affi) > user (~/.config/affi) > default.
    pub fn resolve(flags: &FlagOverrides) -> anyhow::Result<ResolvedConfig> {
        let user    = load_toml(&user_config_path());
        let project = load_toml(&find_project_config()?);
        fn pick(flag: Option<String>, env: &str,
                project: Option<String>, user: Option<String>, default: &str) -> Resolved<String> {
            if let Some(v) = flag { return Resolved { value: v, source: Source::Flag }; }
            if let Ok(v) = std::env::var(env) { return Resolved { value: v, source: Source::Env }; }
            if let Some(v) = project { return Resolved { value: v, source: Source::ProjectConfig }; }
            if let Some(v) = user    { return Resolved { value: v, source: Source::UserConfig }; }
            Resolved { value: default.into(), source: Source::Default }
        }
        Ok(ResolvedConfig {
            format:  pick(flags.format.clone(),  "AFFI_FORMAT",
                          project.as_ref().and_then(|p| p.format.clone()),
                          user.as_ref().and_then(|u| u.format.clone()),  "text"),
            profile: pick(flags.profile.clone(), "AFFI_PROFILE",
                          project.as_ref().and_then(|p| p.profile.clone()),
                          user.as_ref().and_then(|u| u.profile.clone()), "core/v1"),
            // working_dir / receipts_dir / strict resolved identically …
        })
    }
}

pub struct ResolvedConfig { pub format: Resolved<String>, pub profile: Resolved<String> /* … */ }

fn user_config_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"))
        .join("affi").join("config.toml")
}
fn find_project_config() -> anyhow::Result<PathBuf> {     // walk up to repo root
    let mut dir = std::env::current_dir()?;
    loop {
        let c = dir.join(".affi").join("config.toml");
        if c.exists() { return Ok(c); }
        if !dir.pop() { return Ok(PathBuf::from(".affi/config.toml")); }
    }
}
fn load_toml(path: &Path) -> Option<FileConfig> {
    toml::from_str(&std::fs::read_to_string(path).ok()?).ok()
}
```

```rust
// src/verbs/init.rs (new) — same shape as src/verbs/assemble.rs:13
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Scaffold .affi/, write a documented config, seed a starter profile, offer git hooks.
#[verb("init", "receipt")]
pub fn init(force: Option<bool>, with_hooks: Option<bool>, format: Option<String>) -> Result<()> {
    crate::handlers::init(force, with_hooks, format)   // delegates to handler (handlers.rs)
}
```

```rust
// src/handlers.rs (new fn) — init scaffolder. Idempotent: never clobbers without --force.
pub fn init(force: Option<bool>, with_hooks: Option<bool>, format: Option<String>) -> Result<()> {
    use std::path::Path;
    let force = force.unwrap_or(false);
    let affi = Path::new(".affi");
    for sub in ["profiles", "receipts", "cache"] {
        std::fs::create_dir_all(affi.join(sub)).map_err(io_err)?;   // io_err: handlers.rs:63
    }
    let cfg = affi.join("config.toml");
    let wrote = if cfg.exists() && !force { false } else {
        std::fs::write(&cfg, crate::config::DEFAULT_DOCUMENTED_TOML).map_err(io_err)?; true
    };
    let starter = affi.join("profiles/build-test.toml");
    if !starter.exists() || force {
        std::fs::write(&starter, crate::automation::STARTER_PROFILE).map_err(io_err)?;
    }
    // Ignore the volatile working set + cache; KEEP sealed receipts under version control.
    let _ = std::fs::write(affi.join(".gitignore"), "working.json\ncache/\n");
    if with_hooks.unwrap_or(false) { install_git_hook_v2(GitHookSpec::recommended())?; } // 2027
    print_json_or(&format, &serde_json::json!({          // print_json_or: handlers.rs:96
        "initialized": true, "config_written": wrote, "hooks": with_hooks.unwrap_or(false)
    }), || {
        eprintln!("✓ .affi/ scaffolded (profiles/ receipts/ cache/)");
        eprintln!("{} .affi/config.toml", if wrote {"✓ wrote"} else {"· kept"});
        eprintln!("next: `affi run build-test` once, or `affi watch` to auto-certify");
    })
}
```

```toml
# .affi/config.toml — written by `affi init`. Self-documenting so the file teaches itself.
# Precedence (highest first): CLI flag > env AFFI_* > this file > ~/.config/affi/config.toml > defaults.
# See where each value resolved:  affi config explain
format       = "text"            # "text" | "json"        (env: AFFI_FORMAT)
profile      = "core/v1"         # verifier profile        (env: AFFI_PROFILE)
working_dir  = ".affi"           # holds working.json      (env: AFFI_WORKING_DIR)
receipts_dir = ".affi/receipts"  # default assemble output

[watch]                          # shape written now; consumed by `affi watch` in 2027
paths        = ["src", "tests"]
debounce_ms  = 400
on_change    = ["build-test"]
auto_assemble = true
auto_verify   = true

[cache]
enabled = true
dir     = ".affi/cache"          # content-addressed; verdict reused only on byte-identity hit
```

```toml
# .affi/profiles/build-test.toml — STARTER_PROFILE (src/automation.rs const).
# A macro: a named, ordered list of emits. `affi run build-test` replays them.
name = "build-test"
[[step]]
type    = "build"
object  = "repo:main:git"
payload = "Cargo.toml"           # commitment = blake3(file bytes); raw bytes never stored
[[step]]
type    = "test"
object  = "suite:unit:test-suite"
payload = "-"                    # read payload from stdin (e.g. piped test output)
```

**`config explain` UX (the discoverability feature):**
```text
$ AFFI_FORMAT=json affi config explain
field         value           source           overridable by
------------  --------------  ---------------  ----------------------------
format        json            env              flag --format
profile       core/v1         project-config   flag --profile / AFFI_PROFILE
working_dir   .affi           default          AFFI_WORKING_DIR / config
receipts_dir  .affi/receipts  project-config   AFFI_RECEIPTS_DIR / config
resolution order: flag > env (AFFI_*) > .affi/config.toml > ~/.config/affi/config.toml > defaults
```

**Acceptance criteria (2026 H2).**
- `affi init` on an empty dir creates `.affi/{profiles,receipts,cache}/`, `config.toml`,
  `profiles/build-test.toml`, `.gitignore`; re-running without `--force` reports `· kept`
  and changes nothing (idempotent).
- `config explain` prints the correct `Source` for a value set via flag, via `AFFI_*`, via
  project file, and via default — proven by a 4-case table test.
- `affi run build-test` replays the profile through `crate::cli::emit` (the **only** event
  path); `affi run build-test --dry-run` prints the plan and writes nothing (working set
  byte-unchanged).
- Doctrine: grep proves `init`/`config`/`run` never call `ChainAssembler::finalize` or
  construct `Receipt`/`Admitted` directly — only via `crate::cli::*`.

**Cross-workstream deps (2026 H2).** Consume **W1/W3** output contract for `--format`/JSON
(no hand-built JSON, avoids bug B2). Coordinate the genesis fix (**B4**, W1/W7) before the
cache lands in 2027.

---

### 2027 — Close the loop: watch daemon + git-hook suite + verdict cache (Synthesis P2)

**Objective.** Replace the stub with a real provenance watcher driving `FileWatcher`; install
the pre-commit-verify / commit-msg-stamp hook suite; ship the content-addressed verdict cache.
These are Synthesis **P2** rows ("`affi watch` daemon … retire `monitor` stub", "supercharged
git-hook suite", "content-addressed verdict cache").

**Deliverables.**
- `src/automation.rs`: trailing-edge debounce loop, `run_cycle`, verdict cache.
- `src/handlers.rs::watch` + `src/verbs/watch.rs` (`#[verb("watch","receipt")]`), behind
  `feature = "file-watch"`. `monitor` becomes an alias/deprecation shim (B5 retired).
- `install_git_hook_v2` + `GitHookSpec` (pre-commit, commit-msg, optional post-commit).
- `--dry-run` extended to `watch` and the hooks; `--staged` object derivation.

```rust
// src/automation.rs (new; behind feature = "file-watch")
use anyhow::Result;
use std::time::{Duration, Instant};

pub struct WatchOptions {
    pub paths: Vec<String>, pub debounce: Duration,
    pub profiles: Vec<String>, pub auto_assemble: bool, pub auto_verify: bool, pub dry_run: bool,
}

/// Blocking auto-certify loop. Reuses the REAL watcher (quality.rs:1147); returns on Ctrl-C/close.
pub fn watch(opts: WatchOptions) -> Result<()> {
    let mut watcher = crate::quality::file_watcher::FileWatcher::new(  // quality.rs:1168
        opts.paths.first().map(String::as_str).unwrap_or("src"),
        0,  // we own debouncing here; disable the watcher's internal gate
    )?;
    eprintln!("affi watch — paths={:?} debounce={:?}{}",
        opts.paths, opts.debounce, if opts.dry_run { " (dry-run)" } else { "" });
    let mut pending: Option<Instant> = None;
    loop {
        // Trailing-edge debounce: a FileChanged resets the quiet window; we fire once it elapses
        // with no further events — editor save-storms (write+rename+chmod) collapse to one certify.
        match next_change(&mut watcher, opts.debounce) {
            Change::File(p) => { eprintln!("  ∆ {}", p.display()); pending = Some(Instant::now()); }
            Change::Quiet   => { if pending.take().is_some() { run_cycle(&opts)?; } }
            Change::Closed  => break,
        }
    }
    Ok(())
}

fn run_cycle(opts: &WatchOptions) -> Result<()> {
    let started = Instant::now();
    if opts.dry_run {
        eprintln!("  [dry-run] would emit {:?} → assemble → verify", opts.profiles);
        return Ok(());
    }
    for prof in &opts.profiles {                                  // 1. EMIT via sealed seam
        for step in load_profile(prof)?.steps {
            crate::cli::emit(&step.r#type, &[step.object.clone()], &step.payload)?;  // cli.rs:29
        }
    }
    let asm = crate::cli::assemble(None)?;                        // 2. ASSEMBLE  (cli.rs:78)
    eprintln!("  ⛓ sealed {} ({} events)",
        &asm.content_address[..12.min(asm.content_address.len())], asm.event_count);
    if opts.auto_verify {                                         // 3. VERIFY (or cache hit)
        if let Some(v) = cache_get(&asm.content_address) {
            eprintln!("  ✓ {} (cached {})", if v {"ACCEPT"} else {"REJECT"}, &asm.content_address[..12]);
        } else {
            let (code, verdict) = crate::cli::verify(&asm.receipt_path)?;  // cli.rs:110
            cache_put(&asm.content_address, verdict.accepted);   // store the verifier's own verdict
            eprintln!("  {} {} — {}",
                if code == 0 {"✓ ACCEPT"} else {"✗ REJECT"}, &asm.content_address[..12], verdict.reason);
            // Doctrine: report the verdict; never suppress or override it.
        }
    }
    eprintln!("  ↳ cycle {:?}", started.elapsed());
    Ok(())
}

// Content-addressed verdict cache. Key MUST bind to the current genesis seed (chain.rs:22)
// so a seed bump (B4) silently invalidates stale entries — a hit then means *byte-identical
// canonical bytes under the same chain rule* already certified. It can never fabricate ACCEPT.
fn cache_path(addr: &str) -> std::path::PathBuf {
    let key = blake3::hash(format!("{}|{}", crate::chain::GENESIS_SEED_HEX, addr).as_bytes());
    std::path::Path::new(".affi/cache").join(format!("{}.verdict", key.to_hex()))
}
pub fn cache_get(addr: &str) -> Option<bool> {
    std::fs::read_to_string(cache_path(addr)).ok().map(|s| s.trim() == "ACCEPT")
}
pub fn cache_put(addr: &str, accepted: bool) {
    let _ = std::fs::create_dir_all(".affi/cache");
    let _ = std::fs::write(cache_path(addr), if accepted {"ACCEPT"} else {"REJECT"});
}
```

```rust
// src/handlers.rs — superset installer; keeps the old install_git_hook (handlers.rs:2977) working.
pub struct GitHookSpec { pub pre_commit: bool, pub commit_msg: bool, pub post_commit: bool }
impl GitHookSpec {
    pub fn recommended() -> Self { Self { pre_commit: true, commit_msg: true, post_commit: false } }
}
pub fn install_git_hook_v2(spec: GitHookSpec) -> Result<()> {
    let git_dir = determine_git_dir()?;                          // reused: handlers.rs:3117
    let hooks = std::path::Path::new(&git_dir).join("hooks");
    std::fs::create_dir_all(&hooks).map_err(io_err)?;
    if spec.pre_commit { write_hook(&hooks, "pre-commit", PRE_COMMIT_HOOK)?; }
    if spec.commit_msg { write_hook(&hooks, "commit-msg", COMMIT_MSG_HOOK)?; }
    eprintln!("✓ git hooks installed in {}", hooks.display());
    Ok(())
}
#[cfg(unix)]
fn write_hook(dir: &std::path::Path, name: &str, body: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let p = dir.join(name);
    std::fs::write(&p, body).map_err(io_err)?;
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).map_err(io_err)?;
    Ok(())
}

const PRE_COMMIT_HOOK: &str = r#"#!/usr/bin/env bash
# affi pre-commit — certify staged work before it is recorded. Bypass: git commit --no-verify
set -euo pipefail
affi receipt run build-test --staged --quiet || { echo "affi: emit failed" >&2; exit 1; }
addr=$(affi receipt assemble --format json | jq -r .content_address)
if ! affi receipt verify ".affi/receipts/${addr}.json" >/dev/null 2>&1; then
  echo "✗ affi: receipt ${addr:0:12} did NOT certify (REJECT). Commit blocked." >&2
  exit 2   # verify's own REJECT code → git aborts the commit; affi decides nothing
fi
echo "✓ affi: ${addr:0:12} ACCEPT" >&2
echo "$addr" > .affi/.last-commit-addr   # handoff to commit-msg
"#;

const COMMIT_MSG_HOOK: &str = r#"#!/usr/bin/env bash
# affi commit-msg — stamp the certified receipt address into the commit message.
set -euo pipefail
f=".affi/.last-commit-addr"; [ -f "$f" ] || exit 0
addr=$(cat "$f")
grep -q "^Affidavit:" "$1" || printf '\nAffidavit: %s\n' "$addr" >> "$1"
rm -f "$f"
"#;
```

**`affi watch` UX (cache hit on re-trigger):**
```text
$ affi watch
affi watch — paths=["src", "tests"] debounce=400ms
  ∆ src/lib.rs            # saved 3× in 2s → ONE cycle
  ∆ src/lib.rs
  ⛓ sealed 203d3bbf91c4 (2 events)
  ✓ ACCEPT 203d3bbf91c4 — all stages passed
  ↳ cycle 78ms
  ∆ src/lib.rs            # no source change → byte-identical → cache hit, no recompute
  ⛓ sealed 203d3bbf91c4 (2 events)
  ✓ ACCEPT (cached 203d3bbf91c4)
  ↳ cycle 2ms
```

**Pre-commit blocking on REJECT:**
```text
$ git commit -m "wip"
✗ affi: receipt a2d95f1130ab did NOT certify (REJECT). Commit blocked.
# exit 2 — git aborts; deliberate bypass: git commit --no-verify
```

**CI Action (paste-ready):**
```yaml
# .github/workflows/provenance.yml
name: provenance
on: [push, pull_request]
jobs:
  certify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --features file-watch --bin affi
      - run: echo "${{ github.workspace }}/target/release" >> "$GITHUB_PATH"
      - name: Scaffold + toolchain health (W2 doctor, wired by W5)
        run: |
          affi receipt init --format json
          affi receipt doctor          # non-zero on any problem → fails the job
      - name: Record this build as a certified receipt
        run: |
          affi receipt run build-test --assemble --format json | tee assemble.json
          ADDR=$(jq -r .content_address assemble.json)
          affi receipt verify ".affi/receipts/${ADDR}.json"   # exit 0 ACCEPT / 2 REJECT
      - uses: actions/upload-artifact@v4
        if: always()
        with: { name: provenance-receipts, path: .affi/receipts/*.json }
```

**Acceptance criteria (2027).**
- A burst of N filesystem events inside the debounce window produces exactly **one**
  `run_cycle` (trailing-edge debounce); verified by an injected-event test.
- `affi watch` certifies via `crate::cli::{emit,assemble,verify}` only; the `monitor` stub
  string at `handlers.rs:2632` no longer reachable from `watch` (B5 retired).
- Verdict cache: a second cycle over byte-identical bytes returns the stored verdict without
  invoking `verifier::verify`; a one-byte change misses and re-runs the full pipeline.
- Cache key invalidates on a `GENESIS_SEED` change (stale entries unreadable) — a
  hand-crafted ACCEPT file for unseen bytes is never honored (doctrine test).
- `pre-commit` exits `2` (aborting the commit) when `verify` REJECTs; `commit-msg` appends
  exactly one `Affidavit:` trailer.

**Cross-workstream deps (2027).** **W2** authors `doctor`; W5 wires it into the hook suite +
CI. **W6** may embed `watch`/`run` in the REPL. **W8** reviews the cache-entry trust model.

---

### 2028 — Persistent provenance daemon (long-lived service)

**Objective.** Promote the foreground `watch` into a managed, long-lived **daemon**: one
process watching many roots, with a control socket, structured event log, and crash recovery.
Still a pure driver of the sealed seams.

**Deliverables.**
- `affi daemon {start,stop,status,reload}` — manages a background watcher; PID + Unix socket
  under `.affi/daemon/`. `reload` re-reads `config.toml` without dropping watches.
- `[daemon]` config block: `roots`, `socket`, `max_inflight`, `notify_on` (`reject|always`).
- Structured per-cycle JSONL log at `.affi/daemon/cycles.jsonl` (each line: address, verdict,
  duration, cache-hit) — consumed by W6 surfaces and W10 audit.
- Desktop/webhook notification on REJECT (reuse `webhook` feature, `Cargo.toml:173`).

```rust
// src/automation.rs — daemon control plane (behind feature = "file-watch", socket on unix).
pub struct DaemonSpec { pub roots: Vec<String>, pub socket: std::path::PathBuf, pub max_inflight: usize }

pub fn daemon_start(spec: DaemonSpec) -> anyhow::Result<()> {
    write_pidfile(&spec)?;                     // refuse a second start if a live PID holds the socket
    let mut log = CycleLog::open(".affi/daemon/cycles.jsonl")?;   // append-only JSONL, fsync per line
    for root in &spec.roots { spawn_watcher(root, /* reuse run_cycle */)?; }   // FileWatcher per root
    serve_control(&spec.socket, |cmd| match cmd {                // status/stop/reload over the socket
        Ctl::Status => Reply::Status(snapshot()),
        Ctl::Reload => { reload_config(); Reply::Ok }            // re-read config.toml; keep watches
        Ctl::Stop   => Reply::Stopping,
    })?;
    let _ = log;                               // each run_cycle appends one JSONL record
    Ok(())
}
```

```toml
[daemon]
roots        = ["."]                  # repos / worktrees to watch
socket       = ".affi/daemon/affi.sock"
max_inflight = 4                      # cap concurrent certify cycles
notify_on    = "reject"              # desktop/webhook ping only on REJECT (reuse webhook feature)
```

**Acceptance criteria (2028).** `daemon start` is idempotent (second start refuses, points at
the live socket); `status` reports per-root last verdict from `cycles.jsonl`; `reload` applies
a config change with zero missed events; killing the process leaves a parseable JSONL log and
no half-written receipt. Every cycle still routes through `crate::cli::{emit,assemble,verify}`.

**Cross-workstream deps (2028).** **W1** owns the JSONL record schema (output contract);
**W10** consumes `cycles.jsonl` as an audit feed; **W6** renders daemon status.

---

### 2029 — Team & multi-repo orchestration

**Objective.** Lift automation from one tree to a **team**: shared profile registries,
multi-repo fan-out, and a remote verdict cache so a receipt certified once is trusted
fleet-wide (byte-identity only).

**Deliverables.**
- **Profile registry resolution:** `[profiles] sources = [...]` pulls versioned, signed
  profile bundles from a team registry (hand-off to **W4** registry / **W9** distribution),
  pinned by digest — eliminates cross-dev macro drift at org scale.
- **Multi-repo `affi daemon`:** one daemon, many worktrees; per-root config overlays.
- **Shared/remote verdict cache:** `[cache] remote = "..."`; a content-address present in the
  shared cache short-circuits local re-verify. Entries are **address-keyed and signed**
  (W8) — sharing a verdict for *unseen* bytes is impossible by construction.
- **Org policy hooks:** `affi run --require <policy>` blocks if a W10 policy demands emits the
  profile didn't produce (W5 enforces *presence*; W10 owns the *rule*).

```toml
[profiles]
sources = ["git+https://git.example.org/affi-profiles@v3"]   # pinned, signed bundle (W4/W9)
require = ["build", "test", "sbom"]                            # presence enforced; rule owned by W10

[cache]
remote = "https://cache.example.org/affi"   # address-keyed, signed entries (W8); byte-identity hits only
```

**Acceptance criteria (2029).** A profile bundle resolves by pinned digest and is byte-stable
across two machines; a remote-cache hit on machine B (for bytes machine A certified) skips
local `verify` yet records *which* signed entry was trusted; a forged remote entry whose
signature/address fails is ignored and the local pipeline runs. Doctrine: the remote cache can
only echo a prior verifier verdict for identical bytes.

**Cross-workstream deps (2029).** **W4** (registry transport), **W8** (cache-entry signing &
verification), **W9** (bundle distribution/standards), **W10** (policy rules `require` enforces).

---

### 2030 — Org-wide autonomous provenance fabric

**Objective.** Provenance that *happens to the org*: every build/test/release across every
repo lays down a certified receipt automatically, surfaced centrally, with W5 as the
zero-friction automation fabric — and still **certify, not decide**.

**Deliverables.**
- **`affi service`** — a hosted, multi-tenant control plane fronting many daemons (the 2028
  daemon, clustered); HTTP/gRPC API to enqueue `run`/`watch`/`verify` jobs and stream verdicts.
- **Hooks-as-policy at the forge:** generated server-side pre-receive / merge-queue gates
  (GitHub/GitLab) that run the same `verify` the local pre-commit runs — one certify
  definition, local and remote.
- **Fleet verdict cache** with org-wide byte-identity dedup + signed provenance of every cache
  fill (W8); **SLA/throughput dashboards** fed by `cycles.jsonl` aggregates (surfaced by W6).
- **Reproducibility gate:** `affi service` re-runs a sample of cached ACCEPTs from scratch and
  alarms on any address it cannot reproduce — guarding the cache against drift.

```yaml
# .github/workflows/provenance-merge-queue.yml — forge-side gate, same certify as pre-commit.
name: provenance-merge-queue
on:
  merge_group:
jobs:
  certify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          affi service submit --run build-test --assemble --wait --format json | tee r.json
          affi service verify "$(jq -r .content_address r.json)"   # exit 0 ACCEPT / 2 REJECT
```

**Acceptance criteria (2030).** The forge merge-queue gate and the local pre-commit produce
identical verdicts for identical staged bytes; the fleet cache dedups across repos by address
with a signed fill record per entry; the reproducibility sampler rebuilds a random ACCEPT and
matches its address; no path in the fabric can emit ACCEPT for bytes the verifier never saw.

**Cross-workstream deps (2030).** **W8** (fleet-cache signing, reproducibility attestation),
**W9** (cross-org receipt interchange), **W10** (governance reporting over the fabric),
**W6** (dashboards), **W2** (`service doctor` fleet health).

---

## 4. Definition of done @ 2030

W5 is "done" when **all** hold:

1. **Zero-recall provenance.** A developer who runs `affi init` once never again hand-types an
   emit/assemble/verify; saving a file (`watch`/daemon) or committing (hooks) lays down a
   sealed, certified receipt automatically.
2. **The stub is gone.** `affi watch` drives the real `FileWatcher` (`quality.rs:1147`); the
   *"tokio-based watch loop not yet implemented"* string (`handlers.rs:2632`) is unreachable
   from any W5 path; `monitor` is an alias or removed (B5 closed).
3. **Config is layered & explainable.** `flag > env > project > user > default` holds for every
   setting, and `affi config explain` shows each value's `Source` — proven by tests at every
   layer.
4. **Hooks certify, don't decide.** pre-commit blocks on the verifier's own exit `2`;
   commit-msg stamps the content address; CI/forge gates run the identical `verify`.
5. **The cache is doctrinally sound.** Every cache hit is a BLAKE3 byte-identity match keyed to
   the live genesis; a fabricated verdict for unseen bytes is provably never honored; fleet
   fills are signed (W8).
6. **Scale.** A single daemon/service drives certify across many repos/worktrees with shared,
   pinned profiles and an org-wide verdict cache.
7. **Additive & sealed.** No existing source file's behavior was changed by W5; the seal stays
   sealed — W5 code constructs neither `Receipt` nor `Admitted`, only calls `crate::cli::*`.
8. **`--dry-run` everywhere.** Every mutating verb/hook has a plan-only mode that writes
   nothing.

---

## 5. Cross-workstream dependencies (summary)

| Phase | W5 needs FROM | W5 provides TO |
|---|---|---|
| 2026 H2 | **W1/W3** output & exit-code contract (no hand-built JSON, bug B2); **W1/W7** genesis fix (B4) before cache | `init`/`config`/`run` + the `.affi/` layout every other verb writes into |
| 2027 | **W2** `doctor` checks (wired into hooks/CI); **W7** stable `verify` `(i32, Verdict)` contract | `watch` daemon-precursor; git-hook suite; verdict cache; `--staged` |
| 2028 | **W1** JSONL cycle-record schema | `cycles.jsonl` audit feed (**W10**); daemon status surface (**W6**) |
| 2029 | **W4** profile-registry transport; **W8** cache-entry signing; **W9** bundle distribution | multi-repo daemon; remote cache; `--require` presence enforcement (rule = **W10**) |
| 2030 | **W8** fleet-cache attestation; **W2** fleet `doctor`; **W9** interchange | `affi service` fabric; forge merge-queue gates; SLA feed (**W6**/**W10**) |

**Standing boundary reminders.** REPL/TUI = **W6** (W5 only exposes non-interactive drivers).
The doctor's *checks* = **W2** (W5 wires `doctor` into automation). Output/JSON & error codes
= **W1/W3** (W5 consumes). Verifier internals = **W7** (W5 calls `verify`). Receipt/cache
signing = **W8**. Conformance/governance policy = **W9/W10** (W5 enforces presence, never
authors the rule). Across all of it: **certify, don't decide.**
