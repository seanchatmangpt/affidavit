// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chicago TDD Telemetry and Observability System
//!
//! Provides distributed tracing, metrics aggregation, W3C traceparent propagation,
//! and autonomic telemetry envelope formatting.

use crate::error::NounVerbError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Helper to generate unique hex identifiers
fn generate_hex_id(len: usize) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    let time = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut state = time as u64 ^ count;
    let mut hex = String::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (state >> 60) as u8;
        hex.push(char::from_digit(val as u32, 16).unwrap_or('0'));
    }
    hex
}

/// Autonomic Telemetry Envelope Payload Format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomicTelemetryEnvelope<T> {
    pub schema_version: String,
    pub cli_version: String,
    pub timestamp: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub payload: T,
}

impl<T> AutonomicTelemetryEnvelope<T> {
    pub fn new(
        schema_version: &str,
        cli_version: &str,
        trace_id: String,
        span_id: Option<String>,
        payload: T,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());
        Self {
            schema_version: schema_version.to_string(),
            cli_version: cli_version.to_string(),
            timestamp,
            trace_id,
            span_id,
            payload,
        }
    }
}

/// W3C Trace Context representation
#[derive(Debug, Clone)]
pub struct TraceContext {
    trace_id: String,
    span_id: String,
    sampled: bool,
    baggage: HashMap<String, String>,
}

impl TraceContext {
    /// Create a new, randomized TraceContext
    pub fn new() -> Result<Self, NounVerbError> {
        Ok(Self {
            trace_id: generate_hex_id(32),
            span_id: generate_hex_id(16),
            sampled: true,
            baggage: HashMap::new(),
        })
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    pub fn is_sampled(&self) -> bool {
        self.sampled
    }

    pub fn set_sampled(&mut self, sampled: bool) {
        self.sampled = sampled;
    }

    pub fn set_baggage(&mut self, key: &str, value: &str) {
        self.baggage.insert(key.to_string(), value.to_string());
    }

    pub fn get_baggage(&self, key: &str) -> Option<String> {
        self.baggage.get(key).cloned()
    }

    /// Serialize context to a W3C traceparent header value
    pub fn to_traceparent(&self) -> Result<String, NounVerbError> {
        let flags = if self.sampled { "01" } else { "00" };
        Ok(format!("00-{}-{}-{}", self.trace_id, self.span_id, flags))
    }

    /// Parse a W3C traceparent header value
    pub fn from_traceparent(traceparent: &str) -> Result<Self, NounVerbError> {
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() != 4 {
            return Err(NounVerbError::TelemetryError("Invalid W3C traceparent format".into()));
        }
        let version = parts[0];
        let trace_id = parts[1];
        let span_id = parts[2];
        let flags = parts[3];

        if version != "00" {
            return Err(NounVerbError::TelemetryError(format!(
                "Unsupported traceparent version: {}",
                version
            )));
        }
        if trace_id.len() != 32 || !trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(NounVerbError::TelemetryError("Invalid trace_id in traceparent".into()));
        }
        if span_id.len() != 16 || !span_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(NounVerbError::TelemetryError("Invalid span_id in traceparent".into()));
        }
        if flags.len() != 2 || !flags.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(NounVerbError::TelemetryError("Invalid flags in traceparent".into()));
        }

        let sampled = match flags {
            "01" => true,
            "00" => false,
            _ => {
                if let Ok(val) = u8::from_str_radix(flags, 16) {
                    (val & 1) != 0
                } else {
                    false
                }
            }
        };

        Ok(Self {
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            sampled,
            baggage: HashMap::new(),
        })
    }

    /// Extracts a TraceContext by parsing the W3C traceparent from either
    /// environment variables (TRACEPARENT/traceparent) or command-line flags (--traceparent).
    pub fn extract_from_env_or_flags() -> Option<Self> {
        if let Ok(val) = std::env::var("TRACEPARENT") {
            if let Ok(ctx) = Self::from_traceparent(&val) {
                return Some(ctx);
            }
        }
        if let Ok(val) = std::env::var("traceparent") {
            if let Ok(ctx) = Self::from_traceparent(&val) {
                return Some(ctx);
            }
        }

        let args: Vec<String> = std::env::args().collect();
        for i in 0..args.len() {
            if args[i] == "--traceparent" && i + 1 < args.len() {
                if let Ok(ctx) = Self::from_traceparent(&args[i + 1]) {
                    return Some(ctx);
                }
            } else if args[i].starts_with("--traceparent=") {
                let parts: Vec<&str> = args[i].splitn(2, '=').collect();
                if parts.len() == 2 {
                    if let Ok(ctx) = Self::from_traceparent(parts[1]) {
                        return Some(ctx);
                    }
                }
            }
        }
        None
    }
}

