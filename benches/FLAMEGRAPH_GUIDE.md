# Flamegraph Interpretation & Profiling Guide

This guide explains how to generate, analyze, and interpret flamegraphs for the Western Electric quality control benchmarks.

## Quick Start

### 1. Generate Flamegraph

```bash
# Install flamegraph tool (one-time)
cargo install flamegraph

# Generate flamegraph for the benchmark
cd /home/user/affidavit
cargo flamegraph --bench quality_western_electric --features quality-monitor

# Opens in browser automatically or:
open flamegraph.svg
```

### 2. Read the Flamegraph

A flamegraph is a horizontal stacked-area chart where:
- **X-axis width** = time spent in that function
- **Y-axis stacking** = call stack (function A calls function B)
- **Color** = assigned by heuristic (not significant)

**Wider boxes = more time spent = optimization targets**

---

## Flamegraph Interpretation for Each Benchmark

### Benchmark 1: Single Rule Detection (rule_1_sigma_detection)

**Expected flamegraph pattern:**

```
top
├─ [wide box] WesternElectricAnalyzer::new()  ~1%
├─ [WIDEST]   loop over measurements
│  └─ [wide] add_measurement()
│     ├─ [medium] VecDeque::push_back()  ~20%
│     ├─ [medium] VecDeque::pop_front()  ~10% (if window exceeds size)
│     └─ [WIDEST]  check_1_sigma_rule()  ~60%
│        └─ [WIDEST] division (z-score)  ~40%
└─ [medium] push result to vector  ~5%
```

**Hotspots to watch:**
1. **Division operator** (`(value - mean) / stddev`) — most expensive operation
   - *Optimization:* Precompute `1.0 / stddev` once
   
2. **Floating-point math** — inherent cost
   - *Optimization:* Use SIMD if analyzing many metrics simultaneously

3. **VecDeque push/pop** — should be minor
   - *Red flag:* If > 15%, check window size or allocation

**Expected width (100 measurements):**
```
Total: ~1.0 ms
  ├─ add_measurement × 100: ~0.98 ms (98%)
  │  └─ check_1_sigma_rule: ~0.60 ms (60%)
  │     └─ z-score division: ~0.40 ms (40%)
  ├─ new() + finalize: ~0.02 ms (2%)
```

---

### Benchmark 2: All 7 Rules Simultaneous

**Expected pattern (should see 7 calls from loop):**

```
top
└─ loop (100 iterations)
   ├─ add_measurement()
   │  ├─ VecDeque::push_back()      ~3%
   │  ├─ check_1_sigma_rule()       ~10%
   │  ├─ check_9_in_a_row_rule()    ~8%
   │  ├─ check_trend_rule()         ~12%
   │  ├─ check_alternating_rule()   ~10%
   │  ├─ check_2_of_3_rule()        ~15%
   │  ├─ check_4_of_5_rule()        ~12%
   │  └─ check_15_in_row_rule()     ~20%
```

**Rule Complexity (by expected width):**
1. **Rule 15-in-row** (20%) — most expensive (15-point window scan + z-score × 15)
2. **Rule 2-of-3** (15%) — moderate (3-point scan × z-score × 3)
3. **Rule 4-of-5** (12%) — moderate (5-point scan × z-score × 5)
4. **Rule Trend** (12%) — comparison-heavy (6 comparisons)
5. **Rule Alternating** (10%) — moderate (8 comparisons + mean check)
6. **Rule 1σ** (10%) — single z-score
7. **Rule 9-in-a-row** (8%) — simple (9 comparisons vs. limits)

**Optimization priority (by size):**
1. **Rule 15-in-row** — consider caching previous 15 measurements to avoid re-scanning
2. **Rule 2-of-3** — merge z-score computation with Rule 4-of-5 (reuse)
3. **Rules 4-of-5** → shared z-score calculation

---

### Benchmark 3: Rule Variants (σ, window size)

**Expected flamegraph (4×4 = 16 sub-benchmarks):**

Each sub-benchmark should show similar pattern:
```
top
└─ [varying width based on σ value]
   └─ add_measurement × 100
      └─ [rule checks]
```

