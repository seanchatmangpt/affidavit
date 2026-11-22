#!/usr/bin/env bash
# Build the slim gatekeeper verify_witness-only WASM and write measured receipts.
# Does NOT rebuild, re-pin, or slim the fat affidavit-wasm provenance module.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRATE="$ROOT/affidavit-gatekeeper-wasm"
TARGET_DIR="$CRATE/target"
BUILT="$TARGET_DIR/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm"
DIST_DIR="$CRATE/dist"
DIST="$DIST_DIR/affidavit_gatekeeper_verify_witness.wasm"
REG="$CRATE/registry/ARTIFACTS.sha256"
BOUND=256000

cd "$CRATE"
rustup target add wasm32-unknown-unknown >/dev/null
cargo build --locked --lib --target wasm32-unknown-unknown --profile wasm

mkdir -p "$DIST_DIR" "$CRATE/registry"

# Nightly rustc may emit bulk-memory ops; enable them for Binaryen, else copy.
if command -v wasm-opt >/dev/null 2>&1; then
  if wasm-opt -Oz --enable-bulk-memory -o "$DIST" "$BUILT" 2>/tmp/gatekeeper-wasm-opt.err; then
    :
  else
    echo "WARN: wasm-opt failed; using cargo artifact (see /tmp/gatekeeper-wasm-opt.err)" >&2
    cp "$BUILT" "$DIST"
  fi
else
  cp "$BUILT" "$DIST"
  echo "WARN: wasm-opt not found; copied cargo artifact without -Oz pass" >&2
fi

BYTES=$(wc -c < "$DIST" | tr -d ' ')
SHA=$(shasum -a 256 "$DIST" | awk '{print $1}')

{
  echo "# affidavit-gatekeeper-wasm size/sha receipt — measured, never invented"
  echo "# bound_bytes ${BOUND}"
  echo "# Fail-closed: tests/size_court.rs refuses measured > ${BOUND}."
  echo "wasm ${SHA} ${BYTES} affidavit_gatekeeper_verify_witness.wasm"
} > "$REG"

# Producer pin — measured bytes/sha only; never invent under bound.
cat > "$CRATE/registry/artifact-pin.json" <<EOF
{
  "abi_version": 1,
  "bound_bytes": ${BOUND},
  "build": {
    "locked": true,
    "profile": "wasm",
    "target": "wasm32-unknown-unknown",
    "toolchain": "nightly-2026-08-12",
    "wasm_opt": "-Oz --enable-bulk-memory"
  },
  "bytes": ${BYTES},
  "crate": "affidavit-gatekeeper-wasm",
  "exports": [
    "gatekeeper_abi_version",
    "verify_witness"
  ],
  "imports": [],
  "name": "affidavit_gatekeeper_verify_witness.wasm",
  "notes": "Pinned by scripts/gatekeeper_wasm_build.sh after size court. Fat affidavit-wasm (718736) untouched.",
  "schema": "affidavit.gatekeeper-wasm-pin/1",
  "sha256": "${SHA}"
}
EOF

echo "built=${BUILT}"
echo "dist=${DIST}"
echo "bytes=${BYTES}"
echo "sha256=${SHA}"
echo "bound=${BOUND}"

if [[ "$BYTES" -gt "$BOUND" ]]; then
  echo "REFUSED:GATEKEEPER_WASM_OVERSIZE measured=${BYTES} bound=${BOUND}" >&2
  exit 1
fi

echo "ALIVE:GATEKEEPER_WASM_SIZE measured=${BYTES} bound=${BOUND}"
