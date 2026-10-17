// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ggen → RDF generator: converts Rust #[verb] source code to RDF/N-Triples ontology
//!
//! This module parses Rust source code containing #[verb] macro invocations
//! and emits W3C-compliant RDF triples describing the verbs.
//!
//! The generated RDF uses:
//! - Namespace: `http://clap-noun-verb.io/ontology#` (cnv:)
//! - Format: N-Triples (.nt) with full URI expansion
//! - Includes: verb names, descriptions, parameters, return types, trait bounds

use crate::rdf_to_ggen::{ArgumentType, RdfArgumentDefinition, RdfVerbDefinition};
use regex::Regex;
use std::collections::HashSet;

// =============================================================================
// PARSING RUST SOURCE
// =============================================================================

/// Parse a Rust source file for #[verb] functions
///
/// Extracts:
/// - Function name (converted to verb name)
/// - Docstring (becomes rdfs:comment)
/// - Parameter types (become cnv:hasArguments)
/// - Return type (becomes cnv:returnType)
/// - Trait bounds (if present)
///
/// # Example
/// ```ignore
/// let rust_code = r#"
/// /// Load a graph from file
/// #[verb("load")]
/// pub fn graph_load(path: String, format: Option<String>) -> Result<GraphLoadedOutput> {
///     // ...
/// }
/// "#;
///
/// let verbs = parse_rust_source(rust_code)?;
/// assert_eq!(verbs.len(), 1);
/// assert_eq!(verbs[0].name, "load");
/// ```
pub fn parse_rust_source(source: &str) -> Result<Vec<RdfVerbDefinition>, ParseError> {
    let mut verbs = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let _line = lines[i].trim();

        // Look for docstring (/// comments)
        let mut doc_lines = Vec::new();
        while i < lines.len() && lines[i].trim().starts_with("///") {
            let doc = lines[i]
                .trim_start_matches(|c: char| c.is_whitespace())
                .trim_start_matches("///")
                .trim();
            doc_lines.push(doc.to_string());
            i += 1;
        }
        let docstring = doc_lines.join(" ");

        // Look for #[verb(...)] attribute
        if i < lines.len() && lines[i].contains("#[verb(") {
            let attr_line = lines[i];
            if let Some(verb_name) = extract_verb_name(attr_line) {
                // Get function signature
                let mut signature = String::new();
                while i < lines.len() && !lines[i].contains('{') {
                    signature.push_str(lines[i]);
                    signature.push(' ');
                    i += 1;
                }
                if i < lines.len() {
                    signature.push_str(lines[i]);
                    i += 1;
                }

                // Parse the function
                match parse_function_signature(&signature, &verb_name) {
                    Ok(mut verb) => {
                        verb.docstring = docstring.clone();
                        verbs.push(verb);
                    }
                    Err(e) => {
                        eprintln!("Failed to parse signature for {}: {:?}", verb_name, e);
                        eprintln!("Signature was: {}", signature);
                    }
                }
            }
        }

        i += 1;
    }

    Ok(verbs)
}

/// Extract verb name from #[verb("name")] attribute
fn extract_verb_name(attr_line: &str) -> Option<String> {
    let re = Regex::new(r#"#\[verb\("([^"]*)"\)\]"#).ok()?;
    re.captures(attr_line).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())
}

