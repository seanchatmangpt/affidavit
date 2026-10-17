// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shell environment detection and utilities
//!
//! This module detects the user's shell and provides shell-specific utilities.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::shell::detect_shell;
//!
//! if let Some(shell) = detect_shell() {
//!     println!("Detected shell: {}", shell);
//! }
//! ```

use std::env;
use std::path::{Path, PathBuf};

/// Detected shell type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// PowerShell
    PowerShell,
    /// Elvish shell
    Elvish,
    /// Unknown shell
    Unknown,
}

impl ShellType {
    /// Get shell name as string
    pub fn as_str(&self) -> &str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::PowerShell => "powershell",
            ShellType::Elvish => "elvish",
            ShellType::Unknown => "unknown",
        }
    }

    /// Get shell config file path for the current user
    pub fn config_path(&self) -> Option<PathBuf> {
        let home = dirs_home()?;
        match self {
            ShellType::Bash => Some(home.join(".bashrc")),
            ShellType::Zsh => Some(home.join(".zshrc")),
            ShellType::Fish => Some(home.join(".config/fish/config.fish")),
            ShellType::PowerShell => {
                // PowerShell profile paths are dynamic, need to query $PROFILE
                None
            }
            ShellType::Elvish => Some(home.join(".elvish/rc.elv")),
            ShellType::Unknown => None,
        }
    }

    /// Whether this shell supports command substitution with $()
    pub fn supports_command_substitution(&self) -> bool {
        matches!(self, ShellType::Bash | ShellType::Zsh | ShellType::Fish | ShellType::Elvish)
    }

    /// Whether this shell requires special escaping
    pub fn requires_special_escaping(&self) -> bool {
        matches!(self, ShellType::PowerShell)
    }
}

/// Detect the user's shell from environment
///
/// # Returns
///
/// The detected `ShellType`, or `None` if shell could not be detected.
///
/// # Detection order
///
/// 1. Check `SHELL` environment variable (Unix-like)
/// 2. Check `ComSpec` environment variable (Windows)
/// 3. Check `PSModulePath` environment variable (PowerShell)
pub fn detect_shell() -> Option<ShellType> {
    // Try SHELL env var (Unix)
    if let Ok(shell) = env::var("SHELL") {
        if let Some(shell_type) = parse_shell_path(&shell) {
            return Some(shell_type);
        }
    }

    // Try Windows PowerShell
    if env::var("PSModulePath").is_ok() {
        return Some(ShellType::PowerShell);
    }

    // Try ComSpec (CMD or PowerShell on Windows)
    if let Ok(comspec) = env::var("ComSpec") {
        if comspec.contains("powershell") {
            return Some(ShellType::PowerShell);
        }
    }

    None
}

/// Parse shell type from shell path
fn parse_shell_path(path: &str) -> Option<ShellType> {
    let path = Path::new(path);
    let filename = path.file_name()?.to_string_lossy().to_lowercase();

    match filename.as_str() {
        "bash" => Some(ShellType::Bash),
        "zsh" => Some(ShellType::Zsh),
        "fish" => Some(ShellType::Fish),
        "powershell" | "pwsh" => Some(ShellType::PowerShell),
        "elvish" => Some(ShellType::Elvish),
        _ => None,
    }
}

/// Get home directory
fn dirs_home() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        env::var("HOME").ok().map(PathBuf::from)
    }
}

/// Get the shell completion directory for the given shell type
///
/// Returns the standard location where shell completion scripts should be installed
/// for the user's system. This varies by shell:
/// - Bash: `~/.local/share/bash-completion/completions` or `/etc/bash_completion.d`
/// - Zsh: `~/.zsh/completions`
/// - Fish: `~/.config/fish/completions`
/// - PowerShell: `$PROFILE` directory
/// - Elvish: `~/.elvish/lib`
///
/// # Arguments
///
/// * `shell` - The shell type to get the completion directory for
///
/// # Returns
///
/// `Some(PathBuf)` with the completion directory path, or `None` if it cannot be determined
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb::shell::{ShellType, get_completions_dir};
///
/// if let Some(completion_dir) = get_completions_dir(ShellType::Bash) {
///     println!("Install completions to: {}", completion_dir.display());
/// }
/// ```
pub fn get_completions_dir(shell: ShellType) -> Option<PathBuf> {
    let home = dirs_home()?;
    match shell {
        ShellType::Bash => {
            // Try to use .local/share first
            let local_dir = home.join(".local/share/bash-completion/completions");
            if local_dir.parent().map(|p| p.exists()).unwrap_or(false) {
                return Some(local_dir);
            }
            // Fall back to /etc/bash_completion.d (if writable)
            Some(PathBuf::from("/etc/bash_completion.d"))
        }
        ShellType::Zsh => Some(home.join(".zsh/completions")),
        ShellType::Fish => Some(home.join(".config/fish/completions")),
        ShellType::PowerShell => {
            let docs = env::var("PROFILE")
                .ok()
                .and_then(|p| Path::new(&p).parent().map(|p| p.to_path_buf()));
            docs
        }
        ShellType::Elvish => Some(home.join(".elvish/lib")),
        ShellType::Unknown => None,
    }
}

/// Check if the standard output is connected to an interactive terminal
///
/// This is useful for determining whether to display interactive prompts,
/// formatted output, or colors. When not interactive (e.g., piped to a file
/// or another command), you may want to output plain text instead.
///
/// # Returns
///
/// `true` if stdout is connected to an interactive terminal, `false` otherwise
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb::shell::is_interactive;
///
/// if is_interactive() {
///     println!("Interactive mode - showing formatted output");
/// } else {
///     println!("Non-interactive - showing plain output");
/// }
/// ```
pub fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdout)
}

/// Get the appropriate line ending for the given shell type
///
/// Different shells and operating systems use different line endings:
/// - PowerShell on Windows: `\r\n` (CRLF)
/// - Most other shells: `\n` (LF)
///
/// This is useful when generating shell scripts that will be executed
/// in different shell environments.
///
/// # Arguments
///
/// * `shell` - The target shell type
///
/// # Returns
///
/// A static string containing the appropriate line ending
///
/// # Example
///
/// ```rust,ignore
/// use clap_noun_verb::shell::{ShellType, line_ending};
///
/// let ending = line_ending(ShellType::PowerShell);
/// assert_eq!(ending, "\r\n");
/// ```
pub fn line_ending(shell: ShellType) -> &'static str {
    match shell {
        ShellType::PowerShell => "\r\n",
        _ => "\n",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shell_path() {
        assert_eq!(parse_shell_path("/bin/bash"), Some(ShellType::Bash));
        assert_eq!(parse_shell_path("/usr/bin/zsh"), Some(ShellType::Zsh));
        assert_eq!(parse_shell_path("/usr/local/bin/fish"), Some(ShellType::Fish));
    }

    #[test]
    fn test_shell_type_as_str() {
        assert_eq!(ShellType::Bash.as_str(), "bash");
        assert_eq!(ShellType::Zsh.as_str(), "zsh");
        assert_eq!(ShellType::Fish.as_str(), "fish");
    }

    #[test]
    fn test_shell_supports_command_substitution() {
        assert!(ShellType::Bash.supports_command_substitution());
        assert!(ShellType::Zsh.supports_command_substitution());
        assert!(ShellType::Fish.supports_command_substitution());
        assert!(!ShellType::PowerShell.supports_command_substitution());
    }
}
