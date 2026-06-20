//! Stable error codes, diagnostic types, and exit-code catalog.
//!
//! Every user-visible failure in affidavit is represented by an [`ErrorCode`]
//! with a stable `u16` numeric value that will never change between versions.
//! Consumers (scripts, CI pipelines, LSP clients) can key on those numbers.
//!
//! # Exit code catalog
//!
//! See [`exit_codes`] for the stable process-exit values used by the `affi` CLI.
//!
//! # Diagnostics
//!
//! A [`Diag`] is a structured error record: numeric code, human message,
//! optional hint, and optional source span.  It is serializable so that
//! `--format=json` handlers can emit machine-readable diagnostics.
//!
//! # Examples
//!
//! ```rust
//! use affidavit::diag::{Diag, ErrorCode};
//!
//! let d = Diag::new(ErrorCode::ChainHashMismatch, "rolling hash does not match stored value")
//!     .with_hint("re-run `affi assemble` to rebuild the receipt");
//!
//! assert_eq!(d.code, 1001);
//! assert!(d.hint.is_some());
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Exit-code catalog
// ---------------------------------------------------------------------------

/// Stable process-exit codes for the `affi` CLI.
///
/// These values are part of the public contract: scripts and CI pipelines may
/// key on them.  They must not change between crate versions.
pub mod exit_codes {
    /// All pipeline stages passed — ACCEPT verdict.
    pub const OK: i32 = 0;
    /// Receipt did not pass one or more verification stages — REJECT verdict.
    pub const REJECT: i32 = 2;
    /// Bad CLI arguments or usage error.
    pub const USAGE_ERROR: i32 = 3;
    /// I/O failure (file not found, unreadable, etc.).
    pub const IO_ERROR: i32 = 4;
    /// Unexpected internal error (bug in affidavit itself).
    pub const INTERNAL: i32 = 5;
    /// `verify_sla` threshold exceeded.
    pub const SLA_BREACH: i32 = 6;
}

// ---------------------------------------------------------------------------
// Error code catalog
// ---------------------------------------------------------------------------

/// Stable numeric codes for every user-visible failure mode.
///
/// The discriminant values are fixed and will never be reused even if a
/// variant is removed.  Consumers may store or transmit these numbers.
///
/// ## Range assignments
///
/// | Range      | Domain                        |
/// |-----------|-------------------------------|
/// | 1000–1099 | Chain / receipt integrity     |
/// | 1100–1199 | Format / structure errors     |
/// | 1200–1299 | Profile conformance errors    |
/// | 1300–1399 | I/O / environment errors      |
/// | 1400–1499 | Admission gate errors         |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[repr(u16)]
pub enum ErrorCode {
    // Chain / receipt errors: 1000–1099 ---------------------------------
    /// The recomputed rolling BLAKE3 chain hash does not match the stored value.
    ChainHashMismatch = 1001,
    /// The genesis (first-event) hash does not match expectations.
    GenesisHashMismatch = 1002,
    /// A gap was detected in the monotonic `seq` numbering.
    SeqGap = 1003,
    /// Two or more events carry the same `event_id`.
    DuplicateEventId = 1004,
    /// An event's `payload_commitment` is not a valid BLAKE3 hex digest.
    InvalidCommitment = 1005,
    /// The receipt was modified after sealing (structural tampering detected).
    TamperedReceipt = 1006,

    // Format errors: 1100–1199 ------------------------------------------
    /// The `format_version` field is absent or names an unknown schema.
    UnknownFormatVersion = 1100,
    /// The receipt JSON or binary payload cannot be decoded.
    MalformedReceipt = 1101,
    /// An event is missing the required `event_type` field.
    MissingEventType = 1102,

    // Profile errors: 1200–1299 -----------------------------------------
    /// The requested conformance profile is not registered.
    UnknownProfile = 1200,
    /// The receipt violates one or more rules of the active profile.
    ProfileViolation = 1201,

    // I/O / environment errors: 1300–1399 --------------------------------
    /// The receipt file path does not exist.
    ReceiptNotFound = 1300,
    /// The receipt file exists but could not be read (permissions, etc.).
    ReceiptUnreadable = 1301,
    /// The `.affi/` working directory is absent or not initialized.
    WorkingDirMissing = 1302,

    // Admission errors: 1400–1499 ----------------------------------------
    /// An object reference does not match the `id:type[:qualifier]` format.
    InvalidObjectId = 1400,
    /// The `event_type` string is empty or contains illegal characters.
    InvalidEventType = 1401,
}

