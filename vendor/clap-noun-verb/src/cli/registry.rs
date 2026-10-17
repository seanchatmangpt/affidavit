// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compile-time command registry for attribute macros
//!
//! This module provides a registry that collects functions marked with
//! `#[verb]` attributes at compile time using linkme.
//!
//! These attribute macros are provided by the `clap-noun-verb-macros` crate.
//!
//! # Memory Management: Box::leak for Static Strings
//!
//! This module uses `Box::leak()` extensively to convert owned Strings into
//! `&'static str` references required by the clap library for command metadata.
//!
//! ## Why Box::leak is Used
//!
//! The clap command builder API requires all command names, help text, argument
//! names, etc. as `&'static str` (references with static lifetime). Converting
//! dynamic runtime strings (from metadata, configuration, etc.) to static
//! references requires "leaking" the memory so it persists for the program's
//! entire duration:
//!
//! ```rust,ignore
//! // What we need to do (16+ instances in this file):
//! let noun_name: &'static str = Box::leak(noun_name.to_string().into_boxed_str());
//! ```
//!
//! ## Memory Impact Assessment
//!
//! This pattern is **acceptable for CLI applications** because:
//!
//! - **Minimal total allocation**: Typical CLI has <100 commands → <50KB total leaked memory
//! - **One-time cost**: Leaks occur only during initialization (not in hot loops)
//! - **Unavoidable for clap integration**: There's no alternative that maintains
//!   clap's ergonomic API while supporting dynamic command discovery
//! - **Negligible impact**: CLI memory usage dominated by other factors
//!   (parsing, runtime state, etc.), not metadata strings
//!
//! ## Alternative Approaches (Not Used)
//!
//! For reference, other approaches we considered:
//!
//! 1. **once_cell/lazy_static**: Would require refactoring the entire command
//!    builder architecture and runtime registration system. Adds significant
//!    complexity with minimal benefit for CLI use case.
//!
//! 2. **Custom 'static lifetime manager**: Would require unsafe code and careful
//!    lifetime tracking. Not worth the complexity for CLI applications.
//!
//! 3. **Dynamic clap Commands**: Rebuild command structure dynamically (no static
//!    strings). Possible but defeats clap's performance benefits and requires
//!    restructuring around clap's static assumptions.
//!
//! For library use cases or long-running services, alternatives should be
//! investigated. For typical CLI applications, Box::leak is the idiomatic solution.

use crate::cli::value_parser;
use crate::error::Result;
use crate::logic::{HandlerInput, HandlerOutput};
use linkme::distributed_slice;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

thread_local! {
    pub static ACTIVE_COMMAND: std::cell::RefCell<Option<clap::Command>> = const { std::cell::RefCell::new(None) };
}

/// Apply validation constraints and auto-inferred parsers to a clap Arg
///
/// This function applies min/max value and length validators based on the
/// validation metadata stored in ArgMetadata, as well as auto-inferred
/// value parsers for common types.
///
/// For explicit value_parser expressions, it uses pattern matching on
/// the string representation to apply common patterns.
/// Parse a value hint string to clap::ValueHint
///
/// Supported hints (case-insensitive):
/// - file_path, filepath, file - File path
/// - dir_path, dirpath, dir, directory - Directory path
/// - any_path, anypath, path - Any path
/// - executable, exe - Executable command
/// - command, cmd, cmdname - Command name
/// - username, user - Username
/// - hostname, host - Hostname
/// - url - URL
/// - email - Email address
/// - other - Other
fn parse_value_hint(hint: &str) -> clap::ValueHint {
    match hint.to_lowercase().as_str() {
        "file_path" | "filepath" | "file" => clap::ValueHint::FilePath,
        "dir_path" | "dirpath" | "dir" | "directory" => clap::ValueHint::DirPath,
        "any_path" | "anypath" | "path" => clap::ValueHint::AnyPath,
        "executable" | "exe" => clap::ValueHint::ExecutablePath,
        "command" | "cmd" | "cmdname" | "command_name" => clap::ValueHint::CommandName,
        "commandstring" | "command_string" | "cmdstring" => clap::ValueHint::CommandString,
        "commandwitharguments" | "commandwithargs" => clap::ValueHint::CommandWithArguments,
        "username" | "user" => clap::ValueHint::Username,
        "hostname" | "host" => clap::ValueHint::Hostname,
        "url" => clap::ValueHint::Url,
        "email" | "emailaddress" => clap::ValueHint::EmailAddress,
        _ => clap::ValueHint::Other,
    }
}

