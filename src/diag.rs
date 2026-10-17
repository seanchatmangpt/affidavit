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
/// | 1500–1599 | CLI framework surface errors  |
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

    // CLI framework surface errors: 1500–1599 -----------------------------
    /// `clap-noun-verb` could not resolve the requested noun.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::CommandNotFound`].
    CliCommandNotFound = 1500,
    /// `clap-noun-verb` resolved the noun but not the verb.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::VerbNotFound`].
    CliVerbNotFound = 1501,
    /// A CLI argument failed parsing or validation in the framework.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::ArgumentError`].
    CliArgumentError = 1502,
    /// The noun-verb argument structure itself is invalid.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::InvalidStructure`].
    CliInvalidStructure = 1503,
    /// A resolved verb's handler returned an error.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::ExecutionError`].
    CliExecutionError = 1504,
    /// A CLI plugin failed to load or dispatch.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::PluginError`].
    CliPluginError = 1505,
    /// Framework-level validation failed.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::ValidationFailed`].
    CliValidationFailed = 1506,
    /// A CLI middleware step failed.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::MiddlewareError`].
    CliMiddlewareError = 1507,
    /// CLI telemetry emission failed.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::TelemetryError`].
    CliTelemetryError = 1508,
    /// Framework error with no more specific classification.
    ///
    /// Maps [`clap_noun_verb::NounVerbError::Generic`].
    CliGenericError = 1509,
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

            // CLI framework surface errors → exit with the stable diagnostic
            // code encoded modulo 256 (POSIX exit status is 8-bit). The full
            // stable value (1500–1599) is carried in the diagnostic's `code`
            // field; the process exit status distinguishes each variant
            // (1500 → 222 … 1509 → 231).
            ErrorCode::CliCommandNotFound
            | ErrorCode::CliVerbNotFound
            | ErrorCode::CliArgumentError
            | ErrorCode::CliInvalidStructure
            | ErrorCode::CliExecutionError
            | ErrorCode::CliPluginError
            | ErrorCode::CliValidationFailed
            | ErrorCode::CliMiddlewareError
            | ErrorCode::CliTelemetryError
            | ErrorCode::CliGenericError => (self.code() as i32) % 256,
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
            ErrorCode::CliCommandNotFound => "command (noun) not found",
            ErrorCode::CliVerbNotFound => "verb not found for noun",
            ErrorCode::CliArgumentError => "CLI argument parsing failed",
            ErrorCode::CliInvalidStructure => "invalid command structure",
            ErrorCode::CliExecutionError => "command execution failed",
            ErrorCode::CliPluginError => "CLI plugin error",
            ErrorCode::CliValidationFailed => "CLI validation failed",
            ErrorCode::CliMiddlewareError => "CLI middleware error",
            ErrorCode::CliTelemetryError => "CLI telemetry error",
            ErrorCode::CliGenericError => "CLI framework error",
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
            ErrorCode::InvalidCommitment => {
                Some("commitments must be 64-character lowercase hex BLAKE3 digests")
            }
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
            ErrorCode::CliCommandNotFound => Some("run `affi --help` to list the known nouns"),
            ErrorCode::CliVerbNotFound => {
                Some("run `affi <noun> --help` to list the known verbs for that noun")
            }
            ErrorCode::CliArgumentError => {
                Some("check the arguments with `affi <noun> <verb> --help`")
            }
            ErrorCode::CliInvalidStructure => {
                Some("commands follow the `affi <noun> <verb>` pattern")
            }
            ErrorCode::CliExecutionError => None,
            ErrorCode::CliPluginError => None,
            ErrorCode::CliValidationFailed => None,
            ErrorCode::CliMiddlewareError => None,
            ErrorCode::CliTelemetryError => None,
            ErrorCode::CliGenericError => None,
        }
    }
}

