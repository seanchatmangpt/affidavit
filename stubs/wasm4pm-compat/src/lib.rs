//! Minimal build-compatible stub for `wasm4pm-compat`.
//!
//! Provides the API surface that `affidavit` and its 110 reference test files
//! depend on. Semantics are faithful where tests assert behaviour; types are
//! minimal stubs elsewhere.

#![allow(unused)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(unsized_const_params)]

use std::collections::HashMap;
use std::marker::PhantomData;

// ── Require<B> / IsTrue — the const-generic law-enforcement kernel ────────────

pub struct Require<const B: bool>;
pub trait IsTrue {}
impl IsTrue for Require<true> {}

// ── state module ──────────────────────────────────────────────────────────────

pub mod state {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Raw(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Parsed(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Admitted(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Projected(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Exportable(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Receipted(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Witnessed(());
    #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Refused(());
}

// ── evidence module ───────────────────────────────────────────────────────────

pub mod evidence {
    use std::marker::PhantomData;
    use crate::state;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Evidence<T, S, C> {
        pub value: T,
        _state: PhantomData<S>,
        _chain: PhantomData<C>,
    }

    impl<T, S, C> Evidence<T, S, C> {
        pub fn new(value: T) -> Self {
            Evidence { value, _state: PhantomData, _chain: PhantomData }
        }
        pub fn into_inner(self) -> T { self.value }
        pub fn inner(&self) -> &T { &self.value }
    }

    impl<T, C> Evidence<T, state::Raw, C> {
        pub fn raw(value: T) -> Self { Evidence::new(value) }
        pub fn into_parsed(self) -> Evidence<T, state::Parsed, C> { Evidence::new(self.value) }
    }

    impl<T, C> Evidence<T, state::Parsed, C> {
        pub fn into_admitted(self) -> Evidence<T, state::Admitted, C> { Evidence::new(self.value) }
        pub fn into_projected(self) -> Evidence<T, state::Projected, C> { Evidence::new(self.value) }
    }

    impl<T, C> Evidence<T, state::Admitted, C> {
        pub fn into_exportable(self) -> Evidence<T, state::Exportable, C> { Evidence::new(self.value) }
        pub fn into_receipted(self) -> Evidence<T, state::Receipted, C> { Evidence::new(self.value) }
        pub fn into_projected(self) -> Evidence<T, state::Projected, C> { Evidence::new(self.value) }
    }
}

// ── admission module ──────────────────────────────────────────────────────────

pub mod admission {
    use std::marker::PhantomData;
    use crate::evidence::Evidence;
    use crate::state;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Admission<T, W> {
        pub value: T,
        _witness: PhantomData<W>,
    }

    impl<T, W> Admission<T, W> {
        pub fn new(value: T) -> Self { Admission { value, _witness: PhantomData } }
        pub fn into_evidence(self) -> Evidence<T, state::Admitted, W> {
            Evidence::new(self.value)
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Refusal<R, W> {
        pub reason: R,
        _witness: PhantomData<W>,
    }

    impl<R, W> Refusal<R, W> {
        pub fn new(reason: R) -> Self { Refusal { reason, _witness: PhantomData } }
        pub fn into_reason(self) -> R { self.reason }
    }
}

// ── witness module ────────────────────────────────────────────────────────────

pub mod witness {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WitnessFamily { Standard, ApiGrammar, Implementation, Paper }

    pub trait Witness: Send + Sync + 'static {
        fn id(&self) -> &'static str;
        fn family(&self) -> WitnessFamily;
    }

    macro_rules! witness_marker {
        ($name:ident, $key:literal, $family:expr, $title:literal, $year:expr) => {
            pub struct $name;
            impl $name {
                pub const KEY: &'static str = $key;
                pub const FAMILY: WitnessFamily = $family;
                pub const TITLE: &'static str = $title;
                pub const YEAR: Option<u32> = $year;
            }
            impl Witness for $name {
                fn id(&self) -> &'static str { $key }
                fn family(&self) -> WitnessFamily { $family }
            }
        };
    }

    witness_marker!(Ocel20, "ocel-2.0", WitnessFamily::Standard, "OCEL 2.0", Some(2023));
    witness_marker!(PowlPaper, "powl", WitnessFamily::Paper, "POWL", Some(2023));
    witness_marker!(Pm4pyApiGrammar, "pm4py", WitnessFamily::ApiGrammar, "pm4py", Some(2019));
    witness_marker!(WfNetWitness, "wfnet", WitnessFamily::Standard, "WF-net", Some(2000));
}

// ── law module ────────────────────────────────────────────────────────────────

pub mod law {
    use crate::{Require, IsTrue};

    pub struct Between01<const NUM: u64, const DEN: u64>
    where
        Require<{ DEN > 0 }>: IsTrue,
        Require<{ NUM <= DEN }>: IsTrue,
    {
        _priv: (),
    }

    impl<const NUM: u64, const DEN: u64> Between01<NUM, DEN>
    where
        Require<{ DEN > 0 }>: IsTrue,
        Require<{ NUM <= DEN }>: IsTrue,
    {
        pub fn new() -> Self { Between01 { _priv: () } }
        pub fn num(&self) -> u64 { NUM }
        pub fn den(&self) -> u64 { DEN }
    }

    pub struct ConditionCell<const BITS: u8>
    where
        Require<{ BITS <= 8 }>: IsTrue,
    {
        _priv: (),
    }

    impl<const BITS: u8> ConditionCell<BITS>
    where
        Require<{ BITS <= 8 }>: IsTrue,
    {
        pub fn new() -> Self { ConditionCell { _priv: () } }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum ArcDirectionConst { PlaceToTransition, TransitionToPlace }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum EvidenceMode { Raw, Parsed, Admitted, Refused, Projected, Exportable, Witnessed, Receipted }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum ObjectCentricity { CaseCentric, ObjectCentric, Mixed }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PowlProjectionState {
        Unknown, ProcessTreeProjectable, ExceedsProcessTree, RefusedProjection,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RelationLaw { EventToObject, ObjectToObject, ObjectToEvent }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorkflowPattern {
        Sequence,
        ParallelSplit,
        Synchronization,
        ExclusiveChoice,
        SimpleMerge,
        MultiChoice,
        StructuredSynchronizingMerge,
        MultiMerge,
        StructuredDiscriminator,
        ArbitraryCycles,
        ImplicitTermination,
        MultipleInstancesWithoutSync,
        MultipleInstancesWithDesignTimeKnowledge,
        DeferredChoice,
        InterleavedParallelRouting,
        CancelActivity,
        CancelCase,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum QualityMetricKind { Fitness, Precision, F1, Generalization, Simplicity }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum SoundnessState { Unknown, Claimed, Witnessed }

    impl std::fmt::Display for SoundnessState {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum ProcessShapeKind {
        Event, Trace, EventLog, EventStream, XesLog, OcelLog,
        DirectlyFollowsGraph, ObjectCentricDfg, PetriNet, WorkflowNet, ObjectCentricPetriNet,
        ProcessTree, Powl, DeclareModel, ObjectCentricDeclareModel, LogSkeleton,
        OcpqQuery, Alignment, TokenReplay, ConformanceVerdict, PredictionProblem, Receipt,
    }
}

// ── conformance module ────────────────────────────────────────────────────────

pub mod conformance {
    use crate::law::{QualityMetricKind, Between01};
    use crate::{Require, IsTrue};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Fitness(pub f64);
    impl Fitness {
        pub fn new(v: f64) -> Option<Self> {
            if v.is_finite() && v >= 0.0 && v <= 1.0 { Some(Fitness(v)) } else { None }
        }
        pub fn get(&self) -> f64 { self.0 }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Precision(pub f64);
    impl Precision {
        pub fn new(v: f64) -> Option<Self> {
            if v.is_finite() && v >= 0.0 && v <= 1.0 { Some(Precision(v)) } else { None }
        }
        pub fn get(&self) -> f64 { self.0 }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct F1(pub f64);
    impl F1 {
        pub fn new(v: f64) -> Option<Self> {
            if v.is_finite() && v >= 0.0 && v <= 1.0 { Some(F1(v)) } else { None }
        }
        pub fn get(&self) -> f64 { self.0 }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Generalization(pub f64);
    impl Generalization {
        pub fn new(v: f64) -> Option<Self> {
            if v.is_finite() && v >= 0.0 && v <= 1.0 { Some(Generalization(v)) } else { None }
        }
        pub fn get(&self) -> f64 { self.0 }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Simplicity(pub f64);
    impl Simplicity {
        pub fn new(v: f64) -> Option<Self> {
            if v.is_finite() && v >= 0.0 && v <= 1.0 { Some(Simplicity(v)) } else { None }
        }
        pub fn get(&self) -> f64 { self.0 }
    }

    fn clamp_finite(v: f64) -> f64 {
        if v.is_nan() { 0.0 }
        else if v > 1.0 { 1.0 }
        else if v < 0.0 { 0.0 }
        else { v }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ConformanceResult {
        pub fitness: f64,
        pub precision: Option<f64>,
        pub generalization: Option<f64>,
        pub simplicity: Option<f64>,
        pub total_traces: usize,
        pub fitting_traces: usize,
        pub deviating_traces: usize,
    }

    impl ConformanceResult {
        pub fn new(fitness: f64, total: usize, fitting: usize, deviating: usize) -> Self {
            ConformanceResult { fitness, precision: None, generalization: None, simplicity: None,
                total_traces: total, fitting_traces: fitting, deviating_traces: deviating }
        }
        pub fn with_precision(mut self, v: f64) -> Self { self.precision = Some(clamp_finite(v)); self }
        pub fn with_generalization(mut self, v: f64) -> Self { self.generalization = Some(clamp_finite(v)); self }
        pub fn with_simplicity(mut self, v: f64) -> Self { self.simplicity = Some(clamp_finite(v)); self }
        pub fn conformance_rate(&self) -> f64 {
            if self.total_traces == 0 { 0.0 } else { self.fitting_traces as f64 / self.total_traces as f64 }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Deviation { pub position: usize, pub label: String }
    impl Deviation {
        pub fn new(position: usize, label: &str) -> Self {
            Deviation { position, label: label.to_string() }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct ConformanceVerdict {
        pub fitness: Option<Fitness>,
        pub deviations: Vec<Deviation>,
    }
    impl ConformanceVerdict {
        pub fn new() -> Self { ConformanceVerdict { fitness: None, deviations: Vec::new() } }
        pub fn is_perfect(&self) -> bool {
            matches!(&self.fitness, Some(Fitness(f)) if (*f - 1.0).abs() < 1e-12)
                && self.deviations.is_empty()
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ConformanceRefusal {
        EmptyModel, TooManyDeviations, InvalidProfile,
        MissingLog, MissingModel, MissingDeviationPath,
        FitnessUnavailable, PrecisionUnavailable, F1Unavailable,
        GeneralizationUnavailable, SimplicityUnavailable,
    }

    impl std::fmt::Display for ConformanceRefusal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum QualityDimension { Fitness, Precision, F1, Generalization, Simplicity }

    // Metric<KIND, NUM, DEN> — const-generic typed quality metric
    pub struct Metric<const KIND: QualityMetricKind, const NUM: u64, const DEN: u64>
    where
        Require<{ DEN > 0 }>: IsTrue,
        Require<{ NUM <= DEN }>: IsTrue,
    {
        _priv: (),
    }
    impl<const KIND: QualityMetricKind, const NUM: u64, const DEN: u64> Metric<KIND, NUM, DEN>
    where
        Require<{ DEN > 0 }>: IsTrue,
        Require<{ NUM <= DEN }>: IsTrue,
    {
        pub fn new() -> Self { Metric { _priv: () } }
        pub fn kind(&self) -> QualityMetricKind { KIND }
        pub fn num(&self) -> u64 { NUM }
        pub fn den(&self) -> u64 { DEN }
        pub fn as_f64(&self) -> f64 { NUM as f64 / DEN as f64 }
    }

    // Typed const-generic aliases
    pub type FitnessConst<const N: u64, const D: u64> = Metric<{ QualityMetricKind::Fitness }, N, D>;
    pub type PrecisionConst<const N: u64, const D: u64> = Metric<{ QualityMetricKind::Precision }, N, D>;
    pub type F1Const<const N: u64, const D: u64> = Metric<{ QualityMetricKind::F1 }, N, D>;
    pub type GeneralizationConst<const N: u64, const D: u64> = Metric<{ QualityMetricKind::Generalization }, N, D>;
    pub type SimplicityConst<const N: u64, const D: u64> = Metric<{ QualityMetricKind::Simplicity }, N, D>;

    // QualityProfile — five-dimension conformance quality composite type
    pub struct QualityProfile<
        const FN: u64, const FD: u64,
        const PN: u64, const PD: u64,
        const F1N: u64, const F1D: u64,
        const GN: u64, const GD: u64,
        const SN: u64, const SD: u64,
    >
    where
        Require<{ FD > 0 }>: IsTrue, Require<{ FN <= FD }>: IsTrue,
        Require<{ PD > 0 }>: IsTrue, Require<{ PN <= PD }>: IsTrue,
        Require<{ F1D > 0 }>: IsTrue, Require<{ F1N <= F1D }>: IsTrue,
        Require<{ GD > 0 }>: IsTrue, Require<{ GN <= GD }>: IsTrue,
        Require<{ SD > 0 }>: IsTrue, Require<{ SN <= SD }>: IsTrue,
    {
        pub fitness: Between01<FN, FD>,
        pub precision: Between01<PN, PD>,
        pub f1: Between01<F1N, F1D>,
        pub generalization: Between01<GN, GD>,
        pub simplicity: Between01<SN, SD>,
    }

    impl<
        const FN: u64, const FD: u64,
        const PN: u64, const PD: u64,
        const F1N: u64, const F1D: u64,
        const GN: u64, const GD: u64,
        const SN: u64, const SD: u64,
    > QualityProfile<FN, FD, PN, PD, F1N, F1D, GN, GD, SN, SD>
    where
        Require<{ FD > 0 }>: IsTrue, Require<{ FN <= FD }>: IsTrue,
        Require<{ PD > 0 }>: IsTrue, Require<{ PN <= PD }>: IsTrue,
        Require<{ F1D > 0 }>: IsTrue, Require<{ F1N <= F1D }>: IsTrue,
        Require<{ GD > 0 }>: IsTrue, Require<{ GN <= GD }>: IsTrue,
        Require<{ SD > 0 }>: IsTrue, Require<{ SN <= SD }>: IsTrue,
    {
        pub fn new() -> Self {
            QualityProfile {
                fitness: Between01::new(), precision: Between01::new(), f1: Between01::new(),
                generalization: Between01::new(), simplicity: Between01::new(),
            }
        }
        pub fn default() -> Self { Self::new() }
    }
}

// ── ocel module ───────────────────────────────────────────────────────────────

pub mod ocel {
    use std::collections::HashMap;

    // ── OcelLog admission types ──────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq)]
    pub struct Object {
        pub id: String,
        pub obj_type: String,
        // Builder-pattern state
        _attrs: Vec<OcelAttribute>,
    }

    impl Object {
        pub fn new(id: &str, obj_type: &str) -> Self {
            Object { id: id.to_string(), obj_type: obj_type.to_string(), _attrs: Vec::new() }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn object_type(&self) -> &str { &self.obj_type }
        pub fn attributes(&self) -> &[OcelAttribute] { &self._attrs }
        pub fn with_attribute(mut self, a: OcelAttribute) -> Self { self._attrs.push(a); self }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OcelEvent {
        pub id: String,
        pub event_type: String,
        _ts: Option<u64>,
        _attrs: Vec<OcelAttribute>,
    }

    impl OcelEvent {
        pub fn new(id: &str, event_type: &str) -> Self {
            OcelEvent { id: id.to_string(), event_type: event_type.to_string(), _ts: None, _attrs: Vec::new() }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn activity(&self) -> &str { &self.event_type }
        pub fn timestamp_ns(&self) -> Option<u64> { self._ts }
        pub fn attributes(&self) -> &[OcelAttribute] { &self._attrs }
        pub fn at_ns(mut self, ts: u64) -> Self { self._ts = Some(ts); self }
        pub fn with_attribute(mut self, a: OcelAttribute) -> Self { self._attrs.push(a); self }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EventObjectLink {
        pub event_id: String,
        pub object_id: String,
        _qualifier: Option<String>,
    }

    impl EventObjectLink {
        pub fn new(event_id: &str, object_id: &str) -> Self {
            EventObjectLink { event_id: event_id.to_string(), object_id: object_id.to_string(), _qualifier: None }
        }
        pub fn event_id(&self) -> &str { &self.event_id }
        pub fn object_id(&self) -> &str { &self.object_id }
        pub fn qualifier(&self) -> Option<&str> { self._qualifier.as_deref() }
        pub fn qualified(mut self, q: &str) -> Self { self._qualifier = Some(q.to_string()); self }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct ObjectObjectLink {
        pub source_id: String,
        pub target_id: String,
        _qualifier: Option<String>,
    }

    impl ObjectObjectLink {
        pub fn new(source_id: &str, target_id: &str) -> Self {
            ObjectObjectLink { source_id: source_id.to_string(), target_id: target_id.to_string(), _qualifier: None }
        }
        pub fn source_id(&self) -> &str { &self.source_id }
        pub fn target_id(&self) -> &str { &self.target_id }
        pub fn qualifier(&self) -> Option<&str> { self._qualifier.as_deref() }
        pub fn qualified(mut self, q: &str) -> Self { self._qualifier = Some(q.to_string()); self }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct ObjectChange {
        _object_id: String,
        _attribute: String,
        _value: String,
        _ts: Option<u64>,
    }

    impl ObjectChange {
        pub fn new(object_id: &str, attribute: &str, value: &str) -> Self {
            ObjectChange { _object_id: object_id.to_string(), _attribute: attribute.to_string(), _value: value.to_string(), _ts: None }
        }
        pub fn object_id(&self) -> &str { &self._object_id }
        pub fn attribute(&self) -> &str { &self._attribute }
        pub fn value(&self) -> &str { &self._value }
        pub fn timestamp_ns(&self) -> Option<u64> { self._ts }
        pub fn at_ns(mut self, ts: u64) -> Self { self._ts = Some(ts); self }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum OcelRefusal {
        EmptyEventObjectLinks,
        DanglingEventObjectLink,
    }

    impl std::fmt::Display for OcelRefusal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                OcelRefusal::EmptyEventObjectLinks => write!(f, "EmptyEventObjectLinks"),
                OcelRefusal::DanglingEventObjectLink => write!(f, "DanglingEventObjectLink"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct OcelLog {
        objects: Vec<Object>,
        events: Vec<OcelEvent>,
        event_object_links: Vec<EventObjectLink>,
        object_object_links: Vec<ObjectObjectLink>,
        object_changes: Vec<ObjectChange>,
    }

    impl OcelLog {
        pub fn new(
            objects: Vec<Object>, events: Vec<OcelEvent>,
            event_object_links: Vec<EventObjectLink>,
            object_object_links: Vec<ObjectObjectLink>,
            object_changes: Vec<ObjectChange>,
        ) -> Self {
            OcelLog { objects, events, event_object_links, object_object_links, object_changes }
        }
        pub fn objects(&self) -> &[Object] { &self.objects }
        pub fn events(&self) -> &[OcelEvent] { &self.events }
        pub fn event_object_links(&self) -> &[EventObjectLink] { &self.event_object_links }
        pub fn object_object_links(&self) -> &[ObjectObjectLink] { &self.object_object_links }
        pub fn object_changes(&self) -> &[ObjectChange] { &self.object_changes }

        pub fn validate(&self) -> Result<(), OcelRefusal> {
            let obj_ids: std::collections::HashSet<&str> =
                self.objects.iter().map(|o| o.id.as_str()).collect();
            for ev in &self.events {
                let links: Vec<_> = self.event_object_links.iter()
                    .filter(|l| l.event_id == ev.id).collect();
                if links.is_empty() { return Err(OcelRefusal::EmptyEventObjectLinks); }
                for l in links {
                    if !obj_ids.contains(l.object_id.as_str()) {
                        return Err(OcelRefusal::DanglingEventObjectLink);
                    }
                }
            }
            Ok(())
        }
    }

    // ── OCEL attribute types ─────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq)]
    pub enum OcelAttributeValue {
        Integer(i64),
        Float(f64),
        Boolean(bool),
        String(std::string::String),
        TimestampNs(u64),
        List(Vec<OcelAttributeValue>),
        Map(Vec<(std::string::String, OcelAttributeValue)>),
        Null,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OcelAttribute {
        pub key: std::string::String,
        pub value: OcelAttributeValue,
    }

    impl OcelAttribute {
        pub fn new(key: &str, value: OcelAttributeValue) -> Self {
            OcelAttribute { key: key.to_string(), value }
        }
        pub fn string(key: &str, value: &str) -> Self {
            OcelAttribute { key: key.to_string(), value: OcelAttributeValue::String(value.to_string()) }
        }
        pub fn integer(key: &str, value: i64) -> Self {
            OcelAttribute { key: key.to_string(), value: OcelAttributeValue::Integer(value) }
        }
        pub fn float(key: &str, value: f64) -> Self {
            OcelAttribute { key: key.to_string(), value: OcelAttributeValue::Float(value) }
        }
        pub fn boolean(key: &str, value: bool) -> Self {
            OcelAttribute { key: key.to_string(), value: OcelAttributeValue::Boolean(value) }
        }
        pub fn timestamp_ns(key: &str, value: u64) -> Self {
            OcelAttribute { key: key.to_string(), value: OcelAttributeValue::TimestampNs(value) }
        }
    }

    // ── OCEL 2.0 mining types ────────────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq)]
    pub enum OCELAttributeValue {
        Integer(i64),
        Float(f64),
        Boolean(bool),
        String(std::string::String),
        Time(std::string::String),
        Null,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct OCELEventAttribute {
        pub name: std::string::String,
        pub value: OCELAttributeValue,
    }

    impl OCELEventAttribute {
        pub fn string(name: &str, value: std::string::String) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::String(value) }
        }
        pub fn integer(name: &str, value: i64) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::Integer(value) }
        }
        pub fn boolean(name: &str, value: bool) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::Boolean(value) }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct OCELRelationship {
        pub object_id: std::string::String,
        pub qualifier: std::string::String,
    }

    impl OCELRelationship {
        pub fn new(event_id: std::string::String, object_id: std::string::String) -> Self {
            OCELRelationship { object_id, qualifier: std::string::String::new() }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct OCELEvent {
        pub id: std::string::String,
        pub event_type: std::string::String,
        pub relationships: Vec<OCELRelationship>,
        pub attributes: Vec<OCELEventAttribute>,
    }

    impl OCELEvent {
        pub fn new(id: impl Into<std::string::String>, event_type: &str) -> Self {
            OCELEvent { id: id.into(), event_type: event_type.to_string(), ..Default::default() }
        }
        pub fn with_attribute(mut self, a: OCELEventAttribute) -> Self {
            self.attributes.push(a); self
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct OCELObject {
        pub id: std::string::String,
        pub object_type: std::string::String,
        pub attributes: Vec<OCELEventAttribute>,
        pub relationships: Vec<OCELRelationship>,
    }

    impl OCELObject {
        pub fn new(id: impl Into<std::string::String>, object_type: &str) -> Self {
            OCELObject { id: id.into(), object_type: object_type.to_string(), ..Default::default() }
        }
        pub fn with_attribute(mut self, a: OCELEventAttribute) -> Self {
            self.attributes.push(a); self
        }
    }

    #[derive(Debug, Clone)]
    pub struct OCELType {
        pub name: std::string::String,
        pub attributes: Vec<OCELTypeAttribute>,
    }

    #[derive(Debug, Clone)]
    pub struct OCELTypeAttribute {
        pub name: std::string::String,
        pub value_type: std::string::String,
    }

    #[derive(Debug, Clone, Default)]
    pub struct OCEL {
        pub events: Vec<OCELEvent>,
        pub objects: Vec<OCELObject>,
    }

    impl OCEL {
        pub fn new(events: Vec<OCELEvent>, objects: Vec<OCELObject>) -> Self {
            OCEL { events, objects }
        }
        pub fn event_set(&self) -> &Vec<OCELEvent> { &self.events }
        pub fn object_set(&self) -> &Vec<OCELObject> { &self.objects }
        pub fn count_objects_of_type(&self, t: &str) -> usize {
            self.objects.iter().filter(|o| o.object_type == t).count()
        }
        pub fn e2o(&self, event_id: &str) -> Vec<(&str, &str)> {
            self.events.iter()
                .find(|ev| ev.id == event_id)
                .map(|ev| ev.relationships.iter().map(|r| (r.object_id.as_str(), r.qualifier.as_str())).collect())
                .unwrap_or_default()
        }
        pub fn o2o(&self, object_id: &str) -> Vec<(&str, &str)> {
            self.objects.iter()
                .find(|o| o.id == object_id)
                .map(|o| o.relationships.iter().map(|r| (r.object_id.as_str(), r.qualifier.as_str())).collect())
                .unwrap_or_default()
        }
        pub fn eval(&self, event_id: &str) -> Option<HashMap<std::string::String, OCELAttributeValue>> {
            self.events.iter().find(|ev| ev.id == event_id).map(|ev| {
                ev.attributes.iter().map(|a| (a.name.clone(), a.value.clone())).collect()
            })
        }
    }

    // ObjectTypeCardinality
    #[derive(Debug, Clone, Default)]
    pub struct ObjectTypeCardinality {
        pub min_count: Option<usize>,
        pub max_count: Option<usize>,
        pub created_by: Vec<std::string::String>,
        pub terminated_by: Vec<std::string::String>,
    }

    impl ObjectTypeCardinality {
        pub fn admits(&self, count: usize) -> bool {
            let ok_min = self.min_count.map_or(true, |m| count >= m);
            let ok_max = self.max_count.map_or(true, |m| count <= m);
            ok_min && ok_max
        }
    }
}

// ── petri module ──────────────────────────────────────────────────────────────

pub mod petri {
    use std::collections::HashMap;
    use std::marker::PhantomData;
    use crate::{Require, IsTrue};
    use crate::law::{ArcDirectionConst, SoundnessState};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Place { pub id: String }
    impl Place {
        pub fn new(id: &str) -> Self { Place { id: id.to_string() } }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Transition { pub id: String, pub label: String }
    impl Transition {
        pub fn new(id: &str, label: &str) -> Self { Transition { id: id.to_string(), label: label.to_string() } }
        pub fn silent(id: &str) -> Self { Transition { id: id.to_string(), label: String::new() } }
        pub fn id(&self) -> &str { &self.id }
        pub fn label(&self) -> &str { &self.label }
        pub fn is_silent(&self) -> bool { self.label.is_empty() }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ArcDirection { PlaceToTransition, TransitionToPlace }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Arc {
        pub source: String,
        pub target: String,
        pub weight: u32,
        pub object_type: Option<(String, bool)>,
        _dir: ArcDirection,
    }
    impl Arc {
        pub fn new(source: &str, target: &str) -> Self {
            Arc { source: source.to_string(), target: target.to_string(), weight: 1,
                  object_type: None, _dir: ArcDirection::PlaceToTransition }
        }
        pub fn place_to_transition(place: &str, transition: &str) -> Self {
            Arc { source: place.to_string(), target: transition.to_string(), weight: 1,
                  object_type: None, _dir: ArcDirection::PlaceToTransition }
        }
        pub fn transition_to_place(transition: &str, place: &str) -> Self {
            Arc { source: transition.to_string(), target: place.to_string(), weight: 1,
                  object_type: None, _dir: ArcDirection::TransitionToPlace }
        }
        pub fn direction(&self) -> ArcDirection { self._dir.clone() }
        pub fn object_type(&self) -> Option<&str> {
            self.object_type.as_ref().map(|(s, _)| s.as_str())
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Marking(pub HashMap<String, usize>);
    impl Marking {
        pub fn new(entries: impl IntoIterator<Item = (String, usize)>) -> Self {
            Marking(entries.into_iter().collect())
        }
        pub fn empty() -> Self { Marking(HashMap::new()) }
        pub fn mark(&mut self, place_id: &str, tokens: usize) { self.0.insert(place_id.to_string(), tokens); }
        pub fn tokens_on(&self, place_id: &str) -> usize { self.0.get(place_id).copied().unwrap_or(0) }
        pub fn tokens(&self) -> &HashMap<String, usize> { &self.0 }
        pub fn is_empty(&self) -> bool { self.0.is_empty() }
    }

    #[derive(Debug, Clone, Default)]
    pub struct PetriNet {
        pub places: Vec<Place>,
        pub transitions: Vec<Transition>,
        pub arcs: Vec<Arc>,
        pub initial_marking: Marking,
        pub final_marking: Marking,
    }
    impl PetriNet {
        pub fn new(
            places: impl IntoIterator<Item = Place>,
            transitions: impl IntoIterator<Item = Transition>,
            arcs: impl IntoIterator<Item = Arc>,
            initial_marking: Marking,
        ) -> Self {
            PetriNet { places: places.into_iter().collect(), transitions: transitions.into_iter().collect(),
                arcs: arcs.into_iter().collect(), initial_marking, final_marking: Marking::empty() }
        }
        pub fn add_place(&mut self, p: Place) { self.places.push(p); }
        pub fn add_transition(&mut self, t: Transition) { self.transitions.push(t); }
        pub fn add_arc(&mut self, a: Arc) { self.arcs.push(a); }
        pub fn places(&self) -> &[Place] { &self.places }
        pub fn transitions(&self) -> &[Transition] { &self.transitions }
        pub fn arcs(&self) -> &[Arc] { &self.arcs }
        pub fn initial_marking(&self) -> &Marking { &self.initial_marking }
        pub fn validate(&self) -> Result<(), PetriRefusal> { Ok(()) }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PetriRefusal {
        IsolatedPlace,
        IsolatedTransition,
        MissingInitialMarking,
        InvalidWeight,
        MissingFinalMarking,
        UnsafeNet,
        InvalidInstanceBounds,
        ObjectTypeNotPreserved,
        DeadTransition,
    }

    // BipartiteArcConst
    pub struct BipartiteArcConst<const DIR: ArcDirectionConst, W> {
        _place_id: String,
        _transition_id: String,
        _weight: W,
    }
    impl<const DIR: ArcDirectionConst, W: Copy> BipartiteArcConst<DIR, W> {
        pub fn new(place_id: &str, transition_id: &str, weight: W) -> Self {
            BipartiteArcConst { _place_id: place_id.to_string(), _transition_id: transition_id.to_string(), _weight: weight }
        }
        pub fn place_id(&self) -> &str { &self._place_id }
        pub fn transition_id(&self) -> &str { &self._transition_id }
        pub fn weight(&self) -> W { self._weight }
        pub fn direction(&self) -> ArcDirectionConst { DIR }
    }

    // Typestate bipartite arc types
    pub struct PlaceNodeMarker;
    pub struct TransitionNodeMarker;
    pub trait IsValidArc {}

    pub struct PlaceToTransitionArc<P, T, W>(W, PhantomData<(P, T)>);
    impl<P, T, W: Copy> PlaceToTransitionArc<P, T, W> {
        pub fn new(weight: W) -> Self { PlaceToTransitionArc(weight, PhantomData) }
        pub fn weight(&self) -> W { self.0 }
    }
    impl<P, T, W> IsValidArc for PlaceToTransitionArc<P, T, W> {}

    pub struct TransitionToPlaceArc<T, P, W>(W, PhantomData<(T, P)>);
    impl<T, P, W: Copy> TransitionToPlaceArc<T, P, W> {
        pub fn new(weight: W) -> Self { TransitionToPlaceArc(weight, PhantomData) }
        pub fn weight(&self) -> W { self.0 }
    }
    impl<T, P, W> IsValidArc for TransitionToPlaceArc<T, P, W> {}

    // WfNet (value-level, non-const-generic)
    pub struct SoundnessUnknown;
    pub struct SoundnessClaimed;

    pub struct WfNet<S = SoundnessUnknown> {
        _net: PetriNet,
        _final: Marking,
        _state: PhantomData<S>,
    }
    impl WfNet<SoundnessUnknown> {
        pub fn new(net: PetriNet, final_marking: Marking) -> Self {
            WfNet { _net: net, _final: final_marking, _state: PhantomData }
        }
        pub fn validate(&self) -> Result<(), PetriRefusal> {
            if self._final.is_empty() { Err(PetriRefusal::MissingFinalMarking) } else { Ok(()) }
        }
        pub fn net(&self) -> &PetriNet { &self._net }
        pub fn final_marking(&self) -> Option<&Marking> {
            if self._final.is_empty() { None } else { Some(&self._final) }
        }
        pub fn claim_sound(self) -> WfNet<SoundnessClaimed> {
            WfNet { _net: self._net, _final: self._final, _state: PhantomData }
        }
    }
    impl WfNet<SoundnessClaimed> {
        pub fn net(&self) -> &PetriNet { &self._net }
        pub fn final_marking(&self) -> Option<&Marking> {
            if self._final.is_empty() { None } else { Some(&self._final) }
        }
    }

    // WfNetConst (const-generic soundness typestate)
    pub struct SoundnessProof(());
    impl SoundnessProof {
        pub(crate) fn new() -> Self { SoundnessProof(()) }
    }

    pub struct WfNetConst<const S: SoundnessState> { _priv: () }

    impl WfNetConst<{ SoundnessState::Unknown }> {
        pub fn new() -> Self { WfNetConst { _priv: () } }
        pub fn claim_sound(self) -> WfNetConst<{ SoundnessState::Claimed }> { WfNetConst { _priv: () } }
    }
    impl WfNetConst<{ SoundnessState::Claimed }> {
        pub fn witness_soundness(self, _proof: SoundnessProof) -> WfNetConst<{ SoundnessState::Witnessed }> {
            WfNetConst { _priv: () }
        }
    }
    impl<const S: SoundnessState> WfNetConst<S> {
        pub fn soundness_state(&self) -> SoundnessState { S }
    }

    // SeparableWfNet
    pub struct SeparableWfNet<const S: SoundnessState> {
        pub net: WfNetConst<S>,
    }
    impl<const S: SoundnessState> SeparableWfNet<S> {
        pub fn declare_separable(net: WfNetConst<S>) -> Self { SeparableWfNet { net } }
    }

    // CancellationRegion
    pub struct CancellationRegion { _members: Vec<String> }
    impl CancellationRegion {
        pub fn new(members: impl IntoIterator<Item = impl Into<String>>) -> Self {
            CancellationRegion { _members: members.into_iter().map(|s| s.into()).collect() }
        }
        pub fn members(&self) -> &[String] { &self._members }
    }

    // MultipleInstanceSpecConst
    pub struct MultipleInstanceSpecConst<const MIN: usize, const MAX: usize>
    where
        Require<{ MIN >= 1 }>: IsTrue,
        Require<{ MIN <= MAX }>: IsTrue,
    {
        _priv: (),
    }
    impl<const MIN: usize, const MAX: usize> MultipleInstanceSpecConst<MIN, MAX>
    where
        Require<{ MIN >= 1 }>: IsTrue,
        Require<{ MIN <= MAX }>: IsTrue,
    {
        pub fn new() -> Self { MultipleInstanceSpecConst { _priv: () } }
        pub fn min(&self) -> usize { MIN }
        pub fn max(&self) -> usize { MAX }
    }

    // MultipleInstanceSpec (runtime)
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum InstanceCreationKind { Static, Dynamic }

    #[derive(Debug, Clone)]
    pub struct MultipleInstanceSpec {
        pub min: usize,
        pub max: Option<usize>,
        pub threshold: Option<usize>,
        pub creation: InstanceCreationKind,
    }
    impl MultipleInstanceSpec {
        pub fn new(min: usize, max: Option<usize>, threshold: Option<usize>, creation: InstanceCreationKind) -> Self {
            MultipleInstanceSpec { min, max, threshold, creation }
        }
        pub fn validate(&self) -> Result<(), PetriRefusal> {
            if self.min == 0 { return Err(PetriRefusal::InvalidInstanceBounds); }
            if let Some(m) = self.max { if self.min > m { return Err(PetriRefusal::InvalidInstanceBounds); } }
            Ok(())
        }
    }

    // InitialFinalMarkingPair
    pub struct InitialFinalMarkingPair { _initial: Marking, _final: Marking }
    impl InitialFinalMarkingPair {
        pub fn new(initial: Marking, fin: Marking) -> Self {
            InitialFinalMarkingPair { _initial: initial, _final: fin }
        }
        pub fn validate(&self) -> Result<(), PetriRefusal> {
            for (k, v) in &self._initial.0 {
                if *v > 0 && self._final.tokens_on(k) > 0 {
                    return Err(PetriRefusal::UnsafeNet);
                }
            }
            Ok(())
        }
    }

    // ObjectCentricPetriNet
    pub struct ObjectCentricPetriNet { _net: PetriNet, _object_types: Vec<String> }
    impl ObjectCentricPetriNet {
        pub fn new(net: PetriNet, object_types: Vec<String>) -> Self {
            ObjectCentricPetriNet { _net: net, _object_types: object_types }
        }
        pub fn validate(&self) -> Result<(), PetriRefusal> {
            for arc in &self._net.arcs {
                if let Some(ref ot) = arc.object_type {
                    if !self._object_types.contains(&ot.0) {
                        return Err(PetriRefusal::ObjectTypeNotPreserved);
                    }
                }
            }
            Ok(())
        }
    }
}

// ── models module (re-export petri with extra types) ─────────────────────────

pub mod models {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PetriNetRefusal { IsolatedPlace, InvalidStructure, EmptyNet }

    // PackedKeyTable for dense_kernel
    #[derive(Debug, Clone, Default)]
    pub struct PackedKeyTable<V> {
        data: HashMap<u64, (String, V)>,
    }
    impl<V: Clone> PackedKeyTable<V> {
        pub fn new() -> Self { PackedKeyTable { data: HashMap::new() } }
        pub fn insert(&mut self, hash: u64, key: String, value: V) {
            self.data.insert(hash, (key, value));
        }
        pub fn get_by_hash(&self, hash: u64) -> Option<&V> {
            self.data.get(&hash).map(|(_, v)| v)
        }
        pub fn get_by_key(&self, key: &str) -> Option<&V> {
            for (_, (k, v)) in &self.data {
                if k == key { return Some(v); }
            }
            None
        }
    }

    // RuntimeMarking
    pub struct RuntimeMarking<'a> { table: &'a PackedKeyTable<usize> }
    impl<'a> RuntimeMarking<'a> {
        pub fn tokens_on(&self, place_id: &str) -> usize {
            let h = crate::dense_kernel::fnv1a_64(place_id.as_bytes());
            self.table.get_by_hash(h).copied().unwrap_or(0)
        }
    }

    // models::PetriNet — uses PackedKeyTable for initial_marking (distinct from petri::PetriNet)
    pub struct PetriNet {
        pub places: Vec<crate::petri::Place>,
        pub transitions: Vec<crate::petri::Transition>,
        pub arcs: Vec<crate::petri::Arc>,
        pub initial_marking: PackedKeyTable<usize>,
        pub final_marking: PackedKeyTable<usize>,
    }
    impl PetriNet {
        pub fn default() -> Self {
            PetriNet { places: Vec::new(), transitions: Vec::new(), arcs: Vec::new(),
                initial_marking: PackedKeyTable::new(), final_marking: PackedKeyTable::new() }
        }
        pub fn initial_marking(&self) -> RuntimeMarking<'_> {
            RuntimeMarking { table: &self.initial_marking }
        }
        pub fn validate(&self) -> Result<(), PetriNetRefusal> {
            if self.places.is_empty() && self.transitions.is_empty() {
                return Err(PetriNetRefusal::EmptyNet);
            }
            Ok(())
        }
    }
}

// ── dense_kernel module ───────────────────────────────────────────────────────

pub mod dense_kernel {
    pub use crate::models::PackedKeyTable;

    pub fn fnv1a_64(bytes: &[u8]) -> u64 {
        let mut h: u64 = 14695981039346656037;
        for &b in bytes { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
        h
    }

    #[derive(Debug, Clone)]
    pub struct DenseKernel { pub size: usize }
    impl DenseKernel {
        pub fn new(size: usize) -> Self { DenseKernel { size } }
        pub fn validate(&self) -> Result<(), KernelRefusal> {
            if self.size == 0 { Err(KernelRefusal::ZeroSize) } else { Ok(()) }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum KernelRefusal { ZeroSize, TooDense }
}

// ── powl module ───────────────────────────────────────────────────────────────

pub mod powl {
    use std::collections::{HashMap, HashSet, VecDeque};
    use crate::{Require, IsTrue};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PowlNodeId(pub u64);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PowlNodeKind {
        Atom(String),
        Silent,
        PartialOrder(Vec<PowlNodeId>),
        Choice(Vec<PowlNodeId>),
        Loop { body: PowlNodeId, redo: Option<PowlNodeId> },
        ChoiceGraph { nodes: Vec<PowlNodeId>, edges: Vec<(usize, usize)> },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PowlNode { pub id: PowlNodeId, pub kind: PowlNodeKind }
    impl PowlNode {
        pub fn new(id: PowlNodeId, kind: PowlNodeKind) -> Self { PowlNode { id, kind } }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OrderEdge { pub from: PowlNodeId, pub to: PowlNodeId }
    impl OrderEdge {
        pub fn new(from: PowlNodeId, to: PowlNodeId) -> Self { OrderEdge { from, to } }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PowlRefusal {
        CyclicPartialOrder,
        InvalidLoop,
        InvalidChoiceArity { declared: usize, required_min: usize },
        ChoiceGraphDisconnected,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Powl {
        pub nodes: Vec<PowlNode>,
        pub edges: Vec<OrderEdge>,
        pub root: Option<PowlNodeId>,
    }

    impl Powl {
        pub fn new() -> Self { Powl::default() }
        pub fn node_count(&self) -> usize { self.nodes.len() }

        pub fn validate(&self) -> Result<(), PowlRefusal> {
            let node_ids: HashSet<u64> = self.nodes.iter().map(|n| n.id.0).collect();

            // Check loops reference existing nodes
            for n in &self.nodes {
                if let PowlNodeKind::Loop { body, redo } = &n.kind {
                    if !node_ids.contains(&body.0) { return Err(PowlRefusal::InvalidLoop); }
                    if let Some(r) = redo {
                        if !node_ids.contains(&r.0) { return Err(PowlRefusal::InvalidLoop); }
                    }
                }
                if let PowlNodeKind::ChoiceGraph { nodes, .. } = &n.kind {
                    if nodes.len() < 2 { return Err(PowlRefusal::ChoiceGraphDisconnected); }
                }
            }

            // Check partial-order edges for cycles
            if !self.edges.is_empty() {
                let mut in_degree: HashMap<u64, usize> = node_ids.iter().map(|&id| (id, 0)).collect();
                let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();
                for e in &self.edges {
                    *in_degree.entry(e.to.0).or_insert(0) += 1;
                    adj.entry(e.from.0).or_default().push(e.to.0);
                }
                let mut queue: VecDeque<u64> = in_degree.iter()
                    .filter(|(_, &d)| d == 0).map(|(&id, _)| id).collect();
                let mut visited = 0usize;
                while let Some(n) = queue.pop_front() {
                    visited += 1;
                    for &next in adj.get(&n).into_iter().flatten() {
                        let d = in_degree.entry(next).or_insert(0);
                        *d -= 1;
                        if *d == 0 { queue.push_back(next); }
                    }
                }
                if visited != node_ids.len() { return Err(PowlRefusal::CyclicPartialOrder); }
            }
            Ok(())
        }
    }

    // PowlChoiceNode
    #[derive(Debug, Clone)]
    pub struct PowlChoiceNode { branches: Vec<PowlNodeId> }
    impl PowlChoiceNode {
        pub fn new(branches: Vec<PowlNodeId>) -> Self { PowlChoiceNode { branches } }
        pub fn validate(&self) -> Result<(), PowlRefusal> {
            if self.branches.len() < 2 {
                Err(PowlRefusal::InvalidChoiceArity { declared: self.branches.len(), required_min: 2 })
            } else { Ok(()) }
        }
    }

    // ChoiceGraph + StandaloneChoiceGraphNode
    #[derive(Debug, Clone)]
    pub enum StandaloneChoiceGraphNode {
        Start,
        End,
        Activity(String),
        SubModel(usize),
    }

    #[derive(Debug, Clone)]
    pub struct ChoiceGraph {
        nodes: Vec<StandaloneChoiceGraphNode>,
        edges: Vec<(usize, usize)>,
    }
    impl ChoiceGraph {
        pub fn new(nodes: Vec<StandaloneChoiceGraphNode>, edges: Vec<(usize, usize)>) -> Self {
            ChoiceGraph { nodes, edges }
        }
        pub fn successors(&self, idx: usize) -> Vec<usize> {
            self.edges.iter().filter(|(f, _)| *f == idx).map(|(_, t)| *t).collect()
        }
        pub fn predecessors(&self, idx: usize) -> Vec<usize> {
            self.edges.iter().filter(|(_, t)| *t == idx).map(|(f, _)| *f).collect()
        }
    }

    // TypedPowlLoopNode
    pub struct TypedPowlLoopNode<Children, const ARITY: usize>
    where
        Require<{ ARITY == 2 }>: IsTrue,
    {
        pub children: Children,
    }
    impl<Children, const ARITY: usize> TypedPowlLoopNode<Children, ARITY>
    where
        Require<{ ARITY == 2 }>: IsTrue,
    {
        pub fn new(children: Children) -> Self { TypedPowlLoopNode { children } }
    }

    // PowlComposition
    pub const MAX_POWL_DEPTH: usize = 8;

    pub struct PowlComposition<Inner, const DEPTH: usize>
    where
        Require<{ DEPTH <= MAX_POWL_DEPTH }>: IsTrue,
    {
        pub inner: Inner,
    }
    impl<Inner, const DEPTH: usize> PowlComposition<Inner, DEPTH>
    where
        Require<{ DEPTH <= MAX_POWL_DEPTH }>: IsTrue,
    {
        pub fn new(inner: Inner) -> Self { PowlComposition { inner } }
    }
}

// ── powl8_op module ───────────────────────────────────────────────────────────

pub mod powl8_op {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Powl8Op {
        NoOp = 0, Sequence = 1, Choice = 2, Parallel = 3,
        PartialOrder = 4, Loop = 5, Silent = 6, Or = 7, ChoiceGraph = 8,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Powl8OpError { InvalidDiscriminant }

    impl TryFrom<u8> for Powl8Op {
        type Error = Powl8OpError;
        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Powl8Op::NoOp), 1 => Ok(Powl8Op::Sequence), 2 => Ok(Powl8Op::Choice),
                3 => Ok(Powl8Op::Parallel), 4 => Ok(Powl8Op::PartialOrder), 5 => Ok(Powl8Op::Loop),
                6 => Ok(Powl8Op::Silent), 7 => Ok(Powl8Op::Or), 8 => Ok(Powl8Op::ChoiceGraph),
                _ => Err(Powl8OpError::InvalidDiscriminant),
            }
        }
    }
}

// ── bpmn module ───────────────────────────────────────────────────────────────

pub mod bpmn {
    use std::collections::HashSet;

    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BpmnEvent { Start, Intermediate, End, Boundary }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnTask { pub label: String }
    impl BpmnTask {
        pub fn new(label: &str) -> Self { BpmnTask { label: label.to_string() } }
        pub fn name(&self) -> &str { &self.label }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BpmnGateway { Exclusive, Parallel, Inclusive, EventBased, Complex }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnNodeKind {
        Event(BpmnEvent),
        Task(BpmnTask),
        Gateway(BpmnGateway),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnNode { id: String, kind: BpmnNodeKind }
    impl BpmnNode {
        pub fn event(id: &str, e: BpmnEvent) -> Self { BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Event(e) } }
        pub fn task(id: &str, t: BpmnTask) -> Self { BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Task(t) } }
        pub fn gateway(id: &str, g: BpmnGateway) -> Self { BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Gateway(g) } }
        pub fn id(&self) -> &str { &self.id }
        pub fn kind(&self) -> &BpmnNodeKind { &self.kind }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnEdge { source: String, target: String }
    impl BpmnEdge {
        pub fn new(source: &str, target: &str) -> Self { BpmnEdge { source: source.to_string(), target: target.to_string() } }
        pub fn source(&self) -> &str { &self.source }
        pub fn target(&self) -> &str { &self.target }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnRefusal {
        DanglingEdge, MissingStart, MissingStartEvent, MissingEndEvent,
        EmptyProcess, DuplicateNodeId, LaneNodeNotDeclared,
    }

    #[derive(Debug, Clone)]
    pub struct BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> }
    impl BpmnProcess {
        pub fn new(nodes: impl IntoIterator<Item = BpmnNode>, edges: impl IntoIterator<Item = BpmnEdge>) -> Self {
            BpmnProcess { nodes: nodes.into_iter().collect(), edges: edges.into_iter().collect() }
        }
        pub fn nodes(&self) -> &[BpmnNode] { &self.nodes }
        pub fn edges(&self) -> &[BpmnEdge] { &self.edges }

        pub fn validate(&self) -> Result<(), BpmnRefusal> {
            if self.nodes.is_empty() { return Err(BpmnRefusal::EmptyProcess); }
            // Check duplicate ids
            let mut seen = HashSet::new();
            for n in &self.nodes {
                if !seen.insert(n.id()) { return Err(BpmnRefusal::DuplicateNodeId); }
            }
            // Check start/end events
            let has_start = self.nodes.iter().any(|n| matches!(n.kind(), BpmnNodeKind::Event(BpmnEvent::Start)));
            let has_end = self.nodes.iter().any(|n| matches!(n.kind(), BpmnNodeKind::Event(BpmnEvent::End)));
            if !has_start { return Err(BpmnRefusal::MissingStartEvent); }
            if !has_end { return Err(BpmnRefusal::MissingEndEvent); }
            // Check dangling edges
            let node_ids: HashSet<&str> = self.nodes.iter().map(|n| n.id()).collect();
            for e in &self.edges {
                if !node_ids.contains(e.source()) || !node_ids.contains(e.target()) {
                    return Err(BpmnRefusal::DanglingEdge);
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct BpmnLane { id: String, name: String, node_ids: Vec<String> }
    impl BpmnLane {
        pub fn new(id: &str, name: &str, members: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
            BpmnLane { id: id.to_string(), name: name.to_string(),
                node_ids: members.into_iter().map(|s| s.as_ref().to_string()).collect() }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn name(&self) -> &str { &self.name }
        pub fn node_ids(&self) -> &[String] { &self.node_ids }
        pub fn validate(&self, known: &HashSet<&str>) -> Result<(), BpmnRefusal> {
            for id in &self.node_ids {
                if !known.contains(id.as_str()) { return Err(BpmnRefusal::LaneNodeNotDeclared); }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct BpmnPool { id: String, name: String, process: BpmnProcess, lanes: Vec<BpmnLane> }
    impl BpmnPool {
        pub fn new(id: &str, name: &str, process: BpmnProcess, lanes: impl IntoIterator<Item = BpmnLane>) -> Self {
            BpmnPool { id: id.to_string(), name: name.to_string(), process, lanes: lanes.into_iter().collect() }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn name(&self) -> &str { &self.name }
        pub fn process(&self) -> &BpmnProcess { &self.process }
        pub fn lanes(&self) -> &[BpmnLane] { &self.lanes }
    }
}

// ── declare module ────────────────────────────────────────────────────────────

pub mod declare {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Activity(pub String);
    impl Activity {
        pub fn new(name: &str) -> Self { Activity(name.to_string()) }
        pub fn name(&self) -> &str { &self.0 }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum DeclareTemplate {
        Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3,
        RespondedExistence, CoExistence, Response, Precedence, Succession,
        AlternateResponse, AlternatePrecedence, AlternateSuccession,
        ChainResponse, ChainPrecedence, ChainSuccession,
        NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice,
    }
    impl DeclareTemplate {
        pub fn arity(&self) -> u8 {
            match self {
                DeclareTemplate::Existence | DeclareTemplate::Absence | DeclareTemplate::Init |
                DeclareTemplate::Existence2 | DeclareTemplate::Existence3 |
                DeclareTemplate::Absence2 | DeclareTemplate::Absence3 => 1,
                _ => 2,
            }
        }
        pub fn is_negative(&self) -> bool {
            matches!(self,
                DeclareTemplate::Absence | DeclareTemplate::Absence2 | DeclareTemplate::Absence3 |
                DeclareTemplate::NotSuccession | DeclareTemplate::NotChainSuccession | DeclareTemplate::NotCoExistence
            )
        }
        pub fn is_chain(&self) -> bool {
            matches!(self, DeclareTemplate::ChainResponse | DeclareTemplate::ChainPrecedence | DeclareTemplate::ChainSuccession | DeclareTemplate::NotChainSuccession)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DeclareScope {
        SingleObjectScope(String),
        MultiObjectScope(Vec<String>),
        SynchronizedObjectScope(Vec<String>),
        CrossObjectScope(String, String),
        GlobalScope,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclareConstraint {
        pub template: DeclareTemplate,
        pub activation: Activity,
        pub target: Option<Activity>,
        pub scope: DeclareScope,
    }
    impl DeclareConstraint {
        pub fn unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) -> Self {
            DeclareConstraint { template, activation, target: None, scope }
        }
        pub fn binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) -> Self {
            DeclareConstraint { template, activation, target: Some(target), scope }
        }
        pub fn validate(&self) -> Result<(), DeclareRefusal> {
            if self.activation.0.is_empty() { return Err(DeclareRefusal::MissingActivation); }
            match &self.scope {
                DeclareScope::MultiObjectScope(v) if v.is_empty() => return Err(DeclareRefusal::EmptyObjectScope),
                DeclareScope::SynchronizedObjectScope(v) if v.len() < 2 => return Err(DeclareRefusal::SynchronizationViolation),
                _ => {}
            }
            let need_target = self.template.arity() == 2;
            if need_target && self.target.is_none() { return Err(DeclareRefusal::MissingTarget); }
            if !need_target && self.target.is_some() { return Err(DeclareRefusal::InvalidTemplateArity); }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DeclareRefusal {
        EmptyActivity, MissingActivation, BinaryRequiresTarget, UnsupportedTemplate,
        MissingTarget, InvalidTemplateArity, EmptyObjectScope, SynchronizationViolation,
    }

    // OcDeclareConstraint
    #[derive(Debug, Clone)]
    pub struct OcDeclareConstraint {
        pub constraint: DeclareConstraint,
        pub object_types: Vec<String>,
        _synchronized: bool,
    }
    impl OcDeclareConstraint {
        pub fn new(constraint: DeclareConstraint, object_types: Vec<String>) -> Self {
            OcDeclareConstraint { constraint, object_types, _synchronized: false }
        }
        pub fn synchronized(constraint: DeclareConstraint, object_types: Vec<String>) -> Self {
            OcDeclareConstraint { constraint, object_types, _synchronized: true }
        }
        pub fn is_synchronized(&self) -> bool { self._synchronized }
        pub fn validate(&self) -> Result<(), OcDeclareRefusal> {
            if self.object_types.is_empty() { return Err(OcDeclareRefusal::EmptyObjectTypeList); }
            if self._synchronized && self.object_types.len() < 2 {
                return Err(OcDeclareRefusal::SynchronizationRequiresMultipleTypes);
            }
            // ScopeMismatch: not synchronized but scope is SynchronizedObjectScope
            if !self._synchronized {
                if let DeclareScope::SynchronizedObjectScope(_) = &self.constraint.scope {
                    return Err(OcDeclareRefusal::ScopeMismatch);
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum OcDeclareRefusal {
        EmptyObjectTypeList,
        SynchronizationRequiresMultipleTypes,
        ScopeMismatch,
    }
}

// ── dfg module ────────────────────────────────────────────────────────────────

pub mod dfg {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DfgNode(pub String);
    impl DfgNode {
        pub fn new(label: &str) -> Self { DfgNode(label.to_string()) }
        pub fn label(&self) -> &str { &self.0 }
        pub fn activity(&self) -> &str { &self.0 }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DfgWeight(pub u64);
    impl DfgWeight {
        pub fn count(&self) -> usize { self.0 as usize }
    }

    #[derive(Debug, Clone)]
    pub struct DfgEdge { source: String, target: String, count: u64 }
    impl DfgEdge {
        pub fn new(source: &str, target: &str, count: u64) -> Self {
            DfgEdge { source: source.to_string(), target: target.to_string(), count }
        }
        pub fn source(&self) -> &str { &self.source }
        pub fn target(&self) -> &str { &self.target }
        pub fn weight(&self) -> DfgWeight { DfgWeight(self.count) }
    }

    #[derive(Debug, Clone)]
    pub struct DfgEdgeFull { pub source: String, pub target: String, _freq: u64, _dur: Option<u64> }
    impl DfgEdgeFull {
        pub fn new(source: &str, target: &str, freq: u64) -> Self {
            DfgEdgeFull { source: source.to_string(), target: target.to_string(), _freq: freq, _dur: None }
        }
        pub fn with_duration_ns(mut self, d: u64) -> Self { self._dur = Some(d); self }
        pub fn source(&self) -> &str { &self.source }
        pub fn target(&self) -> &str { &self.target }
        pub fn frequency(&self) -> DfgWeight { DfgWeight(self._freq) }
        pub fn duration_ns(&self) -> Option<u64> { self._dur }
    }

    #[derive(Debug, Clone)]
    pub struct Dfg { nodes: Vec<DfgNode>, edges: Vec<DfgEdge> }
    impl Dfg {
        pub fn new(nodes: Vec<DfgNode>, edges: Vec<DfgEdge>) -> Self { Dfg { nodes, edges } }
        pub fn nodes(&self) -> &[DfgNode] { &self.nodes }
        pub fn edges(&self) -> &[DfgEdge] { &self.edges }
        pub fn validate(&self) -> Result<(), DfgRefusal> {
            if self.nodes.is_empty() { return Err(DfgRefusal::EmptyGraph); }
            let node_labels: std::collections::HashSet<&str> = self.nodes.iter().map(|n| n.label()).collect();
            for e in &self.edges {
                if !node_labels.contains(e.source()) || !node_labels.contains(e.target()) {
                    return Err(DfgRefusal::DanglingEdge);
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone)]
    pub struct ObjectCentricDfg(HashMap<String, Dfg>);
    impl ObjectCentricDfg {
        pub fn new() -> Self { ObjectCentricDfg(HashMap::new()) }
        pub fn insert(&mut self, obj_type: &str, dfg: Dfg) { self.0.insert(obj_type.to_string(), dfg); }
        pub fn get(&self, obj_type: &str) -> Option<&Dfg> { self.0.get(obj_type) }
        pub fn with_type_dfg(mut self, obj_type: &str, dfg: Dfg) -> Self { self.insert(obj_type, dfg); self }
        pub fn object_types(&self) -> impl Iterator<Item = &str> { self.0.keys().map(|s| s.as_str()) }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum DfgRefusal { EmptyGraph, DanglingEdge, IsolatedNode }
}

// ── eventlog module ───────────────────────────────────────────────────────────

pub mod eventlog {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub struct Event {
        pub activity: String,
        pub attributes: HashMap<String, String>,
        _ts: Option<u64>,
        _resource: Option<String>,
        _lifecycle: Option<String>,
    }

    impl Event {
        pub fn new(activity: &str) -> Self {
            Event { activity: activity.to_string(), attributes: HashMap::new(),
                _ts: None, _resource: None, _lifecycle: None }
        }
        pub fn at_ns(mut self, ts: u64) -> Self { self._ts = Some(ts); self }
        pub fn by(mut self, resource: &str) -> Self { self._resource = Some(resource.to_string()); self }
        pub fn with_lifecycle(mut self, lc: &str) -> Self { self._lifecycle = Some(lc.to_string()); self }
        pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
            self.attributes.insert(key.to_string(), value.to_string()); self
        }
        pub fn activity(&self) -> &str { &self.activity }
        pub fn timestamp_ns(&self) -> Option<u64> { self._ts }
        pub fn resource(&self) -> Option<&str> { self._resource.as_deref() }
        pub fn lifecycle(&self) -> Option<&str> { self._lifecycle.as_deref() }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Trace {
        pub events: Vec<Event>,
        pub case_id: String,
    }

    impl Trace {
        pub fn new(case_id: &str, events: impl IntoIterator<Item = Event>) -> Self {
            Trace { case_id: case_id.to_string(), events: events.into_iter().collect() }
        }
        pub fn from_events(events: impl IntoIterator<Item = Event>) -> Self {
            Trace { case_id: String::new(), events: events.into_iter().collect() }
        }
        pub fn push(&mut self, event: Event) { self.events.push(event); }
        pub fn events(&self) -> &[Event] { &self.events }
        pub fn case_id(&self) -> &str { &self.case_id }
        pub fn len(&self) -> usize { self.events.len() }
        pub fn is_empty(&self) -> bool { self.events.is_empty() }

        pub fn validate(&self) -> Result<(), EventLogRefusal> {
            if self.events.is_empty() { return Err(EventLogRefusal::EmptyTrace); }
            let mut prev_ts: Option<u64> = None;
            for e in &self.events {
                if let Some(ts) = e.timestamp_ns() {
                    if let Some(p) = prev_ts {
                        if ts < p { return Err(EventLogRefusal::NonMonotonicTrace); }
                    }
                    prev_ts = Some(ts);
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct EventLog { pub traces: Vec<Trace> }

    impl EventLog {
        pub fn new(traces: Vec<Trace>) -> Self { EventLog { traces } }
        pub fn from_traces(traces: impl IntoIterator<Item = Trace>) -> Self {
            EventLog { traces: traces.into_iter().collect() }
        }
        pub fn traces(&self) -> &[Trace] { &self.traces }
        pub fn trace_count(&self) -> usize { self.traces.len() }
        pub fn event_count(&self) -> usize { self.traces.iter().map(|t| t.len()).sum() }
        pub fn len(&self) -> usize { self.traces.len() }

        pub fn validate(&self) -> Result<(), EventLogRefusal> {
            if self.traces.is_empty() { return Err(EventLogRefusal::EmptyLog); }
            for t in &self.traces { t.validate()?; }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum EventLogRefusal { EmptyLog, EmptyTrace, MissingActivity, NonMonotonicTrace }

    #[derive(Debug, Clone)]
    pub struct EventStream {
        events: Vec<Event>,
    }
    impl EventStream {
        pub fn new() -> Self { EventStream { events: Vec::new() } }
        pub fn push(&mut self, event: Event) { self.events.push(event); }
        pub fn len(&self) -> usize { self.events.len() }
        pub fn is_empty(&self) -> bool { self.events.is_empty() }
        pub fn events(&self) -> &[Event] { &self.events }
    }
}

// ── process_tree module ───────────────────────────────────────────────────────

pub mod process_tree {
    use crate::{Require, IsTrue};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ProcessTreeNodeId(pub usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessTreeOperator { Sequence, Xor, Parallel, Or, Loop, Silent }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ProcessTreeNode {
        Activity(String),
        Operator { operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId> },
    }

    impl ProcessTreeNode {
        pub fn leaf(id: ProcessTreeNodeId, label: &str) -> Self { ProcessTreeNode::Activity(label.to_string()) }
        pub fn internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) -> Self {
            ProcessTreeNode::Operator { operator, children }
        }
        pub fn id(&self) -> ProcessTreeNodeId { ProcessTreeNodeId(0) }
    }

    #[derive(Debug, Clone)]
    pub struct ProcessTree {
        pub nodes: Vec<ProcessTreeNode>,
        pub root: Option<ProcessTreeNodeId>,
    }

    impl ProcessTree {
        pub fn new() -> Self { ProcessTree { nodes: Vec::new(), root: None } }
        pub fn node_count(&self) -> usize { self.nodes.len() }
        pub fn root(&self) -> Option<ProcessTreeNodeId> { self.root }

        pub fn admit_shape(&self) -> Result<(), ProcessTreeRefusal> {
            if self.nodes.is_empty() { return Ok(()); }
            let root = match self.root { Some(r) => r, None => return Err(ProcessTreeRefusal::MissingRoot) };
            let n = self.nodes.len();
            // Check all children are in bounds
            for node in &self.nodes {
                if let ProcessTreeNode::Operator { operator, children } = node {
                    for c in children {
                        if c.0 >= n { return Err(ProcessTreeRefusal::DanglingNodeReference); }
                    }
                    match operator {
                        ProcessTreeOperator::Silent => {
                            if !children.is_empty() { return Err(ProcessTreeRefusal::TauLeafWithChildren); }
                        }
                        ProcessTreeOperator::Loop => {
                            if children.len() != 2 { return Err(ProcessTreeRefusal::InvalidArity); }
                        }
                        ProcessTreeOperator::Sequence | ProcessTreeOperator::Xor |
                        ProcessTreeOperator::Parallel | ProcessTreeOperator::Or => {
                            if children.len() < 2 { return Err(ProcessTreeRefusal::BelowMinimumArity); }
                        }
                    }
                }
            }
            // Cycle detection via DFS from root
            let mut visited = vec![false; n];
            let mut in_stack = vec![false; n];
            if self.has_cycle(root.0, &mut visited, &mut in_stack) {
                return Err(ProcessTreeRefusal::CycleDetected);
            }
            Ok(())
        }

        fn has_cycle(&self, idx: usize, visited: &mut Vec<bool>, in_stack: &mut Vec<bool>) -> bool {
            if in_stack[idx] { return true; }
            if visited[idx] { return false; }
            visited[idx] = true;
            in_stack[idx] = true;
            if let ProcessTreeNode::Operator { children, .. } = &self.nodes[idx] {
                for c in children {
                    if self.has_cycle(c.0, visited, in_stack) { return true; }
                }
            }
            in_stack[idx] = false;
            false
        }

        pub fn validate(&self) -> Result<(), ProcessTreeRefusal> { self.admit_shape() }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ProcessTreeRefusal {
        EmptyTree, OrphanNode, InvalidLoopArity, UnsupportedOperator,
        MissingRoot, DanglingNodeReference, TauLeafWithChildren,
        BelowMinimumArity, InvalidArity, CycleDetected,
    }

    // Typed operator nodes
    pub struct TypedLoopNode<Children, const ARITY: usize>
    where
        Require<{ ARITY == 2 }>: IsTrue,
    {
        pub children: Children,
    }
    impl<Children, const ARITY: usize> TypedLoopNode<Children, ARITY>
    where
        Require<{ ARITY == 2 }>: IsTrue,
    {
        pub fn new(children: Children) -> Self { TypedLoopNode { children } }
    }

    macro_rules! typed_op_node {
        ($name:ident) => {
            pub struct $name<Children, const ARITY: usize>
            where
                Require<{ ARITY >= 2 }>: IsTrue,
            {
                pub children: Children,
            }
            impl<Children, const ARITY: usize> $name<Children, ARITY>
            where
                Require<{ ARITY >= 2 }>: IsTrue,
            {
                pub fn new(children: Children) -> Self { $name { children } }
            }
        };
    }

    typed_op_node!(TypedXorNode);
    typed_op_node!(TypedAndNode);
    typed_op_node!(TypedSeqNode);
    typed_op_node!(TypedOrNode);
}

// ── receipt module ────────────────────────────────────────────────────────────

pub mod receipt {
    use crate::law::ProcessShapeKind;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Digest(pub String);
    impl Digest {
        pub fn new(hex: &str) -> Self { Digest(hex.to_string()) }
        pub fn as_str(&self) -> &str { &self.0 }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ReplayHint(pub String);
    impl ReplayHint {
        pub fn new(hint: &str) -> Self { ReplayHint(hint.to_string()) }
        pub fn as_str(&self) -> &str { &self.0 }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ReceiptRefusal {
        EmptyChain, HashMismatch, InvalidDigest,
        MissingSubject, MissingWitness, MissingDigest, MissingReplayHint,
        BrokenChainLink(usize),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ReceiptEnvelope {
        pub subject: String,
        pub witness: String,
        pub digest: Digest,
        pub replay_hint: ReplayHint,
    }

    impl ReceiptEnvelope {
        pub fn new(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Self {
            ReceiptEnvelope { subject: subject.to_string(), witness: witness.to_string(), digest, replay_hint }
        }
        pub fn try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Result<Self, ReceiptRefusal> {
            if subject.is_empty() { return Err(ReceiptRefusal::MissingSubject); }
            if witness.is_empty() { return Err(ReceiptRefusal::MissingWitness); }
            if digest.0.is_empty() { return Err(ReceiptRefusal::MissingDigest); }
            if replay_hint.0.is_empty() { return Err(ReceiptRefusal::MissingReplayHint); }
            Ok(ReceiptEnvelope { subject: subject.to_string(), witness: witness.to_string(), digest, replay_hint })
        }
        pub fn is_well_shaped(&self) -> bool {
            !self.subject.is_empty() && !self.witness.is_empty()
                && !self.digest.0.is_empty() && !self.replay_hint.0.is_empty()
        }
        pub fn verify(&self) -> Result<(), ReceiptRefusal> { Ok(()) }
    }

    #[derive(Debug, Clone)]
    pub struct ReceiptChain { pub run_id: String, pub envelopes: Vec<ReceiptEnvelope> }
    impl ReceiptChain {
        pub fn try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) -> Result<Self, ReceiptRefusal> {
            if envelopes.is_empty() { return Err(ReceiptRefusal::EmptyChain); }
            Ok(ReceiptChain { run_id: run_id.to_string(), envelopes })
        }
        pub fn len(&self) -> usize { self.envelopes.len() }
        pub fn is_empty(&self) -> bool { self.envelopes.is_empty() }
        pub fn extend_with(&mut self, env: ReceiptEnvelope) -> Result<(), ReceiptRefusal> {
            self.envelopes.push(env); Ok(())
        }
    }

    // ReceiptChainConst — const-generic fixed-size chain
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ReceiptChainConst<const N: usize> {
        pub run_id: String,
        pub links: Vec<ReceiptEnvelope>,
    }
    impl<const N: usize> ReceiptChainConst<N> {
        pub fn try_new(run_id: &str, links: [ReceiptEnvelope; N]) -> Result<Self, ReceiptRefusal> {
            if N == 0 { return Err(ReceiptRefusal::EmptyChain); }
            let v: Vec<ReceiptEnvelope> = links.into_iter().collect();
            for (i, env) in v.iter().enumerate() {
                if !env.is_well_shaped() { return Err(ReceiptRefusal::BrokenChainLink(i)); }
            }
            Ok(ReceiptChainConst { run_id: run_id.to_string(), links: v })
        }
        pub fn arity(&self) -> usize { N }
        pub fn root(&self) -> &ReceiptEnvelope { &self.links[0] }
        pub fn tip(&self) -> &ReceiptEnvelope { &self.links[self.links.len() - 1] }
        pub fn iter(&self) -> impl Iterator<Item = &ReceiptEnvelope> { self.links.iter() }
    }

    pub enum ReplayHintKind { FromStart, FromSeq(u64), Latest }

    // ConformanceVerdict (for receipt::ConformanceVerdict enum)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConformanceVerdict { PerfectAlignment, FitnessDeficit, DeadlockEncountered }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ReceiptVerdict {
        Admitted,
        Refused(ReceiptRefusal),
    }
}

// ── interop module ────────────────────────────────────────────────────────────

pub mod formats {
    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FormatKind {
        OcelJson, OcelXml, OcelSqlite, XesXml, BpmnXml, PetriPnml, PowlJson,
    }
}

pub mod interop {
    use std::marker::PhantomData;

    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SummaryShape {
        Counts, TraceVariants, ActivityDistribution, TimingProfile, ObjectTypeDistribution,
    }

    #[non_exhaustive]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FilterShape { Activity, Timeframe, Variant, Attribute, ObjectType }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Pm4pyShape {
        EventLog, ObjectCentricLog, PetriNet, ProcessTree, Bpmn, DirectlyFollowsGraph, Declare,
    }
    impl Pm4pyShape {
        pub fn is_object_centric(&self) -> bool { matches!(self, Pm4pyShape::ObjectCentricLog) }
        pub fn tag(&self) -> &'static str {
            match self {
                Pm4pyShape::EventLog => "event-log",
                Pm4pyShape::ObjectCentricLog => "ocel",
                Pm4pyShape::PetriNet => "petri-net",
                Pm4pyShape::ProcessTree => "process-tree",
                Pm4pyShape::Bpmn => "bpmn",
                Pm4pyShape::DirectlyFollowsGraph => "dfg",
                Pm4pyShape::Declare => "declare",
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum InteropRefusal {
        UnsupportedShape, MissingGrounding, SchemaConflict,
        UngroundedArtifact, FlatClaimOverObjectCentric, DimensionShapeMismatch,
    }

    pub struct ArtifactGrounding<E> {
        pub shape: Pm4pyShape,
        pub evidence_ref: String,
        _evidence: PhantomData<E>,
    }
    impl<E> ArtifactGrounding<E> {
        pub fn new(shape: Pm4pyShape, evidence_ref: &str) -> Self {
            ArtifactGrounding { shape, evidence_ref: evidence_ref.to_string(), _evidence: PhantomData }
        }
        pub fn is_grounded(&self) -> bool { !self.evidence_ref.is_empty() }
        pub fn admit_flat(&self) -> Result<(), InteropRefusal> {
            if !self.is_grounded() { return Err(InteropRefusal::UngroundedArtifact); }
            if self.shape.is_object_centric() { return Err(InteropRefusal::FlatClaimOverObjectCentric); }
            Ok(())
        }
    }

    pub fn check_filter_shape(shape: Pm4pyShape, filter: FilterShape) -> Result<(), InteropRefusal> {
        if matches!(filter, FilterShape::ObjectType) && !shape.is_object_centric() {
            return Err(InteropRefusal::DimensionShapeMismatch);
        }
        Ok(())
    }
}

// ── causal_net module ─────────────────────────────────────────────────────────

pub mod causal_net {
    use std::collections::HashSet;

    #[derive(Debug, Clone)]
    pub struct CausalBinding { pub source_tasks: Vec<String>, pub target_tasks: Vec<String> }

    #[derive(Debug, Clone)]
    pub struct InputBinding<'a>(pub &'a str, pub &'a str);

    #[derive(Debug, Clone)]
    pub struct OutputBinding<'a>(pub &'a str, pub &'a str);

    #[derive(Debug, Clone)]
    pub struct DependencyMeasure(pub f64);

    #[derive(Debug, Clone)]
    pub struct CausalNet {
        pub nodes: Vec<String>,
        pub dependency_measures: Vec<(String, String, f64)>,
        pub inputs: Vec<CausalBinding>,
        pub outputs: Vec<CausalBinding>,
    }

    impl CausalNet {
        pub fn validate(&self) -> Result<(), CausalNetRefusal> {
            for n in &self.nodes {
                if n.is_empty() { return Err(CausalNetRefusal::MissingActivity); }
            }
            for (_, _, score) in &self.dependency_measures {
                if !score.is_finite() || *score < 0.0 || *score > 1.0 {
                    return Err(CausalNetRefusal::InvalidDependencyScore);
                }
            }
            if self.nodes.len() > 1 {
                let mut connected: HashSet<&str> = HashSet::new();
                for (s, t, _) in &self.dependency_measures {
                    connected.insert(s.as_str());
                    connected.insert(t.as_str());
                }
                for n in &self.nodes {
                    if !connected.contains(n.as_str()) {
                        return Err(CausalNetRefusal::DisconnectedGraph);
                    }
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum CausalNetRefusal { MissingActivity, InvalidDependencyScore, DisconnectedGraph }
}

// ── causality module ──────────────────────────────────────────────────────────

pub mod causality {
    use std::marker::PhantomData;

    pub struct CausalLink<From, To>(PhantomData<(From, To)>);
    impl<From, To> CausalLink<From, To> {
        pub fn new() -> Self { CausalLink(PhantomData) }
    }

    pub struct CausalChain<const LENGTH: usize>;
    impl<const LENGTH: usize> CausalChain<LENGTH> {
        pub fn new() -> Self { CausalChain }
        pub fn length(&self) -> usize { LENGTH }
    }

    pub struct CausallyOrderedEvidence<T> { pub inner: T }
    impl<T> CausallyOrderedEvidence<T> {
        pub fn new(inner: T) -> Self { CausallyOrderedEvidence { inner } }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CausalConsistency { Unknown, Consistent, HasCycles, HasContradictions }

    pub struct ConsistencyVerified<T> {
        pub inner: T,
        verdict: CausalConsistency,
    }
    impl<T> ConsistencyVerified<T> {
        pub(crate) fn new(inner: T, verdict: CausalConsistency) -> Self {
            ConsistencyVerified { inner, verdict }
        }
        pub fn verdict(&self) -> CausalConsistency { self.verdict }
        pub fn is_consistent(&self) -> bool { matches!(self.verdict, CausalConsistency::Consistent) }
    }

    pub trait VerifyCausalConsistency<T> {
        fn verify(&self, inner: T) -> ConsistencyVerified<T>;
    }

    pub struct UnknownVerifier;
    impl<T> VerifyCausalConsistency<T> for UnknownVerifier {
        fn verify(&self, inner: T) -> ConsistencyVerified<T> {
            ConsistencyVerified::new(inner, CausalConsistency::Unknown)
        }
    }
}

// ── correlation module ────────────────────────────────────────────────────────

pub mod correlation {
    use std::marker::PhantomData;

    pub struct CorrelationKey<const SCHEMA: &'static str>;
    impl<const SCHEMA: &'static str> CorrelationKey<SCHEMA> {
        pub fn new() -> Self { CorrelationKey }
        pub fn schema(&self) -> &'static str { SCHEMA }
    }

    pub struct CorrelatedLog<A, B, const SCHEMA: &'static str>(PhantomData<(A, B)>);
    impl<A, B, const SCHEMA: &'static str> CorrelatedLog<A, B, SCHEMA> {
        pub fn new() -> Self { CorrelatedLog(PhantomData) }
        pub fn schema(&self) -> &'static str { SCHEMA }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CorrelationSchema { ByCase, ByObject, ByTimestamp, ByAttribute }
}

// ── process_cube module ───────────────────────────────────────────────────────

pub mod process_cube {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CubeDimensionKind {
        Activity, Resource, Time, DataAttribute, ObjectType, CaseAttribute,
    }

    #[derive(Debug, Clone)]
    pub struct ProcessCube { pub dimensions: Vec<CubeDimensionKind> }
    impl ProcessCube {
        pub fn new(dimensions: Vec<CubeDimensionKind>) -> Self { ProcessCube { dimensions } }
    }

    #[derive(Debug, Clone)]
    pub struct ProcessSlice { pub dimension: CubeDimensionKind, pub value: String }
}

// ── prediction module ─────────────────────────────────────────────────────────

pub mod prediction {
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PredictionTarget {
        NextActivity, OutcomeLabel, RemainingTime, DriftSignal, Risk, ComplianceConstraint,
    }
    impl fmt::Display for PredictionTarget {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                PredictionTarget::NextActivity => write!(f, "next-activity"),
                PredictionTarget::OutcomeLabel => write!(f, "outcome-label"),
                PredictionTarget::RemainingTime => write!(f, "remaining-time"),
                PredictionTarget::DriftSignal => write!(f, "drift-signal"),
                PredictionTarget::Risk => write!(f, "risk"),
                PredictionTarget::ComplianceConstraint => write!(f, "compliance-constraint"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PredictionHorizon { FullCase, Events(usize), TimeUnits(u64) }
    impl fmt::Display for PredictionHorizon {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                PredictionHorizon::FullCase => write!(f, "full-case"),
                PredictionHorizon::Events(n) => write!(f, "events({n})"),
                PredictionHorizon::TimeUnits(s) => write!(f, "time({s}s)"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ComplianceKind { Monitoring, Audit, Certification }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PredictionRefusal {
        InsufficientData, InvalidModel, ConvergenceFailure,
        MissingPrefix, MissingTarget, EmptyPrefix, TargetUnsupported,
        NonPrefixTrace, ConstraintNotNamed,
    }
}

// ── ids module ────────────────────────────────────────────────────────────────

pub mod ids {
    use std::marker::PhantomData;
    use std::borrow::Cow;

    pub struct ObjectTypeName<K> { inner: Cow<'static, str>, _kind: PhantomData<K> }
    impl<K> ObjectTypeName<K> {
        pub fn from_static(s: &'static str) -> Self { ObjectTypeName { inner: Cow::Borrowed(s), _kind: PhantomData } }
        pub fn from_owned(s: String) -> Self { ObjectTypeName { inner: Cow::Owned(s), _kind: PhantomData } }
        pub fn as_str(&self) -> &str { &self.inner }
    }

    pub struct EventTypeName<K> { inner: Cow<'static, str>, _kind: PhantomData<K> }
    impl<K> EventTypeName<K> {
        pub fn from_static(s: &'static str) -> Self { EventTypeName { inner: Cow::Borrowed(s), _kind: PhantomData } }
        pub fn from_owned(s: String) -> Self { EventTypeName { inner: Cow::Owned(s), _kind: PhantomData } }
        pub fn as_str(&self) -> &str { &self.inner }
    }

    // Legacy StableId / TypedId kept for old imports
    pub struct StableId(pub String);
    impl StableId {
        pub fn new(id: &str) -> Self { StableId(id.to_string()) }
        pub fn as_str(&self) -> &str { &self.0 }
    }

    pub struct TypedId<T> { pub id: String, _type: PhantomData<T> }
    impl<T> TypedId<T> {
        pub fn new(id: &str) -> Self { TypedId { id: id.to_string(), _type: PhantomData } }
        pub fn as_str(&self) -> &str { &self.id }
    }
}

// ── object_lifecycle module ───────────────────────────────────────────────────

pub mod object_lifecycle {
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ObjectLifecyclePhase { Created, Active, Modified, Archived, Deleted }

    impl fmt::Display for ObjectLifecyclePhase {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ObjectLifecyclePhase::Created => write!(f, "created"),
                ObjectLifecyclePhase::Active => write!(f, "active"),
                ObjectLifecyclePhase::Modified => write!(f, "modified"),
                ObjectLifecyclePhase::Archived => write!(f, "archived"),
                ObjectLifecyclePhase::Deleted => write!(f, "deleted"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct LifecycleEvent { pub event_type: String, pub object_id: String }
    impl LifecycleEvent {
        pub fn new(event_type: &str, object_id: &str) -> Self {
            LifecycleEvent { event_type: event_type.to_string(), object_id: object_id.to_string() }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ObjectLifecycle { pub events: Vec<LifecycleEvent> }
    impl ObjectLifecycle {
        pub fn new(events: Vec<LifecycleEvent>) -> Self { ObjectLifecycle { events } }
        pub fn validate(&self) -> Result<(), LifecycleRefusal> {
            if self.events.is_empty() { Err(LifecycleRefusal::EmptyLifecycle) } else { Ok(()) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LifecycleRefusal { EmptyLifecycle, InvalidTransition, OrphanEvent }
}

// ── temporal module ───────────────────────────────────────────────────────────

pub mod temporal {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TemporalOrder { Before, After, Concurrent, Unknown }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TemporalRelation { Before, After, During, Concurrent }

    #[derive(Debug, Clone)]
    pub struct TemporalConstraint { pub relation: TemporalRelation, pub bound_ms: u64 }
    impl TemporalConstraint {
        pub fn new(relation: TemporalRelation, bound_ms: u64) -> Self {
            TemporalConstraint { relation, bound_ms }
        }
        pub fn validate(&self) -> Result<(), TemporalRefusal> {
            if self.bound_ms == 0 { Err(TemporalRefusal::ZeroBound) } else { Ok(()) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TemporalRefusal { ZeroBound, ConflictingConstraints }
}

// ── diagnostic module ─────────────────────────────────────────────────────────

pub mod diagnostic {
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DiagnosticSeverity { Error, Warning, Info }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DiagnosticKind { Warning, Error, Info }

    #[derive(Debug, Clone)]
    pub struct DiagnosticReport { pub kind: DiagnosticKind, pub message: String, pub stage: String }
    impl DiagnosticReport {
        pub fn new(kind: DiagnosticKind, message: &str, stage: &str) -> Self {
            DiagnosticReport { kind, message: message.to_string(), stage: stage.to_string() }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CompatDiagnostic {
        MissingWitness,
        MissingRoundTripFixture,
        RawEvidenceExportedAsAdmitted,
        LossyProjectionWithoutPolicy,
        HiddenFlattening,
        MissingRefusalPath,
        MissingReceiptShape,
        UnreachablePrimitive,
        MigrationRecommended,
    }

    impl fmt::Display for CompatDiagnostic {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                CompatDiagnostic::MissingWitness =>
                    write!(f, "[Error] missing witness marker — evidence has no court"),
                CompatDiagnostic::MissingRoundTripFixture =>
                    write!(f, "[Error] missing round-trip fixture — serialisation is untested"),
                CompatDiagnostic::RawEvidenceExportedAsAdmitted =>
                    write!(f, "[Error] raw evidence exported as admitted — fiat admission bypasses court"),
                CompatDiagnostic::LossyProjectionWithoutPolicy =>
                    write!(f, "[Error] lossy projection without policy — silent data loss"),
                CompatDiagnostic::HiddenFlattening =>
                    write!(f, "[Error] hidden flattening detected — use LossReport to name the loss"),
                CompatDiagnostic::MissingRefusalPath =>
                    write!(f, "[Error] missing refusal path — law has no reachable rejection"),
                CompatDiagnostic::MissingReceiptShape =>
                    write!(f, "[Error] missing receipt shape — provenance not captured"),
                CompatDiagnostic::UnreachablePrimitive =>
                    write!(f, "[Error] unreachable primitive — ghost type in the surface"),
                CompatDiagnostic::MigrationRecommended =>
                    write!(f, "[Info] migration recommended — newer API available"),
            }
        }
    }
}

// ── ocpq module ───────────────────────────────────────────────────────────────

pub mod ocpq {
    use std::marker::PhantomData;
    use crate::{Require, IsTrue};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum OcpqScopeKind { Open, Closed, SingleType }

    pub struct OcpqQueryConst<const KIND: OcpqScopeKind> {
        _scope: ObjectScopeConst,
    }
    impl<const KIND: OcpqScopeKind> OcpqQueryConst<KIND> {
        pub fn new(scope: ObjectScopeConst) -> Self { OcpqQueryConst { _scope: scope } }
        pub fn kind(&self) -> OcpqScopeKind { KIND }
        pub fn scope(&self) -> &ObjectScopeConst { &self._scope }
    }

    pub struct ObjectScopeConst { pub object_types: Vec<String> }
    impl ObjectScopeConst {
        pub fn new(types: impl IntoIterator<Item = impl Into<String>>) -> Self {
            ObjectScopeConst { object_types: types.into_iter().map(|s| s.into()).collect() }
        }
    }

    // Runtime OCPQ query types
    #[derive(Debug, Clone)]
    pub struct ObjectScope { pub object_types: Vec<String> }
    impl ObjectScope {
        pub fn new(types: impl IntoIterator<Item = impl Into<String>>) -> Self {
            ObjectScope { object_types: types.into_iter().map(|s| s.into()).collect() }
        }
        pub fn is_empty(&self) -> bool { self.object_types.is_empty() }
    }

    #[derive(Debug, Clone)]
    pub enum PredicateKind {
        Event(String),
        Object(String),
        Relation(String),
        Temporal(String),
        Cardinality { min: usize, max: usize },
        Nested,
        ChildSetBound { branch_label: String, min: usize, max: usize },
        E2ORelation { event_var: String, object_var: String, qualifier: Option<String> },
        O2ORelation { source_var: String, target_var: String, qualifier: Option<String> },
        TimeBetweenEvents { from_var: String, to_var: String },
    }

    #[derive(Debug, Clone)]
    pub struct Predicate<T = ()> {
        pub kind: PredicateKind,
        _data: std::marker::PhantomData<T>,
    }
    impl<T> Predicate<T> {
        pub fn new(kind: PredicateKind) -> Self { Predicate { kind, _data: std::marker::PhantomData } }
    }

    #[derive(Debug, Clone)]
    pub struct OcpqQuery {
        pub scope: ObjectScope,
        pub predicates: Vec<Predicate>,
        pub sub_queries: Vec<OcpqQuery>,
    }
    impl OcpqQuery {
        pub fn new(scope: ObjectScope) -> Self {
            OcpqQuery { scope, predicates: Vec::new(), sub_queries: Vec::new() }
        }
        pub fn validate(&self) -> Result<(), OcpqRefusal> {
            if self.scope.is_empty() { Err(OcpqRefusal::EmptyFilter) } else { Ok(()) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum OcpqRefusal { EmptyFilter, UnsupportedScope }

    // Typed predicate kinds
    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum EventPredicateKind { ActivityEquals, AttributeEquals, TimestampInRange }

    pub struct TypedEventPredicate<const KIND: EventPredicateKind> {
        _expr: String,
    }
    impl<const KIND: EventPredicateKind> TypedEventPredicate<KIND> {
        pub fn new(expr: &str) -> Self { TypedEventPredicate { _expr: expr.to_string() } }
        pub fn expression(&self) -> &str { &self._expr }
        pub fn kind(&self) -> EventPredicateKind { KIND }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum ObjectPredicateKind { AttributeEquals, TypeEquals }

    pub struct TypedObjectPredicate<const KIND: ObjectPredicateKind> {
        _expr: String,
    }
    impl<const KIND: ObjectPredicateKind> TypedObjectPredicate<KIND> {
        pub fn new(expr: &str) -> Self { TypedObjectPredicate { _expr: expr.to_string() } }
        pub fn expression(&self) -> &str { &self._expr }
        pub fn kind(&self) -> ObjectPredicateKind { KIND }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, std::marker::ConstParamTy)]
    pub enum RelationPredicateKind { E2O, O2O, TimeBetweenEvents }

    pub struct TypedRelationPredicate<const KIND: RelationPredicateKind> {
        _expr: String,
    }
    impl<const KIND: RelationPredicateKind> TypedRelationPredicate<KIND> {
        pub fn new(expr: &str) -> Self { TypedRelationPredicate { _expr: expr.to_string() } }
        pub fn expression(&self) -> &str { &self._expr }
        pub fn kind(&self) -> RelationPredicateKind { KIND }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ObjectTypeCardinality { One, ZeroOrOne, OneOrMany, ZeroOrMany }
}

// ── multiperspective module ───────────────────────────────────────────────────

pub mod multiperspective {
    use std::marker::PhantomData;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ProcessPerspective { ControlFlow, Data, Resource, Time }

    pub struct ControlFlowPerspective;
    pub struct DataPerspective;
    pub struct ResourcePerspective;
    pub struct TimePerspective;
    pub struct PerspectiveCombination<A, B>(PhantomData<(A, B)>);

    pub struct MultiPerspectiveEvidence<T, P> {
        pub inner: T,
        _perspective: PhantomData<P>,
    }
    impl<T, P> MultiPerspectiveEvidence<T, P> {
        pub fn new(inner: T) -> Self { MultiPerspectiveEvidence { inner, _perspective: PhantomData } }
    }

    #[derive(Debug, Clone)]
    pub struct MultiPerspectiveLog { pub traces: Vec<String> }
    impl MultiPerspectiveLog {
        pub fn new(traces: Vec<String>) -> Self { MultiPerspectiveLog { traces } }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PerspectiveRefusal { MissingPerspective, ConflictingPerspectives }

    pub struct ParityComparer;
    impl ParityComparer {
        pub fn assert_epsilon_close(actual: f64, expected: f64) {
            let diff = (actual - expected).abs();
            if diff >= 1e-6 {
                panic!("parity violation: |{actual} - {expected}| = {diff} >= 1e-6");
            }
        }
    }
}

// ── loss module ───────────────────────────────────────────────────────────────

pub mod loss {
    use std::marker::PhantomData;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LossFunction { MeanSquared, CrossEntropy, Hinge }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum LossRefusal { InvalidParameters, NumericalInstability }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LossPolicy { RefuseLoss, AllowLossWithReport, AllowLossSilent, AllowNamedProjection }

    pub struct ProjectionName(pub &'static str);
    impl ProjectionName {
        pub fn as_str(&self) -> &str { self.0 }
    }

    pub struct NamedLoss {
        pub projection_str: String,
        pub _category: String,
    }
    impl NamedLoss {
        pub fn projection(&self) -> ProjectionNameOwned { ProjectionNameOwned(self.projection_str.clone()) }
        pub fn category(&self) -> &str { &self._category }
    }

    // Runtime projection name (owns the string, unlike the static ProjectionName)
    pub struct ProjectionNameOwned(pub String);
    impl ProjectionNameOwned {
        pub fn as_str(&self) -> &str { &self.0 }
    }

    pub struct LossReport<A, B, Dropped> {
        pub projection: ProjectionName,
        pub policy: LossPolicy,
        _dropped: Dropped,
        _ph: PhantomData<(A, B)>,
    }
    impl<A, B, Dropped> LossReport<A, B, Dropped> {
        pub fn new(proj: ProjectionName, policy: LossPolicy, dropped: Dropped) -> Self {
            LossReport { projection: proj, policy, _dropped: dropped, _ph: PhantomData }
        }
        pub fn into_lost(self) -> Dropped { self._dropped }
        pub fn summary(&self, category: &str) -> NamedLoss {
            NamedLoss { projection_str: self.projection.0.to_string(), _category: category.to_string() }
        }
    }

    // is_lossless specialised for Vec-based Dropped
    impl<A, B, T> LossReport<A, B, Vec<T>> {
        pub fn is_lossless(&self) -> bool { self._dropped.is_empty() }
    }

    pub struct LossChain { steps: Vec<NamedLoss> }
    impl LossChain {
        pub fn new() -> Self { LossChain { steps: Vec::new() } }
        pub fn push(&mut self, step: NamedLoss) { self.steps.push(step); }
        pub fn is_empty(&self) -> bool { self.steps.is_empty() }
        pub fn is_lossless(&self) -> bool { self.steps.is_empty() }
        pub fn len(&self) -> usize { self.steps.len() }
        pub fn steps(&self) -> &[NamedLoss] { &self.steps }
    }
}
