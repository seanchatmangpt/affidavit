// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RDF → ggen generator: converts RDF verb definitions to compilable Rust #[verb] code
//!
//! This module reads RDF triples describing verbs (from ontologies or SPARQL results)
//! and emits ready-to-compile Rust code using the #[verb] macro.
//!
//! Example RDF input:
//! ```turtle
//! ex:LoadGraphVerb a cnv:Verb ;
//!     cnv:hasVerbName "load" ;
//!     cnv:verbAbout "Load a graph from file" ;
//!     cnv:belongsToNoun ex:GraphNoun ;
//!     cnv:hasArguments (ex:PathArg ex:FormatArg) ;
//!     cnv:returnType "GraphLoadedOutput" ;
//!     cnv:HasTraitBound cnv:Send ;
//!     cnv:HasTraitBound cnv:Sync ;
//!     cnv:HasTraitBound cnv:Serialize .
//! ```
//!
//! Generated output:
//! ```rust,ignore
//! /// Load a graph from file
//! #[verb("load")]
//! pub fn graph_load(
//!     path: String,
//!     format: Option<String>,
//! ) -> Result<GraphLoadedOutput> {
//!     // Auto-generated handler skeleton
//!     todo!("Implement load verb handler")
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RDF triple representation (flattened from SPARQL/TTL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default)]
    pub object_type: ObjectType, // DatatypeProperty vs ObjectProperty
}

/// Type of RDF object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    /// Literal value (string, number, boolean, date)
    Literal,
    /// URI reference (relationship to another resource)
    Reference,
}

impl Default for ObjectType {
    fn default() -> Self {
        ObjectType::Literal
    }
}

/// RDF verb definition extracted from ontology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfVerbDefinition {
    /// Verb URI (e.g., "ex:LoadGraphVerb")
    pub verb_uri: String,

    /// Verb name (e.g., "load")
    pub name: String,

    /// Description (e.g., "Load a graph from file")
    pub description: String,

    /// Parent noun URI (e.g., "ex:GraphNoun")
    #[serde(default)]
    pub noun_uri: Option<String>,

    /// Noun name extracted from URI
    #[serde(default)]
    pub noun_name: Option<String>,

    /// Arguments
    #[serde(default)]
    pub arguments: Vec<RdfArgumentDefinition>,

    /// Return type (e.g., "GraphLoadedOutput")
    pub return_type: String,

    /// Required trait bounds (Send, Sync, Serialize, Deserialize, etc.)
    #[serde(default)]
    pub trait_bounds: Vec<String>,

    /// Documentation docstring
    #[serde(default)]
    pub docstring: String,

    /// Async function
    #[serde(default)]
    pub is_async: bool,
}

/// RDF argument/parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfArgumentDefinition {
    /// Argument URI (e.g., "ex:PathArg")
    pub arg_uri: String,

    /// Argument name (e.g., "path")
    pub name: String,

    /// Description
    pub description: String,

    /// Rust type (e.g., "String", "u16", "Vec<String>")
    pub value_type: String,

    /// Is required (positional args are typically true)
    pub required: bool,

    /// Is flag (boolean flag)
    pub is_flag: bool,

    /// Optional default value
    #[serde(default)]
    pub default_value: Option<String>,

    /// Short flag (e.g., Some('v'))
    #[serde(default)]
    pub short_name: Option<char>,

    /// Long flag name (different from `name` if specified)
    #[serde(default)]
    pub long_name: Option<String>,

    /// Allowed values (for enums)
    #[serde(default)]
    pub allowed_values: Vec<String>,

    /// Argument classification
    pub argument_type: ArgumentType,
}

/// Argument type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ArgumentType {
    /// Positional required argument
    Positional,
    /// Optional named argument
    Optional,
    /// Boolean flag
    Flag,
    /// Can be repeated multiple times
    Repeating,
    /// Variadic (collects remaining args)
    Variadic,
}

impl Default for ArgumentType {
    fn default() -> Self {
        ArgumentType::Positional
    }
}

