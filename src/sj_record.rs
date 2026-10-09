//! SJ-aligned campaign record: promotes the campaign record from O (markdown
//! statement) to R (schema-valid receipt) under
//! `schemas/dfcm-receipt.schema.json`, per
//! `docs/sjira/v26.10.8/SJ-ALIGNED-RECORD-DESIGN.md`.
//!
//! The four faces a markdown record cannot carry, and the reason for this
//! module (design §0/§4):
//!
//! 1. subject identity — digest-bound, not name-bound
//!    ([`SjRecord::subject_digest_hex`]);
//! 2. replayable commands with real `exit`/`cwd`;
//! 3. chain-head binding — the BLAKE3 chain head the record binds to;
//! 4. predecessor work-order edges.
//!
//! Chain law (design §1): the campaign's commits, court witnesses, and
//! residue declaration serialize as chain events sealed by
//! [`crate::chain::ChainAssembler::finalize`]; there is no path to an
//! [`SjRecord`] that skips chain re-verification — a tampered base cannot
//! become a record ([`SjRecord::from_json`]).
//!
//! JCS law (design §2): the record document serializes as JCS (RFC 8785)
//! canonical JSON; the subject digest is
//! `BLAKE3(domain_separated(DOMAIN_TAG, [content_address_hex]))` — exactly
//! [`crate::crypto_trust_seal::subject_digest_of`].
//!
//! Certify-don't-decide: this module refuses malformed records with typed
//! refusals and re-derives digests from bytes; it never decides honesty or
//! authorization.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::chain::{recompute_chain, ChainAssembler};
use crate::crypto_trust_canonical::{jcs, CanonicalError};
use crate::crypto_trust_seal::subject_digest_of;
use crate::types::{Blake3Hash, ObjectRef, OperationEvent, Receipt};

/// Domain prefix of the campaign subject string (versioned: a change to the
/// canonical layout is a new domain, never a silent rewrite).
pub const CAMPAIGN_DOMAIN: &str = "affidavit-campaign/v1";

/// Event type for one campaign commit record.
pub const EVENT_COMMIT: &str = "campaign-commit";
/// Event type for one court witness record.
pub const EVENT_COURT: &str = "campaign-court-witness";
/// Event type for the residue declaration.
pub const EVENT_RESIDUE: &str = "campaign-residue";

// ---------------------------------------------------------------------------
// Faces (schema v2 shapes)
// ---------------------------------------------------------------------------

/// Authority ceiling vocabulary of the schema (`origin_authority.ceiling`,
/// `authority.ceiling`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityCeiling {
    /// Observation only.
    Observe,
    /// Selection authority.
    Select,
    /// Construction authority.
    Construct,
    /// Actuation authority.
    Do,
}

impl AuthorityCeiling {
    /// Schema spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            AuthorityCeiling::Observe => "OBSERVE",
            AuthorityCeiling::Select => "SELECT",
            AuthorityCeiling::Construct => "CONSTRUCT",
            AuthorityCeiling::Do => "DO",
        }
    }
}

