# 04 — Quality-of-Life & Workflow Automation

> **Doctrine guard.** Everything below *records* and *certifies*. Nothing here decides honesty.
> `affi watch`, the git hooks, and CI all funnel into the same sealed seams —
> `crate::cli::{emit, assemble, verify}` → `crate::chain::ChainAssembler::finalize` →
> `crate::verifier::verify` → `crate::admission::admit`. Automation can never mint an
> `Admitted` receipt by fiat; it can only drive the canonical path faster.

> **Build caveat.** Code in this doc is *compilable-style* and cites real symbols, but is
> not wired in here. The crate depends on unresolvable `26.6` registry deps
> (`clap-noun-verb`, `wasm4pm-compat`, …), so `cargo build`/`test` will not run in this
> environment. Treat the Rust blocks as patches-in-waiting against the real modules.

---

## 1. Vision — the 1000x leap

Today, provenance is a **manual ritual**. To certify what a build did, a human types
`affi receipt emit … && affi receipt emit … && affi receipt assemble && affi receipt verify`
— every time, by hand, remembering the object-spec grammar, the working-dir, the output
name. Provenance that costs effort gets skipped, and skipped provenance is no provenance.

The 1000x leap is **provenance that happens to you, not because of you**: a zero-friction
loop where saving a file, running a test, or making a commit *automatically* lays down a
sealed, verifiable receipt — and the only time a human sees the machinery is when a
verdict comes back `REJECT`.

| Dimension | Before (today) | After (this design) |
|---|---|---|
| First run | `mkdir`, guess paths, read `golden_run.sh` | `affi init` → scaffolded `.affi/`, config, hooks offered |
| Per-change cost | 4 hand-typed commands, exact grammar | 0 — `affi watch` debounces and emits/assembles/verifies |
| Repeated emit sequences | retyped each time, drift between devs | `affi run <profile>` — one named macro, version-controlled |
| Config | hardcoded flags, copy-paste across repos | layered precedence, `affi config explain` shows resolution |
| Git safety net | manual `verify` before push (forgotten) | pre-commit verify + commit-msg receipt stamp, automatic |
| CI | bespoke YAML per repo | `affi doctor` + paste-ready Action |
| Recompute | re-verify identical bytes every time | content-addressed cache — skip on hash hit |

The shift in *units*: the friction of producing one certified receipt drops from ~4
deliberate commands + recall to **one keystroke (save) and zero recall**. Across a
working day of dozens of edits, builds, and commits, that is the 1000x.

---

## 2. Current state — what the lifecycle requires today

The canonical lifecycle is `examples/golden_run.sh`. Stripped of its temp-dir harness,
the human-facing core is four commands (lines 35–48):

```bash
affi receipt emit --type seed     --object art1:artifact:input  --payload payload_a.txt
affi receipt emit --type validate --object art1:artifact:output --payload payload_b.txt
affi receipt assemble --out receipt.json
affi receipt verify receipt.json     # ACCEPT → exit 0 ; tampered → exit 2
```

What each step demands of the operator, and where it hurts:

- **`emit`** (`src/verbs/emit.rs` → `crate::handlers::emit` → `crate::cli::emit`, src/cli.rs:29).
  Operator must hand-author the `id:type[:qualifier]` object grammar and point `--payload`
  at a real file or `-`. The working set lives at `chain::WORKING_PATH = ".affi/working.json"`
  (src/chain.rs:25); `save_working` (src/chain.rs:161) auto-creates the dir, but nothing
  tells a first-timer it exists. **Friction:** grammar recall, repeated for every event.
- **`assemble`** (`crate::cli::assemble`, src/cli.rs:78) folds the working events through
  `chain::ChainAssembler::finalize` and content-addresses them. Forgetting `--out` yields a
  `<blake3>.json` filename you then have to discover. **Friction:** naming + "what's my hash?"
- **`verify`** (`crate::cli::verify`, src/cli.rs:110) runs the 7-stage pipeline *and*
  `crate::admission::admit`. Exit `0` ACCEPT / `2` REJECT — perfect for automation, but
  today a human runs it manually and often forgets until after a bad push. **Friction:** it's
  opt-in and out-of-band.

**Friction inventory (the real pain):**

1. **No first-run.** There is no `affi init`. No `init`/`doctor`/`config`/`watch` verb
   exists in `src/verbs/mod.rs` (59 verbs, none of these). New users reverse-engineer
   `golden_run.sh`.
2. **No config.** Every invocation re-specifies flags; `--working-dir`, default `--format`,
   default profile, output dir — none are persistable. No precedence model.
