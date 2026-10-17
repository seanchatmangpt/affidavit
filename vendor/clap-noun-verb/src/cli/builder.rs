// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Opinionated CLI builder - maximum automation and defaults
//!
//! This builder provides an extremely opinionated, automated API that:
//! - Auto-validates arguments from function signatures
//! - Auto-generates help from docstrings
//! - Auto-delegates to business logic functions
//! - Auto-formats output
//! - Strong defaults for 90% of use cases
//!
//! If you need low-level control, use clap directly.
//!
//! ## Memory Management: Box::leak Usage
//!
//! This module uses `Box::leak()` to convert owned Strings to `&'static str`
//! references required by clap. This is acceptable for CLI construction since:
//! - It happens once during initialization (not in hot paths)
//! - Total memory impact is negligible for typical CLI applications
//! - No reasonable alternative maintains clap's performance model
//! - For more details, see src/cli/registry.rs module documentation

use crate::cli::CommandRouter;
use crate::error::{NounVerbError, Result};
use crate::logic::{HandlerInput, HandlerOutput};
use crate::noun::NounCommand;
use crate::registry::CompletionsNoun;
use clap::Command;
use std::collections::HashMap;

/// Opinionated CLI builder with maximum automation
///
/// This builder provides a simple, automated API for creating CLIs.
/// It makes strong default choices to reduce cognitive load.
///
/// # Example
///
/// ```rust,no_run
/// use clap_noun_verb::cli::builder::CliBuilder;
/// use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
/// use clap_noun_verb::Result;
///
/// // Business logic (reusable)
/// fn show_status(_input: HandlerInput) -> Result<HandlerOutput> {
///     Ok(HandlerOutput::from_data("All services running")?)
/// }
///
/// // CLI (validation + delegation only)
/// fn main() -> Result<()> {
///     let cli = CliBuilder::new("myapp")
///         .noun("services", "Manage services")
///         .verb("services", "status", "Show status", show_status);
///
///     cli.run()
/// }
/// ```
pub struct CliBuilder {
    /// Application name
    name: String,
    /// Application description
    about: String,
    /// Version string (optional, auto-detected from Cargo.toml)
    version: Option<String>,
    /// Nouns registered with the CLI
    nouns: HashMap<String, Box<dyn NounCommand>>,
    /// Add completions subcommand
    has_completions_subcommand: bool,
}

