#!/usr/bin/env bash
# AFFI-SPEC-26.11.19 PR-8 — affidavit-side ecosystem crypto/git fence checklist.
# Exit 0 when the micro-verifier scaffold compiles and carries no local crypto deps.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MV="$ROOT/micro_verifier"

echo "== micro_verifier court =="
cd "$MV"
cargo test --quiet
cargo clippy --all-targets -- -D warnings
cargo build --quiet --release

echo "== dependency fence (Cargo.toml) =="
# Fail if micro_verifier grows crypto/git crates of its own.
if rg -n '^\s*(blake3|ml-dsa|slh-dsa|ml-kem|ring|openssl|git2|libgit2)\s*=' Cargo.toml; then
  echo "REFUSED_ECOSYSTEM_CRYPTO_DEP: micro_verifier must not depend on crypto/git crates" >&2
  exit 2
fi

echo "== ABI version parity =="
# Header: `#define AF_AOD_ABI_VERSION 1u` — take the leading digits.
HDR_NUM="$(rg -o 'AF_AOD_ABI_VERSION[[:space:]]+[0-9]+' include/affidavit_aod.h | rg -o '[0-9]+' | head -1)"
RS_NUM="$(rg -o 'pub const ABI_VERSION: u32 = [0-9]+' src/lib.rs | rg -o '[0-9]+$' | head -1)"
if [[ -z "$HDR_NUM" || -z "$RS_NUM" || "$HDR_NUM" != "$RS_NUM" ]]; then
  echo "REFUSED_ABI_DRIFT: header=${HDR_NUM:-missing} rust=${RS_NUM:-missing}" >&2
  exit 2
fi

echo "AOD ecosystem fence: ACCEPT (micro_verifier ABI=$RS_NUM)"
