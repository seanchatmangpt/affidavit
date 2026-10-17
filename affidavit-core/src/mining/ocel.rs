//! Object-centric event logs (OCEL) — discovery and conformance per object.
//!
//! Where a [`Trace`] flattens a log into one activity
//! sequence, an OCEL keeps the *objects*: each event references zero or more
//! objects (`{"order-1", "item-7"}`), and the same activity may touch many
//! objects at once (van der Aalst's object-centric process mining). This module
//! projects an [`OcelLog`] onto one classic trace *per object* (its events
//! sorted by the monotonic `timestamp`), then reuses the existing
//! [`DirectlyFollowsGraph`] and token-replay machinery per object *type*:
//!
//! - [`object_centric_dfg`] — one directly-follows model per object type.
//! - [`replay_per_object`] — per-object conformance against those models.
//!
//! All aggregation is `BTreeMap`-backed and every projection sorts by
//! `(timestamp, activity, object id)`, so discovery and conformance are fully
//! deterministic: same log in, byte-identical model out. Object references are
//! borrowed (no owned copies), matching the crate's zero-allocation-beyond-`alloc`
//! discipline. Needs `alloc`, so it lives behind the `alloc` feature like the
//! rest of [`mining`](crate::mining).
//!
//! # The published OCEL v2 type layer (`ocel-compat` feature)
//!
//! The canonical OCEL 2.0 vocabulary ships in the published
//! [`wasm4pm-compat`](https://crates.io/crates/wasm4pm-compat) crate (pinned
//! `=26.8.7` by the root `affidavit` crate). Finding, recorded honestly: that
//! crate is **nightly-only** (unconditional `#![feature(generic_const_exprs)]`,
//! `#![feature(portable_simd)]`, ...) and **std-based** (`String`,
//! `std::collections::{HashMap,HashSet}` throughout), so it is **not
//! `no_std`-compatible**. This crate is `no_std`-first with `forbid(unsafe)`,
//! so the compat types cannot be the default log/event/object layer here.
//! The rebase therefore takes the adapter form: the default build keeps the
//! in-crate borrowed `no_std` layer above (byte-identical behavior, empty-log
//! and no-reference typed refusals included), and behind the `ocel-compat`
//! feature the published types become the *admissible source*: an
//! [`OcelLog`] is constructed from a `wasm4pm_compat::ocel::OcelLog` by
//! `borrow_compat` (behind the `ocel-compat` feature), which also enforces the compat crate's own
//! structural laws. The version pin lives in `Cargo.toml` (`=26.8.7`, exact)
//! — the same pin the root crate carries.
//!
//! Doctrine, preserved: this *certifies* object-centric logs against discovered
//! models; it never decides whether the recorded work was honest.
//!
//! # Example
//!
//! ```
//! use affidavit_core::mining::ocel::{OcelEvent, OcelLog, OcelObject, object_centric_dfg};
//!
//! let log = OcelLog::new(vec![
//!     OcelEvent {
//!         activity: "create",
//!         timestamp: 1,
//!         objects: vec![OcelObject { id: "o-1", object_type: "order" }],
//!     },
//!     OcelEvent {
//!         activity: "pay",
//!         timestamp: 2,
//!         objects: vec![OcelObject { id: "o-1", object_type: "order" }],
//!     },
//! ])
//! .unwrap();
//!
//! let models = object_centric_dfg(&log).unwrap();
//! let order = &models["order"];
//! assert!(order.follows("create", "pay"));
//! ```

use crate::mining::{ConformanceResult, ConformanceVerdict, DirectlyFollowsGraph, Trace};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// An object referenced by an OCEL event, borrowed from the log's data.
///
/// `id` identifies the individual object instance (`"order-42"`); `object_type`
/// names its class (`"order"`), which is the grouping key for discovery.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct OcelObject<'a> {
    /// Instance identifier, unique within its type.
    pub id: &'a str,
    /// Class of the object (`"order"`, `"item"`, ...).
    pub object_type: &'a str,
}

