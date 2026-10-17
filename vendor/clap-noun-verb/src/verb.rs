// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Verb command trait and types for composable CLI patterns

use crate::error::Result;
use clap::{ArgMatches, Command};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Type-safe state map for dependency injection
#[derive(Default, Clone)]
pub struct TypeMap {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl TypeMap {
    /// Create a new empty type map
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a value into the map
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.map.insert(TypeId::of::<T>(), Arc::new(val));
    }

    /// Get a reference to a value
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map.get(&TypeId::of::<T>()).and_then(|arc| arc.downcast_ref::<T>())
    }
}

impl std::fmt::Debug for TypeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeMap").field("keys", &self.map.keys().len()).finish()
    }
}

/// Context information passed to verb commands
#[derive(Debug, Clone)]
pub struct VerbContext {
    /// The verb name being executed
    pub verb: String,
    /// The noun this verb belongs to
    pub noun: Option<String>,
    /// Additional context data
    pub data: HashMap<String, String>,
    /// Typed context extensions
    pub extensions: TypeMap,
}

impl VerbContext {
    /// Create a new verb context
    pub fn new(verb: impl Into<String>) -> Self {
        Self { verb: verb.into(), noun: None, data: HashMap::new(), extensions: TypeMap::new() }
    }

    /// Set the noun this verb belongs to
    pub fn with_noun(mut self, noun: impl Into<String>) -> Self {
        self.noun = Some(noun.into());
        self
    }

    /// Add context data
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add typed extension data
    pub fn with_extension<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.extensions.insert(val);
        self
    }

    /// Get context data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Get typed extension data
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }
}

/// Arguments passed to a verb command
#[derive(Debug)]
pub struct VerbArgs {
    /// The raw clap matches for this verb
    pub matches: ArgMatches,
    /// Parent matches for accessing global arguments (e.g., --verbose, --config)
    pub parent_matches: Option<ArgMatches>,
    /// Context information
    pub context: VerbContext,
}

impl VerbArgs {
    /// Create new verb arguments
    pub fn new(matches: ArgMatches) -> Self {
        Self { matches, parent_matches: None, context: VerbContext::new("") }
    }

    /// Create verb arguments with parent matches for global args access
    pub fn with_parent(mut self, parent: ArgMatches) -> Self {
        self.parent_matches = Some(parent);
        self
    }

    /// Create verb arguments with context
    pub fn with_context(mut self, context: VerbContext) -> Self {
        self.context = context;
        self
    }

