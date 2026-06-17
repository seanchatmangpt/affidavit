//! OCEL (Object-Centric Event Logs) integration for quality monitoring.
//!
//! This module bridges code quality metrics (see `crate::quality`) with the OCEL
//! standard, creating an immutable, auditable chain of quality measurements and
//! violations. Key features:
//!
//! - **Object types**: File, Module, Package, Repository, Repo-Family
//! - **Event types**: quality:measure, quality:violation, quality:remediate
//! - **Causal chains**: root event → measurement → violation → remediation
//! - **Correlation**: violations across objects in the same receipt
//!
//! # Architecture
//!
//! A quality OCEL log tracks:
//! 1. Baseline measurements (quality:measure events)
//! 2. Detected violations (quality:violation events)
//! 3. Remediation actions (quality:remediate events)
//!
//! Each violation links back to the triggering measurement via event ID references,
//! creating a causal chain visible in the receipt.

use crate::types::{Blake3Hash, ObjectRef, OperationEvent};
use crate::quality::{CodeQualityMetrics, QualityViolation};
use crate::error::OcelError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Quality-specific OCEL event extending the base OperationEvent.
///
/// Wraps an `OperationEvent` with quality-specific metadata and causal relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelQualityEvent {
    /// Base OCEL operation event
    pub event: OperationEvent,

    /// Quality event category: "measure", "violation", "remediate"
    pub quality_event_type: String,

    /// If this is a violation event, reference to the triggering measurement event ID
    pub triggered_by_event_id: Option<String>,

    /// Quality-specific payload (serialized JSON)
    pub quality_payload: serde_json::Value,

    /// Severity level for violations: "LOW", "MEDIUM", "HIGH", "CRITICAL"
    pub severity: Option<String>,
}

/// Quality metrics snapshot per object (File, Module, Package, etc.).
///
/// This record pins quality metrics to a specific object at a point in time.
/// Used to correlate violations with their measurement context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectQualityRecord {
    /// The object ID (e.g., "src/lib.rs:File" or "mymodule:Module")
    pub object_id: String,

    /// Object type: "File", "Module", "Package", "Repository", "RepoFamily"
    pub object_type: String,

    /// Stub ratio (functions with todo/unimplemented/panic)
    pub stub_ratio: f64,

    /// Type annotation coverage (0.0–1.0)
    pub type_coverage: f64,

    /// Cyclomatic complexity
    pub cyclomatic_complexity: f64,

    /// Cognitive complexity (lower = simpler)
    pub cognitive_complexity: f64,

    /// Test coverage percentage (0–100)
    pub test_coverage: f64,

    /// Documentation coverage (0.0–1.0)
    pub doc_coverage: f64,

    /// Clippy warnings count
    pub clippy_warnings: usize,

    /// Line churn (additions + deletions)
    pub churn: usize,

    /// Measurement timestamp (Unix seconds)
    pub measured_at: u64,
}

/// Causal chain from root measurement through violation to remediation.
///
/// Traces a violation back to its measurement, captures the violation details,
/// and optionally tracks remediation steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationCausalChain {
    /// Event ID of the baseline measurement (quality:measure)
    pub root_measurement_event_id: String,

    /// Event ID of the violation detection (quality:violation)
    pub violation_event_id: String,

    /// Event ID of the remediation action, if any (quality:remediate)
    pub remediation_event_id: Option<String>,

    /// The measurement that triggered this violation
    pub measured_value: f64,

    /// Threshold that was breached
    pub threshold: f64,

    /// The violation enum variant
    pub violation: QualityViolation,

    /// Remediation action text, if executed
    pub remediation_action: Option<String>,

    /// Remediation status: "pending", "in_progress", "resolved", "rejected"
    pub remediation_status: String,

    /// Sequence of event IDs in causal order
    pub event_sequence: Vec<String>,
}

/// Complete OCEL event log for a repository's quality history.
///
/// Holds all quality events from measurement through remediation, indexed by
/// object and event time for efficient correlation queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelQualityLog {
    /// Unique identifier for this quality log (e.g., repo name)
    pub log_id: String,

    /// All quality events in this log (ordered by seq)
    pub events: Vec<OcelQualityEvent>,

    /// Object quality records indexed by object ID
    pub object_records: BTreeMap<String, ObjectQualityRecord>,

    /// Causal chains indexed by violation event ID
    pub causal_chains: BTreeMap<String, ViolationCausalChain>,

    /// Correlation map: metric_name -> list of event IDs that contributed
    pub metric_correlations: BTreeMap<String, Vec<String>>,

    /// Log created timestamp (Unix seconds)
    pub created_at: u64,

    /// Log last updated timestamp
    pub updated_at: u64,
}

