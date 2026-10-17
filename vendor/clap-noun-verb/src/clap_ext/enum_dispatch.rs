// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Enum-based subcommand composition for advanced clap integration.
//!
//! This module provides zero-boilerplate command composition using Rust enums,
//! leveraging clap 4.5.51's latest features for seamless subcommand mapping.
//!
//! # Examples
//!
//! ```ignore
//! use clap::{Parser, Subcommand};
//! use clap_noun_verb::clap::EnumCommand;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(subcommand)]
//!     command: Commands,
//! }
//!
//! #[derive(Subcommand)]
//! enum Commands {
//!     /// Start the server
//!     Start { port: u16 },
//!     /// Stop the server
//!     Stop { signal: String },
//! }
//!
//! impl EnumCommand for Commands {
//!     fn execute(&self) -> Result<String> {
//!         match self {
//!             Commands::Start { port } => Ok(format!("Starting server on port {}", port)),
//!             Commands::Stop { signal } => Ok(format!("Stopping with signal {}", signal)),
//!         }
//!     }
//! }
//! ```

use std::fmt;

/// Trait for automatic enum variant to subcommand mapping.
///
/// Enables zero-boilerplate subcommand execution using Rust enums.
/// Each enum variant maps to a subcommand with automatic dispatch.
pub trait EnumCommand: Sized {
    /// Execute the command variant and return a result.
    ///
    /// # Errors
    ///
    /// Returns an error if command execution fails.
    fn execute(&self) -> crate::Result<String>;

    /// Get a description of this command variant.
    ///
    /// Default implementation returns the type name.
    fn description(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }

    /// Get the variant name as a string.
    ///
    /// Useful for logging and debugging.
    fn variant_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Builder for composing multiple enum-based subcommands.
#[derive(Debug, Clone)]
pub struct EnumDispatcher {
    /// Name of the dispatcher
    name: String,
    /// Description of available commands
    description: String,
}

impl EnumDispatcher {
    /// Create a new enum dispatcher with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), description: String::new() }
    }

    /// Set the description for this dispatcher.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Get the name of this dispatcher.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description of this dispatcher.
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Default for EnumDispatcher {
    fn default() -> Self {
        Self::new("dispatcher")
    }
}

impl fmt::Display for EnumDispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

/// Configuration for enum-based command flattening.
///
/// Allows nested enum groups to be flattened into the parent command structure.
#[derive(Debug, Clone)]
pub struct FlattenConfig {
    /// Whether to flatten nested commands
    flatten: bool,
    /// Prefix for flattened commands
    prefix: Option<String>,
    /// Maximum nesting depth
    max_depth: usize,
}

impl FlattenConfig {
    /// Create a new flatten configuration.
    pub fn new() -> Self {
        Self { flatten: false, prefix: None, max_depth: 3 }
    }

    /// Enable flattening.
    pub fn enable_flatten(mut self) -> Self {
        self.flatten = true;
        self
    }

    /// Set a prefix for flattened commands.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the maximum nesting depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Check if flattening is enabled.
    pub fn is_flatten_enabled(&self) -> bool {
        self.flatten
    }

    /// Get the prefix.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Get the maximum nesting depth.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Default for FlattenConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for command execution with metadata.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Name of the executed command
    command_name: String,
    /// Full command path (for nested commands)
    command_path: Vec<String>,
    /// Arguments passed to the command
    args: Vec<String>,
    /// Whether this is a nested command
    is_nested: bool,
}

impl CommandContext {
    /// Create a new command context.
    pub fn new(command_name: impl Into<String>) -> Self {
        let name = command_name.into();
        Self {
            command_path: vec![name.clone()],
            command_name: name,
            args: Vec::new(),
            is_nested: false,
        }
    }

    /// Add an argument to the context.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set the command path (for nested commands).
    pub fn with_path(mut self, path: Vec<String>) -> Self {
        self.command_path = path;
        self
    }

    /// Mark this as a nested command.
    pub fn mark_nested(mut self) -> Self {
        self.is_nested = true;
        self
    }

    /// Get the command name.
    pub fn command_name(&self) -> &str {
        &self.command_name
    }

    /// Get the full command path.
    pub fn command_path(&self) -> &[String] {
        &self.command_path
    }

    /// Get the arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Check if this is a nested command.
    pub fn is_nested(&self) -> bool {
        self.is_nested
    }

    /// Get the full command path as a string.
    pub fn full_path(&self) -> String {
        self.command_path.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCommand;

    impl EnumCommand for MockCommand {
        fn execute(&self) -> crate::Result<String> {
            Ok("executed".to_string())
        }
    }

    #[test]
    fn test_enum_dispatcher_creation() {
        let dispatcher = EnumDispatcher::new("test");
        assert_eq!(dispatcher.name(), "test");
        assert_eq!(dispatcher.description(), "");
    }

    #[test]
    fn test_enum_dispatcher_with_description() {
        let dispatcher = EnumDispatcher::new("test").with_description("Test dispatcher");
        assert_eq!(dispatcher.name(), "test");
        assert_eq!(dispatcher.description(), "Test dispatcher");
    }

    #[test]
    fn test_enum_dispatcher_display() {
        let dispatcher = EnumDispatcher::new("test").with_description("Test");
        assert_eq!(dispatcher.to_string(), "test: Test");
    }

    #[test]
    fn test_flatten_config_default() {
        let config = FlattenConfig::default();
        assert!(!config.is_flatten_enabled());
        assert_eq!(config.max_depth(), 3);
    }

    #[test]
    fn test_flatten_config_with_prefix() {
        let config = FlattenConfig::new().enable_flatten().with_prefix("cmd_");
        assert!(config.is_flatten_enabled());
        assert_eq!(config.prefix(), Some("cmd_"));
    }

    #[test]
    fn test_command_context_creation() {
        let ctx = CommandContext::new("test");
        assert_eq!(ctx.command_name(), "test");
        assert_eq!(ctx.full_path(), "test");
        assert!(!ctx.is_nested());
    }

    #[test]
    fn test_command_context_with_args() {
        let ctx = CommandContext::new("test").with_arg("arg1").with_arg("arg2");
        assert_eq!(ctx.args().len(), 2);
    }

    #[test]
    fn test_command_context_nested() {
        let ctx = CommandContext::new("test")
            .mark_nested()
            .with_path(vec!["root".to_string(), "sub".to_string()]);
        assert!(ctx.is_nested());
        assert_eq!(ctx.full_path(), "root sub");
    }

    #[test]
    fn test_mock_command_execute() {
        let cmd = MockCommand;
        let result = cmd.execute();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "executed");
    }
}
