//! Canonical RFC 3339 / ISO-8601 timestamps.
//!
//! Receipt envelopes carry time only as parseable, re-emittable canonical
//! strings — never as locale-formatted prose. Parsing applies offsets, and
//! the canonical form is a fixed UTC RFC 3339 rendering, so two hosts that
//! read the same wire string commit to the same instant.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! A timestamp parse is a decidable structural check. Monotonicity guards
//! (`checked_add`) refuse overflow instead of silently wrapping.

use thiserror::Error;

pub use iso8601_timestamp::Timestamp;

/// Errors produced by canonical timestamp operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CanonicalTimeError {
    /// The string is not a parseable ISO-8601 timestamp.
    #[error("malformed ISO-8601 timestamp: {0}")]
    Malformed(String),
    /// A monotonic arithmetic operation overflowed.
    #[error("timestamp arithmetic overflow")]
    Overflow,
}

/// Parse an ISO-8601 / RFC 3339 string into a canonical [`Timestamp`].
///
/// # Errors
///
/// Returns [`CanonicalTimeError::Malformed`] if the string cannot be parsed.
pub fn parse(s: &str) -> Result<Timestamp, CanonicalTimeError> {
    Timestamp::parse(s).ok_or_else(|| CanonicalTimeError::Malformed(s.to_string()))
}

/// Re-emit a timestamp in the canonical UTC RFC 3339 form.
///
/// Note: `iso8601-timestamp` renders with millisecond precision by default;
/// the canonical form of this module is exactly that rendering.
#[must_use]
pub fn canonical_string(ts: Timestamp) -> String {
    ts.format().to_string()
}

/// Parse and re-emit in one step: the canonicalization law.
///
/// # Errors
///
/// See [`parse`].
pub fn canonicalize(s: &str) -> Result<String, CanonicalTimeError> {
    parse(s).map(canonical_string)
}

/// Checked addition of a duration in milliseconds; refuses overflow.
///
/// # Errors
///
/// Returns [`CanonicalTimeError::Overflow`] if the addition overflows.
pub fn checked_add_ms(ts: Timestamp, millis: u64) -> Result<Timestamp, CanonicalTimeError> {
    ts.checked_add(iso8601_timestamp::Duration::milliseconds(
        i64::try_from(millis).map_err(|_| CanonicalTimeError::Overflow)?,
    ))
    .ok_or(CanonicalTimeError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalization_is_idempotent_for_same_instant() {
        let a = canonicalize("2026-10-03T01:02:03.000Z").expect("parse a");
        let b = canonicalize("2026-10-03T01:02:03.000+00:00").expect("parse b");
        assert_eq!(a, b, "same instant must render identically");
        assert_eq!(canonicalize(&a).expect("reparse"), a, "idempotent");
    }

    #[test]
    fn offsets_normalize_to_instant_equality() {
        // +02:00 offset two hours earlier on the wall clock: same instant.
        let utc = canonicalize("2026-10-03T12:00:00Z").expect("utc");
        let shifted = canonicalize("2026-10-03T14:00:00+02:00").expect("shifted");
        assert_eq!(parse(&utc).expect("ts"), parse(&shifted).expect("ts"));
    }

    #[test]
    fn malformed_input_is_typed_refusal() {
        assert_eq!(
            parse("not a timestamp"),
            Err(CanonicalTimeError::Malformed("not a timestamp".to_string()))
        );
    }

    #[test]
    fn checked_arithmetic_refuses_overflow() {
        let ts = parse("9999-12-31T23:59:59.999Z").expect("parse");
        assert_eq!(
            checked_add_ms(ts, u64::MAX),
            Err(CanonicalTimeError::Overflow)
        );
        let base = parse("2026-10-03T00:00:00.000Z").expect("parse");
        let advanced = checked_add_ms(base, 1_500).expect("add");
        assert_eq!(canonical_string(advanced), "2026-10-03T00:00:01.500Z");
    }
}
