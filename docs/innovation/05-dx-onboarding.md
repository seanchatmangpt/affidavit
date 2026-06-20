# 05 — DX, Onboarding & Discoverability: 1000x Developer Experience

**Status:** Design proposal · **Scope:** new verbs + REPL upgrade + TUI sketch
**Target binary:** `affi` (and `affi-shell`) · **Version baseline:** 26.6.17
**Doctrine constraint:** *certify, don't decide* — none of this changes what a verdict means; it only changes how fast a human reaches one.

> ⚠️ **Build caveat:** This crate depends on `26.6`-pinned local crates (`clap-noun-verb`,
> `wasm4pm-compat`, etc.) that do not resolve in a clean environment. The Rust below is
> **compilable-style** (real types, real seams) but is **not** compiled here. Treat it as a
> spec against `src/handlers.rs`, `src/verbs/`, and `src/bin/affi-shell.rs`.

---

## 1. Vision — The 1000x Leap

Today a new user faces **67 verbs under a single `receipt` noun** (verified: `grep -c '#[verb(... "receipt")' src/verbs` → 67, plus `bench profile`), a README that lists ~12 of them, and `examples/golden_run.sh` that you must *read shell* to learn from. The surface is enormous and **opaque**: there is no `affi tutorial`, no `affi examples`, no command-level `affi search`, no "did you mean", no man pages, and the `affi-shell` REPL hardcodes **16** completion words and dispatches **11** of the 67 verbs (`src/bin/affi-shell.rs:121-138, 229-356`).

The 1000x leap is a path from **"67 opaque verbs and a README"** to **"guided, discoverable, interactive mastery"**:

| Dimension | Today (1x) | Proposed (1000x) |
|---|---|---|
| First 5 minutes | Read README, copy a `cargo run` incantation, hope | `affi tutorial` runs the real emit→assemble→verify→tamper loop *with narration* |
| "What can this do?" | Scroll 67 flat verbs | `affi search tamper`, grouped `--help`, `affi examples` |
| Wrong command | bare clap error | "did you mean `verify`?" + the 3 nearest verbs + an example |
| Recipes | none | `affi examples verify` → copy-pasteable, runnable snippets |
| REPL | 11/67 verbs, no chain context | full dispatch, working-chain context, completion from the live registry |
| Browse a receipt | `show` dumps text to stderr | optional TUI: events ▸ chain ▸ verdict, navigable |
| First run | nothing | banner → `affi doctor` + `affi tutorial` |

**The bet:** discoverability is a *force multiplier*. 67 verbs the user can't find are worth ~12. Make all 67 findable, learnable, and runnable-from-example and the effective surface — and the product's value — goes up by orders of magnitude.

---

## 2. Current State — What Exists, What's Missing

### 2.1 What onboarding exists

- **`examples/golden_run.sh`** — the canonical lifecycle: emits two events (`receipt emit --type seed ...`, `--type validate ...`), `receipt assemble --out receipt.json`, `receipt verify` (ACCEPT, exit 0), then `sed -i 's/"validate"/"tampered"/'` and `receipt verify` again (REJECT, non-zero). This is *the* teaching artifact — but it is a **shell script you read**, not an experience you run interactively. It also shells out via `cargo run` (`golden_run.sh:21`).
- **`README.md`** — lists the four core verbs plus a sampling of `quality`/`sbom` verbs and admits *"For the complete list of all 59 capabilities, run `affi --help`"* (`README.md:101`). The count is already stale (59 vs. 67 in `src/verbs/mod.rs`).
- **`affi-shell`** (`src/bin/affi-shell.rs`) — a `rustyline` REPL with filename completion, a `MatchingBracketHighlighter`, a `HistoryHinter`, multi-line via trailing `\`, and persisted `.affi_history`. Good bones.
- **`receipt search`** (`src/verbs/search.rs` → `handlers::search`, `handlers.rs:1185`) — full-text grep over *receipt payloads/event fields*. **Not** command discovery.

### 2.2 The gaps (the real pain)

1. **No guided tutorial.** The only "tour" is a non-interactive bash file.
2. **67 verbs, one noun, no grouping.** Every verb is `#[verb("...", "receipt")]` (e.g. `src/verbs/show.rs:13`, `search.rs:13`). `affi receipt --help` is an undifferentiated wall: `emit`, `emit-batch`, `emit-from-github`, `dora-metrics`, `sbom-scan`, `hipaa`, `bus-factor`, `gdpr-proof`, … There is no "emission" vs. "verification" vs. "compliance" vs. "analytics" grouping anywhere the user can see.
3. **No command search.** Want "the verb that finds what a change breaks"? It's `find-blast-radius` (`handlers.rs:1234`) — undiscoverable unless you already know the name.
4. **No "did you mean".** `affi receipt verfiy r.json` yields a flat clap error, not a suggestion.
5. **No examples per verb.** Argument shapes live only in `handlers.rs` signatures (`verify(receipt, format, profile, strict)`, `attest(receipt, attestation_type, out, format)`), invisible at the CLI.
6. **REPL is a stub.** It completes 16 words and dispatches 11 verbs (no `diff`, `query`, `timeline`, `attest`, `sbom-*`, `dora-metrics`, …), has **no working-chain context** (each `verify` needs a full path), and its completion list is **hand-maintained and already drifted** from the registry.
7. **No man pages, no first-run nudge, no `doctor`.**

