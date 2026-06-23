# AGENTS.md — operational guide for coding agents

This file is the **ground truth** for any coding agent (Claude Code or otherwise)
working in this repo. Read it before you build. It exists because the obvious
first move here — `cargo build` at the root — fails, and that failure is *not
your bug*. Below is what actually compiles, how to validate each part, and the
traps to avoid.

> The longer narrative in `CLAUDE.md` describes the *intended* `affidavit`
> design. Treat it as the spec; treat **this** file as the current build state.

---

## #1 fact: the root `affidavit` crate does not compile

The root crate (`src/`, `Cargo.toml`) depends on the published crate
**`wasm4pm-compat 26.6.13`**, which **fails to compile under the current Rust
nightly** (~550 errors in that upstream crate). `affidavit` references
`wasm4pm_compat` unconditionally, so:

- `cargo build` / `cargo test` / `cargo clippy` on the root crate **cannot pass** —
  even with `--no-default-features`. Do not "fix" this; it is upstream and out of
  scope unless explicitly asked.
- The **only** root-crate gate that works is formatting:
  `cargo fmt --all -- --check` (rustfmt parses source without resolving deps).
  This is the repo's real CI gate (`.github/workflows/rust.yml`) and it is
  currently green — keep it that way (`cargo fmt --all` to fix drift).

The "missing sibling PATH-crates (`../clap-noun-verb`, …)" explanation you may
still find in some comments is **stale** — those deps resolve from crates.io now;
the real blocker is the broken upstream crate above.

---

## What actually builds — the validate matrix

| Area | Builds? | Validate with |
|---|---|---|
| **root `affidavit` crate** (`src/`) | ❌ build/test/clippy (broken dep) | ✅ `cargo fmt --all -- --check` only |
| **`affidavit-core/`** (zero-dep `no_std` verifier + process mining) | ✅ fully | `cd affidavit-core && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check` — see `affidavit-core/AGENTS.md` |
| **`web/`** (Next.js 15 / React 19 / TS, Node 22) | ✅ fully | `cd web && npm install && npx tsc --noEmit` (tsc is the gate; no ESLint config) |
| **`tools/confevo/`** (Python genetic config optimizer) | ✅ fully | `python3 -m unittest discover -s tools/confevo -p 'test_*.py'` then `python3 tools/confevo/confevo.py run --dry-run` |

**If your change is in one of the ✅ areas, run that area's full check before you
push.** Don't gate your work on the root crate compiling — it won't.

---

## Where to work (project map)

```
affidavit/
├── src/                  root `affidavit` crate — DOES NOT BUILD (broken dep). fmt-only.
├── affidavit-core/       ✅ standalone crate: zero-dep, no_std, forbid(unsafe) verifier
│                            + process-mining module. Strict invariants — read its AGENTS.md.
├── web/                  ✅ self-contained Next.js app (the reliably-installable part)
├── tools/confevo/        ✅ Python (stdlib-only) genetic Cargo-feature optimizer; has README
├── scripts/              bootstrap.sh / check.sh — local setup & the working checks
├── .claude/              SessionStart hook + settings (see below); session-local state ignored
└── .github/workflows/    rust.yml (fmt is the real gate) + web.yml
```

Net-new, self-contained work belongs in `affidavit-core/`, `web/`, or
`tools/confevo/` — places where you can actually compile and test what you wrote.

---

## SessionStart hook (Claude Code on the web)

`.claude/hooks/session-start.sh` runs automatically at the start of remote
(web) sessions (matcher `startup|resume`, so it does **not** re-run on `/clear`
or `compact`). It is **async, cache-aware, idempotent, and remote-only**
(`CLAUDE_CODE_REMOTE`). It:

1. ensures the nightly `rustfmt` + `clippy` components (skipped if present), and
2. runs `npm install` in `web/` (skipped if `node_modules` is current).

It deliberately does **not** run `cargo build/test/clippy` (they can't pass — see
#1). If you need the toolchain set up locally, run `bash scripts/bootstrap.sh`.

**Async note for agents:** because the hook is async, the session starts
immediately while setup warms in the background. On a *warm* container both steps
above are sub-second no-ops, so there is effectively no race. On a *cold* first
session, the background `npm install` may still be running — so if a `web/`
command (`npx tsc`, `npm run build`) fails with missing deps, either wait for the
`.claude/.session-ready` marker the hook writes on completion, or just re-run
`npm install` (it's idempotent — the documented fallback). Rust work
(`cargo fmt`) is unaffected (rustfmt/clippy are preinstalled here).

---

## Conventions

- **Doctrine: "certify, don't decide."** The verifier (and the process-mining
  conformance checker) certify an artifact against a format/model; they never
  decide whether the recorded work was honest. Preserve this framing in code and
  docs.
- **Branching:** develop on a feature branch; do **not** push to `main`. Commit
  with clear messages.
- **No-unwrap policy** in library code (root crate): fallible paths return
  `Result`; `.unwrap()` is for tests only.
- **Don't commit generated artifacts:** `target/`, `tools/confevo/out/`,
  `node_modules/`, `__pycache__/` are git-ignored — keep them that way.
- **Honesty over green:** a check that passes whether or not the work happened
  carries no information (this repo's stated ethos). Report failures plainly;
  don't fake a pass.

---

## Common traps (learned the hard way)

- Running `cargo build`/`test` at the root and concluding the repo is broken —
  it's the upstream dep; use the per-area checks above.
- Editing the root crate's `src/` expecting `cargo test` to confirm it — it
  can't compile. For real Rust you can build+test, work in `affidavit-core/`.
- `/usr/bin/time` is not installed here; don't rely on it in scripts.
- The web `package-lock.json` churns `libc` fields on `npm install` under this
  npm version — that diff is benign.
