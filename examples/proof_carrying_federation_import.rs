//! Proof-carrying federation import — foreign OCEL JSON must pass OCEDO
//! validate (no dangling E2O) before a DFG seal subject is minted.
//!
//! Run: `cargo run --example proof_carrying_federation_import`

use affidavit::mining::ocel_compat_import::{
    admit_foreign_ocel_json, REFUSED_OCEL_IMPORT_BROKEN_LINK, REFUSED_OCEL_IMPORT_MALFORMED,
};
use affidavit::mining::ocel_proof_carrying::proof_carrying_dfg_conformance;

fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Honest foreign OCEL v2 JSON: create → pay → ship on one order.
fn honest_json() -> &'static str {
    r#"{
      "eventTypes": [
        {"name": "create", "attributes": []},
        {"name": "pay", "attributes": []},
        {"name": "ship", "attributes": []}
      ],
      "objectTypes": [{"name": "order", "attributes": []}],
      "events": [
        {
          "id": "e1", "type": "create", "time": "2020-01-01T00:00:00Z",
          "attributes": [],
          "relationships": [{"objectId": "o1", "qualifier": "order"}]
        },
        {
          "id": "e2", "type": "pay", "time": "2020-01-01T00:01:00Z",
          "attributes": [],
          "relationships": [{"objectId": "o1", "qualifier": "order"}]
        },
        {
          "id": "e3", "type": "ship", "time": "2020-01-01T00:02:00Z",
          "attributes": [],
          "relationships": [{"objectId": "o1", "qualifier": "order"}]
        }
      ],
      "objects": [{
        "id": "o1", "type": "order", "attributes": [], "relationships": []
      }]
    }"#
}

fn dangling_json() -> &'static str {
    r#"{
      "eventTypes": [{"name": "create", "attributes": []}],
      "objectTypes": [{"name": "order", "attributes": []}],
      "events": [{
        "id": "e1", "type": "create", "time": "2020-01-01T00:00:00Z",
        "attributes": [],
        "relationships": [{"objectId": "MISSING", "qualifier": "order"}]
      }],
      "objects": [{
        "id": "o1", "type": "order", "attributes": [], "relationships": []
      }]
    }"#
}

fn main() {
    println!("=== proof-carrying federation import → DFG subject ===\n");

    let admitted = admit_foreign_ocel_json(honest_json()).expect("honest foreign JSON admits");
    println!(
        "imported: {} events, {} objects (validate.valid={})",
        admitted.ocel.events.len(),
        admitted.ocel.objects.len(),
        admitted.report.valid
    );

    let normative = vec![
        ("create".into(), "pay".into()),
        ("pay".into(), "ship".into()),
    ];
    let pc = proof_carrying_dfg_conformance(&admitted.ocel, "order", "log:foreign", &normative)
        .expect("imported log must mint after validate");
    println!(
        "fitness={:.3} precision={:.3}",
        pc.scores.fitness, pc.scores.precision
    );
    println!("subject_digest = {}", hex32(&pc.subject_digest));
    println!();

    let broken = admit_foreign_ocel_json(dangling_json()).expect_err("dangling E2O must refuse");
    println!("dangling E2O: {broken}");
    assert!(broken.to_string().contains(REFUSED_OCEL_IMPORT_BROKEN_LINK));
    println!("broken link table never reaches subject mint ✓");
    println!();

    let malformed = admit_foreign_ocel_json("{not-json").expect_err("parse must refuse");
    println!("malformed JSON: {malformed}");
    assert!(malformed
        .to_string()
        .contains(REFUSED_OCEL_IMPORT_MALFORMED));
    println!("malformed wire never reaches subject mint ✓");
}
