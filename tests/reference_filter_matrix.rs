// Reference witness: the pm4py filter×artifact compatibility matrix
// (check_filter_shape), COVERAGE.md §2 — interop filter law.
//
// Law: an ObjectType filter is only valid over an object-centric artifact; every
// other filter dimension is valid over any artifact. A mismatch (ObjectType over
// a flat artifact) is refused as DimensionShapeMismatch. This witnesses the full
// matrix — all four "always-valid" filters admit on a flat log, ObjectType admits
// only on object-centric, and the flat×ObjectType cell is refused.

use wasm4pm_compat::interop::{check_filter_shape, FilterShape, InteropRefusal, Pm4pyShape};

#[test]
fn non_object_type_filters_are_valid_on_any_artifact() {
    for filter in [
        FilterShape::Activity,
        FilterShape::Timeframe,
        FilterShape::Variant,
        FilterShape::Attribute,
    ] {
        assert_eq!(
            check_filter_shape(Pm4pyShape::EventLog, filter),
            Ok(()),
            "{filter:?} is valid over a flat event log"
        );
        assert_eq!(
            check_filter_shape(Pm4pyShape::ObjectCentricLog, filter),
            Ok(()),
            "{filter:?} is valid over an object-centric log too"
        );
    }
}

#[test]
fn object_type_filter_requires_object_centric_artifact() {
    // Valid: ObjectType over an object-centric log.
    assert_eq!(
        check_filter_shape(Pm4pyShape::ObjectCentricLog, FilterShape::ObjectType),
        Ok(()),
        "ObjectType filter admits on an object-centric log"
    );
    // Refused: ObjectType over a flat artifact → DimensionShapeMismatch.
    for flat in [
        Pm4pyShape::EventLog,
        Pm4pyShape::PetriNet,
        Pm4pyShape::Bpmn,
        Pm4pyShape::ProcessTree,
    ] {
        assert_eq!(
            check_filter_shape(flat, FilterShape::ObjectType),
            Err(InteropRefusal::DimensionShapeMismatch),
            "ObjectType filter over flat {flat:?} is a dimension mismatch"
        );
    }
}
