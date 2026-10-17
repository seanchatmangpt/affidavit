// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for clap-noun-verb

use thiserror::Error;

/// Errors that can occur in the noun-verb CLI framework
#[derive(Error, Debug)]
pub enum NounVerbError {
    /// Command not found
    #[error("Command '{noun}' not found{suggestion}")]
    CommandNotFound { noun: String, suggestion: String },

    /// Verb not found for a given noun
    #[error("Verb '{verb}' not found for noun '{noun}'{suggestion}")]
    VerbNotFound { noun: String, verb: String, suggestion: String },

    /// Invalid command structure
    #[error("Invalid command structure: {message}")]
    InvalidStructure { message: String },

    /// Command execution error
    #[error("Command execution failed: {message}")]
    ExecutionError { message: String },

    /// Argument parsing error
    #[error("Argument parsing failed: {message}")]
    ArgumentError { message: String },

    /// Plugin-related error
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Middleware error
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    /// Telemetry error
    #[error("Telemetry error: {0}")]
    TelemetryError(String),

    /// Generic error wrapper
    #[error("Error: {0}")]
    Generic(String),
}

/// Helper function to calculate Levenshtein distance
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut cache: Vec<usize> = (1..=b_len).collect();
    let mut dist = 0;

    for (i, &ca) in a_chars.iter().enumerate() {
        let mut result = i;
        dist = i + 1;
        for (j, &cb) in b_chars.iter().enumerate() {
            let temp = result;
            result = cache[j];
            dist =
                if ca == cb { temp } else { std::cmp::min(std::cmp::min(result, dist), temp) + 1 };
            cache[j] = dist;
        }
    }

    dist
}

/// Find best suggestions from candidates based on Levenshtein distance
pub fn find_best_matches<'a>(input: &str, candidates: &[&'a str]) -> Vec<&'a str> {
    let mut with_distances: Vec<(&str, usize)> = candidates
        .iter()
        .map(|&c| (c, levenshtein_distance(input, c)))
        .filter(|&(_, dist)| dist <= 3 && dist < input.len())
        .collect();

    with_distances.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
    with_distances.into_iter().map(|(c, _)| c).collect()
}

impl NounVerbError {
    /// Enhance error with recovery suggestions from RDF guard validation
    ///
    /// Attempts to provide helpful suggestions using the RDF ontology and SPARQL queries.
    ///
    /// FUTURE: v5.1 - Complete RDF recovery suggestions
    pub fn with_recovery_suggestions(self) -> String {
        // RDF-control feature deferred to v5.1
        self.to_string()
    }

    /// Create a command not found error
    pub fn command_not_found(noun: impl Into<String>) -> Self {
        Self::CommandNotFound { noun: noun.into(), suggestion: String::new() }
    }

