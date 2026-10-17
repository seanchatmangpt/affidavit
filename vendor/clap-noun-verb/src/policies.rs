// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Autonomic CI/CD Policies with suggest-mode recommendations
//!
//! This module provides policy engines that observe system signals and emit
//! recommendations without applying changes. All policies operate in "suggest only"
//! mode to ensure no side effects during CI/CD inspection phases.
//!
//! Each policy:
//! - Observes signals from the build environment
//! - Determines if conditions warrant attention
//! - Recommends actions without applying them
//! - Emits structured events for logging and telemetry

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Verdict on a policy condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyVerdict {
    /// No issues detected
    #[serde(rename = "pass")]
    Pass,
    /// Condition observed, recommendation available
    #[serde(rename = "suggest")]
    Suggest,
    /// Critical issue, strong recommendation
    #[serde(rename = "warn")]
    Warn,
}

/// Mode in which the policy operates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyMode {
    /// Observe and suggest only (no mutations)
    #[serde(rename = "suggest")]
    Suggest,
}

/// Represents a single policy observation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvent {
    /// Policy name (e.g., "target_pressure")
    pub policy: String,
    /// Timestamp when observed
    pub timestamp: String,
    /// Observed signal
    pub signal: String,
    /// Recommendation text
    pub recommendation: String,
    /// Policy verdict
    pub verdict: PolicyVerdict,
}

/// target_pressure policy - monitors disk usage and prune eligibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetPressurePolicy {
    /// Policy name
    pub name: String,
    /// Whether policy is enabled
    pub enabled: bool,
    /// Operation mode
    pub mode: PolicyMode,
    /// Target directory size in GB
    pub target_size_gb: f64,
    /// Maximum acceptable size in GB
    pub max_size_gb: f64,
    /// Days before automatic pruning is suggested
    pub prune_after_days: u32,
    /// Observed signals
    pub observed_signals: Vec<String>,
    /// Recommendation text
    pub recommendation: String,
    /// Current verdict
    pub verdict: PolicyVerdict,
    /// Last event emitted
    pub event_output: Option<PolicyEvent>,
}

impl TargetPressurePolicy {
    /// Create a new target_pressure policy
    pub fn new() -> Self {
        Self {
            name: "target_pressure".to_string(),
            enabled: true,
            mode: PolicyMode::Suggest,
            target_size_gb: 0.0,
            max_size_gb: 5.0,
            prune_after_days: 7,
            observed_signals: Vec::new(),
            recommendation: String::new(),
            verdict: PolicyVerdict::Pass,
            event_output: None,
        }
    }

    /// Set the observed target size in GB
    pub fn with_size(mut self, size_gb: f64) -> Self {
        self.target_size_gb = size_gb;
        self
    }

    /// Set the maximum acceptable size
    pub fn with_max_size(mut self, max_gb: f64) -> Self {
        self.max_size_gb = max_gb;
        self
    }

    /// Evaluate the policy and update verdict
    pub fn evaluate(&mut self) -> PolicyVerdict {
        self.observed_signals.clear();
        self.recommendation.clear();
        self.verdict = PolicyVerdict::Pass;

        if self.target_size_gb > self.max_size_gb {
            let excess = self.target_size_gb - self.max_size_gb;
            self.observed_signals.push(format!(
                "target size {} GB exceeds max {} GB by {} GB",
                self.target_size_gb, self.max_size_gb, excess
            ));
            self.recommendation =
                format!("Run: cargo cicd target prune --force\nEstimated recovery: {} GB", excess);
            self.verdict = PolicyVerdict::Suggest;
        }

        self.verdict
    }

    /// Emit an event for this policy's current state
    pub fn emit_event(&mut self) -> Option<PolicyEvent> {
        if self.verdict != PolicyVerdict::Pass {
            let event = PolicyEvent {
                policy: self.name.to_string(),
                timestamp: chrono::Local::now().to_rfc3339(),
                signal: self.observed_signals.join("; "),
                recommendation: self.recommendation.clone(),
                verdict: self.verdict,
            };
            self.event_output = Some(event.clone());
            Some(event)
        } else {
            None
        }
    }
}