/// Standing vocabulary of the schema (`standing.value` pattern).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandingValue {
    /// No witnessed execution.
    Unknown,
    /// Some witnessed execution, not exact-subject.
    PartialAlive,
    /// Observed execution on the exact admitted subject.
    Alive,
    /// Blocked, optionally with a reason suffix (`BLOCKED:...`).
    Blocked(#[serde(default)] Option<String>),
    /// The build is broken at the pinned subject.
    BuildBroken,
    /// Capability not supported, optionally with a family tag
    /// (`UNSUPPORTED(...)`).
    Unsupported(#[serde(default)] Option<String>),
    /// Typed refusal with its reason (`REFUSED(...)`); the reason is the
    /// `broken_term` carrier, never prose.
    Refused(String),
}

impl StandingValue {
    /// Schema spelling.
    pub fn as_str(&self) -> String {
        match self {
            StandingValue::Unknown => "UNKNOWN".to_string(),
            StandingValue::PartialAlive => "PARTIAL_ALIVE".to_string(),
            StandingValue::Alive => "ALIVE".to_string(),
            StandingValue::Blocked(None) => "BLOCKED".to_string(),
            StandingValue::Blocked(Some(r)) => format!("BLOCKED:{r}"),
            StandingValue::BuildBroken => "BUILD_BROKEN".to_string(),
            StandingValue::Unsupported(None) => "UNSUPPORTED".to_string(),
            StandingValue::Unsupported(Some(f)) => format!("UNSUPPORTED({f})"),
            StandingValue::Refused(r) => format!("REFUSED({r})"),
        }
    }

    /// True for the values the schema's `allOf` conditional binds to a
    /// required `broken_term`.
    pub fn requires_broken_term(&self) -> bool {
        matches!(
            self,
            StandingValue::Blocked(_) | StandingValue::BuildBroken | StandingValue::Refused(_)
        )
    }
}

/// The nine failure-taxonomy terms of the schema's `broken_term` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokenTerm {
    /// Manufacture from unadmitted input.
    MuOnO,
    /// The admission gate never refuses.
    AdmissionVacuous,
    /// Manufacture destroyed admitted capability.
    MuUnlawful,
    /// Receipt missing identity.
    RMissingIdentity,
    /// Receipt missing authority.
    RMissingAuthority,
    /// Receipt missing consequence.
    RMissingConsequence,
    /// Receipt missing replay.
    RMissingReplay,
    /// Receipt missing standing.
    RMissingStanding,
    /// Receipt not fed back into the frontier.
    RNotFedBack,
}

/// `origin_authority` face: the authority the work order originated under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginAuthority {
    /// Optional ceiling at origination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<AuthorityCeiling>,
    /// Lease id, user decision reference, or `NONE`.
    pub grant: String,
    /// The originating actor.
    pub actor: String,
}

/// `provider` face: execution provider identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    /// Provider identity; rewrites against one `provider_execution_id` are an
    /// identity falsifier.
    pub name: String,
    /// Optional transport spelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
}

/// `identity` face.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    /// Work order id or artifact name.
    pub subject: String,
    /// Repository the `subject_sha` must resolve in.
    pub repo: String,
    /// Git commit anchor (40-hex); never a non-commit digest.
    pub subject_sha: String,
    /// Base commit anchor (40-hex).
    pub base_sha: String,
    /// The non-commit digest face: BLAKE3 over the chain-sealed base's
    /// content address (design §2).
    pub subject_digest: SubjectDigest,
}

/// `identity.subject_digest` — `{algorithm: "blake3", value: <64 hex>}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectDigest {
    /// Digest algorithm; this module mints `blake3` only.
    pub algorithm: String,
    /// Lowercase hex digest.
    pub value: String,
}

/// `authority` face: the ceiling actually executed under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authority {
    /// Executed ceiling.
    pub ceiling: AuthorityCeiling,
    /// Lease id, user decision reference, or `NONE`.
    pub grant: String,
    /// Executing actor.
    pub actor: String,
}

/// `consequence` face.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consequence {
    /// Commit shas produced (7..40 hex each).
    pub commits: Vec<String>,
    /// Files changed.
    pub files_changed: Vec<String>,
    /// Remote effects (pushes, issues, deploys).
    pub remote_effects: Vec<String>,
}

/// One replay command — schema `replay.commands[]` shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCommand {
    /// The command string actually executed.
    pub cmd: String,
    /// Real exit code.
    pub exit: i32,
    /// Working directory it ran in.
    pub cwd: String,
    /// Optional summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Optional sha256 of the output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_sha256: Option<String>,
}

/// `replay` face.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    /// The court/gate commands actually executed (minItems 1).
    pub commands: Vec<ReplayCommand>,
    /// Git-tracked durable location — never gitignored tmp (design §4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durable_location: Option<String>,
}

/// `replay_binding` face: causal DAG edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBinding {
    /// Chain event ids this record claims (unreceipted actuation law).
    pub event_ids: Vec<String>,
    /// The assembled BLAKE3 chain head this record binds to.
    pub chain_head_hash: String,
    /// Prior receipts this one extends.
    pub predecessor_work_order_ids: Vec<String>,
}

/// `standing` face.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    /// Standing vocabulary value.
    pub value: StandingValue,
    /// Which replay commands at which subject_sha justify the value.
    pub derived_from: String,
    /// Required for BLOCKED / BUILD_BROKEN / REFUSED (schema allOf).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broken_term: Option<BrokenTerm>,
}

