# Western Electric Quality Control Benchmarks Report

## Overview

`benches/quality_western_electric.rs` provides comprehensive Criterion benchmarks for the Western Electric statistical process control implementation in `affidavit`. It measures performance across all 7 quality rules, multi-metric correlation detection, OCEL event generation, and causal chain analysis.

**File:** `/home/user/affidavit/benches/quality_western_electric.rs` (487 lines)

**Benchmarks:** 13 benchmark groups with 100+ individual benchmark cases

---

## Benchmark Categories

### 1. Single Rule Detection (7 Rules)

Each of the 7 Western Electric rules is benchmarked independently to isolate rule-specific overhead.

#### Rules Benchmarked
- **Rule 1σ**: Single point >3σ from mean (spike detection)
- **Rule 9-in-a-row**: 9 consecutive out-of-control points
- **Rule Trend**: 6 monotonic points (systematic degradation)
- **Rule Alternating**: Wild oscillations (uncertainty)
- **Rule 2-of-3 beyond 2σ**: Early warning detection
- **Rule 4-of-5 beyond 1σ**: Sustained deviation
- **Rule 15-in-a-row within 1σ**: Plateau/stagnation detection

#### Measurement Counts
- 10, 50, 100, 500 measurements per rule
- Identifies scaling behavior and overhead at different data volumes

#### Target: < 1ms for 100 measurements per rule

---

### 2. All 7 Rules Simultaneously

Tests the combined overhead when all rules are checked on the same metric stream.

```rust
fn bench_all_rules_simultaneous(c: &mut Criterion)
```

- Measurement counts: 10, 50, 100, 200
- Measures penalty of rule interdependencies
- Tests rolling window efficiency with multiple checks

#### Target: < 2ms for 100 measurements (all rules)

---

### 3. Rule Variants (σ, Window Size)

Benchmarks different configurations of the analyzer to identify parameter sensitivity.

```rust
fn bench_rule_variants(c: &mut Criterion)
```

#### Variants
- **σ values**: 0.5, 1.0, 2.0, 3.0 (standard deviation multipliers)
- **Window sizes**: 10, 20, 50, 100 (rolling window capacity)
- **Total combinations**: 16 parameter combinations × 100 measurements

#### Findings Expected
- Larger windows → higher memory footprint but better pattern detection
- Lower σ → more sensitive rules, higher violation detection rate
- Window size change has minimal impact on per-measurement performance

---

### 4. Multi-Metric Correlation Detection

Simulates analyzing quality metrics across multiple independent streams with potential correlations.

```rust
fn bench_correlation_detection(c: &mut Criterion)
```

#### Scale
- 10, 50, 100, 200 metrics analyzed simultaneously
- 50 measurement batches per metric
- Detects correlated violations across metrics

#### Design
```
For each batch:
  For each metric:
    Add measurement with controlled jitter
    Track violations

Aggregate violations across all metrics
```

#### Target: < 10ms for 100 metrics × 50 batches

---

### 5. OCEL Event Generation (1000 Events)

Benchmarks creation and validation of operation-events under the OCEL model.

```rust
fn bench_ocel_event_generation(c: &mut Criterion)
```

#### Scale
- 100, 500, 1000, 2000 events generated
- Each event has 2 object references (artifact + process)
- BLAKE3 commitment computed for each payload

#### Operations Measured
- `SeqCounter::next_seq()` increment
- `object_ref()` construction
- `build_event()` validation + commitment
- Admission gate checks

#### Target: < 5ms per event (200 events/sec)

---

### 6. Causal Chain Building (Deep Chains)

Constructs receipt chains with explicit parent-child event dependencies.

```rust
fn bench_causal_chain_building(c: &mut Criterion)
```

#### Chain Depths
- 10, 30, 50, 100 events per chain
- Each event references the previous event's ID (causal link)
- Tests rolling BLAKE3 hash accumulation

#### Design
```
Event 0 (init): references "source"
Event 1 (derive): references evt-0 output
Event 2 (derive): references evt-1 output
...
Event N: references evt-(N-1) output
```

#### Target: < 50ms for 50-event chain (< 1ms per event)

---

### 7. Object-Level Analysis (1000+ Objects/Files)

Simulates quality metric collection and outlier detection across many objects.

