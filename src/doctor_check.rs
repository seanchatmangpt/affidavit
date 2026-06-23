//! `DoctorCheck` framework — linkme-based registry of environment health checks.
// linkme uses #[link_section] internally (required for distributed_slice).
#![allow(unsafe_code)]
//!
//! Each check is a zero-size type implementing [`DoctorCheck`] that is registered
//! into the [`DOCTOR_CHECKS`] distributed slice at link time.  The `affi doctor`
//! command iterates the slice and collects [`Finding`]s.
//!
//! # Adding a check
//!
//! ```rust,ignore
//! use linkme::distributed_slice;
//! use affidavit::doctor_check::{DoctorCheck, Finding, FindingStatus, DOCTOR_CHECKS};
//!
//! struct MyCheck;
//! impl DoctorCheck for MyCheck {
//!     fn id(&self) -> &'static str { "my-check" }
//!     fn run(&self) -> Finding {
//!         Finding::ok("my-check", "all good")
//!     }
//! }
//!
//! #[distributed_slice(DOCTOR_CHECKS)]
//! static MY_CHECK: &dyn DoctorCheck = &MyCheck;
//! ```

use linkme::distributed_slice;

// ---------------------------------------------------------------------------
// Finding
// ---------------------------------------------------------------------------

/// Severity of a doctor finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingStatus {
    /// Check passed — no action required.
    Ok,
    /// Non-fatal — the system works but something could be improved.
    Warn,
    /// Fatal — will cause operational failures if not addressed.
    Fail,
}

impl FindingStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FindingStatus::Ok   => "ok  ",
            FindingStatus::Warn => "warn",
            FindingStatus::Fail => "FAIL",
        }
    }
}

/// Result of a single doctor check.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Stable identifier for the check (e.g. `"genesis-seed"`).
    pub id: &'static str,
    /// Outcome of the check.
    pub status: FindingStatus,
    /// Human-readable description of what was found.
    pub message: String,
    /// Optional remediation step shown when status is Warn or Fail.
    pub remediation: Option<String>,
    /// True if `affi doctor --fix` can apply this remediation automatically.
    pub auto_fixable: bool,
}

impl Finding {
    pub fn ok(id: &'static str, message: impl Into<String>) -> Self {
        Finding { id, status: FindingStatus::Ok, message: message.into(), remediation: None, auto_fixable: false }
    }

    pub fn warn(id: &'static str, message: impl Into<String>, remediation: impl Into<String>) -> Self {
        Finding { id, status: FindingStatus::Warn, message: message.into(), remediation: Some(remediation.into()), auto_fixable: false }
    }

    pub fn fail(id: &'static str, message: impl Into<String>, remediation: impl Into<String>) -> Self {
        Finding { id, status: FindingStatus::Fail, message: message.into(), remediation: Some(remediation.into()), auto_fixable: false }
    }

    pub fn auto_fixable(mut self) -> Self {
        self.auto_fixable = true;
        self
    }
}

// ---------------------------------------------------------------------------
// DoctorCheck trait
// ---------------------------------------------------------------------------

/// A single decidable environment health check.
///
/// Implement this trait and register your struct in [`DOCTOR_CHECKS`] via
/// `#[distributed_slice(DOCTOR_CHECKS)]` to have it run automatically when
/// `affi doctor` is invoked.
pub trait DoctorCheck: Send + Sync + 'static {
    /// Short stable identifier (e.g. `"genesis-seed"`).  Must be unique across
    /// all registered checks.
    fn id(&self) -> &'static str;

    /// Execute the check and return a [`Finding`].
    fn run(&self) -> Finding;
}

// ---------------------------------------------------------------------------
// Distributed slice — the check registry
// ---------------------------------------------------------------------------

/// All registered [`DoctorCheck`] implementations, collected at link time.
///
/// Iterate this slice to run every check in the program:
///
/// ```rust,ignore
/// use affidavit::doctor_check::DOCTOR_CHECKS;
///
/// let findings: Vec<_> = DOCTOR_CHECKS.iter().map(|c| c.run()).collect();
/// ```
#[distributed_slice]
pub static DOCTOR_CHECKS: [&'static dyn DoctorCheck];

// ---------------------------------------------------------------------------
// Built-in checks
// ---------------------------------------------------------------------------

struct GenesisVersionCheck;
impl DoctorCheck for GenesisVersionCheck {
    fn id(&self) -> &'static str { "genesis-seed-version" }
    fn run(&self) -> Finding {
        let version = env!("CARGO_PKG_VERSION");
        Finding::ok(
            "genesis-seed-version",
            format!("Genesis seed compiled for v{version} (compile-time auto-tracking — no drift possible)"),
        )
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _GENESIS_CHECK: &dyn DoctorCheck = &GenesisVersionCheck;

struct AfffiDirCheck;
impl DoctorCheck for AfffiDirCheck {
    fn id(&self) -> &'static str { "affi-working-dir" }
    fn run(&self) -> Finding {
        let working = std::path::Path::new(".affi/working.json");
        let dir     = std::path::Path::new(".affi");
        if working.exists() {
            Finding::ok("affi-working-dir", "Working receipt (.affi/working.json) present")
        } else if dir.exists() {
            Finding::warn(
                "affi-working-dir",
                ".affi/ exists but no working.json — no receipt in progress",
                "Run 'affi emit --type <event_type> --object <id:type>' to start a chain",
            )
        } else {
            Finding::warn(
                "affi-working-dir",
                "No .affi/ directory — not inside a receipt workspace",
                "Run 'affi emit' to initialise the workspace",
            )
        }
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _AFFI_DIR_CHECK: &dyn DoctorCheck = &AfffiDirCheck;

struct CompletionsCheck;
impl DoctorCheck for CompletionsCheck {
    fn id(&self) -> &'static str { "shell-completions" }
    fn run(&self) -> Finding {
        let bash = std::path::Path::new("completions/affi.bash");
        let fish = std::path::Path::new("completions/affi.fish");
        let zsh  = std::path::Path::new("completions/affi.zsh");
        if bash.exists() && fish.exists() && zsh.exists() {
            Finding::ok("shell-completions", "Shell completions present (bash, fish, zsh)")
        } else {
            let missing: Vec<&str> = [
                (!bash.exists()).then_some("bash"),
                (!fish.exists()).then_some("fish"),
                (!zsh.exists()).then_some("zsh"),
            ].into_iter().flatten().collect();
            Finding::warn(
                "shell-completions",
                format!("Shell completions missing for: {}", missing.join(", ")),
                "Run 'cargo run --bin gen_completions' or copy from completions/ in the repo",
            )
        }
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _COMPLETIONS_CHECK: &dyn DoctorCheck = &CompletionsCheck;

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Run all registered checks and return their findings.
pub fn run_all() -> Vec<Finding> {
    DOCTOR_CHECKS.iter().map(|c| c.run()).collect()
}
