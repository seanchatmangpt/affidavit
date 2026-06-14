// Witness: the wasm4pm-compat OCEL court laws that affidavit's admission DEPENDS
// on actually fire against their violations (v32 TY-7 / TY-9).
//
// Affidavit delegates structural admission to `OcelLog::validate()`. That
// delegation is only meaningful if the court's named laws are reachable — a
// refusal variant that no input can produce is the ghost-variant failure (T-2).
// These tests construct each violating input and assert the EXACT named refusal,
// so the court I rely on is proven non-hollow. Failing-when-fake: if the court
// stops refusing (law removed), these fail; remove wasm4pm-compat and they don't
// compile.
//
// Note: affidavit's own admit() path can only produce EmptyEventObjectLinks
// (objects are defined inline, so dangling links cannot arise through the
// producer). DanglingEventObjectLink is therefore witnessed at the court
// boundary directly — proving the law I depend on fires, even though my
// projection cannot trigger it. That honesty is the point: I witness the court's
// guarantee, not a convenient subset.

use wasm4pm_compat::ocel::{EventObjectLink, Object, ObjectChange, ObjectObjectLink, OcelEvent, OcelLog, OcelRefusal};
use wasm4pm_compat::dfg::{Dfg, DfgEdge, DfgNode, DfgRefusal};
use wasm4pm_compat::process_tree::{
    ProcessTree, ProcessTreeNode, ProcessTreeNodeId, ProcessTreeOperator, ProcessTreeRefusal,
};
use wasm4pm_compat::eventlog::{
    Event as ElEvent, EventLogRefusal, Trace as ElTrace,
};
use wasm4pm_compat::bpmn::{BpmnEdge, BpmnEvent, BpmnLane, BpmnNode, BpmnProcess, BpmnRefusal};
use std::collections::HashSet;
use wasm4pm_compat::petri::{Arc, Marking, PetriNet, PetriRefusal, Place, Transition, WfNet};
use wasm4pm_compat::xes::{XesEvent, XesExtension, XesLog, XesRefusal, XesTrace};
use wasm4pm_compat::powl::{PowlChoiceNode, PowlNodeId, PowlRefusal};
use wasm4pm_compat::declare::{
    Activity, DeclareConstraint, DeclareRefusal, DeclareScope, DeclareTemplate,
};
use wasm4pm_compat::receipt::{
    Digest, ReceiptChain, ReceiptEnvelope, ReceiptRefusal, ReplayHint,
};
use wasm4pm_compat::interop::{
    check_filter_shape, ArtifactGrounding, FilterShape, InteropRefusal, Pm4pyShape,
};
use wasm4pm_compat::models::{PetriNet as ModelPetriNet, PetriNetRefusal};

fn log(
    objects: Vec<Object>,
    events: Vec<OcelEvent>,
    e2o: Vec<EventObjectLink>,
) -> OcelLog {
    OcelLog::new(
        objects,
        events,
        e2o,
        Vec::<ObjectObjectLink>::new(),
        Vec::<ObjectChange>::new(),
    )
}

#[test]
fn court_refuses_empty_event_object_links_by_name() {
    // An event with no object links: the log has events but zero e2o links.
    let l = log(
        vec![],
        vec![OcelEvent::new("evt-0", "create")],
        vec![], // the empty room
    );
    assert_eq!(
        l.validate(),
        Err(OcelRefusal::EmptyEventObjectLinks),
        "court must refuse empty event-object links by name"
    );
}

#[test]
fn court_refuses_dangling_event_object_link_by_name() {
    // A link references object "ghost", which is not in the object set.
    let l = log(
        vec![Object::new("real", "artifact")],
        vec![OcelEvent::new("evt-0", "create")],
        vec![EventObjectLink::new("evt-0", "ghost")], // dangling
    );
    assert_eq!(
        l.validate(),
        Err(OcelRefusal::DanglingEventObjectLink),
        "court must refuse a dangling event-object link by name"
    );
}