///
/// Takes ownership of Arg and returns the modified Arg to avoid unnecessary
/// cloning while applying builder-pattern methods that consume and return self.
fn apply_validators(mut arg: clap::Arg, arg_meta: &ArgMetadata) -> clap::Arg {
    // Apply value parser if specified (auto-inferred or explicit)
    // Note: value_parser is stored as a string representation, so we match on the string
    if let Some(ref vp_str) = arg_meta.value_parser {
        // Try to apply value parser from pattern matching
        if value_parser::apply_value_parser(&mut arg, vp_str) {
            return arg;
        }
    }

    // For numeric types with min/max values, apply range validators
    if arg_meta.min_value.is_some() || arg_meta.max_value.is_some() {
        // Try to parse as integers first
        let min_i64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let max_i64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<i64>().ok());
        let min_u64 = arg_meta.min_value.as_ref().and_then(|v| v.parse::<u64>().ok());
        let max_u64 = arg_meta.max_value.as_ref().and_then(|v| v.parse::<u64>().ok());

        // Apply range validators based on what we can parse using match for clarity
        match (min_i64, max_i64, min_u64, max_u64) {
            (Some(min), Some(max), _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(min..=max));
            }
            (Some(min), None, _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(min..));
            }
            (None, Some(max), _, _) => {
                arg = arg.value_parser(clap::value_parser!(i64).range(..=max));
            }
            (_, _, Some(min), Some(max)) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(min..=max));
            }
            (_, _, Some(min), None) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(min..));
            }
            (_, _, None, Some(max)) => {
                arg = arg.value_parser(clap::value_parser!(u64).range(..=max));
            }
            _ => {}
        }
    }

    // For string types with min_length, ensure non-empty
    if let Some(min_len) = arg_meta.min_length {
        if min_len > 0 {
            arg = arg.value_parser(clap::builder::NonEmptyStringValueParser::new());
        }
    }

    arg
}

/// Distributed slice for noun registrations
#[distributed_slice]
pub static __NOUN_REGISTRY: [fn()] = [..];

/// Distributed slice for verb registrations
#[distributed_slice]
pub static __VERB_REGISTRY: [fn()] = [..];

/// Global registry for registered commands
static REGISTRY: OnceLock<Mutex<CommandRegistry>> = OnceLock::new();

/// Command registry for attribute macro discovered functions
pub struct CommandRegistry {
    /// Registered nouns (name -> noun metadata)
    nouns: HashMap<String, NounMetadata>,
    /// Registered verbs (noun_name -> verb_name -> verb metadata)
    verbs: HashMap<String, HashMap<String, VerbMetadata>>,
    /// Root-level verbs (verb_name -> verb metadata) - verbs without a noun
    root_verbs: HashMap<String, VerbMetadata>,
}

/// Metadata for a registered noun
struct NounMetadata {
    #[allow(dead_code)] // Reserved for future use
    name: String,
    about: String,
    long_about: Option<String>,
}

/// Argument metadata for a verb function parameter
#[derive(Clone)]
pub struct ArgMetadata {
    pub name: String,
    pub required: bool,
    pub is_flag: bool,
    pub help: Option<String>,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    /// Short flag character (e.g., 'v' for -v)
    pub short: Option<char>,
    /// Default value as string (will be parsed by clap)
    pub default_value: Option<String>,
    /// Environment variable name
    pub env: Option<String>,
    /// Whether this argument accepts multiple values
    pub multiple: bool,
    /// Custom value name for help text (e.g., "FILE", "PORT")
    pub value_name: Option<String>,
    /// Aliases for the argument (e.g., ["verbose", "v"])
    pub aliases: Vec<String>,
    /// Positional argument index (e.g., 0, 1, 2)
    pub positional: Option<usize>,
    /// Custom action type (e.g., Count, SetFalse)
    pub action: Option<clap::ArgAction>,
    /// Argument group name (for exclusive/multiple groups)
    pub group: Option<String>,
    /// Arguments this requires
    pub requires: Vec<String>,
    /// Arguments this conflicts with
    pub conflicts_with: Vec<String>,
    /// Custom value parser expression (stored as string for macro expansion)
    pub value_parser: Option<String>,
    /// Hide from help text
    pub hide: bool,
    /// Next help heading (for grouping in help)
    pub next_help_heading: Option<String>,
    /// Long help text (separate from help)
    pub long_help: Option<String>,
    /// Next line help formatting
    pub next_line_help: bool,
    /// Display order in help output
    pub display_order: Option<usize>,
    /// Exclusive group flag
    pub exclusive: Option<bool>,
    /// Trailing varargs support
    pub trailing_vararg: bool,
    /// Allow negative numbers
    pub allow_negative_numbers: bool,
    /// Value hint for shell completion (e.g., FilePath, DirPath, Url)
    pub value_hint: Option<String>,
    /// Global flag - propagates to subcommands
    pub global: bool,
}

