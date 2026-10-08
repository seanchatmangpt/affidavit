//! Decidable authority-policy gate (AWS Cedar).
//!
//! Cedar evaluates whether a receipt chain's declared authority satisfies a
//! policy set — lease or work-order preconditions — under a formal semantics
//! proven in Lean, with strictly decidable, bounded-time evaluation. No
//! Turing-complete policy loops: the gate is O(size of policy set), always
//! terminates, always returns Allow or Deny (typed, with diagnostics).
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! The gate decides *policy membership*, mechanically. It never decides
//! whether a policy is *just* — policies are operator-supplied and
//! version-controlled inputs.

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Response,
};
use thiserror::Error;

/// Errors produced by the policy gate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PolicyError {
    /// The policy set failed to parse.
    #[error("policy set parse failed: {0}")]
    Parse(String),
    /// The authorization request was malformed for the policy set.
    #[error("policy request construction failed: {0}")]
    Request(String),
    /// The authorizer returned no decision (should be impossible in
    /// non-partial mode; refused, not defaulted).
    #[error("policy evaluation returned no decision")]
    NoDecision,
}

/// A decidable authority gate over an operator-supplied Cedar policy set.
#[derive(Debug)]
pub struct PolicyGate {
    authorizer: Authorizer,
    policies: PolicySet,
    entities: Entities,
}

/// The gate's verdict: admissible or refused, with the raw Cedar response
/// attached for the receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdict {
    /// The request is admitted by policy.
    Admitted,
    /// The request is refused by policy.
    Refused(String),
}

impl PolicyGate {
    /// Compile an operator-supplied Cedar policy set.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::Parse`] if the policy text is malformed.
    pub fn from_policies(policies: &str) -> Result<Self, PolicyError> {
        let parsed: PolicySet = policies
            .parse()
            .map_err(|e: cedar_policy::ParseErrors| PolicyError::Parse(e.to_string()))?;
        Ok(Self {
            authorizer: Authorizer::new(),
            policies: parsed,
            entities: Entities::empty(),
        })
    }

    /// Evaluate one authority request: may `principal` perform `action` on
    /// `resource`?
    ///
    /// Entity UID strings use Cedar notation, e.g. `Agent::"agent-7"`,
    /// `Action::"emit"`, `Receipt::"chain-1"`.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::Request`] if an entity UID or the request is
    /// malformed.
    pub fn evaluate(
        &self,
        principal: &str,
        action: &str,
        resource: &str,
    ) -> Result<GateVerdict, PolicyError> {
        let principal: EntityUid = principal
            .parse()
            .map_err(|e: cedar_policy::ParseErrors| PolicyError::Request(e.to_string()))?;
        let action: EntityUid = action
            .parse()
            .map_err(|e: cedar_policy::ParseErrors| PolicyError::Request(e.to_string()))?;
        let resource: EntityUid = resource
            .parse()
            .map_err(|e: cedar_policy::ParseErrors| PolicyError::Request(e.to_string()))?;
        let request = Request::new(principal, action, resource, Context::empty(), None)
            .map_err(|e| PolicyError::Request(e.to_string()))?;
        let response: Response =
            self.authorizer
                .is_authorized(&request, &self.policies, &self.entities);
        match response.decision() {
            Decision::Allow => Ok(GateVerdict::Admitted),
            Decision::Deny => Ok(GateVerdict::Refused(
                response
                    .diagnostics()
                    .errors()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICIES: &str = r#"
        permit(principal == Agent::"curator", action == Action::"emit", resource == Receipt::"chain-1");
        permit(principal, action == Action::"read", resource);
        forbid(principal == Agent::"suspended", action, resource);
    "#;

    #[test]
    fn permitted_authority_admits() {
        let gate = PolicyGate::from_policies(POLICIES).expect("policies");
        assert_eq!(
            gate.evaluate(
                r#"Agent::"curator""#,
                r#"Action::"emit""#,
                r#"Receipt::"chain-1""#
            )
            .expect("eval"),
            GateVerdict::Admitted
        );
    }

    #[test]
    fn unpermitted_action_refuses() {
        let gate = PolicyGate::from_policies(POLICIES).expect("policies");
        match gate.evaluate(
            r#"Agent::"curator""#,
            r#"Action::"actuate""#,
            r#"Receipt::"chain-1""#,
        ) {
            Ok(GateVerdict::Refused(_)) => {}
            other => panic!("expected refusal, got {other:?}"),
        }
    }

    #[test]
    fn forbid_outranks_permit() {
        let gate = PolicyGate::from_policies(POLICIES).expect("policies");
        match gate.evaluate(
            r#"Agent::"suspended""#,
            r#"Action::"read""#,
            r#"Receipt::"chain-1""#,
        ) {
            Ok(GateVerdict::Refused(_)) => {}
            other => panic!("expected refusal (forbid), got {other:?}"),
        }
    }

    #[test]
    fn malformed_policy_is_typed_parse_refusal() {
        assert!(matches!(
            PolicyGate::from_policies("permit(broken"),
            Err(PolicyError::Parse(_))
        ));
    }

    #[test]
    fn malformed_entity_uid_is_typed_request_refusal() {
        let gate = PolicyGate::from_policies(POLICIES).expect("policies");
        assert!(matches!(
            gate.evaluate("not-a-uid", r#"Action::"read""#, r#"Receipt::"chain-1""#),
            Err(PolicyError::Request(_))
        ));
    }
}
