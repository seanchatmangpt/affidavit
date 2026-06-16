use thiserror::Error;

/// Unified error type for the Affidavit provenance layer.
/// 
/// Embraces combinatorial maximalism by exhaustively mapping every failure
/// state across the L2 pipeline (ocel -> chain -> verifier -> admission).
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
}

/// A Result type specialized for Affidavit operations.
pub type Result<T> = std::result::Result<T, AffidavitError>;