    /// Add context data to existing arguments
    pub fn add_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context = self.context.with_data(key, value);
        self
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get_data(key)
    }

    /// Get the verb name
    pub fn verb(&self) -> &str {
        &self.context.verb
    }

    /// Get the noun name (if available)
    pub fn noun(&self) -> Option<&str> {
        self.context.noun.as_deref()
    }

    /// Get a required string argument
    ///
    /// Uses `get_raw()` internally to avoid type mismatch panics when
    /// `value_parser` stores values as numeric types (e.g., u16, i64).
    pub fn get_one_str(&self, name: &str) -> Result<String> {
        self.get_one_str_opt(name).ok_or_else(|| {
            crate::error::NounVerbError::argument_error(format!(
                "Required argument '{}' is missing",
                name
            ))
        })
    }

    /// Get an optional string argument
    ///
    /// Uses `get_raw()` internally to get the original CLI string value,
    /// avoiding type mismatch panics when `value_parser` stores values
    /// as numeric types (e.g., u16, i64, f64).
    pub fn get_one_str_opt(&self, name: &str) -> Option<String> {
        // Use get_raw() to get the original string value from CLI
        // This works regardless of what value_parser was used
        if let Some(raw_values) = self.matches.get_raw(name) {
            // get_raw returns an iterator of OsStrings, take the first one
            if let Some(os_str) = raw_values.into_iter().next() {
                return os_str.to_str().map(|s| s.to_string());
            }
        }
        None
    }

    /// Get a required typed argument (e.g., usize, PathBuf)
    pub fn get_one<T>(&self, name: &str) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches.get_one::<T>(name).cloned().ok_or_else(|| {
            crate::error::NounVerbError::argument_error(format!(
                "Required argument '{}' is missing or has invalid type",
                name
            ))
        })
    }

    /// Get an optional typed argument
    pub fn get_one_opt<T>(&self, name: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches.get_one::<T>(name).cloned()
    }

    /// Get multiple values of a typed argument
    pub fn get_many<T>(&self, name: &str) -> Result<Vec<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let values: Vec<T> = self
            .matches
            .get_many::<T>(name)
            .map(|iter| iter.cloned().collect())
            .unwrap_or_default();

        if values.is_empty() {
            Err(crate::error::NounVerbError::argument_error(format!(
                "Required argument '{}' is missing or has no values",
                name
            )))
        } else {
            Ok(values)
        }
    }

    /// Get multiple values of a typed argument (optional, returns empty vec if missing)
    pub fn get_many_opt<T>(&self, name: &str) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.matches.get_many::<T>(name).map(|iter| iter.cloned().collect()).unwrap_or_default()
    }

    /// Check if a flag is set
    pub fn is_flag_set(&self, name: &str) -> bool {
        self.matches.get_flag(name)
    }

    /// Get the count of a flag (for -v, -vv, -vvv patterns)
    pub fn get_flag_count(&self, name: &str) -> u8 {
        self.matches.get_count(name)
    }

    /// Get all argument names as references (zero-copy)
    ///
    /// Returns borrowed references to argument names without allocating Strings.
    /// This is the preferred method when you don't need owned Strings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap::{Command, Arg};
    /// use clap_noun_verb::VerbArgs;
    ///
    /// let cmd = Command::new("test")
    ///     .arg(Arg::new("arg1").default_value("val1"))
    ///     .arg(Arg::new("arg2").default_value("val2"));
    /// let matches = cmd.get_matches_from(vec!["test"]);
    /// let verb_args = VerbArgs::new(matches);
    ///
    /// let args = verb_args.arg_names_refs();
    /// assert!(args.contains(&"arg1"));
    /// assert!(args.contains(&"arg2"));
    /// ```
    pub fn arg_names_refs(&self) -> Vec<&str> {
        self.matches.ids().map(|id| id.as_str()).collect()
    }

    /// Get all argument names as owned Strings
    ///
    /// Returns owned String copies of all argument names.
    /// Prefer [`Self::arg_names_refs`] when you don't need owned Strings,
    /// as it avoids unnecessary allocations.
    ///
    /// # Deprecation Note
    ///
    /// This method allocates a String for each argument name. In most cases,
    /// you should use [`Self::arg_names_refs`] instead for better performance.
    #[deprecated(
        since = "3.6.0",
        note = "use `arg_names_refs()` instead to avoid unnecessary allocations"
    )]
    pub fn arg_names(&self) -> Vec<String> {
        self.matches.ids().map(|id| id.as_str().to_string()).collect()
    }

    /// Get a required PathBuf argument
    pub fn get_path(&self, name: &str) -> Result<std::path::PathBuf> {
        self.get_one::<std::path::PathBuf>(name)
    }

    /// Get an optional PathBuf argument
    pub fn get_path_opt(&self, name: &str) -> Option<std::path::PathBuf> {
        self.get_one_opt::<std::path::PathBuf>(name)
    }

    /// Get a global argument from parent matches (e.g., --verbose, --config)
    pub fn get_global<T>(&self, name: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.parent_matches.as_ref().and_then(|parent| parent.get_one::<T>(name).cloned())
    }

    /// Get a global string argument from parent matches
    ///
    /// Uses `get_raw()` internally to get the original CLI string value,
    /// avoiding type mismatch panics when `value_parser` stores values
    /// as numeric types (e.g., u16, i64, f64).
    pub fn get_global_str(&self, name: &str) -> Option<String> {
        // Use get_raw() to get the original string value from CLI
        // This works regardless of what value_parser was used
        self.parent_matches.as_ref().and_then(|parent| {
            if let Some(raw_values) = parent.get_raw(name) {
                if let Some(os_str) = raw_values.into_iter().next() {
                    return os_str.to_str().map(|s| s.to_string());
                }
            }
            None
        })
    }

    /// Get multiple typed values from global arguments (required)
    ///
    /// Retrieves multiple values from parent matches (global arguments).
    /// Returns an error if the argument is not found or has no values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap::{Command, Arg, ArgAction};
    /// use clap_noun_verb::{VerbArgs, Result};
    ///
    /// # fn main() -> Result<()> {
    /// let parent_cmd = Command::new("parent")
    ///     .arg(Arg::new("hosts").long("host").action(ArgAction::Append));
    /// let parent_matches = parent_cmd.get_matches_from(vec!["parent", "--host", "localhost", "--host", "127.0.0.1"]);
    ///
    /// let cmd = Command::new("child");
    /// let matches = cmd.get_matches_from(vec!["child"]);
    ///
    /// let verb_args = VerbArgs::new(matches).with_parent(parent_matches);
    /// let hosts: Vec<String> = verb_args.get_global_many("hosts")?;
    ///
    /// assert_eq!(hosts, vec!["localhost".to_string(), "127.0.0.1".to_string()]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_global_many<T>(&self, name: &str) -> Result<Vec<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        let values: Vec<T> = self
            .parent_matches
            .as_ref()
            .and_then(|parent| parent.get_many::<T>(name).map(|iter| iter.cloned().collect()))
            .unwrap_or_default();

        if values.is_empty() {
            Err(crate::error::NounVerbError::argument_error(format!(
                "Required global argument '{}' is missing or has no values",
                name
            )))
        } else {
            Ok(values)
        }
    }

    /// Get multiple typed values from global arguments (optional)
    ///
    /// Retrieves multiple values from parent matches (global arguments).
    /// Returns an empty vector if the argument is not found or has no values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap::{Command, Arg, ArgAction};
    /// use clap_noun_verb::VerbArgs;
    ///
    /// let parent_cmd = Command::new("parent")
    ///     .arg(Arg::new("hosts").long("host").action(ArgAction::Append));
    /// let parent_matches = parent_cmd.get_matches_from(vec!["parent", "--host", "localhost"]);
    ///
    /// let cmd = Command::new("child");
    /// let matches = cmd.get_matches_from(vec!["child"]);
    ///
    /// let verb_args = VerbArgs::new(matches).with_parent(parent_matches);
    /// let hosts: Vec<String> = verb_args.get_global_many_opt("hosts");
    ///
    /// assert_eq!(hosts, vec!["localhost".to_string()]);
    /// ```
    pub fn get_global_many_opt<T>(&self, name: &str) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.parent_matches
            .as_ref()
            .and_then(|parent| parent.get_many::<T>(name).map(|iter| iter.cloned().collect()))
            .unwrap_or_default()
    }

    /// Get multiple string values from arguments using get_raw()
    ///
    /// Convenience method that retrieves multiple string values from the original
    /// CLI input, avoiding type mismatch panics when `value_parser` stores values
    /// as numeric types (e.g., u16, i64, f64).
    ///
    /// Works with any value_parser type by extracting the raw CLI string values.
    /// Returns an empty vector if the argument is not found or has no values.
    ///
    /// Use this method when:
    /// - Argument values were parsed into numeric or custom types
    /// - You need the original string representation from the CLI
    /// - You want flexible handling of multi-value arguments
    ///
    /// # Example
    ///
    /// ```rust
    /// use clap::{Command, Arg, ArgAction};
    /// use clap_noun_verb::VerbArgs;
    ///
    /// let cmd = Command::new("test")
    ///     .arg(Arg::new("ports").long("port").action(ArgAction::Append));
    /// let matches = cmd.get_matches_from(vec!["test", "--port", "80", "--port", "443"]);
    ///
    /// let verb_args = VerbArgs::new(matches);
    /// let ports: Vec<String> = verb_args.get_many_opt_str("ports");
    ///
    /// assert_eq!(ports, vec!["80".to_string(), "443".to_string()]);
    /// ```
    pub fn get_many_opt_str(&self, name: &str) -> Vec<String> {
        if let Some(raw_values) = self.matches.get_raw(name) {
            raw_values
                .into_iter()
                .filter_map(|os_str| os_str.to_str().map(|s| s.to_string()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a global flag is set (e.g., --verbose)
    pub fn is_global_flag_set(&self, name: &str) -> bool {
        self.parent_matches.as_ref().map(|parent| parent.get_flag(name)).unwrap_or(false)
    }

    /// Get global flag count (e.g., -v, -vv, -vvv)
    pub fn get_global_flag_count(&self, name: &str) -> u8 {
        self.parent_matches.as_ref().map(|parent| parent.get_count(name)).unwrap_or(0)
    }
}

/// Trait for defining verb commands (e.g., "status", "logs", "restart")
///
/// # Examples
///
/// Implementing `VerbCommand` directly:
///
/// ```rust
/// use clap_noun_verb::{VerbCommand, VerbArgs, Result};
///
/// struct StatusCommand;
///
/// impl VerbCommand for StatusCommand {
///     fn name(&self) -> &'static str { "status" }
///     fn about(&self) -> &'static str { "Show status" }
///     fn run(&self, _args: &VerbArgs) -> Result<()> {
///         println!("All services running");
///         Ok(())
///     }
/// }
/// ```
///
/// Using the `verb!` macro (recommended):
///
/// ```rust,no_run
/// use clap_noun_verb::{verb, VerbArgs, Result};
///
/// let _status = verb!("status", "Show status", |_args: &VerbArgs| -> Result<()> {
///     println!("All services running");
///     Ok(())
/// });
/// ```
pub trait VerbCommand: Send + Sync {
    /// The name of the verb command
    fn name(&self) -> &'static str;

    /// Description of what this verb command does
    fn about(&self) -> &'static str;

    /// Execute the verb command
    ///
    /// # Errors
    ///
    /// Returns `Result::Err` if command execution fails.
    fn run(&self, args: &VerbArgs) -> Result<()>;

    /// Build the clap command for this verb
    ///
    /// Default implementation creates a basic command with name and description.
    /// Override to customize command building.
    fn build_command(&self) -> Command {
        Command::new(self.name()).about(self.about())
    }

    /// Get additional arguments for this verb (override to add custom args)
    ///
    /// Returns an empty vector by default. Override to provide custom arguments.
    fn additional_args(&self) -> Vec<clap::Arg> {
        Vec::new()
    }
}