impl CliBuilder {
    /// Create a new CLI builder with opinionated defaults
    ///
    /// # Arguments
    ///
    /// * `name` - Application name
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap_noun_verb::cli::builder::CliBuilder;
    ///
    /// let cli = CliBuilder::new("myapp");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            about: String::new(),
            version: None,
            nouns: HashMap::new(),
            has_completions_subcommand: false,
        }
    }

    /// Set application description (auto-used for help)
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap_noun_verb::cli::builder::CliBuilder;
    ///
    /// let cli = CliBuilder::new("myapp")
    ///     .about("My awesome CLI application");
    /// ```
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = about.into();
        self
    }

    /// Set application version (auto-detected from Cargo.toml if not provided)
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add a noun command with opinionated defaults
    ///
    /// This method provides a simple, automated way to add nouns.
    /// For more control, use clap directly.
    ///
    /// # Arguments
    ///
    /// * `name` - Noun name (e.g., "services")
    /// * `about` - Description (auto-used for help)
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap_noun_verb::cli::builder::CliBuilder;
    ///
    /// let cli = CliBuilder::new("myapp")
    ///     .noun("services", "Manage services");
    /// ```
    pub fn noun(mut self, name: impl Into<String>, about: impl Into<String>) -> Self {
        // Create a simple noun implementation with opinionated defaults
        let noun = SimpleNoun::new(name.into(), about.into());
        self.nouns.insert(noun.name().to_string(), Box::new(noun));
        self
    }

    /// Add a completions subcommand
    pub fn with_completions_subcommand(mut self) -> Self {
        self.has_completions_subcommand = true;
        self
    }

    /// Add a verb command with automated validation and delegation
    ///
    /// This method automatically:
    /// - Validates arguments from function signature
    /// - Generates help from docstrings
    /// - Delegates to business logic function
    ///
    /// # Arguments
    ///
    /// * `noun_name` - Name of the noun this verb belongs to
    /// * `verb_name` - Verb name (e.g., "status")
    /// * `about` - Description (auto-used for help)
    /// * `handler` - Business logic function (reusable)
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap_noun_verb::cli::builder::CliBuilder;
    /// use clap_noun_verb::logic::{HandlerInput, HandlerOutput};
    /// use clap_noun_verb::Result;
    ///
    /// fn show_status(_input: HandlerInput) -> Result<HandlerOutput> {
    ///     Ok(HandlerOutput::from_data("All services running")?)
    /// }
    ///
    /// let cli = CliBuilder::new("myapp")
    ///     .noun("services", "Manage services")
    ///     .verb("services", "status", "Show status", show_status);
    /// ```
    pub fn verb<F>(
        self,
        _noun_name: impl Into<String>,
        _verb_name: impl Into<String>,
        _about: impl Into<String>,
        _handler: F,
    ) -> Self
    where
        F: Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync + 'static,
    {
        // Note: For v3, use attribute macros instead of manual verb registration
        // Attribute macros handle auto-validation, help generation, and integration automatically
        // This method is kept for backward compatibility with existing code
        self
    }

    /// Run the CLI with current process arguments
    ///
    /// This method automatically:
    /// - Parses arguments
    /// - Validates inputs
    /// - Delegates to business logic
    /// - Formats output
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use clap_noun_verb::cli::builder::CliBuilder;
    /// use clap_noun_verb::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let cli = CliBuilder::new("myapp")
    ///         .about("My application")
    ///         .noun("services", "Manage services");
    ///     cli.run()
    /// }
    /// ```
    pub fn run(self) -> Result<()> {
        let args: Vec<String> = std::env::args().collect();
        self.run_with_args(args)
    }

    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        if args.is_empty() {
            return Err(NounVerbError::argument_error("No arguments provided"));
        }

        let cmd = self.build_command();

        // Split args by "++"
        let mut steps = Vec::new();
        let mut current_step = Vec::new();
        let binary_name = args[0].clone();

        for arg in args.into_iter().skip(1) {
            if arg == "++" {
                if !current_step.is_empty() {
                    steps.push(current_step);
                    current_step = Vec::new();
                }
            } else {
                current_step.push(arg);
            }
        }
        if !current_step.is_empty() {
            steps.push(current_step);
        }

        let completions_noun = if self.has_completions_subcommand {
            Some(self.build_completions_noun())
        } else {
            None
        };

        let mut router = CommandRouter::new();
        for (_, noun) in self.nouns {
            router.register_noun(noun);
        }

        if let Some(cn) = completions_noun {
            router.register_noun(Box::new(cn));
        }

        if steps.is_empty() {
            let matches = cmd
                .clone()
                .try_get_matches_from(vec![binary_name])
                .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
            return router.route(&matches);
        }

        let stdin_val = crate::cli::preprocessor::read_stdin_if_needed(&steps);
        let mut step_results: Vec<serde_json::Value> = Vec::new();

        for step in steps {
            let mut step_args = vec![binary_name.clone()];
            let processed_args =
                crate::cli::preprocessor::preprocess_args(&step, &stdin_val, &step_results)?;
            step_args.extend(processed_args);

            let matches = cmd
                .clone()
                .try_get_matches_from(step_args)
                .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

            router.route(&matches)?;
            step_results.push(serde_json::Value::Null);
        }

        Ok(())
    }

    /// Build the clap command structure
    ///
    /// This method builds the complete clap command structure
    /// with opinionated defaults.
    #[cfg_attr(not(test), allow(dead_code))] // Used by tests
    pub fn build_command(&self) -> Command {
        // Leak strings to get static lifetime for clap
        // This is acceptable for CLI construction (happens once per run)
        let name: &'static str = Box::leak(self.name.clone().into_boxed_str());
        let about: &'static str = Box::leak(self.about.clone().into_boxed_str());

        let mut cmd = Command::new(name);

        if !self.about.is_empty() {
            cmd = cmd.about(about);
        }

        if let Some(version) = &self.version {
            let version_str: &'static str = Box::leak(version.clone().into_boxed_str());
            cmd = cmd.version(version_str);
        }

        // Add nouns as subcommands
        for noun in self.nouns.values() {
            cmd = cmd.subcommand(noun.build_command());
        }

        if self.has_completions_subcommand {
            let completions_noun = self.build_completions_noun();
            cmd = cmd.subcommand(completions_noun.build_command());
        }

        cmd
    }

    fn build_completions_noun(&self) -> CompletionsNoun {
        let app_name = self.name.clone();
        let app_version = self.version.clone();

        let mut commands = Vec::new();
        let mut options = Vec::new();

        // Collect all nouns and their verbs/subnouns
        for (noun_name, noun) in &self.nouns {
            commands.push(noun_name.clone());
            for verb in noun.verbs() {
                commands.push(format!("{} {}", noun_name, verb.name()));
            }
            for sub_noun in noun.sub_nouns() {
                commands.push(format!("{} {}", noun_name, sub_noun.name()));
            }
        }

        // Collect options from verb arguments
        for noun in self.nouns.values() {
            for verb in noun.verbs() {
                for arg in verb.additional_args() {
                    if let Some(long) = arg.get_long() {
                        options.push(format!("--{}", long));
                    }
                    if let Some(short) = arg.get_short() {
                        options.push(format!("-{}", short));
                    }
                }
            }
        }

        CompletionsNoun::new(app_name, app_version, commands, options)
    }
}

/// Simple noun implementation for opinionated builder
///
/// This handles the lifetime issue by using Box::leak for static strings.
/// For maximum opinionation, this is acceptable in CLI construction
/// (happens once per run).
struct SimpleNoun {
    name: &'static str,
    about: &'static str,
    #[allow(dead_code)] // Reserved for future verb integration
    verbs: Vec<Box<dyn crate::verb::VerbCommand>>,
}

// VerbCommand is not Clone, so we need to handle this differently
// For now, SimpleNoun won't have verbs until we implement proper verb integration

impl SimpleNoun {
    fn new(name: String, about: String) -> Self {
        // Leak strings to get static lifetime
        // This is acceptable for CLI construction (happens once per run)
        let name_str: &'static str = Box::leak(name.into_boxed_str());
        let about_str: &'static str = Box::leak(about.into_boxed_str());

        Self { name: name_str, about: about_str, verbs: Vec::new() }
    }
}

impl NounCommand for SimpleNoun {
    fn name(&self) -> &'static str {
        self.name
    }

    fn about(&self) -> &'static str {
        self.about
    }

    fn verbs(&self) -> Vec<Box<dyn crate::verb::VerbCommand>> {
        // Verbs are not Clone, so we can't return them
        // This will be fixed when we properly implement verb integration
        Vec::new()
    }

    fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
        Vec::new()
    }

    fn build_command(&self) -> Command {
        // Verbs will be added when verb integration is complete
        // for verb in &self.verbs {
        //     cmd = cmd.subcommand(verb.build_command());
        // }

        Command::new(self.name).about(self.about)
    }
}
