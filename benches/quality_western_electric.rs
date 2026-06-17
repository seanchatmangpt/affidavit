//! Criterion benchmarks for Western Electric quality control rules.
//!
//! Benchmarks the full statistical process control pipeline including:
//! - Single rule detection (7 rules × N measurements)
//! - All 7 rules simultaneously
//! - Rule variants (different σ thresholds, window sizes)
//! - Correlation detection (100+ metrics)
//! - OCEL event generation (1000 events)
//! - Causal chain building (deep dependency chains)
//! - Object-level analysis (1000+ files/objects)
//!
//! Targets:
//! - Rule detection: < 1ms for 100 measurements
//! - Correlation: < 10ms for 100 metrics
//! - OCEL event gen: < 5ms per event (= 200 events/sec)
//! - Causal chain: < 50ms for 50-event chain
//!
//! Run with: cargo bench --bench quality_western_electric
//! Flamegraph: cargo bench --bench quality_western_electric -- --profile-time=10

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::quality::{CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;

// ============================================================================
// Benchmark 1: Single Rule Detection (Per Rule)
// ============================================================================

fn bench_rule_1_sigma_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_1_sigma");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(2.0), 100);
                    for i in 0..count {
                        let value = if i == count - 1 {
                            20.0 // Spike at the end
                        } else {
                            10.0 + (i as f64 * 0.01).sin()
                        };
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_9_in_row_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_9_in_row");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(2.0), 100);
                    for _ in 0..count {
                        analyzer.add_measurement("test_metric", black_box(20.0));
                        // All OOC
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_trend_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_trend");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(2.0), 100);
                    for i in 0..count {
                        let value = 10.0 + i as f64 * 0.1; // Monotonic increase
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_alternating_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_alternating");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(2.0), 100);
                    for i in 0..count {
                        let value = if i % 2 == 0 { 5.0 } else { 15.0 };
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_2_of_3_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_2_of_3_beyond_2sigma");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(1.0), 100);
                    for i in 0..count {
                        let value = if i % 3 < 2 { 13.0 } else { 10.5 };
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_4_of_5_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_4_of_5_beyond_1sigma");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(1.0), 100);
                    for i in 0..count {
                        let value = if i % 5 < 4 { 11.5 } else { 10.2 };
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

fn bench_rule_15_in_row_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_15_in_row_within_1sigma");
    group.measurement_time(std::time::Duration::from_secs(5));

    for measurement_count in [20, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(1.0), 100);
                    for i in 0..count {
                        let value = 10.0 + (i as f64 * 0.02).sin() * 0.8; // Within 1σ
                        analyzer.add_measurement("test_metric", black_box(value));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Benchmark 2: All 7 Rules Simultaneously
// ============================================================================

fn bench_all_rules_simultaneous(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_rules_simultaneous");
    group.measurement_time(std::time::Duration::from_secs(10));

    for measurement_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(measurement_count),
            measurement_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzer =
                        WesternElectricAnalyzer::new(black_box(10.0), black_box(1.5), 100);
                    for i in 0..count {
                        // Generate varied pattern to trigger different rules
                        let base =
                            10.0 + (i as f64 * 0.05).sin() + if i % 8 == 0 { 5.0 } else { 0.0 };
                        analyzer.add_measurement("metric", black_box(base));
                    }
                    analyzer.violations.len()
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Benchmark 3: Rule Variants (σ, window size)
// ============================================================================

fn bench_rule_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_variants");
    group.measurement_time(std::time::Duration::from_secs(5));

    let sigmas = vec![0.5, 1.0, 2.0, 3.0];
    let window_sizes = vec![10, 20, 50, 100];

    for sigma in &sigmas {
        for window_size in &window_sizes {
            let param = format!("σ={},window={}", sigma, window_size);
            group.bench_with_input(
                BenchmarkId::from_parameter(&param),
                &(sigma, window_size),
                |b, &(s, w)| {
                    b.iter(|| {
                        let mut analyzer =
                            WesternElectricAnalyzer::new(black_box(10.0), black_box(*s), *w);
                        for i in 0..100 {
                            let value = 10.0 + (i as f64 * 0.02).sin();
                            analyzer.add_measurement("metric", black_box(value));
                        }
                        analyzer.violations.len()
                    })
                },
            );
        }
    }
    group.finish();
}

// ============================================================================
// Benchmark 4: Multi-Metric Correlation Detection
// ============================================================================

fn bench_correlation_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlation_detection");
    group.measurement_time(std::time::Duration::from_secs(10));

    for metric_count in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(metric_count),
            metric_count,
            |b, &count| {
                b.iter(|| {
                    let mut analyzers: HashMap<String, WesternElectricAnalyzer> = HashMap::new();

                    // Initialize analyzers for each metric
                    for i in 0..count {
                        let metric_name = format!("metric_{}", i);
                        analyzers.insert(
                            metric_name,
                            WesternElectricAnalyzer::new(black_box(10.0), black_box(1.5), 50),
                        );
                    }

                    // Feed correlated measurements
                    let measurement_batch = 50;
                    for batch in 0..measurement_batch {
                        for (metric_name, analyzer) in analyzers.iter_mut() {
                            let metric_idx = metric_name
                                .strip_prefix("metric_")
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(0);
                            let base = 10.0 + (batch as f64 * 0.1).sin();
                            let jitter = (metric_idx as f64 * 0.1).sin();
                            let value = base + jitter;
                            analyzer.add_measurement(&metric_name, black_box(value));
                        }
                    }

                    // Collect all violations
                    let total_violations: usize =
                        analyzers.iter().map(|(_, a)| a.violations.len()).sum();
                    total_violations
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Benchmark 5: OCEL Event Generation (1000 events)
// ============================================================================

fn bench_ocel_event_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ocel_event_generation");
    group.measurement_time(std::time::Duration::from_secs(10));

    for event_count in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            event_count,
            |b, &count| {
                b.iter(|| {
                    let mut counter = SeqCounter::new();
                    let mut event_count_result = 0;

                    for i in 0..count {
                        let event_type = format!("operation_{}", i % 10);
                        let objects = vec![
                            object_ref(format!("obj_{}", i), "artifact"),
                            object_ref(format!("proc_{}", i % 50), "process"),
                        ];
                        match build_event(
                            &event_type,
                            objects,
                            black_box(b"event_payload"),
                            &mut counter,
                        ) {
                            Ok(_) => event_count_result += 1,
                            Err(_) => {}
                        }
                    }
                    event_count_result
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Benchmark 6: Causal Chain Building (Dependency Chains)
// ============================================================================

fn bench_causal_chain_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("causal_chain_building");
    group.measurement_time(std::time::Duration::from_secs(10));

    for chain_depth in [10, 30, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(chain_depth),
            chain_depth,
            |b, &depth| {
                b.iter(|| {
                    let mut assembler = ChainAssembler::new();
                    let mut counter = SeqCounter::new();

                    for i in 0..depth {
                        let event_type = if i == 0 { "init" } else { "derive" };
                        let parent_obj = if i > 0 {
                            format!("evt-{}", i - 1)
                        } else {
                            "source".to_string()
                        };
                        let objects = vec![
                            object_ref(&parent_obj, "artifact"),
                            object_ref(format!("output_{}", i), "artifact"),
                        ];

                        match build_event(
                            event_type,
                            objects,
                            black_box(format!("chain_payload_{}", i).as_bytes()),
                            &mut counter,
                        ) {
                            Ok(event) => {
                                let _ = assembler.append(event);
                            }
                            Err(_) => {}
                        }
                    }

                    let receipt = assembler.finalize();
                    receipt.events.len()
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Benchmark 7: Object-Level Analysis (1000+ objects/files)
// ============================================================================

fn bench_object_level_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_level_analysis");
    group.measurement_time(std::time::Duration::from_secs(10));

    for object_count in [100, 500, 1000, 2000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &count| {
                b.iter(|| {
                    // Simulate analyzing quality metrics for many files/objects
                    let mut object_metrics: HashMap<String, CodeQualityMetrics> = HashMap::new();

                    for i in 0..count {
                        let obj_name = format!("file_{}", i);
                        let mut metrics = CodeQualityMetrics::default();
                        metrics.stub_ratio = (i as f64 % 10.0) / 100.0;
                        metrics.type_coverage = 1.0 - (i as f64 % 5.0) / 100.0;
                        metrics.churn = (i * 10) % 500;
                        metrics.cyclomatic_complexity = 2.0 + (i as f64 % 8.0);
                        object_metrics.insert(obj_name, metrics);
                    }

                    // Analyze correlations across objects
                    let mut stub_ratios: Vec<f64> =
                        object_metrics.values().map(|m| m.stub_ratio).collect();
                    stub_ratios
                        .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                    let mean = stub_ratios.iter().sum::<f64>() / stub_ratios.len().max(1) as f64;
                    let variance = stub_ratios.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                        / stub_ratios.len().max(1) as f64;
                    let stddev = variance.sqrt();

                    // Count outliers (>2σ)
                    let outlier_count = stub_ratios
                        .iter()
                        .filter(|x| (x - mean).abs() > 2.0 * stddev)
                        .count();

                    outlier_count
                })
            },
        );
    }
    group.finish();
}

// ============================================================================
// Criterion Harness
// ============================================================================

criterion_group!(
    benches,
    bench_rule_1_sigma_detection,
    bench_rule_9_in_row_detection,
    bench_rule_trend_detection,
    bench_rule_alternating_detection,
    bench_rule_2_of_3_detection,
    bench_rule_4_of_5_detection,
    bench_rule_15_in_row_detection,
    bench_all_rules_simultaneous,
    bench_rule_variants,
    bench_correlation_detection,
    bench_ocel_event_generation,
    bench_causal_chain_building,
    bench_object_level_analysis,
);

criterion_main!(benches);