#[test]
fn court_admits_a_well_formed_log() {
    // Positive control: a link-consistent, non-empty log validates.
    let l = log(
        vec![Object::new("real", "artifact")],
        vec![OcelEvent::new("evt-0", "create")],
        vec![EventObjectLink::new("evt-0", "real")],
    );
    assert_eq!(l.validate(), Ok(()), "well-formed log must validate");
}

// ── DfgRefusal — both named variants fired against real violations (TY-9) ──
// A second court law, completely witnessed: every variant of DfgRefusal is
// reached by a constructed violating directly-follows graph. No variant is a
// ghost. (A receipt's events project to a DFG in the discover path; this proves
// the DFG admission law the discovery surface relies on is non-hollow.)

#[test]
fn dfg_court_refuses_empty_graph_by_name() {
    // A DFG with no nodes cannot represent process behaviour.
    let dfg = Dfg::new(Vec::<DfgNode>::new(), Vec::<DfgEdge>::new());
    assert_eq!(
        dfg.validate(),
        Err(DfgRefusal::EmptyGraph),
        "an empty DFG must be refused by name"
    );
}

#[test]
fn dfg_court_refuses_dangling_edge_by_name() {
    // An edge targets "ghost", which is not in the node set.
    let dfg = Dfg::new(
        vec![DfgNode::new("create")],
        vec![DfgEdge::new("create", "ghost", 1)], // dangling target
    );
    assert_eq!(
        dfg.validate(),
        Err(DfgRefusal::DanglingEdge),
        "a DFG edge to a missing node must be refused by name"
    );
}

#[test]
fn dfg_court_admits_a_well_formed_graph() {
    // Positive control: edges only between present nodes.
    let dfg = Dfg::new(
        vec![DfgNode::new("create"), DfgNode::new("release")],
        vec![DfgEdge::new("create", "release", 1)],
    );
    assert_eq!(dfg.validate(), Ok(()), "well-formed DFG must validate");
}

// ── ProcessTreeRefusal — five named variants fired against real violations ──
// Directly relevant: affidavit's `model` verb discovers process trees, so the
// process-tree admission law is on the path the discovery surface relies on.
// Each violation is a constructed malformed tree; `admit_shape()` refuses by the
// specific named variant. No variant here is a ghost.

#[test]
fn process_tree_court_admits_well_formed_sequence() {
    // Positive control: Sequence(a, b) with a declared root.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    t.nodes.push(ProcessTreeNode::Activity("b".into()));
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(1)],
    });
    t.root = Some(ProcessTreeNodeId(2));
    assert_eq!(t.admit_shape(), Ok(()), "well-formed Sequence(a,b) admits");
}

#[test]
fn process_tree_court_refuses_missing_root() {
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::MissingRoot));
}

#[test]
fn process_tree_court_refuses_dangling_node_reference() {
    // An operator child id points out of bounds.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(99)], // 99 is OOB
    });
    t.root = Some(ProcessTreeNodeId(0));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::DanglingNodeReference));
}

#[test]
fn process_tree_court_refuses_tau_leaf_with_children() {
    // A Silent (tau) operator must carry no children.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Silent,
        children: vec![ProcessTreeNodeId(0)], // tau with a child — invalid
    });
    t.root = Some(ProcessTreeNodeId(1));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::TauLeafWithChildren));
}

#[test]
fn process_tree_court_refuses_below_minimum_arity() {
    // Sequence requires >= 2 children; give it one.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(0)], // arity 1 < 2
    });
    t.root = Some(ProcessTreeNodeId(1));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::BelowMinimumArity));
}

#[test]
fn process_tree_court_refuses_loop_invalid_arity() {
    // Loop requires exactly 2 children; give it three.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    t.nodes.push(ProcessTreeNode::Activity("b".into()));
    t.nodes.push(ProcessTreeNode::Activity("c".into()));
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Loop,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(1), ProcessTreeNodeId(2)],
    });
    t.root = Some(ProcessTreeNodeId(3));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::InvalidArity));
}