/// The machine surface of the campaign record — the R's document faces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SjRecordDocument {
    /// Work order id this execution discharged.
    pub work_order_id: String,
    /// Authority the order originated under.
    pub origin_authority: OriginAuthority,
    /// Execution provider identity.
    pub provider: Provider,
    /// Provider-side execution id (joins the OCEL event chain).
    pub provider_execution_id: String,
    /// Subject identity (commit anchor + non-commit digest).
    pub identity: Identity,
    /// Authority actually executed under.
    pub authority: Authority,
    /// Consequence face.
    pub consequence: Consequence,
    /// Replay face.
    pub replay: Replay,
    /// Chain/DAG binding face.
    pub replay_binding: ReplayBinding,
    /// Standing face.
    pub standing: Standing,
}

// ---------------------------------------------------------------------------
// Typed refusals
// ---------------------------------------------------------------------------

/// Typed refusals of the sj-aligned record admission. Each variant names the
/// refused face; none is a stringly panic. These mirror the schema-required
/// key set (the 16-key admission contract) plus the chain/digest laws.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SjRefusal {
    /// A required key is empty.
    #[error("empty required key `{key}`: refused")]
    EmptyKey {
        /// The refused key (dotted schema path).
        key: &'static str,
    },
    /// A commit-sha key is not 7..40 lowercase hex.
    #[error("key `{key}` value `{value}` is not 7..40 lowercase hex: refused")]
    BadCommitSha {
        /// The refused key.
        key: &'static str,
        /// The refused value.
        value: String,
    },
    /// A 40-hex git anchor key is malformed.
    #[error("key `{key}` value `{value}` is not a 40-hex git sha: refused")]
    BadSha {
        /// The refused key.
        key: &'static str,
        /// The refused value.
        value: String,
    },
    /// Standing value outside the schema vocabulary.
    #[error("standing value `{0}` is outside the schema vocabulary: refused")]
    BadStanding(String),
    /// BLOCKED / BUILD_BROKEN / REFUSED without a `broken_term`.
    #[error("standing `{0}` requires `standing.broken_term`: refused")]
    MissingBrokenTerm(String),
    /// `replay.commands` is empty (minItems 1) — a record still
    /// markdown-shaped.
    #[error("replay.commands is empty (minItems 1): refused")]
    EmptyReplay,
    /// A replay command is missing cmd/cwd substance.
    #[error("replay.commands[{index}] {reason}: refused")]
    BadCommand {
        /// Index of the refused command.
        index: usize,
        /// What substance is missing.
        reason: &'static str,
    },
    /// Canonical/JCS encoding of the record failed.
    #[error("canonical encoding failed: {0}")]
    Canonical(String),
    /// Chain event encoding or folding failed.
    #[error("chain error: {0}")]
    Chain(String),
    /// Chain re-verification refused — a tampered base cannot become a
    /// record.
    #[error("chain tamper refusal: {0}")]
    ChainTamper(String),
    /// Re-derived subject digest differs from the claimed one.
    #[error("subject digest mismatch: expected {expected}, claimed {claimed}")]
    DigestMismatch {
        /// Re-derived hex digest.
        expected: String,
        /// Claimed hex digest.
        claimed: String,
    },
    /// `replay_binding.chain_head_hash` differs from the base chain head.
    #[error("replay_binding.chain_head_hash does not bind the base chain head")]
    ChainHeadMismatch,
    /// Chain events rebuilt from the document differ from the carrier's
    /// events — a tampered face cannot ride.
    #[error("chain events rebuilt from the document differ from the carrier chain")]
    EventClaimMismatch,
    /// Record JSON decode failed.
    #[error("record decode failed: {0}")]
    Decode(String),
}

// ---------------------------------------------------------------------------
// Input draft + admission
// ---------------------------------------------------------------------------

/// One campaign commit record (chain-event payload, design §1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitRecord {
    /// Commit sha (7..40 lowercase hex).
    pub sha: String,
    /// One-line subject.
    pub summary: String,
    /// Court results attached to this commit (e.g. `["verify:ACCEPT"]`).
    #[serde(default)]
    pub court_results: Vec<String>,
}