impl Default for TargetPressurePolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// toolchain_mismatch policy - detects Rust toolchain inconsistencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainMismatchPolicy {
    /// Policy name
    pub name: String,
    /// Whether policy is enabled
    pub enabled: bool,
    /// Operation mode
    pub mode: PolicyMode,
    /// Expected toolchain from rust-toolchain.toml
    pub expected_toolchain: String,
    /// Actual toolchain detected
    pub actual_toolchain: String,
    /// Observed signals
    pub observed_signals: Vec<String>,
    /// Recommendation text
    pub recommendation: String,
    /// Current verdict
    pub verdict: PolicyVerdict,
    /// Last event emitted
    pub event_output: Option<PolicyEvent>,
}

impl ToolchainMismatchPolicy {
    /// Create a new toolchain_mismatch policy
    pub fn new() -> Self {
        Self {
            name: "toolchain_mismatch".to_string(),
            enabled: true,
            mode: PolicyMode::Suggest,
            expected_toolchain: String::new(),
            actual_toolchain: String::new(),
            observed_signals: Vec::new(),
            recommendation: String::new(),
            verdict: PolicyVerdict::Pass,
            event_output: None,
        }
    }

    /// Set the expected toolchain
    pub fn with_expected(mut self, toolchain: impl Into<String>) -> Self {
        self.expected_toolchain = toolchain.into();
        self
    }

    /// Set the actual detected toolchain
    pub fn with_actual(mut self, toolchain: impl Into<String>) -> Self {
        self.actual_toolchain = toolchain.into();
        self
    }

    /// Evaluate the policy and update verdict
    pub fn evaluate(&mut self) -> PolicyVerdict {
        self.observed_signals.clear();
        self.recommendation.clear();
        self.verdict = PolicyVerdict::Pass;

        if !self.expected_toolchain.is_empty() && !self.actual_toolchain.is_empty() {
            if self.expected_toolchain != self.actual_toolchain {
                self.observed_signals.push(format!(
                    "rust-toolchain.toml specifies {}, but Cargo.lock indicates {}",
                    self.expected_toolchain, self.actual_toolchain
                ));
                self.recommendation = format!(
                    "Unify toolchain to: {}\nOr update rust-toolchain.toml to: {}",
                    self.expected_toolchain, self.actual_toolchain
                );
                self.verdict = PolicyVerdict::Warn;
            }
        }

        self.verdict
    }

    /// Emit an event for this policy's current state
    pub fn emit_event(&mut self) -> Option<PolicyEvent> {
        if self.verdict != PolicyVerdict::Pass {
            let event = PolicyEvent {
                policy: self.name.to_string(),
                timestamp: chrono::Local::now().to_rfc3339(),
                signal: self.observed_signals.join("; "),
                recommendation: self.recommendation.clone(),
                verdict: self.verdict,
            };
            self.event_output = Some(event.clone());
            Some(event)
        } else {
            None
        }
    }
}

impl Default for ToolchainMismatchPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// trybuild_changed policy - detects changes to trybuild test fixtures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrybuildChangedPolicy {
    /// Policy name
    pub name: String,
    /// Whether policy is enabled
    pub enabled: bool,
    /// Operation mode
    pub mode: PolicyMode,
    /// Path to trybuild fixtures
    pub trybuild_path: PathBuf,
    /// Changed fixture files detected
    pub changed_fixtures: Vec<String>,
    /// Observed signals
    pub observed_signals: Vec<String>,
    /// Recommendation text
    pub recommendation: String,
    /// Current verdict
    pub verdict: PolicyVerdict,
    /// Last event emitted
    pub event_output: Option<PolicyEvent>,
}

impl TrybuildChangedPolicy {
    /// Create a new trybuild_changed policy
    pub fn new() -> Self {
        Self {
            name: "trybuild_changed".to_string(),
            enabled: true,
            mode: PolicyMode::Suggest,
            trybuild_path: PathBuf::from("tests/trybuild"),
            changed_fixtures: Vec::new(),
            observed_signals: Vec::new(),
            recommendation: String::new(),
            verdict: PolicyVerdict::Pass,
            event_output: None,
        }
    }

