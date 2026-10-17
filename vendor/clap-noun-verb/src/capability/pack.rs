// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pack commands - Add and remove capability packages

use serde::{Deserialize, Serialize};

/// Result from pack add operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackAddedOutput {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: String,
}

impl PackAddedOutput {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            status: "added".to_string(),
        }
    }
}

/// Result from pack remove operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackRemovedOutput {
    pub removed_id: String,
    pub status: String,
    pub message: String,
}

impl PackRemovedOutput {
    pub fn new(removed_id: impl Into<String>) -> Self {
        Self {
            removed_id: removed_id.into(),
            status: "removed".to_string(),
            message: "Package successfully removed from registry".to_string(),
        }
    }
}

/// Validate semantic version format
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Domain logic: Generate package ID from name
fn generate_package_id(name: &str) -> String {
    format!("pkg-{}", name.to_lowercase().replace(' ', "-").chars().take(20).collect::<String>())
}

/// Domain logic: Validate and register package
fn register_package(name: &str, version: &str) -> crate::Result<(String, String, String)> {
    if name.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Package name cannot be empty".to_string(),
        ));
    }

    if version.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Version cannot be empty".to_string(),
        ));
    }

    if !is_valid_version(version) {
        return Err(crate::error::NounVerbError::execution_error(
            "Invalid version format. Use semantic versioning (e.g., 1.0.0)".to_string(),
        ));
    }

    let package_id = generate_package_id(name);
    Ok((package_id, name.to_string(), version.to_string()))
}

/// Check if a package exists in the registry (simulated)
fn package_exists(id: &str) -> bool {
    id.starts_with("pkg-")
}

/// Domain logic: Validate and remove package
fn unregister_package(id: &str) -> crate::Result<String> {
    if id.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error(
            "Package ID cannot be empty".to_string(),
        ));
    }

    if !id.starts_with("pkg-") {
        return Err(crate::error::NounVerbError::execution_error(
            "Invalid package ID format. Must start with 'pkg-'".to_string(),
        ));
    }

    if !package_exists(id) {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "Package not found in registry: {}",
            id
        )));
    }

    Ok(id.to_string())
}

/// Add a capability package to the registry
///
/// Registers a new capability package in the capability registry.
/// Packages represent reusable ontology components and semantic capabilities.
///
/// Uses the generated registry implementation for optimized package management.
///
/// # Arguments
/// * `name` - Package name (must be unique)
/// * `version` - Version string (e.g., "1.0.0")
///
/// # Example
/// ```text
/// myapp pack add GraphUtils 1.2.0
/// ```
pub fn add_package(name: String, version: String) -> crate::Result<PackAddedOutput> {
    let registry = super::impl_generated::GeneratedRegistry::new();
    let (pkg_id, pkg_name, pkg_version) = registry.register_package(&name, &version)?;
    Ok(PackAddedOutput::new(pkg_id, pkg_name, pkg_version))
}

/// Remove a capability package from the registry
///
/// Unregisters a capability package from the registry.
/// The package is removed along with all its associated metadata.
///
/// Uses the generated registry implementation for optimized package management.
///
/// # Arguments
/// * `id` - Package ID (e.g., "pkg-graphutils")
///
/// # Example
/// ```text
/// myapp pack remove pkg-graphutils
/// ```
pub fn remove_package(id: String) -> crate::Result<PackRemovedOutput> {
    let registry = super::impl_generated::GeneratedRegistry::new();
    let removed_id = registry.unregister_package(&id)?;
    Ok(PackRemovedOutput::new(removed_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_version() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("26.6.1"));
        assert!(!is_valid_version("1.0"));
    }

    #[test]
    fn test_generate_package_id() {
        assert_eq!(generate_package_id("TestPkg"), "pkg-testpkg");
        assert_eq!(generate_package_id("Graph Utils"), "pkg-graph-utils");
    }

    #[test]
    fn test_register_package_success() {
        let result = register_package("TestPkg", "1.0.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_package_empty_name() {
        let result = register_package("", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_added_output() {
        let output = PackAddedOutput::new("pkg-001", "GraphUtils", "2.1.0");
        assert_eq!(output.id, "pkg-001");
        assert_eq!(output.name, "GraphUtils");
        assert_eq!(output.version, "2.1.0");
        assert_eq!(output.status, "added");
    }

    #[test]
    fn test_pack_removed_output() {
        let output = PackRemovedOutput::new("pkg-001");
        assert_eq!(output.removed_id, "pkg-001");
        assert_eq!(output.status, "removed");
    }
}
