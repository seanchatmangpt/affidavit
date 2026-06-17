# Webhook Integration for Quality Violations (Phase 2)

## Overview

The affidavit quality monitor can POST quality violations detected by Western Electric rules to an external webhook service. This enables real-time notifications to Slack, PagerDuty, Datadog, or any custom HTTP endpoint.

## Features

- **3-Attempt Retry Logic**: Exponential backoff (500ms, 1500ms) for transient failures
- **Graceful Degradation**: Webhook errors don't stop monitoring; violations are logged locally
- **Rich JSON Payload**: Each violation includes rule, metric, value, threshold, z-score, and severity
- **Feature-Gated**: Only available when built with `--features webhook`
- **Environment-Driven**: Reads webhook URL from `WEBHOOK_URL` env var

## Quick Start

### 1. Build with webhook support

```bash
cargo build --features webhook
# or for all features:
cargo build --features all
```

### 2. Set webhook URL

```bash
export WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
# or any generic HTTP endpoint:
export WEBHOOK_URL="https://your-custom-service.com/violations"
```

### 3. Run quality monitor with webhook output

```bash
# Single measurement
affi quality monitor --output webhook,stderr

# With JSON format
affi quality monitor --output webhook,stderr --format json

# Watch mode (Phase 2 feature — currently runs once)
affi quality monitor --watch . --output webhook --interval 10
```

## Webhook Payload Format

Each violation POST includes:

```json
{
  "rule": "Rule1Sigma",
  "metric": "test_coverage",
  "value": 0.45,
  "threshold": 0.88,
  "z_score": 2.1,
  "severity": "CRITICAL",
  "description": "Test coverage dropped below expected control limit (0.45 vs 0.88 threshold, z=2.1σ)"
}
```

### Violation Fields

| Field       | Type   | Description |
|-------------|--------|-------------|
| `rule`      | string | Western Electric rule name (e.g., Rule1Sigma, Rule9InRow) |
| `metric`    | string | Affected metric (e.g., test_coverage, clippy_warnings) |
| `value`     | number | Current metric value |
| `threshold` | number | Control limit that was exceeded |
| `z_score`   | number | Standard deviations from baseline mean |
| `severity`  | string | CRITICAL, HIGH, MEDIUM, or INFO |
| `description` | string | Human-readable explanation of the violation |

## Slack Integration Example

### Using Slack Webhooks

1. Create a Slack incoming webhook: https://api.slack.com/messaging/webhooks
2. Copy the webhook URL
3. Set environment variable:

```bash
export WEBHOOK_URL="https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXX"
```

4. Run monitor:

```bash
affi quality monitor --output webhook,stderr
```

### Slack Message Format

The webhook handler formats violations as Slack messages:

```json
{
  "text": "Code Quality Violation: Rule1Sigma on metric 'test_coverage'",
  "attachments": [
    {
      "color": "danger",
      "title": "CRITICAL - Rule1Sigma",
      "text": "Test coverage dropped below expected control limit",
      "fields": [
        {
          "title": "Rule",
          "value": "Rule1Sigma",
          "short": true
        },
        {
          "title": "Metric",
          "value": "test_coverage",
          "short": true
        },
        {
          "title": "Value",
          "value": "0.45",
          "short": true
        },
        {
          "title": "Threshold",
          "value": "0.88",
          "short": true
        },
        {
          "title": "Z-Score",
          "value": "2.1",
          "short": true
        }
      ],
      "footer": "affidavit quality monitor",
      "ts": 1718641799
    }
  ]
}
```

See `examples/webhook_slack_integration.rs` for complete Slack formatting example.

## Western Electric Rules Reference

The monitor detects 7 control chart violations:

### Rule 1: Single Point Beyond 3σ (Spike Detection)
- **Trigger**: One value >3σ from baseline mean
- **Severity**: CRITICAL
- **Example**: Test coverage drops 30% in one commit

