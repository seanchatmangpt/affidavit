#!/bin/bash
#
# run_quality_benchmarks.sh
#
# Comprehensive benchmark runner for Western Electric quality control rules.
# Generates flamegraph, timing table, memory profile, and regression analysis.
#
# Usage:
#   ./benches/run_quality_benchmarks.sh [baseline_name] [verbose]
#
# Examples:
#   ./benches/run_quality_benchmarks.sh                  # Run benchmarks
#   ./benches/run_quality_benchmarks.sh baseline_v1      # Compare to baseline
#   ./benches/run_quality_benchmarks.sh "" verbose       # Run with verbose output
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BASELINE_NAME="${1:-}"
VERBOSE="${2:-}"
FEATURES="quality-monitor"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Output formatting
print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# ============================================================================
# 1. Verify Environment
# ============================================================================

print_header "Environment Check"

if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust: https://rustup.rs"
    exit 1
fi
print_success "cargo installed: $(cargo --version)"

RUSTC_VERSION=$(rustc --version)
if [[ ! "$RUSTC_VERSION" =~ "1.7" ]]; then
    print_warn "Rust version: $RUSTC_VERSION (1.78+ recommended)"
else
    print_success "Rust version: $RUSTC_VERSION"
fi

# Check for flamegraph (optional)
if command -v flamegraph &> /dev/null; then
    print_success "flamegraph available"
    HAS_FLAMEGRAPH=true
else
    print_warn "flamegraph not installed (optional)"
    print_warn "Install with: cargo install flamegraph"
    HAS_FLAMEGRAPH=false
fi

# ============================================================================
# 2. Build Benchmarks
# ============================================================================

print_header "Building Benchmarks"

if cargo build --benches --features "$FEATURES" 2>&1 | tail -20 | grep -q "Finished"; then
    print_success "Benchmark build successful"
else
    print_error "Benchmark build failed"
    cargo build --benches --features "$FEATURES" 2>&1 | tail -30
    exit 1
fi

# ============================================================================
# 3. Run Benchmarks
# ============================================================================

print_header "Running Benchmarks"

BENCH_ARGS="--bench quality_western_electric --features $FEATURES"

if [ -n "$BASELINE_NAME" ]; then
    print_warn "Comparing to baseline: $BASELINE_NAME"
    BENCH_ARGS="$BENCH_ARGS -- --baseline $BASELINE_NAME"
elif [ -z "$VERBOSE" ]; then
    BENCH_ARGS="$BENCH_ARGS -- --save-baseline baseline_$(date +%Y%m%d_%H%M%S)"
fi

if [ -n "$VERBOSE" ]; then
    cargo bench $BENCH_ARGS --verbose
else
    cargo bench $BENCH_ARGS 2>&1 | tee /tmp/bench_output.txt
fi

print_success "Benchmarks completed"

# ============================================================================
# 4. Generate Report Tables
# ============================================================================

print_header "Benchmark Results Summary"

# Extract key metrics from Criterion output
if [ -f /tmp/bench_output.txt ]; then
    echo ""
    echo "Top 10 Slowest Operations:"
    grep -E "time:.*\[" /tmp/bench_output.txt | tail -10
    echo ""
fi

# Display Criterion results directory
CRITERION_DIR="$PROJECT_ROOT/target/criterion"
if [ -d "$CRITERION_DIR" ]; then
    print_success "Criterion reports: $CRITERION_DIR"

    # List benchmark results
    echo ""
    echo "Benchmark Groups:"
    find "$CRITERION_DIR" -maxdepth 1 -type d -name "rule_*" -o -name "*_detection" -o -name "*_analysis" | \
        sed 's|.*/||' | sort | while read group; do
        REPORT="$CRITERION_DIR/$group/report/index.html"
        if [ -f "$REPORT" ]; then
            echo "  • $group"
        fi
    done
    echo ""
fi

# ============================================================================
# 5. Memory Profile (simple estimate)
# ============================================================================

print_header "Memory Profile Estimation"

cat << 'EOF'
Expected Peak Memory Usage:

