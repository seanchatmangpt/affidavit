// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Capability Management module - Registry and packing operations
//!
//! Provides types and operations for managing capability packages in a registry.

pub mod impl_generated;
pub mod pack;
pub mod registry;

pub use impl_generated::{GeneratedHealthChecker, GeneratedRegistry, PackageMetadata};
pub use pack::{PackAddedOutput, PackRemovedOutput};
pub use registry::{CapabilityPackage, CapabilityRegistry};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_package() {
        let pkg = CapabilityPackage::new("pkg-001", "TestPackage", "1.0.0", "A test package");
        assert!(pkg.validate().is_ok());

        let invalid = CapabilityPackage::new("", "Test", "1.0.0", "Desc");
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_registry_operations() {
        let mut registry = CapabilityRegistry::new();
        let pkg = CapabilityPackage::new("pkg-001", "TestPkg", "1.0.0", "Test");

        registry.add_package(pkg.clone()).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("pkg-001"));

        let removed = registry.remove_package("pkg-001").unwrap();
        assert_eq!(removed, "pkg-001");
        assert!(registry.is_empty());
    }
}