/// Strip ANSI escape sequences from a framework error string.
///
/// `clap-noun-verb` embeds bold-yellow highlighting (`\x1b[1m\x1b[33m…\x1b[0m`)
/// around its Levenshtein suggestions; a structured diagnostic must not carry
/// terminal control bytes into JSON output.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip `ESC [ … <final byte in 0x40..=0x7E>`
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&n) = chars.peek() {
                    chars.next();
                    if n as u32 >= 0x40 && n as u32 <= 0x7E {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Extract the framework's "Did you mean: …?" Levenshtein suggestion from a
/// `NounVerbError` display string, if present.
///
/// Returns the suggestion text (e.g. `"receipt"`) with any ANSI highlighting
/// removed, suitable for the [`Diag`] `hint` field.
fn extract_suggestion(display: &str) -> Option<String> {
    let clean = strip_ansi(display);
    // clap-noun-verb's own Levenshtein path: "Did you mean: <a>, <b>?"
    if let Some(idx) = clean.find("Did you mean: ") {
        let rest = &clean[idx + "Did you mean: ".len()..];
        let end = rest.find('?')?;
        let suggestion = rest[..end].trim();
        if !suggestion.is_empty() {
            return Some(suggestion.to_string());
        }
    }
    // clap v4's parse-layer form:
    //   tip: a similar subcommand exists: 'receipt'
    //   tip: some similar subcommands exist: 'a', 'b'
    for marker in [
        "a similar subcommand exists: ",
        "some similar subcommands exist: ",
    ] {
        if let Some(idx) = clean.find(marker) {
            let rest = &clean[idx + marker.len()..].replace('\'', "");
            let line_end = rest.find('\n').unwrap_or(rest.len());
            let first = rest[..line_end].split(',').next().unwrap_or("").trim();
            if !first.is_empty() {
                return Some(first.to_string());
            }
        }
    }
    None
}

/// Look for the offending token in a clap "unrecognized subcommand" error and
/// classify noun-miss vs verb-miss against the compile-time verb registry.
///
/// Returns the quoted token, if present.
fn unrecognized_subcommand_token(display: &str) -> Option<String> {
    let clean = strip_ansi(display);
    let idx = clean.find("unrecognized subcommand '")?;
    let rest = &clean[idx + "unrecognized subcommand '".len()..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

/// True when `token` names a noun known to the compile-time registry
/// (hyphen/underscore normalization applied).
fn is_known_noun(token: &str) -> bool {
    let normalized = token.replace('-', "_");
    crate::registry::REGISTRY
        .iter()
        .any(|e| e.noun == token || e.noun.replace('-', "_") == normalized)
}

/// True when the clap usage line names a known noun
/// (`Usage: affi receipt [OPTIONS] ...`) — i.e. the failure happened *under*
/// a resolved noun, so an unrecognized subcommand there is a verb miss.
fn usage_line_names_noun(display: &str) -> bool {
    let clean = strip_ansi(display);
    for line in clean.lines() {
        if let Some(rest) = line.trim().strip_prefix("Usage: affi ") {
            if let Some(first) = rest.split_whitespace().next() {
                if is_known_noun(first) {
                    return true;
                }
            }
        }
    }
    false
}

impl Diag {
    /// Build a [`Diag`] from a `clap-noun-verb` framework error.
    ///
    /// Each [`NounVerbError`] variant maps to a stable code in the 1500–1599
    /// CLI surface range (see [`ErrorCode`]). The framework's Levenshtein
    /// "Did you mean" suggestion — when present — becomes the diagnostic's
    /// `hint`, replacing the plain error string that a bare
    /// `clap_noun_verb::Result` exit would print.
    ///
    /// The recommended process exit code for every variant in this range is
    /// the diagnostic code encoded modulo 256 (POSIX exit status is 8-bit:
    /// 1500 → 222 … 1509 → 231, still distinct per variant); the full
    /// stable value rides in the diagnostic's `code` field.
    ///
    /// # Examples
    ///
    /// ```
    /// use affidavit::diag::{Diag, ErrorCode};
    /// use clap_noun_verb::NounVerbError;
    ///
    /// let err = NounVerbError::command_not_found("bogus-noun");
    /// let d = Diag::from_noun_verb_error(&err);
    /// assert_eq!(d.code, 1500);
    /// assert_eq!(ErrorCode::CliCommandNotFound.exit_code(), 1500 % 256);
    /// ```
    pub fn from_noun_verb_error(err: &clap_noun_verb::NounVerbError) -> Self {
        use clap_noun_verb::NounVerbError as E;
        let display = err.to_string();
        let code = match err {
            E::CommandNotFound { .. } => ErrorCode::CliCommandNotFound,
            E::VerbNotFound { .. } => ErrorCode::CliVerbNotFound,
            // clap's parse layer rejects unknown nouns/verbs as
            // ArgumentError("unrecognized subcommand ...") before the
            // framework's CommandNotFound/VerbNotFound path can fire;
            // reclassify against the registry so a miss against a known
            // noun is a verb error and anything else is a noun error.
            E::ArgumentError { .. } => {
                if unrecognized_subcommand_token(&display).is_some() {
                    if usage_line_names_noun(&display) {
                        ErrorCode::CliVerbNotFound
                    } else {
                        ErrorCode::CliCommandNotFound
                    }
                } else {
                    ErrorCode::CliArgumentError
                }
            }
            E::InvalidStructure { .. } => ErrorCode::CliInvalidStructure,
            E::ExecutionError { .. } => ErrorCode::CliExecutionError,
            E::PluginError(_) => ErrorCode::CliPluginError,
            E::ValidationFailed(_) => ErrorCode::CliValidationFailed,
            E::MiddlewareError(_) => ErrorCode::CliMiddlewareError,
            E::TelemetryError(_) => ErrorCode::CliTelemetryError,
            E::Generic(_) => ErrorCode::CliGenericError,
        };
        let mut d = Diag::new(code, display);
        // The framework's Levenshtein suggestion outranks the generic hint.
        if let Some(s) = extract_suggestion(&d.message) {
            d.hint = Some(s);
        } else if let Some(h) = code.hint() {
            d.hint = Some(h.to_string());
        }
        d
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
        assert_eq!(ErrorCode::CliCommandNotFound.code(), 1500);
        assert_eq!(ErrorCode::CliVerbNotFound.code(), 1501);
        assert_eq!(ErrorCode::CliArgumentError.code(), 1502);
        assert_eq!(ErrorCode::CliInvalidStructure.code(), 1503);
        assert_eq!(ErrorCode::CliExecutionError.code(), 1504);
        assert_eq!(ErrorCode::CliPluginError.code(), 1505);
        assert_eq!(ErrorCode::CliValidationFailed.code(), 1506);
        assert_eq!(ErrorCode::CliMiddlewareError.code(), 1507);
        assert_eq!(ErrorCode::CliTelemetryError.code(), 1508);
        assert_eq!(ErrorCode::CliGenericError.code(), 1509);
    }

    #[test]
    fn cli_surface_codes_exit_with_their_own_number() {
        let codes = [
            ErrorCode::CliCommandNotFound,
            ErrorCode::CliVerbNotFound,
            ErrorCode::CliArgumentError,
            ErrorCode::CliInvalidStructure,
            ErrorCode::CliExecutionError,
            ErrorCode::CliPluginError,
            ErrorCode::CliValidationFailed,
            ErrorCode::CliMiddlewareError,
            ErrorCode::CliTelemetryError,
            ErrorCode::CliGenericError,
        ];
        for code in codes {
            let expected = (code.code() as i32) % 256;
            assert_eq!(
                code.exit_code(),
                expected,
                "E{} must exit with its code mod 256",
                code.code()
            );
            assert!((0..=255).contains(&code.exit_code()));
        }
    }

    #[test]
    fn from_noun_verb_error_maps_command_not_found_with_suggestion() {
        let err = clap_noun_verb::NounVerbError::command_not_found_with_candidates(
            "reciept",
            &["receipt", "emit", "verify"],
        );
        let d = Diag::from_noun_verb_error(&err);
        assert_eq!(d.code, 1500);
        assert_eq!(
            d.hint.as_deref(),
            Some("receipt"),
            "hint is the Levenshtein suggestion"
        );
        assert!(
            !d.hint.as_deref().unwrap_or_default().contains('\u{1b}'),
            "no ANSI bytes in hint"
        );
    }

    #[test]
    fn from_noun_verb_error_falls_back_to_generic_hint() {
        let err = clap_noun_verb::NounVerbError::command_not_found("zzzzzz");
        let d = Diag::from_noun_verb_error(&err);
        assert_eq!(d.code, 1500);
        assert_eq!(
            d.hint.as_deref(),
            Some("run `affi --help` to list the known nouns")
        );
    }

    #[test]
    fn from_noun_verb_error_maps_verb_not_found() {
        let err = clap_noun_verb::NounVerbError::verb_not_found_with_candidates(
            "receipt",
            "verrfy",
            &["verify", "validate"],
        );
        let d = Diag::from_noun_verb_error(&err);
        assert_eq!(d.code, 1501);
        assert_eq!(d.hint.as_deref(), Some("verify"));
    }

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(ErrorCode::ChainHashMismatch.exit_code(), exit_codes::REJECT);
        assert_eq!(ErrorCode::TamperedReceipt.exit_code(), exit_codes::REJECT);
        assert_eq!(
            ErrorCode::UnknownFormatVersion.exit_code(),
            exit_codes::REJECT
        );
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
        // POSIX exit status is 8-bit: 1500 % 256 == 220.
        assert_eq!(ErrorCode::CliCommandNotFound.exit_code(), 220);
        assert_eq!(ErrorCode::CliVerbNotFound.exit_code(), 221);
        assert_eq!(ErrorCode::CliArgumentError.exit_code(), 222);
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
        let d =
            Diag::new(ErrorCode::MalformedReceipt, "bad json").with_span("receipt.json", Some(7));
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
            ErrorCode::CliCommandNotFound,
            ErrorCode::CliVerbNotFound,
            ErrorCode::CliArgumentError,
            ErrorCode::CliInvalidStructure,
            ErrorCode::CliExecutionError,
            ErrorCode::CliPluginError,
            ErrorCode::CliValidationFailed,
            ErrorCode::CliMiddlewareError,
            ErrorCode::CliTelemetryError,
            ErrorCode::CliGenericError,
        ];
        for code in codes {
            let msg = code.message();
            assert!(!msg.is_empty(), "E{} has empty message", code.code());
        }
    }
}
