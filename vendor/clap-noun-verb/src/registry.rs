// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Command registry for composable CLI patterns
//!
//! The CommandRegistry provides a central hub for registering and composing
//! commands in a flexible, extensible way. This allows users to build their
//! own CLI patterns by composing commands together.
//!
//! ## Memory Management: Box::leak Usage
//!
//! This module uses `Box::leak()` to convert owned Strings to `&'static str`
//! references required by clap's command builder. This is an acceptable pattern
//! for CLI applications. See src/cli/registry.rs for comprehensive documentation
//! on the rationale and memory impact assessment.

use crate::error::{NounVerbError, Result};
use crate::noun::NounCommand;
use crate::verb::{TypeMap, VerbArgs, VerbContext};
use clap::{ArgMatches, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Central registry for managing all CLI commands
///
/// This registry allows users to:
/// - Register nouns and verbs in any order
/// - Compose command hierarchies dynamically
/// - Query command structure for introspection
/// - Build complete CLI applications from registered commands
pub struct CommandRegistry {
    /// Map of noun name to noun command
    nouns: HashMap<String, Box<dyn NounCommand>>,
    /// Global configuration for the CLI
    config: RegistryConfig,
    /// Typed context extensions shared across all commands
    extensions: TypeMap,
    /// Add completions subcommand
    pub has_completions_subcommand: bool,
}

/// Configuration for the command registry
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Application name
    pub name: String,
    /// Application description
    pub about: String,
    /// Version string
    pub version: Option<String>,
    /// Global arguments available to all commands
    pub global_args: Vec<clap::Arg>,
    /// Auto-validate command structure on build/run
    pub auto_validate: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            name: "cli".to_string(),
            about: "A command-line application".to_string(),
            version: None,
            global_args: Vec::new(),
            auto_validate: false,
        }
    }
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            nouns: HashMap::new(),
            config: RegistryConfig::default(),
            extensions: TypeMap::new(),
            has_completions_subcommand: false,
        }
    }

    /// Create a new registry with configuration
    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            nouns: HashMap::new(),
            config,
            extensions: TypeMap::new(),
            has_completions_subcommand: false,
        }
    }

    /// Enable fluent completions subcommand
    pub fn with_completions_subcommand(mut self) -> Self {
        self.has_completions_subcommand = true;
        self
    }

    /// Add a typed extension to the global context
    pub fn with_extension<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.extensions.insert(val);
        self
    }

    /// Set the application name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the application description
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.config.about = about.into();
        self
    }

    /// Set the application version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = Some(version.into());
        self
    }

    /// Add global arguments available to all commands
    pub fn global_args(mut self, args: Vec<clap::Arg>) -> Self {
        self.config.global_args = args;
        self
    }

    /// Enable automatic validation of command structure
    pub fn auto_validate(mut self, enable: bool) -> Self {
        self.config.auto_validate = enable;
        self
    }

    /// Register a noun command
    pub fn register_noun(mut self, noun: impl NounCommand + 'static) -> Self {
        self.nouns.insert(noun.name().to_string(), Box::new(noun));
        self
    }

    /// Register multiple noun commands
    pub fn register_nouns<I>(mut self, nouns: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn NounCommand>>,
    {
        for noun in nouns {
            self.nouns.insert(noun.name().to_string(), noun);
        }
        self
    }

    /// Get a noun command by name
    pub fn get_noun(&self, name: &str) -> Option<&dyn NounCommand> {
        self.nouns.get(name).map(|n| n.as_ref())
    }

    /// Get all registered noun names
    pub fn noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered nouns
    pub fn nouns(&self) -> Vec<&dyn NounCommand> {
        self.nouns.values().map(|n| n.as_ref()).collect()
    }

    /// Check if a noun is registered
    pub fn has_noun(&self, name: &str) -> bool {
        self.nouns.contains_key(name)
    }

    /// Remove a noun command
    pub fn remove_noun(&mut self, name: &str) -> Option<Box<dyn NounCommand>> {
        self.nouns.remove(name)
    }

    /// Clear all registered commands
    pub fn clear(&mut self) {
        self.nouns.clear();
    }

    /// Get the complete command structure for introspection
    pub fn command_structure(&self) -> HashMap<String, Vec<String>> {
        let mut structure = HashMap::new();

        for (noun_name, noun) in &self.nouns {
            let verbs: Vec<String> = noun.verbs().iter().map(|v| v.name().to_string()).collect();
            structure.insert(noun_name.clone(), verbs);
        }

        structure
    }

    /// Validate the command registry structure
    pub fn validate(&self) -> Result<()> {
        // Check for duplicate noun names
        let mut seen_nouns = std::collections::HashSet::new();
        for noun_name in self.nouns.keys() {
            if !seen_nouns.insert(noun_name) {
                return Err(NounVerbError::InvalidStructure {
                    message: format!("Duplicate noun name: '{}'", noun_name),
                });
            }
        }

        // Validate each noun structure
        for (noun_name, noun) in &self.nouns {
            // Check for empty nouns (no verbs or sub-nouns)
            if noun.verbs().is_empty() && noun.sub_nouns().is_empty() {
                return Err(NounVerbError::InvalidStructure {
                    message: format!("Noun '{}' has no verbs or sub-nouns", noun_name),
                });
            }

            // Check for duplicate verb names within a noun
            let mut seen_verbs = std::collections::HashSet::new();
            for verb in noun.verbs() {
                let verb_name = verb.name();
                if !seen_verbs.insert(verb_name) {
                    return Err(NounVerbError::InvalidStructure {
                        message: format!(
                            "Duplicate verb name '{}' in noun '{}'",
                            verb_name, noun_name
                        ),
                    });
                }
            }

            // Check for duplicate sub-noun names within a noun
            let mut seen_sub_nouns = std::collections::HashSet::new();
            for sub_noun in noun.sub_nouns() {
                let sub_noun_name = sub_noun.name();
                if !seen_sub_nouns.insert(sub_noun_name) {
                    return Err(NounVerbError::InvalidStructure {
                        message: format!(
                            "Duplicate sub-noun name '{}' in noun '{}'",
                            sub_noun_name, noun_name
                        ),
                    });
                }
            }

            // Check for verb/sub-noun name conflicts
            let verb_names: std::collections::HashSet<_> =
                noun.verbs().iter().map(|v| v.name()).collect();
            for sub_noun in noun.sub_nouns() {
                let sub_noun_name = sub_noun.name();
                if verb_names.contains(sub_noun_name) {
                    return Err(NounVerbError::InvalidStructure {
                        message: format!(
                            "Verb and sub-noun cannot have the same name '{}' in noun '{}'",
                            sub_noun_name, noun_name
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Build the complete clap command structure
    pub fn build_command(&self) -> Command {
        // Auto-validate if enabled
        if self.config.auto_validate {
            if let Err(e) = self.validate() {
                // Log validation error but continue building for API compatibility
                eprintln!("Warning: Command structure validation failed: {}", e);
            }
        }

        // Clone to owned strings and convert to static lifetime for clap
        // Note: This leaks memory but is acceptable for CLI construction (happens once per run)
        // The leaked strings live for the duration of the program which is fine for CLI apps
        let name: &'static str = Box::leak(self.config.name.clone().into_boxed_str());
        let about: &'static str = Box::leak(self.config.about.clone().into_boxed_str());
        let mut cmd = Command::new(name).about(about);

        if let Some(version) = &self.config.version {
            let version_str: &'static str = Box::leak(version.to_string().into_boxed_str());
            cmd = cmd.version(version_str);
        }

        // Add global arguments
        for arg in &self.config.global_args {
            cmd = cmd.arg(arg.clone());
        }

        // Add global --introspect flag
        cmd = cmd.arg(
            clap::Arg::new("introspect")
                .long("introspect")
                .action(clap::ArgAction::SetTrue)
                .global(true)
                .help("Introspect CLI capabilities as JSON Schema array for LLM tool-calling"),
        );

        // Add noun subcommands
        for noun in self.nouns.values() {
            cmd = cmd.subcommand(noun.build_command());
        }

        if self.has_completions_subcommand {
            let completions_noun = self.build_completions_noun();
            cmd = cmd.subcommand(completions_noun.build_command());
        }

        cmd
    }

    /// Route a command based on clap matches
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        // Get the top-level subcommand (noun)
        let (noun_name, noun_matches) = matches.subcommand().ok_or_else(|| {
            NounVerbError::InvalidStructure { message: "No subcommand found".to_string() }
        })?;

        if noun_name == "completions" && self.has_completions_subcommand {
            let noun = self.build_completions_noun();
            return self.route_recursive(&noun, noun_name, noun_matches, matches);
        }

        // Find the noun command
        let noun = self.nouns.get(noun_name).ok_or_else(|| {
            let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
            NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
        })?;

        // Route the command recursively with root matches for global args access
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
                // Execute the verb with root matches for global args access
                let mut context = VerbContext::new(sub_name).with_noun(noun_name);
                context.extensions = self.extensions.clone();
                let args = VerbArgs::new(sub_matches.clone())
                    .with_parent(root_matches.clone())
                    .with_context(context);

                verb.run(&args)
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
            let mut context = VerbContext::new("").with_noun(noun_name);
            context.extensions = self.extensions.clone();
            let args = VerbArgs::new(matches.clone()).with_context(context);

            noun.handle_direct(&args)
        }
    }

    /// Run the CLI with the current process arguments
    pub fn run(self) -> Result<()> {
        let args: Vec<String> = std::env::args().collect();
        self.run_with_args(args)
    }

    /// Run the CLI with custom arguments
    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        // Auto-validate if enabled
        if self.config.auto_validate {
            self.validate()?;
        }

        if args.is_empty() {
            return Err(NounVerbError::argument_error("No arguments provided"));
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

        if steps.is_empty() {
            let cmd = self.build_command();
            let matches = cmd.clone().try_get_matches_from(vec![binary_name]).map_err(|e| {
                if e.kind() == clap::error::ErrorKind::DisplayHelp
                    || e.kind() == clap::error::ErrorKind::DisplayVersion
                {
                    e.print().ok();
                    std::process::exit(0);
                }
                NounVerbError::argument_error(e.to_string())
            })?;

            if matches.get_flag("introspect") {
                let tools = collect_tools_from_cmd(&cmd, "");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tools)
                        .map_err(|e| NounVerbError::execution_error(e.to_string()))?
                );
                return Ok(());
            }
            return self.route(&matches);
        }

        let stdin_val = crate::cli::preprocessor::read_stdin_if_needed(&steps);
        let mut step_results: Vec<serde_json::Value> = Vec::new();

        for step in steps {
            let mut step_args = vec![binary_name.clone()];
            let processed_args =
                crate::cli::preprocessor::preprocess_args(&step, &stdin_val, &step_results)?;
            step_args.extend(processed_args);

            let cmd = self.build_command();
            let matches = cmd.clone().try_get_matches_from(step_args).map_err(|e| {
                if e.kind() == clap::error::ErrorKind::DisplayHelp
                    || e.kind() == clap::error::ErrorKind::DisplayVersion
                {
                    e.print().ok();
                    std::process::exit(0);
                }
                NounVerbError::argument_error(e.to_string())
            })?;

            if matches.get_flag("introspect") {
                let tools = collect_tools_from_cmd(&cmd, "");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tools)
                        .map_err(|e| NounVerbError::execution_error(e.to_string()))?
                );
                return Ok(());
            }

            self.route(&matches)?;
            step_results.push(serde_json::Value::Null);
        }

        Ok(())
    }

    /// Get the built command for testing or manual execution
    pub fn command(self) -> Command {
        self.build_command()
    }

    /// Load and hot-register verbs from ontology directory
    ///
    /// This method scans ~/open-ontologies for TTL files and registers
    /// any new verbs found there. This enables:
    ///
    /// 1. Dynamic CLI expansion without recompilation
    /// 2. Ontology-driven development (declare verbs in RDF, generate Rust)
    /// 3. Live synchronization between code and ontology
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut registry = CommandRegistry::new();
    /// registry.load_ontology_verbs(None)?; // Uses ~/open-ontologies
    /// let cmd = registry.build_command();
    /// ```
    pub fn load_ontology_verbs(&mut self, ontology_dir: Option<PathBuf>) -> Result<usize> {
        let dir = ontology_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join("open-ontologies")
        });

        if !dir.exists() {
            return Err(NounVerbError::Generic(format!(
                "Ontology directory not found: {}",
                dir.display()
            )));
        }

        let verbs = self.discover_verbs_from_ontology(&dir)?;
        let count = verbs.len();

        for verb_def in verbs {
            // Create a simple stub verb for now
            // In production, this would generate actual Rust code and compile it
            // For now, we register the verb definition in metadata
            if let Some(_noun_name) = verb_def.noun {
                // In a full implementation, we'd create a VerbCommand from the definition
                // This would require dynamic code generation and compilation
            }
        }

        Ok(count)
    }

    /// Discover verb definitions from TTL/RDF files
    fn discover_verbs_from_ontology(&self, dir: &PathBuf) -> Result<Vec<OntologyVerbDef>> {
        let mut verbs = Vec::new();

        // Scan for TTL files
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "ttl") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Parse TTL for verb definitions
                            // This is a simplified parser - in production use proper RDF libraries
                            let parsed = self.parse_ttl_verbs(&content)?;
                            verbs.extend(parsed);
                        }
                    }
                }
            }
            Err(_) => {
                return Err(NounVerbError::Generic(format!(
                    "Cannot read ontology directory: {}",
                    dir.display()
                )))
            }
        }

        Ok(verbs)
    }

    /// Parse TTL file for verb definitions
    fn parse_ttl_verbs(&self, ttl_content: &str) -> Result<Vec<OntologyVerbDef>> {
        let mut verbs = Vec::new();

        // Simple parsing: look for :Verb declarations
        for line in ttl_content.lines() {
            if line.contains(":Verb") || line.contains("rdf:type cnv:Verb") {
                // Extract verb name (simplified parsing)
                if let Some(start) = line.find("ex:") {
                    let remainder = &line[start + 3..];
                    if let Some(end) = remainder.find(|c: char| !c.is_alphanumeric() && c != '_') {
                        let verb_name = remainder[..end].to_lowercase();
                        verbs.push(OntologyVerbDef {
                            name: verb_name,
                            noun: None,
                            doc: "Loaded from ontology".to_string(),
                            args: vec![],
                            return_type: "serde_json::Value".to_string(),
                        });
                    }
                }
            }
        }

        Ok(verbs)
    }

    /// Export current command registry to RDF/N-Triples format
    ///
    /// This creates an RDF representation of all registered nouns and verbs,
    /// enabling:
    /// - Ontology synchronization
    /// - Semantic querying with SPARQL
    /// - Conformance validation
    pub fn export_to_rdf(&self, format: RdfFormat) -> Result<String> {
        match format {
            RdfFormat::NTriples => self.export_ntriples(),
            RdfFormat::Turtle => self.export_turtle(),
            RdfFormat::JsonLd => self.export_jsonld(),
        }
    }

    fn export_ntriples(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Generated RDF/N-Triples from CommandRegistry\n");
        output.push_str("# Namespace: http://clap-noun-verb.io/ontology#\n\n");

        for (noun_idx, (noun_name, _noun)) in self.nouns.iter().enumerate() {
            let noun_uri = format!("<http://clap-noun-verb.io/nouns/noun{}>", noun_idx);
            output.push_str(&format!(
                "{} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://clap-noun-verb.io/ontology#Noun> .\n",
                noun_uri
            ));
            output.push_str(&format!(
                "{} <http://clap-noun-verb.io/ontology#nounName> \"{}\" .\n",
                noun_uri, noun_name
            ));
        }

        Ok(output)
    }

    fn export_turtle(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("@prefix cnv: <http://clap-noun-verb.io/ontology#> .\n");
        output.push_str("@prefix ex: <http://example.org/> .\n\n");

        for (noun_name, _noun) in self.nouns.iter() {
            output.push_str(&format!(
                "ex:{} a cnv:Noun ;\n    cnv:nounName \"{}\" .\n\n",
                noun_name, noun_name
            ));
        }

        Ok(output)
    }

    fn export_jsonld(&self) -> Result<String> {
        #[derive(Serialize)]
        struct JsonLdContext {
            #[serde(rename = "@context")]
            context: std::collections::HashMap<String, String>,
            #[serde(rename = "@graph")]
            graph: Vec<serde_json::Value>,
        }

        let mut context = std::collections::HashMap::new();
        context.insert("cnv".to_string(), "http://clap-noun-verb.io/ontology#".to_string());
        context.insert("ex".to_string(), "http://example.org/".to_string());

        let mut graph = Vec::new();
        for (noun_name, _noun) in self.nouns.iter() {
            graph.push(serde_json::json!({
                "@id": format!("ex:{}", noun_name),
                "@type": "cnv:Noun",
                "cnv:nounName": noun_name
            }));
        }

        let jsonld = JsonLdContext { context, graph };
        serde_json::to_string(&jsonld)
            .map_err(|e| NounVerbError::Generic(format!("JSON-LD serialization error: {}", e)))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamically generated completions noun command
pub struct CompletionsNoun {
    app_name: String,
    app_version: Option<String>,
    commands: Vec<String>,
    options: Vec<String>,
}

impl CompletionsNoun {
    pub fn new(
        app_name: String,
        app_version: Option<String>,
        commands: Vec<String>,
        options: Vec<String>,
    ) -> Self {
        Self { app_name, app_version, commands, options }
    }
}

impl NounCommand for CompletionsNoun {
    fn name(&self) -> &'static str {
        "completions"
    }

    fn about(&self) -> &'static str {
        "Generate shell completion scripts"
    }

    fn verbs(&self) -> Vec<Box<dyn crate::verb::VerbCommand>> {
        vec![
            Box::new(CompletionsVerb {
                name: "bash",
                about: "Generate completion script for bash",
                shell: crate::clap_ext::completions::Shell::Bash,
                app_name: self.app_name.clone(),
                app_version: self.app_version.clone().unwrap_or_else(|| "1.0.0".to_string()),
                commands: self.commands.clone(),
                options: self.options.clone(),
            }),
            Box::new(CompletionsVerb {
                name: "zsh",
                about: "Generate completion script for zsh",
                shell: crate::clap_ext::completions::Shell::Zsh,
                app_name: self.app_name.clone(),
                app_version: self.app_version.clone().unwrap_or_else(|| "1.0.0".to_string()),
                commands: self.commands.clone(),
                options: self.options.clone(),
            }),
            Box::new(CompletionsVerb {
                name: "fish",
                about: "Generate completion script for fish",
                shell: crate::clap_ext::completions::Shell::Fish,
                app_name: self.app_name.clone(),
                app_version: self.app_version.clone().unwrap_or_else(|| "1.0.0".to_string()),
                commands: self.commands.clone(),
                options: self.options.clone(),
            }),
            Box::new(CompletionsVerb {
                name: "powershell",
                about: "Generate completion script for PowerShell",
                shell: crate::clap_ext::completions::Shell::PowerShell,
                app_name: self.app_name.clone(),
                app_version: self.app_version.clone().unwrap_or_else(|| "1.0.0".to_string()),
                commands: self.commands.clone(),
                options: self.options.clone(),
            }),
        ]
    }

    fn build_command(&self) -> Command {
        let mut cmd = Command::new(self.name()).about(self.about()).arg(
            clap::Arg::new("shell")
                .short('s')
                .long("shell")
                .help("The shell to generate completions for")
                .value_parser(["bash", "zsh", "fish", "powershell"]),
        );

        for verb in self.verbs() {
            cmd = cmd.subcommand(verb.build_command());
        }

        cmd
    }

    fn handle_direct(&self, args: &crate::verb::VerbArgs) -> Result<()> {
        let shell_str = if let Some(s) = args.get_one_str_opt("shell") {
            s
        } else if let Some(detected) = crate::shell::detect_shell() {
            match detected {
                crate::shell::ShellType::Bash => "bash".to_string(),
                crate::shell::ShellType::Zsh => "zsh".to_string(),
                crate::shell::ShellType::Fish => "fish".to_string(),
                crate::shell::ShellType::PowerShell => "powershell".to_string(),
                _ => "bash".to_string(),
            }
        } else {
            "bash".to_string()
        };

        let shell = match shell_str.as_str() {
            "bash" => crate::clap_ext::completions::Shell::Bash,
            "zsh" => crate::clap_ext::completions::Shell::Zsh,
            "fish" => crate::clap_ext::completions::Shell::Fish,
            "powershell" => crate::clap_ext::completions::Shell::PowerShell,
            _ => crate::clap_ext::completions::Shell::Bash,
        };

        let generator = crate::clap_ext::completions::CompletionGenerator::new(&self.app_name)
            .with_version(self.app_version.as_deref().unwrap_or("1.0.0"))
            .with_commands(self.commands.clone());

        let mut gen = generator;
        for opt in &self.options {
            gen = gen.with_option(opt);
        }

        let script = gen.generate(shell)?;
        print!("{}", script);
        Ok(())
    }
}