#[test]
fn process_tree_court_refuses_cycle_detected() {
    // A back-edge: node 0 → node 1 → node 0. Both operators have valid arity (2),
    // all child ids in-bounds, so only cycle detection can refuse it.
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(1), ProcessTreeNodeId(2)],
    });
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(2)], // back to 0
    });
    t.nodes.push(ProcessTreeNode::Activity("a".into()));
    t.root = Some(ProcessTreeNodeId(0));
    assert_eq!(t.admit_shape(), Err(ProcessTreeRefusal::CycleDetected));
}

// ── EventLogRefusal — on affidavit's discovery path (the receipt → EventLog) ──
// affidavit's `model` verb projects a receipt into an event log; the event-log
// admission law guards that surface. Both reachable Trace-level variants fire.

#[test]
fn eventlog_court_refuses_empty_trace() {
    let trace = ElTrace::from_events(Vec::<ElEvent>::new());
    assert_eq!(trace.validate(), Err(EventLogRefusal::EmptyTrace));
}

#[test]
fn eventlog_court_refuses_non_monotonic_trace() {
    // Two events with DECREASING timestamps — time cannot run backwards in a case.
    let trace = ElTrace::from_events(vec![
        ElEvent::new("create").at_ns(100),
        ElEvent::new("release").at_ns(50), // earlier than its predecessor
    ]);
    assert_eq!(trace.validate(), Err(EventLogRefusal::NonMonotonicTrace));
}

#[test]
fn eventlog_court_admits_monotonic_trace() {
    let trace = ElTrace::from_events(vec![
        ElEvent::new("create").at_ns(10),
        ElEvent::new("release").at_ns(20),
    ]);
    assert_eq!(trace.validate(), Ok(()), "a monotonic non-empty trace admits");
}

// ── BpmnRefusal — five named variants fired against real violations ──
// A fifth court law, exercising the BPMN structural admission surface.

fn start_end() -> [BpmnNode; 2] {
    [
        BpmnNode::event("s", BpmnEvent::Start),
        BpmnNode::event("e", BpmnEvent::End),
    ]
}

#[test]
fn bpmn_court_refuses_empty_process() {
    let p = BpmnProcess::new(Vec::<BpmnNode>::new(), Vec::<BpmnEdge>::new());
    assert_eq!(p.validate(), Err(BpmnRefusal::EmptyProcess));
}

#[test]
fn bpmn_court_refuses_duplicate_node_id() {
    let p = BpmnProcess::new(
        [
            BpmnNode::event("dup", BpmnEvent::Start),
            BpmnNode::event("dup", BpmnEvent::End), // same id
        ],
        Vec::<BpmnEdge>::new(),
    );
    assert_eq!(p.validate(), Err(BpmnRefusal::DuplicateNodeId));
}

#[test]
fn bpmn_court_refuses_missing_start_event() {
    let p = BpmnProcess::new(
        [BpmnNode::event("e", BpmnEvent::End)], // no Start
        Vec::<BpmnEdge>::new(),
    );
    assert_eq!(p.validate(), Err(BpmnRefusal::MissingStartEvent));
}

#[test]
fn bpmn_court_refuses_missing_end_event() {
    let p = BpmnProcess::new(
        [BpmnNode::event("s", BpmnEvent::Start)], // no End
        Vec::<BpmnEdge>::new(),
    );
    assert_eq!(p.validate(), Err(BpmnRefusal::MissingEndEvent));
}

#[test]
fn bpmn_court_refuses_dangling_edge() {
    let p = BpmnProcess::new(start_end(), [BpmnEdge::new("s", "ghost")]);
    assert_eq!(p.validate(), Err(BpmnRefusal::DanglingEdge));
}

#[test]
fn bpmn_court_admits_well_formed_process() {
    let p = BpmnProcess::new(start_end(), [BpmnEdge::new("s", "e")]);
    assert_eq!(p.validate(), Ok(()), "start→end with a connecting edge admits");
}

