// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bidirectional sync between Rust ggen code and RDF ontology
//!
//! Ensures that:
//! - Changes to #[verb] functions are reflected in the RDF ontology
//! - Changes to the RDF ontology can generate new #[verb] code
//! - The declared command structure (code) matches the ontology (RDF)
//!
//! ## Algorithm
//!
//! 1. Load v26.6.1 command registry (runtime state)
//! 2. Query RDF ontology for latest definitions
//! 3. Compute diff: which verbs are new, modified, removed
//! 4. For new verbs: generate RDF, commit to ontology
//! 5. For modified: update both systems
//! 6. For removed: purge from ontology
//!
//! ## Conformance Validation
//!
//! Following process mining Chicago TDD doctrine:
//! - Event log is source of truth
//! - If code says verb exists but RDF doesn't reflect it, then mismatch is a defect
//! - Negative test: inject impossible verb definitions, verify rejection

use crate::ggen_to_rdf::{parse_rust_source, verb_definitions_to_ntriples};
use crate::rdf_to_ggen::{rdf_triples_to_verb_definitions, RdfVerbDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// SYNC OPERATION TYPES
// =============================================================================

/// Synchronization operation (add, update, remove)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SyncOperation {
    /// Verb exists in code but not in RDF
    AddToOntology,
    /// Verb exists in both, but definitions differ
    UpdateOntology,
    /// Verb exists in RDF but not in code
    RemoveFromOntology,
    /// Verb definitions match
    NoChange,
}

/// Verb entry in synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbSyncEntry {
    pub verb_name: String,
    pub noun: Option<String>,
    pub operation: SyncOperation,
    pub code_version: Option<RdfVerbDefinition>,
    pub rdf_version: Option<RdfVerbDefinition>,
    pub differences: Vec<String>,
}

/// Complete synchronization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub timestamp: String,
    pub total_verbs: usize,
    pub changes: Vec<VerbSyncEntry>,
    pub summary: SyncSummary,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSummary {
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub unchanged: usize,
    pub conformant: bool,
}

// =============================================================================
// SYNC ENGINE
// =============================================================================

/// Bidirectional synchronization engine
pub struct OntologySync {
    /// Path to Rust source files
    pub source_paths: Vec<PathBuf>,
    /// Path to RDF ontology directory
    pub ontology_path: PathBuf,
}

impl OntologySync {
    /// Create a new sync engine
    pub fn new(source_paths: Vec<PathBuf>, ontology_path: PathBuf) -> Self {
        Self { source_paths, ontology_path }
    }

    /// Perform bidirectional sync
    ///
    /// # Returns
    /// SyncResult containing all differences and operations needed
    pub async fn sync_ggen_with_ontology(&self) -> Result<SyncResult, SyncError> {
        // Step 1: Load verbs from Rust code
        let code_verbs = self.load_verbs_from_code().await?;

        // Step 2: Load verbs from RDF ontology
        let rdf_verbs = self.load_verbs_from_ontology().await?;

        // Step 3: Compute diff
        let changes = self.compute_diff(&code_verbs, &rdf_verbs);

        // Step 4: Generate sync result
        let summary = self.summarize_changes(&changes);

        let result = SyncResult {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| format!("{:?}", d))
                .unwrap_or_else(|_| "unknown".to_string()),
            total_verbs: code_verbs.len().max(rdf_verbs.len()),
            changes,
            summary,
        };

