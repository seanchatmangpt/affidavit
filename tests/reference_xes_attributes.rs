// Reference witness: the XES extension declaration + trace-attribute surface
// (COVERAGE.md §2 — XES extension/attribute accessors; complements XesRefusal).
//
// XesExtension declares a (name, prefix, uri) triple — the namespace declaration
// that makes a `prefix:key` attribute lawful. XesTraceAttributes is a key→value
// bag with a `concept:name` convenience accessor. This witnesses the accessors
// and the with/get attribute round-trip.

use wasm4pm_compat::xes::{XesExtension, XesTraceAttributes};

#[test]
fn xes_extension_exposes_name_prefix_uri() {
    let ext = XesExtension::new(
        "Concept",
        "concept",
        "http://www.xes-standard.org/concept.xesext",
    );
    assert_eq!(ext.name(), "Concept");
    assert_eq!(
        ext.prefix(),
        "concept",
        "the prefix that lawful namespaced keys must reference"
    );
    assert_eq!(ext.uri(), "http://www.xes-standard.org/concept.xesext");
}

#[test]
fn xes_trace_attributes_round_trip() {
    let attrs = XesTraceAttributes::new()
        .with("concept:name", "case-1")
        .with("cost:total", "42");
    assert_eq!(attrs.get("concept:name"), Some("case-1"));
    assert_eq!(attrs.get("cost:total"), Some("42"));
    assert_eq!(attrs.get("absent"), None, "unset key → None");
    // The concept:name convenience accessor.
    assert_eq!(
        attrs.concept_name(),
        Some("case-1"),
        "concept:name surfaced via accessor"
    );
}