#[test]
fn bpmn_court_refuses_lane_node_not_declared() {
    // A lane assigns a node id ("ghost") that the process never declared.
    let lane = BpmnLane::new("l1", "Ops", ["t1", "ghost"]);
    let known: HashSet<&str> = ["t1"].into_iter().collect();
    assert_eq!(lane.validate(&known), Err(BpmnRefusal::LaneNodeNotDeclared));
}

#[test]
fn bpmn_court_admits_lane_with_declared_nodes() {
    let lane = BpmnLane::new("l1", "Ops", ["t1", "t2"]);
    let known: HashSet<&str> = ["t1", "t2"].into_iter().collect();
    assert_eq!(lane.validate(&known), Ok(()), "a lane over declared nodes admits");
}

// ── PetriRefusal — the classical van der Aalst PROPER-COMPLETION criterion ──
// A WfNet (workflow net) with no final marking violates proper completion — ONE
// of the three classical soundness criteria. WfNet::validate checks ONLY this
// one (MissingFinalMarking); option-to-complete and no-dead-transitions are NOT
// checked here (DeadTransition is in fact a ghost variant — zero producers). We
// witness proper-completion only, and do not claim the full soundness triple.

fn simple_net() -> PetriNet {
    PetriNet::new(
        [Place::new("p0"), Place::new("p1")],
        [Transition::new("t0", "fire")],
        [
            Arc::place_to_transition("p0", "t0"),
            Arc::transition_to_place("t0", "p1"),
        ],
        Marking::new([("p0".to_string(), 1)]),
    )
}

#[test]
fn petri_court_refuses_missing_final_marking() {
    // A workflow net with an EMPTY final marking cannot witness proper completion.
    let wf = WfNet::new(simple_net(), Marking::empty());
    assert_eq!(
        wf.validate(),
        Err(PetriRefusal::MissingFinalMarking),
        "a WfNet without a final marking must be refused by name"
    );
}

#[test]
fn petri_court_admits_net_with_final_marking() {
    // Positive control: a declared final marking on p1 admits.
    let wf = WfNet::new(simple_net(), Marking::new([("p1".to_string(), 1)]));
    assert_eq!(wf.validate(), Ok(()), "a WfNet with a final marking admits");
}

// ── XesRefusal — the classical (case-centric) event-log standard ──
// XesLog::validate fires a family of named laws. Four reached against real
// violations; the positive control admits a well-formed log.

#[test]
fn xes_court_refuses_missing_log_name() {
    let log = XesLog::new("", [], [XesTrace::new("c", [XesEvent::new().with("concept:name", "a")])]);
    assert_eq!(log.validate(), Err(XesRefusal::MissingLogName));
}

#[test]
fn xes_court_refuses_no_traces() {
    let log = XesLog::new("l", [], []);
    assert_eq!(log.validate(), Err(XesRefusal::NoTraces));
}

#[test]
fn xes_court_refuses_empty_trace() {
    let log = XesLog::new("l", [], [XesTrace::new("c", [])]);
    assert_eq!(log.validate(), Err(XesRefusal::EmptyTrace));
}

#[test]
fn xes_court_refuses_missing_concept_name() {
    // Event with no concept:name attribute.
    let log = XesLog::new("l", [], [XesTrace::new("c", [XesEvent::new()])]);
    assert_eq!(log.validate(), Err(XesRefusal::MissingConceptName));
}

#[test]
fn xes_court_refuses_invalid_extension() {
    // An extension with an empty prefix is structurally invalid.
    let log = XesLog::new(
        "l",
        [XesExtension::new("Concept", "", "uri")], // empty prefix
        [XesTrace::new("c", [XesEvent::new().with("concept:name", "a")])],
    );
    assert_eq!(log.validate(), Err(XesRefusal::InvalidExtension));
}

