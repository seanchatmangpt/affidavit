# W4 — Onboarding, Discoverability & the Verb Registry

**Workstream:** W4 (of 10) · **Owner role:** Onboarding / Discoverability / Registry
**Binary:** `affi` (+ `affi-shell`) · **Baseline:** 26.6.17 · **Horizon:** 2026 H2 → 2030
**Builds on:** `docs/innovation/05-dx-onboarding.md`, sequenced by `docs/innovation/00-SYNTHESIS.md`
**Doctrine constraint:** *certify, don't decide.* Nothing in W4 adjudicates a receipt. The registry, guide, search, completions, and man pages are **descriptive or pass-through**; they make verdicts *findable*, never *decided differently*.

> **Build caveat.** This crate depends on private-registry `26.6` crates (`clap-noun-verb`,
> `clnrm-core`, `wasm4pm`, `lsp-max`) that do not resolve in a lone checkout, so **nothing
> here was `cargo build`/`test`-verified.** All Rust is *compilable-style* — correct against
> the in-tree patterns (the `#[verb("verb","noun")]` → `crate::handlers::*` seam), pending
> signature finalization against the sibling crates. Citations are `file:line` at this commit.

---

## 1. Mission & Scope

### 1.1 Mission

Turn `affidavit`'s enormous-but-opaque surface — **67 verbs under a single `receipt`
noun** — into a *guided, discoverable, learnable* product. The keystone is **one
source-of-truth registry** (`src/registry.rs`) describing every verb (group, summary,
keywords, examples) that feeds **every** discoverability surface: grouped help, command
search, per-verb examples, "did you mean", shell completions, man pages, and the data
that W2/W3/W6 consume. On top of it: a new **`guide` noun** (`tutorial`, `examples`,
`search`, `man`) and a multi-year learning-path program through 2030.

The bet (from 05 §1): *discoverability is a force multiplier.* 67 verbs a user cannot
find are worth ~12. Make all 67 findable, learnable, and runnable-from-example, and the
effective surface — and the product's value — rises by orders of magnitude.

### 1.2 In scope (W4 owns)

