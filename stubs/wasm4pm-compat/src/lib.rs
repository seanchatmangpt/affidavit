//! Minimal build-compatible stub for `wasm4pm-compat`.
//!
//! Provides exactly the API surface that `affidavit` imports so the build
//! succeeds without the real sibling path crate. Semantics are faithful where
//! needed for tests (OcelLog structural validation) and minimal elsewhere.

pub mod state {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Admitted(());
    impl Admitted {
        pub(crate) fn new() -> Self { Admitted(()) }
    }
}

pub mod evidence {
    use serde::{Deserialize, Serialize};

    /// Admitted evidence carrier. `value` is public so callers can borrow
    /// the inner receipt without consuming the carrier.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Evidence<T, S, C> {
        pub value: T,
        #[serde(skip)]
        _state: std::marker::PhantomData<S>,
        #[serde(skip)]
        _chain: std::marker::PhantomData<C>,
    }

    impl<T, S, C> Evidence<T, S, C> {
        pub(crate) fn new(value: T) -> Self {
            Evidence { value, _state: std::marker::PhantomData, _chain: std::marker::PhantomData }
        }
        /// Consume the carrier, returning the inner value.
        pub fn into_inner(self) -> T { self.value }
        pub fn inner(&self) -> &T { &self.value }
    }
}

pub mod admission {
    use crate::evidence::Evidence;
    use crate::state::Admitted;

    pub struct Admission<T, C> {
        inner: T,
        _chain: std::marker::PhantomData<C>,
    }

    impl<T, C> Admission<T, C> {
        pub fn new(inner: T) -> Self {
            Admission { inner, _chain: std::marker::PhantomData }
        }
        pub fn into_evidence(self) -> Evidence<T, Admitted, C> {
            Evidence::new(self.inner)
        }
    }
}

pub mod ocel {
    use serde::{Deserialize, Serialize};

    // ── Admission-gate types (used by admission.rs) ────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Object { pub id: String, pub obj_type: String }