3. **The watch loop is a stub.** `crate::handlers::monitor` (src/handlers.rs:2532) literally
   prints *"tokio-based watch loop not yet implemented"* (src/handlers.rs:2632) and runs
   once. Yet a real `FileWatcher` already exists — `crate::quality::file_watcher::FileWatcher`
   (src/quality.rs:1129, feature `file-watch`) with notify + mpsc + `Instant` debounce
   (src/quality.rs:1222 `run_watch_loop`). **The engine is built; nothing drives provenance with it.**
4. **The git hook is the wrong hook.** `crate::handlers::install_git_hook` (src/handlers.rs:2977)
   installs a **post-commit** quality monitor that shells `affi receipt monitor` — it never
   verifies a receipt and runs *after* the commit is already made. No pre-commit gate, no
   commit-msg stamp.
5. **No macros / profiles.** The same emit sequences are retyped; teams drift.
6. **No `--dry-run`, no cache.** Identical inputs re-run the full pipeline every time.

The crate *already wants* this: `src/1000x_cli_telepathy_qol.rs` models a `WorkspaceState`
(src/1000x_cli_telepathy_qol.rs:43) with `has_working_receipt`, `src_modified`,
`recent_receipts` and suggests the next command. We turn those *suggestions* into *actions*.

---

## 3. Proposed design

Five additions, all thin verbs over new handlers, none touching the sealed seams:

```
src/verbs/{init,watch,doctor,run,config_cmd}.rs   # new #[verb] wrappers
src/handlers.rs                                    # new pub fn init/watch/doctor/run/config_*
src/config.rs            (new)  — layered Config + resolver + provenance trail
src/automation.rs        (new)  — watch loop, debounce, cache, profile runner
```

### 3.1 `affi init` — scaffold + first-run

Creates `.affi/` (already the home of `working.json`), writes a documented `affi.toml`,
seeds a `profiles/` dir with a starter macro, and *offers* (never forces) git-hook install.
Idempotent: re-running never clobbers an existing config without `--force`.

```rust
// src/handlers.rs — new handler (verb wrapper in src/verbs/init.rs delegates here)
pub fn init(force: Option<bool>, with_hooks: Option<bool>, format: Option<String>) -> Result<()> {
    use std::path::Path;
    let force = force.unwrap_or(false);
    let affi_dir = Path::new(".affi");
    std::fs::create_dir_all(affi_dir.join("profiles")).map_err(io_err)?;
    std::fs::create_dir_all(affi_dir.join("receipts")).map_err(io_err)?;
    std::fs::create_dir_all(affi_dir.join("cache")).map_err(io_err)?;

    let cfg_path = affi_dir.join("config.toml");
    let wrote_cfg = if cfg_path.exists() && !force {
        false
    } else {
        std::fs::write(&cfg_path, crate::config::Config::default_documented_toml()).map_err(io_err)?;
        true
    };

    // Starter profile: the golden_run sequence, as a reusable macro.
    let starter = affi_dir.join("profiles/build-test.toml");
    if !starter.exists() || force {
        std::fs::write(&starter, crate::automation::STARTER_PROFILE).map_err(io_err)?;
    }

    // .gitignore guidance: ignore the volatile working set + cache, KEEP sealed receipts.
    let _ = std::fs::write(affi_dir.join(".gitignore"), "working.json\ncache/\n");

    if with_hooks.unwrap_or(false) {
        install_git_hook_v2(GitHookSpec::recommended())?; // see §3.4
    }

    if format.as_deref() == Some("json") {
        let out = serde_json::json!({
            "initialized": true, "config_written": wrote_cfg,
            "dirs": [".affi/profiles", ".affi/receipts", ".affi/cache"],
            "hooks_installed": with_hooks.unwrap_or(false),
        });
        println!("{}", adapt(serde_json::to_string_pretty(&out).map_err(anyhow::Error::from))?);
        return Ok(());
    }
    println!("✓ .affi/ scaffolded (profiles/, receipts/, cache/)");
    println!("{} .affi/config.toml", if wrote_cfg { "✓ wrote" } else { "· kept existing" });
    println!("✓ starter profile: .affi/profiles/build-test.toml");
    println!("next: `affi watch` to auto-certify, or `affi run build-test` once");
    Ok(())
}
```

```rust
// src/verbs/init.rs — thin wrapper, same shape as src/verbs/assemble.rs
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Scaffold .affi/, write a documented config, and offer git hooks.
#[verb("init", "receipt")]
pub fn init(force: Option<bool>, with_hooks: Option<bool>, format: Option<String>) -> Result<()> {
    crate::handlers::init(force, with_hooks, format)
}
```

### 3.2 Config system — layered precedence

**Resolution order (highest wins):**