1. **`src/registry.rs`** — the single source of truth: `Group` taxonomy + `VerbDoc { noun, verb, group, summary, keywords, examples }` for all 67 verbs. *(05 §3.1)*
2. **Verb taxonomy / grouping** — a stable ~10-group classification of the 67 verbs.
3. **The `guide` noun** — `guide tutorial` (Step/state machine over the real lifecycle), `guide examples` (registry recipes), `guide search` (command discovery — distinct from today's payload-grep `search`), `guide man` (generated man pages). *(05 §3.2–3.6)*
4. **"Did you mean"** — `src/suggest.rs` (Wagner–Fischer edit distance) wired at the **dispatch boundary**: `affi-shell.rs:359` catch-all and the unknown-verb error path. *(05 §3.4)*
5. **Generated man pages + website/docs** from the registry; structured **learning paths**.
6. **Fixing the discoverability defects** (B7/B9): the duplicate throughput module and the README "59 capabilities" drift, and establishing the *true* verb count.

### 1.3 Out of scope (boundaries — referenced, not owned)

| Concern | Owner | W4 relationship |
|---|---|---|
| Output/format contract (`--format/--json`, stdout/stderr split, `Out` handle) | **W1 / W3** | W4 *consumes* it; guide narration writes through it. |
| `affi --explain <CODE>`, error-code catalog, exit-code system | **W3** | W4's `did_you_mean` plugs into W3's error surface. |
| REPL completion *using* the registry, TUI dashboard, LSP diagnostics | **W6** | W4 *provides the registry data*; W6 wires it into interactive surfaces. |
| `doctor` (env + receipt) and `fix` | **W2** | W4's registry answers "which features does this verb need" for doctor. |
| Verification engine, crypto/trust, ecosystem/standards, compliance/governance | W7/W8/W9/W10 | Out of scope; W4 only *describes* their verbs in the registry. |

W4's hard line: **W4 produces the registry; W6 consumes it for live surfaces; W1/W3 own
the bytes that come out.** When a guide surface needs to print, it goes through the W1/W3
`Out` handle — W4 never reintroduces hand-built `format!` JSON (the B2 defect).

---

## 2. Current State & Gaps (cited)

### 2.1 What exists today

- **`examples/golden_run.sh`** — the canonical lifecycle and *the* teaching artifact: emits two events (`receipt emit --type seed …` / `--type validate …`, `golden_run.sh:35-37`), `receipt assemble --out receipt.json` (`:42`), `receipt verify` → ACCEPT/exit 0 (`:48`), then `sed -i 's/"validate"/"tampered"/'` (`:62`) and verify again → REJECT/non-zero (`:69`). But it is a **shell script you read**, and it shells out via `cargo run` (`:21`).
- **`README.md`** — lists the four core verbs and a sampling of quality/sbom verbs; states *"**59 canonical verbs**"* (`README.md:77`) and *"the complete list of all **59 capabilities**, run `affi --help`"* (`README.md:101`). Both counts are wrong (see §2.3).
- **`src/bin/affi-shell.rs`** — a `rustyline` REPL with filename completion, bracket highlighting, history hinting, multi-line via trailing `\`, persisted `.affi_history`. Good bones — but a **hardcoded 16-word** completion vec (`affi-shell.rs:121-138`) and **only 11** verbs dispatched (`affi-shell.rs:229-356`); the catch-all is a bare `Err(anyhow!("Unknown command: {}", …))` (`affi-shell.rs:359`).
- **`receipt search`** (`src/verbs/search.rs:13-16` → `crate::handlers::search`) — full-text grep over **receipt payloads/event fields**, *not* command discovery (its own doc: "Full-text search over receipt payloads (grep-like)", `search.rs:12`).
- **The verb seam** — every verb is a thin wrapper carrying a `///` summary + `#[verb("verb","noun")]` + a delegating body (e.g. `verify.rs:12-21`, `find_blast_radius.rs:12-20`, `show.rs:12-16`). This uniform shape is **exactly the harvestable source** for registry generation (see §3.1).
- **`CheckOutcome`** (`types.rs:270-277`: `{ stage, passed, detail }`) — the in-tree precedent for any "finding" record W4 reuses for guide/doctor framing.

### 2.2 The gaps (the real pain)

1. **No guided tutorial.** The only tour is a non-interactive bash file (`golden_run.sh`).
2. **67 verbs, one noun, no grouping.** Every verb is `#[verb("…","receipt")]` — `grep` confirms **67** receipt verbs (§2.3). `affi receipt --help` is an undifferentiated wall (`emit`, `emit-batch`, `emit-from-github`, `dora-metrics`, `sbom-scan`, `hipaa`, `bus-factor`, `gdpr-proof`, …). No "emission" vs "verification" vs "compliance" vs "analytics" grouping exists anywhere the user can see.
3. **No command search.** "The verb that finds what a change breaks" is `find-blast-radius` (`find_blast_radius.rs:13`) — undiscoverable unless you already know the name. Today's `search` greps payloads, not commands.
4. **No "did you mean".** `affi receipt verfiy r.json` → flat clap error, no suggestion.
5. **No examples per verb.** Argument shapes live only in wrapper/handler signatures (`verify(receipt, format, profile, strict)`, `verify.rs:14-19`), invisible at the CLI.
6. **REPL is a stub.** 16 completion words, 11 dispatched verbs, **no working-chain context** (each `verify` needs a full path), and the completion list is hand-maintained and **already drifted** from the real verb set.
7. **No man pages, no first-run nudge, no learning path.**

### 2.3 Verb-count truth (resolving B7 / B9)

This is itself a discoverability defect — *we did not know how many verbs we had.* Ground
truth at this commit:

- **68** `#[verb(...)]` attributes exist across `src/verbs/` (`grep -rh '#[verb(' src/verbs | wc -l` → 68).
- **67** are filed under noun `"receipt"`; **1** under noun `"bench"` (`receipt-throughput.rs:19`).
- **But** `src/verbs/mod.rs` declares **67** `pub mod` lines, and the lone `bench` verb lives in **`receipt-throughput.rs`** — a **hyphenated filename that cannot be a Rust module path**, so it is **not declared** in `mod.rs` (only `receipt_throughput` is, `mod.rs:46`). It is therefore a **dead file** (B7).
- The compiled twin, `receipt_throughput.rs:13`, re-files the same verb under `"receipt"`.
- **Net live surface: 67 verbs, all under one `receipt` noun; the `bench` noun does not actually ship.** README's "59" (`README.md:77,101`) is stale by 8 (B9).

**The true count `affi` exposes today is 67 receipt verbs.** W4's registry makes this number
*computed, asserted in a test, and rendered* — so it can never silently drift again.

> Note: `bench profile` is cited in 05 §1 as a second noun; the in-tree reality is that the
> only non-`receipt` verb (`bench receipt-throughput`) sits in the dead hyphenated file. W4
> resolves this by deleting the dead file **or** promoting `bench` to a real, declared noun —
> a one-line decision the registry forces us to make explicitly (§3, 2026 H2).

### 2.4 Gap summary

| Capability | Today (1x) | Target (1000x) | Surface |
|---|---|---|---|
| First 5 min | read README, copy a `cargo run` line | `affi guide tutorial` runs the real loop, narrated | `guide tutorial` |
| "What can this do?" | scroll 67 flat verbs | grouped help + `guide search <topic>` | registry + `guide search` |
| Wrong command | bare clap error | "did you mean `verify`?" + 3 nearest + example | `suggest.rs` |
| Recipes | none | `guide examples verify` → runnable snippets | registry examples |
| Verb count | wrong ("59") | computed + test-asserted + rendered | registry test |
| Completions | 16 hand-typed words, drifted | generated for all 67, never drift | registry → completion data |
| Man pages | none | generated troff + markdown for all 67 | `guide man` |

---

## 3. Phased Plan (2026 H2 / 2027 / 2028 / 2029 / 2030)

Each phase lists objectives, deliverables, compilable-style sketches, acceptance criteria,
and cross-workstream dependencies. 2026 H2 is anchored to **Synthesis P0/P1**
(`00-SYNTHESIS.md:118-133`): registry is a P0 keystone (Part B.2); guide/search/did-you-mean
are P1 (`00-SYNTHESIS.md:133`).

---

### 2026 H2 — The Registry Keystone + First-Run Path *(Synthesis P0/P1)*

**Objective.** Stand up `src/registry.rs` as the one source of truth, classify all 67 verbs,
ship `guide tutorial` + `guide examples`, wire "did you mean", and **make the verb count
true** (resolve B7/B9). After this phase a brand-new user reaches their first ACCEPT in
*one command* and every typo within threshold yields a runnable suggestion.

**Deliverables**

- `src/registry.rs`: `Group` enum (taxonomy, §below) + `VerbDoc` + `Example` + a `REGISTRY` static covering **all 67 verbs** (group + 1-line summary + ≥1 example each). *(Synthesis P0: `00-SYNTHESIS.md:123`)*
- A **verb-count integrity test** asserting `registry::all().len()` == count of `#[verb]` wrappers, so drift fails CI.
- `src/suggest.rs`: `edit_distance` (Wagner–Fischer), `nearest`, `did_you_mean`. *(05 §3.4)*
- `src/tutorial.rs` + `src/verbs/tutorial.rs`: the Step/Sandbox/run state machine over the real lifecycle (mirrors `golden_run.sh` semantics in a temp dir). *(05 §3.2)*
- `src/verbs/examples.rs` + `handlers::examples`: registry-backed recipes (`05 §3.3`).
- **B7 fix:** remove the dead `src/verbs/receipt-throughput.rs` (or promote `bench` to a declared noun — decided here, not deferred).
- **B9 fix:** README "59" → the computed true count; point new users at `affi guide tutorial`.
- First-run banner (provided to W3's `affi.rs:6` entry) listing `guide tutorial` / `guide search`.

**The taxonomy (the spine of discoverability).** A stable ~10-group classification of the
67 verbs. Groups are chosen so *every* current verb lands in exactly one, and so a new user
can predict where a capability lives.

```rust
// src/registry.rs (new) — the one place verbs are described.
// Stable taxonomy consumed by help, search, examples, completions, man, and W6 surfaces.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Group {
    Core,         // emit, assemble, verify, show                                (the loop)
    Emission,     // emit-batch, emit-from-{github,gitlab,cicd,cloud,sbom,security,monitoring}
    Verification, // verify-family, verify-sla, verify-compliance, attest, notarize, sign,
                  //   assemble-and-notarize, assemble-with-signature, conformance, policy-enforce
    Display,      // inspect, diff, stats, graph, replay, visualize, catalog, timeline, model, audit
    Query,        // query, search, causality-chain, find-blast-radius, root-cause, explain-incident
    Analytics,    // dora-metrics, team-velocity, tech-debt, security-debt, variance, predict,
                  //   coverage-analysis, dependency-matrix, bus-factor, orphaned-code
    Compliance,   // hipaa, pci-dss, soc2-audit, gdpr-proof, license-compliance
    Sbom,         // sbom-scan, sbom-attest, sbom-compliance, sbom-ntia, sbom-blast-radius
    Quality,      // monitor, trend-analysis, portfolio-health, anomaly-detect
    Workflow,     // install-git-hook, receipt-throughput, diagnose
    Meta,         // guide: tutorial, examples, search, man  (the `guide` noun)
}

/// One described verb. `summary` mirrors the `///` doc on the wrapper (e.g. verify.rs:12),
/// so the registry and the source never disagree.
pub struct VerbDoc {
    pub noun: &'static str,                 // "receipt" | "guide"
    pub verb: &'static str,                 // "find-blast-radius"
    pub group: Group,
    pub summary: &'static str,              // one line, == the wrapper's /// doc
    pub keywords: &'static [&'static str],  // search synonyms: ["tamper","downstream","impact"]
    pub examples: &'static [Example],
    pub needs_features: &'static [&'static str], // e.g. ["discovery"] — answers W2 doctor & W6
}

