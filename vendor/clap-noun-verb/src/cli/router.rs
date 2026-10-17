// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CLI router - validation and delegation only
//!
//! This router enforces the pattern: validate arguments, then
//! delegate to business logic. No business logic is executed here.

use crate::error::{NounVerbError, Result};
use crate::noun::NounCommand;
use crate::verb::VerbCommand as LegacyVerbCommand;
use clap::ArgMatches;
use std::collections::HashMap;

/// Router - validates and delegates only
///
/// This router enforces that CLI code can ONLY validate arguments
/// and delegate to business logic. It does NOT execute business logic.
pub struct CommandRouter {
    /// Nouns registered with the router
    nouns: HashMap<String, Box<dyn NounCommand>>,
}

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        Self { nouns: HashMap::new() }
    }

    /// Register a noun command
    pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) {
        self.nouns.insert(noun.name().to_string(), noun);
    }

    /// Route a command based on clap matches
    ///
    /// This method validates arguments and delegates to business logic.
    /// It does NOT execute business logic directly.
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        // Get the top-level subcommand (noun)
        let (noun_name, noun_matches) = matches
            .subcommand()
            .ok_or_else(|| NounVerbError::invalid_structure("No subcommand found"))?;

        // Find the noun command
        let noun = self.nouns.get(noun_name).ok_or_else(|| {
            let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
            NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
        })?;

        // Route the command recursively with root matches for global args
        self.route_recursive(noun.as_ref(), noun_name, noun_matches, matches)
    }

    /// Recursively route commands through nested noun-verb structure
    #[allow(clippy::only_used_in_recursion)]
    fn route_recursive(
        &self,
        noun: &dyn NounCommand,
        noun_name: &str,
        matches: &ArgMatches,
        root_matches: &ArgMatches,
    ) -> Result<()> {
        // Check if there's a subcommand (either verb or sub-noun)
        if let Some((sub_name, sub_matches)) = matches.subcommand() {
            // First check if it's a verb
            if let Some(verb) = noun.verbs().iter().find(|v| v.name() == sub_name) {
                // Validate args, then delegate to business logic
                // Note: This requires VerbCommand trait with validate() and delegate()
                self.route_legacy_verb(
                    verb.as_ref(),
                    sub_name,
                    sub_matches,
                    root_matches,
                    noun_name,
                )
            } else if let Some(sub_noun) = noun.sub_nouns().iter().find(|n| n.name() == sub_name) {
                // Recursively route to sub-noun, passing root matches for global args
                self.route_recursive(sub_noun.as_ref(), sub_name, sub_matches, root_matches)
            } else {
                // Neither verb nor sub-noun found
                let mut candidates: Vec<&str> = noun.verbs().iter().map(|v| v.name()).collect();
                candidates.extend(noun.sub_nouns().iter().map(|n| n.name()));
                Err(NounVerbError::verb_not_found_with_candidates(noun_name, sub_name, &candidates))
            }
        } else {
            // No subcommand, try direct noun execution
            // Nouns validate and delegate through their handle_direct method
            self.handle_direct_noun(noun, matches, noun_name)
        }
    }

    /// Route a verb (for compatibility with existing verbs)
    ///
    /// This method handles verbs that still use the `run()` method.
    /// Future implementation will use validate + delegate pattern.
    fn route_legacy_verb(
        &self,
        verb: &dyn LegacyVerbCommand,
        verb_name: &str,
        sub_matches: &ArgMatches,
        root_matches: &ArgMatches,
        noun_name: &str,
    ) -> Result<()> {
        // Create VerbArgs for compatibility with legacy verb commands
        use crate::verb::{VerbArgs, VerbContext};
        let context = VerbContext::new(verb_name).with_noun(noun_name);
        let args = VerbArgs::new(sub_matches.clone())
            .with_parent(root_matches.clone())
            .with_context(context);

        // Execute verb (legacy pattern for backward compatibility)
        verb.run(&args)
    }

    /// Handle direct noun execution
    ///
    /// Nouns validate and delegate to business logic through their handle_direct method.
    fn handle_direct_noun(
        &self,
        noun: &dyn NounCommand,
        matches: &ArgMatches,
        noun_name: &str,
    ) -> Result<()> {
        // Nouns handle validation and delegation through handle_direct
        use crate::verb::{VerbArgs, VerbContext};
        let context = VerbContext::new("").with_noun(noun_name);
        let args = VerbArgs::new(matches.clone()).with_context(context);

        noun.handle_direct(&args)
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}