```
1. CLI flags          (e.g. --format json, --profile core/v1)   ← most specific
2. Environment        (AFFI_FORMAT, AFFI_PROFILE, AFFI_WORKING_DIR, …)
3. Project config     ./.affi/config.toml      (walk up to repo root)
4. User config        $XDG_CONFIG_HOME/affi/config.toml  (~/.config/affi/…)
5. Built-in defaults  Config::default()                              ← least specific
```

Every resolved field remembers **where it came from**, so `affi config explain` can show
the trail — discoverability is a first-class feature, not an afterthought.

```rust
// src/config.rs  (new module; re-export from src/lib.rs)
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Where a resolved value came from — surfaced by `affi config explain`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Source { Flag, Env, ProjectConfig, UserConfig, Default }

#[derive(Debug, Clone, Serialize)]
pub struct Resolved<T> { pub value: T, pub source: Source }

/// On-disk config shape. Every field optional so partial files merge cleanly.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FileConfig {
    pub format: Option<String>,        // "json" | "text"
    pub profile: Option<String>,       // verifier profile, e.g. "core/v1"
    pub working_dir: Option<String>,   // overrides chain::WORKING_PATH parent
    pub receipts_dir: Option<String>,  // where `assemble` writes by default
    pub strict: Option<bool>,
    pub watch: Option<WatchConfig>,
    pub cache: Option<CacheConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WatchConfig {
    pub paths: Vec<String>,            // dirs to watch (default ["src", "tests"])
    pub debounce_ms: u64,              // default 400
    pub on_change: Vec<String>,        // profile names to run, in order
    pub auto_assemble: bool,           // assemble after a quiet window
    pub auto_verify: bool,             // verify each assembled receipt
}
impl Default for WatchConfig {
    fn default() -> Self {
        Self { paths: vec!["src".into(), "tests".into()], debounce_ms: 400,
                on_change: vec!["build-test".into()], auto_assemble: true, auto_verify: true }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CacheConfig { pub enabled: bool, pub dir: Option<String> }

/// Fully-resolved config: each field carries its provenance Source.
#[derive(Debug, Clone)]
pub struct Config {
    pub format: Resolved<String>,
    pub profile: Resolved<String>,
    pub working_dir: Resolved<PathBuf>,
    pub receipts_dir: Resolved<PathBuf>,
    pub strict: Resolved<bool>,
    pub watch: WatchConfig,
    pub cache: CacheConfig,
}

impl Config {
    /// Resolve in precedence order. `flags` are the parsed CLI overrides (None = unset).
    pub fn resolve(flags: &FlagOverrides) -> anyhow::Result<Self> {
        let user    = load_toml(&user_config_path()).unwrap_or_default();
        let project = load_toml(&find_project_config()?).unwrap_or_default();

        // helper: flag > env > project > user > default, recording the winner
        fn pick(
            flag: Option<String>, env: &str,
            project: Option<String>, user: Option<String>, default: &str,
        ) -> Resolved<String> {
            if let Some(v) = flag { return Resolved { value: v, source: Source::Flag }; }
            if let Ok(v) = std::env::var(env) { return Resolved { value: v, source: Source::Env }; }
            if let Some(v) = project { return Resolved { value: v, source: Source::ProjectConfig }; }
            if let Some(v) = user    { return Resolved { value: v, source: Source::UserConfig }; }
            Resolved { value: default.to_string(), source: Source::Default }
        }

        let format  = pick(flags.format.clone(),  "AFFI_FORMAT",
                           project.format.clone(), user.format.clone(),  "text");
        let profile = pick(flags.profile.clone(), "AFFI_PROFILE",
                           project.profile.clone(), user.profile.clone(), "core/v1");
        let working = pick(flags.working_dir.clone(), "AFFI_WORKING_DIR",
                           project.working_dir.clone(), user.working_dir.clone(), ".affi");
        let receipts = pick(None, "AFFI_RECEIPTS_DIR",
                           project.receipts_dir.clone(), user.receipts_dir.clone(), ".affi/receipts");

        Ok(Config {
            format,
            profile,
            working_dir:  Resolved { value: PathBuf::from(working.value),  source: working.source },
            receipts_dir: Resolved { value: PathBuf::from(receipts.value), source: receipts.source },
            strict: Resolved {
                value: flags.strict.or(project.strict).or(user.strict).unwrap_or(false),
                source: if flags.strict.is_some() { Source::Flag } else { Source::Default },
            },
            watch: project.watch.or(user.watch).unwrap_or_default(),
            cache: project.cache.or(user.cache).unwrap_or_default(),
        })
    }

    /// The seed config `affi init` writes — documented inline so the file teaches itself.
    pub fn default_documented_toml() -> &'static str { DEFAULT_TOML }
}

#[derive(Debug, Default)]
pub struct FlagOverrides {
    pub format: Option<String>, pub profile: Option<String>,
    pub working_dir: Option<String>, pub strict: Option<bool>,
}

fn user_config_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|_| dirs_home().join(".config"))
        .join("affi").join("config.toml")
}
fn dirs_home() -> PathBuf { std::env::var("HOME").map(PathBuf::from).unwrap_or_default() }

/// Walk up from CWD looking for `.affi/config.toml` (repo-root discovery).
fn find_project_config() -> anyhow::Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let candidate = dir.join(".affi").join("config.toml");
        if candidate.exists() { return Ok(candidate); }
        if !dir.pop() { return Ok(PathBuf::from(".affi/config.toml")); } // absent → defaults
    }
}

fn load_toml(path: &Path) -> Option<FileConfig> {
    let text = std::fs::read_to_string(path).ok()?;
    toml::from_str(&text).ok()   // add `toml = "0.8"` under a new `config` feature
}
```