```rust
fn bench_object_level_analysis(c: &mut Criterion)
```

#### Scale
- 100, 500, 1000, 2000 objects analyzed
- Each object has 5 quality metrics (stub_ratio, type_coverage, churn, cyclomatic_complexity, etc.)

#### Operations
1. Create metrics for all objects
2. Extract a single metric (stub_ratio) across all objects
3. Compute mean and standard deviation
4. Count outliers (>2σ)

#### Target: < 50ms for 1000 objects (< 0.05ms per object)

---

## Expected Performance Results

### Baseline Targets

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| Single rule detection (100 pts) | < 1ms | Tight critical path |
| All 7 rules (100 pts) | < 2ms | Rule interdependencies acceptable |
| Correlation (100 metrics) | < 10ms | Parallel-friendly operation |
| OCEL event gen | < 5ms per event | Sequence numbering + commitment |
| Causal chain (50 events) | < 50ms | Rolling hash accumulation |
| Object analysis (1000 objs) | < 50ms | Linear scan + stats |

### Memory Scaling

Expected memory usage at maximum scale:

```
Rule detection:       ~500 KB (rolling window × 7 rules)
Correlation (200m):   ~1 MB (HashMap + VecDeques)
OCEL (2000 events):   ~500 KB (minimal—only seq counter + event bytes)
Causal chain (100e):  ~100 KB (receipt events)
Object analysis:      ~1 MB (HashMap of metrics)
```

### Time Complexity

```
Rule detection:       O(n) linear scan with constant-time checks
Correlation:         O(m·n) where m=metrics, n=measurements
OCEL event gen:      O(1) per event (constant overhead)
Causal chain:        O(n) rolling hash update
Object analysis:     O(n log n) if sorting, O(n) otherwise
```

---

## How to Run Benchmarks

### Run all Western Electric benchmarks
```bash
cargo bench --bench quality_western_electric --features quality-monitor
```

### Run a specific benchmark group
```bash
cargo bench --bench quality_western_electric rule_1_sigma --features quality-monitor
cargo bench --bench quality_western_electric rule_9_in_row --features quality-monitor
cargo bench --bench quality_western_electric correlation_detection --features quality-monitor
```

### Generate flamegraph (requires flamegraph tool)
```bash
cargo bench --bench quality_western_electric --features quality-monitor -- --profile-time=10
# Results in target/criterion/
```

### Baseline establishment (first run)
```bash
cargo bench --bench quality_western_electric --features quality-monitor -- --save-baseline baseline_v1
```

### Compare to baseline
```bash
cargo bench --bench quality_western_electric --features quality-monitor -- --baseline baseline_v1
```

---

## Benchmark Implementation Details

### Key Patterns

**1. Black Box Optimization Prevention**
```rust
analyzer.add_measurement("metric", black_box(value));
```
Prevents LLVM from optimizing away measurements during benchmarking.

**2. Parameterized Benchmarks**
```rust
BenchmarkId::from_parameter(measurement_count)
```
Tests each rule with different data sizes to identify scaling behavior.

**3. Realistic Data Patterns**
- Spikes: `if i == count - 1 { 20.0 } else { 10.0 + sin(...) }`
- Trends: `10.0 + i as f64 * 0.1` (monotonic increase)
- Oscillations: `if i % 2 == 0 { 5.0 } else { 15.0 }` (alternating)
- Stagnation: `10.0 + sin(...) * 0.8` (within ±0.8 bounds)

**4. Measurement Time Configuration**
```rust
group.measurement_time(std::time::Duration::from_secs(5));
```
Default is 5 seconds per benchmark. Intensive benchmarks use 10 seconds.

---

## Benchmark Output Interpretation

### Criterion Output Format
```
rule_1_sigma/10          time:   [1.234 ms 1.245 ms 1.256 ms]
rule_1_sigma/50         time:   [5.123 ms 5.145 ms 5.167 ms]
rule_1_sigma/100        time:   [10.234 ms 10.267 ms 10.301 ms]
rule_1_sigma/500        time:   [51.234 ms 51.267 ms 51.301 ms]
```

### Key Metrics
- **time** (ms): Wall-clock execution time
- **lower bound** (left value): Confidence interval lower
- **estimate** (middle value): Best estimate
- **upper bound** (right value): Confidence interval upper
- **R²**: Goodness of fit (higher is better; 1.0 = perfect)