/// A campaign draft: the claimed inputs of the record. Construction
/// ([`SjCampaign::new`]) runs the typed admission gates; nothing else in this
/// module accepts an unvalidated draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SjCampaignDraft {
    /// Work order id.
    pub work_order_id: String,
    /// Originating ceiling.
    pub origin_ceiling: Option<AuthorityCeiling>,
    /// Originating grant (`NONE` allowed).
    pub origin_grant: String,
    /// Originating actor.
    pub origin_actor: String,
    /// Provider name.
    pub provider_name: String,
    /// Provider execution id.
    pub provider_execution_id: String,
    /// Subject name.
    pub subject: String,
    /// Repository.
    pub repo: String,
    /// 40-hex commit anchor.
    pub subject_sha: String,
    /// 40-hex base anchor.
    pub base_sha: String,
    /// Campaign commits.
    pub commits: Vec<CommitRecord>,
    /// Residue declaration (the non-code remainder, declared honestly).
    pub residue_declaration: String,
    /// Files changed.
    pub files_changed: Vec<String>,
    /// Remote effects.
    pub remote_effects: Vec<String>,
    /// Court/gate commands actually executed (>= 1).
    pub replay_commands: Vec<ReplayCommand>,
    /// Git-tracked durable location.
    pub durable_location: Option<String>,
    /// Standing value.
    pub standing: StandingValue,
    /// What justifies the standing.
    pub derived_from: String,
    /// Required when standing demands it.
    pub broken_term: Option<BrokenTerm>,
    /// Prior work order ids.
    pub predecessor_work_order_ids: Vec<String>,
    /// Executed authority face.
    pub authority: Authority,
}

/// A validated campaign under construction. Obtainable only through
/// [`SjCampaign::new`] (the admission gate), so every later step operates on
/// law-checked inputs.
#[derive(Debug, Clone)]
pub struct SjCampaign {
    draft: SjCampaignDraft,
}

impl SjCampaign {
    /// Admit a draft or refuse it with typed refusals mirroring the
    /// schema-required key set.
    pub fn new(draft: SjCampaignDraft) -> Result<Self, SjRefusal> {
        validate_common(&draft)?;
        Ok(Self { draft })
    }

    /// Serialize the campaign as chain events: one per commit, one per court
    /// witness, one for the residue declaration (design §1). Every event
    /// payload is a pure function of a law-carrying document face, so the
    /// verifier can rebuild the whole chain from the document alone
    /// ([`SjRecord::verify`]). Deterministic in face order; commitments are
    /// `blake3(payload_json)`.
    pub fn chain_events(&self) -> Vec<OperationEvent> {
        let mut events = Vec::new();
        let mut seq = 0u64;
        for sha in &self
            .draft
            .commits
            .iter()
            .map(|c| c.sha.clone())
            .collect::<Vec<_>>()
        {
            let payload = json!({
                "kind": EVENT_COMMIT,
                "sha": sha,
            });
            events.push(make_event(
                &mut seq,
                EVENT_COMMIT,
                ObjectRef {
                    id: format!("commit:{sha}"),
                    obj_type: "git".to_string(),
                    qualifier: None,
                },
                &payload,
            ));
        }
        for command in &self.draft.replay_commands {
            let payload = court_payload(command);
            events.push(make_event(
                &mut seq,
                EVENT_COURT,
                ObjectRef {
                    id: format!("court:{}", command.cmd),
                    obj_type: "court".to_string(),
                    qualifier: None,
                },
                &payload,
            ));
        }
        let residue = residue_payload(
            &self.draft.standing,
            &self.draft.derived_from,
            self.draft.broken_term,
        );
        events.push(make_event(
            &mut seq,
            EVENT_RESIDUE,
            ObjectRef {
                id: format!("residue:{}", self.draft.work_order_id),
                obj_type: "residue".to_string(),
                qualifier: None,
            },
            &residue,
        ));
        events
    }

    /// Assemble the chain, derive the digest-bound identity, and emit the
    /// finished [`SjRecord`] (design §1–§4).
    pub fn finalize(self) -> Result<SjRecord, SjRefusal> {
        let events = self.chain_events();
        let mut assembler = ChainAssembler::new();
        for event in events {
            assembler
                .append(event)
                .map_err(|e| SjRefusal::Chain(e.to_string()))?;
        }
        let base = assembler.finalize();
        let document = build_document(&self.draft, &base)?;
        Ok(SjRecord { base, document })
    }
}

