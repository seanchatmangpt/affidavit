// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Load Command - Load RDF data from file

use serde::{Deserialize, Serialize};
use std::fs;

/// Result from graph load operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLoadedOutput {
    pub triples_loaded: usize,
    pub source: String,
    pub status: String,
}

impl GraphLoadedOutput {
    pub fn new(triples_loaded: usize, source: impl Into<String>) -> Self {
        Self { triples_loaded, source: source.into(), status: "success".to_string() }
    }
}

/// Domain logic: Parse and count triples from file content
fn parse_triples_from_content(content: &str) -> usize {
    let mut triple_count = 0;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Very simple parser: split by whitespace and take first 3 tokens
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0].starts_with('<') && parts[1].starts_with('<') {
            triple_count += 1;
        }
    }
    triple_count
}

/// Domain logic: Load graph from file
fn load_graph_impl(path: &str) -> crate::Result<(usize, String)> {
    if !std::path::Path::new(path).exists() {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "File not found: {}",
            path
        )));
    }

    let content = fs::read_to_string(path).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Failed to read file: {}", e))
    })?;

    let triple_count = parse_triples_from_content(&content);

    if triple_count == 0 {
        return Err(crate::error::NounVerbError::execution_error(
            "No valid triples found in file".to_string(),
        ));
    }

    Ok((triple_count, path.to_string()))
}

/// Load RDF graph from a Turtle file
///
/// Reads a Turtle (.ttl) format RDF file and loads triples into the graph.
/// Files must contain valid N-Triples format with subject, predicate, object URIs.
///
/// Uses the generated loader implementation for optimized parsing and validation.
///
/// # Arguments
/// * `path` - Path to the Turtle format RDF file
///
/// # Example
/// ```text
/// myapp graph load data/example.ttl
/// ```
pub fn load_graph(path: String) -> crate::Result<GraphLoadedOutput> {
    let loader = super::impl_generated::GeneratedLoader::new();
    let (_graph, triple_count, source) = loader.load_from_file(&path)?;
    Ok(GraphLoadedOutput::new(triple_count, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_triples_from_content() {
        let content = "<s1> <p1> <o1> .\n<s2> <p2> <o2> .\n# comment\n";
        assert_eq!(parse_triples_from_content(content), 2);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_graph_impl("nonexistent.ttl");
        assert!(result.is_err());
    }
}
