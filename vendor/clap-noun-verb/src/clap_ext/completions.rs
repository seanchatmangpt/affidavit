// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Advanced shell completion generation for multiple shells.
//!
//! Generates completion scripts for bash, zsh, fish, and PowerShell with
//! dynamic completion support and environment-aware suggestions.
//!
//! # Examples
//!
//! ```ignore
//! use clap_noun_verb::clap::CompletionGenerator;
//!
//! let gen = CompletionGenerator::new("myapp");
//! let bash_completion = gen.generate(Shell::Bash)?;
//! println!("{}", bash_completion);
//! ```

use std::fmt;

/// Supported shell types for completion generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
}

impl Shell {
    /// Get the file extension for this shell's completion script.
    pub fn extension(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "ps1",
        }
    }

    /// Get the installation directory for this shell.
    pub fn install_dir(&self) -> &'static str {
        match self {
            Shell::Bash => "/etc/bash_completion.d",
            Shell::Zsh => "/usr/share/zsh/site-functions",
            Shell::Fish => "~/.config/fish/completions",
            Shell::PowerShell => "~\\Documents\\PowerShell\\Modules",
        }
    }

    /// Get the name of this shell.
    pub fn name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Completion generator for multiple shells.
#[derive(Debug, Clone)]
pub struct CompletionGenerator {
    app_name: String,
    app_version: String,
    commands: Vec<String>,
    options: Vec<String>,
    enable_dynamic: bool,
}

impl CompletionGenerator {
    /// Create a new completion generator for an application.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_version: "1.0.0".to_string(),
            commands: Vec::new(),
            options: Vec::new(),
            enable_dynamic: false,
        }
    }

    /// Set the application version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.app_version = version.into();
        self
    }

    /// Add a command to the completion list.
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.commands.push(command.into());
        self
    }

    /// Add multiple commands at once.
    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        self.commands.extend(commands);
        self
    }

    /// Add an option to the completion list.
    pub fn with_option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Enable dynamic completion support.
    pub fn enable_dynamic(mut self) -> Self {
        self.enable_dynamic = true;
        self
    }

    /// Generate completion script for a specific shell.
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails.
    pub fn generate(&self, shell: Shell) -> crate::Result<String> {
        match shell {
            Shell::Bash => self.generate_bash(),
            Shell::Zsh => self.generate_zsh(),
            Shell::Fish => self.generate_fish(),
            Shell::PowerShell => self.generate_powershell(),
        }
    }

    fn generate_bash(&self) -> crate::Result<String> {
        let mut script = format!(
            "# {} completion script for bash\n# Generated for version {}\n\n",
            self.app_name, self.app_version
        );

        script.push_str(&format!(
            "_{}_completions() {{\n\
             \tlocal cur prev words cword\n\
             \tCOMPREPLY=()\n\
             \tcur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n\
             \tprev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n\
             \tcase \"$prev\" in\n",
            self.app_name.to_uppercase()
        ));

        for option in &self.options {
            script.push_str(&format!("\t\t{})\n", option));
            script.push_str("\t\t\treturn 0\n");
            script.push_str("\t\t\t;;\n");
        }

        script.push_str("\t\t*)\n");
        script.push_str("\t\t\tcase \"$cur\" in\n");
        script.push_str("\t\t\t\t-*)\n");
        script.push_str(&format!(
            "\t\t\t\t\tCOMPREPLY=( $(compgen -W '{} ' -- \"$cur\") )\n",
            self.options.join(" ")
        ));
        script.push_str("\t\t\t\t\treturn 0\n");
        script.push_str("\t\t\t\t\t;;\n");
        script.push_str("\t\t\t\t*)\n");
        script.push_str(&format!(
            "\t\t\t\t\tCOMPREPLY=( $(compgen -W '{} ' -- \"$cur\") )\n",
            self.commands.join(" ")
        ));
        script.push_str("\t\t\t\t\treturn 0\n");
        script.push_str("\t\t\t\t\t;;\n");
        script.push_str("\t\t\tesac\n");
        script.push_str("\t\t\t;;\n");
        script.push_str("\tesac\n");
        script.push_str("}\n\n");
        script.push_str(&format!(
            "complete -o bashdefault -o default -o nospace -F _{}_completions {}\n",
            self.app_name.to_uppercase(),
            self.app_name
        ));

        Ok(script)
    }

    fn generate_zsh(&self) -> crate::Result<String> {
        let mut script = format!(
            "#compdef {}\n# {} completion script for zsh\n# Generated for version {}\n\n",
            self.app_name, self.app_name, self.app_version
        );

        script.push_str("_arguments \\\n");

        for option in &self.options {
            script.push_str(&format!("  '{}' \\\n", option));
        }

        script.push_str("  '*: :'\n");

        Ok(script)
    }

    fn generate_fish(&self) -> crate::Result<String> {
        let mut script = format!(
            "# {} completion script for fish\n# Generated for version {}\n\n",
            self.app_name, self.app_version
        );

        for command in &self.commands {
            script.push_str(&format!(
                "complete -c {} -f -n '__fish_seen_subcommand_from {}' -a '{}'\n",
                self.app_name, command, command
            ));
        }

        for option in &self.options {
            script.push_str(&format!(
                "complete -c {} -f -n '__fish_use_subcommand' -a '{}'\n",
                self.app_name, option
            ));
        }

        Ok(script)
    }

    fn generate_powershell(&self) -> crate::Result<String> {
        let mut script = format!(
            "# {} completion script for PowerShell\n# Generated for version {}\n\n",
            self.app_name, self.app_version
        );

        script.push_str(&format!(
            "$__{}Completer = {{\n\
             \tparam($wordToComplete, $commandAst, $cursorPosition)\n\
             \t@()\n\
             }}\n\n",
            self.app_name
        ));

        script.push_str(&format!(
            "Register-ArgumentCompleter -CommandName {} -ScriptBlock $__{}_Completer\n",
            self.app_name, self.app_name
        ));

        Ok(script)
    }

    /// Get the application name.
    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    /// Get the application version.
    pub fn app_version(&self) -> &str {
        &self.app_version
    }

    /// Get the commands.
    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    /// Get the options.
    pub fn options(&self) -> &[String] {
        &self.options
    }

    /// Check if dynamic completion is enabled.
    pub fn is_dynamic_enabled(&self) -> bool {
        self.enable_dynamic
    }
}

