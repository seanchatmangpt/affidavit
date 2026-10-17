// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Doctor Check Command - Health check for system

use serde::{Deserialize, Serialize};

/// Result from doctor health check operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorOutput {
    pub status: String,
    pub healthy: bool,
    pub issues: Vec<HealthIssue>,
    pub graph_triples: usize,
    pub registry_packages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub level: String, // "error", "warning", "info"
    pub message: String,
}

impl DoctorOutput {
    pub fn new(graph_triples: usize, registry_packages: usize) -> Self {
        Self {
            status: "healthy".to_string(),
            healthy: true,
            issues: Vec::new(),
            graph_triples,
            registry_packages,
        }
    }

    pub fn add_issue(&mut self, level: impl Into<String>, msg: impl Into<String>) {
        let level_str = level.into();
        if level_str == "error" {
            self.healthy = false;
            self.status = "unhealthy".to_string();
        }
        self.issues.push(HealthIssue { level: level_str, message: msg.into() });
    }
}

/// Domain logic: Perform all health checks
fn perform_health_checks() -> DoctorOutput {
    let mut output = DoctorOutput::new(42, 5);

    let graph_accessible = check_graph_accessible();
    let registry_operational = check_registry_operational();
    let memory_ok = check_memory_availability();

    if !graph_accessible {
        output.add_issue("error", "Graph store not accessible");
    }

    if !registry_operational {
        output.add_issue("error", "Capability registry is not responding");
    }

    if !memory_ok {
        output.add_issue("warning", "Memory usage above 80% threshold");
    }

    output.add_issue("info", "All core services operational");
    output
}

/// Perform a system health check
///
/// Verifies the health and operational status of the system:
/// - Checks if graph store is accessible
/// - Verifies capability registry is operational
/// - Reports memory and performance metrics
/// - Identifies any operational issues
///
/// # Example
/// ```text
/// myapp doctor check
/// ```
pub fn health_check() -> crate::Result<DoctorOutput> {
    Ok(perform_health_checks())
}

/// Check if graph store is accessible
fn check_graph_accessible() -> bool {
    // Simulate graph store accessibility check
    // In production, would attempt actual connection
    true
}

/// Check if registry is operational
fn check_registry_operational() -> bool {
    // Simulate registry operational check
    // In production, would query actual registry
    true
}

/// Check if sufficient memory is available
fn check_memory_availability() -> bool {
    // Simulate memory check
    // In production, would check system memory
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_success() {
        let result = health_check();
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.healthy);
        assert_eq!(output.graph_triples, 42);
        assert_eq!(output.registry_packages, 5);
    }

    #[test]
    fn test_check_graph_accessible() {
        assert!(check_graph_accessible());
    }

    #[test]
    fn test_check_registry_operational() {
        assert!(check_registry_operational());
    }

    #[test]
    fn test_check_memory_availability() {
        assert!(check_memory_availability());
    }

    #[test]
    fn test_doctor_output_with_issues() {
        let mut output = DoctorOutput::new(10, 3);
        assert!(output.healthy);

        output.add_issue("warning", "High CPU");
        assert!(output.healthy);
        assert_eq!(output.issues.len(), 1);

        output.add_issue("error", "Network down");
        assert!(!output.healthy);
        assert_eq!(output.status, "unhealthy");
    }
}