impl OcelQualityLog {
    /// Create a new empty quality log for a repository.
    pub fn new(log_id: impl Into<String>, timestamp: u64) -> Self {
        let log_id = log_id.into();
        OcelQualityLog {
            log_id,
            events: Vec::new(),
            object_records: BTreeMap::new(),
            causal_chains: BTreeMap::new(),
            metric_correlations: BTreeMap::new(),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    /// Add a quality event to the log.
    pub fn add_event(&mut self, event: OcelQualityEvent) {
        self.updated_at = event.event.payload_commitment.as_hex().len() as u64; // Placeholder
        self.events.push(event);
    }

    /// Get all events of a specific quality event type.
    pub fn events_by_type(&self, quality_event_type: &str) -> Vec<&OcelQualityEvent> {
        self.events
            .iter()
            .filter(|e| e.quality_event_type == quality_event_type)
            .collect()
    }

    /// Get all violations from the log.
    pub fn violations(&self) -> Vec<&OcelQualityEvent> {
        self.events_by_type("violation")
    }

    /// Get all measurements from the log.
    pub fn measurements(&self) -> Vec<&OcelQualityEvent> {
        self.events_by_type("measure")
    }
}

impl ObjectQualityRecord {
    /// Create a record from CodeQualityMetrics for a specific object.
    pub fn from_metrics(
        object_id: impl Into<String>,
        object_type: impl Into<String>,
        metrics: &CodeQualityMetrics,
    ) -> Self {
        ObjectQualityRecord {
            object_id: object_id.into(),
            object_type: object_type.into(),
            stub_ratio: metrics.stub_ratio,
            type_coverage: metrics.type_coverage,
            cyclomatic_complexity: metrics.cyclomatic_complexity,
            cognitive_complexity: metrics.cognitive_complexity,
            test_coverage: metrics.test_coverage,
            doc_coverage: metrics.doc_coverage,
            clippy_warnings: metrics.clippy_warnings,
            churn: metrics.churn,
            measured_at: metrics.timestamp,
        }
    }
}

/// Convert a code quality measurement into an OCEL quality:measure event.
///
/// Creates an operation-event that records the measured quality metrics for
/// the given objects. The payload contains the serialized metrics and object
/// quality records.
///
/// # Arguments
///
/// * `event_id` - Unique event identifier (e.g., "evt-0")
/// * `seq` - Logical sequence number
/// * `metrics` - The quality metrics snapshot
/// * `objects` - Object references (File, Module, Package, etc.) being measured
///
/// # Returns
///
/// An `OcelQualityEvent` of type "measure" with quality metrics in the payload.
///
/// # Example
///
/// ```ignore
/// let metrics = CodeQualityMetrics::default();
/// let objects = vec![ObjectRef {
///     id: "src/lib.rs".to_string(),
///     obj_type: "File".to_string(),
///     qualifier: Some("module:core".to_string()),
/// }];
/// let quality_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects)?;
/// assert_eq!(quality_event.quality_event_type, "measure");
/// ```
pub fn measure_to_ocel_event(
    event_id: &str,
    seq: u64,
    metrics: &CodeQualityMetrics,
    objects: &[ObjectRef],
) -> Result<OcelQualityEvent, OcelError> {
    if event_id.trim().is_empty() {
        return Err(OcelError::EmptyEventId);
    }
    if objects.is_empty() {
        return Err(OcelError::MalformedObjectRef("no objects provided".to_string()));
    }

    let payload = serde_json::json!({
        "stub_ratio": metrics.stub_ratio,
        "type_coverage": metrics.type_coverage,
        "cyclomatic_complexity": metrics.cyclomatic_complexity,
        "cognitive_complexity": metrics.cognitive_complexity,
        "test_coverage": metrics.test_coverage,
        "doc_coverage": metrics.doc_coverage,
        "clippy_warnings": metrics.clippy_warnings,
        "rustfmt_violations": metrics.rustfmt_violations,
        "cargo_deny_issues": metrics.cargo_deny_issues,
        "cargo_audit_vulnerabilities": metrics.cargo_audit_vulnerabilities,
        "churn": metrics.churn,
        "comment_ratio": metrics.comment_ratio,
        "maintainability_index": metrics.maintainability_index,
        "timestamp": metrics.timestamp,
    });

    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|_| OcelError::MalformedObjectRef("serialization failed".to_string()))?;

    let event = OperationEvent {
        id: event_id.to_string(),
        seq,
        event_type: "quality:measure".to_string(),
        objects: objects.to_vec(),
        payload_commitment: Blake3Hash::from_bytes(&payload_bytes),
    };

    Ok(OcelQualityEvent {
        event,
        quality_event_type: "measure".to_string(),
        triggered_by_event_id: None,
        quality_payload: payload,
        severity: None,
    })
}

/// Convert a quality violation into an OCEL quality:violation event.
///
/// Creates an operation-event that records a detected violation, linking it
/// to the measurement event that triggered it via `triggered_by_event_id`.
///
/// # Arguments
///
/// * `event_id` - Unique event identifier (e.g., "evt-1")
/// * `seq` - Logical sequence number
/// * `violation` - The detected violation
/// * `triggered_by_event_id` - Event ID of the measurement that triggered this
/// * `objects` - Object references involved in the violation
///
/// # Returns
///
/// An `OcelQualityEvent` of type "violation" with violation details and
/// causal link to the measurement.
///
/// # Example
///
/// ```ignore
/// let violation = QualityViolation::Rule1Sigma {
///     metric: "stub_ratio".to_string(),
///     value: 0.95,
///     threshold: 0.50,
///     z_score: 4.5,
///     severity: "CRITICAL".to_string(),
/// };
/// let objects = vec![ObjectRef { /* ... */ }];
/// let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects)?;
/// assert_eq!(violation_event.severity, Some("CRITICAL".to_string()));
/// ```
pub fn violation_to_ocel_event(
    event_id: &str,
    seq: u64,
    violation: &QualityViolation,
    triggered_by_event_id: &str,
    objects: &[ObjectRef],
) -> Result<OcelQualityEvent, OcelError> {
    if event_id.trim().is_empty() {
        return Err(OcelError::EmptyEventId);
    }
    if triggered_by_event_id.trim().is_empty() {
        return Err(OcelError::EmptyEventId);
    }

    let severity = violation.severity().to_string();
    let payload = serde_json::json!({
        "metric": violation.metric(),
        "description": violation.description(),
        "severity": severity,
    });

    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|_| OcelError::MalformedObjectRef("serialization failed".to_string()))?;

