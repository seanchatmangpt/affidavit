#!/usr/bin/env python3
"""Byte-sync guard for the vendored dfcm receipt schema.

Asserts, in order:
  1. schemas/dfcm-receipt.schema.json (vendored) is BYTE-identical to the canonical
     ~/.zcode/dfcm/receipt.schema.json
  2. the byte-twin ~/.claude/dfcm/receipt.schema.json is byte-identical to the same
     canonical file (extinction-defect guard: the twin dies if one path drifts)
  3. the ALOOP profile's $defs/dfcmReceipt is semantically identical to the vendored
     copy (the inline exists so the profile resolves with no network dependency)

Exit 0 = in sync; exit 1 = drift (fix the canonical graph, re-vendor, never hand-edit
the projections). When ~/.zcode/dfcm does not exist (foreign checkout), checks 1-2 are
skipped and 3 still runs against the vendored copy alone.
"""
import hashlib, json, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
VENDORED = REPO / "schemas/dfcm-receipt.schema.json"
PROFILE = REPO / "schemas/aloop-execution-receipt.schema.json"
CANONICAL_ZCODE = Path.home() / ".zcode/dfcm/receipt.schema.json"
CANONICAL_CLAUDE = Path.home() / ".claude/dfcm/receipt.schema.json"


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def main():
    ok = True
    vendored_bytes = VENDORED.read_bytes()
    print(f"vendored  {VENDORED}: sha256:{sha(VENDORED)}")
    for canonical in (CANONICAL_ZCODE, CANONICAL_CLAUDE):
        if not canonical.exists():
            print(f"skip      {canonical}: not present on this machine")
            continue
        same = canonical.read_bytes() == vendored_bytes
        ok &= same
        print(f"{'OK  ' if same else 'DRIFT'} {canonical}: sha256:{sha(canonical)}")
    profile = json.loads(PROFILE.read_text())
    inline = json.dumps(profile.get("$defs", {}).get("dfcmReceipt"))
    vendored_obj = json.loads(vendored_bytes)
    same = inline == json.dumps(vendored_obj)
    ok &= same
    print(f"{'OK  ' if same else 'DRIFT'} {PROFILE} $defs/dfcmReceipt == vendored copy")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
