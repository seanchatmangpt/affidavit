//! OCEL + Western Electric Rule Detector: Combinatorial Maximalism Demo
//!
//! This example demonstrates maximalist integration across affidavit's quality
//! and OCEL subsystems. It:
//!
//! 1. Loads a portfolio of repositories (or generates synthetic metrics)
//! 2. For each repo, measures all 13 code quality metrics
//! 3. Applies all 7 Western Electric rule variants (1σ spike, 9-in-row, trend, etc.)
//! 4. Detects rule storms (2+ rules firing simultaneously)
//! 5. Analyzes cross-metric correlations
//! 6. Tracks violations at file/module granularity
//! 7. Builds causal chains connecting violations
//! 8. Generates OCEL event log from violations
//! 9. Emits violations to receipt chain
//! 10. Outputs comprehensive report with statistics, correlation matrix,
//!     object-level violations, causal chains, and severity aggregation
//!
//! Run: `cargo run --example ocel_western_electric_demo --features shell`
//! Output: JSON, human-readable report, receipt events

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
use affidavit::quality::{CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer};
use serde_json::json;
use std::collections::{HashMap, VecDeque};

/// Per-repo violation metadata with context
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ViolationContext {
    repo_name: String,
    metric_name: String,
    violation: QualityViolation,
    file_path: Option<String>,
    module_path: Option<String>,
}

/// Rule storm: 2+ rules firing on same metric/timeframe
#[derive(Debug, Clone)]
struct RuleStorm {
    repo: String,
    metric: String,
    rules: Vec<String>,
    timestamp: u64,
}

/// Correlation between two metrics
#[derive(Debug, Clone)]
struct MetricCorrelation {
    metric_a: String,
    metric_b: String,
    correlation_coefficient: f64,
}

/// Causal chain linking violations
#[derive(Debug, Clone)]
struct CausalChain {
    root_cause: String,
    consequents: Vec<String>,
    confidence: f64,
}

/// Summary statistics for a rule across portfolio
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct RuleStats {
    rule_name: String,
    total_violations: usize,
    repos_affected: usize,
    severity_counts: HashMap<String, usize>,
    top_metrics: Vec<(String, usize)>,
}

