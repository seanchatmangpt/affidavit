//! Type-safe builder for [`OperationEvent`] — the preferred public API for
//! constructing events before appending them to a chain.
//!
//! `build()` delegates to [`build_event`], so it applies the same admission
//! checks (non-empty `event_type`, well-formed object refs). Note that an
//! event with zero objects is *constructible*; it is refused later by the
//! OCEL court in `verify`, not here.

use crate::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
use crate::types::{ObjectRef, OperationEvent};

/// Builder for [`OperationEvent`]. Use [`EventBuilder::new`] to start.
///
/// # Examples
///
/// ```
/// use affidavit::event_builder::EventBuilder;
/// use affidavit::ocel::SeqCounter;
///
/// let mut counter = SeqCounter::new();
/// let event = EventBuilder::new("create")
///     .object("file.txt", "artifact")
///     .payload(b"hello world")
///     .build(&mut counter)
///     .expect("build event");
/// assert_eq!(event.event_type, "create");
/// ```
pub struct EventBuilder {
    event_type: String,
    objects: Vec<ObjectRef>,
    payload: Vec<u8>,
}

impl EventBuilder {
    /// Create a new builder with the given event type.
    #[must_use]
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            objects: Vec::new(),
            payload: Vec::new(),
        }
    }

    /// Add an object reference to this event.
    #[must_use]
    pub fn object(mut self, id: impl Into<String>, object_type: impl Into<String>) -> Self {
        self.objects.push(object_ref(id, object_type));
        self
    }

    /// Add a qualified object reference (with a qualifier) to this event.
    #[must_use]
    pub fn qualified_object(
        mut self,
        id: impl Into<String>,
        object_type: impl Into<String>,
        qualifier: impl Into<String>,
    ) -> Self {
        self.objects
            .push(qualified_object_ref(id, object_type, qualifier));
        self
    }

    /// Set the raw payload bytes for this event.
    #[must_use]
    pub fn payload(mut self, payload: impl Into<Vec<u8>>) -> Self {
        self.payload = payload.into();
        self
    }

    /// Set the payload from a string.
    #[must_use]
    pub fn payload_str(mut self, payload: impl Into<String>) -> Self {
        self.payload = payload.into().into_bytes();
        self
    }

    /// Build the event, consuming the builder and advancing the sequence counter.
    ///
    /// Returns `Err` when [`build_event`] rejects the event (e.g. an empty
    /// `event_type`, or an object reference with an empty id or type).
    pub fn build(self, counter: &mut SeqCounter) -> Result<OperationEvent, crate::ocel::OcelError> {
        build_event(&self.event_type, self.objects, &self.payload, counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_empty_event_type() {
        let mut counter = SeqCounter::new();
        let err = EventBuilder::new("")
            .object("f", "artifact")
            .build(&mut counter)
            .expect_err("empty event_type must be refused");
        assert_eq!(err, crate::ocel::OcelError::EmptyEventType);
    }

    #[test]
    fn builder_rejects_empty_object_id() {
        let mut counter = SeqCounter::new();
        let err = EventBuilder::new("create")
            .object("", "artifact")
            .build(&mut counter)
            .expect_err("empty object id must be refused");
        assert_eq!(err, crate::ocel::OcelError::EmptyObjectId(0));
    }

    #[test]
    fn builder_matches_build_event_commitment() {
        let mut a = SeqCounter::new();
        let mut b = SeqCounter::new();
        let via_builder = EventBuilder::new("create")
            .object("f", "artifact")
            .payload_str("data")
            .build(&mut a)
            .expect("builder");
        let direct = build_event("create", vec![object_ref("f", "artifact")], b"data", &mut b)
            .expect("direct");
        assert_eq!(via_builder, direct);
    }
}