    /// Set the path to trybuild fixtures
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.trybuild_path = path.into();
        self
    }

    /// Add a changed fixture
    pub fn add_changed_fixture(&mut self, fixture: impl Into<String>) {
        self.changed_fixtures.push(fixture.into());
    }

    /// Evaluate the policy and update verdict
    pub fn evaluate(&mut self) -> PolicyVerdict {
        self.observed_signals.clear();
        self.recommendation.clear();
        self.verdict = PolicyVerdict::Pass;

        if !self.changed_fixtures.is_empty() {
            let files = self.changed_fixtures.join(", ");
            self.observed_signals.push(format!("Changed trybuild fixtures detected: {}", files));
            self.recommendation =
                "Run: cargo cicd trybuild changed\nBefore running all fixtures".to_string();
            self.verdict = PolicyVerdict::Suggest;
        }

        self.verdict
    }

    /// Emit an event for this policy's current state
    pub fn emit_event(&mut self) -> Option<PolicyEvent> {
        if self.verdict != PolicyVerdict::Pass {
            let event = PolicyEvent {
                policy: self.name.to_string(),
                timestamp: chrono::Local::now().to_rfc3339(),
                signal: self.observed_signals.join("; "),
                recommendation: self.recommendation.clone(),
                verdict: self.verdict,
            };
            self.event_output = Some(event.clone());
            Some(event)
        } else {
            None
        }
    }
}

impl Default for TrybuildChangedPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// git_phase_dirty policy - enforces clean git tree during CI/CD phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitPhaseDirtyPolicy {
    /// Policy name
    pub name: String,
    /// Whether policy is enabled
    pub enabled: bool,
    /// Operation mode
    pub mode: PolicyMode,
    /// Whether clean tree is required
    pub require_clean_tree: bool,
    /// Uncommitted changes exist
    pub has_uncommitted: bool,
    /// Untracked files exist
    pub has_untracked: bool,
    /// Observed signals
    pub observed_signals: Vec<String>,
    /// Recommendation text
    pub recommendation: String,
    /// Current verdict
    pub verdict: PolicyVerdict,
    /// Last event emitted
    pub event_output: Option<PolicyEvent>,
}

impl GitPhaseDirtyPolicy {
    /// Create a new git_phase_dirty policy
    pub fn new() -> Self {
        Self {
            name: "git_phase_dirty".to_string(),
            enabled: true,
            mode: PolicyMode::Suggest,
            require_clean_tree: true,
            has_uncommitted: false,
            has_untracked: false,
            observed_signals: Vec::new(),
            recommendation: String::new(),
            verdict: PolicyVerdict::Pass,
            event_output: None,
        }
    }

    /// Set whether clean tree is required
    pub fn require_clean(mut self, required: bool) -> Self {
        self.require_clean_tree = required;
        self
    }

    /// Mark that uncommitted changes exist
    pub fn with_uncommitted(mut self, has_changes: bool) -> Self {
        self.has_uncommitted = has_changes;
        self
    }

    /// Mark that untracked files exist
    pub fn with_untracked(mut self, has_files: bool) -> Self {
        self.has_untracked = has_files;
        self
    }

    /// Evaluate the policy and update verdict
    pub fn evaluate(&mut self) -> PolicyVerdict {
        self.observed_signals.clear();
        self.recommendation.clear();
        self.verdict = PolicyVerdict::Pass;

        if self.require_clean_tree {
            if self.has_uncommitted {
                self.observed_signals
                    .push("Uncommitted changes detected in working tree".to_string());
                self.verdict = PolicyVerdict::Warn;
            }
            if self.has_untracked {
                self.observed_signals.push("Untracked files detected".to_string());
                self.verdict = PolicyVerdict::Warn;
            }

            if self.verdict == PolicyVerdict::Warn {
                self.recommendation =
                    "Clean tree before closing phase\nOr disable require_clean_tree in cicd.toml"
                        .to_string();
            }
        }

        self.verdict
    }

    /// Emit an event for this policy's current state
    pub fn emit_event(&mut self) -> Option<PolicyEvent> {
        if self.verdict != PolicyVerdict::Pass {
            let event = PolicyEvent {
                policy: self.name.to_string(),
                timestamp: chrono::Local::now().to_rfc3339(),
                signal: self.observed_signals.join("; "),
                recommendation: self.recommendation.clone(),
                verdict: self.verdict,
            };
            self.event_output = Some(event.clone());
            Some(event)
        } else {
            None
        }
    }
}

impl Default for GitPhaseDirtyPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// PolicySet - container for all autonomic policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    /// target_pressure policy
    pub target_pressure: TargetPressurePolicy,
    /// toolchain_mismatch policy
    pub toolchain_mismatch: ToolchainMismatchPolicy,
    /// trybuild_changed policy
    pub trybuild_changed: TrybuildChangedPolicy,
    /// git_phase_dirty policy
    pub git_phase_dirty: GitPhaseDirtyPolicy,
}