    /// Create a command not found error with suggestion candidates
    pub fn command_not_found_with_candidates(noun: impl Into<String>, candidates: &[&str]) -> Self {
        let noun_str = noun.into();
        let matches = find_best_matches(&noun_str, candidates);
        let suggestion = if matches.is_empty() {
            String::new()
        } else {
            let suggestions_str = matches
                .iter()
                .map(|s| format!("\x1b[1m\x1b[33m{}\x1b[0m", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(". Did you mean: {}?", suggestions_str)
        };
        Self::CommandNotFound { noun: noun_str, suggestion }
    }

    /// Create a verb not found error
    pub fn verb_not_found(noun: impl Into<String>, verb: impl Into<String>) -> Self {
        Self::VerbNotFound { noun: noun.into(), verb: verb.into(), suggestion: String::new() }
    }

    /// Create a verb not found error with suggestion candidates
    pub fn verb_not_found_with_candidates(
        noun: impl Into<String>,
        verb: impl Into<String>,
        candidates: &[&str],
    ) -> Self {
        let verb_str = verb.into();
        let matches = find_best_matches(&verb_str, candidates);
        let suggestion = if matches.is_empty() {
            String::new()
        } else {
            let suggestions_str = matches
                .iter()
                .map(|s| format!("\x1b[1m\x1b[33m{}\x1b[0m", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!(". Did you mean: {}?", suggestions_str)
        };
        Self::VerbNotFound { noun: noun.into(), verb: verb_str, suggestion }
    }

    /// Create an invalid structure error
    pub fn invalid_structure(message: impl Into<String>) -> Self {
        Self::InvalidStructure { message: message.into() }
    }

    /// Create an execution error
    pub fn execution_error(message: impl Into<String>) -> Self {
        Self::ExecutionError { message: message.into() }
    }

    /// Create an argument error
    pub fn argument_error(message: impl Into<String>) -> Self {
        Self::ArgumentError { message: message.into() }
    }

    /// Create a missing argument error (helper for common case)
    pub fn missing_argument(name: impl Into<String>) -> Self {
        Self::ArgumentError { message: format!("Required argument '{}' is missing", name.into()) }
    }

    /// Create a validation error with constraints
    pub fn validation_error(
        name: impl Into<String>,
        value: impl Into<String>,
        constraints: Option<&str>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        if let Some(constraints) = constraints {
            Self::ArgumentError {
                message: format!(
                    "Invalid value '{}' for argument '{}'. {}",
                    value, name, constraints
                ),
            }
        } else {
            Self::ArgumentError {
                message: format!("Invalid value '{}' for argument '{}'", value, name),
            }
        }
    }

    /// Create a validation error with range constraints
    pub fn validation_range_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<&str>,
        max: Option<&str>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        let constraint_msg = match (min, max) {
            (Some(min), Some(max)) => format!("Must be between {} and {}", min, max),
            (Some(min), None) => format!("Must be >= {}", min),
            (None, Some(max)) => format!("Must be <= {}", max),
            (None, None) => "Invalid value".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint_msg))
    }

    /// Create a validation error with length constraints
    pub fn validation_length_error(
        name: impl Into<String>,
        value: impl Into<String>,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Self {
        let name = name.into();
        let value = value.into();
        let constraint_msg = match (min, max) {
            (Some(min), Some(max)) => {
                format!("Length must be between {} and {} characters", min, max)
            }
            (Some(min), None) => format!("Length must be at least {} characters", min),
            (None, Some(max)) => format!("Length must be at most {} characters", max),
            (None, None) => "Invalid length".to_string(),
        };
        Self::validation_error(name, value, Some(&constraint_msg))
    }
}

impl From<std::io::Error> for NounVerbError {
    fn from(err: std::io::Error) -> Self {
        Self::ExecutionError { message: err.to_string() }
    }
}

/// Result type alias for noun-verb operations
pub type Result<T> = std::result::Result<T, NounVerbError>;

/// MAPE-K Error Kinds
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidInput,
    PermissionDenied,
    InvariantBreach,
    DeadlineExceeded,
    GuardExceeded,
    CommandNotFound,
    VerbNotFound,
    ExecutionError,
    InternalError,
}

/// Severity level of the error
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
    Critical,
}

/// Recovery Action templates proposed by the MAPE-K recovery layer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ActionTemplate {
    TimeoutAdjustment { suggested_timeout_ms: u64, reason: String },
    CommandFix { suggested_command: String, reason: String },
}

/// Machine-readable, uniform structured error format for autonomic MAPE-K loops
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StructuredError {
    pub kind: ErrorKind,
    pub severity: Severity,
    pub message: String,
    pub details: std::collections::HashMap<String, serde_json::Value>,
    pub action_templates: Vec<ActionTemplate>,
}

