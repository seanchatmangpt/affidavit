// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Noun command trait and types for composable CLI patterns

use crate::error::Result;
use crate::verb::{VerbArgs, VerbCommand};
use clap::Command;
use std::collections::HashMap;

/// Context information passed to noun commands
#[derive(Debug, Clone)]
pub struct NounContext {
    /// The noun name being executed
    pub noun: String,
    /// Additional context data
    pub data: HashMap<String, String>,
}

impl NounContext {
    /// Create a new noun context
    pub fn new(noun: impl Into<String>) -> Self {
        Self { noun: noun.into(), data: HashMap::new() }
    }

    /// Add context data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Get context data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

/// Trait for defining noun commands (e.g., "services", "collector")
///
/// # Examples
///
/// Implementing `NounCommand` directly:
///
/// ```rust
/// use clap_noun_verb::{NounCommand, VerbCommand, VerbArgs, Result};
///
/// struct ServicesCommand;
///
/// impl NounCommand for ServicesCommand {
///     fn name(&self) -> &'static str { "services" }
///     fn about(&self) -> &'static str { "Manage services" }
///     fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
///         vec![Box::new(StatusCommand)]
///     }
/// }
///
/// struct StatusCommand;
///
/// impl VerbCommand for StatusCommand {
///     fn name(&self) -> &'static str { "status" }
///     fn about(&self) -> &'static str { "Show status" }
///     fn run(&self, _args: &VerbArgs) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
///
/// Using the `noun!` macro (recommended):
///
/// ```rust
/// use clap_noun_verb::{noun, verb, VerbArgs};
///
/// let services = noun!("services", "Manage services", [
///     verb!("status", "Show status", |_args: &VerbArgs| { Ok(()) }),
/// ]);
/// ```
pub trait NounCommand: Send + Sync {
    /// The name of the noun command
    fn name(&self) -> &'static str;

    /// Description of what this noun command does
    fn about(&self) -> &'static str;

    /// Get all verb commands associated with this noun
    fn verbs(&self) -> Vec<Box<dyn VerbCommand>>;

    /// Get all sub-noun commands (for nested command groups)
    fn sub_nouns(&self) -> Vec<Box<dyn NounCommand>> {
        Vec::new()
    }

    /// Build the clap command for this noun
    fn build_command(&self) -> Command {
        let mut cmd = Command::new(self.name()).about(self.about());

        // Add verb subcommands (build_command already includes additional_args)
        for verb in self.verbs() {
            cmd = cmd.subcommand(verb.build_command());
        }

        // Add sub-noun commands (for nested command groups)
        for sub_noun in self.sub_nouns() {
            cmd = cmd.subcommand(sub_noun.build_command());
        }

        cmd
    }

    /// Handle the noun command if it has no verbs or sub-nouns (direct execution)
    fn handle_direct(&self, _args: &VerbArgs) -> Result<()> {
        Err(crate::error::NounVerbError::invalid_structure(format!(
            "Noun '{}' has no verbs or sub-nouns and cannot be executed directly",
            self.name()
        )))
    }

    /// Handle a verb command for this noun
    fn handle_verb(&self, verb_name: &str, args: &VerbArgs) -> Result<()> {
        let verb = self.verbs().into_iter().find(|v| v.name() == verb_name).ok_or_else(|| {
            let mut candidates: Vec<&str> = self.verbs().iter().map(|v| v.name()).collect();
            candidates.extend(self.sub_nouns().iter().map(|n| n.name()));
            crate::error::NounVerbError::verb_not_found_with_candidates(
                self.name(),
                verb_name,
                &candidates,
            )
        })?;

        verb.run(args)
    }

    /// Handle a sub-noun command for this noun
    fn handle_sub_noun(&self, sub_noun_name: &str, args: &VerbArgs) -> Result<()> {
        let sub_noun =
            self.sub_nouns().into_iter().find(|n| n.name() == sub_noun_name).ok_or_else(|| {
                let mut candidates: Vec<&str> = self.sub_nouns().iter().map(|n| n.name()).collect();
                candidates.extend(self.verbs().iter().map(|v| v.name()));
                crate::error::NounVerbError::command_not_found_with_candidates(
                    sub_noun_name,
                    &candidates,
                )
            })?;

        sub_noun.handle_direct(args)
    }
}

/// Helper trait for creating compound commands (nouns that contain other nouns)
///
/// # Examples
///
/// ```rust
/// use clap_noun_verb::{NounCommand, CompoundNounCommand, VerbCommand, VerbArgs, Result};
///
/// struct MyCompoundCommand;
///
/// impl NounCommand for MyCompoundCommand {
///     fn name(&self) -> &'static str { "system" }
///     fn about(&self) -> &'static str { "System commands" }
///     fn verbs(&self) -> Vec<Box<dyn VerbCommand>> {
///         vec![]
///     }
/// }
///
/// impl CompoundNounCommand for MyCompoundCommand {}
///
/// let cmd = MyCompoundCommand;
/// assert_eq!(cmd.all_nouns(), vec!["system".to_string()]);
/// ```
pub trait CompoundNounCommand: NounCommand {
    /// Get all nested nouns recursively
    fn all_nouns(&self) -> Vec<String> {
        let mut nouns = vec![self.name().to_string()];
        for sub_noun in self.sub_nouns() {
            nouns.push(sub_noun.name().to_string());
            // For compound sub-nouns, we can't easily recurse without dynamic dispatch
            // This is a limitation of the current trait design
        }
        nouns
    }

    /// Get all verbs recursively
    fn all_verbs(&self) -> HashMap<String, Vec<String>> {
        let mut verbs = HashMap::new();
        verbs.insert(
            self.name().to_string(),
            self.verbs().iter().map(|v| v.name().to_string()).collect(),
        );

        for sub_noun in self.sub_nouns() {
            verbs.insert(
                sub_noun.name().to_string(),
                sub_noun.verbs().iter().map(|v| v.name().to_string()).collect(),
            );
        }

        verbs
    }
}