---

## 3. Proposed Design

Five additions, all under the existing `receipt` noun (and a new **`guide`** noun for meta-commands), all delegating to `crate::handlers::*` exactly like every current verb — so they ride the existing `linkme`/`clap_noun_verb::run()` registration (`src/lib.rs:129`) with zero framework changes.

```
guide tutorial     # interactive, stateful walkthrough (live golden_run.sh)
guide examples     # searchable, copy-pasteable recipes per verb
guide search       # command discovery across all 67 verbs
guide doctor       # environment + working-chain health check
guide man          # generate man pages / markdown for every verb
```

A new noun `guide` keeps these *meta* commands out of the already-crowded `receipt` namespace and signals "start here". They are added as thin wrappers in `src/verbs/` (e.g. `src/verbs/tutorial.rs`) following the `show.rs` template precisely.

### 3.1 Grouping the 67 verbs (the spine of discoverability)

Introduce a **single source of truth** for verb metadata — group, summary, examples — that every surface (`--help`, `search`, `examples`, `tutorial`, REPL completion, `man`) reads. This kills the drift problem (the REPL's stale 16-word list) at the root.

```rust
// src/registry.rs  (new) — the one place verbs are described.
/// Stable taxonomy used by help, search, examples, REPL, and man-pages.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Core,         // emit, assemble, verify, show
    Emission,     // emit-batch, emit-from-github/gitlab/cicd/cloud/...
    Verification, // verify-family, verify-sla, verify-compliance, attest, notarize, sign
    Display,      // inspect, diff, stats, graph, replay, visualize, catalog, timeline
    Query,        // query, search, causality-chain, find-blast-radius, root-cause
    Analytics,    // dora-metrics, team-velocity, tech-debt, security-debt, variance, ...
    Compliance,   // hipaa, pci-dss, soc2-audit, gdpr-proof, license-compliance
    Sbom,         // sbom-scan, sbom-attest, sbom-compliance, sbom-ntia, sbom-blast-radius
    Quality,      // monitor, trend-analysis, portfolio-health, anomaly-detect
    Meta,         // tutorial, examples, search, doctor, man
}

pub struct VerbDoc {
    pub noun: &'static str,        // "receipt" | "guide" | "bench"
    pub verb: &'static str,        // "find-blast-radius"
    pub group: Group,
    pub summary: &'static str,     // one line, mirrors the /// doc on the wrapper
    pub keywords: &'static [&'static str], // search synonyms: ["tamper","downstream","impact"]
    pub examples: &'static [Example],
}

pub struct Example {
    pub title: &'static str,
    pub cmd: &'static str,         // copy-pasteable, e.g. "affi receipt verify receipt.json"
    pub note: &'static str,
}

/// Registered once; ideally generated, but a static slice is fine to start.
/// `linkme` already underpins verb registration, so this can be a
/// `#[distributed_slice]` populated next to each `#[verb]` wrapper later.
pub fn all() -> &'static [VerbDoc] { &REGISTRY }

pub fn by_group(g: Group) -> impl Iterator<Item = &'static VerbDoc> {
    REGISTRY.iter().filter(move |d| d.group == g)
}
```

**Help that teaches.** `affi --help` (and `affi receipt --help`) render *by group*, not flat, with a teaching header pointing at `tutorial`/`examples`. Because `clap-noun-verb` owns the actual parser, the cleanest non-invasive path is a custom top-level `affi guide help` / `affi help` (a `guide` verb) that prints the grouped view from `registry::all()`, while the framework's own `--help` stays as the exhaustive fallback. (If `clap-noun-verb` exposes help-template hooks, wire the same data in; otherwise the `guide help` view is the teaching surface.)

### 3.2 `affi guide tutorial` — a step / state machine

The tutorial is the **live, narrated** `golden_run.sh`. It is a deterministic state machine over `Step`s; each step explains *why*, runs the *real* handler (not a mock), shows output, and waits for the user (or `--auto`/`--yes` for CI). It runs in a temp dir so the user's tree is untouched — mirroring `golden_run.sh:23` (`mktemp -d` + `trap rm -rf`).

```rust
// src/tutorial.rs (new)
use anyhow::Result;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// One teaching beat: narration + a real action against a sandbox dir.
pub struct Step {
    pub title: &'static str,
    pub teach: &'static str,           // the "why", printed before running
    pub command_hint: &'static str,    // the equivalent CLI the user could type
    pub run: fn(&Sandbox) -> Result<StepOutcome>,
    pub expect: Expect,                 // what success looks like (drives narration)
}

