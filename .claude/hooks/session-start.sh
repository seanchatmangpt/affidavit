#!/usr/bin/env bash
#
# session-start.sh — Claude Code on the web SessionStart hook for affidavit.
#
# DESIGN: async + cache-aware ("phase shift" to instant session start)
# --------------------------------------------------------------------
# Best practices applied (see https://code.claude.com/docs/en/claude-code-on-the-web):
#
#  * ASYNC: the first stdout line is the control JSON `{"async": true, ...}`, so
#    the session becomes usable IMMEDIATELY while setup warms in the background,
#    instead of blocking on it. (Revert to synchronous by deleting that one echo.)
#  * CACHE-AWARE / IDEMPOTENT: on a warm container (the common case — containers
#    cache for ~days) the body is a sub-second no-op, so the async race window is
#    effectively zero. Work is only done on a cold start or when inputs changed.
#  * REMOTE-ONLY: guarded on CLAUDE_CODE_REMOTE so it is a no-op locally.
#  * READINESS MARKER: writes .claude/.session-ready on completion. On a *cold*
#    start, a web command (npx tsc / npm run build) could in principle run before
#    the background npm install finishes; if so, either wait for that marker or
#    just re-run `npm install` (it is idempotent — that is the documented fallback).
#
# Trade-off (documented): async trades a tiny cold-start race risk for instant
# startup. The matcher in settings.json limits this hook to `startup|resume`, so
# it never re-runs on /clear or /compact.
#
# WHAT IT SETS UP (and the one thing it deliberately does NOT):
#  1. nightly rustfmt + clippy components — the gate `cargo fmt --all -- --check`.
#  2. web/ dependencies (`npm install`, cache-aware).
#  NOT cargo build/test/clippy: affidavit hard-depends on wasm4pm-compat 26.6.13,
#  which does not compile under current nightly (~550 errors). Running them would
#  only make startup fail. See AGENTS.md.

set -uo pipefail

# --- 0. async control line (MUST be the very first stdout line) -------------
# 600000 ms = 10 min ceiling for the background task (generous for a cold
# `npm install` + component fetch). All human-readable logs below go to stderr
# so stdout carries only this control JSON.
echo '{"async": true, "asyncTimeout": 600000}'

# --- 1. remote-only guard ---------------------------------------------------
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  echo "session-start: not a remote session (CLAUDE_CODE_REMOTE != true) — skipping." >&2
  exit 0
fi

REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
MARKER="$REPO_ROOT/.claude/.session-ready"
rm -f "$MARKER" 2>/dev/null || true

log() { printf '\n\033[1;34m==> session-start:\033[0m %s\n' "$*" >&2; }

# --- 2. Rust formatting toolchain (idempotent; skip if already present) ------
if command -v rustup >/dev/null 2>&1; then
  if rustup component list --toolchain nightly --installed 2>/dev/null | grep -q '^rustfmt'; then
    log "nightly rustfmt + clippy already present — skipping"
  else
    log "adding nightly rustfmt + clippy components"
    rustup component add --toolchain nightly rustfmt clippy >&2 \
      || log "component add failed (cargo fmt may be unavailable)"
  fi
else
  log "rustup not found — skipping component install"
fi

# --- 3. Web deps — cache-aware: install only when missing or lockfile changed -
WEB="$REPO_ROOT/web"
if [ -d "$WEB" ]; then
  if [ ! -d "$WEB/node_modules" ] \
     || [ "$WEB/package-lock.json" -nt "$WEB/node_modules/.package-lock.json" ]; then
    log "installing web/ dependencies (npm install)"
    ( cd "$WEB" && npm install ) >&2 || log "npm install failed"
  else
    log "web/ dependencies already current — skipping npm install"
  fi
else
  log "no web/ directory found — skipping npm install"
fi

# --- 4. readiness marker ----------------------------------------------------
{ date -u +"%Y-%m-%dT%H:%M:%SZ" >"$MARKER"; } 2>/dev/null || : >"$MARKER" 2>/dev/null || true
log "ready (marker: .claude/.session-ready)."
log "  Rust : cargo fmt --all -- --check     (the CI gate)"
log "  Web  : cd web && npx tsc --noEmit      (type-check)"
exit 0