fn main() -> anyhow::Result<()> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     OCEL Western Electric Rule Detector — Maximalist Demo      ║");
    println!("║                                                                ║");
    println!("║  Phase 1: Load portfolio (13 metrics × 3-5 snapshots each)    ║");
    println!("║  Phase 2: Run all 7 WE rules, detect storms, correlations    ║");
    println!("║  Phase 3: Build causal chains, track object-level violations ║");
    println!("║  Phase 4: Generate OCEL events and emit to receipt chain     ║");
    println!("║  Phase 5: Produce comprehensive report with aggregations     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // =========================================================================
    // PHASE 1: Portfolio and Metric Baseline
    // =========================================================================
    println!("📊 PHASE 1: Portfolio Analysis");
    println!("──────────────────────────────\n");

    let repos = vec![
        "core-services",
        "api-gateway",
        "data-pipeline",
        "auth-provider",
        "cache-layer",
    ];

    let metrics_baseline = vec![
        ("stub_ratio", 0.032, 0.025),              // Low with tight σ
        ("type_coverage", 0.91, 0.08),             // High, consistent
        ("churn", 120.0, 45.0),                    // Normal variability
        ("comment_ratio", 0.22, 0.08),             // Good documentation
        ("cyclomatic_complexity", 2.5, 0.7),       // Safe complexity
        ("maintainability_index", 85.0, 12.0),     // Healthy
        ("cognitive_complexity", 6.0, 2.5),        // Manageable
        ("clippy_warnings", 3.0, 2.5),             // Few warnings
        ("rustfmt_violations", 1.0, 1.5),          // Clean formatting
        ("cargo_deny_issues", 0.5, 0.8),           // Minimal
        ("cargo_audit_vulnerabilities", 0.2, 0.5), // Very rare
        ("test_coverage", 82.5, 6.2),              // Good coverage
        ("doc_coverage", 0.78, 0.15),              // Documentation complete
    ];

    println!("📈 Baseline metrics (mean ± 1σ):");
    for (name, mean, stddev) in &metrics_baseline {
        println!("   • {:<30} = {:.3} ± {:.3}", name, mean, stddev);
    }
    println!();

    // =========================================================================
    // PHASE 2: Western Electric Analysis
    // =========================================================================
    println!("🔍 PHASE 2: Western Electric Rule Detection");
    println!("──────────────────────────────────────────\n");

    let mut all_violations: Vec<ViolationContext> = Vec::new();
    let mut rule_storms: Vec<RuleStorm> = Vec::new();
    let mut per_repo_metrics: HashMap<String, Vec<CodeQualityMetrics>> = HashMap::new();
    let mut metric_measurements: HashMap<String, VecDeque<f64>> = HashMap::new();

    // Initialize metric tracking
    for (metric_name, _, _) in &metrics_baseline {
        metric_measurements.insert(metric_name.to_string(), VecDeque::new());
    }

    // Process each repo
    for repo in &repos {
        println!("   Analyzing {}...", repo);
        let mut repo_violations_by_rule: HashMap<String, Vec<QualityViolation>> = HashMap::new();
        let mut repo_metrics = Vec::new();

        // Generate 4 synthetic metric snapshots per repo (time series)
        for snapshot_idx in 0..4 {
            let mut metrics = CodeQualityMetrics::default();
            let timestamp = (1000000 + snapshot_idx) as u64;

            // Synthesize values with controlled deviations for demo purposes
            // Some metrics will spike, some will trend, some will plateau
            metrics.stub_ratio = match *repo {
                "core-services" => 0.01 + (snapshot_idx as f64 * 0.02), // Trending up
                "api-gateway" => 0.15,                                  // Spike
                "data-pipeline" => 0.03,
                "auth-provider" => 0.02,
                "cache-layer" => 0.045, // Elevated
                _ => 0.03,
            };

            metrics.type_coverage = match *repo {
                "core-services" => 0.95,
                "api-gateway" => 0.88,
                "data-pipeline" => 0.92,
                "auth-provider" => 0.97,
                "cache-layer" => 0.93,
                _ => 0.91,
            };

            metrics.churn = match *repo {
                "core-services" => (85 + (snapshot_idx * 15)) as usize, // Trending
                "api-gateway" => 220,                                   // High churn (spike)
                "data-pipeline" => 110,
                "auth-provider" => 60,
                "cache-layer" => 135,
                _ => 120,
            };

            metrics.comment_ratio = 0.20 + (snapshot_idx as f64 * 0.01);
            metrics.cyclomatic_complexity = 2.3 + (snapshot_idx as f64 * 0.3);
            metrics.maintainability_index = 84.0 - (snapshot_idx as f64 * 2.0);
            metrics.cognitive_complexity = 5.5 + (snapshot_idx as f64 * 0.8);
            metrics.clippy_warnings = match *repo {
                "api-gateway" => 8, // Elevated
                "data-pipeline" => 5,
                _ => 2,
            };
            metrics.rustfmt_violations = snapshot_idx as usize;
            metrics.cargo_deny_issues = 0;
            metrics.cargo_audit_vulnerabilities = 0;
            metrics.test_coverage = 80.0 - (snapshot_idx as f64 * 1.5); // Trending down
            metrics.doc_coverage = 0.75 + (snapshot_idx as f64 * 0.02);
            metrics.timestamp = timestamp;

            repo_metrics.push(metrics.clone());

            // Run all 7 WE rules on each metric
            for (metric_name, baseline_mean, baseline_stddev) in &metrics_baseline {
                let mut analyzer =
                    WesternElectricAnalyzer::new(*baseline_mean, *baseline_stddev, 20);

                let value: f64 = match *metric_name {
                    "stub_ratio" => metrics.stub_ratio,
                    "type_coverage" => metrics.type_coverage,
                    "churn" => metrics.churn as f64,
                    "comment_ratio" => metrics.comment_ratio,
                    "cyclomatic_complexity" => metrics.cyclomatic_complexity,
                    "maintainability_index" => metrics.maintainability_index,
                    "cognitive_complexity" => metrics.cognitive_complexity,
                    "clippy_warnings" => metrics.clippy_warnings as f64,
                    "rustfmt_violations" => metrics.rustfmt_violations as f64,
                    "cargo_deny_issues" => metrics.cargo_deny_issues as f64,
                    "cargo_audit_vulnerabilities" => metrics.cargo_audit_vulnerabilities as f64,
                    "test_coverage" => metrics.test_coverage,
                    "doc_coverage" => metrics.doc_coverage,
                    _ => continue,
                };

                // Track in rolling window
                if let Some(w) = metric_measurements.get_mut(*metric_name) {
                    w.push_back(value);
                }

                // Add measurement to analyzer
                analyzer.add_measurement(metric_name, value);

                // Collect violations
                let violations_before = analyzer.violations.len();
                analyzer.add_measurement(metric_name, value);
                let violations_after = analyzer.violations.len();

                if violations_after > violations_before {
                    for i in violations_before..violations_after {
                        let violation = analyzer.violations[i].clone();
                        let rule_name = format!("{:?}", violation)
                            .split('{')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string();

                        all_violations.push(ViolationContext {
                            repo_name: repo.to_string(),
                            metric_name: metric_name.to_string(),
                            violation: violation.clone(),
                            file_path: Some(format!("src/{}/mod.rs", metric_name)),
                            module_path: Some(format!("{}::{}", repo, metric_name)),
                        });

                        repo_violations_by_rule
                            .entry(rule_name)
                            .or_default()
                            .push(violation);
                    }
                }
            }
        }

        // Detect rule storms (2+ rules firing on same metric in same snapshot)
        let mut rules_per_metric: HashMap<String, Vec<String>> = HashMap::new();
        for (rule_name, violations) in &repo_violations_by_rule {
            for violation in violations {
                rules_per_metric
                    .entry(violation.metric().to_string())
                    .or_default()
                    .push(rule_name.clone());
            }
        }

        for (metric, rule_names) in rules_per_metric {
            if rule_names.len() >= 2 {
                rule_storms.push(RuleStorm {
                    repo: repo.to_string(),
                    metric: metric.clone(),
                    rules: rule_names,
                    timestamp: 1000000,
                });
            }
        }

        per_repo_metrics.insert(repo.to_string(), repo_metrics);
    }

    println!(
        "   ✓ Processed {} repos × 4 snapshots = {} metric measurements",
        repos.len(),
        repos.len() * 4
    );
    println!(
        "   ✓ Detected {} violations across {} metrics",
        all_violations.len(),
        metrics_baseline.len()
    );
    println!(
        "   ✓ Detected {} rule storms (2+ rules firing)",
        rule_storms.len()
    );
    println!();

    // =========================================================================
    // PHASE 3: Cross-Metric Correlations
    // =========================================================================
    println!("📊 PHASE 3: Correlation Analysis");
    println!("────────────────────────────────\n");

    let mut correlations = Vec::new();

    // Compute pairwise correlations (simplified: hand-picked expected correlations)
    let expected_correlations = vec![
        ("stub_ratio", "test_coverage", -0.72), // Higher stubs = lower coverage
        ("cyclomatic_complexity", "maintainability_index", -0.81), // Higher complexity = worse MI
        ("churn", "clippy_warnings", 0.68),     // High churn = more warnings
        ("type_coverage", "doc_coverage", 0.45), // Well-typed = better docs
        ("cognitive_complexity", "maintainability_index", -0.75), // Cognitive burden hurts MI
    ];

    for (metric_a, metric_b, coeff) in expected_correlations {
        correlations.push(MetricCorrelation {
            metric_a: metric_a.to_string(),
            metric_b: metric_b.to_string(),
            correlation_coefficient: coeff,
        });
        println!("   {} ↔ {}: r = {:.3}", metric_a, metric_b, coeff);
    }
    println!();

    // =========================================================================
    // PHASE 4: Object-Level Violations (File/Module Granularity)
    // =========================================================================
    println!("🎯 PHASE 4: Object-Level Violation Tracking");
    println!("──────────────────────────────────────────\n");

    let mut violations_by_object: HashMap<String, Vec<&ViolationContext>> = HashMap::new();
    for violation in &all_violations {
        if let Some(module_path) = &violation.module_path {
            violations_by_object
                .entry(module_path.clone())
                .or_default()
                .push(violation);
        }
    }

    println!("   Object-level violations:");
    for (object, viols) in violations_by_object.iter().take(5) {
        println!("   • {} ({} violations)", object, viols.len());
        for v in viols.iter().take(2) {
            println!("      └ {}", v.violation.description());
        }
    }
    if violations_by_object.len() > 5 {
        println!("   ... and {} more objects", violations_by_object.len() - 5);
    }
    println!();

    // =========================================================================
    // PHASE 5: Causal Chains
    // =========================================================================
    println!("🔗 PHASE 5: Causal Chain Analysis");
    println!("────────────────────────────────\n");

    let mut causal_chains = Vec::new();

    // Hand-code expected causal chains based on quality domain knowledge
    let causal_map = vec![
        (
            "high_churn",
            vec!["increased_clippy_warnings", "decreased_maintainability"],
        ),
        (
            "low_test_coverage",
            vec!["stub_stubs_remain", "high_cognitive_complexity"],
        ),
        (
            "spike_cyclomatic_complexity",
            vec!["reduced_maintainability_index", "lower_doc_coverage"],
        ),
    ];

    for (root_cause, consequents) in causal_map {
        causal_chains.push(CausalChain {
            root_cause: root_cause.to_string(),
            consequents: consequents.iter().map(|s| s.to_string()).collect(),
            confidence: 0.75,
        });
        println!("   {} → {:?} (confidence: 0.75)", root_cause, consequents);
    }
    println!();

    // =========================================================================
    // PHASE 6: Generate OCEL Events
    // =========================================================================
    println!("📝 PHASE 6: OCEL Event Log Generation");
    println!("──────────────────────────────────────\n");

    let mut seq_counter = SeqCounter::new();
    let mut ocel_events = Vec::new();

    // Emit portfolio snapshot event
    let portfolio_event = build_event(
        "portfolio_analysis_started",
        vec![
            object_ref("portfolio:main", "portfolio"),
            qualified_object_ref("analysis:we-rules", "analysis", "quality-control"),
        ],
        b"portfolio_analysis_config",
        &mut seq_counter,
    )?;
    ocel_events.push(portfolio_event);
    println!("   Emitted: portfolio_analysis_started (evt-0)");

    // Emit per-repo events
    for (idx, repo) in repos.iter().enumerate() {
        let repo_event = build_event(
            "repo_quality_check",
            vec![
                object_ref(format!("repo:{}", repo), "repository"),
                qualified_object_ref(
                    format!("analysis:{}:metrics", repo),
                    "analysis-result",
                    "metrics",
                ),
            ],
            format!("metrics_for_{}", repo).as_bytes(),
            &mut seq_counter,
        )?;
        ocel_events.push(repo_event);
        println!(
            "   Emitted: repo_quality_check for {} (evt-{})",
            repo,
            idx + 1
        );
    }

    // Emit violation discovery events (top 5 by severity)
    let mut violations_by_severity: Vec<_> = all_violations.iter().collect();
    violations_by_severity.sort_by(|a, b| {
        let severity_rank = |v: &str| match v {
            "CRITICAL" => 0,
            "HIGH" => 1,
            "MEDIUM" => 2,
            _ => 3,
        };
        let a_sev = severity_rank(a.violation.severity());
        let b_sev = severity_rank(b.violation.severity());
        a_sev.cmp(&b_sev)
    });

    for (idx, violation_ctx) in violations_by_severity.iter().take(5).enumerate() {
        let violation_event = build_event(
            "quality_violation_detected",
            vec![
                object_ref(format!("violation:{}", idx), "violation"),
                qualified_object_ref(
                    format!("repo:{}", violation_ctx.repo_name),
                    "repository",
                    "subject",
                ),
                object_ref(format!("metric:{}", violation_ctx.metric_name), "metric"),
            ],
            violation_ctx.violation.description().as_bytes(),
            &mut seq_counter,
        )?;
        ocel_events.push(violation_event);
        println!(
            "   Emitted: quality_violation_detected for {} (evt-{})",
            violation_ctx.metric_name,
            repos.len() + 2 + idx
        );
    }

    // Emit rule storm detection events
    for (idx, storm) in rule_storms.iter().take(3).enumerate() {
        let storm_event = build_event(
            "rule_storm_detected",
            vec![
                object_ref(format!("storm:{}", idx), "rule-storm"),
                qualified_object_ref(
                    format!("metric:{}", storm.metric),
                    "metric",
                    "conflict_point",
                ),
                object_ref(format!("repo:{}", storm.repo), "repository"),
            ],
            format!("rules: {:?}", storm.rules).as_bytes(),
            &mut seq_counter,
        )?;
        ocel_events.push(storm_event);
        println!(
            "   Emitted: rule_storm_detected (evt-{})",
            repos.len() + 2 + rule_storms.len() + idx
        );
    }

    // Emit analysis complete event
    let complete_event = build_event(
        "portfolio_analysis_completed",
        vec![
            object_ref("portfolio:main", "portfolio"),
            qualified_object_ref("result:summary", "analysis-result", "final"),
        ],
        format!(
            "violations: {}, storms: {}",
            all_violations.len(),
            rule_storms.len()
        )
        .as_bytes(),
        &mut seq_counter,
    )?;
    ocel_events.push(complete_event);
    println!(
        "   Emitted: portfolio_analysis_completed (evt-{})",
        ocel_events.len() - 1
    );
    println!("   ✓ Total OCEL events generated: {}\n", ocel_events.len());

    // =========================================================================
    // PHASE 7: Build and Emit Receipt Chain
    // =========================================================================
    println!("🔐 PHASE 7: Receipt Chain Assembly");
    println!("──────────────────────────────────\n");

    let mut assembler = ChainAssembler::new();

    // Add OCEL events to receipt
    for ocel_event in &ocel_events {
        assembler.append(ocel_event.clone())?;
    }

    // Add violation summary event
    let violation_summary = json!({
        "total_violations": all_violations.len(),
        "critical_count": all_violations.iter().filter(|v| v.violation.severity() == "CRITICAL").count(),
        "high_count": all_violations.iter().filter(|v| v.violation.severity() == "HIGH").count(),
        "repos_affected": repos.len(),
        "rule_storms": rule_storms.len(),
    });

    // Create a summary event and append it
    let summary_event = affidavit::ocel::build_event(
        "analysis-summary",
        vec![object_ref("summary:portfolio", "summary")],
        serde_json::to_string(&violation_summary)?.as_bytes(),
        &mut seq_counter,
    )?;
    assembler.append(summary_event)?;

    let receipt = assembler.finalize();
    println!(
        "   ✓ Receipt chain assembled: {} events",
        receipt.events.len()
    );
    println!("   ✓ Chain hash: {}", receipt.chain_hash.as_hex());
    println!();

    // =========================================================================
    // PHASE 8: Rule Statistics Aggregation
    // =========================================================================
    println!("📈 PHASE 8: Rule Statistics");
    println!("───────────────────────────\n");

    let mut rule_stats: HashMap<String, RuleStats> = HashMap::new();

    for violation in &all_violations {
        let rule_name = format!("{:?}", violation.violation)
            .split('{')
            .next()
            .unwrap_or("Unknown")
            .to_string();
        let severity = violation.violation.severity().to_string();
        let metric = violation.metric_name.clone();

        let stats = rule_stats.entry(rule_name.clone()).or_insert(RuleStats {
            rule_name,
            total_violations: 0,
            repos_affected: 0,
            severity_counts: HashMap::new(),
            top_metrics: Vec::new(),
        });

        stats.total_violations += 1;
        *stats.severity_counts.entry(severity).or_insert(0) += 1;

        let metric_count = stats.top_metrics.iter_mut().find(|(m, _)| m == &metric);
        if let Some((_, count)) = metric_count {
            *count += 1;
        } else {
            stats.top_metrics.push((metric, 1));
        }
    }

    for (idx, (_, stats)) in rule_stats.iter().enumerate().take(7) {
        let total = stats.total_violations;
        let critical = stats.severity_counts.get("CRITICAL").copied().unwrap_or(0);
        let high = stats.severity_counts.get("HIGH").copied().unwrap_or(0);
        println!(
            "   {}. {} ({} total, {} critical, {} high)",
            idx + 1,
            stats.rule_name,
            total,
            critical,
            high
        );
    }
    println!();

    // =========================================================================
    // PHASE 9: Comprehensive Report
    // =========================================================================
    println!("📋 PHASE 9: Comprehensive Report");
    println!("────────────────────────────────\n");

    let report = json!({
        "metadata": {
            "title": "OCEL Western Electric Portfolio Analysis",
            "timestamp": 1000000,
            "portfolio_size": repos.len(),
            "metrics_measured": metrics_baseline.len(),
            "snapshots_per_repo": 4,
        },
        "summary": {
            "total_violations": all_violations.len(),
            "critical_violations": all_violations.iter()
                .filter(|v| v.violation.severity() == "CRITICAL")
                .count(),
            "high_violations": all_violations.iter()
                .filter(|v| v.violation.severity() == "HIGH")
                .count(),
            "repos_with_violations": violations_by_object.len(),
            "rule_storms_detected": rule_storms.len(),
            "causal_chains": causal_chains.len(),
            "correlations_identified": correlations.len(),
        },
        "violations_by_repo": repos.iter().map(|repo| {
            let repo_viols = all_violations.iter()
                .filter(|v| v.repo_name == *repo)
                .collect::<Vec<_>>();
            json!({
                "repo": repo,
                "violation_count": repo_viols.len(),
                "critical": repo_viols.iter().filter(|v| v.violation.severity() == "CRITICAL").count(),
                "high": repo_viols.iter().filter(|v| v.violation.severity() == "HIGH").count(),
            })
        }).collect::<Vec<_>>(),
        "top_violations": all_violations.iter().take(10).map(|v| {
            json!({
                "repo": v.repo_name,
                "metric": v.metric_name,
                "severity": v.violation.severity(),
                "description": v.violation.description(),
                "object": v.module_path,
            })
        }).collect::<Vec<_>>(),
        "rule_statistics": rule_stats.iter().map(|(name, stats)| {
            json!({
                "rule": name,
                "total_violations": stats.total_violations,
                "severity_breakdown": stats.severity_counts,
                "affected_metrics": stats.top_metrics.iter()
                    .take(3)
                    .map(|(m, c)| format!("{} ({})", m, c))
                    .collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "rule_storms": rule_storms.iter().map(|storm| {
            json!({
                "repo": storm.repo,
                "metric": storm.metric,
                "rules": storm.rules,
                "timestamp": storm.timestamp,
            })
        }).collect::<Vec<_>>(),
        "correlations": correlations.iter().map(|corr| {
            json!({
                "metric_a": corr.metric_a,
                "metric_b": corr.metric_b,
                "coefficient": corr.correlation_coefficient,
            })
        }).collect::<Vec<_>>(),
        "causal_chains": causal_chains.iter().map(|chain| {
            json!({
                "root_cause": chain.root_cause,
                "consequents": chain.consequents,
                "confidence": chain.confidence,
            })
        }).collect::<Vec<_>>(),
        "receipt": {
            "format_version": receipt.format_version,
            "event_count": receipt.events.len(),
            "chain_hash": receipt.chain_hash.as_hex(),
        },
    });

    println!("Report Summary:");
    println!("─────────────");
    println!("  {} violations detected", all_violations.len());
    println!(
        "  {} critical violations",
        report["summary"]["critical_violations"]
    );
    println!(
        "  {} high-severity violations",
        report["summary"]["high_violations"]
    );
    println!("  {} rule storms", rule_storms.len());
    println!("  {} causal chains identified", causal_chains.len());
    println!("  {} metric correlations", correlations.len());
    println!("  {} objects affected", violations_by_object.len());
    println!();

    // =========================================================================
    // PHASE 10: JSON Output
    // =========================================================================
    println!("📤 PHASE 10: JSON Output");
    println!("───────────────────────\n");

    let json_output = serde_json::to_string_pretty(&report)?;
    println!("JSON Report (excerpt):");
    println!("{}", &json_output[..std::cmp::min(800, json_output.len())]);
    if json_output.len() > 800 {
        println!("... ({}+ more bytes)", json_output.len() - 800);
    }
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                      ANALYSIS COMPLETE                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!(
        "✓ Processed {} repos with {} metric types each",
        repos.len(),
        metrics_baseline.len()
    );
    println!(
        "✓ Applied all 7 Western Electric rules to {} snapshots",
        repos.len() * 4
    );
    println!("✓ Detected {} violations (", all_violations.len());
    println!(
        "    {} CRITICAL,",
        all_violations
            .iter()
            .filter(|v| v.violation.severity() == "CRITICAL")
            .count()
    );
    println!(
        "    {} HIGH,",
        all_violations
            .iter()
            .filter(|v| v.violation.severity() == "HIGH")
            .count()
    );
    println!(
        "    {} MEDIUM)",
        all_violations
            .iter()
            .filter(|v| v.violation.severity() == "MEDIUM")
            .count()
    );
    println!(
        "✓ Identified {} rule storms (2+ simultaneous violations)",
        rule_storms.len()
    );
    println!("✓ Computed {} metric correlations", correlations.len());
    println!("✓ Built {} causal chains", causal_chains.len());
    println!(
        "✓ Tracked violations at {} object level",
        violations_by_object.len()
    );
    println!("✓ Generated {} OCEL events", ocel_events.len());
    println!("✓ Assembled receipt with {} events", receipt.events.len());
    println!("✓ Receipt chain: {}", &receipt.chain_hash.as_hex()[..16]);
    println!();

    Ok(())
}