// =============================================================================
// SPARQL RESULT PARSING - Convert SPARQL JSON results to RDF triples
// =============================================================================

/// SPARQL JSON result binding
#[derive(Debug, Deserialize)]
pub struct SparqlBinding {
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub r#type: String, // "uri", "literal"
}

/// SPARQL JSON results
#[derive(Debug, Deserialize)]
pub struct SparqlResults {
    pub results: SparqlResultList,
}

#[derive(Debug, Deserialize)]
pub struct SparqlResultList {
    pub bindings: Vec<HashMap<String, SparqlBinding>>,
}

// =============================================================================
// MAIN GENERATOR FUNCTION
// =============================================================================

/// Convert RDF verb definition(s) to compilable Rust code
///
/// # Input
/// Either:
/// - SPARQL JSON results containing verb definitions
/// - Pre-parsed RdfVerbDefinition struct
///
/// # Output
/// Ready-to-compile Rust #[verb] function signature + skeleton handler
///
/// # Example
/// ```ignore
/// let rdf_def = RdfVerbDefinition {
///     verb_uri: "ex:LoadGraphVerb".to_string(),
///     name: "load".to_string(),
///     description: "Load a graph from file".to_string(),
///     noun_name: Some("graph".to_string()),
///     arguments: vec![/* ... */],
///     return_type: "GraphLoadedOutput".to_string(),
///     trait_bounds: vec!["Send".to_string(), "Sync".to_string(), "Serialize".to_string()],
///     docstring: "Load a graph from file or stdin".to_string(),
///     is_async: false,
/// };
/// let code = rdf_spec_to_verb_code(&rdf_def);
/// println!("{}", code);
/// ```
pub fn rdf_spec_to_verb_code(verb: &RdfVerbDefinition) -> String {
    let mut code = String::new();

    // Add docstring
    if !verb.docstring.is_empty() {
        code.push_str(&format!("/// {}\n", verb.docstring));
    } else if !verb.description.is_empty() {
        code.push_str(&format!("/// {}\n", verb.description));
    }

    // Add #[verb] attribute
    code.push_str(&format!("#[verb(\"{}\")]\n", verb.name));

    // Add function signature
    let async_keyword = if verb.is_async { "async " } else { "" };
    let function_name = if let Some(ref noun) = verb.noun_name {
        format!("{}_{}", noun, verb.name)
    } else {
        verb.name.clone()
    };

    code.push_str(&format!("pub {}fn {}(\n", async_keyword, function_name));

    // Add arguments
    if verb.arguments.is_empty() {
        code.push_str("    args: VerbArgs,\n");
    } else {
        for arg in &verb.arguments {
            let arg_type = if arg.is_flag {
                "bool".to_string()
            } else if arg.required {
                arg.value_type.clone()
            } else {
                format!("Option<{}>", arg.value_type)
            };

            code.push_str(&format!("    {}: {},\n", arg.name.replace('-', "_"), arg_type));
        }
    }

    // Add return type
    code.push_str(&format!(") -> Result<{}> {{\n", verb.return_type));

    // Add skeleton handler
    code.push_str("    // Auto-generated handler skeleton from RDF\n");
    code.push_str("    // TODO: Implement handler logic\n");
    code.push_str(&format!(
        "    unimplemented!(\"Handler for {} verb not yet implemented\")\n",
        verb.name
    ));

    code.push_str("}\n");

    code
}