/// Observability Span for execution tracking
#[derive(Debug, Clone)]
pub struct Span {
    name: String,
    id: String,
    parent_id: Option<String>,
    status: String,
    has_error: bool,
    events: Vec<(String, String)>,
    start_time: Instant,
    duration: Duration,
    attributes: HashMap<String, String>,
}

impl Span {
    pub fn new(name: &str) -> Result<Self, NounVerbError> {
        Ok(Self {
            name: name.to_string(),
            id: generate_hex_id(16),
            parent_id: None,
            status: "unset".to_string(),
            has_error: false,
            events: Vec::new(),
            start_time: Instant::now(),
            duration: Duration::ZERO,
            attributes: HashMap::new(),
        })
    }

    pub fn new_with_parent(name: &str, parent: &Span) -> Result<Self, NounVerbError> {
        Ok(Self {
            name: name.to_string(),
            id: generate_hex_id(16),
            parent_id: Some(parent.id()),
            status: "unset".to_string(),
            has_error: false,
            events: Vec::new(),
            start_time: Instant::now(),
            duration: Duration::ZERO,
            attributes: HashMap::new(),
        })
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn parent_id(&self) -> Option<String> {
        self.parent_id.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
        if status == "error" {
            self.has_error = true;
        }
    }

    pub fn set_error(&mut self, error: &str) {
        self.status = "error".to_string();
        self.has_error = true;
        self.add_event("error", error);
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn add_event(&mut self, name: &str, details: &str) {
        self.events.push((name.to_string(), details.to_string()));
    }

    pub fn events(&self) -> Vec<(String, String)> {
        self.events.clone()
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    pub fn get_attribute(&self, key: &str) -> Option<String> {
        self.attributes.get(key).cloned()
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
}

/// Metrics Collector
pub struct Metrics {
    service_name: String,
    counters: RwLock<HashMap<String, i64>>,
    gauges: RwLock<HashMap<String, f64>>,
    histograms: RwLock<HashMap<String, Vec<f64>>>,
}

impl Metrics {
    pub fn new(service_name: &str) -> Result<Self, NounVerbError> {
        Ok(Self {
            service_name: service_name.to_string(),
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
        })
    }

    pub fn increment_counter(&self, name: &str) {
        self.add_to_counter(name, 1);
    }

    pub fn add_to_counter(&self, name: &str, value: i64) {
        let mut counters = self.counters.write();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn get_counter(&self, name: &str) -> i64 {
        let counters = self.counters.read();
        counters.get(name).copied().unwrap_or(0)
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write();
        gauges.insert(name.to_string(), value);
    }

    pub fn get_gauge(&self, name: &str) -> f64 {
        let gauges = self.gauges.read();
        gauges.get(name).copied().unwrap_or(0.0)
    }

    pub fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write();
        histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
    }

    pub fn get_histogram_avg(&self, name: &str) -> f64 {
        let histograms = self.histograms.read();
        if let Some(values) = histograms.get(name) {
            if values.is_empty() {
                0.0
            } else {
                let sum: f64 = values.iter().sum();
                sum / values.len() as f64
            }
        } else {
            0.0
        }
    }

    pub fn get_percentile(&self, name: &str, percentile: f64) -> f64 {
        let histograms = self.histograms.read();
        if let Some(values) = histograms.get(name) {
            if values.is_empty() {
                return 0.0;
            }
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let index = ((percentile / 100.0) * (sorted.len() - 1) as f64).round() as usize;
            sorted.get(index).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    pub fn reset(&self) {
        self.counters.write().clear();
        self.gauges.write().clear();
        self.histograms.write().clear();
    }

    pub fn export(&self) -> Result<String, NounVerbError> {
        #[derive(Serialize)]
        struct ExportedMetrics {
            service_name: String,
            counters: HashMap<String, i64>,
            gauges: HashMap<String, f64>,
            histograms_avg: HashMap<String, f64>,
        }

        let counters = self.counters.read().clone();
        let gauges = self.gauges.read().clone();

        let mut histograms_avg = HashMap::new();
        {
            let histograms = self.histograms.read();
            for (k, v) in histograms.iter() {
                let avg = if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
                histograms_avg.insert(k.clone(), avg);
            }
        }

        let export_data = ExportedMetrics {
            service_name: self.service_name.clone(),
            counters,
            gauges,
            histograms_avg,
        };

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| NounVerbError::TelemetryError(e.to_string()))
    }
}

/// Telemetry Manager coordinating tracing lifecycle
pub struct TelemetryManager {
    app_name: String,
}

impl TelemetryManager {
    pub fn new(app_name: &str) -> Result<Self, NounVerbError> {
        Ok(Self { app_name: app_name.to_string() })
    }

    pub fn start_span(&self, name: &str) -> Result<Span, NounVerbError> {
        Span::new(name)
    }

    pub fn start_child_span(&self, parent: &Span, name: &str) -> Result<Span, NounVerbError> {
        Span::new_with_parent(name, parent)
    }

    pub fn end_span(&self, mut span: Span) -> Result<Span, NounVerbError> {
        span.duration = span.start_time.elapsed();
        // Envelope logging simulation
        let envelope = AutonomicTelemetryEnvelope::new(
            "1.0.0",
            "3.8.0",
            span.id(),
            None,
            format!("span_ended: {}", span.name()),
        );
        if let Ok(json) = serde_json::to_string(&envelope) {
            // Emitted log simulation
            let _ = json;
        }
        Ok(span)
    }

    pub fn create_trace_context(&self) -> Result<TraceContext, NounVerbError> {
        TraceContext::new()
    }

    pub fn inject_context(
        &self,
        context: &TraceContext,
    ) -> Result<HashMap<String, String>, NounVerbError> {
        let mut headers = HashMap::new();
        headers.insert("traceparent".to_string(), context.to_traceparent()?);
        for (k, v) in &context.baggage {
            headers.insert(format!("otbaggage-{}", k), v.clone());
        }
        Ok(headers)
    }

    pub fn extract_context(
        &self,
        headers: &HashMap<String, String>,
    ) -> Result<TraceContext, NounVerbError> {
        if let Some(tp) = headers.get("traceparent") {
            let mut ctx = TraceContext::from_traceparent(tp)?;
            for (k, v) in headers {
                if let Some(key) = k.strip_prefix("otbaggage-") {
                    ctx.set_baggage(key, v);
                }
            }
            Ok(ctx)
        } else {
            Err(NounVerbError::TelemetryError("No traceparent header found".into()))
        }
    }

    pub fn record_event(
        &self,
        span: &Span,
        name: &str,
        details: &str,
    ) -> Result<(), NounVerbError> {
        let envelope = AutonomicTelemetryEnvelope::new(
            "1.0.0",
            "3.8.0",
            span.id(),
            None,
            (name.to_string(), details.to_string()),
        );
        if let Ok(json) = serde_json::to_string(&envelope) {
            let _ = json;
        }
        Ok(())
    }
}
