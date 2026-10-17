// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Command handlers - bridge between CLI and business logic
//!
//! Command handlers accept validated arguments from the CLI layer and
//! delegate to core business logic functions.

use crate::error::Result;

/// Trait for command handlers that delegate to business logic
///
/// Handlers validate inputs and delegate to pure business logic functions.
/// This separates CLI concerns from business logic.
pub trait CommandHandler: Send + Sync {
    /// Execute the command with validated inputs
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    fn execute(&self, input: HandlerInput) -> Result<HandlerOutput>;
}

/// Input to a command handler (validated by CLI layer)
#[derive(Debug, Clone)]
pub struct HandlerInput {
    /// Validated arguments as key-value pairs
    pub args: std::collections::HashMap<String, String>,
    /// Validated options as key-value pairs
    pub opts: std::collections::HashMap<String, String>,
    /// Context information (noun, verb names, etc.)
    pub context: HandlerContext,
}

/// Output from a command handler
///
/// In v3, output is automatically serialized to JSON for agent/MCP consumption.
/// The `data` field contains any type that implements `Serialize`.
#[derive(Debug, Clone)]
pub struct HandlerOutput {
    /// Result data (auto-serialized to JSON)
    pub data: serde_json::Value,
    /// Success message (optional)
    pub message: Option<String>,
}

impl HandlerOutput {
    /// Create a new handler output from any serializable type
    pub fn from_data<T: serde::Serialize>(data: T) -> Result<Self> {
        Ok(Self {
            data: serde_json::to_value(data).map_err(|e| {
                crate::error::NounVerbError::execution_error(format!(
                    "Failed to serialize output: {}",
                    e
                ))
            })?,
            message: None,
        })
    }

    /// Create output with a message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Serialize output to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.data).map_err(|e| {
            crate::error::NounVerbError::execution_error(format!(
                "Failed to serialize to JSON: {}",
                e
            ))
        })
    }
}

/// Context information for handler execution
#[derive(Debug, Clone)]
pub struct HandlerContext {
    /// Noun name (if applicable)
    pub noun: Option<String>,
    /// Verb name
    pub verb: String,
    /// Additional context data
    pub data: std::collections::HashMap<String, String>,
}

impl HandlerContext {
    /// Create a new handler context
    pub fn new(verb: impl Into<String>) -> Self {
        Self { noun: None, verb: verb.into(), data: std::collections::HashMap::new() }
    }

    /// Set the noun name
    pub fn with_noun(mut self, noun: impl Into<String>) -> Self {
        self.noun = Some(noun.into());
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }
}