/// Convert SPARQL JSON results to verb definitions
///
/// Expects SPARQL results with these bindings:
/// - ?verb: verb URI
/// - ?verbName: verb name
/// - ?verbAbout: verb description
/// - ?noun: noun URI (optional)
/// - ?nounName: noun name (optional)
/// - ?returnType: return type
/// - ?traitBound: trait bound (can appear multiple times)
///
/// # Example
/// ```json
/// {
///   "results": {
///     "bindings": [
///       {
///         "verb": {"type": "uri", "value": "http://example.org/LoadVerb"},
///         "verbName": {"type": "literal", "value": "load"},
///         "verbAbout": {"type": "literal", "value": "Load data"},
///         "returnType": {"type": "literal", "value": "LoadResult"},
///         "traitBound": {"type": "uri", "value": "http://clap-noun-verb.io/ontology#Send"}
///       }
///     ]
///   }
/// }
/// ```
pub fn sparql_results_to_verb_definitions(
    sparql_json: &str,
) -> Result<Vec<RdfVerbDefinition>, Box<dyn std::error::Error>> {
    let results: SparqlResults = serde_json::from_str(sparql_json)?;
    let mut verbs: HashMap<String, RdfVerbDefinition> = HashMap::new();

    for binding in results.results.bindings {
        // Extract verb URI (required)
        let verb_uri = binding.get("verb").ok_or("Missing ?verb binding")?.value.clone();

        // Extract verb name (required)
        let name = binding.get("verbName").ok_or("Missing ?verbName binding")?.value.clone();

        // Initialize verb if not seen before
        let verb = verbs.entry(verb_uri.clone()).or_insert_with(|| {
            let noun_name = binding
                .get("nounName")
                .map(|b| b.value.clone())
                .or_else(|| binding.get("noun").map(|b| b.value.clone()));

            RdfVerbDefinition {
                verb_uri: verb_uri.clone(),
                name,
                description: binding.get("verbAbout").map(|b| b.value.clone()).unwrap_or_default(),
                noun_uri: binding.get("noun").map(|b| b.value.clone()),
                noun_name,
                arguments: Vec::new(),
                return_type: binding
                    .get("returnType")
                    .map(|b| b.value.clone())
                    .unwrap_or_else(|| "serde_json::Value".to_string()),
                trait_bounds: Vec::new(),
                docstring: binding.get("docstring").map(|b| b.value.clone()).unwrap_or_default(),
                is_async: binding.get("isAsync").map(|b| b.value == "true").unwrap_or(false),
            }
        });

        // Collect trait bounds (can appear multiple times in SPARQL results)
        if let Some(trait_binding) = binding.get("traitBound") {
            let trait_name = trait_binding
                .value
                .split('#')
                .next_back()
                .unwrap_or(&trait_binding.value)
                .to_string();
            if !verb.trait_bounds.contains(&trait_name) {
                verb.trait_bounds.push(trait_name);
            }
        }
    }

    Ok(verbs.into_values().collect())
}

