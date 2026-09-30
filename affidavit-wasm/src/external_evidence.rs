//! JSON/WASM adapter for foreign evidence certification.
//!
//! The portable core owns the admission law. This module only decodes JSON,
//! invokes that law, and emits deterministic evidence receipts. It performs no
//! network I/O, no policy selection, no certificate issuance, and no DO.

use crate::abi::{err, field, obj, AbiError};
use affidavit_core::{
    admit_authzen_evidence, admit_workload_identity, AuthZenActionRef,
    AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef,
    SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD,
    CONSEQUENCE_EVIDENCE_ONLY,
};
use serde_json::{json, Map, Value};

fn evidence_err(error: EvidenceError) -> AbiError {
    err(error.code(), error.to_string())
}

fn required_map_str<'a>(
    value: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, AbiError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", format!("missing or empty string `{name}`")))
}

fn require_object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, AbiError> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| err("bad_field", format!("`{name}` must be an object")))
}

fn digest(value: &Value) -> Result<String, AbiError> {
    let bytes = serde_json::to_vec(value).map_err(|e| err("internal", e.to_string()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_optional_object(value: &Value, name: &str) -> Result<(), AbiError> {
    if let Some(v) = value.get(name) {
        if !v.is_object() {
            return Err(err("bad_field", format!("`{name}` must be an object")));
        }
    }
    Ok(())
}

/// Certify an AuthZEN Authorization API 1.0 observation as authority-free evidence.
pub(crate) fn op_certify_authzen_evidence(
    req: &Value,
) -> Result<Map<String, Value>, AbiError> {
    let request = field(req, "request")?;
    let subject = require_object(request, "subject")?;
    let resource = require_object(request, "resource")?;
    let action = require_object(request, "action")?;

    validate_optional_object(request, "context")?;
    if let Some(properties) = action.get("properties") {
        if !properties.is_object() {
            return Err(err(
                "bad_field",
                "`request.action.properties` must be an object",
            ));
        }
    }

    let subject_type = required_map_str(subject, "type")?;
    let subject_id = required_map_str(subject, "id")?;
    let resource_type = required_map_str(resource, "type")?;
    let resource_id = required_map_str(resource, "id")?;
    let action_name = required_map_str(action, "name")?;

    let decision = field(req, "decision")?
        .as_bool()
        .ok_or_else(|| err("bad_field", "`decision` must be a JSON boolean"))?;
    let pdp = field(req, "policy_decision_point")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", "`policy_decision_point` must be a non-empty string"))?;
    let expected_pdp = field(req, "expected_policy_decision_point")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            err(
                "bad_field",
                "`expected_policy_decision_point` must be a non-empty string",
            )
        })?;
    let expected_principal = field(req, "expected_principal")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", "`expected_principal` must be a non-empty string"))?;
    let expected_effect_digest = field(req, "expected_effect_digest")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            err(
                "bad_field",
                "`expected_effect_digest` must be a non-empty string",
            )
        })?;

    let sarc = AuthZenRequestRef {
        subject: AuthZenEntityRef {
            entity_type: subject_type,
            id: subject_id,
        },
        action: AuthZenActionRef { name: action_name },
        resource: AuthZenEntityRef {
            entity_type: resource_type,
            id: resource_id,
        },
    };
    let observed = AuthZenDecisionEvidenceRef {
        decision,
        policy_decision_point: pdp,
    };

    admit_authzen_evidence(
        sarc,
        observed,
        expected_pdp,
        expected_principal,
        expected_effect_digest,
    )
    .map_err(evidence_err)?;

    let evidence = json!({
        "kind": "authzen-policy-evidence",
        "standard": AUTHZEN_STANDARD,
        "request": request,
        "decision": decision,
        "policy_decision_point": pdp,
        "authority": AUTHORITY_NONE,
        "consequence": CONSEQUENCE_EVIDENCE_ONLY,
    });
    let evidence_digest = digest(&evidence)?;

    Ok(obj(json!({
        "certified": true,
        "standard": AUTHZEN_STANDARD,
        "policy_allows": decision,
        "policy_decision_point": pdp,
        "principal": subject_id,
        "effect_digest": resource_id,
        "action": action_name,
        "authority": AUTHORITY_NONE,
        "consequence": CONSEQUENCE_EVIDENCE_ONLY,
        "evidence_digest": evidence_digest,
        "evidence": evidence,
    })))
}