/// One object-centric event: an activity at a monotonic timestamp touching
/// zero or more objects.
///
/// `timestamp` is monotonic (no wall-clock), matching the crate's
/// determinism doctrine; ordering within a projection breaks ties on
/// `(activity, object id)`, never on time alone.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct OcelEvent<'a> {
    /// The activity performed (`"create"`, `"pay"`, ...).
    pub activity: &'a str,
    /// Monotonic sequence point (not a wall-clock time).
    pub timestamp: u64,
    /// Objects this event touches. Empty is legal: the event then belongs to
    /// no object trace.
    pub objects: Vec<OcelObject<'a>>,
}

/// Typed refusal — constructing or mining a log that cannot yield a model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OcelError {
    /// A log with zero events has no behavior to discover or certify.
    EmptyLog,
    /// Every event in the log references zero objects, so no object trace
    /// (and hence no object-centric model) exists.
    NoObjectReferences,
}

/// An object-centric event log: the full borrowed event slice.
///
/// Constructed only through [`OcelLog::new`], which refuses (typed, no panic)
/// an empty event list. Stored in insertion order; every *projection* sorts
/// deterministically.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OcelLog<'a> {
    events: Vec<OcelEvent<'a>>,
}

impl<'a> OcelLog<'a> {
    /// Construct a log, refusing an empty event list with [`OcelError::EmptyLog`].
    pub fn new(events: Vec<OcelEvent<'a>>) -> Result<Self, OcelError> {
        if events.is_empty() {
            return Err(OcelError::EmptyLog);
        }
        Ok(OcelLog { events })
    }

    /// The events of the log, in insertion order.
    pub fn events(&self) -> &[OcelEvent<'a>] {
        &self.events
    }

    /// Number of events in the log.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True if the log has no events (unconstructable via [`OcelLog::new`];
    /// possible only through `Default`-less direct struct abuse, kept for API
    /// symmetry with [`Trace`]).
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Deterministically project the log onto one trace per object instance.
    ///
    /// For each object (keyed by `(object_type, id)`), collect the activities
    /// of all events touching it, sorted by `(timestamp, activity)` — stable
    /// and total, so the same log always yields the same traces. Returns them
    /// keyed by object type then id (both `BTreeMap`, both deterministic).
    pub fn traces_per_object(&self) -> BTreeMap<(&'a str, &'a str), Trace<'a>> {
        let mut by_object: BTreeMap<(&'a str, &'a str), Vec<OcelEvent<'a>>> = BTreeMap::new();
        for event in &self.events {
            for obj in &event.objects {
                by_object
                    .entry((obj.object_type, obj.id))
                    .or_default()
                    .push(event.clone());
            }
        }
        let mut traces = BTreeMap::new();
        for ((object_type, id), mut events) in by_object {
            events.sort_by_key(|e| (e.timestamp, e.activity));
            traces.insert(
                (object_type, id),
                Trace {
                    activities: events.into_iter().map(|e| e.activity).collect(),
                },
            );
        }
        traces
    }
}

/// Discover the object-centric directly-follows graph: one classic
/// [`DirectlyFollowsGraph`] per object *type*, each discovered from the
/// per-object traces of that type (see [`OcelLog::traces_per_object`]).
///
/// Refuses a log where no event references any object — there is no
/// object-centric model to discover ([`OcelError::NoObjectReferences`]).
pub fn object_centric_dfg<'a>(
    log: &OcelLog<'a>,
) -> Result<BTreeMap<&'a str, DirectlyFollowsGraph>, OcelError> {
    let traces = log.traces_per_object();
    if traces.is_empty() {
        return Err(OcelError::NoObjectReferences);
    }
    let mut models: BTreeMap<&str, Vec<Trace<'_>>> = BTreeMap::new();
    for ((object_type, _), trace) in traces {
        models.entry(object_type).or_default().push(trace);
    }
    let mut dfgs = BTreeMap::new();
    for (object_type, traces) in models {
        dfgs.insert(object_type, DirectlyFollowsGraph::discover(&traces));
    }
    Ok(dfgs)
}

