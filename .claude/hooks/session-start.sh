#!/usr/bin/env bash
#
# session-start.sh — Claude Code on the web SessionStart hook for affidavit.
#
# WHAT THIS SETS UP (and WHY only this)
# -------------------------------------
# A lone checkout of this repo has exactly two things that can be set up and
# checked, and one big thing that CANNOT — this hook is honest about all three:
#
#   WORKS  ok  Rust formatting gate:  `cargo fmt --all -- --check`
#              rustfmt only parses affidavit's own source; it never compiles the
#              dependency graph, so it works with zero external crates. This is
#              the repo's real CI gate (.github/workflows/rust.yml).
#
#   WORKS  ok  Web app (self-contained Next.js, Node 22):
#              `npm install`, then `npx tsc --noEmit` / `npm run build`.
#
#   BROKEN no  `cargo build` / `cargo test` / `cargo clippy`:
#              affidavit hard-depends on the published crate `wasm4pm-compat`
#              (=26.6.13 from crates.io), and THAT crate does not compile under
#              current Rust nightly (~550 errors). No amount of environment
#              setup makes `cargo build`/`test`/`clippy` succeed here, so this
#              hook deliberately does NOT run them — doing so would only make
#              session start fail. (Note: the older "missing sibling path-crate"
#              explanation in CONTRIBUTING.md / the CI workflow is stale; the
#              deps now resolve from crates.io — the blocker is the broken
#              upstream crate above.)
#
# So the hook does the minimum that genuinely helps:
#   1. ensure the nightly rustfmt + clippy components are present (the fmt gate)
#   2. install the web/ dependencies (`npm install` — cached in the container)
#
# Properties: synchronous, idempotent, non-interactive, safe to re-run.

set -euo pipefail

# Only do real work in the remote (Claude Code on the web) environment. Locally
# you already manage your own toolchains; running this there would just be noise.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  echo "session-start: not a remote session (CLAUDE_CODE_REMOTE != true) — skipping."
  exit 0
fi

# Resolve the repo root: prefer the harness-provided var, fall back to this
# script's location so the hook also works when invoked directly for testing.
REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

say() { printf '\n\033[1;34m==> session-start:\033[0m %s\n' "$*"; }

# --- 1. Rust formatting toolchain (the real, working gate) -----------------
# rust-toolchain.toml pins the nightly channel with rustfmt + clippy, so cargo
# selects nightly automatically inside this repo. Add the components explicitly
# anyway: it is a no-op when they are already installed, and it guarantees
# `cargo fmt --all -- --check` is available.
if command -v rustup >/dev/null 2>&1; then
  say "Ensuring nightly rustfmt + clippy components"
  rustup component add --toolchain nightly rustfmt clippy
else
  say "rustup not found — skipping component install (cargo fmt may be unavailable)"
fi

# --- 2. Web app dependencies (self-contained, always installable) ----------
# Prefer `npm install` over `npm ci`: it reuses the container's cached
# node_modules across sessions, which is the faster path here.
if [ -d "$REPO_ROOT/web" ]; then
  say "Installing web/ dependencies (npm install)"
  ( cd "$REPO_ROOT/web" && npm install )
else
  say "no web/ directory found — skipping npm install"
fi

say "ready. Working gates:"
say "  Rust : cargo fmt --all -- --check        (formatting — the CI gate)"
say "  Web  : cd web && npx tsc --noEmit         (type-check)"
say "Note: cargo build/test/clippy are NOT runnable in a lone checkout"
say "      (broken upstream crate wasm4pm-compat 26.6.13)."