impl PolicySet {
    /// Create a new policy set with all policies enabled
    pub fn new() -> Self {
        Self {
            target_pressure: TargetPressurePolicy::new(),
            toolchain_mismatch: ToolchainMismatchPolicy::new(),
            trybuild_changed: TrybuildChangedPolicy::new(),
            git_phase_dirty: GitPhaseDirtyPolicy::new(),
        }
    }

    /// Evaluate all policies
    pub fn evaluate_all(&mut self) -> Vec<PolicyVerdict> {
        vec![
            self.target_pressure.evaluate(),
            self.toolchain_mismatch.evaluate(),
            self.trybuild_changed.evaluate(),
            self.git_phase_dirty.evaluate(),
        ]
    }

    /// Emit all policy events
    pub fn emit_all_events(&mut self) -> Vec<PolicyEvent> {
        let mut events = Vec::new();

        if let Some(event) = self.target_pressure.emit_event() {
            events.push(event);
        }
        if let Some(event) = self.toolchain_mismatch.emit_event() {
            events.push(event);
        }
        if let Some(event) = self.trybuild_changed.emit_event() {
            events.push(event);
        }
        if let Some(event) = self.git_phase_dirty.emit_event() {
            events.push(event);
        }

        events
    }
}

impl Default for PolicySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_pressure_policy_pass() {
        let mut policy = TargetPressurePolicy::new().with_size(2.0).with_max_size(5.0);

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Pass);
        assert_eq!(policy.observed_signals.len(), 0);
    }

    #[test]
    fn test_target_pressure_policy_suggest() {
        let mut policy = TargetPressurePolicy::new().with_size(6.0).with_max_size(5.0);

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Suggest);
        assert_eq!(policy.observed_signals.len(), 1);
        assert!(policy.recommendation.contains("cargo cicd target prune"));
    }

    #[test]
    fn test_toolchain_mismatch_policy_pass() {
        let mut policy =
            ToolchainMismatchPolicy::new().with_expected("nightly").with_actual("nightly");

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn test_toolchain_mismatch_policy_warn() {
        let mut policy =
            ToolchainMismatchPolicy::new().with_expected("nightly").with_actual("stable");

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Warn);
        assert_eq!(policy.observed_signals.len(), 1);
        assert!(policy.recommendation.contains("Unify toolchain"));
    }

    #[test]
    fn test_trybuild_changed_policy_pass() {
        let mut policy = TrybuildChangedPolicy::new();

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn test_trybuild_changed_policy_suggest() {
        let mut policy = TrybuildChangedPolicy::new();
        policy.add_changed_fixture("tests/trybuild/fail1.rs");

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Suggest);
        assert!(policy.recommendation.contains("cargo cicd trybuild changed"));
    }

    #[test]
    fn test_git_phase_dirty_policy_pass() {
        let mut policy = GitPhaseDirtyPolicy::new()
            .require_clean(true)
            .with_uncommitted(false)
            .with_untracked(false);

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Pass);
    }

    #[test]
    fn test_git_phase_dirty_policy_uncommitted() {
        let mut policy = GitPhaseDirtyPolicy::new().require_clean(true).with_uncommitted(true);

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Warn);
        assert!(policy.recommendation.contains("Clean tree before closing phase"));
    }

    #[test]
    fn test_git_phase_dirty_policy_untracked() {
        let mut policy = GitPhaseDirtyPolicy::new().require_clean(true).with_untracked(true);

        let verdict = policy.evaluate();
        assert_eq!(verdict, PolicyVerdict::Warn);
    }

    #[test]
    fn test_policy_event_emission() {
        let mut policy = TargetPressurePolicy::new().with_size(6.0).with_max_size(5.0);

        policy.evaluate();
        let event = policy.emit_event();

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.policy, "target_pressure");
        assert_eq!(event.verdict, PolicyVerdict::Suggest);
    }

    #[test]
    fn test_policy_set_evaluate() {
        let mut set = PolicySet::new();
        set.target_pressure = TargetPressurePolicy::new().with_size(6.0).with_max_size(5.0);

        let verdicts = set.evaluate_all();
        assert_eq!(verdicts.len(), 4);
        assert_eq!(verdicts[0], PolicyVerdict::Suggest);
    }
}