    impl Object {
        pub fn new(id: &str, obj_type: &str) -> Self {
            Object { id: id.to_string(), obj_type: obj_type.to_string() }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OcelEvent { pub id: String, pub event_type: String }

    impl OcelEvent {
        pub fn new(id: &str, event_type: &str) -> Self {
            OcelEvent { id: id.to_string(), event_type: event_type.to_string() }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct EventObjectLink { pub event_id: String, pub object_id: String }

    impl EventObjectLink {
        pub fn new(event_id: &str, object_id: &str) -> Self {
            EventObjectLink { event_id: event_id.to_string(), object_id: object_id.to_string() }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct ObjectObjectLink { pub source_id: String, pub target_id: String }

    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct ObjectChange { pub object_id: String, pub attribute: String }

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
        _o2o: Vec<ObjectObjectLink>,
        _changes: Vec<ObjectChange>,
    }

    impl OcelLog {
        pub fn new(
            objects: Vec<Object>,
            events: Vec<OcelEvent>,
            event_object_links: Vec<EventObjectLink>,
            object_object_links: Vec<ObjectObjectLink>,
            object_changes: Vec<ObjectChange>,
        ) -> Self {
            OcelLog { objects, events, event_object_links, _o2o: object_object_links, _changes: object_changes }
        }

        /// Structural laws: every event needs ≥1 link; every link references a known object.
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

    // ── Mining types (used by mining.rs / model_mining.rs) ─────────────────

    /// Relationship between an OCEL event and an object.
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct OCELRelationship {
        pub object_id: String,
        pub qualifier: String,
    }

    /// OCEL 2.0 event (mining variant — different from OcelEvent above).
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct OCELEvent {
        pub id: String,
        pub event_type: String,
        pub relationships: Vec<OCELRelationship>,
        pub attributes: std::collections::HashMap<String, String>,
    }

    impl OCELEvent {
        /// `id` is owned (callers pass `ev.id.clone()`); `event_type` is borrowed.
        pub fn new(id: impl Into<String>, event_type: &str) -> Self {
            OCELEvent {
                id: id.into(),
                event_type: event_type.to_string(),
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct OCELObject {
        pub id: String,
        pub obj_type: String,
        pub attributes: std::collections::HashMap<String, String>,
    }

    impl OCELObject {
        pub fn new(id: impl Into<String>, obj_type: &str) -> Self {
            OCELObject { id: id.into(), obj_type: obj_type.to_string(), ..Default::default() }
        }
    }

    /// Typed attribute value for OCEL events.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum OCELAttributeValue {
        String(String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
    }

    /// Named typed attribute on an OCEL event.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct OCELEventAttribute {
        pub name: String,
        pub value: OCELAttributeValue,
    }

    impl OCELEventAttribute {
        pub fn string(name: &str, value: String) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::String(value) }
        }
        pub fn integer(name: &str, value: i64) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::Integer(value) }
        }
        pub fn boolean(name: &str, value: bool) -> Self {
            OCELEventAttribute { name: name.to_string(), value: OCELAttributeValue::Boolean(value) }
        }
    }

    /// Full OCEL 2.0 log (mining variant). `new(events, objects)`.
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct OCEL {
        pub events: Vec<OCELEvent>,
        pub objects: Vec<OCELObject>,
    }

    impl OCEL {
        /// Construct from events and objects.
        pub fn new(events: Vec<OCELEvent>, objects: Vec<OCELObject>) -> Self {
            OCEL { events, objects }
        }

        /// Iterate over events. Used as `ocel.event_set()`.
        pub fn event_set(&self) -> impl Iterator<Item = &OCELEvent> {
            self.events.iter()
        }

        /// Return (object_id, qualifier) pairs linked to `event_id`.
        pub fn e2o<'a>(&'a self, event_id: &str) -> impl Iterator<Item = (&'a str, &'a str)> {
            let eid = event_id.to_string();
            self.events.iter()
                .find(move |ev| ev.id == eid)
                .into_iter()
                .flat_map(|ev| ev.relationships.iter())
                .map(|r| (r.object_id.as_str(), r.qualifier.as_str()))
        }
    }
}

pub mod petri {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Place { pub id: String }

    impl Place {
        /// Single-argument constructor: `Place::new("start")`.
        pub fn new(id: &str) -> Self { Place { id: id.to_string() } }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct Transition { pub id: String, pub label: String }

    impl Transition {
        pub fn new(id: &str, label: &str) -> Self {
            Transition { id: id.to_string(), label: label.to_string() }
        }
        pub fn silent(id: &str) -> Self { Transition { id: id.to_string(), label: String::new() } }
        pub fn id(&self) -> &str { &self.id }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Arc { pub source: String, pub target: String, pub weight: u32 }

    impl Arc {
        pub fn new(source: &str, target: &str) -> Self {
            Arc { source: source.to_string(), target: target.to_string(), weight: 1 }
        }
        pub fn place_to_transition(place: &str, transition: &str) -> Self { Self::new(place, transition) }
        pub fn transition_to_place(transition: &str, place: &str) -> Self { Self::new(transition, place) }
    }

    /// Token marking. `Marking::new([("start".to_string(), 1)])`.
    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Marking(pub std::collections::HashMap<String, u32>);

    impl Marking {
        pub fn new(entries: impl IntoIterator<Item = (String, u32)>) -> Self {
            Marking(entries.into_iter().collect())
        }
        pub fn mark(&mut self, place_id: &str, tokens: u32) {
            self.0.insert(place_id.to_string(), tokens);
        }
        pub fn tokens(&self, place_id: &str) -> u32 {
            self.0.get(place_id).copied().unwrap_or(0)
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct PetriNet {
        pub places: Vec<Place>,
        pub transitions: Vec<Transition>,
        pub arcs: Vec<Arc>,
        pub initial_marking: Marking,
        pub final_marking: Marking,
    }

    impl PetriNet {
        /// Construct from all components: `PetriNet::new(places, transitions, arcs, marking)`.
        pub fn new(
            places: Vec<Place>,
            transitions: Vec<Transition>,
            arcs: Vec<Arc>,
            initial_marking: Marking,
        ) -> Self {
            PetriNet { places, transitions, arcs, initial_marking, ..Default::default() }
        }
        pub fn add_place(&mut self, p: Place) { self.places.push(p); }
        pub fn add_transition(&mut self, t: Transition) { self.transitions.push(t); }
        pub fn add_arc(&mut self, a: Arc) { self.arcs.push(a); }
    }
}

pub mod witness {
    pub trait Witness: Send + Sync + 'static {
        fn id(&self) -> &'static str;
        fn family(&self) -> WitnessFamily;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WitnessFamily { Standard, ApiGrammar, Implementation }
}

pub mod powl {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PowlNodeId(pub u64);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PowlNodeKind {
        Atom(String),
        PartialOrder(Vec<PowlNodeId>),
        Choice(Vec<PowlNodeId>),
        Loop { body: PowlNodeId, redo: PowlNodeId },
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct PowlNode {
        pub id: PowlNodeId,
        pub kind: PowlNodeKind,
    }

    impl PowlNode {
        pub fn new(id: PowlNodeId, kind: PowlNodeKind) -> Self { PowlNode { id, kind } }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OrderEdge {
        pub from: PowlNodeId,
        pub to: PowlNodeId,
    }

    impl OrderEdge {
        pub fn new(from: PowlNodeId, to: PowlNodeId) -> Self { OrderEdge { from, to } }
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub enum PowlRefusal {
        #[default]
        CyclicPartialOrder,
    }

    impl std::fmt::Display for PowlRefusal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "CyclicPartialOrder")
        }
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

        /// Validate that the partial-order edges form a DAG (acyclic).
        pub fn validate(&self) -> Result<(), PowlRefusal> {
            // Kahn's topological sort to detect cycles among edge-connected nodes.
            use std::collections::{HashMap, HashSet, VecDeque};
            let node_ids: HashSet<u64> = self.nodes.iter().map(|n| n.id.0).collect();
            if node_ids.is_empty() { return Ok(()); }
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
            if visited == node_ids.len() { Ok(()) } else { Err(PowlRefusal::CyclicPartialOrder) }
        }
    }
}

pub mod bpmn {
    use std::collections::HashSet;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnEvent { Start, End, Intermediate }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnTask { pub label: String }

    impl BpmnTask {
        pub fn new(label: &str) -> Self { BpmnTask { label: label.to_string() } }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnGateway { Exclusive, Parallel, Inclusive }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnNodeKind {
        Event(BpmnEvent),
        Task(BpmnTask),
        Gateway(BpmnGateway),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnNode {
        id: String,
        kind: BpmnNodeKind,
    }

    impl BpmnNode {
        pub fn event(id: &str, event: BpmnEvent) -> Self {
            BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Event(event) }
        }
        pub fn task(id: &str, task: BpmnTask) -> Self {
            BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Task(task) }
        }
        pub fn gateway(id: &str, gw: BpmnGateway) -> Self {
            BpmnNode { id: id.to_string(), kind: BpmnNodeKind::Gateway(gw) }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn kind(&self) -> &BpmnNodeKind { &self.kind }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct BpmnEdge { source: String, target: String }

    impl BpmnEdge {
        pub fn new(source: &str, target: &str) -> Self {
            BpmnEdge { source: source.to_string(), target: target.to_string() }
        }
        pub fn source(&self) -> &str { &self.source }
        pub fn target(&self) -> &str { &self.target }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum BpmnRefusal { DanglingEdge, MissingStart }

    impl std::fmt::Display for BpmnRefusal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                BpmnRefusal::DanglingEdge => write!(f, "DanglingEdge"),
                BpmnRefusal::MissingStart => write!(f, "MissingStart"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> }

    impl BpmnProcess {
        pub fn new(nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge>) -> Self {
            BpmnProcess { nodes, edges }
        }
        pub fn nodes(&self) -> &[BpmnNode] { &self.nodes }
        pub fn edges(&self) -> &[BpmnEdge] { &self.edges }

        pub fn validate(&self) -> Result<(), BpmnRefusal> {
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
    pub struct BpmnLane {
        id: String,
        name: String,
        node_ids: Vec<String>,
    }

    impl BpmnLane {
        pub fn new(id: &str, name: &str, members: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
            BpmnLane {
                id: id.to_string(),
                name: name.to_string(),
                node_ids: members.into_iter().map(|s| s.as_ref().to_string()).collect(),
            }
        }
        pub fn id(&self) -> &str { &self.id }
        pub fn name(&self) -> &str { &self.name }
        pub fn node_ids(&self) -> &[String] { &self.node_ids }

        pub fn validate(&self, known: &HashSet<&str>) -> Result<(), BpmnRefusal> {
            for id in &self.node_ids {
                if !known.contains(id.as_str()) { return Err(BpmnRefusal::DanglingEdge); }
            }
            Ok(())
        }
    }
}
