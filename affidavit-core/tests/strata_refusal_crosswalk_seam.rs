//! Court: affidavit chain refusal vocabulary <-> refusal-vocabulary-pack
//! crosswalk seam (L210).
//!
//! The refusal-vocabulary-pack (sibling repo:
//! `/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack/`) landed the
//! canonical FAIL-CLOSED crosswalk: 51 erlmcp codes -> 18 ash_a2a classes ->
//! 6 strata-valve mappings (rv:ValveMapping) -> A2A wire column
//! (designated JSON-RPC code -32000). This court pins affidavit's side:
//!
//! 1. the crosswalk's valve-mapping column and A2A wire column are
//!    byte-pinned in `tests/fixtures/refusal_crosswalk_valve.json` (plus the
//!    declared 18-class universe and affidavit's own `RejectReason`
//!    adjudication) with a self sha256 pin;
//! 2. a drift meta-test re-derives the same tables from the live sibling
//!    source of truth (`generated/crosswalk.rs`) at test time and fails on
//!    drift; if the sibling checkout is absent the court skips with an
//!    explicit `BLOCKED:` marker (it never silently passes on absence);
//! 3. fail-closed totality: EVERY refusal class name affidavit's chain
//!    emits — the 4 strata-valve class labels carried in sealed chain
//!    events (event ids `strata.valve.refusals.*`) and every
//!    `RejectReason` variant on the chain verify path — must resolve in
//!    the pinned crosswalk; an unmapped class fails the court;
//! 4. sabotage twin: removing a mapping from the fixture copy must make
//!    the resolution court fail (non-vacuity, proven both directions).
//!
//! Classification only: nothing here grants runtime actuation authority.

use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use affidavit_core::RejectReason;

// ---------------------------------------------------------------------------
// Pinned fixture.
// ---------------------------------------------------------------------------

const FIXTURE_PATH: &str = "tests/fixtures/refusal_crosswalk_valve.json";

/// sha256 of the fixture file bytes (byte-pin). Recompute with
/// `shasum -a 256 tests/fixtures/refusal_crosswalk_valve.json` after any
/// intentional re-pin and update here in the same commit.
const FIXTURE_SHA256: &str = "30598206b8cdd539b8c68a524f982f552173195a88321154cf93a8b71e5f45cf";

/// The sibling source of truth (READ-ONLY for this lane).
const SIBLING_CROSSWALK: &str =
    "/Users/sac/ggen-marketplace/packs/refusal-vocabulary-pack/generated/crosswalk.rs";

fn fixture_text() -> &'static str {
    static TEXT: OnceLock<String> = OnceLock::new();
    TEXT.get_or_init(|| fs::read_to_string(FIXTURE_PATH).expect("fixture readable"))
}

fn fixture_json() -> &'static serde_json::Value {
    static JSON: OnceLock<serde_json::Value> = OnceLock::new();
    JSON.get_or_init(|| serde_json::from_str(fixture_text()).expect("fixture is valid JSON"))
}

/// Fail-closed resolution: valve class -> ash_a2a class from the fixture.
fn resolve_valve(fx: &serde_json::Value, valve_name: &str) -> Option<String> {
    fx["valve_mappings"]
        .as_array()?
        .iter()
        .find(|row| row["valve_name"].as_str() == Some(valve_name))
        .and_then(|row| row["ash_a2a_class"].as_str())
        .map(String::from)
}

/// Fail-closed resolution: RejectReason -> ash_a2a class from the fixture.
fn resolve_reject_reason(fx: &serde_json::Value, name: &str) -> Option<String> {
    fx["affidavit_reject_reasons"]
        .as_object()?
        .get(name)
        .and_then(|row| row["ash_a2a_class"].as_str())
        .map(String::from)
}

