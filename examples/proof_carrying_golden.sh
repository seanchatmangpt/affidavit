#!/usr/bin/env bash
# Proof-carrying OCEL v2 golden ladder: subject mint → DECLARE gate →
# federation import → (optional) seal/v2 under crypto-trust.
#
# Run from repo root:
#   bash examples/proof_carrying_golden.sh
set -euo pipefail
cd "$(dirname "$0")/.."

echo "── 1/4 DFG subject mint + temper falsifier ──"
cargo run --quiet --example proof_carrying_ocel_dfg

echo
echo "── 2/4 DECLARE behavioural gate ──"
cargo run --quiet --example proof_carrying_declare

echo
echo "── 3/4 federation import court ──"
cargo run --quiet --example proof_carrying_federation_import

echo
echo "── 4/4 seal/v2 round-trip (crypto-trust) ──"
cargo run --quiet --features crypto-trust --example proof_carrying_ocel_seal

echo
echo "proof-carrying golden ladder: all examples ACCEPT"
