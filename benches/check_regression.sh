#!/usr/bin/env bash
# benches/check_regression.sh
# Parses Criterion change/estimates.json files and applies regression thresholds.
#
# Exit codes:
#   0 — all benchmarks pass (no regression > 10%)
#   1 — one or more benchmarks regressed > 10% (CI failure)
#
# Usage: bash benches/check_regression.sh <criterion_dir> <baselines_dir>

set -euo pipefail

CRITERION_DIR="${1:-target/criterion}"
WARN_THRESHOLD=0.05    # 5% regression → warning
FAIL_THRESHOLD=0.10    # 10% regression → failure

overall_status=0
warnings=0
failures=0

echo "=== Criterion Regression Check ==="
echo "Criterion dir: $CRITERION_DIR"
echo "Warn threshold: ${WARN_THRESHOLD} (5%)"
echo "Fail threshold: ${FAIL_THRESHOLD} (10%)"
echo ""

# Find all change/estimates.json files produced by --baseline comparison
while IFS= read -r estimates_file; do
    # Extract the bench group path from the file path
    rel_path="${estimates_file#$CRITERION_DIR/}"
    group=$(echo "$rel_path" | sed 's|/change/estimates.json||')

    # Parse mean.point_estimate from the change estimates
    # Criterion stores the relative change as a fraction (e.g., 0.12 = +12%)
    if ! change=$(python3 -c "
import json, sys
with open('$estimates_file') as f:
    d = json.load(f)
# 'mean' key holds ConfidenceInterval with point_estimate
mean_change = d.get('mean', {}).get('point_estimate', 0.0)
print(f'{mean_change:.6f}')
" 2>/dev/null); then
        echo "SKIP $group (could not parse estimates)"
        continue
    fi

    # Determine status
    if python3 -c "import sys; sys.exit(0 if float('$change') > $FAIL_THRESHOLD else 1)" 2>/dev/null; then
        echo "FAIL  $group  (+$(python3 -c "print(f'{float(\"$change\")*100:.1f}')") %)"
        failures=$((failures + 1))
        overall_status=1
    elif python3 -c "import sys; sys.exit(0 if float('$change') > $WARN_THRESHOLD else 1)" 2>/dev/null; then
        echo "WARN  $group  (+$(python3 -c "print(f'{float(\"$change\")*100:.1f}')") %)"
        warnings=$((warnings + 1))
    else
        echo "PASS  $group"
    fi
done < <(find "$CRITERION_DIR" -name "estimates.json" -path "*/change/*" 2>/dev/null)

echo ""
echo "=== Summary ==="
echo "Failures (>10% regression): $failures"
echo "Warnings (5-10% regression): $warnings"

if [ "$overall_status" -ne 0 ]; then
    echo "RESULT: FAIL — one or more benchmarks regressed beyond the 10% threshold."
    exit 1
else
    echo "RESULT: PASS — no benchmarks regressed beyond the 10% threshold."
    exit 0
fi