pub enum Expect { Accept, Reject, Info }

pub struct StepOutcome { pub summary: String, pub exit_code: i32 }

/// Isolated scratch dir; auto-removed on drop (like golden_run's trap).
pub struct Sandbox { pub dir: PathBuf }
impl Sandbox {
    pub fn new() -> Result<Self> {
        let dir = std::env::temp_dir().join(format!("affi-tutorial-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }
    pub fn path(&self, name: &str) -> PathBuf { self.dir.join(name) }
}
impl Drop for Sandbox {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.dir); }
}

#[derive(Clone, Copy)]
pub struct RunOpts { pub auto: bool, pub color: bool }

/// The script — same shape as golden_run.sh, but each beat is teachable.
pub fn lifecycle_steps() -> Vec<Step> {
    vec![
        Step {
            title: "Emit your first operation-event",
            teach: "A receipt is a BLAKE3 chain of events. `emit` appends ONE event to \
                    the working receipt (.affi/working.json). The payload is hashed; the \
                    digest becomes the commitment — the raw bytes are never stored.",
            command_hint: "affi receipt emit --type seed --object art1:artifact:input --payload payload_a.txt",
            run: |sb| {
                std::fs::write(sb.path("payload_a.txt"), b"source bytes for stage one\n")?;
                let out = crate::cli::emit(
                    "seed",
                    &["art1:artifact:input".into()],
                    sb.path("payload_a.txt").to_str().unwrap(),
                )?;
                Ok(StepOutcome { summary: format!("seq {} · commit {}…", out.seq, &out.commitment[..12]), exit_code: 0 })
            },
            expect: Expect::Info,
        },
        Step {
            title: "Seal the chain (assemble)",
            teach: "`assemble` folds every event into a rolling chain hash and writes an \
                    immutable, content-addressed receipt. Editing any event later changes \
                    this hash — that's what makes tampering detectable.",
            command_hint: "affi receipt assemble --out receipt.json",
            run: |sb| {
                let out = crate::cli::assemble(Some(sb.path("receipt.json").to_str().unwrap()))?;
                Ok(StepOutcome { summary: format!("content address {}…", &out.content_address[..12]), exit_code: 0 })
            },
            expect: Expect::Info,
        },
        Step {
            title: "Verify an honest receipt → ACCEPT",
            teach: "The 7-stage certify pipeline recomputes the chain and adjudicates. \
                    An untouched receipt passes all stages: verdict ACCEPT, exit 0.",
            command_hint: "affi receipt verify receipt.json",
            run: |sb| {
                let (code, v) = crate::cli::verify(sb.path("receipt.json").to_str().unwrap())?;
                Ok(StepOutcome { summary: format!("{} — {}", if v.accepted {"ACCEPT"} else {"REJECT"}, v.reason), exit_code: code })
            },
            expect: Expect::Accept,
        },
        Step {
            title: "Tamper, then verify → REJECT",
            teach: "Now we flip one event_type on disk (validate → tampered), exactly like \
                    golden_run.sh. The recomputed chain hash no longer matches the stored \
                    one, so chain_integrity fails: verdict REJECT, non-zero exit.",
            command_hint: "sed -i 's/\"validate\"/\"tampered\"/' receipt.json  &&  affi receipt verify receipt.json",
            run: |sb| {
                let p = sb.path("receipt.json");
                let s = std::fs::read_to_string(&p)?.replace("\"seed\"", "\"tampered\"");
                std::fs::write(&p, s)?;
                let (code, v) = crate::cli::verify(p.to_str().unwrap())?;
                Ok(StepOutcome { summary: format!("{} — {}", if v.accepted {"ACCEPT"} else {"REJECT"}, v.reason), exit_code: code })
            },
            expect: Expect::Reject,
        },
    ]
}

/// The runner: narrate → pause → run → confirm expectation.
pub fn run(opts: RunOpts) -> Result<()> {
    let sb = Sandbox::new()?;
    let steps = lifecycle_steps();
    let total = steps.len();
    for (i, step) in steps.iter().enumerate() {
        banner(i + 1, total, step.title, opts.color);
        println!("{}\n", wrap(step.teach, 78));
        println!("  $ {}\n", step.command_hint);
        if !opts.auto { prompt_enter("[enter] to run, q to quit")?; }

        let outcome = (step.run)(&sb)?;
        let ok = match step.expect {
            Expect::Accept => outcome.exit_code == 0,
            Expect::Reject => outcome.exit_code != 0,
            Expect::Info   => true,
        };
        println!("  → {}  {}", if ok { "✓" } else { "✗" }, outcome.summary);
        if !ok { anyhow::bail!("tutorial step {} did not behave as taught", i + 1); }
        println!();
    }
    println!("You ran the full provenance loop: emit → assemble → ACCEPT → REJECT.");
    println!("Next:  affi guide examples verify   ·   affi guide search <topic>   ·   affi-shell");
    Ok(())  // sandbox auto-cleans on drop
}

fn banner(n: usize, total: usize, title: &str, color: bool) {
    let head = format!("── Step {n}/{total} · {title} ");
    if color { println!("\x1b[1;34m{:─<80}\x1b[0m", head); } else { println!("{:─<80}", head); }
}
fn prompt_enter(msg: &str) -> Result<()> {
    print!("{msg} "); io::stdout().flush()?;
    let mut s = String::new(); io::stdin().read_line(&mut s)?;
    if s.trim().eq_ignore_ascii_case("q") { anyhow::bail!("tutorial aborted"); }
    Ok(())
}
fn wrap(s: &str, _w: usize) -> String { s.split_whitespace().collect::<Vec<_>>().join(" ") }
```

The wrapper that exposes it (identical shape to `src/verbs/show.rs`):

```rust
// src/verbs/tutorial.rs (new)
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Interactive, narrated walkthrough of the provenance lifecycle
#[verb("tutorial", "guide")]
pub fn tutorial(auto: Option<bool>, color: Option<String>) -> Result<()> {
    crate::handlers::tutorial(auto.unwrap_or(false), color)   // delegate, per house pattern
}
```

### 3.3 `affi guide examples` — searchable recipes

Reads `registry::Example`s. `affi guide examples` lists groups; `affi guide examples <verb>` prints that verb's recipes; `affi guide examples --search <term>` greps titles+commands+keywords. Everything is copy-pasteable.

```rust
// in src/handlers.rs (new handler, delegates to registry)
pub fn examples(verb: Option<String>, search: Option<String>, format: Option<String>) -> Result<()> {
    let docs = crate::registry::all();
    let hits: Vec<&crate::registry::VerbDoc> = match (&verb, &search) {
        (Some(v), _) => docs.iter().filter(|d| d.verb == v).collect(),
        (_, Some(q)) => {
            let q = q.to_lowercase();
            docs.iter().filter(|d|
                d.examples.iter().any(|e|
                    e.title.to_lowercase().contains(&q) || e.cmd.to_lowercase().contains(&q))
                || d.keywords.iter().any(|k| k.contains(&q))
            ).collect()
        }
        _ => docs.iter().collect(),
    };
    // JSON branch mirrors every other handler (e.g. handlers.rs:122).
    for d in hits {
        eprintln!("\n{} {}  ({:?})", d.noun, d.verb, d.group);
        for ex in d.examples {
            eprintln!("  • {}", ex.title);
            eprintln!("      $ {}", ex.cmd);
            if !ex.note.is_empty() { eprintln!("        {}", ex.note); }
        }
    }
    let _ = format;
    Ok(())
}
```

### 3.4 `affi guide search` + "did you mean" (Levenshtein)

Two uses of the same suggester: explicit discovery (`guide search tamper`) and implicit correction (typo on any verb).

```rust
// src/suggest.rs (new)
/// Classic bounded edit distance (Wagner–Fischer), good enough at 67 verbs.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

pub struct Suggestion { pub verb: &'static str, pub group: crate::registry::Group, pub score: usize }

/// Rank verbs by closeness to `input`. Substring matches sort first (score 0).
pub fn nearest(input: &str, limit: usize) -> Vec<Suggestion> {
    let input = input.to_lowercase();
    let mut ranked: Vec<Suggestion> = crate::registry::all().iter().map(|d| {
        let exact_sub = d.verb.contains(&input)
            || d.keywords.iter().any(|k| k.contains(&input));
        let score = if exact_sub { 0 } else { edit_distance(&input, d.verb) };
        Suggestion { verb: d.verb, group: d.group, score }
    }).collect();
    ranked.sort_by_key(|s| s.score);
    ranked.into_iter().take(limit).collect()
}

/// Used by the error path: only suggest when the typo is *close*.
pub fn did_you_mean(unknown: &str) -> Option<Vec<Suggestion>> {
    let near = nearest(unknown, 3);
    let best = near.first()?;
    // threshold scales with length; reject wild misses.
    if best.score <= (unknown.len() / 3).max(2) { Some(near) } else { None }
}
```

`guide search` handler just calls `nearest(topic, 10)` and prints grouped results with each verb's one-line summary + first example. The **"did you mean"** path wraps the dispatch boundary: in `affi-shell`'s `dispatch` (today the catch-all is `_ => Err(anyhow!("Unknown command: {}", args[0]))`, `src/bin/affi-shell.rs:359`) and, where `clap-noun-verb` surfaces an unknown-subcommand error, in the top-level `run()` adapter.

### 3.5 REPL upgrades (`affi-shell`)

The current REPL (`src/bin/affi-shell.rs`) is the right foundation but undercooked. Five upgrades:

1. **Registry-driven completion** — replace the hardcoded 16-word `verbs` vec (`affi-shell.rs:121-138`) with `registry::all()`. Completion now covers all 67 verbs and never drifts.
2. **Working-chain context** — a `Session` holds the current receipt/working dir so you stay "inside" a chain. The prompt reflects it: `affi(2 evt)>`. `verify` with no arg verifies the session receipt.
3. **Full dispatch** — route *every* verb through `crate::handlers::*` via one table, not the 11 hand-written arms (`affi-shell.rs:229-356`).
4. **Inline `help <verb>` and `examples <verb>`** — pull straight from the registry, so REPL help == CLI help.
5. **"did you mean" on unknown** — call `suggest::did_you_mean` in the catch-all.

```rust
// sketch: src/bin/affi-shell.rs — Session + registry-driven loop
struct Session {
    receipt: Option<std::path::PathBuf>,  // "current" assembled receipt
    emitted: usize,                       // events in the working chain this session
}
impl Session {
    fn prompt(&self) -> String {
        match (self.emitted, &self.receipt) {
            (0, None) => "affi> ".into(),
            (n, None) => format!("affi({n} evt)> "),
            (_, Some(p)) => format!("affi[{}]> ", p.file_name().unwrap().to_string_lossy()),
        }
    }
}

async fn dispatch(line: &str, sess: &mut Session) -> anyhow::Result<()> {
    let args = shlex::split(line).ok_or_else(|| anyhow::anyhow!("bad quoting"))?;
    let Some(cmd) = args.first() else { return Ok(()) };

    match cmd.as_str() {
        "exit" | "quit" => std::process::exit(0),
        "help"     => { print_help(args.get(1).map(String::as_str)); return Ok(()); }
        "examples" => { affidavit::handlers::examples(args.get(1).cloned(), None, None)?; return Ok(()); }
        "use"      => { sess.receipt = args.get(1).map(Into::into); return Ok(()); } // set context
        "receipt" | "guide" | "bench" => {} // fallthrough to verb dispatch below
        other => {
            // implicit context: bare `verify` == `receipt verify <session receipt>`
            if let Some(doc) = affidavit::registry::all().iter().find(|d| d.verb == other) {
                return run_verb(doc, &args[1..], sess).await;
            }
            // unknown → did you mean?
            if let Some(sugs) = affidavit::suggest::did_you_mean(other) {
                eprintln!("unknown command '{other}'. did you mean:");
                for s in sugs { eprintln!("  {}  ({:?})", s.verb, s.group); }
            } else {
                eprintln!("unknown command '{other}' — try `help` or `guide search <topic>`");
            }
            return Ok(());
        }
    }
    // <noun> <verb> [args] path: look up by (noun, verb), spawn_blocking the handler.
    let (noun, verb) = (cmd.as_str(), args.get(1).map(String::as_str).unwrap_or(""));
    match affidavit::registry::all().iter().find(|d| d.noun == noun && d.verb == verb) {
        Some(doc) => run_verb(doc, &args[2..], sess).await,
        None => { eprintln!("unknown {noun} subcommand '{verb}'"); Ok(()) }
    }
}
```

`run_verb` injects the session receipt when a path arg is omitted and `spawn_blocking`s the handler exactly as the current code does (`affi-shell.rs:261-265`).

### 3.6 `affi guide man` — generated man pages

One handler walks `registry::all()` and emits troff (`man`) and Markdown for every verb — group, synopsis (from the handler signature), summary, examples. Wire into the build later as `cargo xtask man` writing to `target/man/affi-receipt-verify.1`. Zero hand-maintenance because it reads the same registry.

### 3.7 First-run experience + `affi guide doctor`

On a bare invocation with no args/subcommand, print a short banner instead of (or before) the framework's usage:

```
affidavit 26.6.17 — the provenance layer. certify, don't decide.
New here?   affi guide tutorial      Health check?  affi guide doctor
67 verbs.   affi guide search <topic>  ·  affi guide examples
```

A `.affi/.welcomed` sentinel (next to the existing `.affi/working.json`) shows it once. `affi guide doctor` checks: binary version, presence/validity of `.affi/working.json`, write-perms on `.affi/`, enabled feature flags (so `stats`/`graph` users learn they need `--features discovery`, mirroring `handlers.rs:851-858`), and whether `affi-shell` is available (`required-features = ["shell"]`, `Cargo.toml:23`).

---

## 4. UX — Transcripts & TUI Mock

### 4.1 `affi guide tutorial`

```
$ affi guide tutorial
affidavit tutorial — you'll run the real provenance loop in a scratch dir.
Nothing here touches your project. Press q at any prompt to quit.

── Step 1/4 · Emit your first operation-event ──────────────────────────────────
A receipt is a BLAKE3 chain of events. `emit` appends ONE event to the working
receipt (.affi/working.json). The payload is hashed; the digest becomes the
commitment — the raw bytes are never stored.

