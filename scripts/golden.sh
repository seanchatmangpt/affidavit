#!/usr/bin/env bash
#
# golden.sh — run the end-to-end "golden run" smoke test.
#
# Wraps examples/golden_run.sh, which drives the REAL `affi` binary through
# the full lifecycle in a temp dir:
#   emit two events -> assemble -> verify (ACCEPT, exit 0)
#   -> corrupt the receipt with sed -> verify (REJECT, non-zero exit).
# That ACCEPT-then-REJECT distinction is the tamper-teeth admission witness
# for the whole pipeline (ARDPRD FR-6).
#
# GUARD: the golden run needs the affi binary, which is built via `cargo run`
# and therefore requires the sibling PATH crates (see CONTRIBUTING.md). We
# probe for that workspace and fail with a clear message if it is absent,
# instead of letting `cargo` emit a wall of unresolved-dependency errors.
#
# Usage:  scripts/golden.sh        (run from anywhere)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GOLDEN="$REPO_ROOT/examples/golden_run.sh"

if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo not found. Install Rust via https://rustup.rs." >&2
  exit 1
fi

if [ ! -f "$GOLDEN" ]; then
  echo "ERROR: $GOLDEN not found." >&2
  exit 1
fi

# The golden run compiles + runs affi via `cargo run`, which needs the five
# sibling PATH crates. Probe the sentinel; refuse early if it is missing.
SIBLING_SENTINEL="$REPO_ROOT/../clap-noun-verb"
if [ ! -d "$SIBLING_SENTINEL" ]; then
  echo "ERROR: cannot run the golden run — the affi binary cannot be built here." >&2
  echo "       Missing sibling crate: $SIBLING_SENTINEL" >&2
  echo "       The affidavit crate depends on five sibling PATH crates that are" >&2
  echo "       NOT in this repo (../clap-noun-verb, ../wasm4pm*, ../lsp-max," >&2
  echo "       ../clnrm, ../chicago-tdd-tools). Check out the full workspace and retry." >&2
  exit 1
fi

echo "==> running the golden run (examples/golden_run.sh)"
echo "    this builds affi via 'cargo run' and exercises ACCEPT then REJECT."
echo
exec bash "$GOLDEN"
