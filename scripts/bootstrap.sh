#!/usr/bin/env bash
#
# bootstrap.sh — set up a local affidavit dev environment.
#
# What it does (in order):
#   1. Sanity-check the toolchains (cargo, rustup nightly, node 22, npm).
#   2. Install the web/ app dependencies (`npm ci`) — the reliably-installable
#      part of this repo.
#   3. Skip the Rust build with a clear explanation.
#
# HONEST NOTE: the sibling crates (clap-noun-verb, wasm4pm, wasm4pm-compat,
# lsp-max, clnrm) ARE published on crates.io and DO resolve via `cargo fetch`.
# The real blocker is that `wasm4pm-compat` version 26.6.13 does not compile
# under current Rust nightly — approximately 550 compiler errors (E0432
# unresolved imports of internal types) in the upstream crate itself.
# `cargo build` / `cargo test` / `cargo clippy` all fail because of this
# broken upstream crate, not because of missing local directories.
#
# Usage:  bash scripts/bootstrap.sh   (or ./scripts/bootstrap.sh)
set -euo pipefail

# Resolve repo root from this script's location, independent of caller cwd.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

say()  { printf '\n\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m  ok\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m  !!\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

# --- 1. toolchain sanity ---------------------------------------------------
say "Checking toolchains"

command -v cargo >/dev/null 2>&1 || die "cargo not found — install Rust via https://rustup.rs"
ok "cargo: $(cargo --version)"

if command -v rustup >/dev/null 2>&1; then
  if rustup toolchain list 2>/dev/null | grep -q '^nightly'; then
    ok "rustup nightly toolchain present (rust-toolchain.toml pins nightly)"
  else
    warn "nightly toolchain not installed; rust-toolchain.toml pins nightly."
    warn "install it with:  rustup toolchain install nightly"
  fi
else
  warn "rustup not found — this repo pins the nightly channel (rust-toolchain.toml)."
fi

command -v node >/dev/null 2>&1 || die "node not found — install Node 22 (see .devcontainer/devcontainer.json)"
node_major="$(node --version | sed -E 's/^v([0-9]+).*/\1/')"
if [ "$node_major" -lt 22 ]; then
  warn "node $(node --version) detected; this repo targets Node 22."
else
  ok "node: $(node --version)"
fi

command -v npm >/dev/null 2>&1 || die "npm not found — it ships with Node."
ok "npm: $(npm --version)"

# --- 2. web/ deps (the reliable part) -------------------------------------
say "Installing web/ dependencies (npm ci)"
( cd "$REPO_ROOT/web" && npm ci )
ok "web/ dependencies installed"

# --- 3. Rust build (skipped) -----------------------------------------------
say "Skipping 'cargo build'"
warn "wasm4pm-compat 26.6.13 (from crates.io) does not compile under current"
warn "Rust nightly — ~550 E0432 errors in the upstream crate itself. This is"
warn "an upstream breakage, not a missing-directory problem. cargo build / test"
warn "/ clippy will fail until wasm4pm-compat publishes a nightly-compatible"
warn "release. The web/ app above is fully self-contained and is ready to use."

say "Bootstrap complete"
ok  "web:   cd web && npm run dev   (or: scripts/web-dev.sh)"
ok  "rust:  blocked on wasm4pm-compat 26.6.13 upstream nightly incompatibility"
