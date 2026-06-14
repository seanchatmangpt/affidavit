// E2E witness: the `--introspect` LLM tool-calling surface (clap-noun-verb DX) —
// the binary emits a JSON Schema array describing every receipt verb, so an LLM
// can discover and call the CLI as tools (COVERAGE.md — DX introspection).
//
// Failing-when-fake: if a verb were unregistered, it would be absent from the
// schema; if the output weren't valid JSON, the parse fails.

use assert_cmd::Command;
use std::process::Command as StdCommand;

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
    let schema: serde_json::Value = serde_json::from_str(&stdout).expect("introspect output is valid JSON");
    let arr = schema.as_array().expect("a JSON array of tool schemas");
    assert!(!arr.is_empty(), "at least one tool schema");

    // Every receipt verb appears as a named tool with a parameters schema.
    let names: Vec<&str> = arr.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str())).collect();
    for verb in ["receipt_emit", "receipt_assemble", "receipt_verify", "receipt_show",
                 "receipt_inspect", "receipt_model", "receipt_conformance", "receipt_diagnose",
                 "receipt_replay", "receipt_graph", "receipt_stats",
                 "receipt_mutate", "receipt_bench"] {
        assert!(names.contains(&verb), "introspect schema must expose {verb}; got {names:?}");
    }
    // Each tool carries a parameters object (so an LLM can construct the call).
    assert!(
        arr.iter().all(|t| t.get("parameters").map(|p| p.is_object()).unwrap_or(false)),
        "every tool schema has a parameters object"
    );
}

// ---------------------------------------------------------------------------
// `affi --help` — basic help surface
// Failing-when-fake: if the binary couldn't start or the help system were
// broken, this test would fail. Verifies the binary starts and produces help.
// ---------------------------------------------------------------------------

#[test]
fn help_flag_outputs_something_about_affi() {
    // `--help` exits with code 0 under clap and prints to stdout.
    let out = StdCommand::new(env!("CARGO_BIN_EXE_affi"))
        .arg("--help")
        .output()
        .expect("run affi --help");
    // clap exits 0 for --help
    assert!(out.status.success(), "--help must exit 0; stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.is_empty(),
        "--help must produce some output"
    );
}

// ---------------------------------------------------------------------------
// `affi receipt help-refs` — stage/terminology reference
// Failing-when-fake: if help-refs verb is unregistered the process exits non-0;
// if the handler is stubbed the word "stage" would be absent from stderr.
// ---------------------------------------------------------------------------

#[test]
fn help_refs_exits_zero_and_mentions_stage() {
    let out = StdCommand::new(env!("CARGO_BIN_EXE_affi"))
        .args(["receipt", "help-refs"])
        .output()
        .expect("run affi receipt help-refs");
    assert!(out.status.success(), "receipt help-refs must exit 0; stderr={}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("stage"),
        "receipt help-refs stderr must mention 'stage'; got: {stderr}"
    );
}
