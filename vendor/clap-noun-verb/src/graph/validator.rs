// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Validate Command - Validate RDF file syntax and structure

use serde::{Deserialize, Serialize};
use std::fs;

/// Result from graph validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResultOutput {
    pub valid: bool,
    pub errors: Vec<ValidationErrorOutput>,
    pub total_triples: usize,
    pub valid_triples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorOutput {
    pub triple_index: usize,
    pub message: String,
}

impl ValidationResultOutput {
    pub fn new(total_triples: usize) -> Self {
        Self { valid: true, errors: Vec::new(), total_triples, valid_triples: total_triples }
    }

    pub fn add_error(&mut self, idx: usize, msg: impl Into<String>) {
        self.valid = false;
        self.valid_triples = self.valid_triples.saturating_sub(1);
        self.errors.push(ValidationErrorOutput { triple_index: idx, message: msg.into() });
    }
}

/// Domain logic: Validate file content
fn validate_file_content(content: &str) -> (usize, Vec<(usize, String)>) {
    let mut total_triples = 0;
    let mut errors = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        total_triples += 1;

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            errors.push((idx + 1, "Triple must have subject, predicate, and object".to_string()));
        } else if !parts[0].starts_with('<') && !parts[0].starts_with('_') {
            errors.push((
                idx + 1,
                "Subject must be URI (start with <) or blank node (start with _)".to_string(),
            ));
        } else if !parts[1].starts_with('<') && !parts[1].starts_with('_') {
            errors.push((
                idx + 1,
                "Predicate must be URI (start with <) or qualified name".to_string(),
            ));
        }
    }

    (total_triples, errors)
}

/// Domain logic: Load and validate file
fn validate_graph_impl(path: &str) -> crate::Result<(usize, Vec<(usize, String)>)> {
    if !std::path::Path::new(path).exists() {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "File not found: {}",
            path
        )));
    }

    let content = fs::read_to_string(path).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Failed to read file: {}", e))
    })?;

    Ok(validate_file_content(&content))
}

/// Validate RDF graph syntax and structure
///
/// Performs structural validation on an RDF file:
/// - Checks for valid Turtle syntax
/// - Validates that all triples have subject, predicate, object
/// - Ensures URI format compliance
/// - Reports all validation errors with line numbers
///
/// Uses the generated validator implementation for optimized validation.
///
/// # Arguments
/// * `path` - Path to RDF file to validate
///
/// # Example
/// ```text
/// myapp graph validate data/example.ttl
/// ```
pub fn validate_graph(path: String) -> crate::Result<ValidationResultOutput> {
    let validator = super::impl_generated::GeneratedValidator::new();
    let (total_triples, errors) = validator.validate_file(&path)?;

    let mut result = ValidationResultOutput::new(total_triples);
    for error in errors {
        result.add_error(error.triple_index, error.message);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_content_valid() {
        let content = "<s1> <p1> <o1> .\n<s2> <p2> <o2> .\n";
        let (total, errors) = validate_file_content(content);
        assert_eq!(total, 2);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_content_with_errors() {
        let content = "<s1> <p1>\n<s2> <p2> <o2> .\n";
        let (total, errors) = validate_file_content(content);
        assert_eq!(total, 2);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validation_result_output() {
        let mut output = ValidationResultOutput::new(10);
        assert!(output.valid);
        assert_eq!(output.valid_triples, 10);

        output.add_error(0, "Invalid subject");
        assert!(!output.valid);
        assert_eq!(output.valid_triples, 9);
        assert_eq!(output.errors.len(), 1);
    }
}
