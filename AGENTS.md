# AGENTS.md — operational guide for coding agents

This file is the **ground truth** for any coding agent (Claude Code or otherwise)
working in this repo. Read it before you build. It exists because the obvious
first move here — `cargo build` at the root — fails, and that failure is *not
your bug*. Below is what actually compiles, how to validate each part, and the
traps to avoid.

> The longer narrative in `CLAUDE.md` describes the *intended* `affidavit`
> design. Treat it as the spec; treat **this** file as the current build state.

---

## #1 fact: the root `affidavit` crate now builds (via local stubs)

It didn't used to. It depends on `wasm4pm`, `wasm4pm-compat`, and `clnrm-core`,
whose published versions fail under the pinned nightly (const-trait) or drag in
300+ transitive deps. Those are now replaced by local **stubs** through
`[patch.crates-io]` in `Cargo.toml` (`stubs/wasm4pm-compat`, `stubs/wasm4pm`,
`stubs/clnrm-core`). As a result:

- `cargo build --all-targets` ✅ and `cargo test` ✅ (789 tests, incl. doctests)
  pass under default features. `cargo fmt --all -- --check` ✅.
- The one remaining red gate is `cargo clippy --all-targets -- -D warnings`:
  `src/lib.rs` sets `#![deny(clippy::print_stdout)]`, yet the library still has
  ~236 raw `println!` calls (plus assorted style lints — ~270 findings total).
  This is **pre-existing lint debt, not a build break** — clearing it means
  routing output through the `Out` handle (`src/output.rs`). Treat it as a
  known, separate cleanup; CI keeps the `clippy` job non-blocking until paid down.
- If you swap the stubs back for the real upstreams, the build breaks again —
  see the `[patch.crates-io]` comment in `Cargo.toml`.

---

## What actually builds — the validate matrix

| Area | Builds? | Validate with |
|---|---|---|
| **root `affidavit` crate** (`src/`) | ✅ build + test (via stubs); ⚠️ clippy red (print_stdout debt) | `cargo build --all-targets && cargo test` + `cargo fmt --all -- --check` |
| **`affidavit-core/`** (zero-dep `no_std` verifier + process mining) | ✅ fully | `cd affidavit-core && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check` — see `affidavit-core/AGENTS.md` |
| **`web/`** (Next.js 15 / React 19 / TS, Node 22) | ✅ fully | `cd web && npm install && npx tsc --noEmit` (tsc is the gate; no ESLint config) |
| **`tools/confevo/`** (Python genetic config optimizer) | ✅ fully | `python3 -m unittest discover -s tools/confevo -t . -p 'test_*.py'` then `python3 tools/confevo/confevo.py run --dry-run` |

**If your change is in one of the ✅ areas, run that area's full check before you
push.** The root crate now builds and tests too — `cargo build --all-targets &&
cargo test` should stay green; only `clippy` is (knowingly) red.

**CI now mirrors this matrix.** Each area has its own workflow that **blocks** on
its real checks (`affidavit-core.yml`, `web.yml`, `confevo.yml`). The root crate's
`rust.yml` blocks on `cargo fmt` **and** `build-and-test` (build + test +
doctests, now that the stubs make it compile); only its `clippy` job is
non-blocking, by design, until the `print_stdout` debt is paid down. A green
check means the area's real checks actually passed, not that they were skipped.

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
└── .github/workflows/    per-area CI; each workflow gates only its own area:
                          • rust.yml           root-crate fmt (real gate) + non-blocking build
                          • affidavit-core.yml test + clippy + no_std build + fmt
                          • web.yml            tsc --noEmit + next build
                          • confevo.yml        python unittest + dry-run
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

- Assuming the root crate can't build (older docs/comments say so) — it builds
  now via the `[patch.crates-io]` stubs. `cargo build --all-targets` and
  `cargo test` pass; `cargo clippy -D warnings` is the only red (print_stdout
  debt). Don't delete the `stubs/` or the `[patch]` block to "fix" deps.
- Running `cargo clippy -D warnings` and concluding the crate is broken — it's
  ~236 `println!` violations of `#![deny(clippy::print_stdout)]`, a known lint
  debt, not a compile failure.
- `/usr/bin/time` is not installed here; don't rely on it in scripts.
- The web `package-lock.json` churns `libc` fields on `npm install` under this
  npm version — that diff is benign.