/// The full fail-closed totality court over a fixture value: every refusal
/// class affidavit's chain emits must resolve, and every mapping target must
/// be a declared ash_a2a class. Returns Err naming the first hole.
fn run_totality_court(fx: &serde_json::Value) -> Result<(), String> {
    // (a) the 4 strata-valve classes affidavit's chain emits as event ids
    //     (ground truth: tests/strata_refusal_seam.rs pins exactly these).
    let emitted_valve_classes = [
        "malformed_frame",
        "fuel_exhausted",
        "parent_discontinuity",
        "arena_failure",
    ];
    for name in emitted_valve_classes {
        if resolve_valve(fx, name).is_none() {
            return Err(format!(
                "UNMAPPED refusal class emitted by affidavit's chain: valve class '{name}' does not resolve in the crosswalk (fail-closed)"
            ));
        }
    }
    // (b) every RejectReason variant on the chain verify path. The match has
    //     NO wildcard arm: adding a variant to RejectReason is a COMPILE
    //     error here — the fail-closed gate cannot be silently outgrown.
    let emitted_reject_reasons: [(&str, &str); 6] = [
        ("wrong_profile", stringify!(RejectReason::WrongProfile)),
        (
            "chain_hash_mismatch",
            stringify!(RejectReason::ChainHashMismatch),
        ),
        (
            "seq_not_contiguous",
            stringify!(RejectReason::SeqNotContiguous { .. }),
        ),
        (
            "duplicate_event_id",
            stringify!(RejectReason::DuplicateEventId),
        ),
        (
            "empty_event_type",
            stringify!(RejectReason::EmptyEventType { .. }),
        ),
        (
            "zero_commitment",
            stringify!(RejectReason::ZeroCommitment { .. }),
        ),
    ];
    // Compile-time exhaustiveness witness: this match must stay exhaustive.
    fn exhaustive_witness(r: RejectReason) -> &'static str {
        match r {
            RejectReason::WrongProfile => "wrong_profile",
            RejectReason::ChainHashMismatch => "chain_hash_mismatch",
            RejectReason::SeqNotContiguous { .. } => "seq_not_contiguous",
            RejectReason::DuplicateEventId => "duplicate_event_id",
            RejectReason::EmptyEventType { .. } => "empty_event_type",
            RejectReason::ZeroCommitment { .. } => "zero_commitment",
        }
    }
    let _ = exhaustive_witness;
    for (name, _witness) in &emitted_reject_reasons {
        if resolve_reject_reason(fx, name).is_none() {
            return Err(format!(
                "UNMAPPED refusal class emitted by affidavit's chain: RejectReason '{name}' does not resolve in the crosswalk (fail-closed)"
            ));
        }
    }
    // (c) totality: every mapping target is a declared ash_a2a class.
    let declared: Vec<&str> = fx["declared_ash_a2a_classes"]
        .as_array()
        .expect("declared class list")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        declared.len(),
        18,
        "declared ash_a2a class universe must stay 18 (ontology G4 totality)"
    );
    let check_target = |kind: &str, key: &str, target: &str, extra: &str| {
        assert!(
            declared.contains(&target),
            "{kind} row '{key}' targets undeclared ash_a2a class '{target}' {extra}"
        );
    };
    for row in fx["valve_mappings"].as_array().expect("valve rows") {
        check_target(
            "valve",
            row["valve_name"].as_str().unwrap_or("?"),
            row["ash_a2a_class"].as_str().expect("valve class"),
            "",
        );
    }
    for row in fx["a2a_wire_mappings"].as_array().expect("wire rows") {
        check_target(
            "a2a_wire",
            row["a2a_atom"].as_str().unwrap_or("?"),
            row["ash_a2a_class"].as_str().expect("wire class"),
            "",
        );
    }
    for (name, row) in fx["affidavit_reject_reasons"]
        .as_object()
        .expect("reject reason table")
    {
        check_target(
            "reject_reason",
            name,
            row["ash_a2a_class"].as_str().expect("reason class"),
            "",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Courts.
// ---------------------------------------------------------------------------

#[test]
fn fixture_is_byte_pinned_and_wellformed() {
    let bytes = fs::read(FIXTURE_PATH).expect("fixture readable");
    let got = hex(&Sha256::digest(&bytes));
    assert_eq!(
        got, FIXTURE_SHA256,
        "fixture drift: tests/fixtures/refusal_crosswalk_valve.json changed without re-pinning FIXTURE_SHA256"
    );
    let fx = fixture_json();
    // Shape laws.
    assert_eq!(
        fx["valve_mappings"].as_array().unwrap().len(),
        6,
        "valve column stays 6 rows (ontology G5 totality)"
    );
    assert_eq!(
        fx["a2a_wire_mappings"].as_array().unwrap().len(),
        2,
        "A2A wire column stays 2 rows"
    );
    // The designated wire law: the transport column carries -32000 only.
    for row in fx["a2a_wire_mappings"].as_array().unwrap() {
        assert_eq!(
            row["json_rpc_code"].as_i64(),
            Some(-32000),
            "wire code must stay the designated -32000 (G3 law set)"
        );
    }
    run_totality_court(fx).expect("honest fixture passes the totality court");
}

#[test]
fn chain_emitted_valve_classes_resolve_in_crosswalk() {
    let fx = fixture_json();
    // The 4 valve classes affidavit's chain carries as sealed event ids
    // `strata.valve.refusals.<name>` resolve fail-closed; the resolved class
    // is exactly the pack's adjudication (L101-verified).
    let expected = [
        ("malformed_frame", "refused_structure"),
        ("fuel_exhausted", "refused_bounds"),
        ("parent_discontinuity", "refused_receipt"),
        ("arena_failure", "refused_consequence"),
    ];
    for (name, class) in expected {
        assert_eq!(
            resolve_valve(fx, name),
            Some(class.to_string()),
            "valve class '{name}' must resolve to '{class}'"
        );
    }
    // Fail-closed on the negative: an out-of-vocabulary atom does NOT resolve.
    assert_eq!(resolve_valve(fx, "not_a_declared_valve_class"), None);
    // The 2 window-capacity guard classes are declared but not emitted here;
    // they must still be pinned (totality of the column).
    for name in ["window_run_capacity", "window_tau_capacity"] {
        assert_eq!(resolve_valve(fx, name), Some("refused_bounds".to_string()));
    }
}

#[test]
fn reject_reason_vocabulary_is_exhaustively_mapped() {
    let fx = fixture_json();
    // Exhaustive, no-wildcard mapping: growing RejectReason breaks this
    // match at compile time — the court cannot be silently outgrown.
    let emitted: [(&str, String); 6] = [
        (
            "wrong_profile",
            resolve_reject_reason(fx, "wrong_profile").expect("wrong_profile mapped"),
        ),
        (
            "chain_hash_mismatch",
            resolve_reject_reason(fx, "chain_hash_mismatch").expect("chain_hash_mismatch mapped"),
        ),
        (
            "seq_not_contiguous",
            resolve_reject_reason(fx, "seq_not_contiguous").expect("seq_not_contiguous mapped"),
        ),
        (
            "duplicate_event_id",
            resolve_reject_reason(fx, "duplicate_event_id").expect("duplicate_event_id mapped"),
        ),
        (
            "empty_event_type",
            resolve_reject_reason(fx, "empty_event_type").expect("empty_event_type mapped"),
        ),
        (
            "zero_commitment",
            resolve_reject_reason(fx, "zero_commitment").expect("zero_commitment mapped"),
        ),
    ];
    // Compile-time exhaustiveness witness over the real enum: adding a
    // variant to RejectReason is a compile error in this test.
    fn witness(r: RejectReason) -> &'static str {
        match r {
            RejectReason::WrongProfile => "wrong_profile",
            RejectReason::ChainHashMismatch => "chain_hash_mismatch",
            RejectReason::SeqNotContiguous { .. } => "seq_not_contiguous",
            RejectReason::DuplicateEventId => "duplicate_event_id",
            RejectReason::EmptyEventType { .. } => "empty_event_type",
            RejectReason::ZeroCommitment { .. } => "zero_commitment",
        }
    }
    let live: [(&str, &str); 6] = [
        (witness(RejectReason::WrongProfile), "refused_profile"),
        (witness(RejectReason::ChainHashMismatch), "refused_receipt"),
        (
            witness(RejectReason::SeqNotContiguous { index: 0, found: 1 }),
            "refused_structure",
        ),
        (witness(RejectReason::DuplicateEventId), "refused_identity"),
        (
            witness(RejectReason::EmptyEventType { index: 0 }),
            "refused_structure",
        ),
        (
            witness(RejectReason::ZeroCommitment { index: 0 }),
            "refused_consequence",
        ),
    ];
    for (name, class) in live {
        let mapped = emitted
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| c.as_str())
            .unwrap_or("<unmapped>");
        assert_eq!(mapped, class, "RejectReason '{name}' must map to '{class}'");
    }
    // Every name the enum emits appears in the pinned fixture table, and the
    // fixture table holds exactly that vocabulary (no orphan rows).
    let fixture_keys = fx["affidavit_reject_reasons"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(fixture_keys.len(), 6);
    for (name, _) in &emitted {
        assert!(fixture_keys.iter().any(|k| k == name), "{name} in fixture");
    }
}

#[test]
fn crosswalk_targets_are_declared_classes() {
    run_totality_court(fixture_json()).expect("all mapping targets declared");
    // Spot-check the adjudicated targets against the pack's rationale.
    let fx = fixture_json();
    assert_eq!(fx["declared_ash_a2a_classes"][0], "blocked_resource");
    assert!(fx["declared_ash_a2a_classes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "refused_consequence"));
}

#[test]
fn fixture_matches_live_source_of_truth() {
    // Drift meta-test: re-derive the tables from the live sibling pack at
    // test time. Absent sibling => explicit BLOCKED skip, never a silent pass.
    if !Path::new(SIBLING_CROSSWALK).exists() {
        eprintln!(
            "BLOCKED: sibling source of truth absent at {SIBLING_CROSSWALK}; drift court skipped (pin remains sha-pinned by fixture_is_byte_pinned_and_wellformed)"
        );
        return;
    }
    let src = fs::read_to_string(SIBLING_CROSSWALK).expect("sibling crosswalk readable");
    let live_valve = parse_sibling_valve(&src);
    let live_wire = parse_sibling_wire(&src);
    let live_declared = parse_sibling_declared(&src);
    assert!(!live_valve.is_empty(), "sibling valve table parsed");
    assert!(!live_wire.is_empty(), "sibling wire table parsed");
    assert_eq!(live_declared.len(), 18, "sibling declares 18 classes");

    let fx = fixture_json();
    // Valve column drift.
    let mut live_valve_sorted = live_valve.clone();
    live_valve_sorted.sort();
    let mut pinned_valve: Vec<(String, String)> = fx["valve_mappings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["valve_name"].as_str().unwrap().to_string(),
                row["ash_a2a_class"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    pinned_valve.sort();
    assert_eq!(
        live_valve_sorted, pinned_valve,
        "DRIFT: pinned valve crosswalk diverges from live source of truth"
    );
    // Wire column drift.
    let mut live_wire_sorted = live_wire.clone();
    live_wire_sorted.sort();
    let mut pinned_wire: Vec<(String, String, i64)> = fx["a2a_wire_mappings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["a2a_atom"].as_str().unwrap().to_string(),
                row["ash_a2a_class"].as_str().unwrap().to_string(),
                row["json_rpc_code"].as_i64().unwrap(),
            )
        })
        .collect();
    pinned_wire.sort();
    assert_eq!(
        live_wire_sorted, pinned_wire,
        "DRIFT: pinned A2A wire crosswalk diverges from live source of truth"
    );
    // Declared universe drift.
    let mut pinned_declared: Vec<String> = fx["declared_ash_a2a_classes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    pinned_declared.sort();
    let mut live_declared_sorted = live_declared.clone();
    live_declared_sorted.sort();
    assert_eq!(
        live_declared_sorted, pinned_declared,
        "DRIFT: pinned declared class universe diverges from live source of truth"
    );
}

#[test]
fn sabotage_twin_removed_mapping_fails_resolution() {
    // Twin of the totality court: remove one mapping from the fixture COPY
    // (in memory; the pinned file on disk is never mutated) and prove the
    // court fails naming the removed class — the court is non-vacuous.
    let fx: serde_json::Value = serde_json::from_str(fixture_text()).expect("fixture valid");
    let mut sabotaged = fx.clone();
    sabotaged["valve_mappings"] = sabotaged["valve_mappings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["valve_name"].as_str() != Some("fuel_exhausted"))
        .cloned()
        .collect();
    assert_eq!(sabotaged["valve_mappings"].as_array().unwrap().len(), 5);
    // 1. Fail-closed resolution goes dark for the removed class.
    assert_eq!(resolve_valve(&sabotaged, "fuel_exhausted"), None);
    // 2. The full totality court fails, naming the hole.
    let verdict = run_totality_court(&sabotaged);
    match verdict {
        Err(msg) => assert!(
            msg.contains("fuel_exhausted") && msg.contains("UNMAPPED"),
            "sabotage failure must name the removed class, got: {msg}"
        ),
        Ok(()) => panic!("sabotage twin leaked: court passed with a mapping removed"),
    }
    // 3. The honest fixture still passes the same court (control).
    run_totality_court(&fx).expect("honest control passes");
    // 4. Sabotaging the reject-reason table fails the court too.
    let mut sabotaged2 = fx.clone();
    if let Some(obj) = sabotaged2["affidavit_reject_reasons"].as_object_mut() {
        obj.remove("duplicate_event_id");
    }
    let verdict2 = run_totality_court(&sabotaged2);
    assert!(
        matches!(verdict2, Err(ref m) if m.contains("duplicate_event_id")),
        "removing a RejectReason mapping must fail the court, got: {verdict2:?}"
    );
}

// ---------------------------------------------------------------------------
// Sibling source-of-truth parser (text-level; the generated file is a
// deterministic ggen projection, so field extraction by marker is stable).
// ---------------------------------------------------------------------------

fn section<'a>(src: &'a str, start_marker: &str) -> &'a str {
    let start = src
        .find(start_marker)
        .unwrap_or_else(|| panic!("sibling file missing marker {start_marker}"));
    let rest = &src[start..];
    let end = rest.find("\n];").unwrap_or(rest.len());
    &rest[..end]
}

fn field(block: &str, key: &str) -> Option<String> {
    let marker = format!("{key}: ");
    let idx = block.find(&marker)?;
    let rest = block[idx + marker.len()..].trim_start();
    let quoted = rest.starts_with('"') || rest.starts_with("Some(\"") || rest.starts_with("None");
    if !quoted {
        return None;
    }
    if rest.starts_with("None") {
        return None;
    }
    let after_some = if rest.starts_with("Some(\"") { 6 } else { 1 };
    let rest = &rest[after_some..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn int_field(block: &str, key: &str) -> Option<i64> {
    let marker = format!("{key}: ");
    let idx = block.find(&marker)?;
    let rest = block[idx + marker.len()..].trim_start();
    let end = rest.find([',', '\n']).unwrap_or(rest.len());
    rest[..end].trim().parse().ok()
}

fn parse_sibling_valve(src: &str) -> Vec<(String, String)> {
    section(src, "pub const VALVE_CROSSWALK")
        .split("ValveEntry {")
        .skip(1)
        .filter_map(|block| {
            let name = field(block, "valve_name")?;
            let class = field(block, "ash_a2a_class")?;
            Some((name, class))
        })
        .collect()
}

fn parse_sibling_wire(src: &str) -> Vec<(String, String, i64)> {
    section(src, "pub const A2A_WIRE_CROSSWALK")
        .split("A2aWireEntry {")
        .skip(1)
        .filter_map(|block| {
            let atom = field(block, "a2a_atom")?;
            let class = field(block, "ash_a2a_class")?;
            let code = int_field(block, "json_rpc_code")?;
            Some((atom, class, code))
        })
        .collect()
}

fn parse_sibling_declared(src: &str) -> Vec<String> {
    section(src, "pub const ASH_A2A_CLASSES")
        .split('"')
        .skip(1)
        .step_by(2)
        .map(String::from)
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