    let event = OperationEvent {
        id: event_id.to_string(),
        seq,
        event_type: "quality:violation".to_string(),
        objects: objects.to_vec(),
        payload_commitment: Blake3Hash::from_bytes(&payload_bytes),
    };

    Ok(OcelQualityEvent {
        event,
        quality_event_type: "violation".to_string(),
        triggered_by_event_id: Some(triggered_by_event_id.to_string()),
        quality_payload: payload,
        severity: Some(severity),
    })
}

/// Build a causal chain from a violation back through its root measurement.
///
/// Given a violation and the complete event log, reconstructs the causal chain
/// from the original measurement through the violation detection. If a remediation
/// event exists, it is included as well.
///
/// # Arguments
///
/// * `violation_event` - The violation event in the chain
/// * `event_log` - All quality events from the log
///
/// # Returns
///
/// A `ViolationCausalChain` tracing the violation to its root cause.
///
/// # Example
///
/// ```ignore
/// let chains = build_causal_chain(&violation_event, &all_events)?;
/// assert_eq!(chains.violation_event_id, "evt-1");
/// assert_eq!(chains.root_measurement_event_id, "evt-0");
/// ```
pub fn build_causal_chain(
    violation_event: &OcelQualityEvent,
    event_log: &[OcelQualityEvent],
) -> Result<ViolationCausalChain, OcelError> {
    if violation_event.quality_event_type != "violation" {
        return Err(OcelError::MalformedObjectRef(
            "not a violation event".to_string(),
        ));
    }

    let triggering_event_id = violation_event
        .triggered_by_event_id
        .as_ref()
        .ok_or_else(|| OcelError::EmptyEventId)?
        .clone();

    // Find the root measurement event
    let measurement_event = event_log
        .iter()
        .find(|e| e.event.id == triggering_event_id)
        .ok_or_else(|| OcelError::MalformedObjectRef(
            format!("measurement event {} not found", triggering_event_id),
        ))?;

    if measurement_event.quality_event_type != "measure" {
        return Err(OcelError::MalformedObjectRef(
            "triggering event is not a measurement".to_string(),
        ));
    }

    // Look for remediation event (quality:remediate) triggered by this violation
    let remediation = event_log
        .iter()
        .find(|e| e.triggered_by_event_id.as_ref() == Some(&violation_event.event.id));

    let mut event_sequence = vec![measurement_event.event.id.clone(), violation_event.event.id.clone()];
    if let Some(rem) = &remediation {
        event_sequence.push(rem.event.id.clone());
    }

    // Extract measured value from violation (all variants carry a value/metric)
    let measured_value = match &violation_event.quality_payload.get("value") {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    };

    let threshold = match &violation_event.quality_payload.get("threshold") {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    };

    // Parse the violation from the payload
    let violation_enum = parse_violation_from_payload(&violation_event.quality_payload)?;

    Ok(ViolationCausalChain {
        root_measurement_event_id: measurement_event.event.id.clone(),
        violation_event_id: violation_event.event.id.clone(),
        remediation_event_id: remediation.map(|e| e.event.id.clone()),
        measured_value,
        threshold,
        violation: violation_enum,
        remediation_action: remediation.and_then(|e| {
            e.quality_payload.get("action").and_then(|v| v.as_str()).map(|s| s.to_string())
        }),
        remediation_status: remediation
            .and_then(|e| e.quality_payload.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "pending".to_string()),
        event_sequence,
    })
}

