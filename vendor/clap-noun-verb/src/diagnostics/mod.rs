// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Diagnostics module - System health checks and doctor mode
//!
//! Provides health monitoring and diagnostic capabilities for CLI applications.

pub mod doctor;

pub use doctor::{DoctorOutput, HealthIssue};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_output() {
        let mut output = DoctorOutput::new(100, 5);
        assert!(output.healthy);

        output.add_issue("warning", "Low memory");
        assert!(output.healthy);

        output.add_issue("error", "Database unreachable");
        assert!(!output.healthy);
        assert_eq!(output.status, "unhealthy");
    }
}
