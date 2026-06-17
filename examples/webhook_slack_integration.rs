//! Webhook sink integration example: Quality violations to Slack.
//!
//! This example demonstrates how to:
//! 1. Measure code quality with Western Electric rules
//! 2. Detect violations (e.g., metrics beyond 3σ)
//! 3. Format violations as Slack messages
//! 4. POST to a Slack webhook (or generic webhook sink)
//!
//! # Usage
//!
//! ```bash
//! # Set your Slack webhook URL (or any generic webhook)
//! export WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
//!
//! # Run the example
//! cargo run --example webhook_slack_integration --features shell
//! ```

use serde_json::json;

fn main() -> anyhow::Result<()> {
    println!("=== Webhook Slack Integration Example ===\n");

    // Simulate a quality violation (as detected by Western Electric rules)
    let violation = create_sample_violation();

    println!("1. Detected Violation:");
    println!("   Rule: Rule1Sigma");
    println!("   Metric: test_coverage");
    println!("   Value: 0.45");
    println!("   Threshold: 0.88");
    println!("   Severity: CRITICAL\n");

    // Format as Slack message
    let slack_message = format_slack_message(&violation);

    println!("2. Slack Message Format:");
    println!("{}\n", serde_json::to_string_pretty(&slack_message)?);

    // Example webhook payload (what gets POSTed)
    let webhook_payload = json!({
        "rule": "Rule1Sigma",
        "metric": "test_coverage",
        "value": 0.45,
        "threshold": 0.88,
        "z_score": 2.1,
        "severity": "CRITICAL",
        "description": "Test coverage dropped below expected control limit (0.45 vs 0.88 threshold, z=2.1σ)",
    });

    println!("3. Webhook POST Payload:");
    println!("{}\n", serde_json::to_string_pretty(&webhook_payload)?);

    // Show integration points
    println!("4. Integration Points:");
    println!("   - affi quality monitor --output webhook");
    println!("   - Environment: WEBHOOK_URL env var");
    println!("   - Retry: 3 attempts with exponential backoff (500ms, 1500ms)");
    println!("   - Failure mode: log and continue (graceful degradation)\n");

    // Slack message structure details
    println!("5. Slack Message Structure:");
    println!("   - Main text: violation summary with severity");
    println!("   - Attachments:");
    println!("     * color: danger (red) for CRITICAL");
    println!("     * fields: rule, metric, value, threshold, z_score, severity");
    println!("     * footer: timestamp, affi source\n");

    println!("6. Example cURL (raw webhook):");
    println!("   curl -X POST https://your-webhook.url \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{}'", serde_json::to_string(&webhook_payload)?);

    Ok(())
}

/// Simulate a quality violation (Rule1Sigma)
fn create_sample_violation() -> serde_json::Value {
    json!({
        "rule": "Rule1Sigma",
        "metric": "test_coverage",
        "value": 0.45,
        "threshold": 0.88,
        "z_score": 2.1,
        "severity": "CRITICAL",
        "description": "Test coverage dropped below expected control limit",
    })
}

/// Format a violation as a Slack message.
///
/// Returns a Slack message payload with:
/// - Primary text summarizing the violation
/// - Attachments with detailed fields and color coding
/// - Footer with timestamp and source attribution
fn format_slack_message(violation: &serde_json::Value) -> serde_json::Value {
    let rule = violation
        .get("rule")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let metric = violation
        .get("metric")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let severity = violation
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN");
    let description = violation
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description");

    // Color based on severity
    let color = match severity {
        "CRITICAL" => "danger",
        "HIGH" => "warning",
        "MEDIUM" => "#FF9900",
        _ => "good",
    };

    // Build fields for the attachment
    let mut fields = vec![
        json!({
            "title": "Rule",
            "value": rule,
            "short": true,
        }),
        json!({
            "title": "Metric",
            "value": metric,
            "short": true,
        }),
    ];

    // Add value/threshold if present
    if let (Some(val), Some(thresh)) = (violation.get("value"), violation.get("threshold")) {
        fields.push(json!({
            "title": "Value",
            "value": format!("{}", val),
            "short": true,
        }));
        fields.push(json!({
            "title": "Threshold",
            "value": format!("{}", thresh),
            "short": true,
        }));
    }

    // Add z-score if present
    if let Some(z) = violation.get("z_score") {
        fields.push(json!({
            "title": "Z-Score",
            "value": format!("{}", z),
            "short": true,
        }));
    }

    json!({
        "text": format!("Code Quality Violation: {} on metric '{}'", rule, metric),
        "attachments": [
            {
                "color": color,
                "title": format!("{} - {}", severity, rule),
                "text": description,
                "fields": fields,
                "footer": "affidavit quality monitor",
                "ts": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            }
        ]
    })
}

/// Example: format multiple violations for a Slack message thread
#[allow(dead_code)]
fn format_violations_batch(violations: &[serde_json::Value]) -> serde_json::Value {
    let total = violations.len();
    let critical_count = violations
        .iter()
        .filter(|v| v.get("severity").and_then(|s| s.as_str()) == Some("CRITICAL"))
        .count();

    let text = if total == 1 {
        "1 code quality violation detected".to_string()
    } else {
        format!(
            "{} code quality violations detected ({} CRITICAL)",
            total, critical_count
        )
    };

    let attachments: Vec<serde_json::Value> = violations
        .iter()
        .map(|v| {
            let rule = v.get("rule").and_then(|r| r.as_str()).unwrap_or("Unknown");
            let severity = v
                .get("severity")
                .and_then(|s| s.as_str())
                .unwrap_or("UNKNOWN");
            let color = match severity {
                "CRITICAL" => "danger",
                "HIGH" => "warning",
                _ => "good",
            };

            json!({
                "color": color,
                "title": format!("[{}] {}", severity, rule),
                "text": v.get("description").and_then(|d| d.as_str()).unwrap_or(""),
                "footer": "affidavit",
                "ts": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            })
        })
        .collect();

    json!({
        "text": text,
        "attachments": attachments
    })
}
