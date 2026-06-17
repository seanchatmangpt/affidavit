//! Simulate monitoring a 300+ repo portfolio for code quality degradation.
//!
//! This example loads the portfolio_test_dataset.json and demonstrates:
//! - Western Electric rule detection across all repos
//! - Violation aggregation and severity ranking
//! - Action item prioritization for governance teams
//!
//! Run with: cargo run --example portfolio_monitor_simulation --features shell

use affidavit::quality::{CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;

fn main() -> anyhow::Result<()> {
    println!("🔍 Portfolio Quality Monitor Simulation");
    println!("======================================\n");

    // Load portfolio dataset
    let portfolio_json = fs::read_to_string("portfolio_test_dataset.json")?;
    let portfolio: Value = serde_json::from_str(&portfolio_json)?;

    let repos = portfolio
        .get("portfolio")
        .and_then(|p| p.get("repositories"))
        .and_then(|r| r.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid portfolio structure"))?;

    println!("📊 Analyzing {} repositories...\n", repos.len());

    // Aggregate results
    let mut repo_violations: HashMap<String, Vec<String>> = HashMap::new();
    let mut violation_counts: HashMap<String, usize> = HashMap::new();
    let mut critical_repos: Vec<String> = Vec::new();
    let mut warning_repos: Vec<String> = Vec::new();

    // Process each repo
    for repo in repos {
        let repo_name = repo
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = repo
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let violations = repo.get("violations").and_then(|v| v.as_array());

        if let Some(violations_array) = violations {
            let mut repo_violation_descs = Vec::new();

            for violation in violations_array {
                let rule = violation
                    .get("rule")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let severity = violation
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("MEDIUM");

                let desc = violation
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                repo_violation_descs.push(format!("[{}] {} — {}", severity, rule, desc));

                *violation_counts.entry(rule.to_string()).or_insert(0) += 1;

                if severity == "CRITICAL" || status == "anomaly_detected" {
                    critical_repos.push(repo_name.to_string());
                } else if severity == "HIGH" {
                    warning_repos.push(repo_name.to_string());
                }
            }

            repo_violations.insert(repo_name.to_string(), repo_violation_descs);
        }
    }

    // Print critical repos
    if !critical_repos.is_empty() {
        println!("🚨 CRITICAL VIOLATIONS ({} repos)", critical_repos.len());
        println!("──────────────────────────────────────");
        critical_repos.sort();
        critical_repos.dedup();
        for repo in &critical_repos {
            println!("  ❌ {}", repo);
            if let Some(viols) = repo_violations.get(repo) {
                for v in viols {
                    println!("     └ {}", v);
                }
            }
        }
        println!();
    }

    // Print warning repos
    if !warning_repos.is_empty() {
        println!(
            "⚠️  HIGH-SEVERITY VIOLATIONS ({} repos)",
            warning_repos.len()
        );
        println!("──────────────────────────────────────");
        warning_repos.sort();
        warning_repos.dedup();
        for repo in warning_repos.iter().take(10) {
            println!("  ⚠️  {}", repo);
            if let Some(viols) = repo_violations.get(repo) {
                for v in viols.iter().take(2) {
                    println!("     └ {}", v);
                }
            }
        }
        if warning_repos.len() > 10 {
            println!("  ... and {} more", warning_repos.len() - 10);
        }
        println!();
    }

    // Print violation rule summary
    println!("📋 Western Electric Rule Violations");
    println!("──────────────────────────────────");
    let mut rules: Vec<_> = violation_counts.iter().collect();
    rules.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
    for (rule, count) in rules {
        let emoji = match rule.as_str() {
            "Rule1Sigma" => "📈",
            "Rule9InRow" => "🪓",
            "RuleTrend" => "📉",
            "RuleAlternating" => "🎪",
            "Rule2of3Beyond2Sigma" => "⚡",
            "Rule4of5Beyond1Sigma" => "🌊",
            "Rule15InRowWithin1Sigma" => "🔄",
            _ => "❓",
        };
        println!("  {} {}: {} violations", emoji, rule, count);
    }
    println!();

    // Print synthetic Western Electric analysis
    println!("🎯 Quality Control Status");
    println!("────────────────────────");
    println!("  Baseline: stub_ratio = 0.032 ± 0.025");
    println!("  Baseline: type_coverage = 0.91 ± 0.08");
    println!("  Baseline: test_coverage = 82.5 ± 6.2%");
    println!();
    println!(
        "  In Control:   {} repos",
        repos.len() - critical_repos.len() - warning_repos.len()
    );
    println!("  Warnings:     {} repos", warning_repos.len());
    println!("  Critical:     {} repos", critical_repos.len());
    println!();

    // Print recommendations
    println!("📌 Action Items (by severity)");
    println!("────────────────────────────");
    println!("  1. [CRITICAL] affidavit");
    println!("     → Audit commits for placeholder code (stub spike, type collapse)");
    println!("  2. [CRITICAL] zombie-repo-abandoned");
    println!("     → Archive or delete (9 REJECT verdicts, 0 commits in 90 days)");
    println!("  3. [HIGH] thrashing-repo-unstable");
    println!("     → Architecture review (ACCEPT/REJECT alternating pattern)");
    println!("  4. [HIGH] degrading-quality");
    println!("     → Schedule refactoring (6-point monotonic quality decline)");
    println!("  5. [MEDIUM] early-warning-plateau");
    println!("     → Monitor and optimize (quality stagnant at marginal threshold)");
    println!();

    // Emit sample violation event
    println!("📤 Sample Receipt Event (for audit trail)");
    println!("─────────────────────────────────────────");
    let sample_event = json!({
        "event_type": "quality.violation",
        "timestamp": 1718655600,
        "repo": "affidavit",
        "rule": "Rule1Sigma",
        "metric": "stub_ratio",
        "value": 0.12,
        "baseline_mean": 0.02,
        "baseline_stddev": 0.01,
        "z_score": 10.0,
        "severity": "CRITICAL",
        "description": "Stub ratio spike: 6x increase over baseline"
    });
    println!("{}", serde_json::to_string_pretty(&sample_event)?);
    println!();

    println!("✅ Portfolio analysis complete.");
    println!("Use 'affi receipt monitor --watch <repo> --rules all' to test live.\n");

    Ok(())
}
