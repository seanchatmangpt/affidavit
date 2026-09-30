#!/usr/bin/env bash
# perf_budget_check.sh — machine-relative drift alarm for the crypto trust plane.
#
# Re-measures the real criterion suite (benches/crypto_trust_bench.rs, feature
# `crypto-trust`) with the same settings docs/CRYPTO_TRUST_PLANE.md was recorded
# with (--sample-size 10 --warm-up-time 1 --measurement-time 2), extracts each
# bench's `median.point_estimate` from target/criterion/<bench>/new/estimates.json,
# and compares it against the recorded medians in the doc's "Measured performance"
# table. Fails (exit 2, naming bench + ratio) when any median exceeds 10x the
# recorded number. Being FASTER never fails. Wired into .github/workflows/rust.yml
# (job `perf-budget`, runs after build-and-test).
#
# Environment:
#   CTP_SKIP_PERF_BUDGET=1   print SKIP and exit 0 without running anything
#   CTP_PERF_DOC=<path>      measured-performance doc (default docs/CRYPTO_TRUST_PLANE.md)
#   CTP_CRITERION_DIR=<dir>  criterion output dir (default <CARGO_TARGET_DIR:-target>/criterion)
#   CTP_PERF_TOLERANCE=<x>   alarm ratio, default 10
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${CTP_SKIP_PERF_BUDGET:-0}" == "1" ]]; then
  echo "SKIP: CTP_SKIP_PERF_BUDGET=1 - crypto-trust performance budget check not run"
  exit 0
fi

PERF_DOC="${CTP_PERF_DOC:-$REPO_ROOT/docs/CRYPTO_TRUST_PLANE.md}"
CRITERION_DIR="${CTP_CRITERION_DIR:-}"
TOLERANCE="${CTP_PERF_TOLERANCE:-10}"

if [[ ! -f "$PERF_DOC" ]]; then
  echo "FAIL(no-doc): measured-performance doc not found: $PERF_DOC" >&2
  exit 1
fi
command -v cargo >/dev/null 2>&1 || { echo "FAIL(no-cargo): cargo not on PATH" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "FAIL(no-python3): python3 not on PATH" >&2; exit 1; }

if [[ -z "$CRITERION_DIR" ]]; then
  CRITERION_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}/criterion"
fi

cd "$REPO_ROOT"
echo "==> cargo bench --features crypto-trust --bench crypto_trust_bench (sample-size 10, warm-up 1s, measurement 2s)"
cargo bench --features crypto-trust --bench crypto_trust_bench -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 2

export PERF_DOC CRITERION_DIR TOLERANCE
python3 - <<'PYEOF'
import json
import os
import re
import sys

doc_path = os.environ["PERF_DOC"]
criterion_dir = os.environ["CRITERION_DIR"]
tolerance = float(os.environ["TOLERANCE"])

# Doc rows look like:  | `ctp_es256_sign` | ES256 sign | 131.47 µs | machine |
# Durations are converted to ns; criterion's estimates.json is already in ns.
unit_ns = {"ns": 1.0, "us": 1e3, "µs": 1e3, "μs": 1e3, "ms": 1e6, "s": 1e9}
row_re = re.compile(
    r"^\|\s*`(ctp_[A-Za-z0-9_]+)`\s*\|[^|]*\|\s*"
    r"([0-9]+(?:\.[0-9]+)?)\s*(ns|µs|μs|us|ms|s)\s*\|",
    re.M,
)
with open(doc_path, encoding="utf-8") as f:
    text = f.read()
recorded = {m.group(1): float(m.group(2)) * unit_ns[m.group(3)] for m in row_re.finditer(text)}
if not recorded:
    print(
        "FAIL(no-table): no measured-performance rows parsed from %s" % doc_path,
        file=sys.stderr,
    )
    sys.exit(1)


def human(ns):
    for label, scale in (("ms", 1e6), ("us", 1e3), ("ns", 1.0)):
        if ns >= scale:
            return "%.2f %s" % (ns / scale, label)
    return "%.2f ns" % ns


violations = []
missing = []
print("%-28s %14s %14s %8s" % ("bench", "recorded", "measured", "ratio"))
for bench in sorted(recorded):
    estimates = os.path.join(criterion_dir, bench, "new", "estimates.json")
    if not os.path.exists(estimates):
        missing.append(bench)
        continue
    with open(estimates, encoding="utf-8") as f:
        measured_ns = json.load(f)["median"]["point_estimate"]
    recorded_ns = recorded[bench]
    ratio = measured_ns / recorded_ns
    flag = ""
    if ratio > tolerance:
        flag = "  <-- EXCEEDS BUDGET"
        violations.append((bench, measured_ns, recorded_ns, ratio))
    print(
        "%-28s %14s %14s %7.2fx%s"
        % (bench, human(recorded_ns), human(measured_ns), ratio, flag)
    )

if missing:
    for bench in missing:
        print(
            "FAIL(missing): criterion result not found for %s: %s/%s/new/estimates.json"
            % (bench, criterion_dir, bench),
            file=sys.stderr,
        )
    sys.exit(1)

if violations:
    for bench, measured_ns, recorded_ns, ratio in violations:
        print(
            "FAIL(budget): %s median %s is %.2fx the recorded %s (tolerance %.0fx) — see %s"
            % (bench, human(measured_ns), ratio, human(recorded_ns), tolerance, doc_path),
            file=sys.stderr,
        )
    sys.exit(2)

print(
    "OK: %d crypto-trust benches within the %.0fx recorded-median budget (%s)"
    % (len(recorded), tolerance, doc_path)
)
PYEOF