impl ErrorCode {
    /// The stable numeric code for this error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::diag::ErrorCode;
    /// assert_eq!(ErrorCode::ChainHashMismatch.code(), 1001);
    /// ```
    #[inline]
    pub fn code(self) -> u16 {
        self as u16
    }

    /// The recommended process-exit code when this error terminates the CLI.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::diag::{ErrorCode, exit_codes};
    /// assert_eq!(ErrorCode::ReceiptNotFound.exit_code(), exit_codes::IO_ERROR);
    /// ```
    pub fn exit_code(self) -> i32 {
        match self {
            // Chain / receipt integrity → REJECT (the verifier rejected the receipt)
            ErrorCode::ChainHashMismatch
            | ErrorCode::GenesisHashMismatch
            | ErrorCode::SeqGap
            | ErrorCode::DuplicateEventId
            | ErrorCode::InvalidCommitment
            | ErrorCode::TamperedReceipt => exit_codes::REJECT,

            // Format errors → REJECT
            ErrorCode::UnknownFormatVersion
            | ErrorCode::MalformedReceipt
            | ErrorCode::MissingEventType => exit_codes::REJECT,

            // Profile errors → REJECT
            ErrorCode::UnknownProfile | ErrorCode::ProfileViolation => exit_codes::REJECT,

            // I/O errors → IO_ERROR
            ErrorCode::ReceiptNotFound
            | ErrorCode::ReceiptUnreadable
            | ErrorCode::WorkingDirMissing => exit_codes::IO_ERROR,

            // Admission errors → USAGE_ERROR (bad input to `emit`)
            ErrorCode::InvalidObjectId | ErrorCode::InvalidEventType => exit_codes::USAGE_ERROR,
        }
    }

    /// A terse, stable English description of what went wrong.
    ///
    /// The returned string is intentionally generic; a [`Diag`] carries the
    /// context-specific detail.
    pub fn message(self) -> &'static str {
        match self {
            ErrorCode::ChainHashMismatch => "chain hash mismatch",
            ErrorCode::GenesisHashMismatch => "genesis hash mismatch",
            ErrorCode::SeqGap => "sequence number gap detected",
            ErrorCode::DuplicateEventId => "duplicate event id",
            ErrorCode::InvalidCommitment => "invalid payload commitment",
            ErrorCode::TamperedReceipt => "receipt has been tampered with",
            ErrorCode::UnknownFormatVersion => "unknown format version",
            ErrorCode::MalformedReceipt => "malformed receipt",
            ErrorCode::MissingEventType => "missing event type",
            ErrorCode::UnknownProfile => "unknown conformance profile",
            ErrorCode::ProfileViolation => "profile violation",
            ErrorCode::ReceiptNotFound => "receipt file not found",
            ErrorCode::ReceiptUnreadable => "receipt file is unreadable",
            ErrorCode::WorkingDirMissing => "working directory missing or uninitialized",
            ErrorCode::InvalidObjectId => "invalid object id (expected id:type[:qualifier])",
            ErrorCode::InvalidEventType => "invalid event type",
        }
    }

    /// An optional remediation hint shown to the user.
    ///
    /// Returns `None` for errors that have no generic fix suggestion.
    pub fn hint(self) -> Option<&'static str> {
        match self {
            ErrorCode::ChainHashMismatch | ErrorCode::TamperedReceipt => Some(
                "re-run `affi assemble` from the original working directory to rebuild the receipt",
            ),
            ErrorCode::GenesisHashMismatch => Some(
                "the first event's commitment may have been modified; rebuild from source events",
            ),
            ErrorCode::SeqGap => Some(
                "events must be emitted in contiguous sequence; \
                 re-emit the missing events before assembling",
            ),
            ErrorCode::DuplicateEventId => Some(
                "each event must have a unique id; \
                 check for accidental double-emit in your pipeline",
            ),
            ErrorCode::InvalidCommitment => Some(
                "commitments must be 64-character lowercase hex BLAKE3 digests",
            ),
            ErrorCode::UnknownFormatVersion => {
                Some("this version of affidavit supports `format_version = \"core/v1\"` only")
            }
            ErrorCode::MalformedReceipt => {
                Some("verify the file is valid JSON and was produced by `affi assemble`")
            }
            ErrorCode::MissingEventType => {
                Some("every event must include a non-empty `event_type` field")
            }
            ErrorCode::UnknownProfile => {
                Some("run `affi model --export-types` to list supported profiles")
            }
            ErrorCode::ProfileViolation => {
                Some("run `affi diagnose <receipt>` for stage-level detail")
            }
            ErrorCode::ReceiptNotFound => Some("check the path and that the file exists"),
            ErrorCode::ReceiptUnreadable => {
                Some("check file permissions; the receipt must be readable by the current user")
            }
            ErrorCode::WorkingDirMissing => {
                Some("run `affi emit` at least once to initialise the `.affi/` directory")
            }
            ErrorCode::InvalidObjectId => Some(
                "object ids must follow the pattern `id:type` or `id:type:qualifier` \
                 (e.g. `repo:git`, `suite:test:unit`)",
            ),
            ErrorCode::InvalidEventType => Some(
                "event types must be non-empty lowercase strings \
                 with optional hyphens (e.g. `build`, `audit-log`)",
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Span — source location for a diagnostic
// ---------------------------------------------------------------------------

/// A reference to a position in a source file, attached to a [`Diag`].
///
/// When the LSP integration surfaces a diagnostic, `file` is the receipt path
/// and `line` (0-based) is the JSON line where the problem was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Path to the file where the diagnostic originates (receipt path, source
    /// file, etc.).
    pub file: String,
    /// Optional 0-based line number within the file.
    pub line: Option<u32>,
}

// ---------------------------------------------------------------------------
// Diag — a structured diagnostic
// ---------------------------------------------------------------------------

/// A structured, serializable diagnostic record.
///
/// A `Diag` is produced whenever affidavit encounters a user-visible failure.
/// It carries a stable numeric code, a human-readable message, an optional
/// remediation hint, and an optional source location ([`Span`]).
///
/// # Examples
///
/// ```rust
/// use affidavit::diag::{Diag, ErrorCode};
///
/// let d = Diag::new(ErrorCode::SeqGap, "gap between seq 2 and seq 4")
///     .with_hint("re-emit the missing event with seq=3");
///
/// assert_eq!(d.code, 1003);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diag {
    /// Stable numeric error code (see [`ErrorCode`]).
    pub code: u16,
    /// Human-readable description of this specific failure.
    pub message: String,
    /// Optional suggestion for how to remediate the error.
    pub hint: Option<String>,
    /// Optional source location (file path + line number).
    pub span: Option<Span>,
}

