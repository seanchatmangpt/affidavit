// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Interactive REPL shell execution loop with autocomplete and history parsing
//! Gated behind the `repl` feature.

use crate::error::Result;
use crate::CommandRegistry;
use std::path::PathBuf;

/// Interactive REPL shell execution helper
pub struct Repl {
    registry: CommandRegistry,
    history_file: Option<PathBuf>,
}

impl Repl {
    /// Create a new REPL shell execution helper from the command registry
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry, history_file: None }
    }

    /// Configure a history file path for the REPL session
    pub fn with_history_file(mut self, path: PathBuf) -> Self {
        self.history_file = Some(path);
        self
    }

    /// Get the command registry configuration
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    /// Run the interactive REPL shell execution loop
    #[cfg(feature = "repl")]
    pub fn run(&self) -> Result<()> {
        let config_builder = rustyline::Config::builder();
        let config = config_builder
            .behavior(rustyline::config::Behavior::PreferTerm)
            .completion_type(rustyline::CompletionType::List)
            .build();

        let helper = ReplHelper { commands: self.registry.command_structure() };

        let mut rl = rustyline::Editor::with_config(config).map_err(|e| {
            crate::NounVerbError::execution_error(format!(
                "Failed to initialize REPL editor: {}",
                e
            ))
        })?;
        rl.set_helper(Some(helper));

        // Load history if file exists
        if let Some(ref path) = self.history_file {
            if path.exists() {
                if let Err(e) = rl.load_history(path) {
                    eprintln!("Warning: Failed to load command history: {}", e);
                }
            }
        }

        let app_name = self.registry.build_command().get_name().to_string();
        println!("Welcome to the {} interactive REPL shell.", app_name);
        println!("Type 'help' to see available commands, or 'exit' / 'quit' to exit.");

        loop {
            let prompt = format!("{}> ", app_name);
            let readline = rl.readline(&prompt);
            match readline {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Save line to history
                    if let Err(e) = rl.add_history_entry(trimmed) {
                        eprintln!("Warning: Failed to save line to history: {}", e);
                    }

                    if trimmed == "exit" || trimmed == "quit" {
                        break;
                    }

                    if trimmed == "help" {
                        let mut cmd = self.registry.build_command();
                        if let Err(e) = cmd.print_help() {
                            eprintln!("Error displaying help: {}", e);
                        }
                        continue;
                    }

                    // Parse shell words
                    let args = match split_shell_words(trimmed) {
                        Some(args) => args,
                        None => {
                            eprintln!("Error: Unmatched quote or escape character.");
                            continue;
                        }
                    };

                    // Reconstruct argv matching the binary execution pattern (args[0] is app name)
                    let mut full_args = vec![app_name.clone()];
                    full_args.extend(args);

                    let cmd = self.registry.build_command();
                    match cmd.try_get_matches_from(full_args) {
                        Ok(matches) => {
                            if let Err(e) = self.registry.route(&matches) {
                                eprintln!("Error executing command: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history file
        if let Some(ref path) = self.history_file {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).ok();
                }
            }
            if let Err(e) = rl.save_history(path) {
                eprintln!("Warning: Failed to save command history: {}", e);
            }
        }

        Ok(())
    }

    /// Run the interactive REPL shell execution loop (stub implementation when `repl` is disabled)
    #[cfg(not(feature = "repl"))]
    pub fn run(&self) -> Result<()> {
        Err(crate::NounVerbError::execution_error(
            "REPL feature is not enabled. Build with --features repl to enable it.",
        ))
    }
}

/// Helper for rustyline autocomplete
#[cfg(feature = "repl")]
struct ReplHelper {
    commands: std::collections::HashMap<String, Vec<String>>,
}

#[cfg(feature = "repl")]
impl rustyline::Helper for ReplHelper {}

#[cfg(feature = "repl")]
impl rustyline::completion::Completer for ReplHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let text_to_complete = &line[..pos];
        let start = text_to_complete.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
        let current_word = &text_to_complete[start..];

        let words: Vec<&str> = text_to_complete.split_whitespace().collect();
        let ends_with_space = text_to_complete.ends_with(char::is_whitespace);

        let mut suggestions = Vec::new();

        if words.is_empty() || (words.len() == 1 && current_word.is_empty() && ends_with_space) {
            // Suggest nouns + builtins
            for noun in self.commands.keys() {
                suggestions.push(noun.clone());
            }
            suggestions.push("exit".to_string());
            suggestions.push("help".to_string());
            suggestions.push("quit".to_string());
        } else if words.len() == 1 && !ends_with_space {
            // Completing the first word
            for noun in self.commands.keys() {
                if noun.starts_with(current_word) {
                    suggestions.push(noun.clone());
                }
            }
            for builtin in &["exit", "help", "quit"] {
                if builtin.starts_with(current_word) {
                    suggestions.push(builtin.to_string());
                }
            }
        } else {
            // Suggest verbs for the noun
            let noun = words[0];
            if let Some(verbs) = self.commands.get(noun) {
                if (words.len() == 1 && ends_with_space)
                    || (words.len() == 2 && current_word.is_empty() && ends_with_space)
                {
                    for verb in verbs {
                        suggestions.push(verb.clone());
                    }
                } else if words.len() == 2 && !ends_with_space {
                    for verb in verbs {
                        if verb.starts_with(current_word) {
                            suggestions.push(verb.clone());
                        }
                    }
                }
            }
        }

        let pairs = suggestions
            .into_iter()
            .map(|s| rustyline::completion::Pair { display: s.clone(), replacement: s })
            .collect();

        Ok((start, pairs))
    }
}

