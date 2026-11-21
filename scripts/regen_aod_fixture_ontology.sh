#!/usr/bin/env bash
# Regenerate / verify the checked-in AOD fixture CompiledOntology pin.
#
# Source of truth: ontology/aod-fixture.ttl + FIXTURE_* in
# affidavit-core/src/aod/fixture_ontology.rs
#
# Usage:
#   ./scripts/regen_aod_fixture_ontology.sh           # print derived hash
#   ./scripts/regen_aod_fixture_ontology.sh --check    # fail if pin drifts
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/affidavit-core"

MODE="${1:-}"
if [[ "$MODE" == "--check" ]]; then
  cargo test --lib aod::fixture_ontology::tests::fixture_hash_pin_matches_derivation -- --exact
  echo "OK: FIXTURE_ONTOLOGY_HASH pin matches derive_ontology_hash"
  exit 0
fi

cargo test --lib aod::fixture_ontology::tests::fixture_hash_pin_matches_derivation -- --exact --nocapture 2>&1 | tee /tmp/aod-fixture-regen.log
echo
echo "If the pin drifted, update FIXTURE_ONTOLOGY_HASH / FIXTURE_ONTOLOGY_HASH_HEX"
echo "in affidavit-core/src/aod/fixture_ontology.rs and ontology/aod-fixture.ttl"
echo "from derive_ontology_hash(FIXTURE_SCHEMA_ID, FIXTURE_STRIDES)."