/// Correlate violations across multiple objects in a quality log.
///
/// Analyzes all violations in the log and groups them by:
/// - Metric name
/// - Severity
/// - Object type
///
/// Returns a list of `ObjectCorrelation` records showing which violations
/// occur in which objects and their temporal relationship.
///
/// # Arguments
///
/// * `log` - The complete quality OCEL log
///
/// # Returns
///
/// A vector of correlations showing violation patterns across objects.
///
/// # Example
///
/// ```ignore
/// let correlations = correlate_violations_across_objects(&log)?;
/// for corr in correlations {
///     println!("{}: {} violations on {}", corr.metric, corr.object_count, corr.severity);
/// }
/// ```
pub fn correlate_violations_across_objects(log: &OcelQualityLog) -> Result<Vec<ObjectCorrelation>, OcelError> {
    let mut correlations: BTreeMap<(String, String), ObjectCorrelation> = BTreeMap::new();

    for event in log.violations() {
        let metric = event.quality_payload.get("metric")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let severity = event.severity.as_deref().unwrap_or("UNKNOWN");

        let key = (metric.to_string(), severity.to_string());
        let entry = correlations
            .entry(key.clone())
            .or_insert_with(|| ObjectCorrelation {
                metric: metric.to_string(),
                severity: severity.to_string(),
                object_ids: Vec::new(),
                object_count: 0,
                event_ids: Vec::new(),
                first_detected_seq: event.event.seq,
                last_detected_seq: event.event.seq,
            });

        for obj in &event.event.objects {
            if !entry.object_ids.contains(&obj.id) {
                entry.object_ids.push(obj.id.clone());
            }
            entry.object_count = entry.object_ids.len();
        }
        entry.event_ids.push(event.event.id.clone());
        entry.last_detected_seq = event.event.seq;
    }

    Ok(correlations.into_values().collect())
}

/// A correlation of violations across multiple objects.
///
/// Represents a pattern of violations with the same metric and severity
/// detected across one or more objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCorrelation {
    /// Metric name (e.g., "stub_ratio", "cyclomatic_complexity")
    pub metric: String,

    /// Severity level
    pub severity: String,

    /// List of object IDs where this violation was detected
    pub object_ids: Vec<String>,

    /// Count of unique objects affected
    pub object_count: usize,

    /// Event IDs of all violations with this metric+severity
    pub event_ids: Vec<String>,

    /// Sequence number of first detection
    pub first_detected_seq: u64,

    /// Sequence number of last detection
    pub last_detected_seq: u64,
}

