//! Capability register row #15 — OCEL v2 federation import court.
//!
//! Foreign JSON → court surface:
//! 1. `import_ocel_json` parses the wire bytes (serde refusal on malformed JSON).
//! 2. `ocel::validate` runs the OCEDO/OCPQ invariants.
//! 3. Any dangling E2O/O2O link (or empty E2O) → typed
//!    `REFUSED_OCEL_IMPORT_BROKEN_LINK`.
//! 4. Other validate defects → typed `REFUSED_OCEL_IMPORT_INVALID`.
//!
//! Falsifier: a broken link table that ACCEPTs fails the court.

use std::collections::HashMap;

// `import_ocel_json` lives in `mod_ocel` but is not re-exported from
// `import::ocel` (only ndjson is); reach it through the public submodule.
use wasm4pm_compat::import::ocel::mod_ocel::import_ocel_json;
use wasm4pm_compat::ocel::validate::{validate, ValidationReport};
use wasm4pm_compat::ocel::OCEL;

use crate::mining::MiningError;

/// Typed refusal when an event/object relationship points at a missing id.
pub const REFUSED_OCEL_IMPORT_BROKEN_LINK: &str = "REFUSED_OCEL_IMPORT_BROKEN_LINK";

/// Typed refusal for other OCEL-v2 validate defects (undeclared types, …).
pub const REFUSED_OCEL_IMPORT_INVALID: &str = "REFUSED_OCEL_IMPORT_INVALID";

/// Typed refusal when the foreign JSON cannot be parsed as OCEL.
pub const REFUSED_OCEL_IMPORT_MALFORMED: &str = "REFUSED_OCEL_IMPORT_MALFORMED";

/// An admitted foreign OCEL after federation import.
#[derive(Debug)]
pub struct AdmittedForeignOcel {
    /// The parsed, validated log.
    pub ocel: OCEL,
    /// The validate report (must be `valid: true`).
    pub report: ValidationReport,
}

fn is_broken_link(code: &str) -> bool {
    matches!(code, "DANGLING_E2O" | "DANGLING_O2O" | "E2O_EMPTY")
}

/// Import foreign OCEL JSON and admit it only when OCEL-v2 validate passes.
pub fn admit_foreign_ocel_json(json: &str) -> Result<AdmittedForeignOcel, MiningError> {
    let ocel = import_ocel_json(json).map_err(|e| {
        MiningError::OcelConversion(format!("{REFUSED_OCEL_IMPORT_MALFORMED}: {e}"))
    })?;
    let report = validate(&ocel, &HashMap::new());
    if report.valid {
        return Ok(AdmittedForeignOcel { ocel, report });
    }
    let broken = report.errors.iter().any(|e| is_broken_link(&e.code));
    let code = if broken {
        REFUSED_OCEL_IMPORT_BROKEN_LINK
    } else {
        REFUSED_OCEL_IMPORT_INVALID
    };
    let detail = report
        .errors
        .iter()
        .map(|e| format!("{}:{}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("; ");
    Err(MiningError::OcelConversion(format!("{code}: {detail}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal well-formed OCEL v2 JSON (declared types, one object, one event).
    fn honest_json() -> &'static str {
        r#"{
          "eventTypes": [{"name": "create", "attributes": []}],
          "objectTypes": [{"name": "order", "attributes": []}],
          "events": [{
            "id": "e1",
            "type": "create",
            "time": "2020-01-01T00:00:00Z",
            "attributes": [],
            "relationships": [{"objectId": "o1", "qualifier": "order"}]
          }],
          "objects": [{
            "id": "o1",
            "type": "order",
            "attributes": [],
            "relationships": []
          }]
        }"#
    }

    fn dangling_e2o_json() -> &'static str {
        r#"{
          "eventTypes": [{"name": "create", "attributes": []}],
          "objectTypes": [{"name": "order", "attributes": []}],
          "events": [{
            "id": "e1",
            "type": "create",
            "time": "2020-01-01T00:00:00Z",
            "attributes": [],
            "relationships": [{"objectId": "MISSING", "qualifier": "order"}]
          }],
          "objects": [{
            "id": "o1",
            "type": "order",
            "attributes": [],
            "relationships": []
          }]
        }"#
    }

    #[test]
    fn honest_foreign_json_admits() {
        let admitted = admit_foreign_ocel_json(honest_json()).expect("honest log must admit");
        assert!(admitted.report.valid);
        assert_eq!(admitted.ocel.events.len(), 1);
        assert_eq!(admitted.ocel.objects.len(), 1);
    }

    #[test]
    fn dangling_e2o_refuses_broken_link() {
        let err = admit_foreign_ocel_json(dangling_e2o_json()).expect_err("dangling must refuse");
        assert!(
            err.to_string().contains(REFUSED_OCEL_IMPORT_BROKEN_LINK),
            "got: {err}"
        );
    }

    #[test]
    fn malformed_json_refuses_typed() {
        let err = admit_foreign_ocel_json("{not-json").expect_err("parse must refuse");
        assert!(err.to_string().contains(REFUSED_OCEL_IMPORT_MALFORMED));
    }

    #[test]
    fn temper_falsifier_flips_accept_to_refuse() {
        assert!(admit_foreign_ocel_json(honest_json()).is_ok());
        assert!(admit_foreign_ocel_json(dangling_e2o_json()).is_err());
    }
}