/// The finished record: the chain-sealed base receipt (tamper carrier) plus
/// the schema-shaped document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SjRecord {
    /// The chain-sealed base receipt carrying the campaign events.
    pub base: Receipt,
    /// The schema-shaped document faces.
    pub document: SjRecordDocument,
}

/// Internal wire form: document faces flattened beside the base receipt.
#[derive(Serialize, Deserialize)]
struct SjRecordWire {
    #[serde(flatten)]
    document: SjRecordDocument,
    base: Receipt,
}

impl SjRecord {
    /// JCS (RFC 8785) canonical JSON of the record document — the digest
    /// pre-image law of design §2. No "approximately canonical" serialization;
    /// integers beyond 2^53 refuse typed as
    /// [`SjRefusal::Canonical`].
    pub fn canonical_json(&self) -> Result<String, SjRefusal> {
        let value = serde_json::to_value(&self.document)
            .map_err(|e| SjRefusal::Canonical(e.to_string()))?;
        jcs(&value).map_err(|e: CanonicalError| SjRefusal::Canonical(e.to_string()))
    }

    /// The 32-byte subject digest, re-derived from the base receipt's content
    /// address (`subject_digest_of`), never read back from the document.
    pub fn subject_digest(&self) -> Result<[u8; 32], SjRefusal> {
        subject_digest_of(&self.base).map_err(|e| SjRefusal::Chain(e.to_string()))
    }

    /// Hex of [`SjRecord::subject_digest`].
    pub fn subject_digest_hex(&self) -> Result<String, SjRefusal> {
        Ok(hex_lower(&self.subject_digest()?))
    }

    /// Full verification: document admission (16-key contract), event
    /// reconstruction equality (every claimed event is exactly the
    /// chain-carried event), chain recompute, chain-head binding, and digest
    /// re-derivation.
    pub fn verify(&self) -> Result<(), SjRefusal> {
        validate_document(&self.document)?;

        let rebuilt = rebuild_events(&self.document)?;
        if rebuilt != self.base.events {
            return Err(SjRefusal::EventClaimMismatch);
        }
        recompute_chain(&self.base.events).map_err(|e| SjRefusal::ChainTamper(e.to_string()))?;

        let head = self.base.chain_hash.as_hex().to_string();
        if self.document.replay_binding.chain_head_hash != head {
            return Err(SjRefusal::ChainHeadMismatch);
        }

        let derived = hex_lower(&self.subject_digest()?);
        if self.document.identity.subject_digest.value != derived {
            return Err(SjRefusal::DigestMismatch {
                expected: derived,
                claimed: self.document.identity.subject_digest.value.clone(),
            });
        }
        Ok(())
    }

    /// JSON wire form: document faces flattened beside the base receipt.
    pub fn to_json(&self) -> Result<String, SjRefusal> {
        let wire = SjRecordWire {
            document: self.document.clone(),
            base: self.base.clone(),
        };
        serde_json::to_string(&wire).map_err(|e| SjRefusal::Decode(e.to_string()))
    }

    /// Deserialize and re-verify: the base receipt's deserialization
    /// re-verifies the BLAKE3 chain (a tampered base cannot become a record),
    /// then the full document law runs ([`SjRecord::verify`]).
    pub fn from_json(bytes: &str) -> Result<Self, SjRefusal> {
        let wire: SjRecordWire =
            serde_json::from_str(bytes).map_err(|e| SjRefusal::Decode(e.to_string()))?;
        let record = SjRecord {
            base: wire.base,
            document: wire.document,
        };
        record.verify()?;
        Ok(record)
    }
}

// ---------------------------------------------------------------------------
// Shared admission law
// ---------------------------------------------------------------------------