**The self-documenting `.affi/config.toml` that `init` writes:**

```toml
# .affi/config.toml — affidavit project config.
# Precedence (highest first): CLI flag > env (AFFI_*) > this file > ~/.config/affi/config.toml > defaults.
# Inspect the resolved values and where each came from with:  affi config explain

format       = "text"        # default output: "text" | "json"   (env: AFFI_FORMAT)
profile      = "core/v1"     # verifier profile                   (env: AFFI_PROFILE)
working_dir  = ".affi"       # holds working.json                 (env: AFFI_WORKING_DIR)
receipts_dir = ".affi/receipts"  # where `assemble` writes sealed receipts

[watch]
paths        = ["src", "tests"]   # directories the daemon watches recursively
debounce_ms  = 400                # coalesce bursts of saves into one run
on_change    = ["build-test"]     # profiles to run when the dust settles
auto_assemble = true              # seal a receipt after a quiet window
auto_verify   = true              # certify each sealed receipt (exit 2 on REJECT)

[cache]
enabled = true
dir     = ".affi/cache"           # content-addressed; skip re-verify on hash hit
```

### 3.3 `affi watch` — the auto-certify daemon

Drives the **existing** `crate::quality::file_watcher::FileWatcher` (src/quality.rs:1158)
— but instead of measuring quality, it runs the **provenance** loop: debounce → run the
configured profiles (emit) → `assemble` → `verify`. This replaces the stub at
src/handlers.rs:2632. Gated on the existing `file-watch` feature (notify + shell + tokio,
Cargo.toml:135).

**Loop shape & debounce.** A change wakes the watcher; we *coalesce* a burst by resetting a
quiet-window timer on every event and only acting once it expires (the same `Instant`-based
idea as src/quality.rs:1226, generalized to a trailing-edge debounce). Editor "save"
storms (write + rename + chmod) collapse into one certify.