pub struct Example {
    pub title: &'static str,
    pub cmd:   &'static str,                // copy-pasteable: "affi receipt verify receipt.json"
    pub note:  &'static str,                // optional teaching note (exit codes, gotchas)
}

pub fn all() -> &'static [VerbDoc] { &REGISTRY }
pub fn by_group(g: Group) -> impl Iterator<Item = &'static VerbDoc> {
    REGISTRY.iter().filter(move |d| d.group == g)
}
pub fn find(noun: &str, verb: &str) -> Option<&'static VerbDoc> {
    REGISTRY.iter().find(|d| d.noun == noun && d.verb == verb)
}

// Seed entry — the same shape ×67. `summary` is copied verbatim from verify.rs:12.
static REGISTRY: &[VerbDoc] = &[
    VerbDoc {
        noun: "receipt", verb: "verify", group: Group::Core,
        summary: "Run the certify pipeline over a receipt and print the verdict",
        keywords: &["certify", "verdict", "accept", "reject", "check"],
        examples: &[
            Example { title: "Verify an assembled receipt (ACCEPT → exit 0)",
                      cmd: "affi receipt verify receipt.json", note: "" },
            Example { title: "Machine-readable verdict for CI gating",
                      cmd: "affi receipt verify receipt.json --format json",
                      note: "exit 0 on ACCEPT, 2 on REJECT — gate your pipeline on it." },
        ],
        needs_features: &[],
    },
    // … 66 more, one per #[verb] wrapper …
];
```

**Verb-count integrity (drift can never return).**

```rust
// tests/registry_truth.rs (new, W4-owned)
#[test]
fn registry_covers_every_verb_exactly_once() {
    // WIRES_VERBS is generated from the #[verb] wrappers (build script or const list).
    // The point: registry.len() must equal the real verb count, and README must match.
    let n = affidavit::registry::all().len();
    assert_eq!(n, 67, "registry drifted from the live verb surface");
    // No (noun, verb) appears twice.
    let mut seen = std::collections::HashSet::new();
    for d in affidavit::registry::all() {
        assert!(seen.insert((d.noun, d.verb)), "duplicate verb: {} {}", d.noun, d.verb);
    }
}
```

**"Did you mean" (Wagner–Fischer), wired at the dispatch boundary.**

```rust
// src/suggest.rs (new)
/// Classic bounded edit distance (Wagner–Fischer); ample at 67 verbs.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