impl Diag {
    /// Construct a new `Diag` with the given code and message.
    ///
    /// The hint and span fields default to `None`; use the builder methods
    /// [`with_hint`](Self::with_hint) and [`with_span`](Self::with_span) to
    /// populate them.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::diag::{Diag, ErrorCode};
    /// let d = Diag::new(ErrorCode::MalformedReceipt, "unexpected EOF at byte 42");
    /// assert_eq!(d.code, 1101);
    /// ```
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Diag {
            code: code.code(),
            message: message.into(),
            hint: None,
            span: None,
        }
    }

    /// Attach a remediation hint to this diagnostic.
    ///
    /// If the caller does not supply a hint, [`ErrorCode::hint`] provides a
    /// generic fallback that can be injected at display time.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Attach a source location to this diagnostic.
    ///
    /// `file` is typically the receipt path; `line` is 0-based.
    pub fn with_span(mut self, file: impl Into<String>, line: Option<u32>) -> Self {
        self.span = Some(Span {
            file: file.into(),
            line,
        });
        self
    }

    /// Build a `Diag` from an [`ErrorCode`] and a `std::error::Error`.
    ///
    /// The error's `Display` output becomes the message.  The code's generic
    /// [`hint`](ErrorCode::hint) is attached when available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use affidavit::diag::{Diag, ErrorCode};
    /// use std::io;
    ///
    /// let err = io::Error::new(io::ErrorKind::NotFound, "no such file");
    /// let d = Diag::from_error(ErrorCode::ReceiptNotFound, &err);
    /// assert_eq!(d.code, 1300);
    /// ```
    pub fn from_error(code: ErrorCode, err: &dyn std::error::Error) -> Self {
        let mut d = Diag::new(code, err.to_string());
        if let Some(h) = code.hint() {
            d.hint = Some(h.to_string());
        }
        d
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_discriminants_are_stable() {
        assert_eq!(ErrorCode::ChainHashMismatch.code(), 1001);
        assert_eq!(ErrorCode::GenesisHashMismatch.code(), 1002);
        assert_eq!(ErrorCode::SeqGap.code(), 1003);
        assert_eq!(ErrorCode::DuplicateEventId.code(), 1004);
        assert_eq!(ErrorCode::InvalidCommitment.code(), 1005);
        assert_eq!(ErrorCode::TamperedReceipt.code(), 1006);
        assert_eq!(ErrorCode::UnknownFormatVersion.code(), 1100);
        assert_eq!(ErrorCode::MalformedReceipt.code(), 1101);
        assert_eq!(ErrorCode::MissingEventType.code(), 1102);
        assert_eq!(ErrorCode::UnknownProfile.code(), 1200);
        assert_eq!(ErrorCode::ProfileViolation.code(), 1201);
        assert_eq!(ErrorCode::ReceiptNotFound.code(), 1300);
        assert_eq!(ErrorCode::ReceiptUnreadable.code(), 1301);
        assert_eq!(ErrorCode::WorkingDirMissing.code(), 1302);
        assert_eq!(ErrorCode::InvalidObjectId.code(), 1400);
        assert_eq!(ErrorCode::InvalidEventType.code(), 1401);
    }

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(ErrorCode::ChainHashMismatch.exit_code(), exit_codes::REJECT);
        assert_eq!(ErrorCode::TamperedReceipt.exit_code(), exit_codes::REJECT);
        assert_eq!(ErrorCode::UnknownFormatVersion.exit_code(), exit_codes::REJECT);
        assert_eq!(ErrorCode::ProfileViolation.exit_code(), exit_codes::REJECT);
        assert_eq!(ErrorCode::ReceiptNotFound.exit_code(), exit_codes::IO_ERROR);
        assert_eq!(
            ErrorCode::WorkingDirMissing.exit_code(),
            exit_codes::IO_ERROR
        );
        assert_eq!(
            ErrorCode::InvalidObjectId.exit_code(),
            exit_codes::USAGE_ERROR
        );
        assert_eq!(
            ErrorCode::InvalidEventType.exit_code(),
            exit_codes::USAGE_ERROR
        );
    }

    #[test]
    fn diag_new_sets_code_and_message() {
        let d = Diag::new(ErrorCode::SeqGap, "gap at seq 5");
        assert_eq!(d.code, 1003);
        assert_eq!(d.message, "gap at seq 5");
        assert!(d.hint.is_none());
        assert!(d.span.is_none());
    }

    #[test]
    fn diag_with_hint_chains() {
        let d = Diag::new(ErrorCode::SeqGap, "gap").with_hint("fix it");
        assert_eq!(d.hint.as_deref(), Some("fix it"));
    }

    #[test]
    fn diag_with_span_chains() {
        let d = Diag::new(ErrorCode::MalformedReceipt, "bad json")
            .with_span("receipt.json", Some(7));
        let span = d.span.as_ref().expect("span should be set");
        assert_eq!(span.file, "receipt.json");
        assert_eq!(span.line, Some(7));
    }

    #[test]
    fn diag_from_error_attaches_generic_hint() {
        use std::io;
        let err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let d = Diag::from_error(ErrorCode::ReceiptNotFound, &err);
        assert_eq!(d.code, 1300);
        assert!(d.hint.is_some(), "generic hint should be attached");
    }

    #[test]
    fn diag_is_serializable() {
        let d = Diag::new(ErrorCode::InvalidObjectId, "bad id 'foo'")
            .with_hint("use id:type format")
            .with_span("receipt.json", Some(3));
        let json = serde_json::to_string(&d).expect("serialize");
        assert!(json.contains("1400"));
        assert!(json.contains("bad id"));
    }

    #[test]
    fn exit_code_catalog_values() {
        assert_eq!(exit_codes::OK, 0);
        assert_eq!(exit_codes::REJECT, 2);
        assert_eq!(exit_codes::USAGE_ERROR, 3);
        assert_eq!(exit_codes::IO_ERROR, 4);
        assert_eq!(exit_codes::INTERNAL, 5);
        assert_eq!(exit_codes::SLA_BREACH, 6);
    }

    #[test]
    fn all_error_codes_have_message() {
        let codes = [
            ErrorCode::ChainHashMismatch,
            ErrorCode::GenesisHashMismatch,
            ErrorCode::SeqGap,
            ErrorCode::DuplicateEventId,
            ErrorCode::InvalidCommitment,
            ErrorCode::TamperedReceipt,
            ErrorCode::UnknownFormatVersion,
            ErrorCode::MalformedReceipt,
            ErrorCode::MissingEventType,
            ErrorCode::UnknownProfile,
            ErrorCode::ProfileViolation,
            ErrorCode::ReceiptNotFound,
            ErrorCode::ReceiptUnreadable,
            ErrorCode::WorkingDirMissing,
            ErrorCode::InvalidObjectId,
            ErrorCode::InvalidEventType,
        ];
        for code in codes {
            let msg = code.message();
            assert!(!msg.is_empty(), "E{} has empty message", code.code());
        }
    }
}