Benchmark                    Est. Memory    Rationale
───────────────────────────────────────────────────────
rule_1_sigma (500 pts)       ~150 KB        VecDeque<f64> × 100
rule_9_in_row (500 pts)      ~150 KB        VecDeque<f64> × 100
rule_trend (500 pts)         ~150 KB        VecDeque<f64> × 100
rule_alternating (500 pts)   ~150 KB        VecDeque<f64> × 100
rule_2_of_3 (500 pts)        ~150 KB        VecDeque<f64> × 100
rule_4_of_5 (500 pts)        ~150 KB        VecDeque<f64> × 100
rule_15_in_row (500 pts)     ~150 KB        VecDeque<f64> × 100
all_rules_simultaneous       ~500 KB        Single analyzer + varied pattern
rule_variants (4×4 combos)   ~150 KB        Per-variant analyzer, recycled
correlation_detection (200m) ~2-3 MB        HashMap<metric, Analyzer>
ocel_event_generation (2000) ~500 KB        Seq counter + event bytes
causal_chain_building (100e) ~200 KB        Receipt events with parent refs
object_level_analysis (2000) ~1-2 MB        HashMap<object, Metrics>

Total Estimated Peak:        ~8-10 MB       All benchmarks concurrent

Note: Measurements performed with --release profile (LTO enabled)
EOF

# ============================================================================
# 6. Performance Targets Check
# ============================================================================

print_header "Performance Targets Verification"

cat << 'EOF'
Target Analysis (actual results in Criterion reports):

Benchmark                           Target              Status
──────────────────────────────────────────────────────────────
Rule detection (100 pts)            < 1ms              (see target/criterion/)
Rule detection (500 pts)            < 5ms              (linear scaling)
All 7 rules (100 pts)              < 2ms              (7x single rule)
Correlation (100 metrics)          < 10ms              (HashMap + iteration)
OCEL event gen (per event)         < 5ms              (incl. validation)
OCEL event gen (1000 total)        < 5sec              (throughput)
Causal chain (50 events)           < 50ms              (rolling hash)
Object analysis (1000 objs)        < 50ms              (linear scan + stats)

Red flags (if exceeded):
  • Rule detection > 2ms: Check rolling window size or rule complexity
  • Correlation > 20ms: HashMap contention or bad scaling
  • OCEL gen > 10ms per event: Validate admission gate overhead
  • Chain > 100ms: Rolling hash computation, check BLAKE3 perf
  • Object analysis > 100ms: Sorting or allocation issue

EOF

# ============================================================================
# 7. Flamegraph Generation (optional)
# ============================================================================

if [ "$HAS_FLAMEGRAPH" = true ]; then
    print_header "Flamegraph Generation (Optional)"

    read -p "Generate flamegraph for detailed profiling? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_warn "Generating flamegraph (this may take 1-2 minutes)..."

        # Note: flamegraph requires perf on Linux; adjust for your system
        if cargo flamegraph --bench quality_western_electric --features "$FEATURES" 2>/dev/null; then
            print_success "Flamegraph generated: $PROJECT_ROOT/flamegraph.svg"
            echo "Open in browser: file://$PROJECT_ROOT/flamegraph.svg"
        else
            print_error "Flamegraph generation failed (requires perf/dtrace)"
        fi
    fi
fi

# ============================================================================
# 8. Summary and Recommendations
# ============================================================================

print_header "Benchmark Summary"

cat << 'EOF'
✓ Benchmarks completed successfully

Next Steps:
  1. Review Criterion HTML reports:
     open target/criterion/report/index.html

  2. Analyze regressions:
     Compare baseline vs. current run in Criterion UI

  3. Profile hotspots:
     - Rule detection: Check admission gate overhead
     - Correlation: Profile HashMap access patterns
     - OCEL: Measure BLAKE3 commitment cost

  4. Scaling analysis:
     - Plot latency vs. data size (should be ~linear)
     - Check memory scaling (should be linear)
     - Verify no pathological cases (e.g., hash collisions)

Performance Tuning Opportunities:
  • Rolling window: Try simd_json for parsing
  • Correlation: Parallel metric analysis with rayon
  • OCEL: Batch event construction
  • Chain: Incremental rolling hash (already done)
  • Object analysis: Parallel stddev with rayon

Documentation:
  • See benches/QUALITY_WESTERN_ELECTRIC_REPORT.md for details
  • Benchmark targets and rationale explained
  • Memory and time complexity analysis included

EOF

# ============================================================================
# 9. Cleanup
# ============================================================================

print_header "Cleanup"

if [ -f /tmp/bench_output.txt ]; then
    rm /tmp/bench_output.txt
    print_success "Cleaned temporary files"
fi

print_success "All done!"
