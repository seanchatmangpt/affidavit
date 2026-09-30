//! Foreign authorization/identity evidence law.
//!
//! This module owns the *certification boundary* for evidence imported from
//! AuthZEN and SPIFFE. It deliberately performs no network I/O, policy choice,
//! certificate issuance, or actuation. A successful admission means only that
//! the observed evidence is structurally valid and bound to the exact expected
//! subject. It never grants authority.
//!
//! `AuthZEN allow != authority != DO` and `SPIFFE identity != authority`.

use core::fmt;

/// Standing emitted for all foreign evidence admitted by this module.
pub const AUTHORITY_NONE: &str = "NONE";

/// Consequence ceiling emitted for all foreign evidence admitted by this module.
pub const CONSEQUENCE_EVIDENCE_ONLY: &str = "EVIDENCE_ONLY";

/// Canonical standard identity for the supported AuthZEN wire contract.
pub const AUTHZEN_STANDARD: &str = "OpenID AuthZEN Authorization API 1.0";

/// Canonical scheme prefix for SPIFFE workload identities.
pub const SPIFFE_PREFIX: &str = "spiffe://";

/// Typed refusal for AuthZEN/SPIFFE evidence admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceError {
    /// A required value is empty.
    EmptyField(&'static str),
    /// The SPIFFE URI does not use the `spiffe://` scheme.
    InvalidSpiffeScheme,
    /// The SPIFFE URI has no trust domain.
    MissingTrustDomain,
    /// The SPIFFE authority contains user-info, a port, whitespace, or another
    /// form that is not admitted as a trust-domain identity.
    InvalidSpiffeAuthority,
    /// SPIFFE IDs cannot contain URI query or fragment components.
    SpiffeQueryOrFragment,
    /// The observed SPIFFE ID differs from the exact expected ID.
    SpiffeIdMismatch,
    /// The observed SPIFFE trust domain differs from the expected domain.
    TrustDomainMismatch,
    /// The workload evidence was not produced by a successful upstream SVID verifier.
    WorkloadNotVerified,
    /// JWT-SVID evidence is refused unless the caller explicitly admits it.
    JwtNotAdmitted,
    /// The AuthZEN PDP identifier is not an HTTPS origin-like identifier.
    InvalidPolicyDecisionPoint,
    /// The observed PDP differs from the exact expected PDP.
    PolicyDecisionPointMismatch,
    /// The AuthZEN subject does not bind the exact expected principal.
    PrincipalMismatch,
    /// The AuthZEN resource does not bind the exact expected effect.
    EffectMismatch,
}

impl EvidenceError {
    /// Stable machine-readable refusal code used by portable adapters.
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField(_) => "empty_field",
            Self::InvalidSpiffeScheme => "invalid_spiffe_scheme",
            Self::MissingTrustDomain => "missing_trust_domain",
            Self::InvalidSpiffeAuthority => "invalid_spiffe_authority",
            Self::SpiffeQueryOrFragment => "spiffe_query_or_fragment",
            Self::SpiffeIdMismatch => "spiffe_id_mismatch",
            Self::TrustDomainMismatch => "trust_domain_mismatch",
            Self::WorkloadNotVerified => "workload_not_verified",
            Self::JwtNotAdmitted => "jwt_not_admitted",
            Self::InvalidPolicyDecisionPoint => "invalid_policy_decision_point",
            Self::PolicyDecisionPointMismatch => "pdp_mixup",
            Self::PrincipalMismatch => "principal_mismatch",
            Self::EffectMismatch => "effect_digest_mismatch",
        }
    }
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(name) => write!(f, "required field `{name}` is empty"),
            Self::InvalidSpiffeScheme => write!(f, "SPIFFE ID must use spiffe://"),
            Self::MissingTrustDomain => write!(f, "SPIFFE ID is missing a trust domain"),
            Self::InvalidSpiffeAuthority => write!(f, "SPIFFE trust-domain authority is invalid"),
            Self::SpiffeQueryOrFragment => write!(f, "SPIFFE ID cannot contain query or fragment"),
            Self::SpiffeIdMismatch => write!(f, "SPIFFE ID does not match the admitted identity"),
            Self::TrustDomainMismatch => write!(f, "SPIFFE trust domain does not match"),
            Self::WorkloadNotVerified => write!(f, "workload identity was not verified upstream"),
            Self::JwtNotAdmitted => write!(f, "JWT-SVID evidence requires explicit admission"),
            Self::InvalidPolicyDecisionPoint => write!(f, "AuthZEN PDP must be an HTTPS identifier"),
            Self::PolicyDecisionPointMismatch => write!(f, "AuthZEN PDP identity mismatch"),
            Self::PrincipalMismatch => write!(f, "AuthZEN subject principal mismatch"),
            Self::EffectMismatch => write!(f, "AuthZEN resource/effect mismatch"),
        }
    }
}