#[cfg(feature = "repl")]
impl rustyline::hint::Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

#[cfg(feature = "repl")]
impl rustyline::highlight::Highlighter for ReplHelper {}

#[cfg(feature = "repl")]
impl rustyline::validate::Validator for ReplHelper {}

/// Simple parser to split a shell line into arguments, respecting quotes and escapes
pub fn split_shell_words(s: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut escaped = false;

    for c in s.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' && !in_single_quotes {
            in_double_quotes = !in_double_quotes;
        } else if c == '\'' && !in_double_quotes {
            in_single_quotes = !in_single_quotes;
        } else if c.is_whitespace() && !in_double_quotes && !in_single_quotes {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if in_double_quotes || in_single_quotes || escaped {
        None
    } else {
        if !current.is_empty() {
            words.push(current);
        }
        Some(words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandRegistry;

    #[test]
    fn test_split_shell_words() {
        assert_eq!(split_shell_words(""), Some(vec![]));
        assert_eq!(
            split_shell_words("pack install"),
            Some(vec!["pack".to_string(), "install".to_string()])
        );
        assert_eq!(
            split_shell_words("pack install \"my package\""),
            Some(vec!["pack".to_string(), "install".to_string(), "my package".to_string()])
        );
        assert_eq!(
            split_shell_words("pack install 'my package'"),
            Some(vec!["pack".to_string(), "install".to_string(), "my package".to_string()])
        );
        assert_eq!(split_shell_words("pack install \"unclosed"), None);
    }

    #[allow(unused_variables)]
    #[test]
    fn test_repl_stub() {
        let registry = CommandRegistry::new();
        let repl = Repl::new(registry);
        let repl = repl.with_history_file(PathBuf::from("/tmp/history"));
        #[cfg(not(feature = "repl"))]
        {
            assert!(repl.run().is_err());
        }
    }

    #[cfg(feature = "repl")]
    #[test]
    fn test_repl_autocomplete() {
        let registry = CommandRegistry::new().name("test-app");
        let helper = ReplHelper { commands: registry.command_structure() };

        // When registry is empty, first word autocomplete lists builtins
        let line = "ex";
        let ctx_history = rustyline::history::DefaultHistory::new();
        let ctx = rustyline::Context::new(&ctx_history);
        let res = rustyline::completion::Completer::complete(&helper, line, line.len(), &ctx);
        assert!(res.is_ok());
        let (start, pairs) = res.unwrap();
        assert_eq!(start, 0);
        assert!(pairs.iter().any(|p| p.replacement == "exit"));
    }
}
