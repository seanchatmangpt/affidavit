// Reference witness: positive DFG (directly-follows graph) construction +
// accessors, complementing DfgRefusal (COVERAGE.md §2 — DFG positive shapes).
//
// Builds a well-formed DFG (nodes + weighted edges between declared nodes) and
// exercises node/edge accessors, edge source/target/weight, and validate() admit.

use wasm4pm_compat::dfg::{Dfg, DfgEdge, DfgNode};

#[test]
fn well_formed_dfg_validates_and_exposes_structure() {
    let dfg = Dfg::new(
        vec![
            DfgNode::new("create"),
            DfgNode::new("transform"),
            DfgNode::new("release"),
        ],
        vec![
            DfgEdge::new("create", "transform", 5),
            DfgEdge::new("transform", "release", 3),
        ],
    );

    assert_eq!(
        dfg.validate(),
        Ok(()),
        "edges only between declared nodes → admits"
    );
    assert_eq!(dfg.nodes().len(), 3, "three activities");
    assert_eq!(dfg.edges().len(), 2, "two directly-follows edges");

    assert_eq!(dfg.nodes()[0].activity(), "create");

    let e0 = &dfg.edges()[0];
    assert_eq!(e0.source(), "create");
    assert_eq!(e0.target(), "transform");
    assert_eq!(
        e0.weight().count(),
        5,
        "edge carries its directly-follows frequency"
    );
}