pub struct Suggestion { pub verb: &'static str, pub group: crate::registry::Group, pub score: usize }

/// Rank verbs by closeness. Substring/keyword hits sort first (score 0).
pub fn nearest(input: &str, limit: usize) -> Vec<Suggestion> {
    let input = input.to_lowercase();
    let mut ranked: Vec<Suggestion> = crate::registry::all().iter().map(|d| {
        let hit = d.verb.contains(&input) || d.keywords.iter().any(|k| k.contains(&input));
        let score = if hit { 0 } else { edit_distance(&input, d.verb) };
        Suggestion { verb: d.verb, group: d.group, score }
    }).collect();
    ranked.sort_by_key(|s| s.score);
    ranked.into_iter().take(limit).collect()
}

/// Error-path helper: only suggest when the typo is *close* (threshold scales with length).
pub fn did_you_mean(unknown: &str) -> Option<Vec<Suggestion>> {
    let near = nearest(unknown, 3);
    let best = near.first()?;
    (best.score <= (unknown.len() / 3).max(2)).then_some(near)
}
```

> Wiring point (W4 provides, W6/W3 host): the catch-all at `affi-shell.rs:359`
> (`_ => Err(anyhow!("Unknown command: {}", args[0]))`) calls `suggest::did_you_mean`; where
> `clap-noun-verb` surfaces an unknown-subcommand error, the same helper feeds W3's error
> surface. W4 owns the suggester; the *host* of the error string is W3/W6.

**Tutorial Step machine (live, narrated `golden_run.sh`).**

```rust
// src/tutorial.rs (new) — deterministic Step machine over a sandbox dir.
use anyhow::Result;
use std::path::PathBuf;

/// One teaching beat: narration + a real action against an isolated scratch dir.
pub struct Step {
    pub title: &'static str,
    pub teach: &'static str,          // the "why", printed before running
    pub command_hint: &'static str,   // the equivalent CLI the user could type
    pub run: fn(&Sandbox) -> Result<StepOutcome>,
    pub expect: Expect,               // what success looks like (drives narration)
}
pub enum Expect { Accept, Reject, Info }
pub struct StepOutcome { pub summary: String, pub exit_code: i32 }

/// Isolated scratch dir; auto-removed on drop (mirrors golden_run.sh:24's trap).
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
pub struct RunOpts { pub auto: bool, pub color: bool } // --auto/--yes for CI

/// The script — same four beats as golden_run.sh (:35-69), each one teachable.
pub fn lifecycle_steps() -> Vec<Step> {
    vec![
        Step { title: "Emit your first operation-event",
            teach: "A receipt is a BLAKE3 chain of events. `emit` appends ONE event to the \
                    working receipt (.affi/working.json). The payload is hashed; the digest \
                    becomes the commitment — the raw bytes are never stored.",
            command_hint: "affi receipt emit --type seed --object art1:artifact:input --payload a.txt",
            run: |sb| {
                std::fs::write(sb.path("a.txt"), b"source bytes for stage one\n")?;
                // Delegate to the SAME handler the CLI uses (handlers::emit), never a mock.
                crate::handlers::emit("seed".into(),
                    vec!["art1:artifact:input".into()],
                    sb.path("a.txt").to_string_lossy().into(), None)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                Ok(StepOutcome { summary: "event emitted (seq 0)".into(), exit_code: 0 })
            },
            expect: Expect::Info },
        Step { title: "Seal the chain (assemble)",
            teach: "`assemble` folds every event into a rolling chain hash and writes an \
                    immutable, content-addressed receipt. Editing any event later changes \
                    this hash — that is what makes tampering detectable.",
            command_hint: "affi receipt assemble --out receipt.json",
            run: |sb| {
                crate::handlers::assemble(None, Some(sb.path("receipt.json").to_string_lossy().into()))
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                Ok(StepOutcome { summary: "receipt assembled".into(), exit_code: 0 })
            },
            expect: Expect::Info },
        Step { title: "Verify an honest receipt → ACCEPT",
            teach: "The 7-stage certify pipeline recomputes the chain and adjudicates. An \
                    untouched receipt passes all stages: verdict ACCEPT, exit 0.",
            command_hint: "affi receipt verify receipt.json",
            run: |sb| {
                crate::handlers::verify(sb.path("receipt.json").to_string_lossy().into(),
                    None, None, None).map_err(|e| anyhow::anyhow!("{e}"))?;
                Ok(StepOutcome { summary: "ACCEPT — all 7 stages passed".into(), exit_code: 0 })
            },
            expect: Expect::Accept },
        Step { title: "Tamper, then verify → REJECT",
            teach: "Now we flip one event_type on disk (validate → tampered), exactly like \
                    golden_run.sh:62. The recomputed chain hash no longer matches the stored \
                    one, so chain_integrity fails: verdict REJECT, non-zero exit.",
            command_hint: "sed -i 's/\"validate\"/\"tampered\"/' receipt.json && affi receipt verify receipt.json",
            run: |sb| {
                let p = sb.path("receipt.json");
                let s = std::fs::read_to_string(&p)?.replace("\"validate\"", "\"tampered\"");
                std::fs::write(&p, s)?;
                let code = match crate::handlers::verify(p.to_string_lossy().into(), None, None, None) {
                    Ok(()) => 0, Err(_) => 2,   // REJECT surfaces as the verifier's non-zero exit
                };
                Ok(StepOutcome { summary: "REJECT — chain hash mismatch at chain_integrity".into(),
                                 exit_code: code })
            },
            expect: Expect::Reject },
    ]
}

/// Runner: narrate → pause (unless --auto) → run → confirm expectation.
pub fn run(opts: RunOpts) -> Result<()> {
    let sb = Sandbox::new()?;
    let steps = lifecycle_steps();
    for (i, step) in steps.iter().enumerate() {
        // banner + wrapped `teach` + `$ command_hint` go through the W1/W3 Out handle.
        if !opts.auto { /* prompt [enter]/q via W3 output handle */ }
        let outcome = (step.run)(&sb)?;
        let ok = match step.expect {
            Expect::Accept => outcome.exit_code == 0,
            Expect::Reject => outcome.exit_code != 0,
            Expect::Info   => true,
        };
        if !ok { anyhow::bail!("tutorial step {} did not behave as taught", i + 1); }
    }
    // sandbox auto-cleans on drop
    Ok(())
}
```

**The wrapper (identical shape to `show.rs:12-16` / `verify.rs:13-21`):**

```rust
// src/verbs/tutorial.rs (new) — thin #[verb] under the new `guide` noun.
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Interactive, narrated walkthrough of the provenance lifecycle
#[verb("tutorial", "guide")]
pub fn tutorial(auto: Option<bool>, color: Option<String>) -> Result<()> {
    crate::handlers::tutorial(auto.unwrap_or(false), color) // delegate, per house pattern
}
```

**Acceptance criteria (2026 H2)**

- `affi guide tutorial` runs the real emit→assemble→ACCEPT→tamper→REJECT loop in a temp dir; `--auto` runs non-interactively and is a smoke test in CI.
- `registry::all().len()` == live `#[verb]` count, asserted by `tests/registry_truth.rs`; CI fails if README's number or the registry drifts.
- Every typo within threshold (`affi receipt verfiy`, `affi receipt blast-radius`) yields ≥1 correct suggestion with group + a runnable example.
- `affi guide examples verify` prints ≥2 copy-pasteable recipes sourced from the registry.
- The dead `receipt-throughput.rs` is gone (B7) and README no longer says "59" (B9).
- **Doctrine:** the tutorial calls `handlers::verify` *verbatim*; it reports the verdict, never mints one.

**Cross-workstream deps (2026 H2)**

- **W1/W3 (output contract):** guide narration prints through the `Out` handle; W4 must not hand-format JSON (avoids re-introducing B2). Hard dependency for tutorial output.
- **W3 (error surface):** `did_you_mean` feeds W3's unknown-verb message + `--explain` cross-links.
- **W2 (doctor):** registry's `needs_features` answers "which features does this verb need."
- **W6 (REPL/TUI/LSP):** *provides* `registry::all()` so W6 can replace the 16-word hardcoded list (`affi-shell.rs:121-138`). Soft — W6 starts once the registry static lands.

---

### 2027 — Generated Surfaces: Completions, Man Pages, Grouped Help, `guide search`

**Objective.** Make the registry *generate* every static discoverability artifact so none is
hand-maintained. Ship `guide man`, generated shell completions for **all 67** verbs, grouped
help, and the real `guide search` (command discovery, finally distinct from payload-`search`).

**Deliverables**

- `src/verbs/man.rs` + `handlers::man`: walk `registry::all()`, emit **troff** (`affi-receipt-verify.1`) and **Markdown** per verb (synopsis from the wrapper signature, summary, examples, group). *(05 §3.6)*
- `xtask`/build step: `cargo xtask man` → `target/man/*.1`; `cargo xtask completions` → `completions/{bash,zsh,fish,pwsh}` for all 67 verbs (replaces the stale `completions/*`, fixing B8 — W4 supplies the registry data; W3 owns the generator template).
- `src/verbs/search_cmd.rs` + `handlers::guide_search`: `nearest(topic, 10)`, grouped output with each verb's summary + first example.
- **Grouped help view** (`affi guide help` / `affi help`): renders `registry::all()` *by group*, with a teaching header pointing at `tutorial`/`examples`. The framework's exhaustive `--help` stays as fallback.
- A generated **website/docs** page (mdBook or static): one section per group, one card per verb, examples inlined — built from the registry in CI so docs never drift.

```rust
// handlers::man — single source, two renderers (troff + markdown).
pub fn man(format: Option<String>, verb: Option<String>) -> Result<()> {
    let docs = crate::registry::all().iter()
        .filter(|d| verb.as_deref().map_or(true, |v| d.verb == v));
    match format.as_deref() {
        Some("md") => for d in docs { /* render Markdown via Out handle */ },
        _          => for d in docs { /* render troff .TH/.SH NAME/SYNOPSIS/EXAMPLES */ },
    }
    Ok(())
}
```

**Acceptance criteria (2027)**

- `affi guide man --format md verify` and troff output both render group, synopsis, summary, examples for any verb; `cargo xtask man` writes one `.1` per verb.
- Generated completions cover **all 67** verbs across bash/zsh/fish/pwsh and are regenerated in CI (no committed drift; B8 closed).
- `affi guide search tamper` returns `find-blast-radius`, `verify`, etc. with summaries — *command* discovery, provably distinct from `receipt search` (payload grep, `search.rs:12`).
- Grouped help shows all 10 groups with every verb under exactly one; a CI check fails if any verb is ungrouped.

**Cross-workstream deps (2027)**

- **W3** owns the completion/help *template* and `clap-noun-verb` help-hook wiring; W4 supplies the registry data and the `guide` rendering. Shared.
- **W6** consumes the same registry for REPL completion + inline `help <verb>` / `examples <verb>` (REPL help == CLI help). W4 provides; W6 hosts.
- **W9 (ecosystem/standards):** generated man pages + website become the published doc surface; W4 hands W9 the registry-driven build.

---

### 2028 — Zero-Drift Registry & Structured Learning Paths

**Objective.** Eliminate registry maintenance permanently by colocating metadata with each
`#[verb]`, and graduate from a single tutorial to **role-based learning paths** (multi-step
curricula) for distinct personas.

**Deliverables**

- `linkme`-backed `#[verb_doc]` so each verb's `VerbDoc` lives **next to** its `#[verb]` wrapper, collected into the registry at link time. `linkme` is already a declared-but-unused dep (`Cargo.toml:42`, per `00-SYNTHESIS.md:54`) — this is its second idiomatic use after W2's doctor checks. Kills drift at the root: adding a verb *forces* a doc entry or the build is incomplete.
- **Learning paths**: a `Path` over ordered `Step`/lessons that thread multiple verbs (e.g. "Compliance officer": `emit-from-cicd` → `verify-compliance` → `soc2-audit` → `attest`). `guide tutorial --path compliance`.
- Path **progress persistence** (a sentinel beside `.affi/working.json`) so a user resumes mid-curriculum.
- "Next step" nudges: after any verb, optionally surface the next lesson in the active path (opt-in, doctrine-safe — suggestion only).

```rust
// src/registry.rs — colocated metadata, collected by linkme (zero-drift).
#[linkme::distributed_slice]
pub static VERB_DOCS: [fn() -> VerbDoc] = [..];

// next to src/verbs/verify.rs's #[verb]:
#[linkme::distributed_slice(crate::registry::VERB_DOCS)]
static VERIFY_DOC: fn() -> VerbDoc = || VerbDoc { /* group, summary, keywords, examples */ };

// src/guide/path.rs — multi-verb curriculum.
pub struct Lesson { pub verb: &'static str, pub why: &'static str, pub try_cmd: &'static str }
pub struct Path { pub id: &'static str, pub audience: &'static str, pub lessons: &'static [Lesson] }

pub static PATHS: &[Path] = &[
    Path { id: "compliance", audience: "Compliance / audit",
        lessons: &[
            Lesson { verb: "emit-from-cicd",     why: "capture the build as evidence",
                     try_cmd: "affi receipt emit-from-cicd …" },
            Lesson { verb: "verify-compliance",  why: "check against a named framework",
                     try_cmd: "affi receipt verify-compliance receipt.json --framework soc2" },
            Lesson { verb: "soc2-audit",         why: "SOC2 control-mapping audit",
                     try_cmd: "affi receipt soc2-audit receipt.json" },
            Lesson { verb: "attest",             why: "bind an attestation to the chain",
                     try_cmd: "affi receipt attest receipt.json --type soc2" },
        ] },
    // "platform-engineer", "sbom-owner", "incident-responder" …
];
```

**Acceptance criteria (2028)**

- Adding a new `#[verb]` without a colocated `#[verb_doc]` fails the integrity test — drift is structurally impossible.
- ≥4 learning paths ship (compliance, platform, SBOM, incident); `affi guide tutorial --path <id>` runs each end-to-end in a sandbox.
- Path progress persists and resumes across invocations.
- **Doctrine:** "next step" nudges suggest commands only; no path step adjudicates.

**Cross-workstream deps (2028)**

- **W2** shares the `linkme` collection pattern (doctor checks + verb docs use the same mechanism).
- **W5 (workflow automation):** learning paths reference W5's `init`/hooks (e.g. an "onboarding" path that runs `affi init`). W4 references; W5 owns those verbs.
- **W6:** REPL surfaces "next lesson" hints from the active path via the registry.

---

### 2029 — Adaptive Discovery & Telemetry-Informed Help

**Objective.** Make discoverability *adaptive*: surface the right verb/example at the right
moment, informed by **local, opt-in** usage signals — never phoning home, always doctrine-safe.

**Deliverables**

- Local usage ledger (opt-in, `.affi/` scoped): which verbs a user runs, which errors they hit, which suggestions they accept — used only to reorder *local* suggestions and propose unexplored verbs.
- **Context-aware "did you mean"**: rank suggestions by `(edit_distance, local_frequency, group_affinity)` so a Compliance-path user typing `verfy` gets `verify` and `verify-compliance` first.
- **`guide search` query expansion**: synonym/keyword graph over `VerbDoc.keywords` so "impact"/"downstream"/"breakage" all resolve to `find-blast-radius`.
- **"You haven't tried"** nudges: after N sessions, suggest a high-value unexplored verb relevant to the user's groups.
- Localization scaffold: registry `summary`/`examples` become keyable for translated man/help (W4 supplies keys; actual translation is community/W9).

```rust
// src/suggest.rs — adaptive ranking on top of the 2026 Wagner–Fischer core.
pub struct Signals<'a> { pub local_freq: &'a dyn Fn(&str) -> u32, pub active_group: Option<crate::registry::Group> }

pub fn nearest_adaptive(input: &str, limit: usize, sig: &Signals) -> Vec<Suggestion> {
    let mut v = nearest(input, limit * 4);
    v.sort_by(|a, b| {
        // primary: edit distance; tie-break: local frequency desc, then group affinity.
        a.score.cmp(&b.score)
            .then((sig.local_freq)(b.verb).cmp(&(sig.local_freq)(a.verb)))
            .then(group_affinity(b, sig).cmp(&group_affinity(a, sig)))
    });
    v.truncate(limit);
    v
}
fn group_affinity(s: &Suggestion, sig: &Signals) -> u8 { u8::from(sig.active_group == Some(s.group)) }
```

**Acceptance criteria (2029)**

- Adaptive ranking is **fully local and opt-in**; with telemetry off, behaviour is identical to the 2026 deterministic suggester (verifiable by a flag-off test).
- Synonym expansion: ≥3 distinct queries resolve to each non-obvious verb (`find-blast-radius`, `root-cause`, `bus-factor`).
- A "you haven't tried" nudge fires at most once per session and only for verbs in the user's active groups.
- **Doctrine:** every adaptive surface is suggestion-only; ranking changes *what is shown*, never *what verifies*.

**Cross-workstream deps (2029)**

- **W1/W3:** local usage ledger uses the shared config/telemetry opt-in plumbing (no W4-private store).
- **W6:** the TUI/REPL render adaptive suggestions and the unexplored-verb nudges.
- **W10 (governance):** the opt-in usage ledger is governed by W10's privacy/consent policy; W4 must conform.

---

### 2030 — Discoverability as a Platform Guarantee

**Objective.** Lock discoverability in as an *invariant*, not a feature: a machine-readable
registry contract other tools build on, with CI gates guaranteeing every verb is grouped,
documented, exampled, completion-covered, and man-page'd — forever.

**Deliverables**

- **`affi guide registry --json`**: emit the full registry as a stable, versioned schema (groups, verbs, summaries, keywords, examples, `needs_features`) for IDEs, doc sites, and downstream tooling.
- **Discoverability CI gate** (W4-owned, runs in the W10 governance pipeline): fails the build if any verb lacks a group, a summary, ≥1 example, a completion entry, or a man page — turning "67 verbs, one noun, no grouping" into an *impossible* regression.
- **Stable registry-schema versioning** (`registry/v1`) so external consumers pin a contract; breaking changes are gated like the `core/v1` receipt format.
- **Onboarding analytics dashboard** (opt-in, aggregate): time-to-first-ACCEPT, path completion rates, suggestion-acceptance — to *measure* the 1000x claim, not just assert it.
- A maintained, generated **doc website** as the canonical reference, rebuilt from the registry on every release.

```rust
// handlers::guide_registry — the machine-readable contract.
pub fn guide_registry(format: Option<String>) -> Result<()> {
    // Serialize registry::all() as registry/v1 JSON via the W1/W3 Out handle.
    // Stable field order; schema_version: "registry/v1". Other tools pin this.
    Ok(())
}
```

```rust
// tests/discoverability_gate.rs (W4-owned, governance-run) — the 2030 invariant.
#[test]
fn every_verb_is_fully_discoverable() {
    for d in affidavit::registry::all() {
        assert!(!d.summary.is_empty(),           "{} {} has no summary", d.noun, d.verb);
        assert!(!d.examples.is_empty(),          "{} {} has no example", d.noun, d.verb);
        // group is non-optional by type; completion + man coverage checked by xtask in CI.
    }
}
```

**Acceptance criteria (2030)**

- `affi guide registry --json` emits `registry/v1`; a downstream consumer (e.g. an IDE plugin or the doc site) builds solely from it.
- The discoverability CI gate is required-to-merge: no verb can ship ungrouped/undocumented/unexampled/uncompleted/un-manned.
- Onboarding metrics show time-to-first-ACCEPT is a single command for a new user (vs. "read README + assemble a `cargo run` line" at baseline).
- **Doctrine:** the registry contract and gate are purely descriptive; they certify *documentation completeness*, never receipt honesty.

**Cross-workstream deps (2030)**

- **W10:** the discoverability gate runs inside W10's governance/compliance CI; registry-schema versioning follows W10's change-control.
- **W9:** `registry/v1` JSON is the ecosystem integration point (IDEs, doc generators, third-party tools).
- **W3:** `registry --json` emits through the W1/W3 output contract; no bespoke serialization.

---

## 4. Definition of Done @ 2030

W4 is done when **discoverability is an invariant**, not an aspiration:

1. **One source of truth.** `src/registry.rs` describes all verbs (group, summary, keywords, examples, `needs_features`); metadata is `linkme`-colocated with each `#[verb]`, so it cannot drift. The live verb count is computed and test-asserted (no repeat of the "59 vs 67" defect; B7/B9 permanently closed).
2. **Every surface is registry-fed.** Grouped help, `guide search`, `guide examples`, `guide man`, shell completions (all 67, 4 shells), the doc website, and W6's REPL/TUI/LSP completion all read the *same* registry. No hand-maintained verb list survives anywhere (the `affi-shell.rs:121-138` list is gone).
3. **A new user reaches ACCEPT in one command.** `affi guide tutorial` runs the real lifecycle; role-based learning paths carry users from novice to mastery; adaptive, local-only suggestions and a "did you mean" make the 67-verb surface navigable.
4. **A machine-readable contract.** `affi guide registry --json` (`registry/v1`) is consumed by external tooling; a required CI gate guarantees every verb stays grouped, documented, exampled, completion-covered, and man-page'd.
5. **Doctrine held throughout.** Every W4 surface is descriptive or pass-through. `verify` semantics are reused verbatim; W4 makes verdicts *findable*, never *decided differently*.

**Measured outcome:** 100% of verbs reachable via `guide search` and documented via
`guide examples` (vs ~12 in the README at baseline); completion/help **provably never drift**;
time-to-first-ACCEPT is one command.

---

## 5. Cross-Workstream Dependencies — What the Registry Provides

The registry is the W4 keystone *because other workstreams consume it.* This is the contract
W4 commits to and the matrix of who depends on what.

### 5.1 What W4 provides (the registry API surface)

| Provided artifact | API | Consumers |
|---|---|---|
| Per-verb metadata | `registry::all() -> &[VerbDoc]` | W2, W3, W6, W9 |
| Group lookup | `registry::by_group(Group)` | W3 (grouped help), W6 (TUI sections) |
| Exact lookup | `registry::find(noun, verb)` | W6 (REPL dispatch table), W3 (`--explain` cross-links) |
| Feature requirements | `VerbDoc.needs_features` | **W2** (doctor: "this verb needs `--features discovery`") |
| Examples | `VerbDoc.examples` | W3 (help), W6 (REPL `examples <verb>`), W9 (docs) |
| Suggester | `suggest::did_you_mean`, `nearest` | **W3** (error surface), **W6** (REPL catch-all `affi-shell.rs:359`) |
| Machine contract | `guide registry --json` (`registry/v1`, 2030) | **W9** (IDEs/doc tools), W10 (governance) |

### 5.2 Dependency matrix

| W4 provides → | W2 Doctor | W3 CLI/Contract | W6 Interactive |
|---|---|---|---|
| `VerbDoc.needs_features` | feature-health checks ("`stats` needs `discovery`", per `handlers.rs:851-858` precedent) | — | gating hints in TUI |
| `registry::all()` | — | grouped `--help`, completion data | replaces 16-word list (`affi-shell.rs:121-138`); full 67-verb dispatch (vs 11, `:229-356`) |
| `suggest::did_you_mean` | — | unknown-verb error + `--explain` link | REPL catch-all suggestion (`affi-shell.rs:359`) |
| `VerbDoc.examples` | remediation snippets | `guide examples`, man synopsis | REPL `examples <verb>`, TUI detail pane |
| `registry/v1` JSON | — | — | LSP completion/hover source |

### 5.3 What W4 depends on (so W4 stays in its lane)

- **W1/W3 — output/format contract (`Out`, `--format/--json`, stdout/stderr split).** Every guide surface prints through it; W4 never hand-builds JSON (would re-introduce B2, `handlers.rs:157,303,324`). **Hard dependency** for all guide output.
- **W3 — error-code/exit-code catalog + `--explain`.** `did_you_mean` is *hosted by* W3's error surface; W4 supplies the suggestions, W3 owns the wire format.
- **W6 — interactive surfaces.** W4 does **not** modify the REPL/TUI/LSP; it hands W6 the registry + suggester and W6 wires them into live completion, the `Session` working-chain context, and LSP diagnostics.
- **W2 — doctor.** W4's `needs_features` is consumed by W2's feature-health checks; W4 does not build doctor itself.
- **W10 — governance.** W4's local usage ledger (2029) and discoverability gate (2030) run under W10's privacy/consent and change-control policy.

### 5.4 Boundary restatement

W4 **owns the description of the surface** (registry, taxonomy, guide noun, suggester, man
pages, learning paths, docs site) and the **CI guarantee** that the surface stays described.
W4 **does not own** the bytes that come out (W1/W3), the interactive surfaces that render the
registry (W6), the doctor that consumes `needs_features` (W2), or the verification/crypto/
compliance verbs it merely *catalogs* (W7/W8/W9/W10). The registry is the seam between them.

---

*W4 roadmap · grounded at commit-time against `src/verbs/` (67 receipt verbs + 1 dead `bench`
file), `src/bin/affi-shell.rs`, `README.md`, `examples/golden_run.sh`, and the
`docs/innovation/` synthesis. Compilable-style only; not `cargo`-verified (private `26.6` deps).*
