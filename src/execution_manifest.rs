//! Immutable execution-manifest binding for consequence receipts.
//!
//! A manifest describes the exact execution universe used to produce a
//! consequence. It certifies identity and drift; it does not decide whether
//! an authority grant or policy is substantively correct.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Exact, serializable execution universe bound to a receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionManifest {
    pub exact_subject: String,
    pub subject_digest: String,
    pub repository: Option<String>,
    pub base_sha: Option<String>,
    pub ontology_digest: Option<String>,
    pub policy_digest: Option<String>,
    pub capability_manifest_digest: Option<String>,
    pub tool_manifest_digest: Option<String>,
    pub planner_identity: Option<String>,
    pub planner_version: Option<String>,
    pub generator_identity: Option<String>,
    pub generator_version: Option<String>,
    pub runtime_identity: Option<String>,
    pub runtime_version: Option<String>,
    pub dependency_lock_digest: Option<String>,
    pub authority_grant_digest: Option<String>,
    pub intent_digest: String,
    #[serde(default)]
    pub environment_constraints: BTreeMap<String, String>,
}

/// The small receipt-side binding carried across serialization boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBinding {
    pub execution_manifest_digest: String,
    pub exact_subject: String,
    pub subject_digest: String,
    pub intent_digest: String,
    pub authority_grant_digest: Option<String>,
}

/// Typed certification failures. These are evidence classifications, not policy decisions.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManifestRefusal {
    #[error("MANIFEST_DIGEST_MISMATCH")]
    ManifestDigestMismatch,
    #[error("SUBJECT_IDENTITY_CHANGED")]
    SubjectIdentityChanged,
    #[error("AUTHORITY_BINDING_CHANGED")]
    AuthorityBindingChanged,
    #[error("POLICY_BINDING_CHANGED")]
    PolicyBindingChanged,
    #[error("ONTOLOGY_BINDING_CHANGED")]
    OntologyBindingChanged,
    #[error("TOOL_SURFACE_CHANGED")]
    ToolSurfaceChanged,
    #[error("INTENT_BINDING_CHANGED")]
    IntentBindingChanged,
    #[error("INVALID_MANIFEST:{0}")]
    InvalidManifest(&'static str),
}

impl ExecutionManifest {
    /// Validate the minimum exact-subject identity required for certification.
    pub fn validate(&self) -> Result<(), ManifestRefusal> {
        if self.exact_subject.trim().is_empty() {
            return Err(ManifestRefusal::InvalidManifest("exact_subject"));
        }
        if self.subject_digest.trim().is_empty() {
            return Err(ManifestRefusal::InvalidManifest("subject_digest"));
        }
        if self.intent_digest.trim().is_empty() {
            return Err(ManifestRefusal::InvalidManifest("intent_digest"));
        }
        Ok(())
    }

    /// Canonical digest. Struct-field order plus BTreeMap ordering makes the
    /// serialized witness deterministic for the same manifest value.
    pub fn digest(&self) -> Result<String, ManifestRefusal> {
        self.validate()?;
        let bytes =
            serde_json::to_vec(self).map_err(|_| ManifestRefusal::InvalidManifest("serialize"))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    /// Produce the compact receipt binding for this exact manifest.
    pub fn binding(&self) -> Result<ExecutionBinding, ManifestRefusal> {
        Ok(ExecutionBinding {
            execution_manifest_digest: self.digest()?,
            exact_subject: self.exact_subject.clone(),
            subject_digest: self.subject_digest.clone(),
            intent_digest: self.intent_digest.clone(),
            authority_grant_digest: self.authority_grant_digest.clone(),
        })
    }
}

/// Verify that a serialized receipt binding still names this exact manifest.
pub fn verify_binding(
    manifest: &ExecutionManifest,
    binding: &ExecutionBinding,
) -> Result<(), ManifestRefusal> {
    if manifest.digest()? != binding.execution_manifest_digest {
        return Err(ManifestRefusal::ManifestDigestMismatch);
    }
    if manifest.exact_subject != binding.exact_subject
        || manifest.subject_digest != binding.subject_digest
    {
        return Err(ManifestRefusal::SubjectIdentityChanged);
    }
    if manifest.intent_digest != binding.intent_digest {
        return Err(ManifestRefusal::IntentBindingChanged);
    }
    if manifest.authority_grant_digest != binding.authority_grant_digest {
        return Err(ManifestRefusal::AuthorityBindingChanged);
    }
    Ok(())
}

/// Return the first semantics-bearing drift that requires requalification.
pub fn requalification_reason(
    before: &ExecutionManifest,
    after: &ExecutionManifest,
) -> Result<Option<ManifestRefusal>, ManifestRefusal> {
    before.validate()?;
    after.validate()?;
    if before.exact_subject != after.exact_subject || before.subject_digest != after.subject_digest
    {
        return Ok(Some(ManifestRefusal::SubjectIdentityChanged));
    }
    if before.authority_grant_digest != after.authority_grant_digest {
        return Ok(Some(ManifestRefusal::AuthorityBindingChanged));
    }
    if before.policy_digest != after.policy_digest {
        return Ok(Some(ManifestRefusal::PolicyBindingChanged));
    }
    if before.ontology_digest != after.ontology_digest {
        return Ok(Some(ManifestRefusal::OntologyBindingChanged));
    }
    if before.tool_manifest_digest != after.tool_manifest_digest {
        return Ok(Some(ManifestRefusal::ToolSurfaceChanged));
    }
    if before.intent_digest != after.intent_digest {
        return Ok(Some(ManifestRefusal::IntentBindingChanged));
    }
    if before != after {
        return Ok(Some(ManifestRefusal::ManifestDigestMismatch));
    }
    Ok(None)
}