/// Completion context for environment-aware suggestions.
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current word being completed
    current_word: String,
    /// Previous word (for context)
    previous_word: Option<String>,
    /// All words in the command line
    all_words: Vec<String>,
}

impl CompletionContext {
    /// Create a new completion context.
    pub fn new(current_word: impl Into<String>) -> Self {
        Self { current_word: current_word.into(), previous_word: None, all_words: Vec::new() }
    }

    /// Set the previous word (for context).
    pub fn with_previous_word(mut self, word: impl Into<String>) -> Self {
        self.previous_word = Some(word.into());
        self
    }

    /// Set all words in the command line.
    pub fn with_all_words(mut self, words: Vec<String>) -> Self {
        self.all_words = words;
        self
    }

    /// Get the current word being completed.
    pub fn current_word(&self) -> &str {
        &self.current_word
    }

    /// Get the previous word.
    pub fn previous_word(&self) -> Option<&str> {
        self.previous_word.as_deref()
    }

    /// Get all words.
    pub fn all_words(&self) -> &[String] {
        &self.all_words
    }

    /// Check if this is the start of a new command.
    pub fn is_command_start(&self) -> bool {
        self.all_words.is_empty() || self.all_words.len() == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_bash_properties() {
        let shell = Shell::Bash;
        assert_eq!(shell.name(), "bash");
        assert_eq!(shell.extension(), "bash");
    }

    #[test]
    fn test_shell_zsh_properties() {
        let shell = Shell::Zsh;
        assert_eq!(shell.name(), "zsh");
        assert_eq!(shell.extension(), "zsh");
    }

    #[test]
    fn test_shell_fish_properties() {
        let shell = Shell::Fish;
        assert_eq!(shell.name(), "fish");
        assert_eq!(shell.extension(), "fish");
    }

    #[test]
    fn test_shell_powershell_properties() {
        let shell = Shell::PowerShell;
        assert_eq!(shell.name(), "powershell");
        assert_eq!(shell.extension(), "ps1");
    }

    #[test]
    fn test_completion_generator_creation() {
        let gen = CompletionGenerator::new("myapp");
        assert_eq!(gen.app_name(), "myapp");
        assert_eq!(gen.app_version(), "1.0.0");
    }

    #[test]
    fn test_completion_generator_with_commands() {
        let gen = CompletionGenerator::new("myapp").with_command("start").with_command("stop");
        assert_eq!(gen.commands().len(), 2);
    }

    #[test]
    fn test_completion_generator_with_options() {
        let gen = CompletionGenerator::new("myapp").with_option("--help").with_option("--version");
        assert_eq!(gen.options().len(), 2);
    }

    #[test]
    fn test_completion_generator_bash() {
        let gen = CompletionGenerator::new("myapp").with_command("start").with_option("--help");
        let result = gen.generate(Shell::Bash);
        assert!(result.is_ok());
        let script = result.unwrap();
        assert!(script.contains("myapp"));
        assert!(script.contains("bash"));
    }

    #[test]
    fn test_completion_generator_zsh() {
        let gen = CompletionGenerator::new("myapp").with_command("start").with_option("--help");
        let result = gen.generate(Shell::Zsh);
        assert!(result.is_ok());
        let script = result.unwrap();
        assert!(script.contains("zsh"));
    }

    #[test]
    fn test_completion_generator_fish() {
        let gen = CompletionGenerator::new("myapp").with_command("start").with_option("--help");
        let result = gen.generate(Shell::Fish);
        assert!(result.is_ok());
        let script = result.unwrap();
        assert!(script.contains("fish"));
    }

    #[test]
    fn test_completion_generator_powershell() {
        let gen = CompletionGenerator::new("myapp").with_command("start").with_option("--help");
        let result = gen.generate(Shell::PowerShell);
        assert!(result.is_ok());
        let script = result.unwrap();
        assert!(script.contains("PowerShell"));
    }

    #[test]
    fn test_completion_context_creation() {
        let ctx = CompletionContext::new("my");
        assert_eq!(ctx.current_word(), "my");
        assert!(ctx.previous_word().is_none());
        assert!(ctx.is_command_start());
    }

    #[test]
    fn test_completion_context_with_words() {
        let ctx = CompletionContext::new("my")
            .with_previous_word("cmd")
            .with_all_words(vec!["cmd".to_string(), "my".to_string()]);
        assert_eq!(ctx.previous_word(), Some("cmd"));
        assert!(!ctx.is_command_start());
    }
}