/// The 16 schema-required leaf keys: identity(4) + authority(3) +
/// consequence(3) + replay.commands(1) + per-command(3) + standing(2).
fn validate_document(doc: &SjRecordDocument) -> Result<(), SjRefusal> {
    require_non_empty("work_order_id", &doc.work_order_id)?;
    require_non_empty("origin_authority.grant", &doc.origin_authority.grant)?;
    require_non_empty("origin_authority.actor", &doc.origin_authority.actor)?;
    require_non_empty("provider.name", &doc.provider.name)?;
    require_non_empty("provider_execution_id", &doc.provider_execution_id)?;

    require_non_empty("identity.subject", &doc.identity.subject)?;
    require_non_empty("identity.repo", &doc.identity.repo)?;
    require_sha("identity.subject_sha", &doc.identity.subject_sha)?;
    require_sha("identity.base_sha", &doc.identity.base_sha)?;

    require_non_empty("authority.grant", &doc.authority.grant)?;
    require_non_empty("authority.actor", &doc.authority.actor)?;

    if doc.consequence.commits.is_empty() {
        return Err(SjRefusal::EmptyKey {
            key: "consequence.commits",
        });
    }
    for sha in &doc.consequence.commits {
        require_commit_sha("consequence.commits[]", sha)?;
    }
    if doc.consequence.files_changed.is_empty() {
        return Err(SjRefusal::EmptyKey {
            key: "consequence.files_changed",
        });
    }
    if doc.consequence.remote_effects.is_empty() {
        return Err(SjRefusal::EmptyKey {
            key: "consequence.remote_effects",
        });
    }

    if doc.replay.commands.is_empty() {
        return Err(SjRefusal::EmptyReplay);
    }
    for (index, command) in doc.replay.commands.iter().enumerate() {
        if command.cmd.is_empty() {
            return Err(SjRefusal::BadCommand {
                index,
                reason: "empty cmd",
            });
        }
        if command.cwd.trim().is_empty() {
            return Err(SjRefusal::BadCommand {
                index,
                reason: "empty cwd",
            });
        }
    }

    require_non_empty("standing.derived_from", &doc.standing.derived_from)?;
    let spelled = doc.standing.value.as_str();
    if !standing_in_vocabulary(&spelled) {
        return Err(SjRefusal::BadStanding(spelled));
    }
    if doc.standing.value.requires_broken_term() && doc.standing.broken_term.is_none() {
        return Err(SjRefusal::MissingBrokenTerm(spelled));
    }
    Ok(())
}

/// Draft-side admission: the same key law plus the chain-payload inputs.
fn validate_common(draft: &SjCampaignDraft) -> Result<(), SjRefusal> {
    if draft.commits.is_empty() {
        return Err(SjRefusal::EmptyKey { key: "commits" });
    }
    for commit in &draft.commits {
        require_commit_sha("commits[].sha", &commit.sha)?;
        require_non_empty("commits[].summary", &commit.summary)?;
    }
    require_non_empty("residue_declaration", &draft.residue_declaration)?;
    if draft.replay_commands.is_empty() {
        return Err(SjRefusal::EmptyReplay);
    }
    for command in &draft.replay_commands {
        if command.cmd.is_empty() {
            return Err(SjRefusal::BadCommand {
                index: 0,
                reason: "empty cmd",
            });
        }
    }
    // The 16-key law runs once over the draft's law-carrying faces; the
    // digest/binding faces are chain-derived at finalize, so they are checked
    // there over their real values.
    require_non_empty("work_order_id", &draft.work_order_id)?;
    require_non_empty("origin_authority.grant", &draft.origin_grant)?;
    require_non_empty("origin_authority.actor", &draft.origin_actor)?;
    require_non_empty("provider.name", &draft.provider_name)?;
    require_non_empty("provider_execution_id", &draft.provider_execution_id)?;
    require_non_empty("identity.subject", &draft.subject)?;
    require_non_empty("identity.repo", &draft.repo)?;
    require_sha("identity.subject_sha", &draft.subject_sha)?;
    require_sha("identity.base_sha", &draft.base_sha)?;
    require_non_empty("authority.grant", &draft.authority.grant)?;
    require_non_empty("authority.actor", &draft.authority.actor)?;
    if draft.files_changed.is_empty() {
        return Err(SjRefusal::EmptyKey {
            key: "consequence.files_changed",
        });
    }
    if draft.remote_effects.is_empty() {
        return Err(SjRefusal::EmptyKey {
            key: "consequence.remote_effects",
        });
    }
    require_non_empty("standing.derived_from", &draft.derived_from)?;
    let spelled = draft.standing.as_str();
    if !standing_in_vocabulary(&spelled) {
        return Err(SjRefusal::BadStanding(spelled));
    }
    if draft.standing.requires_broken_term() && draft.broken_term.is_none() {
        return Err(SjRefusal::MissingBrokenTerm(spelled));
    }
    Ok(())
}