/// Borrowed, parsed SPIFFE workload identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpiffeIdRef<'a> {
    /// Exact input identity.
    pub uri: &'a str,
    /// Trust domain from the URI authority.
    pub trust_domain: &'a str,
    /// Workload path, including the leading slash, or the empty string.
    pub path: &'a str,
}

impl<'a> SpiffeIdRef<'a> {
    /// Parse a SPIFFE ID without allocation.
    pub fn parse(raw: &'a str) -> Result<Self, EvidenceError> {
        let rest = raw
            .strip_prefix(SPIFFE_PREFIX)
            .ok_or(EvidenceError::InvalidSpiffeScheme)?;
        if rest.contains('?') || rest.contains('#') {
            return Err(EvidenceError::SpiffeQueryOrFragment);
        }
        let (authority, path) = match rest.find('/') {
            Some(index) => (&rest[..index], &rest[index..]),
            None => (rest, ""),
        };
        if authority.is_empty() {
            return Err(EvidenceError::MissingTrustDomain);
        }
        if authority.contains('@')
            || authority.contains(':')
            || authority.bytes().any(|b| b.is_ascii_whitespace())
        {
            return Err(EvidenceError::InvalidSpiffeAuthority);
        }
        Ok(Self {
            uri: raw,
            trust_domain: authority,
            path,
        })
    }
}

/// Supported SVID evidence kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvidType {
    /// X.509-SVID evidence.
    X509,
    /// JWT-SVID evidence.
    Jwt,
}

/// Borrowed evidence produced by an upstream SPIFFE/SVID verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkloadIdentityEvidenceRef<'a> {
    /// Exact SPIFFE identity.
    pub identity: SpiffeIdRef<'a>,
    /// SVID evidence kind.
    pub svid_type: SvidType,
    /// Digest/identity of the trust bundle used by the upstream verifier.
    pub bundle_digest: &'a str,
    /// Whether the upstream verifier reported successful verification.
    pub verified: bool,
}

/// Admit workload identity evidence against an exact expected identity.
///
/// Success certifies identity evidence only. It never grants authorization or
/// actuation authority.
pub fn admit_workload_identity(
    evidence: WorkloadIdentityEvidenceRef<'_>,
    expected_spiffe_id: &str,
    expected_trust_domain: &str,
    allow_jwt: bool,
) -> Result<(), EvidenceError> {
    if evidence.bundle_digest.is_empty() {
        return Err(EvidenceError::EmptyField("bundle_digest"));
    }
    if !evidence.verified {
        return Err(EvidenceError::WorkloadNotVerified);
    }
    if evidence.identity.uri != expected_spiffe_id {
        return Err(EvidenceError::SpiffeIdMismatch);
    }
    if evidence.identity.trust_domain != expected_trust_domain {
        return Err(EvidenceError::TrustDomainMismatch);
    }
    if evidence.svid_type == SvidType::Jwt && !allow_jwt {
        return Err(EvidenceError::JwtNotAdmitted);
    }
    Ok(())
}

/// Borrowed AuthZEN Subject or Resource entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthZenEntityRef<'a> {
    /// AuthZEN entity type.
    pub entity_type: &'a str,
    /// AuthZEN entity identifier.
    pub id: &'a str,
}

/// Borrowed AuthZEN Action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthZenActionRef<'a> {
    /// AuthZEN 1.0 Action `name`.
    pub name: &'a str,
}

/// Borrowed canonical AuthZEN SARC request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthZenRequestRef<'a> {
    /// Subject.
    pub subject: AuthZenEntityRef<'a>,
    /// Action.
    pub action: AuthZenActionRef<'a>,
    /// Resource.
    pub resource: AuthZenEntityRef<'a>,
}

/// Borrowed external AuthZEN policy decision evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthZenDecisionEvidenceRef<'a> {
    /// Strict boolean decision returned by the PDP.
    pub decision: bool,
    /// Exact policy decision point identifier that produced the observation.
    pub policy_decision_point: &'a str,
}