/// Batch convert RDF triples to verb definitions
///
/// Groups triples by subject (verb URI) and extracts properties.
///
/// Recognized predicates (cnv: namespace):
/// - hasVerbName → name
/// - verbAbout → description
/// - belongsToNoun → noun_uri
/// - hasArguments → arguments (list/collection)
/// - returnType → return_type
/// - HasTraitBound → trait_bounds (append)
/// - docstring → docstring
/// - isAsync → is_async
pub fn rdf_triples_to_verb_definitions(
    triples: Vec<RdfTriple>,
) -> Result<Vec<RdfVerbDefinition>, Box<dyn std::error::Error>> {
    let mut verb_map: HashMap<String, RdfVerbDefinition> = HashMap::new();

    for triple in triples {
        // Only process verb-related triples
        if !triple.predicate.contains("Verb") && !triple.predicate.contains("returnType") {
            continue;
        }

        // Initialize verb from subject
        let verb = verb_map.entry(triple.subject.clone()).or_insert_with(|| RdfVerbDefinition {
            verb_uri: triple.subject.clone(),
            name: String::new(),
            description: String::new(),
            noun_uri: None,
            noun_name: None,
            arguments: Vec::new(),
            return_type: "serde_json::Value".to_string(),
            trait_bounds: Vec::new(),
            docstring: String::new(),
            is_async: false,
        });

        // Match predicate and extract value
        match triple.predicate.as_str() {
            p if p.contains("hasVerbName") => {
                verb.name = triple.object.trim_matches('"').to_string();
            }
            p if p.contains("verbAbout") => {
                verb.description = triple.object.trim_matches('"').to_string();
            }
            p if p.contains("belongsToNoun") => {
                verb.noun_uri = Some(triple.object);
            }
            p if p.contains("returnType") => {
                verb.return_type = triple.object.trim_matches('"').to_string();
            }
            p if p.contains("HasTraitBound") => {
                let trait_name =
                    triple.object.split('#').next_back().unwrap_or(&triple.object).to_string();
                if !verb.trait_bounds.contains(&trait_name) {
                    verb.trait_bounds.push(trait_name);
                }
            }
            p if p.contains("docstring") => {
                verb.docstring = triple.object.trim_matches('"').to_string();
            }
            p if p.contains("isAsync") => {
                verb.is_async = triple.object.to_lowercase() == "true";
            }
            _ => {}
        }
    }

    Ok(verb_map.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdf_spec_to_verb_code_basic() {
        let verb = RdfVerbDefinition {
            verb_uri: "ex:LoadGraphVerb".to_string(),
            name: "load".to_string(),
            description: "Load a graph from file".to_string(),
            noun_uri: Some("ex:GraphNoun".to_string()),
            noun_name: Some("graph".to_string()),
            arguments: vec![
                RdfArgumentDefinition {
                    arg_uri: "ex:PathArg".to_string(),
                    name: "path".to_string(),
                    description: "File path".to_string(),
                    value_type: "String".to_string(),
                    required: true,
                    is_flag: false,
                    default_value: None,
                    short_name: None,
                    long_name: None,
                    allowed_values: vec![],
                    argument_type: ArgumentType::Positional,
                },
                RdfArgumentDefinition {
                    arg_uri: "ex:FormatArg".to_string(),
                    name: "format".to_string(),
                    description: "File format".to_string(),
                    value_type: "String".to_string(),
                    required: false,
                    is_flag: false,
                    default_value: Some("ttl".to_string()),
                    short_name: Some('f'),
                    long_name: None,
                    allowed_values: vec!["ttl".to_string(), "nt".to_string(), "rdf".to_string()],
                    argument_type: ArgumentType::Optional,
                },
            ],
            return_type: "GraphLoadedOutput".to_string(),
            trait_bounds: vec!["Send".to_string(), "Sync".to_string(), "Serialize".to_string()],
            docstring: "Load a graph from file or stdin".to_string(),
            is_async: false,
        };

        let code = rdf_spec_to_verb_code(&verb);
        assert!(code.contains("/// Load a graph from file or stdin"));
        assert!(code.contains("#[verb(\"load\")]"));
        assert!(code.contains("pub fn graph_load("));
        assert!(code.contains("path: String,"));
        assert!(code.contains("format: Option<String>,"));
        assert!(code.contains("Result<GraphLoadedOutput>"));
    }

    #[test]
    fn test_rdf_spec_to_verb_code_async() {
        let verb = RdfVerbDefinition {
            verb_uri: "ex:QueryVerb".to_string(),
            name: "query".to_string(),
            description: "Query the database".to_string(),
            noun_uri: Some("ex:DatabaseNoun".to_string()),
            noun_name: Some("database".to_string()),
            arguments: vec![],
            return_type: "QueryResult".to_string(),
            trait_bounds: vec!["Send".to_string(), "Sync".to_string()],
            docstring: String::new(),
            is_async: true,
        };

        let code = rdf_spec_to_verb_code(&verb);
        assert!(code.contains("pub async fn database_query("));
        assert!(code.contains("Result<QueryResult>"));
    }

    #[test]
    fn test_sparql_results_to_verb_definitions() {
        let sparql_json = r#"{
            "results": {
                "bindings": [
                    {
                        "verb": {"type": "uri", "value": "http://example.org/LoadVerb"},
                        "verbName": {"type": "literal", "value": "load"},
                        "verbAbout": {"type": "literal", "value": "Load data"},
                        "returnType": {"type": "literal", "value": "LoadResult"}
                    }
                ]
            }
        }"#;

        let verbs = sparql_results_to_verb_definitions(sparql_json).unwrap();
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs[0].name, "load");
        assert_eq!(verbs[0].return_type, "LoadResult");
    }
}
