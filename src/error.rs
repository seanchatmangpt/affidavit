use thiserror::Error;

/// Errors raised while building or validating OCEL events.
///
/// # Examples
///
/// ```rust
/// use affidavit::error::OcelError;
/// let err = OcelError::EmptyEventType;
/// assert_eq!(err.to_string(), "event_type must be non-empty");
/// ```
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OcelError {
    /// The event_type was empty or whitespace-only.
    #[error("event_type must be non-empty")]
    EmptyEventType,
    /// The event id was empty or whitespace-only.
    #[error("event id must be non-empty")]
    EmptyEventId,
    /// An object reference had an empty id.
    #[error("object ref at index {0} has an empty id")]
    EmptyObjectId(usize),
    /// An object reference had an empty obj_type.
    #[error("object ref at index {0} has an empty obj_type")]
    EmptyObjectType(usize),
    /// An object reference string could not be parsed as `id:type`.
    #[error("object ref '{0}' is not in 'id:type' or 'id:type:qualifier' form")]
    MalformedObjectRef(String),
}

/// Errors raised while assembling, serializing, or persisting receipts.
#[derive(Debug, Error)]
pub enum ChainError {
    /// Failure during canonical JSON encoding.
    #[error("canonical encoding failed: {0}")]
    Encode(#[source] serde_json::Error),
    /// Failure during receipt decoding.
    #[error("receipt decode failed: {0}")]
    Decode(#[source] serde_json::Error),
    /// Standard I/O failure.
    #[error("io error at {path}: {source}")]
    Io {
        /// File path where the error occurred.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Errors specific to quantum-resistant operations.
#[derive(Debug, Error)]
pub enum PqcError {
    /// Failure during PQC signature generation.
    #[error("PQC Signing failed: {0}")]
    Signing(String),
    /// Failure during PQC signature verification.
    #[error("PQC Verification failed: {0}")]
    Verification(String),
    /// Failure during KEM encapsulation.
    #[error("KEM Encapsulation failed: {0}")]
    Encapsulation(String),
    /// Underlying chain error.
    #[error("Chain error: {0}")]
    Chain(#[from] ChainError),
}

/// Errors in the discovery or Petri net mining pipeline.
#[derive(Debug, Error)]
pub enum MiningError {
    /// Wasm4pm HIM discovery failed.
    #[error("wasm4pm HIM failed: {0}")]
    Him(String),
    /// Admission was refused during mining.
    #[error("admission refused: {0}")]
    Admission(String),
    /// Conversion to OCEL format failed.
    #[error("OCEL conversion failed: {0}")]
    OcelConversion(String),
}

/// Failures in the distributed sharding and DHT verification layer.
#[derive(Debug, Error)]
pub enum ShardingError {
    /// Distributed Hash Table error.
    #[error("DHT error: {0}")]
    Dht(String),
    /// Chain integrity mismatch at a specific shard.
    #[error("Chain integrity failure at shard {index}: expected {expected}, found {found}")]
    ChainMismatch {
        /// Index of the failing shard.
        index: usize,
        /// Expected hex-encoded chain hash.
        expected: String,
        /// Actual hex-encoded chain hash.
        found: String,
    },
    /// Logical sequence mismatch at a specific shard.
    #[error("Continuity failure at shard {index}: expected seq {expected}, found {found}")]
    SeqMismatch {
        /// Index of the failing shard.
        index: usize,
        /// Expected sequence number.
        expected: u64,
        /// Actual sequence number.
        found: u64,
    },
    /// Inconsistency at the boundary between two shards.
    #[error("Shard boundary mismatch between {index} and {next}")]
    BoundaryMismatch {
        /// Index of the first shard.
        index: usize,
        /// Index of the following shard.
        next: usize,
    },
    /// Generic sharding failure.
    #[error("Verification failed: {0}")]
    Failure(String),
}

/// Errors that can occur during process activity prediction.
#[derive(Debug, Error)]
pub enum PredictionError {
    /// Wasm4pm discovery failed.
    #[error("wasm4pm discovery failed: {0}")]
    Wasm4pm(String),
    /// Admission refused during prediction.
    #[error("admission refused: {0}")]
    Admission(String),
    /// Invalid top-k parameter.
    #[error("invalid top-k: {0} (must be at least 1)")]
    InvalidTopK(usize),
}

/// Violations of Service Level Objectives.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SloViolation {
    /// P99 latency exceeded the defined threshold.
    #[error("p99 latency SLO breach: observed {observed_ms}ms > threshold {threshold_ms}ms")]
    LatencyP99 {
        /// Measured latency in milliseconds.
        observed_ms: f64,
        /// Defined threshold in milliseconds.
        threshold_ms: f64,
    },
    /// Error rate exceeded the defined threshold.
    #[error("error rate SLO breach: observed {observed_pct:.3}% > threshold {threshold_pct:.3}%")]
    ErrorRate {
        /// Measured error percentage.
        observed_pct: f64,
        /// Defined threshold percentage.
        threshold_pct: f64,
    },
    /// Availability fell below the defined threshold.
    #[error("availability SLO breach: observed {observed_pct:.3}% < threshold {threshold_pct:.3}%")]
    Availability {
        /// Measured availability percentage.
        observed_pct: f64,
        /// Defined threshold percentage.
        threshold_pct: f64,
    },
}

/// Unified error type for the Affidavit provenance layer.
///
/// Embraces combinatorial maximalism by exhaustively mapping every failure
/// state across the L2 pipeline (ocel -> chain -> verifier -> admission).
///
/// # Examples
///
/// ```rust
/// use affidavit::AffidavitError;
/// let err = AffidavitError::Validation("Missing sequence number".to_string());
/// assert!(err.to_string().contains("Validation error"));
/// ```
#[derive(Error, Debug)]
pub enum AffidavitError {
    /// Standard IO failures during receipt persistence or payload reading.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failures during JSON (de)serialization of receipts or OCEL logs.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Parsing failures for object references (id:type:qualifier).
    #[error("Parse error: {0}")]
    Parse(String),

    /// Well-formedness or structural validation failures (ocel layer).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Admission refused by the OCEL court or the certifier pipeline.
    #[error("Admission refused: {0}")]
    AdmissionRefused(String),

    /// Verification completed but yielded a REJECT verdict.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// General execution failures during CLI operations.
    #[error("Execution error: {0}")]
    Execution(String),

    /// Failures managing the working receipt (.affi/working.json).
    #[error("Working receipt error: {0}")]
    WorkingReceipt(String),

    /// Failures in BLAKE3 content-addressing or chain hashing.
    #[error("Content addressing error: {0}")]
    ContentAddressing(String),

    /// Failures in process discovery (wasm4pm) or quality metrics.
    #[error("Discovery error: {0}")]
    Discovery(String),

    /// Failures in LSP diagnostic mapping.
    #[error("LSP error: {0}")]
    Lsp(String),

    /// Wrapped OCEL building/validation errors.
    #[error(transparent)]
    Ocel(#[from] OcelError),

    /// Wrapped chain assembly/persistence errors.
    #[error(transparent)]
    Chain(#[from] ChainError),

    /// Wrapped post-quantum sealing errors.
    #[error(transparent)]
    Pqc(#[from] PqcError),

    /// Wrapped discovery/mining errors.
    #[error(transparent)]
    Mining(#[from] MiningError),

    /// Wrapped sharding/distributed verification errors.
    #[error(transparent)]
    Sharding(#[from] ShardingError),

    /// Wrapped activity prediction errors.
    #[error(transparent)]
    Prediction(#[from] PredictionError),

    /// Wrapped SLO violation errors.
    #[error(transparent)]
    Slo(#[from] SloViolation),
}

/// A Result type specialized for Affidavit operations.
///
/// # Examples
///
/// ```rust
/// use affidavit::error::Result;
/// fn operation() -> Result<()> {
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, AffidavitError>;
