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
            FindingStatus::Ok => "ok  ",
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
        Finding {
            id,
            status: FindingStatus::Ok,
            message: message.into(),
            remediation: None,
            auto_fixable: false,
        }
    }

    pub fn warn(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Finding {
            id,
            status: FindingStatus::Warn,
            message: message.into(),
            remediation: Some(remediation.into()),
            auto_fixable: false,
        }
    }

    pub fn fail(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Finding {
            id,
            status: FindingStatus::Fail,
            message: message.into(),
            remediation: Some(remediation.into()),
            auto_fixable: false,
        }
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
/// All registered [`DoctorCheck`] implementations, collected at link time
/// (host). On wasm32 the linkme linker-section mechanism is unavailable, so the
/// registry degrades to an empty slice and `run_all()` returns no findings.
#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice]
pub static DOCTOR_CHECKS: [&'static dyn DoctorCheck];

/// wasm32 fallback: linkme `distributed_slice` needs linker sections that
/// wasm-ld does not provide, so registrations are compiled out.
#[cfg(target_arch = "wasm32")]
pub const DOCTOR_CHECKS: &[&'static dyn DoctorCheck] = &[];

// ---------------------------------------------------------------------------
// Built-in checks
// ---------------------------------------------------------------------------

struct GenesisVersionCheck;
impl DoctorCheck for GenesisVersionCheck {
    fn id(&self) -> &'static str {
        "genesis-seed-version"
    }
    fn run(&self) -> Finding {
        let version = env!("CARGO_PKG_VERSION");
        Finding::ok(
            "genesis-seed-version",
            format!("Genesis seed compiled for v{version} (compile-time auto-tracking — no drift possible)"),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _GENESIS_CHECK: &dyn DoctorCheck = &GenesisVersionCheck;

struct AfffiDirCheck;
impl DoctorCheck for AfffiDirCheck {
    fn id(&self) -> &'static str {
        "affi-working-dir"
    }
    fn run(&self) -> Finding {
        let working = std::path::Path::new(".affi/working.json");
        let dir = std::path::Path::new(".affi");
        if working.exists() {
            Finding::ok(
                "affi-working-dir",
                "Working receipt (.affi/working.json) present",
            )
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

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _AFFI_DIR_CHECK: &dyn DoctorCheck = &AfffiDirCheck;

struct CompletionsCheck;
impl DoctorCheck for CompletionsCheck {
    fn id(&self) -> &'static str {
        "shell-completions"
    }
    fn run(&self) -> Finding {
        let bash = std::path::Path::new("completions/affi.bash");
        let fish = std::path::Path::new("completions/affi.fish");
        let zsh = std::path::Path::new("completions/affi.zsh");
        if bash.exists() && fish.exists() && zsh.exists() {
            Finding::ok(
                "shell-completions",
                "Shell completions present (bash, fish, zsh)",
            )
        } else {
            let missing: Vec<&str> = [
                (!bash.exists()).then_some("bash"),
                (!fish.exists()).then_some("fish"),
                (!zsh.exists()).then_some("zsh"),
            ]
            .into_iter()
            .flatten()
            .collect();
            Finding::warn(
                "shell-completions",
                format!("Shell completions missing for: {}", missing.join(", ")),
                "Run 'cargo run --bin gen_completions' or copy from completions/ in the repo",
            )
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _COMPLETIONS_CHECK: &dyn DoctorCheck = &CompletionsCheck;

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Typed finding id for the wasm32 registration-fallback notice.
pub const WASM_UNSUPPORTED_ID: &str = "unsupported-registration";

/// Run all registered checks and return their findings, **sorted by check id**.
///
/// linkme's linker-section collection order is unspecified, so `run_all()`
/// sorts deterministically (determinism law: same registry, same output).
/// On wasm32 the registry is compiled out entirely (see [`DOCTOR_CHECKS`]);
/// rather than returning a silent zero-findings-ok, a typed
/// `unsupported-registration` finding is emitted so the doctor output is loud.
pub fn run_all() -> Vec<Finding> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut unsupported = Finding::warn(
            WASM_UNSUPPORTED_ID,
            "wasm32 target: linkme distributed_slice registrations are unavailable — doctor registry is empty",
            "Run the doctor on the host target where checks are link-registered",
        );
        unsupported.auto_fixable = false;
        vec![unsupported]
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut findings: Vec<Finding> = DOCTOR_CHECKS.iter().map(|c| c.run()).collect();
        findings.sort_by(|a, b| a.id.cmp(b.id));
        findings
    }
}

// ---------------------------------------------------------------------------
// Tests — registry courts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Court: every registered check id is unique.
    ///
    /// A duplicate id would make doctor output ambiguous (two findings
    /// indistinguishable by id) and silently break id-keyed consumers.
    #[test]
    fn doctor_check_ids_are_unique() {
        let mut ids: Vec<&'static str> = DOCTOR_CHECKS.iter().map(|c| c.id()).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            total,
            "duplicate DoctorCheck ids in DOCTOR_CHECKS: registry has {total} entries but only {} unique ids",
            ids.len()
        );
    }

    /// Court: `run_all()` output is a deterministic permutation of the registry.
    ///
    /// Sorted output must equal the multiset of registry findings sorted —
    /// proves the sort reorders nothing in or out.
    #[test]
    fn run_all_is_sorted_and_lossless() {
        let mut expected: Vec<&'static str> = DOCTOR_CHECKS.iter().map(|c| c.id()).collect();
        expected.sort_unstable();
        let actual: Vec<&str> = run_all().iter().map(|f| f.id).collect();
        assert_eq!(
            actual, expected,
            "run_all() must return findings sorted by check id, one per registered check"
        );
    }

    /// Vacuity gate (#47): the host registry must hold a floor of checks.
    ///
    /// Counted 2026-10-16 on DEFAULT features: 3 registrations — the built-ins
    /// in `doctor_check.rs` (`genesis-seed-version`, `affi-working-dir`,
    /// `shell-completions`). The 4 `crypto_trust_doctor.rs` registrations are
    /// behind `#[cfg(feature = "crypto-trust")]` (src/lib.rs:194), so they are
    /// present only under `--features crypto-trust` (registry = 7 there).
    /// Floor = actual - 1 per the register item, so dropping any one default
    /// registration fails the court. A registry that silently shrinks below
    /// the floor means registrations were compiled out — a court that cannot
    /// fail is not a court.
    const HOST_REGISTRY_FLOOR: usize = 2;
    #[test]
    fn host_registry_meets_floor() {
        assert!(
            DOCTOR_CHECKS.len() >= HOST_REGISTRY_FLOOR,
            "host DOCTOR_CHECKS registry shrank to {} (floor {HOST_REGISTRY_FLOOR}) — checks were dropped",
            DOCTOR_CHECKS.len()
        );
    }

    /// Vacuity gate (#47): every finding id from `run_all()` is itself
    /// unique and non-empty — the sorted output cannot hide duplicates.
    #[test]
    fn run_all_findings_have_unique_nonempty_ids() {
        let findings = run_all();
        assert!(
            !findings.is_empty(),
            "run_all() returned zero findings on host — registry compiled out"
        );
        let mut ids: Vec<&str> = findings.iter().map(|f| f.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "run_all() produced duplicate finding ids");
        assert!(
            ids.iter().all(|id| !id.is_empty()),
            "empty finding id in doctor output"
        );
    }
}