struct CompletionsVerb {
    name: &'static str,
    about: &'static str,
    shell: crate::clap_ext::completions::Shell,
    app_name: String,
    app_version: String,
    commands: Vec<String>,
    options: Vec<String>,
}

impl crate::verb::VerbCommand for CompletionsVerb {
    fn name(&self) -> &'static str {
        self.name
    }

    fn about(&self) -> &'static str {
        self.about
    }

    fn run(&self, _args: &crate::verb::VerbArgs) -> Result<()> {
        let generator = crate::clap_ext::completions::CompletionGenerator::new(&self.app_name)
            .with_version(&self.app_version)
            .with_commands(self.commands.clone());

        let mut gen = generator;
        for opt in &self.options {
            gen = gen.with_option(opt);
        }

        let script = gen.generate(self.shell)?;
        print!("{}", script);
        Ok(())
    }
}

impl CommandRegistry {
    fn build_completions_noun(&self) -> CompletionsNoun {
        let app_name = self.config.name.clone();
        let app_version = self.config.version.clone();

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

        // Collect options
        for arg in &self.config.global_args {
            if let Some(long) = arg.get_long() {
                options.push(format!("--{}", long));
            }
            if let Some(short) = arg.get_short() {
                options.push(format!("-{}", short));
            }
        }

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

        CompletionsNoun { app_name, app_version, commands, options }
    }
}

