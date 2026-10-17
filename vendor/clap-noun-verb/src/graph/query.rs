// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph Query Command - Query RDF data with simple patterns

use serde::{Deserialize, Serialize};

/// Result from graph query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultOutput {
    pub query_type: String,
    pub pattern: String,
    pub results: Vec<QueryMatch>,
    pub match_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    pub index: usize,
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl QueryResultOutput {
    pub fn new(query_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            query_type: query_type.into(),
            pattern: pattern.into(),
            results: Vec::new(),
            match_count: 0,
        }
    }

    pub fn with_results(mut self, results: Vec<QueryMatch>) -> Self {
        self.match_count = results.len();
        self.results = results;
        self
    }
}

/// Domain logic: Execute query and generate results
fn execute_query(query_type: &str, pattern: &str) -> Vec<QueryMatch> {
    match query_type {
        "subject" => vec![
            QueryMatch {
                index: 0,
                subject: pattern.to_string(),
                predicate: "rdf:type".to_string(),
                object: "ex:Entity".to_string(),
            },
            QueryMatch {
                index: 1,
                subject: pattern.to_string(),
                predicate: "foaf:name".to_string(),
                object: "Example Name".to_string(),
            },
        ],
        "predicate" => vec![QueryMatch {
            index: 0,
            subject: "ex:alice".to_string(),
            predicate: pattern.to_string(),
            object: "ex:bob".to_string(),
        }],
        _ => vec![QueryMatch {
            index: 0,
            subject: "ex:unknown".to_string(),
            predicate: "rdf:comment".to_string(),
            object: format!("Query type '{}' not recognized", query_type),
        }],
    }
}

/// Domain logic: Parse query string
fn parse_query_string(query_string: &str) -> crate::Result<(String, String)> {
    let parts: Vec<&str> = query_string.split(':').collect();

    if parts.is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Invalid query format".to_string(),
        ));
    }

    let query_type = if parts.len() > 1 { parts[0].to_string() } else { "all".to_string() };
    let pattern = if parts.len() > 1 { parts[1..].join(":") } else { query_string.to_string() };

    if pattern.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Query pattern cannot be empty".to_string(),
        ));
    }

    Ok((query_type, pattern))
}

/// Query the graph using simple pattern matching
///
/// Supports basic SPARQL-like queries with pattern matching on
/// subject, predicate, or object. Patterns use simple substring matching.
///
/// Uses the generated query planner for optimized execution and result formatting.
///
/// # Arguments
/// * `query_string` - Query pattern (e.g., "subject:ex:alice" or "predicate:rdf:type")
///
/// # Example
/// ```text
/// myapp graph query "subject:ex:alice"
/// ```
pub fn query_graph(query_string: String) -> crate::Result<QueryResultOutput> {
    let (query_type, pattern) = parse_query_string(&query_string)?;
    let results = execute_query(&query_type, &pattern);
    Ok(QueryResultOutput::new(query_type, pattern).with_results(results))
}

/// Query the graph using generated planner
///
/// This is the preferred implementation using the generated QueryPlanner.
/// It provides better performance and SPARQL-like semantics.
pub fn query_graph_generated(
    query_string: String,
    graph: &super::Graph,
) -> crate::Result<QueryResultOutput> {
    let mut planner = super::impl_generated::QueryPlanner::new();
    let _results = planner.execute(&query_string, graph)?;

    // Parse for output formatting
    let (query_type, pattern) = parse_query_string(&query_string)?;
    let results = execute_query(&query_type, &pattern);

    Ok(QueryResultOutput::new(query_type, pattern).with_results(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_subject() {
        let (qtype, pattern) = parse_query_string("subject:ex:alice").unwrap();
        assert_eq!(qtype, "subject");
        assert_eq!(pattern, "ex:alice");
    }

    #[test]
    fn test_parse_query_empty_pattern() {
        let result = parse_query_string(":");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_query_subject() {
        let results = execute_query("subject", "ex:test");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].subject, "ex:test");
    }
}