/// The result of certifying a whole log: per-object replay results plus the
/// aggregate verdict (conformant iff *every* object's trace is conformant).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OcelConformance<'a> {
    /// `(object_type, id) -> replay result` for every object in the log.
    pub per_object: BTreeMap<(&'a str, &'a str), ConformanceResult>,
}

impl<'a> OcelConformance<'a> {
    /// Conformant iff every object trace replays conformantly against its
    /// type's model. A log whose objects all conform is conformant.
    pub fn verdict(&self) -> ConformanceVerdict {
        if self
            .per_object
            .values()
            .all(|r| r.verdict() == ConformanceVerdict::Conformant)
        {
            ConformanceVerdict::Conformant
        } else {
            ConformanceVerdict::NonConformant
        }
    }

    /// Replay fitness aggregated over all objects:
    /// sum(legal_moves) / sum(total_moves), 1.0 when no moves exist.
    pub fn fitness(&self) -> f64 {
        let legal: u64 = self.per_object.values().map(|r| r.legal_moves).sum();
        let total: u64 = self.per_object.values().map(|r| r.total_moves).sum();
        if total == 0 {
            1.0
        } else {
            legal as f64 / total as f64
        }
    }
}

/// Token-replay conformance, per object: each object's projected trace is
/// replayed against the discovered model of its object *type*
/// ([`object_centric_dfg`] of the same log).
///
/// Refuses the same degenerate logs as discovery does.
pub fn replay_per_object<'a>(log: &OcelLog<'a>) -> Result<OcelConformance<'a>, OcelError> {
    let models = object_centric_dfg(log)?;
    replay_per_object_against(&models, log)
}

/// Replay a log's per-object traces against *reference* models (typically
/// discovered from a different, known-good log). This is where
/// non-conformance is actually observable: a self-discovered model contains
/// every behavior of its own log by construction.
///
/// Object types absent from the reference models are non-conformant (no model
/// to satisfy).
pub fn replay_per_object_against<'a>(
    models: &BTreeMap<&'a str, DirectlyFollowsGraph>,
    log: &OcelLog<'a>,
) -> Result<OcelConformance<'a>, OcelError> {
    if log.traces_per_object().is_empty() {
        return Err(OcelError::NoObjectReferences);
    }
    let mut per_object = BTreeMap::new();
    for ((object_type, id), trace) in log.traces_per_object() {
        let result = match models.get(object_type) {
            Some(model) => crate::mining::replay(model, &trace),
            None => ConformanceResult {
                legal_moves: 0,
                total_moves: trace.len().saturating_sub(1) as u64,
                start_ok: false,
                end_ok: false,
                first_violation: None,
            },
        };
        per_object.insert((object_type, id), result);
    }
    Ok(OcelConformance { per_object })
}

/// Adapter onto the published OCEL v2 types (`wasm4pm-compat =26.8.7`).
///
/// Compiled only under the `ocel-compat` feature (see the module docs for why
/// this cannot be the default layer). The published types are the admissible
/// source: [`borrow_compat`] projects a `&CompatLog` into this module's
/// borrowed, deterministic layer without copying the strings — the returned
/// [`OcelLog`] borrows its ids/activities from the compat log itself.
#[cfg(feature = "ocel-compat")]
pub mod compat {
    use super::{OcelError, OcelEvent, OcelLog, OcelObject};
    use alloc::collections::BTreeMap;
    use alloc::vec::Vec;
    pub use wasm4pm_compat::ocel::{
        EventObjectLink, Object as CompatObject, ObjectChange, OcelAttribute, OcelAttributeValue,
        OcelEvent as CompatEvent, OcelLog as CompatLog, OcelRefusal,
    };