### Rule 2: 9-in-a-Row (Zombie Code)
- **Trigger**: 9 consecutive out-of-control points
- **Severity**: CRITICAL
- **Example**: 9 consecutive commits with high Clippy warnings

### Rule 3: Trend (Systematic Degradation)
- **Trigger**: 6 monotonic points (all increasing or all decreasing)
- **Severity**: HIGH
- **Example**: Code complexity steadily increasing over 6 commits

### Rule 4: Alternating (Uncertainty/Hallucination)
- **Trigger**: Wild swings up-down-up-down pattern
- **Severity**: HIGH
- **Example**: Churn ratio oscillating between 0.1 and 0.5

### Rule 5: 2-of-3 Beyond 2σ (Early Warning)
- **Trigger**: 2 or more of last 3 points beyond 2σ limits
- **Severity**: HIGH
- **Example**: 2 of last 3 commits have low doc coverage

### Rule 6: 4-of-5 Beyond 1σ (Sustained Deviation)
- **Trigger**: 4 of last 5 points beyond 1σ limits
- **Severity**: MEDIUM
- **Example**: High complexity in 4 of last 5 commits

### Rule 7: 15-in-a-Row Within 1σ (Stagnation/Plateau)
- **Trigger**: 15 consecutive points within ±1σ (no improvement)
- **Severity**: INFO/WARNING
- **Example**: Test coverage stuck at 0.88 for 15 commits

## Retry Behavior

When a POST fails, the handler retries with exponential backoff:

```
Attempt 1: Immediate POST
  → Success: return OK
  → 2xx: HTTP 200-299 (success)
  → 4xx: HTTP 400-499 (fail immediately, don't retry)
  → 5xx: HTTP 500-599 (retry)

Attempt 2: Wait 500ms, then retry

Attempt 3: Wait 1500ms, then retry

After 3 attempts: Log failure and continue (don't crash)
```

Example log output:
```
[webhook] attempt 1/3: POST https://hooks.slack.com/services/...
[webhook] attempt 1 failed: HTTP 503: server error; retrying
[webhook] attempt 2/3: POST https://hooks.slack.com/services/...
[webhook] success (HTTP 200)
```

## Environment Variables

| Variable | Required | Example |
|----------|----------|---------|
| `WEBHOOK_URL` | Yes (if --output includes webhook) | `https://hooks.slack.com/services/...` |
| `RUST_LOG` | Optional | `debug` for verbose webhook logs |

## Output Channels

Combine multiple output channels with comma separation:

```bash
# Multiple outputs
affi quality monitor --output stderr,json,events,webhook

# Available channels:
# - stderr: human-readable violations to stderr
# - json: JSON output to stdout
# - events: emit quality.violation events to receipt chain
# - webhook: POST violations to WEBHOOK_URL
```

## Error Handling

### Webhook URL not set

```bash
export WEBHOOK_URL=""  # Empty
affi quality monitor --output webhook
# Output: [monitor] warning: webhook output requested but WEBHOOK_URL env var not set
```

### Transient failures (5xx errors)

```
[webhook] attempt 1/3: POST https://...
[webhook] attempt 1 failed: HTTP 503: server error; retrying
[webhook] attempt 2/3: POST https://...
[webhook] success (HTTP 200)
```

### Permanent failures (4xx errors)

```
[webhook] attempt 1/3: POST https://...
[webhook] attempt 1 failed: HTTP 400: client error (no retry)
[webhook] failed after 1 attempts: HTTP 400: client error (no retry)
[quality] (violations logged locally, monitoring continues)
```

### Webhook unreachable

```
[webhook] attempt 1/3: POST https://unreachable.local
[webhook] attempt 1 failed: HTTP POST failed; retrying
[webhook] attempt 2/3: POST https://unreachable.local
[webhook] attempt 2 failed: HTTP POST failed; retrying
[webhook] attempt 3/3: POST https://unreachable.local
[webhook] failed after 3 attempts: HTTP POST failed
[quality] (violations logged locally, monitoring continues)
```

