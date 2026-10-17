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
    /// Evidence validity ended before the caller-supplied observation time.
    ///
    /// Determinism law: expiry is never compared against a wall clock. The
    /// caller supplies `now`; identical inputs always yield identical refusals.
    EvidenceExpired,
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
            Self::EvidenceExpired => "evidence_expired",
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
            Self::InvalidPolicyDecisionPoint => {
                write!(f, "AuthZEN PDP must be an HTTPS identifier")
            }
            Self::PolicyDecisionPointMismatch => write!(f, "AuthZEN PDP identity mismatch"),
            Self::PrincipalMismatch => write!(f, "AuthZEN subject principal mismatch"),
            Self::EffectMismatch => write!(f, "AuthZEN resource/effect mismatch"),
            Self::EvidenceExpired => write!(f, "evidence expired before the observation time"),
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

/// Admit evidence validity at a caller-supplied observation time.
///
/// Deterministic: no wall-clock. `now` must be supplied by the caller
/// (e.g. carried in the observation itself). Evidence whose validity ends
/// strictly before `now` is refused with [`EvidenceError::EvidenceExpired`];
/// evidence valid exactly at `now` is admitted.
pub fn admit_evidence_validity(valid_until: u64, now: u64) -> Result<(), EvidenceError> {
    if valid_until < now {
        return Err(EvidenceError::EvidenceExpired);
    }
    Ok(())
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

    #[test]
    fn malformed_spiffe_prefixes_are_refused() {
        assert_eq!(
            SpiffeIdRef::parse("https://prod.acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeScheme)
        );
        assert_eq!(
            SpiffeIdRef::parse("prod.acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeScheme)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://"),
            Err(EvidenceError::MissingTrustDomain)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe:///workload"),
            Err(EvidenceError::MissingTrustDomain)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://user@prod.acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://prod.acme.com:8443"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://prod .acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://prod.acme.com/billing#fragment"),
            Err(EvidenceError::SpiffeQueryOrFragment)
        );
    }

    #[test]
    fn x509_admission_succeeds_and_every_typed_refusal_fires() {
        let identity = SpiffeIdRef::parse("spiffe://prod.acme.com/billing/api").unwrap();
        let base = WorkloadIdentityEvidenceRef {
            identity,
            svid_type: SvidType::X509,
            bundle_digest: "sha256:bundle",
            verified: true,
        };
        assert_eq!(
            admit_workload_identity(
                base,
                "spiffe://prod.acme.com/billing/api",
                "prod.acme.com",
                false
            ),
            Ok(())
        );

        let mut evidence = base;
        evidence.bundle_digest = "";
        assert_eq!(
            admit_workload_identity(evidence, identity.uri, identity.trust_domain, false),
            Err(EvidenceError::EmptyField("bundle_digest"))
        );

        let mut evidence = base;
        evidence.verified = false;
        assert_eq!(
            admit_workload_identity(evidence, identity.uri, identity.trust_domain, false),
            Err(EvidenceError::WorkloadNotVerified)
        );

        let mut evidence = base;
        evidence.identity = SpiffeIdRef::parse("spiffe://prod.acme.com/other").unwrap();
        assert_eq!(
            admit_workload_identity(evidence, identity.uri, identity.trust_domain, false),
            Err(EvidenceError::SpiffeIdMismatch)
        );

        let mut evidence = base;
        evidence.identity = SpiffeIdRef::parse("spiffe://staging.acme.com/billing/api").unwrap();
        assert_eq!(
            admit_workload_identity(evidence, evidence.identity.uri, "prod.acme.com", false),
            Err(EvidenceError::TrustDomainMismatch)
        );

        let mut evidence = base;
        evidence.svid_type = SvidType::Jwt;
        assert_eq!(
            admit_workload_identity(evidence, identity.uri, identity.trust_domain, false),
            Err(EvidenceError::JwtNotAdmitted)
        );
        assert_eq!(
            admit_workload_identity(evidence, identity.uri, identity.trust_domain, true),
            Ok(())
        );
    }

    #[test]
    fn authzen_missing_sarc_fields_are_refused_in_order() {
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

        {
            let mut request = request;
            request.subject.entity_type = "";
            assert_eq!(
                admit_authzen_evidence(request, decision, "https://pdp.example.com", "a", "123"),
                Err(EvidenceError::EmptyField("subject.type"))
            );
        }
        {
            let mut request = request;
            request.subject.id = "";
            assert_eq!(
                admit_authzen_evidence(request, decision, "https://pdp.example.com", "", "123"),
                Err(EvidenceError::EmptyField("subject.id"))
            );
        }
        {
            let mut request = request;
            request.action.name = "";
            assert_eq!(
                admit_authzen_evidence(request, decision, "https://pdp.example.com", "a", "123"),
                Err(EvidenceError::EmptyField("action.name"))
            );
        }
        {
            let mut request = request;
            request.resource.entity_type = "";
            assert_eq!(
                admit_authzen_evidence(request, decision, "https://pdp.example.com", "a", "123"),
                Err(EvidenceError::EmptyField("resource.type"))
            );
        }
        {
            let mut request = request;
            request.resource.id = "";
            assert_eq!(
                admit_authzen_evidence(request, decision, "https://pdp.example.com", "a", ""),
                Err(EvidenceError::EmptyField("resource.id"))
            );
        }
    }

    #[test]
    fn authzen_invalid_and_mismatched_surfaces_are_refused() {
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
        let decision = |pdp: &'static str| AuthZenDecisionEvidenceRef {
            decision: true,
            policy_decision_point: pdp,
        };

        assert_eq!(
            admit_authzen_evidence(
                request,
                decision("http://pdp.example.com"),
                "http://pdp.example.com",
                "alice@example.com",
                "123"
            ),
            Err(EvidenceError::InvalidPolicyDecisionPoint)
        );
        assert_eq!(
            admit_authzen_evidence(
                request,
                decision("https://"),
                "https://",
                "alice@example.com",
                "123"
            ),
            Err(EvidenceError::InvalidPolicyDecisionPoint)
        );
        assert_eq!(
            admit_authzen_evidence(
                request,
                decision("https://pdp.example.com?x=1"),
                "https://pdp.example.com?x=1",
                "alice@example.com",
                "123"
            ),
            Err(EvidenceError::InvalidPolicyDecisionPoint)
        );
        assert_eq!(
            admit_authzen_evidence(
                request,
                decision("https://pdp.example.com"),
                "https://pdp.example.com",
                "bob@example.com",
                "123"
            ),
            Err(EvidenceError::PrincipalMismatch)
        );
        assert_eq!(
            admit_authzen_evidence(
                request,
                decision("https://pdp.example.com"),
                "https://pdp.example.com",
                "alice@example.com",
                "456"
            ),
            Err(EvidenceError::EffectMismatch)
        );
    }

    #[test]
    fn expiry_is_deterministic_on_caller_supplied_now() {
        assert_eq!(admit_evidence_validity(100, 100), Ok(()));
        assert_eq!(
            admit_evidence_validity(100, 101),
            Err(EvidenceError::EvidenceExpired)
        );
        assert_eq!(admit_evidence_validity(100, 0), Ok(()));
        assert_eq!(EvidenceError::EvidenceExpired.code(), "evidence_expired");
    }
}