/// Parse a Rust function signature
///
/// Extracts:
/// - Function name
/// - Parameters (name, type)
/// - Return type
/// - async/sync
fn parse_function_signature(
    signature: &str,
    verb_name: &str,
) -> Result<RdfVerbDefinition, ParseError> {
    // Normalize whitespace - join lines into single signature
    let normalized_sig = signature
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Extract function name (between "fn" and "(")
    let fn_regex = Regex::new(r"(?:async\s+)?(?:pub\s+)?fn\s+(\w+)\s*\(")
        .map_err(|_| ParseError::InvalidSignature)?;

    let function_name = fn_regex
        .captures(&normalized_sig)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .unwrap_or("unknown");

    // Determine if async
    let is_async = normalized_sig.contains("async");

    // Extract return type
    let return_type =
        extract_return_type(&normalized_sig).unwrap_or_else(|| "serde_json::Value".to_string());

    // Extract parameters
    let arguments = extract_parameters(&normalized_sig)?;

    // Derive noun name from function name (e.g., "graph_load" → "graph")
    let noun_name = if function_name.contains('_') {
        let parts: Vec<&str> = function_name.split('_').collect();
        if parts.len() > 1 {
            Some(parts[0].to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok(RdfVerbDefinition {
        verb_uri: format!("ex:{}Verb", capitalize_first(verb_name)),
        name: verb_name.to_string(),
        description: String::new(),
        noun_uri: noun_name.as_ref().map(|n| format!("ex:{}Noun", capitalize_first(n))),
        noun_name,
        arguments,
        return_type,
        trait_bounds: extract_trait_bounds(signature),
        docstring: String::new(),
        is_async,
    })
}

/// Extract return type from function signature
/// Handles: Result<Type>, async Result<Type>, -> Type, etc.
fn extract_return_type(signature: &str) -> Option<String> {
    let re = Regex::new(r"->(\s*(?:impl\s+)?[^{]+?)(?:\s*\{|$)").ok()?;
    re.captures(signature).and_then(|cap| cap.get(1)).map(|m| m.as_str().trim().to_string())
}

/// Extract function parameters
fn extract_parameters(signature: &str) -> Result<Vec<RdfArgumentDefinition>, ParseError> {
    let mut parameters = Vec::new();

    // Extract content between parentheses
    let start = signature.find('(').ok_or(ParseError::InvalidSignature)?;

    // Find closing paren - scan forward and match parens
    let mut paren_depth = 0;
    let mut end = start;
    for (i, ch) in signature[start..].chars().enumerate() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }

    if end == start {
        return Err(ParseError::InvalidSignature);
    }

    let params_str = &signature[start + 1..end];

    // Split by comma and process each parameter
    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() || param == "args: VerbArgs" {
            continue;
        }

        // Match: name: Type
        if let Some((name, type_str)) = param.split_once(':') {
            let name = name.trim().to_string();
            let type_str = type_str.trim();

            let (is_optional, value_type) = if type_str.starts_with("Option<") {
                let inner = &type_str[7..];
                (true, inner.trim_end_matches('>').to_string())
            } else {
                (false, type_str.to_string())
            };

            let is_flag = type_str == "bool" || type_str == "Option<bool>";

            parameters.push(RdfArgumentDefinition {
                arg_uri: format!("ex:{}Arg", capitalize_first(&name)),
                name: name.clone(),
                description: String::new(),
                value_type,
                required: !is_optional && !is_flag,
                is_flag,
                default_value: None,
                short_name: None,
                long_name: None,
                allowed_values: vec![],
                argument_type: if is_flag {
                    ArgumentType::Flag
                } else if is_optional {
                    ArgumentType::Optional
                } else {
                    ArgumentType::Positional
                },
            });
        }
    }

    Ok(parameters)
}

/// Extract trait bounds from function signature
/// Looks for generic trait bounds: <T: Send + Sync>
fn extract_trait_bounds(signature: &str) -> Vec<String> {
    let mut bounds = HashSet::new();

    // Look for common trait bounds in return types
    if signature.contains("Send") {
        bounds.insert("Send".to_string());
    }
    if signature.contains("Sync") {
        bounds.insert("Sync".to_string());
    }
    if signature.contains("Serialize") {
        bounds.insert("Serialize".to_string());
    }
    if signature.contains("Deserialize") {
        bounds.insert("Deserialize".to_string());
    }

    bounds.into_iter().collect()
}

// =============================================================================
// RDF EMISSION - Convert verbs to N-Triples
// =============================================================================

/// Generate N-Triples (RDF) from verb definitions
///
/// Output format: W3C N-Triples (.nt)
/// - Full URI expansion (no prefixes)
/// - One triple per line
/// - Language tags for strings (e.g., "label"@en)
/// - Typed literals for non-strings (e.g., "true"^^xsd:boolean)
///
/// # Output Example
/// ```ntriples
/// <http://example.org/LoadVerb> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Verb> .
/// <http://example.org/LoadVerb> <http://clap-noun-verb.io/ontology#hasVerbName> "load"@en .
/// <http://example.org/LoadVerb> <http://clap-noun-verb.io/ontology#verbAbout> "Load a graph from file"@en .
/// ```
pub fn verb_definitions_to_ntriples(verbs: &[RdfVerbDefinition]) -> String {
    let mut ntriples = String::new();

    // Ontology header
    ntriples.push_str("# Auto-generated from Rust source code\n");
    ntriples.push_str("# Generated by ggen_to_rdf converter\n");
    ntriples.push_str("# Format: N-Triples (RDF 1.1)\n");
    ntriples.push_str("# Timestamp: ");
    ntriples.push_str(
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    );
    ntriples.push_str("\n\n");

    // Generate triples for each verb
    for verb in verbs {
        ntriples.push_str(&generate_verb_triples(verb));
        ntriples.push('\n');
    }

    ntriples
}

/// Generate N-Triples for a single verb definition
fn generate_verb_triples(verb: &RdfVerbDefinition) -> String {
    let mut triples = String::new();
    let verb_uri = &verb.verb_uri;

    // Type declaration
    triples.push_str(&format!(
        "<{}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Verb> .\n",
        verb_uri
    ));

    // Verb name
    triples.push_str(&format!(
        "<{}> <http://clap-noun-verb.io/ontology#hasVerbName> \"{}\"@en .\n",
        verb_uri, verb.name
    ));

    // Verb description
    if !verb.description.is_empty() {
        let escaped = escape_rdf_string(&verb.description);
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#verbAbout> \"{}\"@en .\n",
            verb_uri, escaped
        ));
    }

    // Docstring (if different from description)
    if !verb.docstring.is_empty() && verb.docstring != verb.description {
        let escaped = escape_rdf_string(&verb.docstring);
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#docstring> \"{}\"@en .\n",
            verb_uri, escaped
        ));
    }

    // Belongs to noun
    if let Some(noun_uri) = &verb.noun_uri {
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#belongsToNoun> <{}> .\n",
            verb_uri, noun_uri
        ));
    }

    // Return type
    triples.push_str(&format!(
        "<{}> <http://clap-noun-verb.io/ontology#returnType> \"{}\"@en .\n",
        verb_uri, verb.return_type
    ));

    // Async flag
    if verb.is_async {
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#isAsync> \"true\"^^<http://www.w3.org/2001/XMLSchema#boolean> .\n",
            verb_uri
        ));
    }

    // Arguments
    for arg in verb.arguments.iter() {
        let arg_uri = &arg.arg_uri;

        // Type
        triples.push_str(&format!(
            "<{}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Argument> .\n",
            arg_uri
        ));

        // Name
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#hasArgumentName> \"{}\"@en .\n",
            arg_uri, arg.name
        ));

        // Description
        if !arg.description.is_empty() {
            let escaped = escape_rdf_string(&arg.description);
            triples.push_str(&format!(
                "<{}> <http://clap-noun-verb.io/ontology#argumentAbout> \"{}\"@en .\n",
                arg_uri, escaped
            ));
        }

        // Value type
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#valueType> \"{}\"@en .\n",
            arg_uri, arg.value_type
        ));

        // Required flag
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#required> \"{}\"^^<http://www.w3.org/2001/XMLSchema#boolean> .\n",
            arg_uri, arg.required
        ));

        // Argument type
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#argumentType> <http://clap-noun-verb.io/ontology#{:?}> .\n",
            arg_uri, arg.argument_type
        ));

        // Link to verb
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#hasArguments> <{}> .\n",
            verb_uri, arg_uri
        ));
    }

    // Trait bounds
    for trait_bound in &verb.trait_bounds {
        triples.push_str(&format!(
            "<{}> <http://clap-noun-verb.io/ontology#HasTraitBound> <http://clap-noun-verb.io/ontology#{}> .\n",
            verb_uri, trait_bound
        ));
    }

    triples
}

