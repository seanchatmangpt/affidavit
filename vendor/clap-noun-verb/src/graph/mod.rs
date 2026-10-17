// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RDF Graph module - Load, query, and validate RDF data
//!
//! Provides core types and operations for RDF triple management and querying.

pub mod impl_generated;
pub mod loader;
pub mod query;
pub mod validator;

pub use impl_generated::{GeneratedLoader, GeneratedValidator, QueryPlanner};
pub use loader::GraphLoadedOutput;
pub use query::QueryResultOutput;
pub use validator::ValidationResultOutput;

use serde::{Deserialize, Serialize};

/// Represents an RDF triple: subject-predicate-object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Triple {
    /// Create a new RDF triple
    pub fn new(
        subject: impl Into<String>,
        predicate: impl Into<String>,
        object: impl Into<String>,
    ) -> Self {
        Self { subject: subject.into(), predicate: predicate.into(), object: object.into() }
    }

    /// Validate that triple components are non-empty
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.subject.trim().is_empty() {
            return Err("Subject cannot be empty".to_string());
        }
        if self.predicate.trim().is_empty() {
            return Err("Predicate cannot be empty".to_string());
        }
        if self.object.trim().is_empty() {
            return Err("Object cannot be empty".to_string());
        }
        Ok(())
    }
}

/// In-memory RDF graph store
#[derive(Debug, Clone)]
pub struct Graph {
    triples: Vec<Triple>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self { triples: Vec::new() }
    }

    /// Create a graph from triples
    pub fn from_triples(triples: Vec<Triple>) -> Self {
        Self { triples }
    }

    /// Add a triple to the graph
    pub fn add_triple(&mut self, triple: Triple) -> std::result::Result<(), String> {
        triple.validate()?;
        self.triples.push(triple);
        Ok(())
    }

    /// Get all triples
    pub fn triples(&self) -> &[Triple] {
        &self.triples
    }

    /// Query graph by subject pattern (simple substring match)
    pub fn query_by_subject(&self, pattern: &str) -> Vec<Triple> {
        self.triples.iter().filter(|t| t.subject.contains(pattern)).cloned().collect()
    }

    /// Query graph by predicate pattern
    pub fn query_by_predicate(&self, pattern: &str) -> Vec<Triple> {
        self.triples.iter().filter(|t| t.predicate.contains(pattern)).cloned().collect()
    }

    /// Get triple count
    pub fn len(&self) -> usize {
        self.triples.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.triples.is_empty()
    }

    /// Validate all triples in the graph
    pub fn validate_all(&self) -> Vec<ValidationError> {
        self.triples
            .iter()
            .enumerate()
            .filter_map(|(idx, triple)| {
                triple
                    .validate()
                    .err()
                    .map(|msg| ValidationError { triple_index: idx, message: msg })
            })
            .collect()
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// SPARQL-style query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub variable: String,
    pub values: Vec<String>,
}

impl QueryResult {
    pub fn new(variable: impl Into<String>, values: Vec<String>) -> Self {
        Self { variable: variable.into(), values }
    }
}

/// Validation error for RDF content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub triple_index: usize,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triple_creation() {
        let triple = Triple::new("subject", "predicate", "object");
        assert_eq!(triple.subject, "subject");
        assert_eq!(triple.predicate, "predicate");
        assert_eq!(triple.object, "object");
    }

    #[test]
    fn test_triple_validation() {
        let valid = Triple::new("s", "p", "o");
        assert!(valid.validate().is_ok());

        let invalid = Triple::new("", "p", "o");
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_graph_operations() {
        let mut graph = Graph::new();
        assert!(graph.is_empty());

        let triple = Triple::new("ex:person", "rdf:type", "ex:Person");
        graph.add_triple(triple.clone()).unwrap();

        assert_eq!(graph.len(), 1);
        assert_eq!(graph.triples()[0], triple);
    }

    #[test]
    fn test_graph_query() {
        let mut graph = Graph::new();
        graph.add_triple(Triple::new("ex:alice", "rdf:type", "ex:Person")).unwrap();
        graph.add_triple(Triple::new("ex:bob", "rdf:type", "ex:Person")).unwrap();
        graph.add_triple(Triple::new("ex:alice", "foaf:knows", "ex:bob")).unwrap();

        let results = graph.query_by_subject("alice");
        assert_eq!(results.len(), 2);
    }
}