        Ok(result)
    }

    /// Load verbs from Rust source files
    async fn load_verbs_from_code(&self) -> Result<Vec<RdfVerbDefinition>, SyncError> {
        let mut all_verbs = Vec::new();

        for path in &self.source_paths {
            let content = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| SyncError::IoError(e.to_string()))?;

            let verbs = parse_rust_source(&content)
                .map_err(|e| SyncError::ParseError(format!("{:?}", e)))?;

            all_verbs.extend(verbs);
        }

        Ok(all_verbs)
    }

    /// Load verbs from RDF ontology
    async fn load_verbs_from_ontology(&self) -> Result<Vec<RdfVerbDefinition>, SyncError> {
        // Try to load from .nt (N-Triples) files in ontology directory
        let mut all_verbs = Vec::new();

        let entries = std::fs::read_dir(&self.ontology_path)
            .map_err(|e| SyncError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SyncError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().map(|ext| ext == "nt").unwrap_or(false) {
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(|e| SyncError::IoError(e.to_string()))?;

                let triples = parse_ntriples(&content)?;
                let verbs = rdf_triples_to_verb_definitions(triples)
                    .map_err(|e| SyncError::RdfError(format!("{}", e)))?;

                all_verbs.extend(verbs);
            }
        }

        Ok(all_verbs)
    }

    /// Compute differences between code and RDF versions
    fn compute_diff(
        &self,
        code_verbs: &[RdfVerbDefinition],
        rdf_verbs: &[RdfVerbDefinition],
    ) -> Vec<VerbSyncEntry> {
        let mut changes = Vec::new();
        let code_map: HashMap<_, _> =
            code_verbs.iter().map(|v| ((&v.name, &v.noun_name), v.clone())).collect();
        let rdf_map: HashMap<_, _> =
            rdf_verbs.iter().map(|v| ((&v.name, &v.noun_name), v.clone())).collect();

        // Check for new and modified verbs in code
        for (key, code_verb) in &code_map {
            match rdf_map.get(key) {
                None => {
                    // New verb
                    changes.push(VerbSyncEntry {
                        verb_name: code_verb.name.clone(),
                        noun: code_verb.noun_name.clone(),
                        operation: SyncOperation::AddToOntology,
                        code_version: Some(code_verb.clone()),
                        rdf_version: None,
                        differences: vec!["Verb missing in RDF ontology".to_string()],
                    });
                }
                Some(rdf_verb) => {
                    // Check if modified
                    let diffs = compute_verb_differences(code_verb, rdf_verb);
                    if diffs.is_empty() {
                        changes.push(VerbSyncEntry {
                            verb_name: code_verb.name.clone(),
                            noun: code_verb.noun_name.clone(),
                            operation: SyncOperation::NoChange,
                            code_version: Some(code_verb.clone()),
                            rdf_version: Some(rdf_verb.clone()),
                            differences: vec![],
                        });
                    } else {
                        changes.push(VerbSyncEntry {
                            verb_name: code_verb.name.clone(),
                            noun: code_verb.noun_name.clone(),
                            operation: SyncOperation::UpdateOntology,
                            code_version: Some(code_verb.clone()),
                            rdf_version: Some(rdf_verb.clone()),
                            differences: diffs,
                        });
                    }
                }
            }
        }

        // Check for removed verbs (in RDF but not in code)
        for (key, rdf_verb) in &rdf_map {
            if !code_map.contains_key(key) {
                changes.push(VerbSyncEntry {
                    verb_name: rdf_verb.name.clone(),
                    noun: rdf_verb.noun_name.clone(),
                    operation: SyncOperation::RemoveFromOntology,
                    code_version: None,
                    rdf_version: Some(rdf_verb.clone()),
                    differences: vec!["Verb exists in RDF but not in code".to_string()],
                });
            }
        }

        changes
    }

    /// Summarize sync changes
    fn summarize_changes(&self, changes: &[VerbSyncEntry]) -> SyncSummary {
        let mut summary =
            SyncSummary { added: 0, modified: 0, removed: 0, unchanged: 0, conformant: true };

        for change in changes {
            match change.operation {
                SyncOperation::AddToOntology => summary.added += 1,
                SyncOperation::UpdateOntology => summary.modified += 1,
                SyncOperation::RemoveFromOntology => {
                    summary.removed += 1;
                    summary.conformant = false;
                }
                SyncOperation::NoChange => summary.unchanged += 1,
            }
        }

        summary.conformant = summary.removed == 0;
        summary
    }

    /// Write sync results to RDF ontology files
    pub async fn apply_sync(&self, result: &SyncResult) -> Result<(), SyncError> {
        // Group verbs to add/update by noun
        let mut verbs_to_write: HashMap<Option<String>, Vec<RdfVerbDefinition>> = HashMap::new();

        for entry in &result.changes {
            match &entry.operation {
                SyncOperation::AddToOntology | SyncOperation::UpdateOntology => {
                    if let Some(code_version) = &entry.code_version {
                        verbs_to_write
                            .entry(entry.noun.clone())
                            .or_insert_with(Vec::new)
                            .push(code_version.clone());
                    }
                }
                SyncOperation::RemoveFromOntology => {
                    // TODO: Remove from ontology
                }
                SyncOperation::NoChange => {}
            }
        }

        // Write N-Triples files
        for (noun, verbs) in verbs_to_write {
            let ntriples = verb_definitions_to_ntriples(&verbs);
            let noun_name = noun.as_deref().unwrap_or("root");
            let output_path = self.ontology_path.join(format!("{}-verbs.nt", noun_name));

            tokio::fs::write(&output_path, &ntriples)
                .await
                .map_err(|e| SyncError::IoError(e.to_string()))?;
        }

        Ok(())
    }
}

