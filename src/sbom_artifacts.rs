//! SBOM artifact collection and forensics bundle creation
//!
//! Collects and packages SBOM analysis artifacts including supply-chain graphs,
//! vulnerability assessments, compliance reports, and provenance attestations
//! into replayable forensics bundles (inspired by clnrm_prototype patterns).
//!
//! This module brings filesystem interaction patterns from the prototype:
//! - RAII guards for resource cleanup
//! - Structured artifact collection with metadata
//! - Sensitive data redaction
//! - Proper error handling with context
//! - File verification before operations

use crate::error::{AffidavitError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Forensics bundle containing all SBOM analysis artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomForensicsBundle {
    /// Bundle metadata
    pub metadata: BundleMetadata,
    /// Supply-chain graph (adjacency representation)
    pub graph: Option<SupplyChainGraph>,
    /// Vulnerability assessment results
    pub vulnerabilities: Vec<VulnerabilityRecord>,
    /// Compliance assessment results
    pub compliance: Vec<ComplianceRecord>,
    /// Risk propagation results
    pub risk_propagation: Vec<RiskPropagationRecord>,
    /// Provenance attestation
    pub attestation: Option<AttestationData>,
    /// Sensitive data redaction map
    pub redactions: HashMap<String, String>,
}

/// Bundle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Bundle version
    pub version: String,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
    /// SBOM source identifier
    pub sbom_id: String,
    /// Bundle ID
    pub bundle_id: String,
    /// SBOM format (CycloneDX, SPDX, etc.)
    pub sbom_format: String,
    /// Description
    pub description: Option<String>,
}

/// Supply-chain graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainGraph {
    /// Number of components
    pub component_count: usize,
    /// Number of dependency edges
    pub edge_count: usize,
    /// Adjacency list (component ID -> dependent IDs)
    pub adjacency: HashMap<String, Vec<String>>,
    /// Component metadata
    pub components: HashMap<String, ComponentNode>,
}

/// Component node in supply-chain graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    /// Component package URL (purl)
    pub purl: String,
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Supplier/maintainer
    pub supplier: Option<String>,
    /// Licenses (if known)
    pub licenses: Vec<String>,
}

/// Vulnerability record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityRecord {
    /// CVE identifier
    pub cve_id: String,
    /// Affected component
    pub affected_component: String,
    /// Severity level
    pub severity: String,
    /// CVSS score
    pub cvss_score: Option<f64>,
    /// VEX statement (if applicable)
    pub vex_status: Option<String>,
}

/// Compliance assessment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecord {
    /// Framework name (NTIA, SLSA, etc.)
    pub framework: String,
    /// Pass/fail result
    pub passed: bool,
    /// Compliance score (0.0-1.0)
    pub score: f64,
    /// Failed requirements
    pub failures: Vec<String>,
}

/// Risk propagation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPropagationRecord {
    /// Component with risk
    pub component: String,
    /// Root vulnerability
    pub root_cve: String,
    /// Propagated to (components)
    pub propagated_to: Vec<String>,
    /// Blast radius
    pub blast_radius: usize,
}

/// Provenance attestation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationData {
    /// SLSA provenance format version
    pub slsa_version: String,
    /// Builder identity
    pub builder: Option<String>,
    /// Attestation timestamp
    pub timestamp: u64,
    /// Build environment hash
    pub environment_hash: String,
}

/// SBOM artifact collector
///
/// Collects and packages SBOM analysis artifacts with proper error handling,
/// file verification, and sensitive data redaction (inspired by clnrm_prototype).
pub struct SbomArtifactCollector {
    /// Working directory for temporary files
    work_dir: PathBuf,
    /// Whether to redact sensitive information
    redact_sensitive: bool,
}

impl SbomArtifactCollector {
    /// Create a new SBOM artifact collector
    pub fn new() -> Result<Self> {
        let work_dir = std::env::temp_dir().join("affidavit-sbom-artifacts");
        std::fs::create_dir_all(&work_dir)?;

        Ok(Self {
            work_dir,
            redact_sensitive: true,
        })
    }

    /// Enable/disable sensitive data redaction
    pub fn with_redaction(mut self, redact: bool) -> Self {
        self.redact_sensitive = redact;
        self
    }

    /// Create a new forensics bundle with metadata
    pub fn create_bundle(
        &self,
        sbom_id: &str,
        sbom_format: &str,
        description: Option<String>,
    ) -> SbomForensicsBundle {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let bundle_id = format!("sbom_bundle_{}", timestamp);

        SbomForensicsBundle {
            metadata: BundleMetadata {
                version: "1.0".to_string(),
                created_at: timestamp,
                sbom_id: sbom_id.to_string(),
                bundle_id,
                sbom_format: sbom_format.to_string(),
                description,
            },
            graph: None,
            vulnerabilities: Vec::new(),
            compliance: Vec::new(),
            risk_propagation: Vec::new(),
            attestation: None,
            redactions: HashMap::new(),
        }
    }