/// Boundary corpus (register row 78). Each cell is mutation-sensitive: the
/// comment above each assert names what silently breaks if the verdict flips.
#[cfg(test)]
mod boundary_corpus {
    use super::*;

    // --- SPIFFE parse strictness -------------------------------------------

    #[test]
    fn spiffe_scheme_is_case_sensitive_strict() {
        // Grammar pins exact-lowercase scheme via strip_prefix("spiffe://").
        // Mutation: a case-insensitive rewrite (eq_ignore_ascii_case) would
        // admit "SPIFFE://..." / "Spiffe://..." URIs, letting non-canonical
        // spellings collide with the exact-match binding in
        // admit_workload_identity — two spellings of "the same" identity
        // would both admit.
        assert_eq!(
            SpiffeIdRef::parse("SPIFFE://prod.acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeScheme)
        );
        assert_eq!(
            SpiffeIdRef::parse("Spiffe://prod.acme.com/billing"),
            Err(EvidenceError::InvalidSpiffeScheme)
        );
        // Deliberately permissive boundary, pinned by named reason: the trust
        // domain BODY is not case-folded (SPIFFE trust domains are DNS names
        // and DNS is case-preserving), so uppercase inside the authority
        // parses and round-trips byte-exact. Mutation: adding an
        // ascii-lowercase normalization here would break SpiffeIdRef::uri
        // byte-exactness and desync the exact
        // `identity.uri != expected_spiffe_id` comparison.
        let upper = SpiffeIdRef::parse("spiffe://PROD.acme.com/billing").unwrap();
        assert_eq!(
            upper.trust_domain, "PROD.acme.com",
            "reason: trust-domain body is case-preserving by grammar (no folding)"
        );
        assert_eq!(upper.uri, "spiffe://PROD.acme.com/billing");
    }

    #[test]
    fn spiffe_trailing_slash_is_a_nonempty_path_not_empty() {
        // Mutation: treating the trailing slash as a terminator that yields
        // path "" would make "spiffe://td.example/x/" and
        // "spiffe://td.example/x" indistinguishable, collapsing two distinct
        // SPIFFE IDs into one admission binding.
        let id = SpiffeIdRef::parse("spiffe://td.example/x/").unwrap();
        assert_eq!(id.trust_domain, "td.example");
        assert_eq!(
            id.path,
            "/x/",
            "reason: grammar keeps everything from the first '/' verbatim; the trailing slash is path content, distinct from no path"
        );
        // Deliberately permissive boundary, pinned by named reason: an empty
        // path segment ("//") is NOT refused — this grammar validates only
        // scheme, authority and query/fragment; per-segment path validation
        // is delegated to the upstream SPIFFE verifier. Mutation: adding
        // segment validation here would refuse identities a conforming
        // upstream verifier already accepted.
        let dbl = SpiffeIdRef::parse("spiffe://td.example//x").unwrap();
        assert_eq!(
            dbl.path, "//x",
            "reason: path passes through unvalidated by design"
        );
    }

    #[test]
    fn spiffe_empty_path_is_a_valid_root_identity() {
        // Mutation: refusing empty path would break admission of
        // trust-domain-level root identities; silently normalizing to "/"
        // would break uri byte-exactness against expected_spiffe_id.
        let id = SpiffeIdRef::parse("spiffe://td.example").unwrap();
        assert_eq!(id.trust_domain, "td.example");
        assert_eq!(
            id.path, "",
            "reason: no '/' in input; grammar yields empty path and preserves uri exactly"
        );
        assert_eq!(id.uri, "spiffe://td.example");
        let evidence = WorkloadIdentityEvidenceRef {
            identity: id,
            svid_type: SvidType::X509,
            bundle_digest: "sha256:b",
            verified: true,
        };
        assert_eq!(
            admit_workload_identity(evidence, "spiffe://td.example", "td.example", false),
            Ok(())
        );
    }

    #[test]
    fn spiffe_trust_domain_char_grammar_refuses_exactly_at_userinfo_port_ws() {
        // The grammar refuses ONLY '@', ':' and ASCII whitespace in the
        // authority. Deliberately permissive on everything else — pinned, not
        // guessed: SPIFFE delegates trust-domain character law to DNS and the
        // upstream verifier; this module certifies structure, not registrar
        // policy. Mutation: tightening to a DNS-label charset (e.g. refusing
        // '_' or uppercase) would reject identities a conforming upstream
        // verifier may produce; loosening (admitting ':') would let
        // port/userinfo-carrying authorities slip past
        // InvalidSpiffeAuthority.
        assert_eq!(
            SpiffeIdRef::parse("spiffe://under_score.example/w").map(|id| id.trust_domain),
            Ok("under_score.example"),
            "reason: '_' is outside the refused charset; grammar is deliberately permissive beyond @ : whitespace"
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://td%2eexample/w").map(|id| id.trust_domain),
            Ok("td%2eexample"),
            "reason: percent-encoding is not decoded or refused here; upstream verifier owns decoding"
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://bad\tchar/w"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
        assert_eq!(
            SpiffeIdRef::parse("spiffe://bad\nchar/w"),
            Err(EvidenceError::InvalidSpiffeAuthority)
        );
    }

    // --- Validity clock edges ----------------------------------------------

    #[test]
    fn validity_boundary_is_half_open_valid_until_equals_now_admits() {
        // Mutation: flipping `<` to `<=` in admit_evidence_validity would
        // refuse evidence still valid AT the observation instant — a
        // timestamp collision (integer clocks) would spuriously expire live
        // evidence.
        assert_eq!(admit_evidence_validity(100, 100), Ok(()));
    }

    #[test]
    fn validity_first_expired_tick_is_now_equals_valid_until_plus_one() {
        // Mutation: an off-by-one in the other direction (valid_until < now
        // becomes valid_until < now - 1, or the comparison is on the wrong
        // operand) would admit evidence expired exactly one tick ago.
        assert_eq!(
            admit_evidence_validity(100, 101),
            Err(EvidenceError::EvidenceExpired)
        );
    }

    #[test]
    fn validity_u64_near_wrap_extremes_stay_deterministic() {
        // Mutation: any interior arithmetic (e.g. `valid_until - now` or a
        // saturating add) instead of the direct comparison would panic in
        // debug on underflow or wrap silently in release at the u64 edges.
        // The direct `valid_until < now` comparison is total on u64: it
        // cannot overflow.
        assert_eq!(admit_evidence_validity(u64::MAX, u64::MAX), Ok(()));
        assert_eq!(
            admit_evidence_validity(u64::MAX, u64::MAX - 1),
            Ok(()),
            "reason: evidence expiring at u64::MAX is still valid one tick before the wrap"
        );
        assert_eq!(
            admit_evidence_validity(u64::MAX - 1, u64::MAX),
            Err(EvidenceError::EvidenceExpired)
        );
        assert_eq!(admit_evidence_validity(0, 0), Ok(()));
        assert_eq!(
            admit_evidence_validity(0, 1),
            Err(EvidenceError::EvidenceExpired)
        );
        assert_eq!(
            admit_evidence_validity(u64::MAX, 0),
            Ok(()),
            "reason: no wrap arithmetic exists; a caller-supplied now of 0 against far-future expiry admits"
        );
    }

    // --- AuthZEN field-omission refusals ------------------------------------

    #[test]
    fn authzen_entirely_omitted_sarc_fields_are_refused_field_by_field() {
        // Each cell omits (empties) exactly one wire field. Mutation: dropping
        // any of the five EmptyField gates in admit_authzen_evidence would
        // admit evidence with a structurally absent Subject/Action/Resource
        // field — an AuthZEN request that never round-tripped the SARC shape
        // would certify as bound evidence.
        let pdp = "https://pdp.example.com";
        let decision = AuthZenDecisionEvidenceRef {
            decision: true,
            policy_decision_point: pdp,
        };
        let base = AuthZenRequestRef {
            subject: AuthZenEntityRef {
                entity_type: "user",
                id: "alice",
            },
            action: AuthZenActionRef { name: "can_read" },
            resource: AuthZenEntityRef {
                entity_type: "account",
                id: "effect-1",
            },
        };
        let expect = |request: AuthZenRequestRef<'_>, field: &'static str| {
            assert_eq!(
                admit_authzen_evidence(request, decision, pdp, "alice", "effect-1"),
                Err(EvidenceError::EmptyField(field)),
                "omitting {field} must refuse with EmptyField(\"{field}\")"
            );
        };
        let mut r = base;
        r.subject.entity_type = "";
        expect(r, "subject.type");
        let mut r = base;
        r.subject.id = "";
        expect(r, "subject.id");
        let mut r = base;
        r.action.name = "";
        expect(r, "action.name");
        let mut r = base;
        r.resource.entity_type = "";
        expect(r, "resource.type");
        let mut r = base;
        r.resource.id = "";
        expect(r, "resource.id");

        // Omitted PDP identifier is refused even when the request is whole.
        // Mutation: reordering the PDP check after the principal/effect
        // comparisons, or dropping the `pdp.len() <= prefix` guard, would let
        // a bare "https://" observe-and-match an empty expected PDP.
        let empty_pdp = AuthZenDecisionEvidenceRef {
            decision: true,
            policy_decision_point: "",
        };
        assert_eq!(
            admit_authzen_evidence(base, empty_pdp, "", "alice", "effect-1"),
            Err(EvidenceError::InvalidPolicyDecisionPoint)
        );
    }
}
