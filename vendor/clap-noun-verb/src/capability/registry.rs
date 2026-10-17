// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Capability Registry - Storage and lookup of capability packages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capability package metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

impl CapabilityPackage {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
        }
    }

    /// Validate package metadata
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Package ID cannot be empty".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Package name cannot be empty".to_string());
        }
        if self.version.trim().is_empty() {
            return Err("Package version cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Global capability registry (simulated)
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    packages: HashMap<String, CapabilityPackage>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self { packages: HashMap::new() }
    }

    /// Add a package to the registry
    pub fn add_package(&mut self, pkg: CapabilityPackage) -> Result<(), String> {
        pkg.validate()?;
        self.packages.insert(pkg.id.clone(), pkg);
        Ok(())
    }

    /// Remove a package from the registry
    pub fn remove_package(&mut self, id: &str) -> Result<String, String> {
        self.packages
            .remove(id)
            .map(|pkg| pkg.id)
            .ok_or_else(|| format!("Package not found: {}", id))
    }

    /// Get all packages
    pub fn packages(&self) -> Vec<CapabilityPackage> {
        self.packages.values().cloned().collect()
    }

    /// Get package count
    pub fn len(&self) -> usize {
        self.packages.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    /// Check if package exists
    pub fn contains(&self, id: &str) -> bool {
        self.packages.contains_key(id)
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_creation() {
        let pkg = CapabilityPackage::new("pkg-001", "GraphUtils", "1.0.0", "Graph operations");
        assert_eq!(pkg.id, "pkg-001");
        assert_eq!(pkg.name, "GraphUtils");
        assert_eq!(pkg.version, "1.0.0");
    }

    #[test]
    fn test_package_validation() {
        let valid = CapabilityPackage::new("p1", "n", "v", "d");
        assert!(valid.validate().is_ok());

        let invalid = CapabilityPackage::new("", "name", "1.0.0", "desc");
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_registry_add_remove() {
        let mut reg = CapabilityRegistry::new();
        let pkg = CapabilityPackage::new("pkg-001", "TestPkg", "1.0.0", "Test");

        reg.add_package(pkg).unwrap();
        assert_eq!(reg.len(), 1);

        let removed = reg.remove_package("pkg-001").unwrap();
        assert_eq!(removed, "pkg-001");
        assert!(reg.is_empty());
    }
}