```rust
// src/automation.rs  (new; behind feature = "file-watch")
use anyhow::Result;
use std::time::{Duration, Instant};

pub const STARTER_PROFILE: &str = r#"# .affi/profiles/build-test.toml
# A macro: a named, ordered list of emits. `affi run build-test` replays it.
name = "build-test"
[[step]]
type    = "build"
object  = "repo:main:git"
payload = "Cargo.toml"      # commitment = blake3(file bytes); raw bytes never stored
[[step]]
type    = "test"
object  = "suite:unit:test-suite"
payload = "-"               # read payload from stdin (e.g. piped test output)
"#;

pub struct WatchOptions {
    pub paths: Vec<String>,
    pub debounce: Duration,
    pub profiles: Vec<String>,
    pub auto_assemble: bool,
    pub auto_verify: bool,
    pub dry_run: bool,
}

/// Blocking auto-certify loop. Returns only on Ctrl-C / channel close.
pub fn watch(opts: WatchOptions) -> Result<()> {
    // Reuse the real watcher; its mpsc Receiver gives us FileChanged notifications.
    // (FileWatcher::new in src/quality.rs:1168 sets up notify + RecursiveMode::Recursive.)
    let mut watcher = crate::quality::file_watcher::FileWatcher::new(
        opts.paths.first().map(String::as_str).unwrap_or("src"),
        0, // we own debouncing here, so disable the watcher's internal gate
    )?;
    eprintln!("affi watch — paths={:?} debounce={:?}{}",
        opts.paths, opts.debounce, if opts.dry_run { " (dry-run)" } else { "" });

    let mut pending: Option<Instant> = None;
    loop {
        // Trailing-edge debounce: block until a change, then drain the burst with a
        // short timeout; only fire once no new event has arrived for `debounce`.
        match watcher.next_change(opts.debounce) {
            Change::File(path) => {
                eprintln!("  ∆ {}", path.display());
                pending = Some(Instant::now());
            }
            Change::Quiet => {
                if let Some(since) = pending.take() {
                    let _ = since; // window elapsed with no further events
                    run_cycle(&opts)?;
                }
            }
            Change::Closed => break,
        }
    }
    Ok(())
}

fn run_cycle(opts: &WatchOptions) -> Result<()> {
    let cycle_start = Instant::now();
    if opts.dry_run {
        eprintln!("  [dry-run] would run profiles {:?} → assemble → verify", opts.profiles);
        return Ok(());
    }
    // 1. EMIT: replay each profile's steps through the sealed seam crate::cli::emit.
    for prof in &opts.profiles {
        for step in load_profile(prof)?.steps {
            crate::cli::emit(&step.r#type, &step.object, &step.payload)?;  // src/cli.rs:29
        }
    }
    // 2. ASSEMBLE: fold + content-address via crate::cli::assemble (src/cli.rs:78).
    let asm = crate::cli::assemble(None)?;
    eprintln!("  ⛓ sealed {} ({} events)", asm.content_address, asm.event_count);

    // 3. VERIFY — but skip if we've already certified these exact bytes (cache, §3.6).
    if opts.auto_verify {
        if let Some(hit) = cache_get(&asm.content_address) {
            eprintln!("  ✓ {} (cached {})", verdict_word(hit), &asm.content_address[..12]);
        } else {
            let (code, verdict) = crate::cli::verify(&asm.receipt_path)?; // src/cli.rs:110
            cache_put(&asm.content_address, verdict.accepted);
            eprintln!("  {} {} — {}",
                if code == 0 { "✓ ACCEPT" } else { "✗ REJECT" },
                &asm.content_address[..12], verdict.reason);
            // Doctrine: we report the verdict; we never suppress or override it.
        }
    }
    eprintln!("  ↳ cycle {:?}", cycle_start.elapsed());
    Ok(())
}
```

The verb wrapper mirrors `src/verbs/monitor.rs` (which already declares `watch`, `interval`,
`format` params):

```rust
// src/verbs/watch.rs
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Auto-emit, assemble, and verify on file changes (debounced).
#[verb("watch", "receipt")]
pub fn watch(
    paths: Option<String>,       // comma-separated; falls back to config [watch].paths
    debounce_ms: Option<u64>,
    profile: Option<String>,     // comma-separated profile names
    dry_run: Option<bool>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::watch(paths, debounce_ms, profile, dry_run, format)
}
```

### 3.4 Supercharged git integration

Today's `install_git_hook` (src/handlers.rs:2977) installs **one** post-commit quality hook.
We keep it and add a provenance-aware hook *suite* — selectable, all routed through the
same verbs so behavior is identical to what a human would type.

| Hook | Fires | Action | Doctrine |
|---|---|---|---|
| `pre-commit` | before commit is recorded | `affi run --staged` → `assemble` → `verify`; **block on exit 2** | certifies; the *commit* decides, not affi |
| `commit-msg` | message assembled | append `Affidavit: <content-address>` trailer to the message | stamps provenance, decides nothing |
| `post-commit` | after commit | (existing) quality monitor, non-blocking | unchanged |