## Example: Custom Webhook Handler

Implement a simple webhook server to receive violations:

```python
# webhook_server.py
from flask import Flask, request
import json
import sys

app = Flask(__name__)

@app.route('/violations', methods=['POST'])
def receive_violation():
    violation = request.json
    severity = violation.get('severity', 'UNKNOWN')
    metric = violation.get('metric', 'unknown')
    rule = violation.get('rule', 'Unknown')
    
    print(f"[{severity}] {rule} on {metric}", file=sys.stderr)
    print(json.dumps(violation, indent=2))
    
    # Return 200 to indicate success
    return {'status': 'received'}, 200

if __name__ == '__main__':
    app.run(port=5000)
```

Run it:

```bash
python webhook_server.py &
export WEBHOOK_URL="http://localhost:5000/violations"
affi quality monitor --output webhook
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Quality Monitor
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build affidavit
        run: cargo build --features webhook
      
      - name: Monitor quality
        env:
          WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
        run: ./target/debug/affi quality monitor --output webhook,stderr
```

### GitLab CI

```yaml
quality:
  image: rust:latest
  script:
    - cargo build --features webhook
    - export WEBHOOK_URL=$SLACK_WEBHOOK_URL
    - ./target/debug/affi quality monitor --output webhook,stderr
  only:
    - push
    - merge_requests
```

## Testing

Run the example:

```bash
cargo run --example webhook_slack_integration --features shell
```

Mock webhook with `nc` (netcat):

```bash
# Terminal 1: Listen on port 8080
nc -l -p 8080

# Terminal 2: Trigger monitor
export WEBHOOK_URL="http://localhost:8080/violations"
affi quality monitor --output webhook
```

## Feature Flags

### Minimum build (webhook only)

```bash
cargo build --features webhook
```

### Full quality monitoring

```bash
cargo build --features quality-monitor,webhook
```

### All features (includes webhook)

```bash
cargo build --features all
```

## Troubleshooting

### 1. Webhook feature not enabled

```
[webhook] skipped: shell feature not enabled (build with --features shell)
```

**Fix**: Build with `--features webhook`

### 2. WEBHOOK_URL not set

```
[monitor] warning: webhook output requested but WEBHOOK_URL env var not set
```

**Fix**: `export WEBHOOK_URL="https://..."`

### 3. HTTP 403 (Unauthorized)

```
[webhook] failed after 1 attempts: HTTP 403: client error (no retry)
```

**Fix**: Check webhook URL and authentication token

### 4. Tokio runtime error

```
thread 'main' panicked at 'no runtime found'
```

**Fix**: Build with `--features shell,tokio` (included in webhook feature)

## Design Rationale

### Graceful Degradation

Webhook failures don't crash the monitor because:
- Monitoring integrity is more important than webhook delivery
- Network issues are transient; violations are still logged locally
- Users can inspect local logs if webhook fails

### Exponential Backoff

Retry strategy:
- Immediate first attempt (fast path)
- 500ms second attempt (allow brief service recovery)
- 1500ms third attempt (allow longer service restart)
- 3 attempts total balances responsiveness vs. resource usage

### Feature-Gated

Webhook support is optional (`--features webhook`) because:
- Not all users need external notifications
- Reduces dependency surface area for simple use cases
- Allows webhook feature to depend on `reqwest` and `tokio`

## Related Documentation

- `src/quality.rs`: Western Electric analyzer implementation
- `src/handlers.rs`: Handler functions including `send_violation_webhook`
- `examples/webhook_slack_integration.rs`: Slack message formatting example
- CLAUDE.md: Full project architecture

## References

- [Western Electric Rules](https://en.wikipedia.org/wiki/Western_Electric_rules)
- [Slack Incoming Webhooks](https://api.slack.com/messaging/webhooks)
- [Statistical Process Control](https://en.wikipedia.org/wiki/Statistical_process_control)