/// Admit AuthZEN evidence against an exact principal, effect and PDP.
///
/// Both allow and deny decisions can be valid evidence. A successful return
/// therefore does **not** mean "authorized"; it means only that the evidence
/// binds the exact expected request subject.
pub fn admit_authzen_evidence(
    request: AuthZenRequestRef<'_>,
    decision: AuthZenDecisionEvidenceRef<'_>,
    expected_policy_decision_point: &str,
    expected_principal: &str,
    expected_effect_digest: &str,
) -> Result<(), EvidenceError> {
    for (name, value) in [
        ("subject.type", request.subject.entity_type),
        ("subject.id", request.subject.id),
        ("action.name", request.action.name),
        ("resource.type", request.resource.entity_type),
        ("resource.id", request.resource.id),
    ] {
        if value.is_empty() {
            return Err(EvidenceError::EmptyField(name));
        }
    }

    let pdp = decision.policy_decision_point;
    if !pdp.starts_with("https://")
        || pdp.len() <= "https://".len()
        || pdp.contains('?')
        || pdp.contains('#')
    {
        return Err(EvidenceError::InvalidPolicyDecisionPoint);
    }
    if pdp != expected_policy_decision_point {
        return Err(EvidenceError::PolicyDecisionPointMismatch);
    }
    if request.subject.id != expected_principal {
        return Err(EvidenceError::PrincipalMismatch);
    }
    if request.resource.id != expected_effect_digest {
        return Err(EvidenceError::EffectMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_spiffe_example_is_exactly_parsed() {
        let id = SpiffeIdRef::parse("spiffe://prod.acme.com/billing/api").unwrap();
        assert_eq!(id.trust_domain, "prod.acme.com");
        assert_eq!(id.path, "/billing/api");
        assert_eq!(id.uri, "spiffe://prod.acme.com/billing/api");
    }

    #[test]
    fn spiffe_query_port_and_jwt_default_are_refused() {
        assert_eq!(
            SpiffeIdRef::parse("spiffe://prod.acme.com/billing/api?role=admin"),
            Err(EvidenceError::SpiffeQueryOrFragment)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://prod.acme.com:8443/billing/api"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
        let identity = SpiffeIdRef::parse("spiffe://prod.acme.com/billing/api").unwrap();
        let evidence = WorkloadIdentityEvidenceRef {
            identity,
            svid_type: SvidType::Jwt,
            bundle_digest: "sha256:bundle",
            verified: true,
        };
        assert_eq!(
            admit_workload_identity(
                evidence,
                "spiffe://prod.acme.com/billing/api",
                "prod.acme.com",
                false,
            ),
            Err(EvidenceError::JwtNotAdmitted)
        );
    }

    #[test]
    fn authzen_figure_14_shape_binds_but_does_not_mean_authority() {
        let request = AuthZenRequestRef {
            subject: AuthZenEntityRef {
                entity_type: "user",
                id: "alice@example.com",
            },
            action: AuthZenActionRef { name: "can_read" },
            resource: AuthZenEntityRef {
                entity_type: "account",
                id: "123",
            },
        };
        let decision = AuthZenDecisionEvidenceRef {
            decision: true,
            policy_decision_point: "https://pdp.example.com",
        };
        assert_eq!(
            admit_authzen_evidence(
                request,
                decision,
                "https://pdp.example.com",
                "alice@example.com",
                "123",
            ),
            Ok(())
        );
        assert_eq!(AUTHORITY_NONE, "NONE");
        assert_eq!(CONSEQUENCE_EVIDENCE_ONLY, "EVIDENCE_ONLY");
    }

    #[test]
    fn deny_is_still_valid_evidence_and_mixups_are_refused() {
        let request = AuthZenRequestRef {
            subject: AuthZenEntityRef {
                entity_type: "user",
                id: "alice@example.com",
            },
            action: AuthZenActionRef { name: "can_read" },
            resource: AuthZenEntityRef {
                entity_type: "account",
                id: "123",
            },
        };
        let deny = AuthZenDecisionEvidenceRef {
            decision: false,
            policy_decision_point: "https://pdp.example.com",
        };
        assert_eq!(
            admit_authzen_evidence(
                request,
                deny,
                "https://pdp.example.com",
                "alice@example.com",
                "123",
            ),
            Ok(())
        );
        assert_eq!(
            admit_authzen_evidence(
                request,
                deny,
                "https://other.example.com",
                "alice@example.com",
                "123",
            ),
            Err(EvidenceError::PolicyDecisionPointMismatch)
        );
    }
}
