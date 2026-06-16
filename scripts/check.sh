#!/usr/bin/env bash
#
# check.sh — run the checks that work in this repo without the sibling crates.
#
# Two groups:
#   1. Rust formatting gate: `cargo fmt --all -- --check`. Formatting needs NO
#      dependencies, so this works in a single-repo checkout. (Requires the
#      rustfmt component: `rustup component add rustfmt`.)
#   2. Web: type-check (`npx tsc --noEmit`).
#
# NOT run here: `cargo build` / `cargo clippy` / `cargo test`, which need the
# five sibling PATH crates (see CONTRIBUTING.md). Use scripts/golden.sh for
# the end-to-end smoke once the full workspace is present.
#
# Exits non-zero if any check fails. Usage:  scripts/check.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

fail=0

# --- 1. Rust formatting (no sibling crates required) -----------------------
echo "==> [1/2] cargo fmt --all -- --check"
if ! command -v cargo >/dev/null 2>&1; then
  echo "    SKIP: cargo not found (install Rust via https://rustup.rs)."
  fail=1
elif ! cargo fmt --version >/dev/null 2>&1; then
  echo "    SKIP: rustfmt component not installed."
  echo "          install it with: rustup component add rustfmt"
  fail=1
else
  if ( cd "$REPO_ROOT" && cargo fmt --all -- --check ); then
    echo "    formatting OK."
  else
    echo "    FAIL: code is not rustfmt-clean. Run 'cargo fmt --all' to fix." >&2
    fail=1
  fi
fi
echo

# --- 2. Web type-check -----------------------------------------------------
echo "==> [2/2] web type-check"
if ! command -v npm >/dev/null 2>&1; then
  echo "    SKIP: npm not found (install Node 22)."
  fail=1
else
  if [ ! -d "$REPO_ROOT/web/node_modules" ]; then
    echo "    web/node_modules missing; installing (npm ci)"
    ( cd "$REPO_ROOT/web" && npm ci )
  fi
  echo "    -> npx tsc --noEmit"
  if ( cd "$REPO_ROOT/web" && npx tsc --noEmit ); then
    echo "       type-check OK."
  else
    echo "       FAIL: TypeScript type-check failed." >&2
    fail=1
  fi
fi
echo

if [ "$fail" -ne 0 ]; then
  echo "==> check.sh: one or more checks failed or were skipped." >&2
  exit 1
fi
echo "==> check.sh: all checks passed."
