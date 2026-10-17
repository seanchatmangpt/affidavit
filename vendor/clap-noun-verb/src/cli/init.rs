// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scaffolding and project initialization
//!
//! This module provides utilities for initializing a new clap-noun-verb project,
//! including generating default configuration files.

use crate::error::{NounVerbError, Result};
use std::path::Path;

/// Generate a default clap-nv.toml configuration file
///
/// This template provides examples of common configuration patterns.
/// If the file already exists, it will return an error unless `force` is true.
pub fn scaffold_config(force: bool) -> Result<()> {
    let path = Path::new("clap-nv.toml");

    if path.exists() && !force {
        return Err(NounVerbError::execution_error(
            "clap-nv.toml already exists. Use --force to overwrite.",
        ));
    }

    let content = r#"# clap-noun-verb configuration file
# This file provides default arguments for your CLI.
# Structured data is automatically flattened to command-line arguments.

# --- GLOBAL SETTINGS ---

# Top-level keys become standard arguments (--verbose, --host)
# verbose = true
# host = "localhost"
# port = 8080

# --- MODULE SETTINGS ---

# Nested tables translate to dotted arguments (--database.url)
# [database]
# url = "postgres://localhost/db"
# pool_size = 5

# --- AGENT SETTINGS ---

# [agent]
# model = "anthropic:claude-3-sonnet"
# temperature = 0.7

# --- ARRAY ARGUMENTS ---

# Arrays are expanded into multiple arguments (--tags api --tags prod)
# tags = ["api", "production"]
"#;

    std::fs::write(path, content).map_err(|e| {
        NounVerbError::execution_error(format!("Failed to write clap-nv.toml: {}", e))
    })?;

    println!("Successfully initialized clap-nv.toml");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scaffold_config() {
        let dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        // Act
        scaffold_config(false).expect("Should succeed");

        // Assert
        let path = dir.path().join("clap-nv.toml");
        assert!(path.exists());
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("clap-noun-verb configuration file"));

        std::env::set_current_dir(original_dir).unwrap();
    }
}