```rust
// src/handlers.rs — superset installer (keeps the old entrypoint working)
pub struct GitHookSpec { pub pre_commit: bool, pub commit_msg: bool, pub post_commit: bool,
                         pub threshold: String }
impl GitHookSpec {
    pub fn recommended() -> Self {
        Self { pre_commit: true, commit_msg: true, post_commit: false, threshold: "HIGH".into() }
    }
}

pub fn install_git_hook_v2(spec: GitHookSpec) -> Result<()> {
    let git_dir = determine_git_dir()?;                          // reused, src/handlers.rs
    let hooks = std::path::Path::new(&git_dir).join("hooks");
    std::fs::create_dir_all(&hooks).map_err(io_err)?;

    if spec.pre_commit  { write_hook(&hooks, "pre-commit",  PRE_COMMIT_HOOK)?; }
    if spec.commit_msg  { write_hook(&hooks, "commit-msg",  COMMIT_MSG_HOOK)?; }
    if spec.post_commit { write_hook(&hooks, "post-commit", &generate_post_commit_hook(&spec.threshold))?; }
    println!("✓ git hooks installed in {}", hooks.display());
    Ok(())
}

#[cfg(unix)]
fn write_hook(dir: &std::path::Path, name: &str, body: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, body).map_err(io_err)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).map_err(io_err)?;
    Ok(())
}

const PRE_COMMIT_HOOK: &str = r#"#!/usr/bin/env bash
# affi pre-commit — certify staged work before it is recorded. Bypass: git commit --no-verify
set -euo pipefail
affi receipt run --staged --quiet || { echo "affi: emit failed"   >&2; exit 1; }
addr=$(affi receipt assemble --format json | jq -r .content_address)
if ! affi receipt verify ".affi/receipts/${addr}.json" >/dev/null 2>&1; then
  echo "✗ affi: receipt ${addr:0:12} did NOT certify (REJECT). Commit blocked." >&2
  echo "  inspect: affi receipt verify .affi/receipts/${addr}.json" >&2
  exit 2     # verify's own REJECT code → git aborts the commit
fi
echo "✓ affi: ${addr:0:12} ACCEPT" >&2
echo "$addr" > .affi/.last-commit-addr   # handoff to commit-msg
"#;

const COMMIT_MSG_HOOK: &str = r#"#!/usr/bin/env bash
# affi commit-msg — stamp the certified receipt address into the commit message.
set -euo pipefail
addr_file=".affi/.last-commit-addr"
[ -f "$addr_file" ] || exit 0
addr=$(cat "$addr_file")
grep -q "^Affidavit:" "$1" || printf '\nAffidavit: %s\n' "$addr" >> "$1"
rm -f "$addr_file"
"#;
```

`affi doctor` is the CI-and-onboarding health check: it certifies the *toolchain*, not a
receipt — verifying that `.affi/` exists, config parses, hooks are installed, the
`file-watch` binary is built, and the golden lifecycle round-trips.

```rust
// src/handlers.rs
pub fn doctor(format: Option<String>) -> Result<()> {
    let mut checks: Vec<(&str, bool, String)> = Vec::new();
    let mut probe = |name, ok, note: String| checks.push((name, ok, note));

    probe("config present", std::path::Path::new(".affi/config.toml").exists(),
          ".affi/config.toml".into());
    let cfg_ok = crate::config::Config::resolve(&Default::default()).is_ok();
    probe("config parses", cfg_ok, "layered resolution".into());
    probe("git hook (pre-commit)",
          std::path::Path::new(".git/hooks/pre-commit").exists(), "blocks REJECT".into());
    probe("file-watch built", cfg!(feature = "file-watch"),
          "feature = \"file-watch\"".into());

    // Round-trip the sealed seams on a throwaway event: emit→assemble→verify.
    let lifecycle_ok = (|| -> anyhow::Result<bool> {
        crate::cli::emit("doctor", &["probe:diagnostic:health".into()], "Cargo.toml")?;
        let a = crate::cli::assemble(Some(".affi/cache/doctor.json"))?;
        Ok(crate::cli::verify(&a.receipt_path)?.0 == 0)
    })().unwrap_or(false);
    probe("lifecycle round-trip", lifecycle_ok, "emit→assemble→verify".into());

    let all = checks.iter().all(|(_, ok, _)| *ok);
    if format.as_deref() == Some("json") {
        let arr: Vec<_> = checks.iter()
            .map(|(n, ok, note)| serde_json::json!({"check": n, "ok": ok, "note": note})).collect();
        println!("{}", adapt(serde_json::to_string_pretty(
            &serde_json::json!({"healthy": all, "checks": arr})).map_err(anyhow::Error::from))?);
    } else {
        for (n, ok, note) in &checks { eprintln!("  {} {n} — {note}", if *ok {"✓"} else {"✗"}); }
        eprintln!("{}", if all {"doctor: healthy"} else {"doctor: PROBLEMS FOUND"});
    }
    if !all { std::process::exit(2); }   // CI-friendly: non-zero on any failure
    Ok(())
}
```

### 3.5 Profiles, aliases & macros

A **profile** is a named, version-controlled emit sequence in `.affi/profiles/<name>.toml`
(see `STARTER_PROFILE`). `affi run <name>` replays its steps through `crate::cli::emit`,
then optionally assembles. `--staged` derives objects from `git diff --cached --name-only`
so a pre-commit emit reflects exactly what's being committed. This kills the "retype the
golden_run sequence" friction (§2) and stops cross-dev drift.