// =============================================================================
// DIFFERENCE DETECTION
// =============================================================================

/// Compute field-level differences between code and RDF verb definitions
fn compute_verb_differences(code: &RdfVerbDefinition, rdf: &RdfVerbDefinition) -> Vec<String> {
    let mut diffs = Vec::new();

    if code.name != rdf.name {
        diffs.push(format!("Name: code={}, rdf={}", code.name, rdf.name));
    }

    if code.description != rdf.description {
        diffs.push(format!(
            "Description: code != rdf (code has {} chars, rdf has {} chars)",
            code.description.len(),
            rdf.description.len()
        ));
    }

    if code.return_type != rdf.return_type {
        diffs.push(format!("Return type: code={}, rdf={}", code.return_type, rdf.return_type));
    }

    if code.arguments.len() != rdf.arguments.len() {
        diffs.push(format!(
            "Argument count: code={}, rdf={}",
            code.arguments.len(),
            rdf.arguments.len()
        ));
    }

    if code.is_async != rdf.is_async {
        diffs.push(format!("Async: code={}, rdf={}", code.is_async, rdf.is_async));
    }

    diffs
}

// =============================================================================
// N-TRIPLES PARSING
// =============================================================================

use crate::rdf_to_ggen::RdfTriple;

/// Parse N-Triples format to RDF triples
fn parse_ntriples(content: &str) -> Result<Vec<RdfTriple>, SyncError> {
    let mut triples = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Remove trailing period and whitespace
        let line = if line.ends_with('.') { &line[..line.len() - 1] } else { line };

        // Split into subject predicate object
        let parts: Vec<&str> = line.split('>').map(|p| p.trim()).collect();
        if parts.len() < 3 {
            continue;
        }

        let subject = parts[0].trim_start_matches('<').to_string();
        let predicate = parts[1].trim_start_matches('<').to_string();
        let object = parts[2].trim_matches(|c| c == '<' || c == '"').to_string();

        triples.push(RdfTriple {
            subject,
            predicate,
            object,
            object_type: crate::rdf_to_ggen::ObjectType::Literal,
        });
    }

    Ok(triples)
}

// =============================================================================
// ERROR TYPES
// =============================================================================

#[derive(Debug)]
pub enum SyncError {
    IoError(String),
    ParseError(String),
    RdfError(String),
    ConformanceError(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::IoError(msg) => write!(f, "IO error: {}", msg),
            SyncError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SyncError::RdfError(msg) => write!(f, "RDF error: {}", msg),
            SyncError::ConformanceError(msg) => write!(f, "Conformance error: {}", msg),
        }
    }
}

impl std::error::Error for SyncError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_verb_differences() {
        let code = RdfVerbDefinition {
            verb_uri: "ex:LoadVerb".to_string(),
            name: "load".to_string(),
            description: "Load data".to_string(),
            noun_uri: None,
            noun_name: None,
            arguments: vec![],
            return_type: "Result".to_string(),
            trait_bounds: vec![],
            docstring: String::new(),
            is_async: false,
        };

        let rdf = RdfVerbDefinition {
            verb_uri: "ex:LoadVerb".to_string(),
            name: "load".to_string(),
            description: "Load data from file".to_string(),
            noun_uri: None,
            noun_name: None,
            arguments: vec![],
            return_type: "Result".to_string(),
            trait_bounds: vec![],
            docstring: String::new(),
            is_async: false,
        };

        let diffs = compute_verb_differences(&code, &rdf);
        assert!(!diffs.is_empty());
        assert!(diffs[0].contains("Description"));
    }

    #[test]
    fn test_parse_ntriples() {
        let ntriples = r#"
<http://example.org/LoadVerb> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Verb> .
<http://example.org/LoadVerb> <http://clap-noun-verb.io/ontology#hasVerbName> "load" .
        "#;

        let triples = parse_ntriples(ntriples).unwrap();
        assert_eq!(triples.len(), 2);
    }
}
