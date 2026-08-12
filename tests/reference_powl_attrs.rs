// Reference witness: POWL node kinds and OCEL attribute values — including their
// DATA-carrying variants — are constructed as exhaustive censuses
// (COVERAGE.md §2 — POWL node model + OCEL attribute value model).
//
// wasm4pm-compat v26.8.7 models POWL 2.0 with explicit Start/End boundaries and
// a directed ChoiceGraph. The legacy flat Choice/Loop node variants are absent:
// loops and non-block-structured decisions are represented by ChoiceGraph edges.
//
//   • PowlNodeKind — Start, End, Atom, Silent, PartialOrder, ChoiceGraph.
//   • OcelAttributeValue — the OCEL attribute value union (incl. nested List/Map).

use wasm4pm_compat::ocel::OcelAttributeValue as V;
use wasm4pm_compat::powl::{PowlNodeId, PowlNodeKind as PK};

#[test]
fn powl_node_kinds_are_constructed_with_payloads() {
    let all = [
        PK::Start,
        PK::End,
        PK::Atom("place_order".to_string()),
        PK::Silent,
        PK::PartialOrder(vec![PowlNodeId(0), PowlNodeId(1), PowlNodeId(2)]),
        PK::ChoiceGraph {
            nodes: vec![PowlNodeId(0), PowlNodeId(1)],
            edges: vec![],
        },
    ];
    // Exhaustive no-wildcard match — compile-time census of the canonical
    // POWL 2.0 node taxonomy. Upstream enum drift must break this witness.
    fn kind(p: &PK) -> &'static str {
        match p {
            PK::Start => "start",
            PK::End => "end",
            PK::Atom(_) => "atom",
            PK::Silent => "silent",
            PK::PartialOrder(_) => "partial-order",
            PK::ChoiceGraph { .. } => "choice-graph",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().map(kind).collect();
    assert_eq!(
        s.len(),
        6,
        "all six canonical POWL 2.0 node kinds constructed with payloads"
    );
}

#[test]
fn ocel_attribute_values_are_constructed_including_nested() {
    let all = [
        V::Integer(42),
        V::Float(3.5),
        V::Boolean(true),
        V::String("x".to_string()),
        V::TimestampNs(1_000),
        V::List(vec![V::Integer(1), V::Integer(2)]),
        V::Map(vec![("k".to_string(), V::Boolean(false))]),
        V::Null,
    ];
    // Exhaustive no-wildcard match — census of the attribute-value union,
    // including the recursive List/Map variants.
    fn tag(v: &V) -> &'static str {
        match v {
            V::Integer(_) => "int",
            V::Float(_) => "float",
            V::Boolean(_) => "bool",
            V::String(_) => "string",
            V::TimestampNs(_) => "ts",
            V::List(_) => "list",
            V::Map(_) => "map",
            V::Null => "null",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().map(tag).collect();
    assert_eq!(
        s.len(),
        8,
        "all eight OCEL attribute-value variants constructed"
    );
}