    /// Save bundle to file with error context
    pub fn save_bundle(&self, bundle: &SbomForensicsBundle, path: PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(bundle)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Load bundle from file with verification
    pub fn load_bundle(&self, path: PathBuf) -> Result<SbomForensicsBundle> {
        if !path.exists() {
            return Err(AffidavitError::Execution(
                format!("SBOM bundle file not found: {}", path.display()),
            ));
        }

        let content = std::fs::read_to_string(&path)?;
        let bundle: SbomForensicsBundle = serde_json::from_str(&content)?;

        Ok(bundle)
    }

    /// Check if a value is sensitive and should be redacted
    fn is_sensitive_value(&self, value: &str) -> bool {
        let sensitive_patterns = [
            "password", "secret", "token", "key", "credential",
            "aws_", "github_", "gitlab_", "docker_",
            "private", "api_key",
        ];

        sensitive_patterns
            .iter()
            .any(|pattern| value.to_lowercase().contains(pattern))
    }

    /// Redact sensitive values from a string
    pub fn redact_sensitive(&self, value: &str) -> String {
        if self.redact_sensitive && self.is_sensitive_value(value) {
            "[REDACTED]".to_string()
        } else {
            value.to_string()
        }
    }
}

impl Default for SbomArtifactCollector {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            work_dir: std::env::temp_dir().join("affidavit-sbom-artifacts"),
            redact_sensitive: true,
        })
    }
}

/// RAII guard for SBOM artifact lifecycle management
///
/// Ensures proper cleanup of artifact resources even on panic
pub struct SbomArtifactGuard {
    bundle: Option<SbomForensicsBundle>,
    path: Option<PathBuf>,
    cleanup_actions: Vec<Box<dyn FnOnce() + Send + Sync>>,
}

impl SbomArtifactGuard {
    /// Create a new artifact guard
    pub fn new(bundle: SbomForensicsBundle) -> Self {
        Self {
            bundle: Some(bundle),
            path: None,
            cleanup_actions: Vec::new(),
        }
    }

    /// Add a cleanup action to be executed on drop
    pub fn add_cleanup_action<F>(mut self, action: F) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        self.cleanup_actions.push(Box::new(action));
        self
    }

    /// Set the path where the bundle is/will be saved
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    /// Get a reference to the bundle
    pub fn bundle(&self) -> Option<&SbomForensicsBundle> {
        self.bundle.as_ref()
    }

    /// Take ownership of the bundle
    pub fn take_bundle(mut self) -> Option<SbomForensicsBundle> {
        self.bundle.take()
    }

    /// Manually trigger cleanup
    pub fn cleanup(mut self) {
        for action in self.cleanup_actions.drain(..) {
            action();
        }
    }
}

impl Drop for SbomArtifactGuard {
    fn drop(&mut self) {
        // Execute cleanup actions in reverse order
        while let Some(action) = self.cleanup_actions.pop() {
            action();
        }

        // Optional: cleanup the file if it exists
        if let Some(ref path) = self.path {
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_creation() {
        let collector = SbomArtifactCollector::new().unwrap();
        let bundle = collector.create_bundle(
            "test-sbom",
            "CycloneDX",
            Some("Test bundle".to_string()),
        );

        assert_eq!(bundle.metadata.sbom_id, "test-sbom");
        assert_eq!(bundle.metadata.sbom_format, "CycloneDX");
        assert!(!bundle.metadata.bundle_id.is_empty());
    }

    #[test]
    fn test_sensitive_redaction() {
        let collector = SbomArtifactCollector::new().unwrap();
        assert_eq!(collector.redact_sensitive("normal_value"), "normal_value");
        assert_eq!(collector.redact_sensitive("api_key_secret"), "[REDACTED]");
        assert_eq!(collector.redact_sensitive("password123"), "[REDACTED]");
    }

    #[test]
    fn test_artifact_guard() {
        let collector = SbomArtifactCollector::new().unwrap();
        let bundle = collector.create_bundle("test", "SPDX", None);
        let guard = SbomArtifactGuard::new(bundle).add_cleanup_action(|| {
            // Cleanup action
        });

        assert!(guard.bundle().is_some());
    }

    #[test]
    fn test_bundle_serialization() {
        let collector = SbomArtifactCollector::new().unwrap();
        let bundle = collector.create_bundle("test-sbom", "CycloneDX", None);

        let json = serde_json::to_string(&bundle).unwrap();
        let deserialized: SbomForensicsBundle = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.metadata.sbom_id, bundle.metadata.sbom_id);
    }
}
