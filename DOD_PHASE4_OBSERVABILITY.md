# Definition of Done — Phase 4: OpenTelemetry & Observability

**Project:** affidavit v26.6.14  
**Branch:** `claude/zen-cerf-oq87br`  
**Phase:** 4 of the DX/QOL 1000x initiative  
**Date:** 2026-06-14  
**Author:** Sean Chatman (xpointsh@gmail.com)

---

## Table of Contents

1. [Phase Overview](#phase-overview)
2. [Feature 1 — Verifier Stage Spans (`affi receipt trace`)](#feature-1--verifier-stage-spans)
3. [Feature 2 — OTel Metrics Dashboard](#feature-2--otel-metrics-dashboard)
4. [Feature 3 — OTel Baggage Propagation](#feature-3--otel-baggage-propagation)
5. [Feature 4 — Span Events at Verifier Stages](#feature-4--span-events-at-verifier-stages)
6. [Feature 5 — SLO Monitoring](#feature-5--slo-monitoring)
7. [Cross-Cutting Concerns](#cross-cutting-concerns)
8. [E2E Test Template](#e2e-test-template)
9. [Reference Tables](#reference-tables)

---

## Phase Overview

Phase 4 instruments the existing `affidavit` certify pipeline with real OpenTelemetry signals — traces, metrics, and SLO enforcement — without changing the pipeline's deterministic behavior. All new code is gated behind `feature = "otel"` (traces/baggage/span-events) and `feature = "metrics"` (Prometheus exporter + Grafana dashboard). The `"metrics"` feature implicitly depends on `"otel"`.

### Modified / New Files

| File | Status | Feature gate |
|------|--------|--------------|
| `src/tracing.rs` | MODIFY | `otel`, `metrics` |
| `src/metrics.rs` | NEW | `metrics` |
| `dashboards/affidavit.json` | NEW | (static artifact) |
| `tests/e2e_observability.rs` | NEW | `otel` (dev-dep only) |

### Invariants That Must Not Break

- `cargo test` (no features) passes: 30 tests continue to pass
- `cargo test --features otel` passes: existing 30 tests + new observability tests pass
- The `verify` pipeline verdict remains pure and deterministic over receipt bytes
- No wall-clock timestamps are introduced into the receipt chain
- The `_seal` field on `Receipt` remains private and unconstructable externally

---

## Feature 1 — Verifier Stage Spans

**Summary:** `affi receipt trace --span-export=jaeger` opens a parent `verify` span and six child spans, one per certify-pipeline stage. Spans are exported via OTLP/Jaeger. The existing `trace_verify` wrapper in `src/tracing.rs` becomes the parent span; each stage function gains a child span. Gated behind `feature = "otel"`.

### 1.1 Acceptance Criteria

> All scenarios use a valid 2-event receipt assembled by `ChainAssembler::finalize`.

**AC-1.1** — Parent span emitted  
Given a receipt at `receipt.json`,  
When `affi receipt trace --span-export=jaeger receipt.json` is invoked,  
Then the `InMemorySpanExporter` captures exactly one span with `name == "affi.verify"` and `status == OK`.

**AC-1.2** — Six child spans, one per stage  
Given the same invocation,  
When spans are collected from the exporter,  
Then there are exactly 6 child spans whose `name` attributes are, in order: `"affi.verify.decode"`, `"affi.verify.check_format"`, `"affi.verify.chain_integrity"`, `"affi.verify.continuity"`, `"affi.verify.verify_commitments"`, `"affi.verify.evaluate_profile"`.

**AC-1.3** — Parent-child linkage  
Given the captured spans,  
When inspecting each stage span's `parent_span_id`,  
Then it equals the `span_id` of the `"affi.verify"` parent span (not the trace root).

**AC-1.4** — Trace ID is uniform  
Given all 7 captured spans (1 parent + 6 children),  
When comparing `trace_id` across all spans,  
Then all 7 share the same `trace_id`.

**AC-1.5** — Stage outcome encoded as span status  
Given a tampered receipt whose chain hash does not match,  
When spans are captured,  
Then the `"affi.verify.chain_integrity"` span has `status == Error` and `status_description` contains `"chain hash mismatch"`.

**AC-1.6** — `receipt.path` attribute on parent  
Given any invocation,  
When the parent span's attributes are inspected,  
Then `receipt.path` is present and equals the absolute path passed to the command.

**AC-1.7** — `stage.index` attribute on each child span  
Given the six child spans,  
When `stage.index` is read from each,  
Then the values are `0`, `1`, `2`, `3`, `4`, `5` respectively, corresponding to pipeline order.

**AC-1.8** — No spans emitted without `feature = "otel"`  
Given a build compiled without the `otel` feature,  
When `cargo test` is run,  
Then no OTel SDK initialization occurs, no OTLP exporter is linked, and `captured_spans()` still works via the existing thread-local recorder (backward-compatible).

**AC-1.9** — Span timing: child spans do not overlap parent  
Given the 7 captured spans,  
When `start_time` and `end_time` of each child are compared to the parent,  
Then every child's `start_time >= parent.start_time` and every child's `end_time <= parent.end_time`.

**AC-1.10** — `verdict.accepted` attribute on parent span  
Given a valid receipt,  
When the parent span's attributes are inspected,  
Then `verdict.accepted` is `true` (boolean); for a tampered receipt it is `false`.

### 1.2 Span Hierarchy Specification

```
affi.verify  [TraceId=T, SpanId=P, parent=none]
│  Attributes:
│    receipt.path          → string  (absolute file path)
│    receipt.event_count   → int     (number of events in the receipt)
│    receipt.format_version → string ("core/v1")
│    verdict.accepted      → bool
│    verdict.reason        → string
│
├── affi.verify.decode  [TraceId=T, SpanId=C0, parent=P]
│     stage.index  → 0
│     stage.name   → "decode"
│     stage.passed → bool
│     stage.detail → string
│
├── affi.verify.check_format  [TraceId=T, SpanId=C1, parent=P]
│     stage.index  → 1
│     stage.name   → "check_format"
│     stage.passed → bool
│     stage.detail → string
│
├── affi.verify.chain_integrity  [TraceId=T, SpanId=C2, parent=P]
│     stage.index  → 2
│     stage.name   → "chain_integrity"
│     stage.passed → bool
│     stage.detail → string
│
├── affi.verify.continuity  [TraceId=T, SpanId=C3, parent=P]
│     stage.index  → 3
│     stage.name   → "continuity"
│     stage.passed → bool
│     stage.detail → string
│
├── affi.verify.verify_commitments  [TraceId=T, SpanId=C4, parent=P]
│     stage.index  → 4
│     stage.name   → "verify_commitments"
│     stage.passed → bool
│     stage.detail → string
│
└── affi.verify.evaluate_profile  [TraceId=T, SpanId=C5, parent=P]
      stage.index  → 5
      stage.name   → "evaluate_profile"
      stage.passed → bool
      stage.detail → string
```

**Naming convention:** `affi.<operation>.<stage>` — always lowercase, dot-separated.

**Status mapping:**

| `stage.passed` | OTel Span Status |
|---|---|
| `true` | `StatusCode::Ok` |
| `false` | `StatusCode::Error` with `status_description = stage.detail` |

### 1.3 Implementation Notes (`src/tracing.rs`)

The existing `trace_verify` function must be extended under `#[cfg(feature = "otel")]`:

```rust
#[cfg(feature = "otel")]
pub fn trace_verify_with_stages<F>(receipt_path: &str, f: F) -> Verdict
where
    F: FnOnce(&dyn Fn(&str, usize, &CheckOutcome)) -> Verdict,
{
    // Open parent span "affi.verify", pass stage_span_hook to f,
    // set parent span attributes from returned Verdict.
}
```

The `#[cfg(not(feature = "otel"))]` path falls back to the existing `trace_verify` with no OTel overhead.

### 1.4 Test Strategy

- Use `opentelemetry_sdk::testing::InMemorySpanExporter` (already a dev-dependency via `opentelemetry = "0.20"`).
- No real Jaeger process needed: `--span-export=jaeger` in test mode uses the in-memory exporter injected via a test-only `TracerProvider` override.
- Each test calls `exporter.reset()` before the scenario to prevent cross-test contamination.
- Tests tagged `#[cfg(feature = "otel")]` are compiled only when the feature is active.

### 1.5 Graceful Degradation

| Failure condition | Expected behavior |
|---|---|
| Jaeger endpoint unreachable (connection refused) | Spans are dropped silently; `affi` exits with the normal verdict exit code; a `WARN` is printed to stderr: `"[affi] OTel export failed: <reason>; continuing"` |
| OTLP exporter times out | Same as above; timeout is 500 ms max |
| `OTEL_EXPORTER_OTLP_ENDPOINT` env var unset | Falls back to `http://localhost:4317`; logs a single `INFO` line |
| OTel SDK panics during initialization | Caught via `std::panic::catch_unwind`; `affi` continues without instrumentation |

---

## Feature 2 — OTel Metrics Dashboard

**Summary:** `src/metrics.rs` introduces `MetricsCollector` which records per-stage latency histograms, error counters, and receipt throughput. A Prometheus exporter scrape endpoint is exposed. A Grafana dashboard `dashboards/affidavit.json` ships with pre-built panels. Gated behind `feature = "metrics"` (which implies `feature = "otel"`).

### 2.1 Acceptance Criteria

**AC-2.1** — `MetricsCollector::new()` compiles and initializes  
Given `affidavit` compiled with `--features metrics`,  
When `MetricsCollector::new()` is called,  
Then no panic occurs and the returned collector holds valid OTel Meter handles.

**AC-2.2** — Stage latency histogram records observation  
Given a `MetricsCollector`,  
When `collector.record_stage_latency("chain_integrity", Duration::from_millis(12))` is called,  
Then the `affidavit_stage_latency_ms` histogram has exactly one observation with `stage="chain_integrity"`.

**AC-2.3** — Error counter increments on `stage.passed == false`  
Given a verify run that fails `chain_integrity`,  
When `MetricsCollector::record_stage_error("chain_integrity")` is called,  
Then `affidavit_stage_errors_total{stage="chain_integrity"}` increments by 1.

**AC-2.4** — Receipt throughput counter increments  
Given any completed verify call,  
When `MetricsCollector::record_receipt_verified(accepted: bool)` is called,  
Then `affidavit_receipts_verified_total{verdict="accept"}` or `{verdict="reject"}` increments by 1 respectively.

**AC-2.5** — Prometheus scrape output is valid  
Given the Prometheus exporter running at `http://localhost:9090/metrics`,  
When `curl -s http://localhost:9090/metrics` is executed,  
Then the response body contains lines matching `affidavit_stage_latency_ms_bucket`, `affidavit_stage_errors_total`, and `affidavit_receipts_verified_total`.

**AC-2.6** — Histogram buckets include SLO boundary  
Given the `affidavit_stage_latency_ms` histogram definition,  
When its bucket boundaries are inspected,  
Then the set includes `100.0` (the p99 SLO threshold in ms).

**AC-2.7** — `MetricsCollector` is `Send + Sync`  
Given `MetricsCollector`,  
When it is moved into a `std::thread::spawn` closure,  
Then it compiles without error (verified by a `#[test]` that calls `let _: Box<dyn Send + Sync> = Box::new(MetricsCollector::new())`).

**AC-2.8** — Dashboard JSON is valid JSON  
Given `dashboards/affidavit.json`,  
When parsed with `serde_json::from_str`,  
Then no error occurs.

**AC-2.9** — All 6 stage labels are present in metrics  
Given a full verify run over a valid receipt,  
When all stage metrics are collected,  
Then the label `stage` appears with values: `decode`, `check_format`, `chain_integrity`, `continuity`, `verify_commitments`, `evaluate_profile`.

**AC-2.10** — Prometheus exporter port is configurable  
Given `AFFIDAVIT_METRICS_PORT=9091` is set in the environment,  
When the metrics server starts,  
Then it listens on port `9091`, not `9090`.

### 2.2 Prometheus Metrics Definition Table

| Metric Name | Type | Description | Labels |
|---|---|---|---|
| `affidavit_stage_latency_ms` | Histogram | Wall-clock duration of each verifier stage in milliseconds | `stage` |
| `affidavit_stage_errors_total` | Counter | Total number of times a stage returned `passed=false` | `stage` |
| `affidavit_receipts_verified_total` | Counter | Total receipts that completed the certify pipeline | `verdict` (`accept`\|`reject`) |
| `affidavit_verify_duration_ms` | Histogram | End-to-end duration of a full verify call in milliseconds | — |
| `affidavit_chain_events_total` | Histogram | Number of events in receipts processed | — |
| `affidavit_active_verify_ops` | Gauge | Number of verify operations currently in flight | — |
| `affidavit_slo_p99_latency_ms` | Gauge | Current computed p99 latency across all stages (ms) | — |
| `affidavit_slo_error_rate_pct` | Gauge | Current error rate as a percentage (0–100) | — |
| `affidavit_slo_availability_pct` | Gauge | Current availability percentage (0–100) | — |

**Histogram bucket boundaries for `affidavit_stage_latency_ms`:**  
`[1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]`

**Histogram bucket boundaries for `affidavit_verify_duration_ms`:**  
`[5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 5000.0]`

### 2.3 Grafana Dashboard Panel Specifications

Dashboard UID: `affidavit-observability-v1`  
Dashboard title: `affidavit — Provenance Pipeline Observability`  
Data source: `Prometheus`  
Refresh interval: `10s`

| Panel # | Title | Prometheus Query | Visualization |
|---|---|---|---|
| 1 | Receipt Throughput (rate/min) | `rate(affidavit_receipts_verified_total[1m]) * 60` | Time series (line) |
| 2 | Accept vs Reject Split | `sum by (verdict)(affidavit_receipts_verified_total)` | Pie chart |
| 3 | Stage p99 Latency (ms) | `histogram_quantile(0.99, sum by (stage, le)(rate(affidavit_stage_latency_ms_bucket[5m])))` | Bar gauge (by stage) |
| 4 | Stage Error Rate | `rate(affidavit_stage_errors_total[5m])` | Time series (line, by stage) |
| 5 | End-to-End Verify p50/p95/p99 | `histogram_quantile(0.99, rate(affidavit_verify_duration_ms_bucket[5m]))` | Stat (3 panels stacked) |
| 6 | Active Verify Operations | `affidavit_active_verify_ops` | Stat (current value) |
| 7 | Chain Event Count Distribution | `histogram_quantile(0.50, rate(affidavit_chain_events_total_bucket[5m]))` | Histogram |
| 8 | SLO: p99 Latency ≤ 100 ms | `affidavit_slo_p99_latency_ms` with threshold at `100` | Gauge (red above 100) |
| 9 | SLO: Error Rate ≤ 0.1% | `affidavit_slo_error_rate_pct` with threshold at `0.1` | Gauge (red above 0.1) |
| 10 | SLO: Availability ≥ 99.9% | `affidavit_slo_availability_pct` with threshold at `99.9` | Gauge (red below 99.9) |

### 2.4 Grafana Dashboard JSON Skeleton

```json
{
  "uid": "affidavit-observability-v1",
  "title": "affidavit — Provenance Pipeline Observability",
  "tags": ["affidavit", "provenance", "otel", "slo"],
  "schemaVersion": 36,
  "version": 1,
  "refresh": "10s",
  "time": { "from": "now-1h", "to": "now" },
  "templating": {
    "list": [
      {
        "name": "datasource",
        "type": "datasource",
        "query": "prometheus",
        "current": { "text": "Prometheus", "value": "Prometheus" }
      }
    ]
  },
  "panels": [
    {
      "id": 1,
      "title": "Receipt Throughput (rate/min)",
      "type": "timeseries",
      "gridPos": { "x": 0, "y": 0, "w": 12, "h": 8 },
      "targets": [
        {
          "expr": "rate(affidavit_receipts_verified_total[1m]) * 60",
          "legendFormat": "{{verdict}}"
        }
      ]
    },
    {
      "id": 2,
      "title": "Accept vs Reject Split",
      "type": "piechart",
      "gridPos": { "x": 12, "y": 0, "w": 12, "h": 8 },
      "targets": [
        {
          "expr": "sum by (verdict)(affidavit_receipts_verified_total)",
          "legendFormat": "{{verdict}}"
        }
      ]
    },
    {
      "id": 3,
      "title": "Stage p99 Latency (ms)",
      "type": "bargauge",
      "gridPos": { "x": 0, "y": 8, "w": 24, "h": 8 },
      "targets": [
        {
          "expr": "histogram_quantile(0.99, sum by (stage, le)(rate(affidavit_stage_latency_ms_bucket[5m])))",
          "legendFormat": "{{stage}}"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "thresholds": {
            "steps": [
              { "color": "green", "value": 0 },
              { "color": "yellow", "value": 50 },
              { "color": "red", "value": 100 }
            ]
          }
        }
      }
    },
    {
      "id": 4,
      "title": "Stage Error Rate",
      "type": "timeseries",
      "gridPos": { "x": 0, "y": 16, "w": 12, "h": 8 },
      "targets": [
        {
          "expr": "rate(affidavit_stage_errors_total[5m])",
          "legendFormat": "{{stage}}"
        }
      ]
    },
    {
      "id": 5,
      "title": "End-to-End Verify p99 (ms)",
      "type": "stat",
      "gridPos": { "x": 12, "y": 16, "w": 12, "h": 8 },
      "targets": [
        {
          "expr": "histogram_quantile(0.99, rate(affidavit_verify_duration_ms_bucket[5m]))",
          "legendFormat": "p99"
        },
        {
          "expr": "histogram_quantile(0.95, rate(affidavit_verify_duration_ms_bucket[5m]))",
          "legendFormat": "p95"
        },
        {
          "expr": "histogram_quantile(0.50, rate(affidavit_verify_duration_ms_bucket[5m]))",
          "legendFormat": "p50"
        }
      ]
    },
    {
      "id": 8,
      "title": "SLO: p99 Latency vs 100ms Threshold",
      "type": "gauge",
      "gridPos": { "x": 0, "y": 24, "w": 8, "h": 8 },
      "targets": [
        { "expr": "affidavit_slo_p99_latency_ms", "legendFormat": "p99 ms" }
      ],
      "fieldConfig": {
        "defaults": {
          "min": 0,
          "max": 200,
          "thresholds": {
            "steps": [
              { "color": "green", "value": 0 },
              { "color": "red", "value": 100 }
            ]
          }
        }
      }
    },
    {
      "id": 9,
      "title": "SLO: Error Rate vs 0.1% Threshold",
      "type": "gauge",
      "gridPos": { "x": 8, "y": 24, "w": 8, "h": 8 },
      "targets": [
        { "expr": "affidavit_slo_error_rate_pct", "legendFormat": "error %" }
      ],
      "fieldConfig": {
        "defaults": {
          "min": 0,
          "max": 5,
          "thresholds": {
            "steps": [
              { "color": "green", "value": 0 },
              { "color": "red", "value": 0.1 }
            ]
          }
        }
      }
    },
    {
      "id": 10,
      "title": "SLO: Availability vs 99.9% Threshold",
      "type": "gauge",
      "gridPos": { "x": 16, "y": 24, "w": 8, "h": 8 },
      "targets": [
        { "expr": "affidavit_slo_availability_pct", "legendFormat": "availability %" }
      ],
      "fieldConfig": {
        "defaults": {
          "min": 99,
          "max": 100,
          "thresholds": {
            "steps": [
              { "color": "red", "value": 0 },
              { "color": "green", "value": 99.9 }
            ]
          }
        }
      }
    }
  ]
}
```

### 2.5 Test Strategy

- Unit tests in `src/metrics.rs` use `opentelemetry_sdk::metrics::InMemoryMetricsExporter` (available via `opentelemetry_sdk` dev-dep).
- Each test calls `MetricsCollector::new_with_exporter(exporter.clone())` so that observations can be retrieved via `exporter.get_finished_metrics()`.
- Prometheus scrape tests use `prometheus_client::encoding::text::encode` to generate scrape output in-process, asserting that metric names and label values appear.
- Dashboard JSON validity is asserted by a `#[test] fn dashboard_json_is_valid()` that does `serde_json::from_str::<serde_json::Value>(include_str!("../dashboards/affidavit.json")).unwrap()`.

### 2.6 Graceful Degradation

| Failure condition | Expected behavior |
|---|---|
| Prometheus scrape server fails to bind port | `affi` starts without metrics; logs `WARN "[affi] metrics server bind failed on :<port>: <reason>; metrics disabled"` |
| Prometheus registry full | Observations silently dropped; no panic |
| `AFFIDAVIT_METRICS_PORT` not a valid port number | Falls back to `9090`; logs a `WARN` |
| OTel Meter initialization fails | `MetricsCollector` returns a no-op collector via `MetricsCollector::noop()` |

---

## Feature 3 — OTel Baggage Propagation

**Summary:** Attach three key-value pairs to the OTel Baggage context when a verify operation begins: `receipt_id`, `timestamp`, and `format_version`. These propagate automatically to all child spans and downstream systems via the W3C Baggage header. Implemented in `src/tracing.rs`. Gated behind `feature = "otel"`.

### 3.1 Acceptance Criteria

**AC-3.1** — Baggage is set before child spans are created  
Given a verify operation with a receipt whose `chain_hash` is `"abc123..."` and `format_version` is `"core/v1"`,  
When the parent `affi.verify` span opens,  
Then `opentelemetry::baggage::get_baggage(cx).get("receipt_id")` returns the receipt's `chain_hash` value.

**AC-3.2** — `receipt_id` baggage key equals the receipt's `chain_hash`  
Given any receipt produced by `ChainAssembler::finalize`,  
When baggage is set,  
Then `baggage["receipt_id"] == receipt.chain_hash.as_hex()`.

**AC-3.3** — `timestamp` baggage key is set to ISO 8601 UTC  
Given any verify invocation,  
When baggage is read immediately after context propagation,  
Then `baggage["timestamp"]` parses as a valid ISO 8601 UTC datetime string (format: `YYYY-MM-DDTHH:MM:SSZ`).

**AC-3.4** — `format_version` baggage key matches receipt field  
Given a receipt with `format_version = "core/v1"`,  
When baggage is inspected,  
Then `baggage["format_version"] == "core/v1"`.

**AC-3.5** — Baggage does not mutate the receipt  
Given the same receipt before and after a verify-with-baggage call,  
When the receipt's fields are compared,  
Then `receipt.chain_hash`, `receipt.format_version`, and `receipt.events` are byte-identical before and after.

**AC-3.6** — Baggage context propagates to all 6 child stage spans  
Given a verify call with baggage set on the parent context,  
When each child stage span's baggage is read via `Context::current()`,  
Then `baggage["receipt_id"]` is non-empty for each of the 6 stage spans.

**AC-3.7** — Baggage is cleared after verify completes  
Given a single-threaded verify call,  
When the parent span exits its scope,  
Then the attached `Context` is dropped and `Context::current()` no longer contains `receipt_id` baggage (no baggage leaks between calls).

**AC-3.8** — Baggage keys use exact casing and spelling  
Given any verify invocation,  
When all baggage key names are enumerated,  
Then they are exactly: `receipt_id`, `timestamp`, `format_version` — lowercase, underscore-separated, no additional keys.

**AC-3.9** — `timestamp` is set once per verify call (not per stage)  
Given a single verify call that runs 6 stages,  
When the `timestamp` baggage value is sampled at stage 1 and at stage 6,  
Then both samples are identical (the timestamp is frozen at the start of the verify call).

**AC-3.10** — Baggage is not emitted when `feature = "otel"` is disabled  
Given a build without the `otel` feature,  
When `trace_verify` is called,  
Then no baggage API is invoked (verified by absence of the `opentelemetry::baggage` import in the `#[cfg(not(feature = "otel"))]` path).

### 3.2 Baggage Key Specification

| Key | Value Type | Description | Example Value |
|---|---|---|---|
| `receipt_id` | `string` (64-char lowercase hex) | The receipt's `chain_hash` — unique, content-addressed identifier | `"203d3bbf..."` |
| `timestamp` | `string` (ISO 8601 UTC, `Z` suffix) | Wall-clock time when the verify call began (NOT part of the chain; metadata only) | `"2026-06-14T09:00:00Z"` |
| `format_version` | `string` | The receipt's declared `format_version` field | `"core/v1"` |

**Wire format:** W3C Baggage header — `baggage: receipt_id=203d3bbf...,timestamp=2026-06-14T09:00:00Z,format_version=core%2Fv1`

**Note:** `format_version` must be percent-encoded when placed in the W3C Baggage header because `/` is not a valid baretoken character.

### 3.3 Implementation Notes (`src/tracing.rs`)

```rust
#[cfg(feature = "otel")]
fn build_baggage(receipt: &Receipt) -> opentelemetry::baggage::Baggage {
    opentelemetry::baggage::Baggage::new()
        .with("receipt_id", receipt.chain_hash.as_hex().to_string())
        .with("timestamp", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .with("format_version", receipt.format_version.clone())
}
```

The returned `Baggage` is attached to the current `Context` via `Context::with_baggage` before child spans are created.

### 3.4 Test Strategy

- Tests inject a mock `SystemTime` provider so `timestamp` values are deterministic.
- `opentelemetry::Context::current()` is read inside a `with_context` block in tests to assert baggage values.
- Baggage propagation to children is tested by reading `Context::current()` inside a synthetic stage callback.

### 3.5 Graceful Degradation

| Failure condition | Expected behavior |
|---|---|
| `chrono` timestamp fails (system clock unavailable) | `timestamp` baggage key is set to `"unknown"`; verify proceeds normally |
| OTel SDK baggage API unavailable | `build_baggage` returns early; no baggage is set; no panic |
| Baggage header too large for downstream system (>4KB) | Values are truncated to 256 chars each with suffix `"...(truncated)"` |

---

## Feature 4 — Span Events at Verifier Stages

**Summary:** At the entry and exit of each verifier stage, emit named OTel span events (structured log points on the span timeline). Entry events indicate stage start; exit events capture the outcome. This gives sub-span resolution without creating additional child spans. Implemented in `src/tracing.rs`. Gated behind `feature = "otel"`.

### 4.1 Acceptance Criteria

**AC-4.1** — Entry event emitted at start of each stage  
Given a verify call,  
When the `"affi.verify.chain_integrity"` span is active,  
Then the span contains an event named `"stage.start"` with attribute `stage.name = "chain_integrity"`.

**AC-4.2** — Exit event emitted at end of each stage  
Given a verify call,  
When a stage span completes,  
Then the span contains an event named `"stage.end"` with attributes `stage.name`, `stage.passed`, and `stage.detail`.

**AC-4.3** — Events are in chronological order  
Given a single stage span with both `"stage.start"` and `"stage.end"` events,  
When their timestamps are compared,  
Then `stage.start.timestamp < stage.end.timestamp`.

**AC-4.4** — `"stage.end"` event on error includes failure reason  
Given a tampered receipt that fails `chain_integrity`,  
When the `"affi.verify.chain_integrity"` span events are inspected,  
Then the `"stage.end"` event has `stage.passed = false` and `stage.detail` contains the mismatch description.

**AC-4.5** — Events carry `stage.index` attribute  
Given any stage span,  
When the `"stage.start"` and `"stage.end"` events are read,  
Then both carry `stage.index` matching the stage's position in the pipeline (0–5).

**AC-4.6** — No span events emitted without `feature = "otel"`  
Given a build without `feature = "otel"`,  
When `cargo test` runs,  
Then no `add_event` calls are compiled in and no OTel event API is imported.

**AC-4.7** — Parent span carries a `"verify.start"` event  
Given the parent `affi.verify` span,  
When span events are listed,  
Then there is exactly one event named `"verify.start"` before any stage events.

**AC-4.8** — Parent span carries a `"verify.end"` event  
Given the parent `affi.verify` span after all stages complete,  
When span events are listed,  
Then there is exactly one event named `"verify.end"` with attribute `verdict.accepted` (bool).

**AC-4.9** — Span events have monotonically increasing timestamps across all stages  
Given the complete ordered sequence of `"stage.start"` and `"stage.end"` events across all stage spans,  
When timestamps are sorted,  
Then they are in the same order as pipeline execution (stage 0 start → stage 0 end → stage 1 start → ... → stage 5 end).

**AC-4.10** — Total event count is exactly 14 per verify call  
Given any verify call (pass or fail),  
When all span events across all 7 spans (1 parent + 6 children) are counted,  
Then the total is 14: 1 `verify.start` + 1 `verify.end` on parent + (2 events × 6 stages = 12 events).

### 4.2 Span Event Attribute Table

| Event Name | Span it appears on | Attributes |
|---|---|---|
| `verify.start` | `affi.verify` | `receipt.path`, `receipt.event_count` |
| `verify.end` | `affi.verify` | `verdict.accepted`, `verdict.reason` |
| `stage.start` | `affi.verify.<stage>` | `stage.name`, `stage.index` |
| `stage.end` | `affi.verify.<stage>` | `stage.name`, `stage.index`, `stage.passed`, `stage.detail` |

### 4.3 Implementation Notes (`src/tracing.rs`)

Span events are added via `span.add_event(name, attributes)` on the active span reference. Each stage function's wrapper calls `add_event("stage.start", ...)` before invoking the stage function and `add_event("stage.end", ...)` after receiving the `CheckOutcome`.

### 4.4 Test Strategy

- Retrieve events via `span.events()` on `SpanData` from `InMemorySpanExporter`.
- Assert event names, ordering by `timestamp`, and attribute presence.
- Test the 14-event-per-call invariant with a `#[test] fn span_event_count_is_14()`.

### 4.5 Graceful Degradation

| Failure condition | Expected behavior |
|---|---|
| Active span is `NoopSpan` (feature disabled) | `add_event` is a no-op; no panic |
| Event attributes exceed OTel SDK limits | SDK silently drops excess attributes; no panic |
| Clock resolution too low for ordered timestamps | Events are still emitted; ordering test is skipped with `#[ignore]` on low-resolution targets |

---

## Feature 5 — SLO Monitoring

**Summary:** `src/metrics.rs` exposes `compute_sli()` which reads from a rolling window of recorded observations and computes three SLIs: p99 latency, error rate percentage, and availability percentage. Three SLO thresholds are enforced. Gated behind `feature = "metrics"`.

### 5.1 Acceptance Criteria

**AC-5.1** — `compute_sli()` returns a `ServiceLevelIndicators` struct  
Given a `MetricsCollector` with at least one recorded observation,  
When `collector.compute_sli()` is called,  
Then it returns `Ok(ServiceLevelIndicators { latency_p99_ms, error_rate_pct, availability_pct })` with no panic.

**AC-5.2** — p99 latency computed from histogram observations  
Given 100 stage latency observations where 99 are 10 ms and 1 is 500 ms,  
When `compute_sli().latency_p99_ms` is read,  
Then the value is between 400.0 and 600.0 (approximately the 500 ms outlier).

**AC-5.3** — Error rate computed from error counter and total counter  
Given 1000 total verify calls and 1 recorded stage error,  
When `compute_sli().error_rate_pct` is read,  
Then the value is approximately `0.1` (within ±0.01).

**AC-5.4** — Availability computed from accept/reject counts  
Given 999 accepted receipts and 1 rejected receipt,  
When `compute_sli().availability_pct` is read,  
Then the value is approximately `99.9` (within ±0.01).

**AC-5.5** — `check_slo()` returns `Ok(())` when all thresholds pass  
Given SLIs where `latency_p99_ms = 80`, `error_rate_pct = 0.05`, `availability_pct = 99.95`,  
When `collector.check_slo()` is called,  
Then it returns `Ok(())`.

**AC-5.6** — `check_slo()` returns `Err` when p99 latency breaches threshold  
Given SLIs where `latency_p99_ms = 150` (above the 100 ms threshold),  
When `collector.check_slo()` is called,  
Then it returns `Err(SloViolation::LatencyP99 { observed_ms: 150.0, threshold_ms: 100.0 })`.

**AC-5.7** — `check_slo()` returns `Err` when error rate breaches threshold  
Given SLIs where `error_rate_pct = 0.5` (above the 0.1% threshold),  
When `collector.check_slo()` is called,  
Then it returns `Err(SloViolation::ErrorRate { observed_pct: 0.5, threshold_pct: 0.1 })`.

**AC-5.8** — `check_slo()` returns `Err` when availability breaches threshold  
Given SLIs where `availability_pct = 99.5` (below the 99.9% threshold),  
When `collector.check_slo()` is called,  
Then it returns `Err(SloViolation::Availability { observed_pct: 99.5, threshold_pct: 99.9 })`.

**AC-5.9** — SLI gauge metrics are published after each compute  
Given a `MetricsCollector`,  
When `compute_sli()` is called,  
Then `affidavit_slo_p99_latency_ms`, `affidavit_slo_error_rate_pct`, and `affidavit_slo_availability_pct` gauge metrics are updated in the Prometheus registry.

**AC-5.10** — `compute_sli()` on an empty collector returns zero-state  
Given a freshly initialized `MetricsCollector` with no observations,  
When `compute_sli()` is called,  
Then it returns `Ok(ServiceLevelIndicators { latency_p99_ms: 0.0, error_rate_pct: 0.0, availability_pct: 100.0 })` without error.

### 5.2 SLI/SLO Reference Table

| SLI Name | Computation | SLO Threshold | Breach Direction | `SloViolation` Variant |
|---|---|---|---|---|
| `latency_p99_ms` | 99th percentile of `affidavit_stage_latency_ms` histogram over rolling 5-min window | ≤ 100 ms | above | `SloViolation::LatencyP99` |
| `error_rate_pct` | `(sum(affidavit_stage_errors_total) / sum(affidavit_receipts_verified_total)) * 100` | ≤ 0.1% | above | `SloViolation::ErrorRate` |
| `availability_pct` | `(affidavit_receipts_verified_total{verdict="accept"} / affidavit_receipts_verified_total) * 100` | ≥ 99.9% | below | `SloViolation::Availability` |

### 5.3 `ServiceLevelIndicators` and `SloViolation` Types

```rust
// src/metrics.rs (public API surface)

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceLevelIndicators {
    /// 99th percentile stage latency in milliseconds.
    pub latency_p99_ms: f64,
    /// Error rate as a percentage (0.0–100.0).
    pub error_rate_pct: f64,
    /// Availability as a percentage (0.0–100.0).
    pub availability_pct: f64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SloViolation {
    #[error("p99 latency SLO breach: observed {observed_ms}ms > threshold {threshold_ms}ms")]
    LatencyP99 { observed_ms: f64, threshold_ms: f64 },

    #[error("error rate SLO breach: observed {observed_pct:.3}% > threshold {threshold_pct:.3}%")]
    ErrorRate { observed_pct: f64, threshold_pct: f64 },

    #[error("availability SLO breach: observed {observed_pct:.3}% < threshold {threshold_pct:.3}%")]
    Availability { observed_pct: f64, threshold_pct: f64 },
}

pub const SLO_LATENCY_P99_MS: f64 = 100.0;
pub const SLO_ERROR_RATE_PCT: f64 = 0.1;
pub const SLO_AVAILABILITY_PCT: f64 = 99.9;
```

### 5.4 Test Strategy

- `compute_sli()` tests are pure unit tests in `src/metrics.rs` that inject observations via `MetricsCollector::record_*` and then assert the computed SLI values.
- Floating-point comparisons use `(observed - expected).abs() < epsilon` with `epsilon = 0.01`.
- `check_slo()` tests assert both the `Ok` and the three `Err` variants.
- An integration test in `tests/e2e_observability.rs` runs a full verify pipeline, calls `check_slo()`, and asserts `Ok(())` for a valid receipt under normal conditions.

### 5.5 Graceful Degradation

| Failure condition | Expected behavior |
|---|---|
| Rolling window has zero observations | `compute_sli()` returns zero-state (AC-5.10); no divide-by-zero panic |
| Histogram bucket data unavailable (SDK failure) | `latency_p99_ms` is `f64::NAN`; `check_slo()` treats NaN as a threshold breach and returns `Err(SloViolation::LatencyP99 { observed_ms: NAN, threshold_ms: 100.0 })` |
| Counter overflow (u64 wraps) | Observations are modular; no panic; SLI computation uses checked arithmetic |

---

## Cross-Cutting Concerns

### Feature Flag Structure

```toml
# Cargo.toml additions for Phase 4

[features]
default = []
otel = [
    "dep:opentelemetry",
    "dep:opentelemetry_sdk",
    "dep:opentelemetry-otlp",
]
metrics = [
    "otel",
    "dep:opentelemetry-prometheus",
    "dep:prometheus",
]

[dependencies]
opentelemetry        = { version = "0.20", features = ["trace", "baggage"], optional = true }
opentelemetry_sdk    = { version = "0.20", features = ["trace", "metrics"], optional = true }
opentelemetry-otlp   = { version = "0.13", features = ["grpc-tonic"], optional = true }
opentelemetry-prometheus = { version = "0.13", optional = true }
prometheus           = { version = "0.13", optional = true }

[dev-dependencies]
opentelemetry        = { version = "0.20", features = ["trace", "testing"] }
opentelemetry_sdk    = { version = "0.20", features = ["trace", "metrics", "testing"] }
```

### Compile-Time Invariants

- `cargo build` (no features): zero OTel code linked, binary size unchanged from Phase 3 baseline.
- `cargo build --features otel`: OTel trace + baggage code included; metrics code excluded.
- `cargo build --features metrics`: all observability code included.
- `cargo clippy --features otel -- -D warnings`: no new clippy warnings.
- `cargo clippy --features metrics -- -D warnings`: no new clippy warnings.

### No-Feature Backward Compatibility

The following public API signatures must not change regardless of feature flags:

- `affidavit::tracing::SpanRecord` struct (existing fields)
- `affidavit::tracing::captured_spans() -> Vec<SpanRecord>`
- `affidavit::tracing::clear_spans()`
- `affidavit::tracing::trace_verify<F, T>(receipt_path: &str, f: F) -> T`
- `affidavit::verifier::verify(receipt: &Receipt) -> Verdict`

### Documentation Requirements

- `src/tracing.rs`: all new public items under `feature = "otel"` have `/// # Feature gate\n/// Requires `feature = "otel"`.` in their doc comment.
- `src/metrics.rs`: module-level doc comment explaining the `MetricsCollector`, SLI computation, and SLO thresholds.
- `CLAUDE.md` "Integration Points" section updated with a "Observability (OTel)" subsection linking to Feature 1–5 and the dashboard file.
- `README.md` updated with a one-paragraph "Observability" section covering `--features otel` and `--features metrics`.

---

## E2E Test Template

File: `tests/e2e_observability.rs`

```rust
//! End-to-end observability tests for Phase 4 (OpenTelemetry & Observability).
//!
//! These tests require `feature = "otel"`. Run with:
//!   cargo test --features otel --test e2e_observability
//!
//! No real Jaeger or Prometheus instance is needed: all assertions use
//! in-memory exporters injected via the test-only `TracerProvider` API.

#![cfg(feature = "otel")]

use affidavit::chain::ChainAssembler;
use affidavit::tracing::{clear_spans, captured_spans};
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent};
use affidavit::verifier::verify;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;
use opentelemetry_sdk::trace::{Config, TracerProvider};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn make_event(id: &str, seq: u64, payload: &[u8]) -> OperationEvent {
    OperationEvent {
        id: id.to_string(),
        seq,
        event_type: "build".to_string(),
        objects: vec![ObjectRef {
            id: format!("repo:{id}"),
            obj_type: "git".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(payload),
    }
}

/// Assemble a valid 3-event receipt.
fn valid_receipt() -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    asm.append(make_event("e0", 0, b"payload-zero")).expect("append e0");
    asm.append(make_event("e1", 1, b"payload-one")).expect("append e1");
    asm.append(make_event("e2", 2, b"payload-two")).expect("append e2");
    asm.finalize()
}

/// Build an `InMemorySpanExporter` and a `TracerProvider` backed by it.
/// Returns `(provider, exporter)`.
fn build_test_provider() -> (TracerProvider, InMemorySpanExporter) {
    let exporter = InMemorySpanExporter::default();
    let provider = TracerProvider::builder()
        .with_config(Config::default())
        .with_simple_exporter(exporter.clone())
        .build();
    (provider, exporter)
}

// ---------------------------------------------------------------------------
// Feature 1: Verifier Stage Spans
// ---------------------------------------------------------------------------

#[test]
fn f1_parent_span_emitted_with_correct_name() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let parent = spans
        .iter()
        .find(|s| s.name == "affi.verify")
        .expect("parent span 'affi.verify' must be present");

    assert_eq!(parent.status, opentelemetry_sdk::trace::Status::Ok);
    exporter.reset();
}

#[test]
fn f1_six_child_stage_spans_emitted() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");

    let expected_stages = [
        "affi.verify.decode",
        "affi.verify.check_format",
        "affi.verify.chain_integrity",
        "affi.verify.continuity",
        "affi.verify.verify_commitments",
        "affi.verify.evaluate_profile",
    ];

    for expected_name in &expected_stages {
        assert!(
            spans.iter().any(|s| s.name == *expected_name),
            "missing child span: {expected_name}"
        );
    }

    let child_count = spans.iter().filter(|s| s.name != "affi.verify").count();
    assert_eq!(child_count, 6, "expected exactly 6 child stage spans, got {child_count}");
    exporter.reset();
}

#[test]
fn f1_all_spans_share_same_trace_id() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    assert!(!spans.is_empty(), "at least one span must be emitted");

    let trace_id = spans[0].span_context.trace_id();
    for span in &spans {
        assert_eq!(
            span.span_context.trace_id(),
            trace_id,
            "span '{}' has a different trace_id",
            span.name
        );
    }
    exporter.reset();
}

#[test]
fn f1_child_spans_have_parent_span_as_parent() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let parent = spans
        .iter()
        .find(|s| s.name == "affi.verify")
        .expect("parent span must be present");
    let parent_span_id = parent.span_context.span_id();

    let children: Vec<_> = spans.iter().filter(|s| s.name != "affi.verify").collect();
    assert_eq!(children.len(), 6);

    for child in children {
        let parent_id = child.parent_span_id;
        assert_eq!(
            parent_id, parent_span_id,
            "child span '{}' parent_span_id mismatch",
            child.name
        );
    }
    exporter.reset();
}

#[test]
fn f1_tampered_receipt_chain_integrity_span_is_error() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let mut receipt = valid_receipt();
    // Tamper event 1 payload commitment without recomputing chain hash.
    receipt.events[1].payload_commitment = Blake3Hash::from_bytes(b"tampered-payload");

    affidavit::tracing::trace_verify_with_stages("tampered.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let chain_span = spans
        .iter()
        .find(|s| s.name == "affi.verify.chain_integrity")
        .expect("chain_integrity span must be present");

    assert!(
        matches!(chain_span.status, opentelemetry_sdk::trace::Status::Error { .. }),
        "chain_integrity span must have Error status for tampered receipt"
    );
    exporter.reset();
}

#[test]
fn f1_verdict_accepted_attribute_on_parent_span() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let parent = spans
        .iter()
        .find(|s| s.name == "affi.verify")
        .expect("parent span must be present");

    let verdict_attr = parent
        .attributes
        .iter()
        .find(|(k, _)| k.as_str() == "verdict.accepted")
        .expect("verdict.accepted attribute must be present on parent span");

    assert_eq!(
        verdict_attr.1,
        opentelemetry::Value::Bool(true),
        "verdict.accepted must be true for a valid receipt"
    );
    exporter.reset();
}

// ---------------------------------------------------------------------------
// Feature 3: Baggage Propagation
// ---------------------------------------------------------------------------

#[test]
fn f3_receipt_id_baggage_equals_chain_hash() {
    let receipt = valid_receipt();
    let expected_receipt_id = receipt.chain_hash.as_hex().to_string();

    let observed_receipt_id = affidavit::tracing::capture_baggage_value_for_test(
        &receipt,
        "receipt_id",
        || verify(&receipt),
    );

    assert_eq!(
        observed_receipt_id,
        Some(expected_receipt_id),
        "baggage[receipt_id] must equal receipt.chain_hash"
    );
}

#[test]
fn f3_format_version_baggage_matches_receipt_field() {
    let receipt = valid_receipt();

    let observed = affidavit::tracing::capture_baggage_value_for_test(
        &receipt,
        "format_version",
        || verify(&receipt),
    );

    assert_eq!(
        observed,
        Some("core/v1".to_string()),
        "baggage[format_version] must equal receipt.format_version"
    );
}

#[test]
fn f3_receipt_fields_unchanged_after_baggage_set() {
    let receipt = valid_receipt();
    let original_hash = receipt.chain_hash.as_hex().to_string();
    let original_format = receipt.format_version.clone();
    let original_event_count = receipt.events.len();

    // baggage setting must not mutate the receipt
    let _ = affidavit::tracing::capture_baggage_value_for_test(
        &receipt,
        "receipt_id",
        || verify(&receipt),
    );

    assert_eq!(receipt.chain_hash.as_hex(), original_hash);
    assert_eq!(receipt.format_version, original_format);
    assert_eq!(receipt.events.len(), original_event_count);
}

// ---------------------------------------------------------------------------
// Feature 4: Span Events
// ---------------------------------------------------------------------------

#[test]
fn f4_total_span_events_is_14() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let total_events: usize = spans.iter().map(|s| s.events.len()).sum();

    assert_eq!(
        total_events, 14,
        "expected 14 span events total (2 on parent + 2 per stage × 6 stages), got {total_events}"
    );
    exporter.reset();
}

#[test]
fn f4_stage_start_event_present_on_each_child_span() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let children: Vec<_> = spans.iter().filter(|s| s.name != "affi.verify").collect();

    for child in children {
        let has_start = child.events.iter().any(|e| e.name == "stage.start");
        assert!(has_start, "child span '{}' missing 'stage.start' event", child.name);
    }
    exporter.reset();
}

#[test]
fn f4_stage_end_event_present_on_each_child_span() {
    let (provider, exporter) = build_test_provider();
    let _guard = opentelemetry::global::set_tracer_provider(provider);

    let receipt = valid_receipt();
    affidavit::tracing::trace_verify_with_stages("receipt.json", |_hook| verify(&receipt));

    let spans = exporter.get_finished_spans().expect("get spans");
    let children: Vec<_> = spans.iter().filter(|s| s.name != "affi.verify").collect();

    for child in children {
        let has_end = child.events.iter().any(|e| e.name == "stage.end");
        assert!(has_end, "child span '{}' missing 'stage.end' event", child.name);
    }
    exporter.reset();
}

// ---------------------------------------------------------------------------
// Feature 5: SLO Monitoring
// ---------------------------------------------------------------------------

#[cfg(feature = "metrics")]
mod slo_tests {
    use affidavit::metrics::{MetricsCollector, SLO_AVAILABILITY_PCT, SLO_ERROR_RATE_PCT, SLO_LATENCY_P99_MS};

    #[test]
    fn f5_compute_sli_returns_zero_state_when_empty() {
        let collector = MetricsCollector::new_noop();
        let sli = collector.compute_sli().expect("compute_sli must not fail on empty collector");

        assert_eq!(sli.latency_p99_ms, 0.0, "empty latency_p99_ms must be 0.0");
        assert_eq!(sli.error_rate_pct, 0.0, "empty error_rate_pct must be 0.0");
        assert_eq!(sli.availability_pct, 100.0, "empty availability_pct must be 100.0");
    }

    #[test]
    fn f5_check_slo_passes_for_healthy_system() {
        let collector = MetricsCollector::new_noop();
        // Record 100 fast successful verifies.
        for _ in 0..100 {
            collector.record_stage_latency("chain_integrity", std::time::Duration::from_millis(10));
            collector.record_receipt_verified(true);
        }
        assert!(collector.check_slo().is_ok(), "healthy system must pass all SLOs");
    }

    #[test]
    fn f5_check_slo_fails_on_high_p99_latency() {
        let collector = MetricsCollector::new_noop();
        // Inject a 200ms latency observation to breach the 100ms threshold.
        collector.record_stage_latency("chain_integrity", std::time::Duration::from_millis(200));
        collector.record_receipt_verified(true);

        let result = collector.check_slo();
        assert!(
            matches!(result, Err(affidavit::metrics::SloViolation::LatencyP99 { .. })),
            "expected LatencyP99 violation, got: {result:?}"
        );
    }

    #[test]
    fn f5_check_slo_fails_on_high_error_rate() {
        let collector = MetricsCollector::new_noop();
        // Record 999 successes and 2 errors -> error_rate = 2/1001 ≈ 0.2%.
        for _ in 0..999 {
            collector.record_receipt_verified(true);
        }
        collector.record_stage_error("chain_integrity");
        collector.record_stage_error("chain_integrity");
        collector.record_receipt_verified(false);
        collector.record_receipt_verified(false);

        let result = collector.check_slo();
        assert!(
            matches!(result, Err(affidavit::metrics::SloViolation::ErrorRate { .. })),
            "expected ErrorRate violation, got: {result:?}"
        );
    }

    #[test]
    fn f5_check_slo_fails_on_low_availability() {
        let collector = MetricsCollector::new_noop();
        // 99 accepts, 1 reject = 99.0% availability, breaching the 99.9% threshold.
        for _ in 0..99 {
            collector.record_receipt_verified(true);
        }
        collector.record_receipt_verified(false);

        let result = collector.check_slo();
        assert!(
            matches!(result, Err(affidavit::metrics::SloViolation::Availability { .. })),
            "expected Availability violation, got: {result:?}"
        );
    }

    #[test]
    fn f5_slo_constants_match_spec() {
        assert_eq!(SLO_LATENCY_P99_MS, 100.0, "p99 threshold must be 100ms");
        assert_eq!(SLO_ERROR_RATE_PCT, 0.1, "error rate threshold must be 0.1%");
        assert_eq!(SLO_AVAILABILITY_PCT, 99.9, "availability threshold must be 99.9%");
    }
}

// ---------------------------------------------------------------------------
// Thread-local backward-compatibility check (no-feature path)
// ---------------------------------------------------------------------------

/// This test runs even without `feature = "otel"` to ensure the existing
/// thread-local span recorder still works after Phase 4 changes.
#[test]
fn backward_compat_thread_local_recorder_still_works() {
    clear_spans();
    let receipt = valid_receipt();
    affidavit::tracing::trace_verify("receipt.json", || {
        let _ = verify(&receipt);
    });
    let spans = captured_spans();
    assert!(
        spans.iter().any(|s| s.operation == "verify"),
        "thread-local recorder must still capture 'verify' spans after Phase 4 changes"
    );
}
```

---

## Reference Tables

### Span Attribute Name Table

| Attribute Key | Value Type | Description | Present On |
|---|---|---|---|
| `receipt.path` | `string` | Absolute file path of the receipt | `affi.verify` (parent) |
| `receipt.event_count` | `int` | Number of `OperationEvent`s in the receipt | `affi.verify` (parent) |
| `receipt.format_version` | `string` | `receipt.format_version` field value | `affi.verify` (parent) |
| `verdict.accepted` | `bool` | `true` = ACCEPT, `false` = REJECT | `affi.verify` (parent) |
| `verdict.reason` | `string` | `verdict.reason` field from `Verdict` | `affi.verify` (parent) |
| `stage.index` | `int` | 0-based position in the pipeline | all child spans |
| `stage.name` | `string` | Stage name (matches `CheckOutcome.stage`) | all child spans |
| `stage.passed` | `bool` | Whether the stage's check passed | all child spans |
| `stage.detail` | `string` | `CheckOutcome.detail` text | all child spans |

### Baggage Key Table

| Key | Value Type | Encoding | Description |
|---|---|---|---|
| `receipt_id` | `string` | Raw hex (64 lowercase chars) | `receipt.chain_hash.as_hex()` |
| `timestamp` | `string` | ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) | Wall-clock time at verify-call start |
| `format_version` | `string` | Percent-encoded for W3C header (`core%2Fv1`) | `receipt.format_version` field |

### SLI/SLO Summary Table

| SLI | Formula | Threshold | Breach |
|---|---|---|---|
| p99 latency | 99th pct of `affidavit_stage_latency_ms` (5-min window) | ≤ 100 ms | Above |
| error rate | `(stage_errors / receipts_verified) × 100` | ≤ 0.1% | Above |
| availability | `(accepts / receipts_verified) × 100` | ≥ 99.9% | Below |

### Prometheus Metrics Quick Reference

| Metric | Type | Labels | Unit |
|---|---|---|---|
| `affidavit_stage_latency_ms` | Histogram | `stage` | ms |
| `affidavit_stage_errors_total` | Counter | `stage` | count |
| `affidavit_receipts_verified_total` | Counter | `verdict` | count |
| `affidavit_verify_duration_ms` | Histogram | — | ms |
| `affidavit_chain_events_total` | Histogram | — | count |
| `affidavit_active_verify_ops` | Gauge | — | count |
| `affidavit_slo_p99_latency_ms` | Gauge | — | ms |
| `affidavit_slo_error_rate_pct` | Gauge | — | % |
| `affidavit_slo_availability_pct` | Gauge | — | % |

### Stage Name to Pipeline Index Mapping

| Stage Name | `stage.index` | `CheckOutcome.stage` value | Failing span status |
|---|---|---|---|
| `decode` | 0 | `"decode"` | `Error("format_version is empty or unparseable")` |
| `check_format` | 1 | `"check_format"` | `Error("expected format_version core/v1, found <actual>")` |
| `chain_integrity` | 2 | `"chain_integrity"` | `Error("chain hash mismatch: stored <x>, recomputed <y>")` |
| `continuity` | 3 | `"continuity"` | `Error("seq gap at position <N>: ...")` |
| `verify_commitments` | 4 | `"verify_commitments"` | `Error("event <id> has a malformed commitment")` |
| `evaluate_profile` | 5 | `"evaluate_profile"` | `Error("event <id> has an empty event_type")` |

---

## Definition of "Done" Checklist

Phase 4 is done when **every item below is checked**:

### Code

- [ ] `src/tracing.rs` compiles clean with `--features otel` and without features
- [ ] `src/metrics.rs` (new file) compiles clean with `--features metrics`
- [ ] `dashboards/affidavit.json` exists and passes `serde_json::from_str` validation
- [ ] `tests/e2e_observability.rs` exists and all tests pass under `--features otel`
- [ ] All 30 existing tests pass with no features enabled (`cargo test`)
- [ ] All 30 existing tests pass with `--features otel` enabled
- [ ] `cargo clippy --features metrics -- -D warnings` reports zero warnings
- [ ] No `unwrap()` in non-test `feature = "otel"` or `feature = "metrics"` code paths

### Acceptance Criteria

- [ ] All 10 AC for Feature 1 (Verifier Stage Spans) pass
- [ ] All 10 AC for Feature 2 (OTel Metrics Dashboard) pass
- [ ] All 10 AC for Feature 3 (OTel Baggage Propagation) pass
- [ ] All 10 AC for Feature 4 (Span Events) pass
- [ ] All 10 AC for Feature 5 (SLO Monitoring) pass

### Test Coverage

- [ ] `f1_parent_span_emitted_with_correct_name` passes
- [ ] `f1_six_child_stage_spans_emitted` passes
- [ ] `f1_all_spans_share_same_trace_id` passes
- [ ] `f1_child_spans_have_parent_span_as_parent` passes
- [ ] `f1_tampered_receipt_chain_integrity_span_is_error` passes
- [ ] `f1_verdict_accepted_attribute_on_parent_span` passes
- [ ] `f3_receipt_id_baggage_equals_chain_hash` passes
- [ ] `f3_format_version_baggage_matches_receipt_field` passes
- [ ] `f3_receipt_fields_unchanged_after_baggage_set` passes
- [ ] `f4_total_span_events_is_14` passes
- [ ] `f4_stage_start_event_present_on_each_child_span` passes
- [ ] `f4_stage_end_event_present_on_each_child_span` passes
- [ ] `f5_compute_sli_returns_zero_state_when_empty` passes
- [ ] `f5_check_slo_passes_for_healthy_system` passes
- [ ] `f5_check_slo_fails_on_high_p99_latency` passes
- [ ] `f5_check_slo_fails_on_high_error_rate` passes
- [ ] `f5_check_slo_fails_on_low_availability` passes
- [ ] `f5_slo_constants_match_spec` passes
- [ ] `backward_compat_thread_local_recorder_still_works` passes

### Invariants

- [ ] `affidavit::verifier::verify` verdict is identical with and without `--features otel` for the same receipt bytes
- [ ] `Receipt._seal` remains private and `cargo test --test ui` (trybuild) still passes
- [ ] `cargo build` (no features) binary size does not increase by more than 1 KB from Phase 3 baseline
- [ ] `OTEL_EXPORTER_OTLP_ENDPOINT` unset → `affi` falls back to `http://localhost:4317` and logs one `INFO` line; no panic

### Documentation

- [ ] All new public items in `src/tracing.rs` have doc comments stating the feature gate
- [ ] `src/metrics.rs` has a module-level doc comment covering `MetricsCollector`, SLI computation, and SLO thresholds
- [ ] `CLAUDE.md` "Integration Points" section updated with "Observability (OTel)" subsection
- [ ] `README.md` "Observability" paragraph added covering `--features otel` and `--features metrics`

---

*Definition of Done — Phase 4: OpenTelemetry & Observability*  
*affidavit v26.6.14 | Branch: `claude/zen-cerf-oq87br` | 2026-06-14*