/// Certify verifier-produced SPIFFE/SVID workload evidence.
pub(crate) fn op_certify_spiffe_evidence(
    req: &Value,
) -> Result<Map<String, Value>, AbiError> {
    let spiffe_id = field(req, "spiffe_id")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", "`spiffe_id` must be a non-empty string"))?;
    let expected_spiffe_id = field(req, "expected_spiffe_id")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", "`expected_spiffe_id` must be a non-empty string"))?;
    let expected_trust_domain = field(req, "expected_trust_domain")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            err(
                "bad_field",
                "`expected_trust_domain` must be a non-empty string",
            )
        })?;
    let bundle_digest = field(req, "bundle_digest")?
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| err("bad_field", "`bundle_digest` must be a non-empty string"))?;
    let verified = field(req, "verified")?
        .as_bool()
        .ok_or_else(|| err("bad_field", "`verified` must be a JSON boolean"))?;
    let svid_type_raw = field(req, "svid_type")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`svid_type` must be \"x509\" or \"jwt\""))?;
    let svid_type = match svid_type_raw {
        "x509" => SvidType::X509,
        "jwt" => SvidType::Jwt,
        _ => {
            return Err(err(
                "bad_field",
                "`svid_type` must be \"x509\" or \"jwt\"",
            ))
        }
    };
    let allow_jwt = match req.get("allow_jwt") {
        None => false,
        Some(value) => value
            .as_bool()
            .ok_or_else(|| err("bad_field", "`allow_jwt` must be a JSON boolean"))?,
    };

    let identity = SpiffeIdRef::parse(spiffe_id).map_err(evidence_err)?;
    let evidence_ref = WorkloadIdentityEvidenceRef {
        identity,
        svid_type,
        bundle_digest,
        verified,
    };
    admit_workload_identity(
        evidence_ref,
        expected_spiffe_id,
        expected_trust_domain,
        allow_jwt,
    )
    .map_err(evidence_err)?;

    let evidence = json!({
        "kind": "spiffe-workload-identity-evidence",
        "standard": "SPIFFE",
        "spiffe_id": identity.uri,
        "trust_domain": identity.trust_domain,
        "path": identity.path,
        "svid_type": svid_type_raw,
        "bundle_digest": bundle_digest,
        "verified": verified,
        "authority": AUTHORITY_NONE,
        "consequence": CONSEQUENCE_EVIDENCE_ONLY,
    });
    let evidence_digest = digest(&evidence)?;

    Ok(obj(json!({
        "certified": true,
        "standard": "SPIFFE",
        "spiffe_id": identity.uri,
        "trust_domain": identity.trust_domain,
        "path": identity.path,
        "svid_type": svid_type_raw,
        "authority": AUTHORITY_NONE,
        "consequence": CONSEQUENCE_EVIDENCE_ONLY,
        "evidence_digest": evidence_digest,
        "evidence": evidence,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(value: Value) -> Value {
        let raw = serde_json::to_vec(&value).unwrap();
        serde_json::from_slice(&crate::abi::call(&raw)).unwrap()
    }

    #[test]
    fn authzen_figure_14_is_certified_as_evidence_only() {
        let request = json!({
            "subject": {"type": "user", "id": "alice@example.com"},
            "resource": {"type": "account", "id": "123"},
            "action": {"name": "can_read", "properties": {"method": "GET"}},
            "context": {"time": "1985-10-26T01:22-07:00"}
        });
        let response = run(json!({
            "op": "certify_authzen_evidence",
            "request": request,
            "decision": true,
            "policy_decision_point": "https://pdp.example.com",
            "expected_policy_decision_point": "https://pdp.example.com",
            "expected_principal": "alice@example.com",
            "expected_effect_digest": "123"
        }));
        assert_eq!(response["ok"], true, "{response}");
        assert_eq!(response["certified"], true);
        assert_eq!(response["policy_allows"], true);
        assert_eq!(response["authority"], "NONE");
        assert_eq!(response["consequence"], "EVIDENCE_ONLY");
        assert_eq!(response["action"], "can_read");
    }

    #[test]
    fn authzen_allow_cannot_cross_principal_effect_or_pdp_fences() {
        let request = json!({
            "subject": {"type": "user", "id": "alice@example.com"},
            "resource": {"type": "account", "id": "123"},
            "action": {"name": "can_read"}
        });
        for (field_name, value, code) in [
            ("expected_principal", "mallory@example.com", "principal_mismatch"),
            ("expected_effect_digest", "999", "effect_digest_mismatch"),
            ("expected_policy_decision_point", "https://other.example.com", "pdp_mixup"),
        ] {
            let mut input = json!({
                "op": "certify_authzen_evidence",
                "request": request,
                "decision": true,
                "policy_decision_point": "https://pdp.example.com",
                "expected_policy_decision_point": "https://pdp.example.com",
                "expected_principal": "alice@example.com",
                "expected_effect_digest": "123"
            });
            input[field_name] = json!(value);
            let response = run(input);
            assert_eq!(response["ok"], false, "{response}");
            assert_eq!(response["error"]["code"], code, "{response}");
        }
    }

    #[test]
    fn canonical_spiffe_example_is_certified_but_jwt_is_default_refused() {
        let base = json!({
            "op": "certify_spiffe_evidence",
            "spiffe_id": "spiffe://prod.acme.com/billing/api",
            "expected_spiffe_id": "spiffe://prod.acme.com/billing/api",
            "expected_trust_domain": "prod.acme.com",
            "svid_type": "x509",
            "bundle_digest": "sha256:canonical-example-bundle",
            "verified": true
        });
        let response = run(base.clone());
        assert_eq!(response["ok"], true, "{response}");
        assert_eq!(response["trust_domain"], "prod.acme.com");
        assert_eq!(response["path"], "/billing/api");
        assert_eq!(response["authority"], "NONE");

        let mut jwt = base;
        jwt["svid_type"] = json!("jwt");
        let response = run(jwt);
        assert_eq!(response["ok"], false, "{response}");
        assert_eq!(response["error"]["code"], "jwt_not_admitted");
    }
}
