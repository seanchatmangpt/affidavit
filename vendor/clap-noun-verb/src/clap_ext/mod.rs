// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced clap integration (Phase 7 - v4.3)
//!
//! This module provides cutting-edge integration with clap 4.5.51, featuring:
//!
//! - **Enum-based subcommand composition** - Zero-boilerplate command dispatch
//! - **Custom value parsers** - Type-safe parsing for URLs, ports, JSON, CSV, etc.
//! - **Advanced completions** - Multi-shell completion generation (bash, zsh, fish, PowerShell)
//!
//! # Features
//!
//! - Automatic subcommand mapping from Rust enums
//! - Composable value parser builders
//! - Environment-aware completion suggestions
//! - Dynamic completion support
//! - Flattened command composition
//!
//! # Example
//!
//! ```ignore
//! use clap::Parser;
//! use clap_noun_verb::clap::{EnumCommand, CompletionGenerator, Shell};
//!
//! #[derive(Parser)]
//! struct Cli {
//!     #[command(subcommand)]
//!     command: Commands,
//! }
//!
//! #[derive(clap::Subcommand)]
//! enum Commands {
//!     Start { port: u16 },
//!     Stop,
//! }
//!
//! impl EnumCommand for Commands {
//!     fn execute(&self) -> clap_noun_verb::Result<String> {
//!         match self {
//!             Commands::Start { port } => Ok(format!("Starting on port {}", port)),
//!             Commands::Stop => Ok("Stopping".to_string()),
//!         }
//!     }
//! }
//! ```

pub mod completions;
pub mod enum_dispatch;
pub mod value_parsers;

// Re-exports for convenience
pub use completions::{CompletionContext, CompletionGenerator, Shell};
pub use enum_dispatch::{CommandContext, EnumCommand, EnumDispatcher, FlattenConfig};
pub use value_parsers::{
    CsvList, ParserConfig, ValidatedJson, ValidatedPort, ValidatedUrl, ValueParserBuilder,
};
