// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Argument and option validator - validates CLI inputs only
//!
//! This module contains validation logic for CLI arguments and options.
//! It does NOT contain business logic - only validation.

use crate::error::{NounVerbError, Result};
use clap::ArgMatches;
use std::collections::HashMap;

/// Validator for CLI arguments and options
///
/// This validator ensures that arguments are valid before delegating
/// to business logic functions.
pub struct ArgValidator;

impl ArgValidator {
    /// Create a new argument validator
    pub fn new() -> Self {
        Self
    }

    /// Validate and extract a required string argument
    ///
    /// # Errors
    ///
    /// Returns an error if the argument is missing or invalid.
    pub fn validate_required_str(&self, matches: &ArgMatches, name: &str) -> Result<String> {
        matches.get_one::<String>(name).cloned().ok_or_else(|| {
            NounVerbError::argument_error(format!("Required argument '{}' is missing", name))
        })
    }

    /// Validate and extract an optional string argument
    pub fn validate_optional_str(&self, matches: &ArgMatches, name: &str) -> Option<String> {
        matches.get_one::<String>(name).cloned()
    }

    /// Validate and extract a required typed argument
    ///
    /// # Errors
    ///
    /// Returns an error if the argument is missing or invalid type.
    pub fn validate_required<T>(&self, matches: &ArgMatches, name: &str) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        matches.get_one::<T>(name).cloned().ok_or_else(|| {
            NounVerbError::argument_error(format!(
                "Required argument '{}' is missing or has invalid type",
                name
            ))
        })
    }

    /// Validate and extract an optional typed argument
    pub fn validate_optional<T>(&self, matches: &ArgMatches, name: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        matches.get_one::<T>(name).cloned()
    }

    /// Validate and extract multiple values of a typed argument
    ///
    /// # Errors
    ///
    /// Returns an error if the argument is missing or has no values.
    pub fn validate_many<T>(&self, matches: &ArgMatches, name: &str) -> Result<Vec<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let values: Vec<T> =
            matches.get_many::<T>(name).map(|iter| iter.cloned().collect()).unwrap_or_default();

        if values.is_empty() {
            Err(NounVerbError::argument_error(format!(
                "Required argument '{}' is missing or has no values",
                name
            )))
        } else {
            Ok(values)
        }
    }

    /// Validate and extract optional multiple values
    pub fn validate_many_opt<T>(&self, matches: &ArgMatches, name: &str) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        matches.get_many::<T>(name).map(|iter| iter.cloned().collect()).unwrap_or_default()
    }

    /// Check if a flag is set
    pub fn validate_flag(&self, matches: &ArgMatches, name: &str) -> bool {
        matches.get_flag(name)
    }

    /// Get the count of a flag (for -v, -vv, -vvv patterns)
    pub fn validate_flag_count(&self, matches: &ArgMatches, name: &str) -> u8 {
        matches.get_count(name)
    }

    /// Extract all validated arguments as a map
    ///
    /// This extracts all string arguments into a map for delegation
    /// to business logic functions.
    pub fn extract_args(&self, matches: &ArgMatches) -> HashMap<String, String> {
        let mut args = HashMap::new();

        // Extract all string arguments
        for id in matches.ids() {
            let name = id.as_str();
            if let Some(value) = self.validate_optional_str(matches, name) {
                args.insert(name.to_string(), value);
            }
        }

        args
    }

    /// Extract all validated options as a map
    ///
    /// This extracts only flags (boolean arguments) into a map.
    /// Does NOT extract count-based arguments (use validate_flag_count() directly for those).
    ///
    /// NOTE: This method only handles flags. For count arguments, call validate_flag_count()
    /// directly since get_count() may panic if called on non-count arguments.
    pub fn extract_opts(&self, matches: &ArgMatches) -> HashMap<String, String> {
        let mut opts = HashMap::new();

        // Extract flags only - get_flag is safe to call on all argument types
        // It returns false for non-flag arguments, so no panics are possible
        for id in matches.ids() {
            let name = id.as_str();
            if matches.get_flag(name) {
                opts.insert(name.to_string(), "true".to_string());
            }
        }

        opts
    }

    /// Safely check if a flag or count argument is present
    ///
    /// Returns true if the argument is a flag set to true or a count with value > 0.
    ///
    /// # Panics
    ///
    /// Panics if called with an argument that is neither a flag nor a count argument.
    /// Use this method when you know an argument is a flag or count type.
    pub fn is_present(&self, matches: &ArgMatches, name: &str) -> bool {
        matches.get_flag(name) || matches.get_count(name) > 0
    }
}

impl Default for ArgValidator {
    fn default() -> Self {
        Self::new()
    }
}
