// Reference witness: the RUNTIME MultipleInstanceSpec — positive validate path,
// InstanceCreationKind variants, threshold field (COVERAGE.md §2 — runtime
// multi-instance spec; complements the const-generic twin in §2.3bz and the two
// refusal paths in reference_petri_refusals.rs).
//
// The const-generic MultipleInstanceSpecConst makes bound violations unrepresentable;
// this runtime struct makes them *refusable* (validate → InvalidInstanceBounds) while
// admitting valid specs. This witnesses the Ok side plus the Static/Dynamic creation
// distinction and the optional threshold. Failing-when-fake: if validate() rejected
// a valid spec, or creation kinds collapsed, the asserts fail.

use wasm4pm_compat::petri::{InstanceCreationKind, MultipleInstanceSpec};

#[test]
fn valid_runtime_spec_passes_validation_and_keeps_its_fields() {
    let spec = MultipleInstanceSpec::new(2, Some(5), Some(3), InstanceCreationKind::Dynamic);
    assert!(spec.validate().is_ok(), "min=2 <= max=5 is a lawful multiplicity");
    assert_eq!(spec.min, 2);
    assert_eq!(spec.max, Some(5));
    assert_eq!(spec.threshold, Some(3), "threshold preserved");
    assert_eq!(spec.creation, InstanceCreationKind::Dynamic);
}

#[test]
fn unbounded_max_is_lawful_and_creation_kinds_are_distinct() {
    // max = None means "no upper bound" — still lawful as long as min >= 1.
    let open = MultipleInstanceSpec::new(1, None, None, InstanceCreationKind::Static);
    assert!(open.validate().is_ok(), "min>=1 with no max is lawful");
    assert_eq!(open.creation, InstanceCreationKind::Static);
    // The two creation kinds are genuinely distinct values.
    assert_ne!(InstanceCreationKind::Static, InstanceCreationKind::Dynamic);
}
