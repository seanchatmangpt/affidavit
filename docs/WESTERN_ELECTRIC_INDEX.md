# Western Electric Rules Documentation Index

This directory contains comprehensive documentation for the Western Electric statistical process control rules as implemented in the affidavit provenance layer.

## Documents

### [WESTERN_ELECTRIC_COMPLETE.md](./WESTERN_ELECTRIC_COMPLETE.md) ⭐ START HERE

**The complete reference guide** (~3,200 words, 1,224 lines)

- **Section 1:** Western Electric Rules Explained (280 lines)
  - Historical context from Bell Labs (1950s)
  - Baseline fundamentals (mean, σ, control limits)
  - Detailed explanation of all 7 rules with examples
  - Rule severity and interpretation guide
  - Decision tree for which rule fires when

- **Section 2:** OCEL Mapping (130 lines)
  - Object-Centric Event Logs integration
  - Quality metrics as OCEL objects
  - Event types and relationships
  - Example OCEL quality chain (JSON)
  - Causality modeling

- **Section 3:** Rule Variants & Tuning (220 lines)
  - Tunable parameters for each rule
  - STRICT vs. RELAXED vs. AGGRESSIVE configurations
  - Tuning guide for σ levels and window sizes
  - Sensitivity vs. false positive trade-offs
  - Baseline recommendations by metric

- **Section 4:** Multi-Dimensional Analysis (220 lines)
  - Correlation patterns in code quality
  - Simultaneous violations and amplification
  - Root cause inference from metric correlations
  - Causal chain analysis (upstream → violation → downstream)
  - 5-step decision framework

- **Section 5:** Diagrams & Visualizations (100 lines)
  - Western Electric rule decision tree (Mermaid)
  - OCEL quality event model (Mermaid)
  - Correlation matrix heatmap (ASCII)
  - Causal chain example (Mermaid)

- **Quick Reference:** Summary tables and appendix
  - Rule summary table
  - Metric baseline recommendations
  - Severity to action mapping
  - Rust implementation reference

## Quick Navigation

### By Use Case

**"I want to understand the 7 rules"**
→ Section 1.3 (Rule Definitions & Interpretations)

**"I need to tune parameters for my team"**
→ Section 3.2 (Tuning Guide)

**"Multiple violations are firing; what does it mean?"**
→ Section 4 (Multi-Dimensional Analysis)

**"How do I integrate with OCEL?"**
→ Section 2 (OCEL Mapping)

**"I need a quick reference"**
→ Quick Reference section

### By Metric

All metrics are covered in:
- **Section 3.2:** Tuning σ Levels Table
- **Quick Reference:** Metric Baseline Recommendations

Metrics included:
- stub_ratio
- test_coverage
- cyclomatic_complexity
- maintainability_index
- clippy_warnings
- doc_coverage

## Implementation Reference

The Western Electric rules are implemented in **`src/quality.rs`**:

```rust
pub struct WesternElectricAnalyzer {
    pub baseline_mean: f64,
    pub baseline_stddev: f64,
    pub rolling_window: VecDeque<f64>,
    pub window_size: usize,
    pub control_limits: (f64, f64),
    pub violations: Vec<QualityViolation>,
}
```

See Section 5 (Appendix) for implementation details.

## Key Concepts

### The 7 Rules (Quick Summary)

| Rule | Name | Window | Trigger | Severity |
|------|------|--------|---------|----------|
| 1 | Spike Detection | 1 | abs(z) > 3.0 | CRITICAL |
| 2 | 9-in-a-Row (Zombie) | 9 | count≥9 OOC | CRITICAL |
| 3 | Trend | 6 | 6 monotonic | HIGH |
| 4 | Alternating | 8 | 7+ crossings | HIGH |
| 5 | 2-of-3@2σ | 3 | count≥2 | HIGH |
| 6 | 4-of-5@1σ | 5 | count≥4 | MEDIUM |
| 7 | 15-in-Row@1σ | 15 | all 15 | INFO |