```rust
// src/handlers.rs
pub fn run(profile: String, staged: Option<bool>, assemble: Option<bool>,
           dry_run: Option<bool>, format: Option<String>) -> Result<()> {
    let prof = crate::automation::load_profile(&profile)
        .map_err(|e| NounVerbError::execution_error(format!("profile '{profile}': {e}")))?;
    let mut steps = prof.steps;
    if staged.unwrap_or(false) { steps = crate::automation::staged_steps()?; } // from git index

    if dry_run.unwrap_or(false) {
        for s in &steps { eprintln!("  [dry-run] emit --type {} --object {}", s.r#type, s.object); }
        return Ok(());
    }
    for s in &steps {
        let objs: Vec<String> = s.object.split(',').map(|x| x.trim().to_string()).collect();
        adapt(crate::cli::emit(&s.r#type, &objs, &s.payload))?;   // sealed seam
    }
    if assemble.unwrap_or(false) {
        let a = adapt(crate::cli::assemble(None))?;
        print_json_or(&format, &serde_json::json!({"content_address": a.content_address}),
            || println!("ran '{profile}' ({} steps) → {}", steps.len(), a.content_address))?;
    } else {
        println!("ran profile '{profile}' — {} event(s) staged in working set", steps.len());
    }
    Ok(())
}
```

### 3.6 `--dry-run` everywhere + content-addressed caching

- **`--dry-run`** on `emit`, `assemble`, `run`, `watch`, and the hooks prints the *plan*
  (objects, payload sources, the would-be commands) and mutates nothing — safe to wire into
  CI preview steps and to explore profiles.
- **Cache.** `verify` is pure over a receipt's content address (the assemble output's
  `content_address`, src/cli.rs:92). Cache `address → verdict.accepted` under
  `.affi/cache/`. On a cache hit, `watch`/CI skip the full 7-stage + admission re-run. This
  is *not* a doctrine compromise: the address is `blake3(canonical bytes)`, so a hit means
  *byte-identical* bytes already certified — the cache returns the same verdict the pipeline
  would, never a manufactured one.

```rust
// src/automation.rs — trivial, transparent cache. Any miss falls back to the real pipeline.
fn cache_path(addr: &str) -> std::path::PathBuf {
    std::path::Path::new(".affi/cache").join(format!("{addr}.verdict"))
}
pub fn cache_get(addr: &str) -> Option<bool> {
    std::fs::read_to_string(cache_path(addr)).ok().map(|s| s.trim() == "ACCEPT")
}
pub fn cache_put(addr: &str, accepted: bool) {
    let _ = std::fs::create_dir_all(".affi/cache");
    let _ = std::fs::write(cache_path(addr), if accepted {"ACCEPT"} else {"REJECT"});
}
```

---

## 4. UX — terminal transcripts

### 4.1 First run

```text
$ affi init --with-hooks
✓ .affi/ scaffolded (profiles/, receipts/, cache/)
✓ wrote .affi/config.toml
✓ starter profile: .affi/profiles/build-test.toml
✓ git hooks installed in .git/hooks   (pre-commit, commit-msg)
next: `affi watch` to auto-certify, or `affi run build-test` once
```

### 4.2 Config resolution, made discoverable

```text
$ AFFI_FORMAT=json affi config explain
field         value             source           overridable by
------------  ----------------  ---------------  -----------------------------
format        json              env              flag --format
profile       core/v1           project-config   flag --profile / AFFI_PROFILE
working_dir   .affi             default          AFFI_WORKING_DIR / config
receipts_dir  .affi/receipts    project-config   AFFI_RECEIPTS_DIR / config
strict        false             default          flag --strict
watch.paths   [src, tests]      project-config   [watch] paths in config.toml
cache.enabled true              project-config   [cache] enabled

resolution order: flag > env (AFFI_*) > .affi/config.toml > ~/.config/affi/config.toml > defaults
```

### 4.3 An `affi watch` session reacting to edits

```text
$ affi watch
affi watch — paths=["src", "tests"] debounce=400ms

# (you save src/lib.rs three times in two seconds — one cycle, not three)
  ∆ src/lib.rs
  ∆ src/lib.rs
  ∆ src/lib.rs
  ⛓ sealed 203d3bbf91c4… (2 events)
  ✓ ACCEPT 203d3bbf91c4 — all stages passed
  ↳ cycle 78ms

# (you edit tests/e2e.rs)
  ∆ tests/e2e.rs
  ⛓ sealed a2d95f1130ab… (2 events)
  ✓ ACCEPT a2d95f1130ab — all stages passed
  ↳ cycle 71ms

# (no source change — re-trigger hits the cache, no recompute)
  ∆ src/lib.rs
  ⛓ sealed 203d3bbf91c4… (2 events)
  ✓ ACCEPT (cached 203d3bbf91c4)
  ↳ cycle 2ms
^C
```

### 4.4 A pre-commit hook firing (block on REJECT)

