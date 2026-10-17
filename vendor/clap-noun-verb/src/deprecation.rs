// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deprecation tracking and migration helpers
//!
//! This module provides utilities for marking verbs and arguments as deprecated,
//! showing helpful migration messages to users.
//!
//! # Example
//!
//! ```rust,ignore
//! use clap_noun_verb::deprecation::{Deprecation, DeprecationType};
//!
//! let deprecation = Deprecation::new(DeprecationType::Verb)
//!     .since("3.6.0")
//!     .note("This command has been replaced with 'new-name'")
//!     .suggestion("Use 'myapp services new-name' instead");
//! ```

use std::fmt;

/// Type of deprecated entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeprecationType {
    /// Deprecated noun (command group)
    Noun,
    /// Deprecated verb (subcommand)
    Verb,
    /// Deprecated argument/flag
    Argument,
}

/// Information about a deprecated item with migration guidance
#[derive(Debug, Clone)]
pub struct Deprecation {
    /// Type of deprecated item
    pub item_type: DeprecationType,
    /// Version in which deprecation was introduced
    pub since: Option<String>,
    /// Removal version (when it will be deleted)
    pub removed_in: Option<String>,
    /// Additional context about deprecation
    pub note: Option<String>,
    /// Suggested alternative
    pub suggestion: Option<String>,
}

impl Deprecation {
    /// Create a new deprecation with a given type
    pub fn new(item_type: DeprecationType) -> Self {
        Self { item_type, since: None, removed_in: None, note: None, suggestion: None }
    }

    /// Set the version when deprecation was introduced
    pub fn since(mut self, version: impl Into<String>) -> Self {
        self.since = Some(version.into());
        self
    }

    /// Set the version when this will be removed
    pub fn removed_in(mut self, version: impl Into<String>) -> Self {
        self.removed_in = Some(version.into());
        self
    }

    /// Set additional context/reason for deprecation
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Set the suggested replacement
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Format deprecation as a warning message
    pub fn warning_message(&self, item_name: &str) -> String {
        let mut message = format!("⚠️  {} '{}' is deprecated", self.item_type, item_name);

        if let Some(since) = &self.since {
            message.push_str(&format!(" since v{}", since));
        }

        if let Some(removed) = &self.removed_in {
            message.push_str(&format!(" (will be removed in v{})", removed));
        }

        message.push('\n');

        if let Some(note) = &self.note {
            message.push_str(&format!("\n  {}\n", note));
        }

        if let Some(suggestion) = &self.suggestion {
            message.push_str(&format!("\n  💡 Suggestion: {}\n", suggestion));
        }

        message
    }

    /// Format deprecation as help text
    pub fn help_text(&self, item_name: &str) -> String {
        match (self.since.as_ref(), self.suggestion.as_ref()) {
            (Some(since), Some(suggestion)) => {
                format!("[DEPRECATED since v{}] {} → {}", since, item_name, suggestion)
            }
            (Some(since), None) => {
                format!("[DEPRECATED since v{}] {}", since, item_name)
            }
            (None, Some(suggestion)) => {
                format!("[DEPRECATED] {} → {}", item_name, suggestion)
            }
            (None, None) => format!("[DEPRECATED] {}", item_name),
        }
    }