/// Build the document from a validated draft and the sealed base.
/// A required string key carries substance (non-empty after trim).
fn require_non_empty(key: &'static str, value: &str) -> Result<(), SjRefusal> {
    if value.trim().is_empty() {
        Err(SjRefusal::EmptyKey { key })
    } else {
        Ok(())
    }
}

/// A 40-hex lowercase git anchor (schema `^[0-9a-f]{40}$`).
fn require_sha(key: &'static str, value: &str) -> Result<(), SjRefusal> {
    if value.len() == 40
        && value.bytes().all(|b| b.is_ascii_hexdigit())
        && value.bytes().all(|b| !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(SjRefusal::BadSha {
            key,
            value: value.to_string(),
        })
    }
}

/// A 7..40 lowercase-hex commit sha (schema `^[0-9a-f]{7,40}$`).
fn require_commit_sha(key: &'static str, value: &str) -> Result<(), SjRefusal> {
    let hex_ok = (7..=40).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
    if hex_ok {
        Ok(())
    } else {
        Err(SjRefusal::BadCommitSha {
            key,
            value: value.to_string(),
        })
    }
}

/// Build the document from a validated draft and the sealed base.
fn build_document(draft: &SjCampaignDraft, base: &Receipt) -> Result<SjRecordDocument, SjRefusal> {
    let subject_digest = subject_digest_of(base).map_err(|e| SjRefusal::Chain(e.to_string()))?;
    let document = SjRecordDocument {
        work_order_id: draft.work_order_id.clone(),
        origin_authority: OriginAuthority {
            ceiling: draft.origin_ceiling,
            grant: draft.origin_grant.clone(),
            actor: draft.origin_actor.clone(),
        },
        provider: Provider {
            name: draft.provider_name.clone(),
            transport: None,
        },
        provider_execution_id: draft.provider_execution_id.clone(),
        identity: Identity {
            subject: draft.subject.clone(),
            repo: draft.repo.clone(),
            subject_sha: draft.subject_sha.clone(),
            base_sha: draft.base_sha.clone(),
            subject_digest: SubjectDigest {
                algorithm: "blake3".to_string(),
                value: hex_lower(&subject_digest),
            },
        },
        authority: draft.authority.clone(),
        consequence: Consequence {
            commits: draft.commits.iter().map(|c| c.sha.clone()).collect(),
            files_changed: draft.files_changed.clone(),
            remote_effects: draft.remote_effects.clone(),
        },
        replay: Replay {
            commands: draft.replay_commands.clone(),
            durable_location: draft.durable_location.clone(),
        },
        replay_binding: ReplayBinding {
            event_ids: base.events.iter().map(|e| e.id.clone()).collect(),
            chain_head_hash: base.chain_hash.as_hex().to_string(),
            predecessor_work_order_ids: draft.predecessor_work_order_ids.clone(),
        },
        standing: Standing {
            value: draft.standing.clone(),
            derived_from: draft.derived_from.clone(),
            broken_term: draft.broken_term,
        },
    };
    validate_document(&document)?;
    Ok(document)
}

/// Residue event payload: the standing declaration rides IN the chain, so a
/// tampered standing/derived_from/broken_term face cannot rebuild the carrier
/// events ([`SjRecord::verify`] refuses). The free-text residue declaration is
/// admission-checked on the draft but is not a law-carrying payload face.
fn residue_payload(
    standing: &StandingValue,
    derived_from: &str,
    broken_term: Option<BrokenTerm>,
) -> serde_json::Value {
    json!({
        "kind": EVENT_RESIDUE,
        "standing": standing.as_str(),
        "derived_from": derived_from,
        "broken_term": broken_term.map(broken_term_spelling),
    })
}