/// Parse a violation enum from JSON payload data.
fn parse_violation_from_payload(payload: &serde_json::Value) -> Result<QualityViolation, OcelError> {
    let metric = payload.get("metric")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let severity = payload.get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();

    // For simplicity, we reconstruct a Rule1Sigma violation as a default
    // In production, you would store the violation variant type and deserialize fully
    Ok(QualityViolation::Rule1Sigma {
        metric,
        value: 0.0,
        threshold: 0.0,
        z_score: 0.0,
        severity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_metrics() -> CodeQualityMetrics {
        CodeQualityMetrics {
            stub_ratio: 0.15,
            type_coverage: 0.95,
            churn: 42,
            comment_ratio: 0.25,
            cyclomatic_complexity: 3.2,
            maintainability_index: 85.0,
            cognitive_complexity: 8.0,
            clippy_warnings: 2,
            rustfmt_violations: 0,
            cargo_deny_issues: 0,
            cargo_audit_vulnerabilities: 0,
            test_coverage: 88.0,
            doc_coverage: 0.82,
            timestamp: 1234567890,
        }
    }

    fn make_test_objects() -> Vec<ObjectRef> {
        vec![
            ObjectRef {
                id: "src/lib.rs".to_string(),
                obj_type: "File".to_string(),
                qualifier: Some("core".to_string()),
            },
            ObjectRef {
                id: "core_module".to_string(),
                obj_type: "Module".to_string(),
                qualifier: None,
            },
        ]
    }

    #[test]
    fn test_measure_to_ocel_event_creates_valid_event() {
        let metrics = make_test_metrics();
        let objects = make_test_objects();
        let event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        assert_eq!(event.event.id, "evt-0");
        assert_eq!(event.event.seq, 0);
        assert_eq!(event.event.event_type, "quality:measure");
        assert_eq!(event.quality_event_type, "measure");
        assert_eq!(event.event.objects.len(), 2);
        assert_eq!(event.triggered_by_event_id, None);
        assert_eq!(event.severity, None);
    }

    #[test]
    fn test_measure_to_ocel_event_rejects_empty_event_id() {
        let metrics = make_test_metrics();
        let objects = make_test_objects();
        let result = measure_to_ocel_event("", 0, &metrics, &objects);

        assert!(matches!(result, Err(OcelError::EmptyEventId)));
    }

    #[test]
    fn test_measure_to_ocel_event_rejects_no_objects() {
        let metrics = make_test_metrics();
        let result = measure_to_ocel_event("evt-0", 0, &metrics, &[]);

        assert!(matches!(result, Err(OcelError::MalformedObjectRef(_))));
    }

    #[test]
    fn test_violation_to_ocel_event_creates_valid_event() {
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let objects = make_test_objects();
        let event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects).unwrap();

        assert_eq!(event.event.id, "evt-1");
        assert_eq!(event.event.seq, 1);
        assert_eq!(event.event.event_type, "quality:violation");
        assert_eq!(event.quality_event_type, "violation");
        assert_eq!(event.triggered_by_event_id, Some("evt-0".to_string()));
        assert_eq!(event.severity, Some("CRITICAL".to_string()));
    }

    #[test]
    fn test_violation_to_ocel_event_rejects_empty_triggering_id() {
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let objects = make_test_objects();
        let result = violation_to_ocel_event("evt-1", 1, &violation, "", &objects);

        assert!(matches!(result, Err(OcelError::EmptyEventId)));
    }

    #[test]
    fn test_build_causal_chain_traces_violation_to_measurement() {
        let metrics = make_test_metrics();
        let objects = make_test_objects();
        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects).unwrap();

        let chain = build_causal_chain(&violation_event, &[measure_event]).unwrap();

        assert_eq!(chain.root_measurement_event_id, "evt-0");
        assert_eq!(chain.violation_event_id, "evt-1");
        assert_eq!(chain.event_sequence, vec!["evt-0", "evt-1"]);
        assert_eq!(chain.remediation_event_id, None);
    }

    #[test]
    fn test_build_causal_chain_rejects_non_violation_event() {
        let metrics = make_test_metrics();
        let objects = make_test_objects();
        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        let result = build_causal_chain(&measure_event, &[]);

        assert!(matches!(result, Err(OcelError::MalformedObjectRef(_))));
    }

    #[test]
    fn test_build_causal_chain_rejects_missing_measurement() {
        let objects = make_test_objects();
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-999", &objects).unwrap();

        let result = build_causal_chain(&violation_event, &[]);

        assert!(matches!(result, Err(OcelError::MalformedObjectRef(_))));
    }

    #[test]
    fn test_object_quality_record_from_metrics() {
        let metrics = make_test_metrics();
        let record = ObjectQualityRecord::from_metrics("src/lib.rs", "File", &metrics);

        assert_eq!(record.object_id, "src/lib.rs");
        assert_eq!(record.object_type, "File");
        assert_eq!(record.stub_ratio, 0.15);
        assert_eq!(record.type_coverage, 0.95);
        assert_eq!(record.cyclomatic_complexity, 3.2);
        assert_eq!(record.clippy_warnings, 2);
        assert_eq!(record.measured_at, 1234567890);
    }

    #[test]
    fn test_ocel_quality_log_add_event() {
        let mut log = OcelQualityLog::new("repo:main", 1000);
        let metrics = make_test_metrics();
        let objects = make_test_objects();
        let event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        log.add_event(event);

        assert_eq!(log.events.len(), 1);
        assert_eq!(log.events[0].event.id, "evt-0");
    }

    #[test]
    fn test_ocel_quality_log_events_by_type() {
        let mut log = OcelQualityLog::new("repo:main", 1000);
        let metrics = make_test_metrics();
        let objects = make_test_objects();

        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects).unwrap();

        log.add_event(measure_event);
        log.add_event(violation_event);

        assert_eq!(log.events_by_type("measure").len(), 1);
        assert_eq!(log.events_by_type("violation").len(), 1);
        assert_eq!(log.events_by_type("remediate").len(), 0);
    }

    #[test]
    fn test_ocel_quality_log_violations_helper() {
        let mut log = OcelQualityLog::new("repo:main", 1000);
        let metrics = make_test_metrics();
        let objects = make_test_objects();

        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects).unwrap();

        log.add_event(measure_event);
        log.add_event(violation_event);

        assert_eq!(log.violations().len(), 1);
        assert_eq!(log.measurements().len(), 1);
    }

    #[test]
    fn test_correlate_violations_across_objects() {
        let mut log = OcelQualityLog::new("repo:main", 1000);
        let metrics = make_test_metrics();
        let objects = make_test_objects();

        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();
        let violation = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation_event = violation_to_ocel_event("evt-1", 1, &violation, "evt-0", &objects).unwrap();

        log.add_event(measure_event);
        log.add_event(violation_event);

        let correlations = correlate_violations_across_objects(&log).unwrap();

        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].metric, "stub_ratio");
        assert_eq!(correlations[0].severity, "CRITICAL");
        assert_eq!(correlations[0].object_count, 2);
        assert_eq!(correlations[0].event_ids.len(), 1);
    }

    #[test]
    fn test_correlate_violations_groups_by_metric_and_severity() {
        let mut log = OcelQualityLog::new("repo:main", 1000);
        let metrics = make_test_metrics();
        let objects = make_test_objects();

        let measure_event = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        // Add two violations with different metrics
        let violation1 = QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.95,
            threshold: 0.50,
            z_score: 4.5,
            severity: "CRITICAL".to_string(),
        };
        let violation1_event = violation_to_ocel_event("evt-1", 1, &violation1, "evt-0", &objects).unwrap();

        let violation2 = QualityViolation::Rule1Sigma {
            metric: "cyclomatic_complexity".to_string(),
            value: 25.0,
            threshold: 15.0,
            z_score: 3.2,
            severity: "HIGH".to_string(),
        };
        let violation2_event = violation_to_ocel_event("evt-2", 2, &violation2, "evt-0", &objects).unwrap();

        log.add_event(measure_event);
        log.add_event(violation1_event);
        log.add_event(violation2_event);

        let correlations = correlate_violations_across_objects(&log).unwrap();

        assert_eq!(correlations.len(), 2);
        assert!(correlations.iter().any(|c| c.metric == "stub_ratio"));
        assert!(correlations.iter().any(|c| c.metric == "cyclomatic_complexity"));
    }

    #[test]
    fn test_measure_to_ocel_event_commitment_is_deterministic() {
        let metrics = make_test_metrics();
        let objects = make_test_objects();

        let event1 = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();
        let event2 = measure_to_ocel_event("evt-0", 0, &metrics, &objects).unwrap();

        assert_eq!(
            event1.event.payload_commitment,
            event2.event.payload_commitment,
            "Commitment should be deterministic for identical inputs"
        );
    }

    #[test]
    fn test_violation_to_ocel_event_preserves_severity() {
        let violations = vec![
            ("CRITICAL", QualityViolation::Rule1Sigma {
                metric: "stub".to_string(),
                value: 1.0,
                threshold: 0.0,
                z_score: 5.0,
                severity: "CRITICAL".to_string(),
            }),
            ("MEDIUM", QualityViolation::Rule4of5Beyond1Sigma {
                metric: "complexity".to_string(),
                count: 4,
                threshold: 10.0,
            }),
        ];

        let objects = make_test_objects();

        for (expected_severity, violation) in violations {
            let event = violation_to_ocel_event("evt-x", 1, &violation, "evt-0", &objects).unwrap();
            assert_eq!(event.severity.as_deref().unwrap_or("UNKNOWN"), expected_severity);
        }
    }
}