```text
$ git commit -m "tighten verifier stage 4"
✓ affi: 6ef47c8290a1 ACCEPT
[main 1a2b3c4] tighten verifier stage 4
 2 files changed, 31 insertions(+)

# message now carries the stamp:
$ git log -1 --format=%B
tighten verifier stage 4

Affidavit: 6ef47c8290a1b3d4e5f6...

# a tampered / non-certifying change is stopped before it lands:
$ git commit -m "wip"
✗ affi: receipt a2d95f1130ab did NOT certify (REJECT). Commit blocked.
  inspect: affi receipt verify .affi/receipts/a2d95f1130ab.json
# (exit 2 — git aborts; bypass deliberately with `git commit --no-verify`)
```

### 4.5 A CI run

```text
$ affi doctor
  ✓ config present — .affi/config.toml
  ✓ config parses — layered resolution
  ✓ git hook (pre-commit) — blocks REJECT
  ✓ file-watch built — feature = "file-watch"
  ✓ lifecycle round-trip — emit→assemble→verify
doctor: healthy
```

### 4.6 Ready-to-paste GitHub Actions

```yaml
# .github/workflows/provenance.yml
name: provenance
on: [push, pull_request]

jobs:
  certify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build affi
        run: cargo build --release --features file-watch --bin affi

      - name: Add affi to PATH
        run: echo "${{ github.workspace }}/target/release" >> "$GITHUB_PATH"

      - name: Scaffold + health check
        run: |
          affi receipt init --format json
          affi receipt doctor          # exits non-zero on any problem → fails the job

      - name: Record this build as a certified receipt
        run: |
          affi receipt run build-test --assemble --format json | tee assemble.json
          ADDR=$(jq -r .content_address assemble.json)
          affi receipt verify ".affi/receipts/${ADDR}.json"   # exit 0 ACCEPT / 2 REJECT

      - name: Upload provenance receipt
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: provenance-receipts
          path: .affi/receipts/*.json
```

---

## 5. Integration & rollout

### Touched surfaces (all additive — no existing file's behavior changes)

| Area | New | Reused / cited |
|---|---|---|
| Verbs | `src/verbs/{init,watch,doctor,run,config_cmd}.rs` | macro pattern from `src/verbs/{emit,assemble,verify,monitor}.rs` |
| Handlers | `init`, `watch`, `doctor`, `run`, `config_explain`, `install_git_hook_v2` in `src/handlers.rs` | `adapt`, `io_err`, `print_json_or`, `determine_git_dir`, `generate_post_commit_hook` |
| Library | `src/config.rs`, `src/automation.rs`; re-export from `src/lib.rs` | `crate::cli::{emit,assemble,verify}`, `crate::chain::WORKING_PATH` |
| Watch engine | trailing-edge debounce + `run_cycle` | `crate::quality::file_watcher::FileWatcher` (src/quality.rs:1158) |
| Features (Cargo.toml) | `config = ["dep:toml"]`; fold into `core` | existing `file-watch` (notify+shell+tokio), `json-output` |

**New dependency:** `toml = "0.8"` (config parsing), behind a `config` feature folded into
`core` so `affi init`/`config` work out of the box.

### Phasing

**P0 — make the common path effortless**
- `affi init` + `src/config.rs` resolver + `config explain` — **M**
- `--dry-run` on `emit`/`assemble`/`run` (plan-only branch in each handler) — **S**
- `affi run <profile>` + `STARTER_PROFILE` macro format — **M**

**P1 — close the automation loop**
- `affi watch` over the existing `FileWatcher`, replacing the src/handlers.rs:2632 stub — **M**
- `install_git_hook_v2` (pre-commit verify + commit-msg stamp), keeping the old verb — **M**
- `affi doctor` toolchain health check for CI + onboarding — **S**

**P2 — polish & scale**
- Content-addressed verdict cache (`.affi/cache/`) wired into `watch` + CI — **S**
- `--staged` object derivation from the git index — **M**
- Webhook/desktop notification on `watch` REJECT (reuse `webhook` feature, Cargo.toml:173) — **L**

### Doctrine checklist (every feature re-validated)

- `init`/`config`/`run`/`watch` only *drive* `crate::cli::{emit,assemble,verify}`; none
  construct a `Receipt` or mint `Admitted` directly — the seal stays sealed.
- Hooks and CI surface the verifier's own exit code (`0`/`2`); they *report*, never override.
- The cache returns a stored verdict **only** on a content-address (byte-identity) hit — it
  cannot fabricate an ACCEPT for bytes the pipeline hasn't seen.
- `affi doctor` certifies the toolchain's readiness; it does not opine on whether any
  recorded work is "honest." Certify, don't decide — end to end.
