// Reference witness: the performance-DFG edge (frequency + time) and the
// object-centric DFG (per-object-type DFGs), COVERAGE.md §2 — advanced DFG.
//
// DfgEdgeFull carries a directly-follows frequency AND an optional duration_ns
// (the time perspective on a DFG edge). ObjectCentricDfg maps each object type to
// its own flat Dfg — discovery's object-centric output. This witnesses both.

use wasm4pm_compat::dfg::{Dfg, DfgEdge, DfgEdgeFull, DfgNode, ObjectCentricDfg};

#[test]
fn performance_dfg_edge_carries_frequency_and_duration() {
    let edge = DfgEdgeFull::new("create", "release", 5).with_duration_ns(1_000_000);
    assert_eq!(edge.source(), "create");
    assert_eq!(edge.target(), "release");
    assert_eq!(edge.frequency().0, 5, "directly-follows frequency");
    assert_eq!(edge.duration_ns(), Some(1_000_000), "time perspective: mean duration");

    // Without a duration, the time perspective is absent (not fabricated).
    let no_time = DfgEdgeFull::new("a", "b", 1);
    assert_eq!(no_time.duration_ns(), None);
}

#[test]
fn object_centric_dfg_holds_per_type_dfgs() {
    let order_dfg = Dfg::new(
        vec![DfgNode::new("create"), DfgNode::new("ship")],
        vec![DfgEdge::new("create", "ship", 3)],
    );
    let item_dfg = Dfg::new(vec![DfgNode::new("pick")], vec![]);

    let ocdfg = ObjectCentricDfg::new()
        .with_type_dfg("order", order_dfg)
        .with_type_dfg("item", item_dfg);

    let types: Vec<&str> = {
        let mut t: Vec<&str> = ocdfg.object_types().collect();
        t.sort_unstable();
        t
    };
    assert_eq!(types, vec!["item", "order"], "both object types present");
    assert_eq!(ocdfg.get("order").unwrap().edges().len(), 1, "order dfg has its edge");
    assert!(ocdfg.get("nonexistent").is_none(), "unknown object type → None");
}