#[test]
fn xes_court_refuses_missing_trace_name() {
    // A trace with an empty name (log name present, concept ext declared).
    let log = XesLog::new(
        "l",
        [XesExtension::new("Concept", "concept", "uri")],
        [XesTrace::new("", [XesEvent::new().with("concept:name", "a")])], // empty trace name
    );
    assert_eq!(log.validate(), Err(XesRefusal::MissingTraceName));
}

#[test]
fn xes_court_refuses_undeclared_extension_prefix() {
    // `concept:name` is namespaced; with no declared `concept` extension, the
    // prefix is undeclared and the court refuses by name.
    let log = XesLog::new(
        "l",
        [], // no extensions declared
        [XesTrace::new("c", [XesEvent::new().with("concept:name", "create")])],
    );
    assert_eq!(log.validate(), Err(XesRefusal::UndeclaredExtensionPrefix));
}

#[test]
fn xes_court_admits_well_formed_log() {
    // Positive control: declare the `concept` extension so the namespaced
    // `concept:name` key references a declared prefix.
    let log = XesLog::new(
        "l",
        [XesExtension::new("Concept", "concept", "http://www.xes-standard.org/concept.xesext")],
        [XesTrace::new("c", [XesEvent::new().with("concept:name", "create")])],
    );
    assert_eq!(log.validate(), Ok(()), "a named log with a declared extension admits");
}

// ── PowlRefusal — POWL (partially-ordered workflow language) choice arity ──
// A choice node with fewer than two branches degrades to a trivial projection;
// the court refuses it by name, carrying the arity data in the variant.

#[test]
fn powl_court_refuses_invalid_choice_arity() {
    let bad = PowlChoiceNode::new(vec![PowlNodeId(0)]); // one branch < 2
    assert_eq!(
        bad.validate(),
        Err(PowlRefusal::InvalidChoiceArity { declared: 1, required_min: 2 }),
        "a single-branch choice must be refused with its arity data"
    );
}

#[test]
fn powl_court_admits_well_formed_choice() {
    let ok = PowlChoiceNode::new(vec![PowlNodeId(0), PowlNodeId(1)]);
    assert!(ok.validate().is_ok(), "a two-branch choice admits");
}

// ── DeclareRefusal — DECLARE constraint structural law (object-centric) ──
// DECLARE constraints carry an activation, optional target, and an object scope.
// Three named laws fired against real malformed constraints.

#[test]
fn declare_court_refuses_missing_activation() {
    let c = DeclareConstraint::unary(
        DeclareTemplate::Existence,
        Activity::new(""), // empty activation
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.validate(), Err(DeclareRefusal::MissingActivation));
}

#[test]
fn declare_court_refuses_empty_object_scope() {
    let c = DeclareConstraint::unary(
        DeclareTemplate::Existence,
        Activity::new("place_order"),
        DeclareScope::MultiObjectScope(vec![]), // empty scope
    );
    assert_eq!(c.validate(), Err(DeclareRefusal::EmptyObjectScope));
}

#[test]
fn declare_court_refuses_synchronization_violation() {
    let c = DeclareConstraint::unary(
        DeclareTemplate::Existence,
        Activity::new("place_order"),
        DeclareScope::SynchronizedObjectScope(vec!["order".into()]), // < 2 types
    );
    assert_eq!(c.validate(), Err(DeclareRefusal::SynchronizationViolation));
}

#[test]
fn declare_court_refuses_missing_target() {
    // A binary template (Response, arity 2) built without a target.
    let c = DeclareConstraint::unary(
        DeclareTemplate::Response,
        Activity::new("place_order"),
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.validate(), Err(DeclareRefusal::MissingTarget));
}

#[test]
fn declare_court_refuses_invalid_template_arity() {
    // A unary template (Existence, arity 1) given a target it must not have.
    let c = DeclareConstraint::binary(
        DeclareTemplate::Existence,
        Activity::new("place_order"),
        Activity::new("ship"),
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.validate(), Err(DeclareRefusal::InvalidTemplateArity));
}