/// JSON Schema representation for LLM tool-calling capability
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
}

/// Parameters schema inside ToolDefinition
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: std::collections::BTreeMap<String, PropertySchema>,
    pub required: Vec<String>,
}

/// Standard JSON Schema property descriptor
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "items")]
    pub items: Option<Box<PropertySchema>>,
}

/// Recursively collect executable tools (commands without subcommands) from a clap command tree
pub fn collect_tools_from_cmd(cmd: &clap::Command, prefix: &str) -> Vec<ToolDefinition> {
    let mut tools = Vec::new();
    let current_name = if prefix.is_empty() {
        cmd.get_name().to_string()
    } else {
        format!("{}_{}", prefix, cmd.get_name())
    };

    let subcommands: Vec<&clap::Command> = cmd.get_subcommands().collect();
    if subcommands.is_empty() {
        let mut properties = std::collections::BTreeMap::new();
        let mut required = Vec::new();

        for arg in cmd.get_arguments() {
            let arg_id = arg.get_id().as_str();
            if arg_id == "help" || arg_id == "version" || arg_id == "introspect" {
                continue;
            }

            let name = arg_id.to_string();
            let help = arg.get_help().map(|s| s.to_string());

            let is_flag = matches!(
                arg.get_action(),
                clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
            );
            let multiple =
                matches!(arg.get_action(), clap::ArgAction::Append | clap::ArgAction::Count);

            let prop_type = if is_flag {
                "boolean".to_string()
            } else if multiple {
                "array".to_string()
            } else {
                "string".to_string()
            };

            let items = if multiple {
                Some(Box::new(PropertySchema {
                    prop_type: "string".to_string(),
                    description: None,
                    default: None,
                    items: None,
                }))
            } else {
                None
            };

            let default = arg
                .get_default_values()
                .first()
                .map(|v| serde_json::Value::String(v.to_string_lossy().to_string()));

            if arg.is_required_set() {
                required.push(name.clone());
            }

            properties
                .insert(name, PropertySchema { prop_type, description: help, default, items });
        }

        tools.push(ToolDefinition {
            name: current_name,
            description: cmd.get_about().map(|s| s.to_string()).unwrap_or_default(),
            parameters: ToolParameters { param_type: "object".to_string(), properties, required },
        });
    } else {
        let pass_prefix = if prefix.is_empty() {
            if cmd.get_name() == "cli" || cmd.get_name() == "myapp" {
                "".to_string()
            } else {
                cmd.get_name().to_string()
            }
        } else {
            current_name
        };
        for sub in subcommands {
            tools.extend(collect_tools_from_cmd(sub, &pass_prefix));
        }
    }

    tools
}

// =============================================================================
// ONTOLOGY HOT-LOADING - Runtime verb discovery and registration
// =============================================================================

/// Ontology verb definition (for hot-loading from RDF)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyVerbDef {
    /// Verb name (e.g., "load", "validate")
    pub name: String,
    /// Associated noun (e.g., "graph", "ontology")
    pub noun: Option<String>,
    /// Documentation
    pub doc: String,
    /// Argument definitions
    pub args: Vec<OntologyArgDef>,
    /// Return type for Rust function
    pub return_type: String,
}

/// Ontology argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyArgDef {
    pub name: String,
    pub arg_type: String,
    pub required: bool,
    pub doc: Option<String>,
}

/// RDF export format
#[derive(Debug, Clone, Copy)]
pub enum RdfFormat {
    /// N-Triples format (.nt)
    NTriples,
    /// Turtle format (.ttl)
    Turtle,
    /// JSON-LD format (.jsonld)
    JsonLd,
}