/// Metadata for a registered verb
struct VerbMetadata {
    #[allow(dead_code)] // Reserved for future use
    noun_name: String,
    #[allow(dead_code)] // Reserved for future use
    verb_name: String,
    about: String,
    args: Vec<ArgMetadata>,
    handler_fn: Box<dyn Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync>,
}

impl CommandRegistry {
    /// Initialize the registry (called once during first access)
    pub fn init() -> &'static Mutex<CommandRegistry> {
        let registry = REGISTRY.get_or_init(|| {
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });

        for init_fn in __NOUN_REGISTRY {
            init_fn();
        }
        for init_fn in __VERB_REGISTRY {
            init_fn();
        }

        registry
    }

    /// Get the global registry instance
    pub fn get() -> &'static Mutex<CommandRegistry> {
        Self::init()
    }

    /// Register a noun (called by macro-generated code)
    pub fn register_noun(name: &'static str, about: &'static str) {
        let registry = REGISTRY.get_or_init(|| {
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());
        reg.nouns.entry(name.to_string()).or_insert_with(|| NounMetadata {
            name: name.to_string(),
            about: about.to_string(),
            long_about: None,
        });
    }

    /// Register a verb (called by macro-generated code)
    pub fn register_verb<F>(
        noun_name: &'static str,
        verb_name: &'static str,
        about: &'static str,
        handler: F,
    ) where
        F: Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync + 'static,
    {
        Self::register_verb_with_args(noun_name, verb_name, about, Vec::new(), handler)
    }

    /// Register a verb with argument metadata
    pub fn register_verb_with_args<F>(
        noun_name: &'static str,
        verb_name: &'static str,
        about: &'static str,
        args: Vec<ArgMetadata>,
        handler: F,
    ) where
        F: Fn(HandlerInput) -> Result<HandlerOutput> + Send + Sync + 'static,
    {
        let registry = REGISTRY.get_or_init(|| {
            Mutex::new(CommandRegistry {
                nouns: HashMap::new(),
                verbs: HashMap::new(),
                root_verbs: HashMap::new(),
            })
        });
        let mut reg = registry.lock().unwrap_or_else(|e| e.into_inner());

        let verb_metadata = VerbMetadata {
            noun_name: noun_name.to_string(),
            verb_name: verb_name.to_string(),
            about: about.to_string(),
            args,
            handler_fn: Box::new(handler),
        };

        if noun_name.is_empty() {
            reg.root_verbs.insert(verb_name.to_string(), verb_metadata);
        } else {
            reg.verbs
                .entry(noun_name.to_string())
                .or_default()
                .insert(verb_name.to_string(), verb_metadata);
        }
    }

    /// Get all registered nouns
    pub fn get_nouns(&self) -> Vec<(&str, &str)> {
        self.nouns.iter().map(|(name, meta)| (name.as_str(), meta.about.as_str())).collect()
    }

    /// Get all verbs for a noun
    pub fn get_verbs(&self, noun_name: &str) -> Vec<(&str, &str)> {
        self.verbs
            .get(noun_name)
            .map(|verbs| {
                verbs.iter().map(|(name, meta)| (name.as_str(), meta.about.as_str())).collect()
            })
            .unwrap_or_default()
    }

    /// Get all verbs for a noun with their full metadata including arguments
    pub fn get_verbs_with_metadata(&self, noun_name: &str) -> Vec<(&str, &str, &Vec<ArgMetadata>)> {
        self.verbs
            .get(noun_name)
            .map(|verbs| {
                verbs
                    .iter()
                    .map(|(name, meta)| (name.as_str(), meta.about.as_str(), &meta.args))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all registered noun names
    pub fn get_all_noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|n| n.as_str()).collect()
    }

    /// Execute a verb handler
    pub fn execute_verb(
        &self,
        noun_name: &str,
        verb_name: &str,
        input: HandlerInput,
    ) -> Result<HandlerOutput> {
        let verbs = self.verbs.get(noun_name).ok_or_else(|| {
            let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
            crate::error::NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
        })?;

        let verb = verbs.get(verb_name).ok_or_else(|| {
            let candidates: Vec<&str> = verbs.keys().map(|s| s.as_str()).collect();
            crate::error::NounVerbError::verb_not_found_with_candidates(
                noun_name,
                verb_name,
                &candidates,
            )
        })?;

        (verb.handler_fn)(input)
    }

    /// Build clap command structure from registry
    pub fn build_command(&self) -> clap::Command {
        let mut cmd = clap::Command::new("cli")
            .version(env!("CARGO_PKG_VERSION"))
            .arg_required_else_help(true)
            .arg(clap::Arg::new("format")
                .long("format")
                .global(true)
                .value_parser(clap::builder::PossibleValuesParser::new(crate::format::OutputFormat::available_formats()))
                .help("Output format"))
            .arg(clap::Arg::new("select")
                .long("select")
                .global(true)
                .help("Select/project nested JSON output using JSONPath, key selection, or JMESPath query projections"))
            .arg(clap::Arg::new("introspect")
                .long("introspect")
                .global(true)
                .action(clap::ArgAction::SetTrue)
                .help("Introspect CLI capabilities as JSON Schema array for LLM tool-calling"))
            .arg(clap::Arg::new("structured-errors")
                .long("structured-errors")
                .global(true)
                .action(clap::ArgAction::SetTrue)
                .help("Output errors using StructuredError format"))
            .arg(clap::Arg::new("autonomic")
                .long("autonomic")
                .global(true)
                .action(clap::ArgAction::SetTrue)
                .help("Enable autonomic features and output structured errors"));

        // Add root-level verbs directly as subcommands
        for (verb_name, verb_meta) in &self.root_verbs {
            let verb_cmd = self.build_verb_command(verb_name, verb_meta);
            cmd = cmd.subcommand(verb_cmd);
        }

        // Add nouns with their nested verbs
        for (noun_name, noun_meta) in &self.nouns {
            let noun_cmd = self.build_noun_command(noun_name, noun_meta);
            cmd = cmd.subcommand(noun_cmd);
        }

        ACTIVE_COMMAND.with(|cell| {
            *cell.borrow_mut() = Some(cmd.clone());
        });

        cmd
    }

    /// Build a noun command with all its verb subcommands
    fn build_noun_command(&self, noun_name: &str, noun_meta: &NounMetadata) -> clap::Command {
        let noun_name_static: &'static str = Box::leak(noun_name.to_string().into_boxed_str());
        let about: &'static str = Box::leak(noun_meta.about.clone().into_boxed_str());
        let mut noun_cmd = clap::Command::new(noun_name_static).about(about);

        if let Some(ref long_about) = noun_meta.long_about {
            let long_about_static: &'static str = Box::leak(long_about.clone().into_boxed_str());
            noun_cmd = noun_cmd.long_about(long_about_static);
        }

        if let Some(verbs) = self.verbs.get(noun_name) {
            for (verb_name, verb_meta) in verbs {
                let verb_cmd = self.build_verb_command(verb_name, verb_meta);
                noun_cmd = noun_cmd.subcommand(verb_cmd);
            }
        }

        noun_cmd
    }

    /// Build a verb command with all its arguments
    fn build_verb_command(&self, verb_name: &str, verb_meta: &VerbMetadata) -> clap::Command {
        let verb_name_static: &'static str = Box::leak(verb_name.to_string().into_boxed_str());
        let about: &'static str = Box::leak(verb_meta.about.clone().into_boxed_str());
        let mut verb_cmd = clap::Command::new(verb_name_static).about(about);

        verb_cmd = self.add_arg_groups(verb_cmd, verb_meta);
        verb_cmd = self.add_arguments(verb_cmd, verb_meta);

        verb_cmd
    }

    /// Add argument groups to a command
    fn add_arg_groups(
        &self,
        mut verb_cmd: clap::Command,
        verb_meta: &VerbMetadata,
    ) -> clap::Command {
        let mut groups: std::collections::HashMap<String, (bool, Vec<String>)> =
            std::collections::HashMap::new();
        for arg_meta in &verb_meta.args {
            if let Some(ref group_name) = arg_meta.group {
                let exclusive = arg_meta.exclusive.unwrap_or(true);
                let entry =
                    groups.entry(group_name.clone()).or_insert_with(|| (exclusive, Vec::new()));
                entry.1.push(arg_meta.name.clone());
                if !exclusive {
                    entry.0 = false;
                }
            }
        }

        for (group_name, (exclusive, arg_names)) in &groups {
            if arg_names.len() > 1 {
                let group_static: &'static str = Box::leak(group_name.clone().into_boxed_str());
                let group = clap::ArgGroup::new(group_static).multiple(!exclusive);
                let mut group = group;
                for arg_name in arg_names {
                    let arg_name_static: &'static str =
                        Box::leak(arg_name.clone().into_boxed_str());
                    group = group.arg(arg_name_static);
                }
                verb_cmd = verb_cmd.group(group);
            }
        }

        verb_cmd
    }

    /// Add arguments to a command
    fn add_arguments(
        &self,
        mut verb_cmd: clap::Command,
        verb_meta: &VerbMetadata,
    ) -> clap::Command {
        for arg_meta in &verb_meta.args {
            let arg = self.build_argument(arg_meta);
            verb_cmd = verb_cmd.arg(arg);
        }
        verb_cmd
    }

    /// Build a single argument
    fn build_argument(&self, arg_meta: &ArgMetadata) -> clap::Arg {
        let arg_name: &'static str = Box::leak(arg_meta.name.clone().into_boxed_str());
        let default_value_name: &'static str =
            Box::leak(arg_meta.name.to_uppercase().into_boxed_str());

        let mut arg = if let Some(index) = arg_meta.positional {
            let mut pos_arg = clap::Arg::new(arg_name).index(index);
            if arg_meta.trailing_vararg {
                pos_arg = pos_arg.num_args(1..);
            }
            pos_arg
        } else {
            if matches!(arg_meta.action.as_ref(), Some(clap::ArgAction::SetFalse)) {
                let long_name = format!("no-{}", arg_name.replace('_', "-"));
                let long_static: &'static str = Box::leak(long_name.into_boxed_str());
                clap::Arg::new(arg_name).long(long_static)
            } else {
                // Canonical long flag is idiomatic kebab-case (--profile-id), matching
                // clap's derive convention. The verbatim snake_case spelling
                // (--profile_id) is kept as an alias so existing scripts/tests that
                // used the underscore form keep working (backward compatible).
                let long_name = arg_name.replace('_', "-");
                if long_name == arg_name {
                    clap::Arg::new(arg_name).long(arg_name)
                } else {
                    let long_static: &'static str = Box::leak(long_name.into_boxed_str());
                    clap::Arg::new(arg_name).long(long_static).alias(arg_name)
                }
            }
        };

        if arg_meta.positional.is_none() {
            if let Some(short_char) = arg_meta.short {
                arg = arg.short(short_char);
            }

            for alias in &arg_meta.aliases {
                let alias_static: &'static str = Box::leak(alias.clone().into_boxed_str());
                arg = arg.alias(alias_static);
            }
        }

        if let Some(ref env_var) = arg_meta.env {
            let env_static: &'static str = Box::leak(env_var.clone().into_boxed_str());
            arg = arg.env(env_static);
        }

        if let Some(ref default_val) = arg_meta.default_value {
            let default_static: &'static str = Box::leak(default_val.clone().into_boxed_str());
            arg = arg.default_value(default_static);
        }

        if let Some(action) = &arg_meta.action {
            arg = arg.action(action.clone());
        } else if arg_meta.is_flag {
            arg = arg.action(clap::ArgAction::SetTrue);
        } else {
            let value_name: &'static str = if let Some(ref vn) = arg_meta.value_name {
                Box::leak(vn.clone().into_boxed_str())
            } else {
                default_value_name
            };
            arg = arg.value_name(value_name);

            if arg_meta.multiple {
                arg = arg.action(clap::ArgAction::Append);
            }

            if arg_meta.required {
                arg = arg.required(true);
            }

            arg = apply_validators(arg, arg_meta);

            if arg_meta.allow_negative_numbers {
                arg = arg.allow_negative_numbers(true);
            }
        }

        if let Some(help_text) = &arg_meta.help {
            let help: &'static str = Box::leak(help_text.clone().into_boxed_str());
            arg = arg.help(help);
        }

        if let Some(long_help_text) = &arg_meta.long_help {
            let long_help: &'static str = Box::leak(long_help_text.clone().into_boxed_str());
            arg = arg.long_help(long_help);
        }

        if arg_meta.next_line_help {
            arg = arg.next_line_help(true);
        }

        if let Some(order) = arg_meta.display_order {
            arg = arg.display_order(order);
        }

        for req in &arg_meta.requires {
            let req_static: &'static str = Box::leak(req.clone().into_boxed_str());
            arg = arg.requires(req_static);
        }

        for conflict in &arg_meta.conflicts_with {
            let conflict_static: &'static str = Box::leak(conflict.clone().into_boxed_str());
            arg = arg.conflicts_with(conflict_static);
        }

        if let Some(ref group_name) = arg_meta.group {
            let group_static: &'static str = Box::leak(group_name.clone().into_boxed_str());
            arg = arg.group(group_static);
        }

        if arg_meta.hide {
            arg = arg.hide(true);
        }

        if let Some(ref heading) = arg_meta.next_help_heading {
            let heading_static: &'static str = Box::leak(heading.clone().into_boxed_str());
            arg = arg.help_heading(heading_static);
        }

        if let Some(ref hint) = arg_meta.value_hint {
            arg = arg.value_hint(parse_value_hint(hint));
        }

        if arg_meta.global {
            arg = arg.global(true);
        }

        if let Some(true) = arg_meta.exclusive {
            arg = arg.exclusive(true);
        }

        arg
    }

    /// Extract a value from ArgMatches as a string
    fn extract_value_as_string(verb_matches: &clap::ArgMatches, arg_name: &str) -> Option<String> {
        if let Some(raw_values) = verb_matches.get_raw(arg_name) {
            if let Some(os_str) = raw_values.into_iter().next() {
                return os_str.to_str().map(|s| s.to_string());
            }
        }
        None
    }

    /// Extract arguments from clap matches into a HashMap
    fn extract_args(
        &self,
        verb_meta: &VerbMetadata,
        verb_matches: &clap::ArgMatches,
    ) -> std::collections::HashMap<String, String> {
        let mut args_map = std::collections::HashMap::new();

        for arg_meta in &verb_meta.args {
            let arg_name = &arg_meta.name;

            if let Some(_index) = arg_meta.positional {
                if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                    args_map.insert(arg_name.clone(), value);
                }
            } else if let Some(action) = &arg_meta.action {
                match action {
                    clap::ArgAction::Count => {
                        let count = verb_matches.get_count(arg_name);
                        args_map.insert(arg_name.clone(), count.to_string());
                    }
                    clap::ArgAction::SetTrue => {
                        if verb_matches.get_flag(arg_name) {
                            args_map.insert(arg_name.clone(), "true".to_string());
                        }
                    }
                    clap::ArgAction::SetFalse => {
                        if verb_matches.get_flag(arg_name) {
                            args_map.insert(arg_name.clone(), "true".to_string());
                        } else {
                            args_map.insert(arg_name.clone(), "false".to_string());
                        }
                    }
                    clap::ArgAction::Append => {
                        if let Some(values) = verb_matches.get_many::<String>(arg_name) {
                            let values_vec: Vec<String> = values.cloned().collect();
                            args_map.insert(arg_name.clone(), values_vec.join(","));
                        }
                    }
                    _ => {
                        if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                            args_map.insert(arg_name.clone(), value);
                        }
                    }
                }
            } else if arg_meta.is_flag {
                if verb_matches.get_flag(arg_name) {
                    args_map.insert(arg_name.clone(), "true".to_string());
                }
            } else {
                if let Some(value) = Self::extract_value_as_string(verb_matches, arg_name) {
                    args_map.insert(arg_name.clone(), value);
                }
            }
        }

        args_map
    }

    /// Run CLI with auto-discovered commands
    pub fn run(&self, args: Vec<String>) -> Result<()> {
        if args.is_empty() {
            return Err(crate::error::NounVerbError::argument_error("No arguments provided"));
        }

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

        // If no steps, run default help
        if steps.is_empty() {
            let mut cmd = self.build_command();
            cmd.print_help().map_err(|e| {
                crate::error::NounVerbError::execution_error(format!("Failed to print help: {}", e))
            })?;
            return Ok(());
        }

        // Read stdin once if needed by any step
        let stdin_val = crate::cli::preprocessor::read_stdin_if_needed(&steps);
        let mut step_results: Vec<serde_json::Value> = Vec::new();

        for step in steps {
            let mut step_args = vec![binary_name.clone()];
            let processed_args =
                crate::cli::preprocessor::preprocess_args(&step, &stdin_val, &step_results)?;
            step_args.extend(processed_args);

            let output = self.execute_single_step(step_args)?;
            step_results.push(output.data);
        }

        Ok(())
    }

    /// Execute a single CLI command step and return the handler output
    pub fn execute_single_step(&self, args: Vec<String>) -> Result<HandlerOutput> {
        let cmd = self.build_command();

        let requested = args.iter().any(|arg| arg == "--structured-errors" || arg == "--autonomic")
            || std::env::var("STRUCTURED_ERRORS").is_ok()
            || std::env::var("AUTONOMIC").is_ok();

        let matches = match cmd.clone().try_get_matches_from(args) {
            Ok(m) => m,
            Err(e) => {
                let exit_code = e.exit_code();
                let help_or_version_msg = e.to_string();

                if requested {
                    let err =
                        crate::error::NounVerbError::argument_error(help_or_version_msg.clone());
                    let structured = crate::error::StructuredError::from_error(&err);
                    let formatted =
                        serde_json::to_string_pretty(&serde_json::json!({ "error": structured }))
                            .unwrap_or_else(|_| "{}".to_string());
                    eprintln!("{}", formatted);
                    return Err(err);
                }

                print!("{}", help_or_version_msg);

                if exit_code == 0 {
                    return Ok(HandlerOutput {
                        data: serde_json::Value::Null,
                        message: Some(help_or_version_msg),
                    });
                } else {
                    return Err(crate::error::NounVerbError::argument_error(help_or_version_msg));
                }
            }
        };

        if matches.get_flag("introspect") {
            let tools = crate::registry::collect_tools_from_cmd(&cmd, "");
            let json_str = serde_json::to_string_pretty(&tools)
                .map_err(|e| crate::error::NounVerbError::execution_error(e.to_string()))?;
            println!("{}", json_str);
            return Ok(HandlerOutput { data: serde_json::Value::Null, message: Some(json_str) });
        }

        let format_str = matches.get_one::<String>("format").cloned().or_else(|| {
            if let Some((_, sub_matches)) = matches.subcommand() {
                sub_matches.get_one::<String>("format").cloned().or_else(|| {
                    if let Some((_, verb_matches)) = sub_matches.subcommand() {
                        verb_matches.get_one::<String>("format").cloned()
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        });

        let output_format = format_str
            .as_deref()
            .and_then(|s| s.parse::<crate::format::OutputFormat>().ok())
            .unwrap_or(crate::format::OutputFormat::JsonPretty);

        let flag_requested = matches.get_flag("structured-errors") || matches.get_flag("autonomic");

        let result = self.execute_step_internal(&matches, output_format);
        if let Err(ref e) = result {
            if requested || flag_requested {
                let structured = crate::error::StructuredError::from_error(e);
                let formatted = match output_format {
                    crate::format::OutputFormat::Json => {
                        serde_json::to_string(&serde_json::json!({ "error": structured }))
                            .unwrap_or_else(|_| "{}".to_string())
                    }
                    crate::format::OutputFormat::Yaml => {
                        format!(
                            "error:\n  kind: {:?}\n  severity: {:?}\n  message: \"{}\"\n",
                            structured.kind, structured.severity, structured.message
                        )
                    }
                    _ => serde_json::to_string_pretty(&serde_json::json!({ "error": structured }))
                        .unwrap_or_else(|_| "{}".to_string()),
                };
                eprintln!("{}", formatted);
            }
        }
        result
    }

    fn execute_step_internal(
        &self,
        matches: &clap::ArgMatches,
        output_format: crate::format::OutputFormat,
    ) -> Result<HandlerOutput> {
        let select_str = matches.get_one::<String>("select").cloned().or_else(|| {
            if let Some((_, sub_matches)) = matches.subcommand() {
                sub_matches.get_one::<String>("select").cloned().or_else(|| {
                    if let Some((_, verb_matches)) = sub_matches.subcommand() {
                        verb_matches.get_one::<String>("select").cloned()
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        });

        if let Some((subcommand_name, sub_matches)) = matches.subcommand() {
            if let Some(verb_meta) = self.root_verbs.get(subcommand_name) {
                let args_map = self.extract_args(verb_meta, sub_matches);

                let input = crate::logic::HandlerInput {
                    args: args_map,
                    opts: std::collections::HashMap::new(),
                    context: crate::logic::HandlerContext::new(subcommand_name),
                };

                let mut output = self.execute_root_verb(subcommand_name, input)?;
                if let Some(ref select_expr) = select_str {
                    output.data = apply_select(&output.data, select_expr).map_err(|e| {
                        crate::error::NounVerbError::execution_error(format!(
                            "Selection error: {}",
                            e
                        ))
                    })?;
                }
                let formatted = output_format.format(&output.data).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!("Format error: {}", e))
                })?;
                // VENDORED PATCH (affidavit B13): a unit-returning verb
                // (`Result<()>`) serializes to Value::Null; printing the
                // formatted Null emitted a stray trailing "null" line after
                // every success path. Suppress the data print when the verb
                // returned no data — the verb's own output stands alone.
                if output_format != crate::format::OutputFormat::Quiet && !output.data.is_null() {
                    println!("{}", formatted);
                }
                Ok(output)
            } else if let Some((verb_name, verb_matches)) = sub_matches.subcommand() {
                let noun_name = subcommand_name;
                let args_map = if let Some(verbs) = self.verbs.get(noun_name) {
                    if let Some(verb_meta) = verbs.get(verb_name) {
                        self.extract_args(verb_meta, verb_matches)
                    } else {
                        std::collections::HashMap::new()
                    }
                } else {
                    std::collections::HashMap::new()
                };

                let input = crate::logic::HandlerInput {
                    args: args_map,
                    opts: std::collections::HashMap::new(),
                    context: crate::logic::HandlerContext::new(verb_name).with_noun(noun_name),
                };

                let mut output = self.execute_verb(noun_name, verb_name, input)?;
                if let Some(ref select_expr) = select_str {
                    output.data = apply_select(&output.data, select_expr).map_err(|e| {
                        crate::error::NounVerbError::execution_error(format!(
                            "Selection error: {}",
                            e
                        ))
                    })?;
                }
                let formatted = output_format.format(&output.data).map_err(|e| {
                    crate::error::NounVerbError::execution_error(format!("Format error: {}", e))
                })?;
                // VENDORED PATCH (affidavit B13): a unit-returning verb
                // (`Result<()>`) serializes to Value::Null; printing the
                // formatted Null emitted a stray trailing "null" line after
                // every success path. Suppress the data print when the verb
                // returned no data — the verb's own output stands alone.
                if output_format != crate::format::OutputFormat::Quiet && !output.data.is_null() {
                    println!("{}", formatted);
                }
                Ok(output)
            } else {
                let noun_name = subcommand_name;
                if let Some(noun_meta) = self.nouns.get(noun_name) {
                    let noun_name_static: &'static str =
                        Box::leak(noun_name.to_string().into_boxed_str());
                    let about_static: &'static str =
                        Box::leak(noun_meta.about.clone().into_boxed_str());

                    let mut noun_cmd = clap::Command::new(noun_name_static).about(about_static);

                    if let Some(ref long_about) = noun_meta.long_about {
                        let long_about_static: &'static str =
                            Box::leak(long_about.clone().into_boxed_str());
                        noun_cmd = noun_cmd.long_about(long_about_static);
                    }

                    if let Some(verbs) = self.verbs.get(noun_name) {
                        for (verb_name, verb_meta) in verbs {
                            let verb_name_static: &'static str =
                                Box::leak(verb_name.clone().into_boxed_str());
                            let verb_about_static: &'static str =
                                Box::leak(verb_meta.about.clone().into_boxed_str());
                            noun_cmd = noun_cmd.subcommand(
                                clap::Command::new(verb_name_static).about(verb_about_static),
                            );
                        }
                    }

                    noun_cmd.print_help().map_err(|e| {
                        crate::error::NounVerbError::execution_error(format!(
                            "Failed to print help: {}",
                            e
                        ))
                    })?;
                    Ok(HandlerOutput { data: serde_json::Value::Null, message: None })
                } else {
                    Err(crate::error::NounVerbError::invalid_structure("No verb specified"))
                }
            }
        } else {
            let mut cmd = self.build_command();
            cmd.print_help().map_err(|e| {
                crate::error::NounVerbError::execution_error(format!("Failed to print help: {}", e))
            })?;
            Ok(HandlerOutput { data: serde_json::Value::Null, message: None })
        }
    }

    /// Execute a root-level verb handler (verbs without a noun)
    pub fn execute_root_verb(&self, verb_name: &str, input: HandlerInput) -> Result<HandlerOutput> {
        let verb = self.root_verbs.get(verb_name).ok_or_else(|| {
            let mut candidates: Vec<&str> = self.root_verbs.keys().map(|s| s.as_str()).collect();
            candidates.extend(self.nouns.keys().map(|s| s.as_str()));
            crate::error::NounVerbError::command_not_found_with_candidates(verb_name, &candidates)
        })?;

        (verb.handler_fn)(input)
    }
}

fn apply_select(
    value: &serde_json::Value,
    expr: &str,
) -> std::result::Result<serde_json::Value, String> {
    let clean_expr = if expr == "$" || expr == "@" {
        "@"
    } else if expr.starts_with("$.") {
        &expr[2..]
    } else if expr.starts_with("$[") {
        &expr[1..]
    } else {
        expr
    };

    if clean_expr == "@" {
        return Ok(value.clone());
    }

    let compiled = jmespath::compile(clean_expr)
        .map_err(|e| format!("Invalid query expression '{}': {}", expr, e))?;

    let result = compiled
        .search(value)
        .map_err(|e| format!("Failed to evaluate query '{}': {}", expr, e))?;

    let json_val =
        serde_json::to_value(&*result).map_err(|e| format!("Serialization error: {}", e))?;

    Ok(json_val)
}