**Key observation:** Width should NOT change significantly with σ or window size
- **Good:** All variants have same width (σ doesn't affect latency)
- **Bad:** Window size variants get progressively wider (allocation issue)

**Red flags:**
- σ=0.5 significantly wider than σ=3.0 → z-score computation variance
- Window=100 is 2× wider than window=10 → VecDeque allocation issue

---

### Benchmark 4: Correlation Detection

**Expected pattern (200 metrics, 50 batches):**

```
top
└─ for_each metric_batch
   └─ for each metric in HashMap
      └─ add_measurement()
         └─ [7 rule checks]
```

**Expected hotspots:**
1. **HashMap iteration** (~5%)  — accessing 200 entry lookups
2. **add_measurement** (~85%)  — rule checks (same as before)
3. **HashMap insertion/clone** (~10%) — if reallocating

**Scaling check:**
- 10 metrics: ~2.5 ms
- 100 metrics: ~25 ms
- **Linear?** Yes → good scaling
- **Sublinear?** Excellent → cache effects
- **Superlinear?** Bad → HashMap collision/allocation issue

**Optimization:** If superlinear, use `FxHashMap` (faster for small integers)

---

### Benchmark 5: OCEL Event Generation

**Expected pattern (1000 events):**

```
top
└─ loop (1000 times)
   ├─ SeqCounter::next_seq()            <1%
   ├─ object_ref() × 2                  ~5%
   ├─ build_event()
   │  ├─ OperationEvent struct build    ~15%
   │  ├─ Blake3Hash::from_bytes()       ~60% ← BLAKE3 hash!
   │  └─ validate_event()               ~10%
   └─ asm.append()                      ~10%
```

**Flamegraph width interpretation:**

```
Total: ~5.0 ms (for 1000 events)
  Per event: ~5.0 μs
  ├─ BLAKE3 hash: ~3.0 μs (60%)
  ├─ Event struct build: ~0.75 μs (15%)
  ├─ append/validation: ~1.0 μs (20%)
  └─ object_ref/seq: ~0.25 μs (5%)
```

**BLAKE3 is the bottleneck** (~60% of time) — this is expected and acceptable. BLAKE3 is already optimized (multi-block lane processing).

**Red flags:**
- BLAKE3 < 30% of time → possible inefficiency (ensure RELEASE build)
- append() > 20% → check Receipt chain accumulation logic

---

### Benchmark 6: Causal Chain Building

**Expected pattern (50-event chain):**

```
top
└─ loop (50 times)
   ├─ build_event()                    ~40%
   │  └─ Blake3Hash::from_bytes()      ~25% (of total)
   ├─ assembler.append()               ~30%
   │  └─ chain hash update (BLAKE3)    ~25% (of total)
   └─ object_ref × 2                   ~30%
```

**Stacked calls view:**

```
Event 0 (init):
  build_event + append → rolling hash state_0

Event 1 (derive):
  build_event + append → rolling hash state_1 = f(state_0, event_bytes_1)
  
Event N:
  rolling hash computation includes ALL prior events
```

**Critical observation:** Rolling hash should show **linear scaling**, not exponential:
- Event 1: ~0.1 ms (initial setup)
- Event 10: ~1 ms (10× the initial)
- Event 50: ~5 ms (50× the initial)

**If you see super-linear (e.g., Event 50 = 100× Event 1):** Possible issue with:
- Repeated finalization in loop (re-hashing entire chain)
- Unbounded append operation
- Memory allocation/reallocation

**Expected box widths (50 events, ~3.75 ms total):**
```
Event construction: 1.5 ms (40%)
Append/rolling hash: 1.5 ms (40%)
Validation: 0.5 ms (15%)
Object refs: 0.25 ms (5%)
```

---

### Benchmark 7: Object-Level Analysis

**Expected pattern (1000 objects):**

```
top
├─ HashMap construction                ~50%
│  ├─ for i in 0..1000
│  │  ├─ CodeQualityMetrics creation   ~2%
│  │  └─ HashMap::insert()             ~3%
├─ Extract stub_ratios vector          ~5%
├─ Compute mean                        ~5%
├─ Compute variance/stddev            ~30%
│  └─ for each value in vec: (x-mean)²
└─ Count outliers                      ~10%
```

**Scaling verification (critical for this benchmark):**

```
Objects    Total      Stddev Comp  Growth   Analysis
────────────────────────────────────────────────────
100        ~2.5 ms    ~0.3 ms      baseline
500        ~12.5 ms   ~1.5 ms      5× (good)
1000       ~25 ms     ~3 ms        10× (linear) ✓
2000       ~50 ms     ~6 ms        20× (linear) ✓
```

**Red flag:** If stddev computation is > 50% of total:
- Data not pre-allocated (Vec reallocation)
- Repeated iteration of full vector
- Poor cache locality

**Good pattern:** stddev = O(n) where n = object count

---

## Flamegraph Analysis Workflow

### Step 1: Identify Hotspots

```bash
# Open flamegraph in browser
open flamegraph.svg

# Look for:
# - Widest boxes (most time)
# - Nested boxes (where time is spent in call stack)
# - Surprising functions (things you didn't expect)
```

### Step 2: Calculate Percentages

```
For each hotspot box:
  Percentage = Box_Width / Total_Width
  
Example:
  BLAKE3 hash = 1.5cm / 5.0cm total = 30%
```

### Step 3: Cross-Reference with Targets

Use the **TIMING_TABLE.md** as reference:

```
Expected breakdown for OCEL (1000 events):
  - BLAKE3: ~3.0 μs / 5.0 μs = 60% ✓
  - Validation: ~0.5 μs / 5.0 μs = 10% ✓
  - Append: ~1.0 μs / 5.0 μs = 20% ✓
  
If your flamegraph shows:
  - BLAKE3: 80% ✗ possible optimization
  - BLAKE3: 40% ✓ possible regression or build mode issue
```

### Step 4: Investigate Regressions

If a function is wider than expected:

1. **Check build profile:**
   ```bash
   # Verify you're using --release
   cargo bench --release --bench quality_western_electric --features quality-monitor
   ```

2. **Profile with perf (Linux):**
   ```bash
   perf record -F 99 cargo bench --bench quality_western_electric --features quality-monitor
   perf report
   ```

3. **Check for allocation/deallocation:**
   Look for `alloc`, `free`, `realloc` boxes in flamegraph

4. **Verify no debug assertions:**
   ```bash
   # In Cargo.toml [profile.bench]:
   [profile.bench]
   opt-level = 3
   debug-assertions = false
   ```

---

## Common Flamegraph Patterns & Interpretations

### Pattern 1: "Flat Top" (Good)
```
═════════════════════════════════════════════════════════
  add_measurement() spans full width
```
**Interpretation:** Single hot function doing work—optimal for CPU cache

---

### Pattern 2: "Sawtooth" (Good)
```
─────────────────┐
  check_1_sigma  │─────────────────┐
                 └─ check_9_in_row │─────┐
                                    └─ check_trend
```
**Interpretation:** Sequential rule execution, minimal overhead

---

### Pattern 3: "Cliff Drop" (Possible Issue)
```
═════════════════════════════════════════
  [wide box]
  
─────────
  [narrow]
```
**Interpretation:** Sudden performance drop—possible allocation/reallocation boundary

---

### Pattern 4: "Many Thin Layers" (Possible Issue)
```
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
```
**Interpretation:** Deep call stack, function call overhead significant

---

## Performance Profiling Checklist

Use this checklist when analyzing flamegraphs:

- [ ] Is the widest box a expected hot function (rule checks, BLAKE3)?
- [ ] Are rule checks similar width (expected: Trend ≈ Alternating ≈ 2-of-3)?
- [ ] Is BLAKE3 60% for OCEL benchmark? (expected range: 50–70%)
- [ ] Does scaling look linear? (10× data → 10× time?)
- [ ] No surprise allocations? (no `malloc`/`realloc` boxes)
- [ ] Function inlining appears correct? (simple functions not nested)
- [ ] Call graph depth reasonable? (< 20 levels typical)
- [ ] Build is `--release` mode? (check window title)
- [ ] No debug assertions? (would add 5–10% overhead)
- [ ] Consistent across multiple runs? (no high variance)

---

## Flamegraph Comparison (A/B)

To compare two flamegraphs (before/after optimization):

### Step 1: Generate Both Versions

```bash
# Before optimization
git stash
cargo flamegraph --bench quality_western_electric --features quality-monitor
cp flamegraph.svg flamegraph_before.svg

# After optimization
git stash pop
cargo flamegraph --bench quality_western_electric --features quality-monitor
cp flamegraph.svg flamegraph_after.svg
```

### Step 2: Overlay Comparison

Some flamegraph tools support comparison:
```bash
# Using flamegraph-rs with diff
# (Advanced feature, may require custom scripting)
```

### Step 3: Manual Comparison

```
Hotspot Function      Before    After     Improvement
─────────────────────────────────────────────────────
check_1_sigma         10%       8%        20% faster ✓
check_15_in_row       20%       15%       25% faster ✓
BLAKE3 hash           60%       60%       No change (expected)
VecDeque ops          5%        4%        Small gain
```

---

## Tools & Commands Reference

### Generate Flamegraph
```bash
# Basic
cargo flamegraph --bench quality_western_electric --features quality-monitor

# With custom settings
cargo flamegraph --bench quality_western_electric --features quality-monitor -- --profile-time=20

# Save with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
cp flamegraph.svg flamegraph_$TIMESTAMP.svg
```

### View Flamegraph
```bash
# macOS
open flamegraph.svg

# Linux
firefox flamegraph.svg &

# Web browser
python3 -m http.server 8000  # then visit http://localhost:8000/flamegraph.svg
```

### Analyze with perf (Linux)
```bash
# Record
perf record -F 99 --call-graph=dwarf cargo bench --bench quality_western_electric --features quality-monitor

# View
perf report
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

---

## Troubleshooting

### Flamegraph Not Generated
- **Cause:** `flamegraph` binary not installed
- **Solution:** `cargo install flamegraph`

### Flamegraph is Blank/Mostly Idle
- **Cause:** Benchmark too short, not enough samples
- **Solution:** Use `--profile-time=20` to extend profiling

### Can't Install Flamegraph (macOS)
- **Cause:** dtrace requires system permissions
- **Solution:** `sudo` may be required, or use Linux VM

### Flamegraph Shows Nothing Related to Benchmark
- **Cause:** Wrong binary or feature flags
- **Solution:** Verify with `cargo build --benches --features quality-monitor --verbose`

---

## References

- [Flamegraph.pl](http://www.brendangregg.com/flamegraphs.html) — Original concept
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) — Rust wrapper
- [perf](https://perf.wiki.kernel.org/index.php/Main_Page) — Linux profiler
- [Brendan Gregg's Blog](http://www.brendangregg.com/) — Performance tuning

---

**Last Updated:** 2026-06-17  
**Benchmark:** quality_western_electric  
**Expected Runtime:** 30–60 seconds (including profiling overhead)
