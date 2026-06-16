// E2E witness: the `--introspect` LLM tool-calling surface (clap-noun-verb DX) —
// the binary emits a JSON Schema array describing every receipt verb, so an LLM
// can discover and call the CLI as tools (COVERAGE.md — DX introspection).
//
// Failing-when-fake: if a verb were unregistered, it would be absent from the
// schema; if the output weren't valid JSON, the parse fails.

use assert_cmd::Command;

#[test]
fn introspect_emits_valid_schema_for_all_verbs() {
    let out = Command::cargo_bin("affi")
        .expect("affi binary")
        .arg("--introspect")
        .output()
        .expect("run --introspect");
    assert!(out.status.success(), "--introspect exits cleanly");

    let stdout = String::from_utf8(out.stdout).expect("utf8");
    // It is valid JSON (an array of tool schemas).
    let schema: serde_json::Value =
        serde_json::from_str(&stdout).expect("introspect output is valid JSON");
    let arr = schema.as_array().expect("a JSON array of tool schemas");
    assert!(!arr.is_empty(), "at least one tool schema");

    // Every receipt verb appears as a named tool with a parameters schema.
    let names: Vec<&str> = arr
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    for verb in [
        "receipt_emit",
        "receipt_assemble",
        "receipt_verify",
        "receipt_show",
        "receipt_inspect",
        "receipt_model",
        "receipt_conformance",
        "receipt_diagnose",
        "receipt_replay",
        "receipt_graph",
        "receipt_stats",
    ] {
        assert!(
            names.contains(&verb),
            "introspect schema must expose {verb}; got {names:?}"
        );
    }
    // Each tool carries a parameters object (so an LLM can construct the call).
    assert!(
        arr.iter()
            .all(|t| t.get("parameters").map(|p| p.is_object()).unwrap_or(false)),
        "every tool schema has a parameters object"
    );
}
