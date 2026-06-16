//! OCEL event/object/relationship model + builders.
//!
//! Owned by the `ocel` phase-2 agent. Codes against `crate::types`.
//!
//! This module turns raw operation inputs into well-formed [`OperationEvent`]s.
//! An event commits to its payload bytes (BLAKE3) rather than carrying them, so
//! a verifier can check the commitment without ever seeing the payload. Logical
//! sequence numbers come from a caller-provided monotonic counter — never
//! wall-clock — preserving determinism.

use crate::types::{Blake3Hash, ObjectRef, OperationEvent};
use thiserror::Error;

/// A monotonic logical sequence counter for assigning event `seq` values.
///
/// Deterministic and time-free: each [`next_seq`](SeqCounter::next_seq) yields the
/// current value then advances by one, so a fixed sequence of construction
/// calls always produces the same ordering.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeqCounter {
    value: u64,
}

impl SeqCounter {
    /// Create a counter starting at sequence zero.
    pub fn new() -> Self {
        SeqCounter { value: 0 }
    }

    /// Create a counter starting at an explicit sequence value.
    pub fn starting_at(value: u64) -> Self {
        SeqCounter { value }
    }

    /// Return the current sequence value, then advance the counter by one.
    pub fn next_seq(&mut self) -> u64 {
        let current = self.value;
        self.value += 1;
        current
    }

    /// Peek at the value the next [`next_seq`](SeqCounter::next_seq) call would return.
    pub fn peek(&self) -> u64 {
        self.value
    }
}

/// Errors raised while building or validating OCEL events.
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

/// Build an unqualified object reference.
pub fn object_ref(id: impl Into<String>, obj_type: impl Into<String>) -> ObjectRef {
    ObjectRef {
        id: id.into(),
        obj_type: obj_type.into(),
        qualifier: None,
    }
}

/// Build a qualified object reference describing the object's role in the event.
pub fn qualified_object_ref(
    id: impl Into<String>,
    obj_type: impl Into<String>,
    qualifier: impl Into<String>,
) -> ObjectRef {
    ObjectRef {
        id: id.into(),
        obj_type: obj_type.into(),
        qualifier: Some(qualifier.into()),
    }
}

/// Parse a CLI-style `id:type` or `id:type:qualifier` string into an [`ObjectRef`].
///
/// The first colon separates id from type; an optional third colon-delimited
/// segment is treated as the qualifier. Empty id or type yields an error.
pub fn parse_object_ref(spec: &str) -> Result<ObjectRef, OcelError> {
    let mut parts = spec.splitn(3, ':');
    let id = parts.next().unwrap_or("");
    let obj_type = parts.next();
    let qualifier = parts.next();
    match obj_type {
        Some(ty) if !id.is_empty() && !ty.is_empty() => Ok(ObjectRef {
            id: id.to_string(),
            obj_type: ty.to_string(),
            qualifier: qualifier.filter(|q| !q.is_empty()).map(|q| q.to_string()),
        }),
        _ => Err(OcelError::MalformedObjectRef(spec.to_string())),
    }
}

/// Construct an [`OperationEvent`] from its parts, committing to payload bytes.
///
/// The `payload_commitment` is `BLAKE3(payload)` — the raw payload is never
/// stored. The `seq` is drawn from `counter`, advancing it by one. The `id` is
/// derived deterministically as `evt-{seq}` so a fixed call sequence is stable.
/// The resulting event is validated before it is returned.
///
/// # Example: see `examples/ocel_events.rs` (run: `cargo run --example ocel_events`)
pub fn build_event(
    event_type: impl Into<String>,
    objects: Vec<ObjectRef>,
    payload: &[u8],
    counter: &mut SeqCounter,
) -> Result<OperationEvent, OcelError> {
    let event_type = event_type.into();
    let seq = counter.next_seq();
    let event = OperationEvent {
        id: format!("evt-{seq}"),
        seq,
        event_type,
        objects,
        payload_commitment: Blake3Hash::from_bytes(payload),
    };
    validate_event(&event)?;
    Ok(event)
}

/// Validate that an [`OperationEvent`] is well-formed.
///
/// Checks: non-empty id, non-empty event_type, and that every object reference
/// carries a non-empty id and obj_type. Returns the first violation found.
pub fn validate_event(event: &OperationEvent) -> Result<(), OcelError> {
    if event.id.trim().is_empty() {
        return Err(OcelError::EmptyEventId);
    }
    if event.event_type.trim().is_empty() {
        return Err(OcelError::EmptyEventType);
    }
    for (i, obj) in event.objects.iter().enumerate() {
        if obj.id.trim().is_empty() {
            return Err(OcelError::EmptyObjectId(i));
        }
        if obj.obj_type.trim().is_empty() {
            return Err(OcelError::EmptyObjectType(i));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_event_yields_stable_commitment_for_fixed_bytes() {
        let mut c1 = SeqCounter::new();
        let mut c2 = SeqCounter::new();
        let e1 = build_event("write", vec![object_ref("o1", "file")], b"hello", &mut c1).unwrap();
        let e2 = build_event("write", vec![object_ref("o1", "file")], b"hello", &mut c2).unwrap();
        assert_eq!(e1.payload_commitment, e2.payload_commitment);
        assert_eq!(e1.payload_commitment, Blake3Hash::from_bytes(b"hello"));
        assert_ne!(e1.payload_commitment, Blake3Hash::from_bytes(b"world"));
    }

    #[test]
    fn seq_counter_is_monotonic_and_deterministic() {
        let mut c = SeqCounter::new();
        let a = build_event("a", vec![], b"", &mut c).unwrap();
        let b = build_event("b", vec![], b"", &mut c).unwrap();
        assert_eq!(a.seq, 0);
        assert_eq!(b.seq, 1);
        assert_eq!(a.id, "evt-0");
        assert_eq!(b.id, "evt-1");
        assert_eq!(c.peek(), 2);
    }

    #[test]
    fn validation_catches_empty_event_type() {
        let mut c = SeqCounter::new();
        let err = build_event("", vec![], b"x", &mut c).unwrap_err();
        assert_eq!(err, OcelError::EmptyEventType);

        let err2 = build_event("   ", vec![], b"x", &mut SeqCounter::new()).unwrap_err();
        assert_eq!(err2, OcelError::EmptyEventType);
    }

    #[test]
    fn validation_catches_empty_object_fields() {
        let mut c = SeqCounter::new();
        let err = build_event("op", vec![object_ref("", "file")], b"x", &mut c).unwrap_err();
        assert_eq!(err, OcelError::EmptyObjectId(0));

        let err2 = build_event(
            "op",
            vec![object_ref("o1", "")],
            b"x",
            &mut SeqCounter::new(),
        )
        .unwrap_err();
        assert_eq!(err2, OcelError::EmptyObjectType(0));
    }

    #[test]
    fn parse_object_ref_handles_qualifier_and_errors() {
        assert_eq!(
            parse_object_ref("o1:file").unwrap(),
            object_ref("o1", "file")
        );
        assert_eq!(
            parse_object_ref("o1:file:input").unwrap(),
            qualified_object_ref("o1", "file", "input")
        );
        assert_eq!(
            parse_object_ref("nope").unwrap_err(),
            OcelError::MalformedObjectRef("nope".to_string())
        );
        assert_eq!(
            parse_object_ref(":file").unwrap_err(),
            OcelError::MalformedObjectRef(":file".to_string())
        );
    }
}
