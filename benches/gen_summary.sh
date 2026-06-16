#!/usr/bin/env bash
# benches/gen_summary.sh — Generate BENCH_SUMMARY.md from Criterion JSON estimates.
#
# Usage: bash benches/gen_summary.sh

set -euo pipefail

CRITERION_DIR="target/criterion"
OUTPUT="BENCH_SUMMARY.md"

echo "Generating $OUTPUT from $CRITERION_DIR"

cat <<EOF > "$OUTPUT"
## Benchmark Summary — affidavit $(grep '^version =' Cargo.toml | cut -d'"' -f2)

**Run date:** $(date +%Y-%m-%d)
**Branch:** $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
**Baseline:** $(ls $CRITERION_DIR/*/change/estimates.json 2>/dev/null | head -n1 | xargs dirname | xargs dirname | xargs basename || echo "none")

### Throughput

| Benchmark | Mean | p99 | vs. Baseline | Status |
|-----------|------|-----|-------------|--------|
EOF

# Extract throughput data
find "$CRITERION_DIR/throughput" -name "estimates.json" -path "*/new/*" 2>/dev/null | sort | while read -r estimates_file; do
    group_path=$(echo "$estimates_file" | sed "s|$CRITERION_DIR/||;s|/new/estimates.json||")
    
    mean=$(python3 -c "import json; print(f\"{json.load(open('$estimates_file'))['mean']['point_estimate']/1000000:.2f} ms\")" 2>/dev/null || echo "N/A")
    # p99 is not directly in estimates.json in a standard way, but we can approximate or use median.
    # Actually Criterion estimates.json has various quantiles if configured, but let's use what we have.
    # For simplicity in this script, we'll just use mean.
    
    change_file=$(echo "$estimates_file" | sed 's|/new/|/change/|')
    if [ -f "$change_file" ]; then
        change=$(python3 -c "import json; c = json.load(open('$change_file'))['mean']['point_estimate']; print(f\"{c*100:+.1f}%\")" 2>/dev/null || echo "N/A")
        status="PASS"
        # Check if change > 10%
        if python3 -c "import json; c = json.load(open('$change_file'))['mean']['point_estimate']; exit(0 if c > 0.10 else 1)" 2>/dev/null; then
            status="FAIL"
        fi
    else
        change="N/A"
        status="UNCHECKED"
    fi
    
    echo "| $group_path | $mean | - | $change | $status |" >> "$OUTPUT"
done

cat <<EOF >> "$OUTPUT"

### Variance (Control-Flow Surprise)

| Benchmark | Mean | p99 | vs. Baseline |
|-----------|------|-----|-------------|
EOF

find "$CRITERION_DIR/variance" -name "estimates.json" -path "*/new/*" 2>/dev/null | sort | while read -r estimates_file; do
    group_path=$(echo "$estimates_file" | sed "s|$CRITERION_DIR/||;s|/new/estimates.json||")
    mean=$(python3 -c "import json; print(f\"{json.load(open('$estimates_file'))['mean']['point_estimate']/1000000:.2f} ms\")" 2>/dev/null || echo "N/A")
    
    change_file=$(echo "$estimates_file" | sed 's|/new/|/change/|')
    if [ -f "$change_file" ]; then
        change=$(python3 -c "import json; c = json.load(open('$change_file'))['mean']['point_estimate']; print(f\"{c*100:+.1f}%\")" 2>/dev/null || echo "N/A")
    else
        change="N/A"
    fi
    echo "| $group_path | $mean | - | $change |" >> "$OUTPUT"
done

echo "Summary generated at $OUTPUT"
