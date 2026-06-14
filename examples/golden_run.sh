#!/usr/bin/env bash
#
# golden_run.sh — end-to-end smoke for the affidavit Provenance Layer.
#
# Drives the real built `affi` binary through the full lifecycle:
#   emit (two events) -> assemble -> verify (ACCEPT, exit 0)
#   -> corrupt the receipt with sed -> verify (REJECT, non-zero exit).
#
# Self-contained: runs entirely inside a temp dir so the repo working tree
# is untouched. Invoke from the repo root:  bash examples/golden_run.sh

set -euo pipefail

# Resolve the repo root from this script's location so `cargo run` finds the
# manifest regardless of the caller's cwd.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFEST="$REPO_ROOT/Cargo.toml"

# A stable wrapper around the actual binary.
affi() { cargo run --quiet --manifest-path "$MANIFEST" --bin affi -- "$@"; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

echo "=== work dir: $WORK ==="

# --- emit: two operation-events with small payloads ----------------------
printf 'source bytes for stage one\n'  > payload_a.txt
printf 'source bytes for stage two\n'  > payload_b.txt

echo
echo "--- emit event 1 ---"
affi receipt emit --type seed     --object art1:artifact:input  --payload payload_a.txt
echo "--- emit event 2 ---"
affi receipt emit --type validate --object art1:artifact:output --payload payload_b.txt

# --- assemble: finalize into an immutable content-addressed receipt -------
echo
echo "--- assemble ---"
affi receipt assemble --out receipt.json

# --- verify honest receipt: expect ACCEPT, exit 0 ------------------------
echo
echo "--- verify (honest, expect ACCEPT / exit 0) ---"
set +e
affi receipt verify receipt.json
honest_code=$?
set -e
echo "exit code: $honest_code"
if [ "$honest_code" -ne 0 ]; then
  echo "FAIL: honest receipt did not ACCEPT (exit $honest_code)" >&2
  exit 1
fi

# --- corrupt: flip an event_type in place with sed -----------------------
# Tampering one event's bytes must propagate through the rolling chain hash,
# so chain_integrity will recompute a different hash than the stored one.
echo
echo "--- corrupt receipt.json (sed: validate -> tampered) ---"
sed -i.bak 's/"validate"/"tampered"/' receipt.json
echo "edited; original kept at receipt.json.bak"

# --- verify tampered receipt: expect REJECT, non-zero exit ---------------
echo
echo "--- verify (tampered, expect REJECT / non-zero exit) ---"
set +e
affi receipt verify receipt.json
tampered_code=$?
set -e
echo "exit code: $tampered_code"
if [ "$tampered_code" -eq 0 ]; then
  echo "FAIL: tampered receipt was accepted (exit 0)" >&2
  exit 1
fi

echo
echo "=== GOLDEN RUN OK: ACCEPT(0) then REJECT($tampered_code) ==="