  $ affi receipt emit --type seed --object art1:artifact:input --payload payload_a.txt

[enter] to run, q to quit
  → ✓  seq 0 · commit 6ef47c82a1b3…

── Step 2/4 · Seal the chain (assemble) ────────────────────────────────────────
`assemble` folds every event into a rolling chain hash and writes an immutable,
content-addressed receipt. Editing any event later changes this hash — that's
what makes tampering detectable.

  $ affi receipt assemble --out receipt.json

[enter] to run, q to quit
  → ✓  content address 203d3bbf91ac…

── Step 3/4 · Verify an honest receipt → ACCEPT ────────────────────────────────
The 7-stage certify pipeline recomputes the chain and adjudicates. An untouched
receipt passes all stages: verdict ACCEPT, exit 0.

  $ affi receipt verify receipt.json

[enter] to run, q to quit
  → ✓  ACCEPT — all 7 stages passed

── Step 4/4 · Tamper, then verify → REJECT ─────────────────────────────────────
Now we flip one event_type on disk (seed → tampered), exactly like golden_run.sh.
The recomputed chain hash no longer matches the stored one, so chain_integrity
fails: verdict REJECT, non-zero exit.

  $ sed -i 's/"seed"/"tampered"/' receipt.json  &&  affi receipt verify receipt.json

[enter] to run, q to quit
  → ✓  REJECT — chain hash mismatch at stage chain_integrity

You ran the full provenance loop: emit → assemble → ACCEPT → REJECT.
Next:  affi guide examples verify   ·   affi guide search <topic>   ·   affi-shell
```

### 4.2 `affi guide examples verify`

```
$ affi guide examples verify

receipt verify  (Core)
  • Verify an assembled receipt (ACCEPT → exit 0)
      $ affi receipt verify receipt.json
  • Machine-readable verdict for CI gating
      $ affi receipt verify receipt.json --format json
        exit code is 0 on ACCEPT, 2 on REJECT — gate your pipeline on it.
  • Verify every receipt in a directory
      $ affi receipt verify-family ./receipts/
  • Verify against a compliance framework
      $ affi receipt verify-compliance receipt.json --framework soc2

Related:  attest · notarize · diagnose · inspect      (affi guide search verify)
```

### 4.3 "Did you mean" correction

```
$ affi receipt verfiy receipt.json
error: unknown verb 'verfiy' for noun 'receipt'

did you mean:
  verify           (Core)         run the 7-stage certify pipeline
  verify-family    (Verification) verify every receipt in a directory
  verify-sla       (Verification) check a receipt meets SLA targets

try:  affi guide examples verify   ·   affi guide search verify
```

```
$ affi receipt blast-radius
error: unknown verb 'blast-radius' for noun 'receipt'

did you mean:
  find-blast-radius (Query)  find downstream events/objects affected by a change
  sbom-blast-radius (Sbom)   propagate vulnerability risk through the dep graph
```

### 4.4 REPL session (`affi-shell`)

```
$ affi-shell
Affidavit Shell — v26.6.17 · 67 verbs · type `help`, `help <verb>`, or `guide search <topic>`
affi> emit --type seed --object art1:artifact:input --payload ./a.txt
emitted event evt-0 (seq 0)
affi(1 evt)> emit --type validate --object art1:artifact:output --payload ./b.txt
emitted event evt-1 (seq 1)
affi(2 evt)> assemble --out receipt.json
assembled receipt -> receipt.json  ·  content address 203d3bbf91ac…
affi[receipt.json]> verify                       # no path → uses session receipt
verdict: ACCEPT [core/v1] — all 7 stages passed
affi[receipt.json]> verfy
unknown command 'verfy'. did you mean:
  verify   (Core)
  variance (Analytics)
affi[receipt.json]> help find-blast-radius
receipt find-blast-radius  (Query)
  find downstream events/objects affected by a change
  $ affi receipt find-blast-radius <change-event> ./receipts/
affi[receipt.json]> guide search compliance
  verify-compliance (Verification)  verify against a named framework
  soc2-audit        (Compliance)    SOC2 control-mapping audit
  hipaa             (Compliance)    HIPAA safeguards proof
  pci-dss           (Compliance)    PCI-DSS change-management evidence
  gdpr-proof        (Compliance)    GDPR data-integrity proof
  sbom-compliance   (Sbom)          NTIA minimum-element compliance
affi[receipt.json]> ^D
Goodbye!
```

### 4.5 TUI dashboard sketch (`affi guide tui <receipt>`, optional, behind `ui`)

A read-only browser over a receipt/store. Three panes: event list (left), event detail (right), verdict/chain status (bottom). Built on the existing `ui` feature (`colored`, `indicatif`; add `ratatool` later).

```
┌ affidavit ─ receipt.json ───────────────────────────────── core/v1 · 2 events ┐
│ EVENTS                         │ EVENT DETAIL                                   │
│ ▸ [0] seed         art1▸input  │  seq          0                               │
│   [1] validate     art1▸output │  event_id     evt-0                           │
│                                │  event_type   seed                            │
│                                │  commitment   6ef47c82a1b3c9d4e5f6…           │
│                                │  objects      art1:artifact/input             │
│                                │                                               │
│                                │  payload      (not stored — commitment only)  │
├────────────────────────────────┴───────────────────────────────────────────────┤
│ CHAIN  203d3bbf91ac…  │  VERDICT  ✓ ACCEPT  │  7/7 stages  │  press v=verify d=diff│
│ stages: decode ✓ format ✓ chain ✓ continuity ✓ commit ✓ profile ✓ verdict ✓     │
└──────────────────────────────────────────────────────────────────────────────────┘
  ↑/↓ move event   enter expand   v verify   d diff   g graph   q quit
```

Interactions: `↑/↓` select event, `enter` expand objects/commitment, `v` re-run `cli::verify` and repaint the verdict bar (red on REJECT, highlighting the failing stage), `g` render the DFG via `handlers::graph`, `q` quit. Tampered receipts show the failing stage in red with the mismatch reason inline — the same information `golden_run.sh` proves on the command line, but navigable.

---

## 5. Integration & Rollout

### 5.1 Touched / new surface

| Item | New / Touched | Notes |
|---|---|---|
| `src/registry.rs` | **new** | single source of truth: group, summary, keywords, examples for all 67 verbs |
| `src/suggest.rs` | **new** | `edit_distance`, `nearest`, `did_you_mean` |
| `src/tutorial.rs` | **new** | `Step`/`Sandbox`/`run`; mirrors `golden_run.sh` semantics |
| `src/verbs/{tutorial,examples,search_cmd,doctor,man}.rs` | **new** | thin `#[verb("…","guide")]` wrappers, copy of `show.rs` shape |
| `src/handlers.rs` | **touched** | add `tutorial/examples/guide_search/doctor/man` handlers (append-only; existing handlers unchanged) |
| `src/bin/affi-shell.rs` | **touched** | registry-driven completion; `Session` context; full dispatch; `did_you_mean` |
| `src/lib.rs` | **touched** | `pub mod registry; pub mod suggest; pub mod tutorial;` (mirrors existing `pub mod` block, `lib.rs:64-104`) |
| `src/bin/affi.rs` | **touched** | first-run banner before `affidavit::run()` (`affi.rs:6`) |
| `Cargo.toml` | **touched** | TUI feature (`ui` exists; add `ratatui` dep) — only for the optional TUI |
| `README.md` / `docs/INDEX.md` | **touched** | point new users at `affi guide tutorial`; fix the stale "59 verbs" |
| Existing 67 verb wrappers & their handlers | **untouched** | doctrine and verdicts unchanged |

**Doctrine guardrail:** every new surface is *descriptive or pass-through*. `tutorial`/`examples`/`search`/`doctor`/`man`/TUI never adjudicate; `verify` semantics (`cli::verify` + `admission::admit`, `cli.rs:110-138`) are reused verbatim. We make verdicts *findable*, never *decided* differently.

### 5.2 Priorities & sizing (S ≤ 1d · M ≈ 2–4d · L ≈ 1–2wk)

**P0 — make a new user productive today**
- `affi guide tutorial` (state machine + 4 lifecycle steps) — **M**
- `src/registry.rs` seeded for the ~12 core verbs + grouping enum — **M**
- `affi guide examples` reading the registry — **S**
- First-run banner + `affi guide doctor` (version, `.affi/` health, feature flags) — **S**

**P1 — tame the 67-verb surface**
- Fill `registry.rs` for all 67 verbs (group + 1-line summary + ≥1 example each) — **M**
- `affi guide search` + "did you mean" (`suggest.rs`) wired into dispatch — **M**
- Grouped `affi guide help` view — **S**
- REPL: registry-driven completion + `Session` context + full dispatch — **M**

**P2 — polish & depth**
- `affi guide man` (troff + markdown) + `cargo xtask man` — **M**
- TUI dashboard behind `ui`/`ratatui` — **L**
- `linkme`-backed `#[verb_doc]` so registry entries live *next to* each `#[verb]` (kills all drift permanently) — **M**
- Shell completion scripts (bash/zsh/fish) generated from the registry — **S**

### 5.3 Success signals

- Time-to-first-ACCEPT for a new user drops from "read README + assemble a `cargo run` line" to **one command** (`affi guide tutorial`).
- 100% of 67 verbs reachable via `guide search` and documented via `guide examples` (vs. ~12 in the README today).
- REPL completion/help **provably never drift** (single registry; ideally `linkme`-generated).
- Every typo within edit-distance threshold yields a correct suggestion + a runnable example.