/// Generate SPARQL INSERT DATA statement (with prefixes)
/// Useful for loading into RDF databases interactively
pub fn verb_definitions_to_sparql_insert(verbs: &[RdfVerbDefinition]) -> String {
    let mut sparql = String::new();
    sparql.push_str("PREFIX cnv: <http://clap-noun-verb.io/ontology#>\n");
    sparql.push_str("PREFIX ex: <http://example.org/>\n");
    sparql.push_str("PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n");
    sparql.push_str("PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\n");
    sparql.push_str("INSERT DATA {\n");

    for verb in verbs {
        sparql.push_str(&format!(
            "  ex:{} a cnv:Verb ;\n",
            verb.verb_uri.split('/').next_back().unwrap_or("Verb")
        ));
        sparql.push_str(&format!("    cnv:hasVerbName \"{}\" ;\n", verb.name));

        if !verb.description.is_empty() {
            let escaped = escape_rdf_string(&verb.description);
            sparql.push_str(&format!("    cnv:verbAbout \"{}\" ;\n", escaped));
        }

        if let Some(noun_uri) = &verb.noun_uri {
            sparql.push_str(&format!(
                "    cnv:belongsToNoun ex:{} ;\n",
                noun_uri.split('/').next_back().unwrap_or("Noun")
            ));
        }

        sparql.push_str(&format!("    cnv:returnType \"{}\" ;\n", verb.return_type));

        if verb.is_async {
            sparql.push_str("    cnv:isAsync \"true\"^^xsd:boolean ;\n");
        }

        for arg in &verb.arguments {
            sparql.push_str(&format!(
                "    cnv:hasArguments ex:{} ;\n",
                arg.arg_uri.split('/').next_back().unwrap_or("Arg")
            ));
        }

        sparql.push_str("  .\n");

        // Arguments
        for arg in &verb.arguments {
            sparql.push_str(&format!(
                "  ex:{} a cnv:Argument ;\n",
                arg.arg_uri.split('/').next_back().unwrap_or("Arg")
            ));
            sparql.push_str(&format!("    cnv:hasArgumentName \"{}\" ;\n", arg.name));
            sparql.push_str(&format!("    cnv:valueType \"{}\" ;\n", arg.value_type));
            sparql.push_str(&format!("    cnv:required \"{}\"^^xsd:boolean ;\n", arg.required));
            sparql.push_str("  .\n");
        }
    }

    sparql.push_str("}\n");
    sparql
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn escape_rdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// =============================================================================
// ERROR TYPES
// =============================================================================

#[derive(Debug)]
pub enum ParseError {
    InvalidSignature,
    MissingParameter,
    InvalidReturnType,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidSignature => write!(f, "Invalid function signature"),
            ParseError::MissingParameter => write!(f, "Missing parameter"),
            ParseError::InvalidReturnType => write!(f, "Invalid return type"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_source_simple() {
        let source = r#"
/// Load a graph from file
#[verb("load")]
pub fn graph_load(path: String, format: Option<String>) -> Result<GraphLoadedOutput> {
    unimplemented!()
}
"#;

        let verbs = parse_rust_source(source).unwrap();
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs[0].name, "load");
        assert_eq!(verbs[0].return_type, "Result<GraphLoadedOutput>");
    }

    #[test]
    fn test_extract_verb_name() {
        let attr = r#"#[verb("status")]"#;
        assert_eq!(extract_verb_name(attr), Some("status".to_string()));
    }

    #[test]
    fn test_verb_definitions_to_ntriples() {
        let verb = RdfVerbDefinition {
            verb_uri: "ex:LoadVerb".to_string(),
            name: "load".to_string(),
            description: "Load data".to_string(),
            noun_uri: Some("ex:GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![],
            return_type: "LoadResult".to_string(),
            trait_bounds: vec!["Send".to_string(), "Sync".to_string()],
            docstring: String::new(),
            is_async: false,
        };

        let ntriples = verb_definitions_to_ntriples(&[verb]);
        // N-Triples format expands all URIs fully (no prefix abbreviations)
        assert!(ntriples.contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>"));
        assert!(ntriples.contains("<http://clap-noun-verb.io/ontology#Verb>"));
        assert!(ntriples.contains("\"load\"@en"));
        assert!(ntriples.contains("HasTraitBound"));
    }
}