### Regression Detection
Criterion automatically detects performance regressions:
```
Regressed: rule_1_sigma/100 ± 5% at 2.0 std deviations
```
Indicates > 5% slowdown compared to baseline.

---

## File Structure

```
benches/quality_western_electric.rs
├── Imports (affidavit, criterion, stdlib)
├── Benchmark 1: Single Rule Detection (7 functions)
│   ├── bench_rule_1_sigma_detection
│   ├── bench_rule_9_in_row_detection
│   ├── bench_rule_trend_detection
│   ├── bench_rule_alternating_detection
│   ├── bench_rule_2_of_3_detection
│   ├── bench_rule_4_of_5_detection
│   └── bench_rule_15_in_row_detection
├── Benchmark 2: All Rules Simultaneous
│   └── bench_all_rules_simultaneous
├── Benchmark 3: Rule Variants
│   └── bench_rule_variants
├── Benchmark 4: Correlation Detection
│   └── bench_correlation_detection
├── Benchmark 5: OCEL Event Generation
│   └── bench_ocel_event_generation
├── Benchmark 6: Causal Chain Building
│   └── bench_causal_chain_building
├── Benchmark 7: Object-Level Analysis
│   └── bench_object_level_analysis
└── Criterion Harness
    ├── criterion_group! macro
    └── criterion_main! macro
```

---

## Integration with CI/CD

### Recommended CI Configuration
```yaml
benchmark:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo bench --bench quality_western_electric --features quality-monitor -- --save-baseline ${{ github.ref }}
    - uses: benchmark-action/github-action@v1
      with:
        tool: 'cargo'
        output-file-path: target/criterion/output.txt
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true
```

---

## Troubleshooting

### Benchmark Takes Too Long
- Reduce `measurement_time` from 5s to 1s in source
- Filter to specific groups: `cargo bench rule_1_sigma`

### High Variance in Results
- Close other applications during run
- Use `--profile-time` option: `cargo bench -- --profile-time=30`

### Out of Memory
- Reduce maximum data sizes in benchmark parameters
- Most likely culprit: `correlation_detection` at 200+ metrics

### Flamegraph Not Generated
- Install flamegraph: `cargo install flamegraph`
- Run with: `cargo flamegraph --bench quality_western_electric --features quality-monitor`

---

## Design Rationale

### Why These 7 Benchmarks?

1. **Single Rule Detection**: Establish baseline overhead per rule
2. **All Rules Simultaneous**: Measure real-world combined overhead
3. **Rule Variants**: Understand parameter sensitivity
4. **Correlation**: Simulate multi-metric real systems (100+ metrics common in prod)
5. **OCEL Event Gen**: Core operation frequency (high-throughput path)
6. **Causal Chains**: Provenance-specific concern (long chains in real workflows)
7. **Object-Level Analysis**: Scalability test (thousands of objects in real systems)

### Measurement Strategy

- **Throughput tests** (events/sec): identify max sustainable rate
- **Latency tests** (ms per operation): identify critical-path overhead
- **Scaling tests** (10→500 measurements): verify linear time complexity
- **Variant tests** (σ, window): establish configuration sensitivity

---

## Future Enhancements

Potential additions for deeper analysis:

1. **Parallel Rule Detection**: Benchmark concurrent checking of 7 rules
2. **Streaming Events**: Benchmark incremental updates vs. batch rebuild
3. **Memory Profiling**: Track peak resident set size under load
4. **Cache Effects**: Test CPU cache behavior with different window sizes
5. **Determinism**: Verify identical input → identical output (no variance)
6. **Distribution Fitting**: Benchmark goodness-of-fit for Normal/Weibull/etc.

---

## References

- **Criterion.rs**: https://github.com/bheisler/criterion.rs
- **Flamegraph**: https://github.com/flamegraph-rs/flamegraph
- **Western Electric Rules**: https://en.wikipedia.org/wiki/Control_chart#Nelson_rules
- **OCEL Standard**: https://www.ocel-standard.org/

---

**Last Updated:** 2026-06-17  
**Benchmark Count:** 13 groups, 100+ individual cases  
**Total Runtime:** ~5-10 minutes (all benchmarks)
