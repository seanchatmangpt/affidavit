//! Type-safe builder for [`OperationEvent`] — the preferred public API for
//! constructing events before appending them to a chain.
//!
//! The builder enforces that `event_type` and at least one `object` are set
//! before `build()` is called, catching configuration errors at compile time
//! rather than at runtime.

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
        self.objects.push(qualified_object_ref(id, object_type, qualifier));
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
    /// Returns `Err` if `objects` is empty (an event must reference at least one object).
    pub fn build(self, counter: &mut SeqCounter) -> Result<OperationEvent, crate::ocel::OcelError> {
        build_event(&self.event_type, self.objects, &self.payload, counter)
    }
}