/// The residue spelling of a broken term (schema enum value).
fn broken_term_spelling(term: BrokenTerm) -> &'static str {
    match term {
        BrokenTerm::MuOnO => "mu_on_O",
        BrokenTerm::AdmissionVacuous => "admission_vacuous",
        BrokenTerm::MuUnlawful => "mu_unlawful",
        BrokenTerm::RMissingIdentity => "R_missing_identity",
        BrokenTerm::RMissingAuthority => "R_missing_authority",
        BrokenTerm::RMissingConsequence => "R_missing_consequence",
        BrokenTerm::RMissingReplay => "R_missing_replay",
        BrokenTerm::RMissingStanding => "R_missing_standing",
        BrokenTerm::RNotFedBack => "R_not_fed_back",
    }
}

/// Deterministic event builder: commitments are `blake3(payload_json)`.
fn make_event(
    seq: &mut u64,
    event_type: &str,
    object: ObjectRef,
    payload: &serde_json::Value,
) -> OperationEvent {
    let payload_bytes = serde_json::to_vec(payload).unwrap_or_else(|_| Vec::new());
    let event = OperationEvent {
        id: format!("evt-{seq}"),
        seq: *seq,
        event_type: event_type.to_string(),
        objects: vec![object],
        payload_commitment: Blake3Hash::from_bytes(&payload_bytes),
    };
    *seq += 1;
    event
}

/// Rebuild the chain events purely from the document's own faces. This is the
/// reconstruction the verifier replays: a tampered document face that rides in
/// a chain payload (commit shas, standing/derived_from/broken_term) cannot
/// rebuild the carrier's events and refuses as
/// [`SjRefusal::EventClaimMismatch`].
///
/// The rebuildable payload fields are exactly those carried in the chain
/// payloads: commit shas (order-sensitive) and the residue/standing
/// declaration. Faces that are projections (digests, bindings) are checked by
/// their own laws in [`SjRecord::verify`].
fn rebuild_events(doc: &SjRecordDocument) -> Result<Vec<OperationEvent>, SjRefusal> {
    let mut events = Vec::new();
    let mut seq = 0u64;
    for sha in &doc.consequence.commits {
        let payload = json!({
            "kind": EVENT_COMMIT,
            "sha": sha,
        });
        events.push(make_event(
            &mut seq,
            EVENT_COMMIT,
            ObjectRef {
                id: format!("commit:{sha}"),
                obj_type: "git".to_string(),
                qualifier: None,
            },
            &payload,
        ));
    }
    for command in &doc.replay.commands {
        let payload = court_payload(command);
        events.push(make_event(
            &mut seq,
            EVENT_COURT,
            ObjectRef {
                id: format!("court:{}", command.cmd),
                obj_type: "court".to_string(),
                qualifier: None,
            },
            &payload,
        ));
    }
    let residue = residue_payload(
        &doc.standing.value,
        &doc.standing.derived_from,
        doc.standing.broken_term,
    );
    events.push(make_event(
        &mut seq,
        EVENT_RESIDUE,
        ObjectRef {
            id: format!("residue:{}", doc.work_order_id),
            obj_type: "residue".to_string(),
            qualifier: None,
        },
        &residue,
    ));
    Ok(events)
}

/// Court-witness event payload — a pure function of the replayed command, so
/// the court witness rides in the chain and rebuilds from `replay.commands`.
fn court_payload(command: &ReplayCommand) -> serde_json::Value {
    json!({
        "kind": EVENT_COURT,
        "cmd": command.cmd,
        "exit": command.exit,
        "cwd": command.cwd,
    })
}

/// Lowercase hex, hand-rolled so the module needs no `hex` feature.
fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Schema-vocabulary check for a spelled standing value.
fn standing_in_vocabulary(spelled: &str) -> bool {
    match spelled {
        "UNKNOWN" | "PARTIAL_ALIVE" | "ALIVE" | "BUILD_BROKEN" => true,
        _ => {
            (spelled.starts_with("BLOCKED:") && spelled.len() > "BLOCKED:".len())
                || (spelled.starts_with("BLOCKED") && spelled == "BLOCKED")
                || (spelled.starts_with("UNSUPPORTED(")
                    && spelled.ends_with(')')
                    && spelled.len() > "UNSUPPORTED()".len())
                || (spelled.starts_with("REFUSED(")
                    && spelled.ends_with(')')
                    && spelled.len() > "REFUSED()".len())
        }
    }
}