#[test]
fn declare_court_admits_well_formed_constraint() {
    let c = DeclareConstraint::unary(
        DeclareTemplate::Existence,
        Activity::new("place_order"),
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.validate(), Ok(()), "a unary constraint with activation + scope admits");
}

// ── ReceiptRefusal — the receipt-shape law, in affidavit's OWN domain ──
// affidavit mints receipts; the court's receipt-envelope/chain laws are the
// most directly relevant of all. Six named variants fired against real
// violations (4 envelope-shape + EmptyChain + BrokenChainLink).

fn good_envelope() -> ReceiptEnvelope {
    ReceiptEnvelope::try_from_parts(
        "subject",
        "witness",
        Digest::new("d"),
        ReplayHint::new("h"),
    )
    .expect("well-shaped envelope")
}

#[test]
fn receipt_court_refuses_missing_subject() {
    let r = ReceiptEnvelope::try_from_parts("", "w", Digest::new("d"), ReplayHint::new("h"));
    assert_eq!(r, Err(ReceiptRefusal::MissingSubject));
}

#[test]
fn receipt_court_refuses_missing_witness() {
    let r = ReceiptEnvelope::try_from_parts("s", "", Digest::new("d"), ReplayHint::new("h"));
    assert_eq!(r, Err(ReceiptRefusal::MissingWitness));
}

#[test]
fn receipt_court_refuses_missing_digest() {
    let r = ReceiptEnvelope::try_from_parts("s", "w", Digest::new(""), ReplayHint::new("h"));
    assert_eq!(r, Err(ReceiptRefusal::MissingDigest));
}

#[test]
fn receipt_court_refuses_missing_replay_hint() {
    let r = ReceiptEnvelope::try_from_parts("s", "w", Digest::new("d"), ReplayHint::new(""));
    assert_eq!(r, Err(ReceiptRefusal::MissingReplayHint));
}

#[test]
fn receipt_court_refuses_empty_chain() {
    let r = ReceiptChain::try_new("run-x", vec![]);
    assert!(matches!(r, Err(ReceiptRefusal::EmptyChain)));
}

#[test]
fn receipt_court_admits_well_shaped_chain() {
    let r = ReceiptChain::try_new("run-x", vec![good_envelope()]);
    assert!(r.is_ok(), "a chain of well-shaped envelopes admits");
}

// ── InteropRefusal — the convergence/divergence guard at the interop boundary ──
// The interop grammar refuses smuggling an object-centric shape under a flat
// claim (FlatClaimOverObjectCentric) — the same convergence/divergence concern
// OCEL exists for, enforced at the pm4py boundary.

#[test]
fn interop_court_refuses_ungrounded_artifact() {
    let g = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, ""); // no evidence ref
    assert_eq!(g.admit_flat(), Err(InteropRefusal::UngroundedArtifact));
}

#[test]
fn interop_court_refuses_flat_claim_over_object_centric() {
    let g = ArtifactGrounding::<()>::new(Pm4pyShape::ObjectCentricLog, "ocel:fixture-1");
    assert_eq!(g.admit_flat(), Err(InteropRefusal::FlatClaimOverObjectCentric));
}

#[test]
fn interop_court_refuses_dimension_shape_mismatch() {
    // An object-type filter over a non-object-centric artifact is a shape mismatch.
    assert_eq!(
        check_filter_shape(Pm4pyShape::EventLog, FilterShape::ObjectType),
        Err(InteropRefusal::DimensionShapeMismatch)
    );
}

#[test]
fn interop_court_admits_grounded_flat_artifact() {
    let g = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, "xes:fixture-1");
    assert_eq!(g.admit_flat(), Ok(()), "a grounded flat artifact admits");
}

// ── PetriNetRefusal (models::PetriNet) — the empty-net law ──

#[test]
fn petrinet_court_refuses_empty_net() {
    assert_eq!(
        ModelPetriNet::default().validate(),
        Err(PetriNetRefusal::EmptyNet),
        "an empty Petri net is refused by name"
    );
}