### Severity to Action

- **CRITICAL** (6+ rules or Rule 1/2) → Stop merges; emergency review (now)
- **HIGH** (3–5 rules) → Code review meeting (today)
- **MEDIUM** (2 rules) → Assign + monitor (this week)
- **INFO** (1 rule or Rule 7) → Monitor; auto-correct likely (next review)

### Correlation Patterns

Strong negative correlations:
- stub_ratio ↑ → test_coverage ↓ (r = −0.82)
- cyclomatic ↑ → maintainability ↓ (r = −0.88)

See Section 4.1 for full correlation matrix.

## Examples Provided

### Practical Examples in Document

1. **Baseline calculation** (Section 1.2)
   - stub_ratio μ=0.025, σ=0.011

2. **Rule 1 scenario** (Section 1.3)
   - Sudden spike in test coverage drop

3. **Rule 3 scenario** (Section 1.3)
   - Cyclomatic complexity increasing over 6 commits

4. **OCEL JSON example** (Section 2.3)
   - Full quality event chain with objects and events

5. **Correlation patterns** (Section 4.3)
   - 4 real-world scenarios with diagnosis

6. **Causal chain** (Section 4.4)
   - Root cause inference flowchart

## Tuning Recommendations

### By Team Velocity

- **High-velocity** (20+ commits/day) → window_size = 7–10
- **Standard** (5–10 commits/day) → window_size = 15–20
- **Low-velocity** (1–5 commits/day) → window_size = 25–30
- **New project** → window_size = 20–30 (conservative)

### By Sensitivity Preference

- **STRICT** → Higher σ thresholds, larger windows (fewer false positives, slower detection)
- **BALANCED** → Standard WE parameters + Rule 5 as early warning (recommended)
- **RELAXED** → Lower σ thresholds, smaller windows (faster detection, more alerts)

## Integration with Affidavit

Quality measurements can be emitted to the provenance receipt chain:

```bash
# Measure quality
affi emit --type quality-measured \
  --object metric:stub_ratio:main \
  --payload '{"value": 0.028, "z-score": 0.27}'

# Assemble receipt
affi assemble --working-dir .affi

# Verify and check for violations
affi verify receipt-latest.json --format json
```

See Section 2.5 for OCEL mapping rules.

## Testing & Validation

The implementation includes 30+ unit tests:
- 19 tests in `src/quality.rs` (rule correctness)
- 6 tests in `src/handlers.rs` (event dispatch)
- 4 E2E tests in `tests/`
- 1 UI test for CLI

See `tests/quality_monitor.rs` for integration test patterns.

## References

- **Western Electric Rules Origin:** Bell Labs (1950s), manufacturing SPC
- **OCEL Standard:** Object-Centric Event Logs v1.0 (https://www.ocel-standard.org/)
- **Implementation:** `src/quality.rs` in affidavit
- **Related:** CLAUDE.md, README.md

## FAQ

**Q: Which rule should I enable first?**
A: Enable all 7. Start with the standard configuration (Section 3.1). Rules 1–4 detect acute problems; Rules 5–7 detect emerging issues.

**Q: How often should baselines be recalibrated?**
A: Every 100–200 commits or quarterly, whichever is sooner. Document each recalibration as a `baseline_computed` event in OCEL.

**Q: Can I use this for non-code metrics?**
A: Yes. The framework works for any continuous metric: deployment frequency, incident count, feature delivery time, etc.

**Q: What if my σ is very large?**
A: Indicates high natural variance in the metric. Consider whether the metric is too noisy, or whether your process is genuinely unpredictable. Increase window size before increasing σ threshold.

**Q: How do I know if Rule 7 is good or bad?**
A: Context-dependent. Stable high test coverage = good. Stable low maintainability = bad. Check if the plateau is at your target level.

---

**Version:** 1.0  
**Last Updated:** 2026-06-17  
**License:** MIT OR Apache-2.0