    /// Check if this deprecation is ready for removal
    pub fn is_removable(&self, current_version: &str) -> bool {
        if let Some(removed) = &self.removed_in {
            ParsedVersion::parse(current_version) >= ParsedVersion::parse(removed)
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum PreReleaseIdentifier {
    Numeric(u64),
    AlphaNumeric(String),
}

impl Ord for PreReleaseIdentifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Numeric(n1), Self::Numeric(n2)) => n1.cmp(n2),
            (Self::AlphaNumeric(s1), Self::AlphaNumeric(s2)) => s1.cmp(s2),
            (Self::Numeric(_), Self::AlphaNumeric(_)) => std::cmp::Ordering::Less,
            (Self::AlphaNumeric(_), Self::Numeric(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for PreReleaseIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Vec<PreReleaseIdentifier>,
}

impl ParsedVersion {
    fn parse(v: &str) -> Self {
        let v = v.strip_prefix('v').unwrap_or(v);
        // Split by '+' to discard build metadata
        let version_part = v.split('+').next().unwrap_or(v);

        let mut parts = version_part.splitn(2, '-');
        let core = parts.next().unwrap_or("");
        let pre = parts.next();

        let mut core_parts = core.split('.');
        let major = core_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = core_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = core_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

        let mut pre_release = Vec::new();
        if let Some(pre_str) = pre {
            for id in pre_str.split('.') {
                if !id.is_empty() {
                    if id.chars().all(|c| c.is_ascii_digit()) {
                        let num = id.parse::<u64>().unwrap_or(0);
                        pre_release.push(PreReleaseIdentifier::Numeric(num));
                    } else {
                        pre_release.push(PreReleaseIdentifier::AlphaNumeric(id.to_string()));
                    }
                }
            }
        }

        Self { major, minor, patch, pre_release }
    }
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let major_cmp = self.major.cmp(&other.major);
        if major_cmp != std::cmp::Ordering::Equal {
            return major_cmp;
        }

        let minor_cmp = self.minor.cmp(&other.minor);
        if minor_cmp != std::cmp::Ordering::Equal {
            return minor_cmp;
        }

        let patch_cmp = self.patch.cmp(&other.patch);
        if patch_cmp != std::cmp::Ordering::Equal {
            return patch_cmp;
        }

        match (self.pre_release.is_empty(), other.pre_release.is_empty()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => {
                let mut self_iters = self.pre_release.iter();
                let mut other_iters = other.pre_release.iter();
                loop {
                    match (self_iters.next(), other_iters.next()) {
                        (Some(id1), Some(id2)) => {
                            let cmp = id1.cmp(id2);
                            if cmp != std::cmp::Ordering::Equal {
                                return cmp;
                            }
                        }
                        (Some(_), None) => return std::cmp::Ordering::Greater,
                        (None, Some(_)) => return std::cmp::Ordering::Less,
                        (None, None) => return std::cmp::Ordering::Equal,
                    }
                }
            }
        }
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DeprecationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noun => write!(f, "Noun"),
            Self::Verb => write!(f, "Verb"),
            Self::Argument => write!(f, "Argument"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deprecation_message() {
        let dep = Deprecation::new(DeprecationType::Verb)
            .since("3.5.0")
            .removed_in("4.0.0")
            .note("This verb has been renamed for clarity")
            .suggestion("Use 'new-verb' instead");

        let msg = dep.warning_message("old-verb");
        assert!(msg.contains("old-verb"));
        assert!(msg.contains("deprecated"));
        assert!(msg.contains("3.5.0"));
        assert!(msg.contains("4.0.0"));
        assert!(msg.contains("new-verb"));
    }

    #[test]
    fn test_help_text() {
        let dep = Deprecation::new(DeprecationType::Argument).since("3.6.0");

        let text = dep.help_text("--old-flag");
        assert!(text.contains("DEPRECATED"));
        assert!(text.contains("--old-flag"));
    }

    #[test]
    fn test_removable() {
        let dep = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0");

        assert!(!dep.is_removable("3.9.0"));
        assert!(dep.is_removable("4.0.0"));
        assert!(dep.is_removable("4.1.0"));
        assert!(dep.is_removable("10.0.0"));
        assert!(!dep.is_removable("3.10.0"));

        let dep2 = Deprecation::new(DeprecationType::Verb).removed_in("3.10.0");
        assert!(dep2.is_removable("3.10.0"));
        assert!(!dep2.is_removable("3.9.5"));
    }

    #[test]
    fn test_semver_pre_release_removable() {
        // Pre-release versions should not be removable if the removal version is release
        let dep = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0");
        assert!(!dep.is_removable("4.0.0-alpha"));
        assert!(!dep.is_removable("4.0.0-alpha.1"));
        assert!(!dep.is_removable("4.0.0-beta"));
        assert!(dep.is_removable("4.0.0"));
        assert!(dep.is_removable("4.0.1-alpha"));

        // Comparing pre-releases against pre-releases
        let dep_pre = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0-beta");
        assert!(!dep_pre.is_removable("4.0.0-alpha"));
        assert!(!dep_pre.is_removable("4.0.0-alpha.1"));
        assert!(dep_pre.is_removable("4.0.0-beta"));
        assert!(dep_pre.is_removable("4.0.0-beta.1"));
        assert!(dep_pre.is_removable("4.0.0"));

        // Numeric pre-release field comparison (2 < 11)
        let dep_num = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0-alpha.11");
        assert!(!dep_num.is_removable("4.0.0-alpha.2"));
        assert!(dep_num.is_removable("4.0.0-alpha.11"));
        assert!(dep_num.is_removable("4.0.0-alpha.12"));
    }

    #[test]
    fn test_semver_build_metadata_removable() {
        // Build metadata should be ignored for comparison
        let dep = Deprecation::new(DeprecationType::Verb).removed_in("4.0.0");
        assert!(dep.is_removable("4.0.0+build.123"));
        assert!(dep.is_removable("4.0.0+build.456"));
        assert!(!dep.is_removable("3.9.9+build.123"));
    }
}