    /// Project a published [`CompatLog`] into this module's borrowed
    /// `no_std` layer, without copying any id/activity/type string — the
    /// returned log borrows from `log`.
    ///
    /// Typed refusals (no panic):
    /// - [`OcelError::EmptyLog`] when the compat log has zero events.
    /// - [`OcelError::NoObjectReferences`] when no event references any
    ///   object — the same condition the compat crate's own
    ///   `OcelLog::validate` refuses as `OcelRefusal::EmptyEventObjectLinks`.
    ///
    /// Deterministic: objects per event are gathered in link insertion order;
    /// object *types* are resolved through a `BTreeMap` lookup, so the same
    /// compat log always yields the same projection.
    pub fn borrow_compat(log: &CompatLog) -> Result<OcelLog<'_>, OcelError> {
        if log.events().is_empty() {
            return Err(OcelError::EmptyLog);
        }
        if log.event_object_links().is_empty() {
            return Err(OcelError::NoObjectReferences);
        }
        let mut object_types: BTreeMap<&str, &str> = BTreeMap::new();
        for object in log.objects() {
            object_types.insert(object.id(), object.object_type());
        }
        let mut events: Vec<OcelEvent<'_>> = Vec::with_capacity(log.events().len());
        for event in log.events() {
            let objects: Vec<OcelObject<'_>> = log
                .event_object_links()
                .iter()
                .filter(|link| link.event_id() == event.id())
                .filter_map(|link| {
                    object_types.get(link.object_id()).map(|&ty| OcelObject {
                        id: link.object_id(),
                        object_type: ty,
                    })
                })
                .collect();
            events.push(OcelEvent {
                activity: event.activity(),
                timestamp: event.timestamp_ns().unwrap_or(0),
                objects,
            });
        }
        OcelLog::new(events)
    }

    #[cfg(all(test, feature = "ocel-compat"))]
    mod compat_tests {
        use super::*;
        use crate::mining::ocel::{object_centric_dfg, replay_per_object};
        use crate::mining::ConformanceVerdict;

        /// Version pin: the mapping below relies on exact API shapes of
        /// `wasm4pm-compat` 26.8.7 — `timestamp_ns() == None` for the 0
        /// sentinel, accessor (not field) access, and `OcelRefusal` as the
        /// typed structural refusal. The `=26.8.7` exact requirement in
        /// `Cargo.toml` is the enforcement; this test fails loudly if the
        /// semantics this adapter depends on drift.
        #[test]
        fn version_pin_compat_api_shapes_we_rely_on() {
            // 0 is the no-timestamp sentinel, not a time.
            let bare = CompatEvent::new("e0", "act");
            assert_eq!(bare.timestamp_ns(), None);
            assert_eq!(bare.activity(), "act");
            let stamped = CompatEvent::new("e1", "act").at_ns(7);
            assert_eq!(stamped.timestamp_ns(), Some(7));

            // Object identity is (id, object_type) via accessors.
            let object = CompatObject::new("o-1", "order");
            assert_eq!(object.id(), "o-1");
            assert_eq!(object.object_type(), "order");

            // The compat crate refuses an object-centrically empty log.
            let empty = CompatLog::new([], [CompatEvent::new("e0", "orphan")], [], [], []);
            assert_eq!(empty.validate(), Err(OcelRefusal::EmptyEventObjectLinks));
            // ...and so does our projection, in the module's own vocabulary.
            assert_eq!(
                borrow_compat(&empty).unwrap_err(),
                OcelError::NoObjectReferences
            );
        }

        #[test]
        fn compat_log_mines_and_conforms_end_to_end() {
            let log = CompatLog::new(
                [
                    CompatObject::new("o-1", "order"),
                    CompatObject::new("i-7", "item"),
                ],
                [
                    CompatEvent::new("e1", "create").at_ns(1),
                    CompatEvent::new("e2", "pick").at_ns(2),
                    CompatEvent::new("e3", "pay").at_ns(3),
                    CompatEvent::new("e4", "pack").at_ns(4),
                    CompatEvent::new("e5", "ship").at_ns(5),
                ],
                [
                    EventObjectLink::new("e1", "o-1"),
                    EventObjectLink::new("e2", "i-7"),
                    EventObjectLink::new("e3", "o-1"),
                    EventObjectLink::new("e4", "i-7"),
                    EventObjectLink::new("e5", "o-1"),
                ],
                [],
                [],
            );
            let borrowed = borrow_compat(&log).unwrap();

            let models = object_centric_dfg(&borrowed).unwrap();
            assert_eq!(models.len(), 2);
            assert!(models["order"].follows("create", "pay"));
            assert!(models["item"].follows("pick", "pack"));

            let result = replay_per_object(&borrowed).unwrap();
            assert_eq!(result.verdict(), ConformanceVerdict::Conformant);
            assert!((result.fitness() - 1.0).abs() < 1e-9);
        }

        #[test]
        fn compat_log_dangling_link_object_is_dropped_from_projection() {
            // A link whose object is absent from log.objects() cannot resolve
            // a type; borrow_compat must not panic — the reference is skipped
            // (the compat crate's own validate() would refuse it outright).
            let log = CompatLog::new(
                [CompatObject::new("o-1", "order")],
                [CompatEvent::new("e1", "create").at_ns(1)],
                [
                    EventObjectLink::new("e1", "o-1"),
                    EventObjectLink::new("e1", "ghost"),
                ],
                [],
                [],
            );
            let borrowed = borrow_compat(&log).unwrap();
            assert_eq!(borrowed.traces_per_object().len(), 1);
            assert_eq!(
                borrowed.traces_per_object()[&("order", "o-1")].activities,
                ["create"]
            );
        }

        #[test]
        fn compat_empty_log_is_refused_typed() {
            let log = CompatLog::new([], [], [], [], []);
            assert_eq!(borrow_compat(&log).unwrap_err(), OcelError::EmptyLog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(id: &'static str, ty: &'static str) -> OcelObject<'static> {
        OcelObject {
            id,
            object_type: ty,
        }
    }

    /// Two orders, each touched by create→pay→ship; an item touched by
    /// pick→pack on some of the same events (multi-object events).
    fn sample_log() -> OcelLog<'static> {
        OcelLog::new(vec![
            OcelEvent {
                activity: "create",
                timestamp: 1,
                objects: vec![obj("o-1", "order")],
            },
            OcelEvent {
                activity: "pick",
                timestamp: 2,
                objects: vec![obj("i-7", "item")],
            },
            OcelEvent {
                activity: "pay",
                timestamp: 3,
                objects: vec![obj("o-1", "order")],
            },
            OcelEvent {
                activity: "pack",
                timestamp: 4,
                objects: vec![obj("i-7", "item")],
            },
            OcelEvent {
                activity: "ship",
                timestamp: 5,
                objects: vec![obj("o-1", "order")],
            },
        ])
        .unwrap()
    }

    #[test]
    fn multi_object_log_discovery_gives_one_dfg_per_type() {
        let log = sample_log();
        let models = object_centric_dfg(&log).unwrap();

        assert_eq!(models.len(), 2);
        let order = &models["order"];
        assert!(order.follows("create", "pay"));
        assert!(order.follows("pay", "ship"));
        assert!(!order.follows("create", "ship"));
        assert_eq!(order.trace_count, 1);

        let item = &models["item"];
        assert!(item.follows("pick", "pack"));
        assert!(!item.follows("create", "pay"));
        assert!(item.is_start("pick"));
        assert!(item.is_end("pack"));
    }

    #[test]
    fn interleaved_timestamps_project_deterministically() {
        // Events for one object inserted out of order; projection must sort
        // by timestamp, not insertion order.
        let log = OcelLog::new(vec![
            OcelEvent {
                activity: "pay",
                timestamp: 5,
                objects: vec![obj("o-1", "order")],
            },
            OcelEvent {
                activity: "create",
                timestamp: 2,
                objects: vec![obj("o-1", "order")],
            },
        ])
        .unwrap();
        let traces = log.traces_per_object();
        assert_eq!(
            traces[&("order", "o-1")].activities,
            ["create", "pay"] // sorted by timestamp
        );
        let models = object_centric_dfg(&log).unwrap();
        assert!(models["order"].follows("create", "pay"));
        assert!(!models["order"].follows("pay", "create"));
    }

    #[test]
    fn conformant_log_replays_per_object() {
        let log = sample_log();
        let result = replay_per_object(&log).unwrap();

        assert_eq!(result.per_object.len(), 2);
        assert!(result.per_object.contains_key(&("order", "o-1")));
        assert!(result.per_object.contains_key(&("item", "i-7")));
        assert_eq!(result.verdict(), ConformanceVerdict::Conformant);
        assert!((result.fitness() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn divergent_object_is_nonconformant() {
        // Reference log: orders follow create→pay→ship. Unknown object type is
        // also refused by the reference models.
        let reference = sample_log();
        let models = object_centric_dfg(&reference).unwrap();

        // The reference log conforms to its own model.
        assert_eq!(
            replay_per_object_against(&models, &reference)
                .unwrap()
                .verdict(),
            ConformanceVerdict::Conformant
        );

        // A divergent log: o-9 does create→ship, which is not an edge of the
        // reference model (create>pay, pay>ship only). Also an unknown object
        // type "invoice" with no reference model at all.
        let divergent = OcelLog::new(vec![
            OcelEvent {
                activity: "create",
                timestamp: 1,
                objects: vec![obj("o-9", "order")],
            },
            OcelEvent {
                activity: "ship",
                timestamp: 2,
                objects: vec![obj("o-9", "order"), obj("inv-1", "invoice")],
            },
        ])
        .unwrap();

        let result = replay_per_object_against(&models, &divergent).unwrap();
        // o-9: illegal move create>ship.
        assert_eq!(result.verdict(), ConformanceVerdict::NonConformant);
        let o9 = &result.per_object[&("order", "o-9")];
        assert_eq!(o9.legal_moves, 0);
        assert_eq!(o9.total_moves, 1);
        assert_eq!(o9.first_violation, Some(0));
        // inv-1: object type absent from the reference models.
        let inv = &result.per_object[&("invoice", "inv-1")];
        assert!(!inv.start_ok);
        assert!(!inv.end_ok);
        assert!((result.fitness() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn determinism_two_runs_identical() {
        let log = sample_log();
        let m1 = object_centric_dfg(&log).unwrap();
        let m2 = object_centric_dfg(&log).unwrap();
        assert_eq!(m1, m2);

        let r1 = replay_per_object(&log).unwrap();
        let r2 = replay_per_object(&log).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn empty_log_is_refused_typed_not_panic() {
        let err = OcelLog::new(Vec::new()).unwrap_err();
        assert_eq!(err, OcelError::EmptyLog);
    }

    #[test]
    fn objectless_log_is_refused_typed_not_panic() {
        let log = OcelLog::new(vec![OcelEvent {
            activity: "orphan",
            timestamp: 1,
            objects: Vec::new(),
        }])
        .unwrap();
        assert_eq!(
            object_centric_dfg(&log).unwrap_err(),
            OcelError::NoObjectReferences
        );
        assert_eq!(
            replay_per_object(&log).unwrap_err(),
            OcelError::NoObjectReferences
        );
    }

    #[test]
    fn event_shared_by_two_objects_appears_in_both_traces() {
        let log = OcelLog::new(vec![
            OcelEvent {
                activity: "assemble",
                timestamp: 1,
                objects: vec![obj("i-1", "item"), obj("o-1", "order")],
            },
            OcelEvent {
                activity: "ship",
                timestamp: 2,
                objects: vec![obj("o-1", "order")],
            },
        ])
        .unwrap();
        let traces = log.traces_per_object();
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[&("item", "i-1")].activities, ["assemble"]);
        assert_eq!(traces[&("order", "o-1")].activities, ["assemble", "ship"]);
    }

    #[test]
    fn log_len_and_events_roundtrip() {
        let log = sample_log();
        assert_eq!(log.len(), 5);
        assert!(!log.is_empty());
        assert_eq!(log.events().len(), 5);
        assert_eq!(log.events()[0].activity, "create");
    }
}