impl StructuredError {
    /// Create a deadline exceeded error format with actual and target latency details
    pub fn deadline_exceeded(deadline_ms: u64, actual_ms: u64) -> Self {
        let mut details = std::collections::HashMap::new();
        details.insert("deadline_ms".to_string(), serde_json::json!(deadline_ms));
        details.insert("actual_ms".to_string(), serde_json::json!(actual_ms));

        Self {
            kind: ErrorKind::DeadlineExceeded,
            severity: Severity::Critical,
            message: format!("Deadline {}ms exceeded, took {}ms", deadline_ms, actual_ms),
            details,
            action_templates: vec![ActionTemplate::TimeoutAdjustment {
                suggested_timeout_ms: actual_ms + 100,
                reason: "Increase deadline budget to match observed latency".to_string(),
            }],
        }
    }

    /// Map a standard NounVerbError to StructuredError format
    pub fn from_error(err: &NounVerbError) -> Self {
        let mut details = std::collections::HashMap::new();
        let mut action_templates = Vec::new();
        let mut severity = Severity::Error;

        let kind = match err {
            NounVerbError::CommandNotFound { noun, suggestion } => {
                details.insert("noun".to_string(), serde_json::Value::String(noun.clone()));
                if !suggestion.is_empty() {
                    details.insert(
                        "suggestion".to_string(),
                        serde_json::Value::String(suggestion.clone()),
                    );
                    let clean = suggestion
                        .replace("\x1b[1m\x1b[33m", "")
                        .replace("\x1b[0m", "")
                        .replace(". Did you mean: ", "")
                        .replace("?", "");
                    let candidates: Vec<&str> = clean.split(", ").collect();
                    if let Some(first) = candidates.first() {
                        if !first.is_empty() {
                            action_templates.push(ActionTemplate::CommandFix {
                                suggested_command: first.to_string(),
                                reason: format!(
                                    "Suggested correction for misspelled command '{}'",
                                    noun
                                ),
                            });
                        }
                    }
                }
                ErrorKind::CommandNotFound
            }
            NounVerbError::VerbNotFound { noun, verb, suggestion } => {
                details.insert("noun".to_string(), serde_json::Value::String(noun.clone()));
                details.insert("verb".to_string(), serde_json::Value::String(verb.clone()));
                if !suggestion.is_empty() {
                    details.insert(
                        "suggestion".to_string(),
                        serde_json::Value::String(suggestion.clone()),
                    );
                    let clean = suggestion
                        .replace("\x1b[1m\x1b[33m", "")
                        .replace("\x1b[0m", "")
                        .replace(". Did you mean: ", "")
                        .replace("?", "");
                    let candidates: Vec<&str> = clean.split(", ").collect();
                    if let Some(first) = candidates.first() {
                        if !first.is_empty() {
                            action_templates.push(ActionTemplate::CommandFix {
                                suggested_command: format!("{} {}", noun, first),
                                reason: format!(
                                    "Suggested correction for misspelled verb '{}'",
                                    verb
                                ),
                            });
                        }
                    }
                }
                ErrorKind::VerbNotFound
            }
            NounVerbError::InvalidStructure { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvalidInput
            }
            NounVerbError::ExecutionError { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                if message.to_lowercase().contains("deadline")
                    || message.to_lowercase().contains("timeout")
                    || message.to_lowercase().contains("budget exceeded")
                {
                    severity = Severity::Critical;
                    action_templates.push(ActionTemplate::TimeoutAdjustment {
                        suggested_timeout_ms: 1000,
                        reason: "Increase deadline budget due to execution timeout".to_string(),
                    });
                    ErrorKind::DeadlineExceeded
                } else {
                    ErrorKind::ExecutionError
                }
            }
            NounVerbError::ArgumentError { message } => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvalidInput
            }
            NounVerbError::PluginError(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
            NounVerbError::ValidationFailed(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InvariantBreach
            }
            NounVerbError::MiddlewareError(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
            NounVerbError::TelemetryError(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
            NounVerbError::Generic(message) => {
                details.insert("message".to_string(), serde_json::Value::String(message.clone()));
                ErrorKind::InternalError
            }
        };

        StructuredError { kind, severity, message: err.to_string(), details, action_templates }
    }
}
