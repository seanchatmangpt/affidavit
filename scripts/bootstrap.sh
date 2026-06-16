#!/usr/bin/env bash
#
# bootstrap.sh — set up a local affidavit dev environment.
#
# What it does (in order):
#   1. Sanity-check the toolchains (cargo, rustup nightly, node 22, npm).
#   2. Install the web/ app dependencies (`npm ci`) — the reliably-installable
#      part of this repo.
#   3. Build the Rust crate ONLY if the sibling path-crates are present.
#
# HONEST NOTE: the `affidavit` crate depends on FIVE sibling path-crates
# (../clap-noun-verb, ../wasm4pm, ../wasm4pm-compat, ../lsp-max, ../clnrm) and
# ../chicago-tdd-tools (dev-dep). A bare checkout WITHOUT those siblings cannot
# `cargo build`. This script detects their absence and skips the Rust build
# with a clear explanation instead of failing noisily.
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

# --- 3. Rust build, only if siblings are present --------------------------
say "Checking for sibling path-crates required by Cargo.toml"
SIBLINGS=(clap-noun-verb wasm4pm wasm4pm-compat lsp-max clnrm chicago-tdd-tools)
missing=()
for s in "${SIBLINGS[@]}"; do
  if [ ! -d "$REPO_ROOT/../$s" ]; then
    missing+=("../$s")
  fi
done

# The presence of ../clap-noun-verb is the canonical signal that we are inside
# the full sibling workspace; gate the cargo build on it (and report any others).
if [ -d "$REPO_ROOT/../clap-noun-verb" ] && [ "${#missing[@]}" -eq 0 ]; then
  say "Sibling workspace detected — building the Rust crate (cargo build)"
  ( cd "$REPO_ROOT" && cargo build )
  ok "cargo build succeeded"
else
  warn "Skipping 'cargo build': missing sibling path-crate(s): ${missing[*]:-<none listed>}"
  warn "The affidavit crate is NOT buildable as a lone checkout — it requires the"
  warn "full sibling workspace (../clap-noun-verb, ../wasm4pm, ../wasm4pm-compat,"
  warn "../lsp-max, ../clnrm, ../chicago-tdd-tools) next to this repo."
  warn "The web/ app above is fully self-contained and is ready to use."
fi

say "Bootstrap complete"
ok  "web:   cd web && npm run dev   (or: scripts/web-dev.sh)"
ok  "rust:  available only inside the full sibling workspace"
