#!/usr/bin/env python3
"""AOD Zero-Text Audit Gate (AFFI-SPEC-26.11.19 PR-2 / DoD).

Scan sealed `.aod` artifacts for domain tokens, ontology IRIs, and ASCII
headers. A clean Zero-String Sealed file must yield zero hits under:

  1. `strings -n 4` (printable runs of length ≥ 4)
  2. A UTF-8 heuristic that walks contiguous printable / multi-byte UTF-8

Domain deny-list (case-sensitive unless noted) includes at least:
  CTRT, ontology, parquet, Arrow, schema, affidavit, lease_nullifier,
  and JSON keys row_count / "row_count".

Exit codes:
  0  — every scanned file is clean
  1  — usage / I/O error
  2  — one or more domain-token / IRI / ASCII-header hits (REFUSED_AOD_TEXT)

Usage:
  python3 scripts/aod_zero_text_gate.py PATH [PATH ...]
  python3 scripts/aod_zero_text_gate.py --dir DIR   # recursive *.aod
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable, List, Sequence, Tuple

# ---------------------------------------------------------------------------
# Deny-list (exact contiguous byte/UTF-8 substrings, case-sensitive)
# ---------------------------------------------------------------------------

DOMAIN_TOKENS: Tuple[str, ...] = (
    "CTRT",
    "ontology",
    "parquet",
    "Arrow",
    "schema",
    "affidavit",
    "lease_nullifier",
    "row_count",
    '"row_count"',
    # Additional wire/header leakage that would violate zero-text.
    "affidavit-core",
    "ARROW",
    "PARQUET",
    "Ontology",
)

# Ontology / RDF IRI shapes that must never appear as contiguous UTF-8.
IRI_PATTERNS: Tuple[re.Pattern[str], ...] = (
    re.compile(r"https?://[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%\-]+"),
    re.compile(r"urn:[A-Za-z0-9][A-Za-z0-9:_\-./]*", re.IGNORECASE),
    re.compile(r"rdfs?:[A-Za-z_][A-Za-z0-9_]*"),
    re.compile(r"owl:[A-Za-z_][A-Za-z0-9_]*"),
    re.compile(r"xsd:[A-Za-z_][A-Za-z0-9_]*"),
)

# Printable ASCII (space through tilde), matching typical `strings -n` default.
_PRINTABLE = frozenset(range(0x20, 0x7F))


def _is_printable_byte(b: int) -> bool:
    return b in _PRINTABLE


def extract_strings_n(data: bytes, min_len: int = 4) -> List[str]:
    """Pure-Python `strings -n MIN` over a byte buffer (ASCII printable runs)."""
    out: List[str] = []
    buf = bytearray()
    for b in data:
        if _is_printable_byte(b):
            buf.append(b)
        else:
            if len(buf) >= min_len:
                out.append(buf.decode("ascii"))
            buf.clear()
    if len(buf) >= min_len:
        out.append(buf.decode("ascii"))
    return out


def extract_utf8_runs(data: bytes, min_len: int = 4) -> List[str]:
    """UTF-8 heuristic: decode whole file with replacement, then take
    contiguous runs of printable Unicode (letters/digits/punct) of length ≥ N.

    Catches multi-byte UTF-8 domain tokens that `strings` (ASCII-only) misses.
    """
    text = data.decode("utf-8", errors="replace")
    # Keep letters, digits, underscore, hyphen, colon, slash, quote, dot —
    # enough to surface IRIs and JSON keys without treating raw binary as text.
    runs: List[str] = []
    cur: List[str] = []

    def flush() -> None:
        if len(cur) >= min_len:
            runs.append("".join(cur))
        cur.clear()

    for ch in text:
        o = ord(ch)
        if ch == "\ufffd":
            flush()
            continue
        # Printable ASCII or non-ASCII letter/number (UTF-8 domain leakage).
        if 0x20 <= o <= 0x7E or ch.isalnum() or ch in "_-:/.\"":
            cur.append(ch)
        else:
            flush()
    flush()
    return runs


def run_system_strings(path: Path, min_len: int = 4) -> List[str]:
    """Prefer the host `strings -n N` when available; else pure-Python fallback."""
    binary = shutil.which("strings")
    if binary is None:
        return extract_strings_n(path.read_bytes(), min_len)
    try:
        proc = subprocess.run(
            [binary, "-n", str(min_len), "--", str(path)],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return extract_strings_n(path.read_bytes(), min_len)
    if proc.returncode not in (0, 1):
        # Some strings builds exit 1 on empty; treat non-zero stderr as hard fail.
        if proc.stderr.strip():
            raise RuntimeError(
                f"strings failed on {path}: exit={proc.returncode} stderr={proc.stderr!r}"
            )
    lines = [ln for ln in proc.stdout.splitlines() if ln]
    return lines


def ascii_header_hit(data: bytes) -> bool:
    """True when the first 4 bytes are all printable ASCII (forbidden magic)."""
    if len(data) < 4:
        return True
    return all(_is_printable_byte(b) for b in data[:4])


def scan_blob(data: bytes, label: str) -> List[str]:
    """Return human-readable hit messages for one blob. Empty ⇒ clean."""
    hits: List[str] = []
    if ascii_header_hit(data):
        magic = data[:4].hex() if len(data) >= 4 else data.hex()
        hits.append(f"{label}: ASCII header magic ({magic})")

    # Exact deny-list over raw bytes (case-sensitive).
    for token in DOMAIN_TOKENS:
        needle = token.encode("utf-8")
        if needle in data:
            hits.append(f"{label}: domain token {token!r}")

    # IRI shapes over UTF-8 heuristic runs + whole decode.
    text = data.decode("utf-8", errors="replace")
    for pat in IRI_PATTERNS:
        m = pat.search(text)
        if m:
            hits.append(f"{label}: ontology IRI {m.group(0)!r}")

    return hits


def scan_file(path: Path) -> List[str]:
    data = path.read_bytes()
    hits = scan_blob(data, str(path))

    # Cross-check: every strings / UTF-8 run must not contain a deny token.
    try:
        string_runs = run_system_strings(path, 4)
    except RuntimeError as exc:
        return hits + [str(exc)]
    utf8_runs = extract_utf8_runs(data, 4)
    combined = set(string_runs) | set(utf8_runs)

    for run in sorted(combined):
        for token in DOMAIN_TOKENS:
            if token in run:
                msg = f"{path}: strings/utf8 run contains {token!r} (run={run!r})"
                if msg not in hits:
                    hits.append(msg)
        for pat in IRI_PATTERNS:
            m = pat.search(run)
            if m:
                msg = f"{path}: strings/utf8 run contains IRI {m.group(0)!r}"
                if msg not in hits:
                    hits.append(msg)
    return hits


def collect_targets(paths: Sequence[Path], directory: Path | None) -> List[Path]:
    targets: List[Path] = []
    for p in paths:
        if p.is_dir():
            targets.extend(sorted(p.rglob("*.aod")))
        else:
            targets.append(p)
    if directory is not None:
        targets.extend(sorted(directory.rglob("*.aod")))
    # De-dupe while preserving order.
    seen = set()
    out: List[Path] = []
    for t in targets:
        key = t.resolve()
        if key not in seen:
            seen.add(key)
            out.append(t)
    return out


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Zero-Text Audit Gate over sealed .aod artifacts (AFFI-SPEC-26.11.19 PR-2)."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        help=".aod file(s) or directories containing them",
    )
    parser.add_argument(
        "--dir",
        type=Path,
        default=None,
        help="Recursively scan DIR for *.aod",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run built-in positive/negative fixtures and exit",
    )
    args = parser.parse_args(list(argv) if argv is not None else None)

    if args.self_test:
        return _self_test()

    targets = collect_targets(args.paths, args.dir)
    if not targets:
        print("aod_zero_text_gate: no .aod targets given", file=sys.stderr)
        return 1

    all_hits: List[str] = []
    for path in targets:
        if not path.is_file():
            print(f"aod_zero_text_gate: missing file {path}", file=sys.stderr)
            return 1
        hits = scan_file(path)
        if hits:
            all_hits.extend(hits)
        else:
            print(f"PASS: {path} (zero domain tokens / IRIs / ASCII headers)")

    if all_hits:
        print("REFUSED_AOD_TEXT: zero-text audit failed", file=sys.stderr)
        for h in all_hits:
            print(f"  HIT: {h}", file=sys.stderr)
        return 2

    print(f"aod_zero_text_gate: PASS ({len(targets)} file(s))")
    return 0


def _self_test() -> int:
    """Positive (must pass) + negative (must refuse) in-memory fixtures."""
    clean = bytes([0x9F, 0x1A, 0xC4, 0x01]) + bytes(range(256)) * 2
    dirty_token = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"xxxx" + b"CTRT" + b"yyyy"
    dirty_iri = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"see http://example.org/onto#X "
    dirty_ascii = b"AOD1" + b"\x00" * 64
    dirty_json = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b'{"row_count":2}'

    failures = 0
    if scan_blob(clean, "clean"):
        print("SELFTEST FAIL: clean blob should pass", file=sys.stderr)
        failures += 1
    for label, blob in (
        ("token", dirty_token),
        ("iri", dirty_iri),
        ("ascii-header", dirty_ascii),
        ("json-key", dirty_json),
    ):
        if not scan_blob(blob, label):
            print(f"SELFTEST FAIL: {label} blob should refuse", file=sys.stderr)
            failures += 1

    # Deny-list coverage check.
    required = {
        "CTRT",
        "ontology",
        "parquet",
        "Arrow",
        "schema",
        "affidavit",
        "lease_nullifier",
        "row_count",
    }
    missing = required - set(DOMAIN_TOKENS)
    if missing:
        print(f"SELFTEST FAIL: deny-list missing {missing}", file=sys.stderr)
        failures += 1

    if failures:
        return 2
    print("aod_zero_text_gate: self-test PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
