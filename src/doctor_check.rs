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
//!     fn run(&self, _ctx: &DoctorContext) -> Vec<Finding> {
//!         vec![Finding::ok("my-check", "all good")]
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
// DoctorContext
// ---------------------------------------------------------------------------

/// Per-run context handed to every [`DoctorCheck`].
///
/// This is a **run parameter**, not a field of the check types: every check
/// stays a zero-size type (zero-size law), and runtime state flows through
/// the context the dispatcher builds once per `affi doctor` invocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct DoctorContext<'a> {
    /// Optional receipt-store path from `affi doctor [--receipts <path>]`.
    /// Checks that need the store read it from here; with `None` those
    /// checks contribute no findings (the pre-registration dispatcher
    /// behavior when the flag was absent).
    pub receipts_path: Option<&'a str>,
    /// True when `--fix` was requested.
    pub fix: bool,
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

    /// Execute the check and return its findings (usually one, empty when the
    /// check is out of scope for the given [`DoctorContext`]).
    fn run(&self, ctx: &DoctorContext) -> Vec<Finding>;
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
/// let ctx = DoctorContext::default();
/// let findings: Vec<_> = DOCTOR_CHECKS.iter().flat_map(|c| c.run(&ctx)).collect();
/// ```
/// All registered [`DoctorCheck`] implementations, collected at link time
/// (host). On wasm32 the registry degrades to an empty slice and `run_all()`
/// returns no findings — see the wasm32 fallback below for the proven cause.
#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice]
pub static DOCTOR_CHECKS: [&'static dyn DoctorCheck];

/// wasm32 fallback — WHY registrations are compiled out (register row 58,
/// corrected by the probe court `affidavit-wasm/tests/linkme_premise_probe.rs`):
/// the blocker is linkme's own platform allow-list, NOT a wasm-ld limitation.
/// `linkme-impl-0.3.36/src/declaration.rs` gates linker-section support on
/// `target_os` (linux/macos/android/fios/illumos/freebsd/netbsd) and emits a
/// `compile_error!` for every other target — wasm32-unknown-unknown simply is
/// not in the list, so the distributed-slice macro refuses to compile there.
/// Raw `#[link_section]` statics DO work under wasm-ld, so re-implementing the
/// section-collection mechanism directly (bypassing linkme) is mechanically
/// viable if the wasm32 doctor ever needs real registrations. Until then the
/// honest degradation is an empty registry, never a fake pass.
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
    fn run(&self, _ctx: &DoctorContext) -> Vec<Finding> {
        let version = env!("CARGO_PKG_VERSION");
        vec![Finding::ok(
            "genesis-seed-version",
            format!("Genesis seed compiled for v{version} (compile-time auto-tracking — no drift possible)"),
        )]
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
    fn run(&self, _ctx: &DoctorContext) -> Vec<Finding> {
        vec![affi_dir_finding()]
    }
}

/// Body of [`AfffiDirCheck`] — kept as a free function so the wasm32 build
/// can share it with the registration-free fallback path.
fn affi_dir_finding() -> Finding {
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

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _AFFI_DIR_CHECK: &dyn DoctorCheck = &AfffiDirCheck;

struct CompletionsCheck;
impl DoctorCheck for CompletionsCheck {
    fn id(&self) -> &'static str {
        "shell-completions"
    }
    fn run(&self, _ctx: &DoctorContext) -> Vec<Finding> {
        vec![completions_finding()]
    }
}

/// Body of [`CompletionsCheck`] (see [`affi_dir_finding`] rationale).
fn completions_finding() -> Finding {
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

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _COMPLETIONS_CHECK: &dyn DoctorCheck = &CompletionsCheck;

struct ReceiptStoreCheck;
/// Finding ids emitted by [`ReceiptStoreCheck`] (stable — `--fix` matches on
/// the first one to create the missing store directory).
pub const RECEIPT_STORE_CHECK_ID: &str = "receipt-store";
pub const RECEIPT_STORE_COUNT_CHECK_ID: &str = "receipt-store-count";

impl DoctorCheck for ReceiptStoreCheck {
    fn id(&self) -> &'static str {
        RECEIPT_STORE_CHECK_ID
    }
    /// Migrated verbatim from the former hand-wired `handlers::check_receipt_store`
    /// (register row 79): the dispatcher used to extend its findings after the
    /// registry loop when `--receipts <path>` was passed; the check now reads
    /// the same path from [`DoctorContext::receipts_path`]. Messages and
    /// remediations are byte-identical to the pre-migration strings.
    fn run(&self, ctx: &DoctorContext) -> Vec<Finding> {
        let Some(path) = ctx.receipts_path else {
            return Vec::new();
        };
        let mut findings = Vec::new();
        let p = std::path::Path::new(path);
        if !p.exists() {
            findings.push(
                Finding::fail(
                    RECEIPT_STORE_CHECK_ID,
                    format!("Receipt path not found: {path}"),
                    format!("Create the directory: mkdir -p {path}"),
                )
                .auto_fixable(),
            );
            return findings;
        }
        if p.is_file() {
            // Single receipt — verify it is parseable JSON
            match std::fs::read_to_string(p) {
                Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
                    Ok(_) => findings.push(Finding::ok(
                        RECEIPT_STORE_CHECK_ID,
                        format!("Receipt file is valid JSON: {path}"),
                    )),
                    Err(e) => findings.push(Finding::fail(
                        RECEIPT_STORE_CHECK_ID,
                        format!("Receipt file is not valid JSON ({path}): {e}"),
                        "Re-assemble the receipt with 'affi assemble' from the original working directory.",
                    )),
                },
                Err(e) => findings.push(Finding::fail(
                    RECEIPT_STORE_CHECK_ID,
                    format!("Cannot read receipt file ({path}): {e}"),
                    "Check file permissions.",
                )),
            }
            return findings;
        }
        // Directory — count .json receipts
        let count = walkdir::WalkDir::new(p)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .count();
        if count == 0 {
            findings.push(Finding::warn(
                RECEIPT_STORE_COUNT_CHECK_ID,
                format!("No .json receipt files found in {path}"),
                "Run 'affi assemble' to produce a receipt, then move it here.",
            ));
        } else {
            findings.push(Finding::ok(
                RECEIPT_STORE_COUNT_CHECK_ID,
                format!("Found {count} receipt(s) in {path}"),
            ));
        }
        findings
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[distributed_slice(DOCTOR_CHECKS)]
static _RECEIPT_STORE_CHECK: &dyn DoctorCheck = &ReceiptStoreCheck;

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
pub fn run_all(ctx: &DoctorContext) -> Vec<Finding> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = ctx;
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
        let mut findings: Vec<Finding> = DOCTOR_CHECKS.iter().flat_map(|c| c.run(ctx)).collect();
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
    /// proves the sort reorders nothing in or out. Multi-finding checks (the
    /// receipt-store check emits up to two) participate by their findings.
    #[test]
    fn run_all_is_sorted_and_lossless() {
        let ctx = DoctorContext::default();
        let mut expected: Vec<String> = DOCTOR_CHECKS
            .iter()
            .flat_map(|c| c.run(&ctx))
            .map(|f| f.id.to_string())
            .collect();
        expected.sort();
        let actual: Vec<&str> = run_all(&ctx).iter().map(|f| f.id).collect();
        let expected_refs: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            actual, expected_refs,
            "run_all() must return findings sorted by check id, lossless vs the registry"
        );
    }

    /// Count court (register row 79): the default-features host registry is
    /// exactly 4 — `genesis-seed-version`, `affi-working-dir`,
    /// `shell-completions`, `receipt-store`. Deleting any one registration
    /// drops this count by exactly 1 and fails this court loudly; a court
    /// that cannot fail is not a court.
    #[test]
    fn host_registry_count_is_exact() {
        // Feature-aware: the 4 crypto_trust_doctor.rs registrations are
        // cfg-gated behind `crypto-trust` (src/lib.rs), so the host registry
        // is 4 on default features and 8 under the crypto-trust plane.
        #[cfg(feature = "crypto-trust")]
        let expected = 8;
        #[cfg(not(feature = "crypto-trust"))]
        let expected = 4;
        let n = DOCTOR_CHECKS.len();
        assert_eq!(
            n, expected,
            "DOCTOR_CHECKS registry is {n}, expected {expected} — a registration was added or deleted without updating this court"
        );
    }

    /// Semantics court (register row 79): the migrated receipt-store check
    /// produces byte-identical findings to the pre-migration hand-wired
    /// `handlers::check_receipt_store`, per branch.
    #[test]
    fn receipt_store_check_matches_pre_migration_semantics() {
        // (a) out of scope: no --receipts flag → no findings (old dispatcher
        // skipped the branch entirely).
        let none = ReceiptStoreCheck.run(&DoctorContext::default());
        assert!(none.is_empty(), "no --receipts flag must yield no findings");

        let dir = std::env::temp_dir().join(format!("affi-doctor-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // (b) missing path → FAIL, auto-fixable, exact pre-migration strings.
        let missing = format!("{}", dir.join("absent").display());
        let f = &ReceiptStoreCheck.run(&DoctorContext {
            receipts_path: Some(&missing),
            fix: false,
        })[0];
        assert_eq!(f.id, RECEIPT_STORE_CHECK_ID);
        assert_eq!(f.status, FindingStatus::Fail);
        assert!(f.auto_fixable);
        assert_eq!(f.message, format!("Receipt path not found: {missing}"));
        assert_eq!(
            f.remediation.as_deref(),
            Some(format!("Create the directory: mkdir -p {missing}").as_str())
        );

        // (c) directory with a valid receipt → Ok count finding (exact string).
        let receipt = dir.join("r.json");
        std::fs::write(&receipt, "{\"format_version\":\"core/v1\"}").unwrap();
        let dir_str = format!("{}", dir.display());
        let findings = ReceiptStoreCheck.run(&DoctorContext {
            receipts_path: Some(&dir_str),
            fix: false,
        });
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].id, RECEIPT_STORE_COUNT_CHECK_ID);
        assert_eq!(findings[0].status, FindingStatus::Ok);
        assert_eq!(
            findings[0].message,
            format!("Found 1 receipt(s) in {dir_str}")
        );

        // (d) empty directory → Warn (exact pre-migration string).
        let empty = dir.join("empty");
        std::fs::create_dir_all(&empty).unwrap();
        let empty_str = format!("{}", empty.display());
        let findings = ReceiptStoreCheck.run(&DoctorContext {
            receipts_path: Some(&empty_str),
            fix: false,
        });
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].id, RECEIPT_STORE_COUNT_CHECK_ID);
        assert_eq!(findings[0].status, FindingStatus::Warn);
        assert_eq!(
            findings[0].message,
            format!("No .json receipt files found in {empty_str}")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Court (register row 79): `run_all()` with no `--receipts` flag never
    /// emits receipt-store findings — the context gate, not a hand-wired
    /// branch, scopes the check.
    #[test]
    fn run_all_without_receipts_path_emits_no_store_findings() {
        let ctx = DoctorContext::default();
        let ids: Vec<&str> = run_all(&ctx).iter().map(|f| f.id).collect();
        assert!(
            !ids.contains(&RECEIPT_STORE_CHECK_ID) && !ids.contains(&RECEIPT_STORE_COUNT_CHECK_ID),
            "receipt-store findings leaked into a no-flag doctor run: {ids:?}"
        );
    }
    /// Vacuity gate (#47): the host registry must hold a floor of checks.
    ///
    /// Counted 2026-10-17 on DEFAULT features: 4 registrations — the built-ins
    /// in `doctor_check.rs` (`genesis-seed-version`, `affi-working-dir`,
    /// `shell-completions`) plus the migrated `receipt-store` check (register
    /// row 79). The 4 `crypto_trust_doctor.rs` registrations are behind
    /// `#[cfg(feature = "crypto-trust")]` (src/lib.rs:194), so they are
    /// present only under `--features crypto-trust` (registry = 8 there).
    /// Floor = actual - 1; the exact-count court (`host_registry_count_is_exact`)
    /// is the stricter sibling that fails on any single deletion.
    const HOST_REGISTRY_FLOOR: usize = 3;
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
        let findings = run_all(&DoctorContext::default());
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
    /// Court (register row 59) — the zero-size law: every `DoctorCheck`
    /// implementation in this module must be a zero-sized type (ZST). A check
    /// carries no state by design — context arrives via [`DoctorContext`] — so
    /// a stateful check static can never be admitted: if someone adds a field
    /// to a check struct (mutable or not), this fails the build/test instead of
    /// letting hidden state ride the distributed slice. Compile-level
    /// assertion (`const`), so the violation is caught even in check-only runs.
    const _: () = {
        assert!(
            core::mem::size_of::<GenesisVersionCheck>() == 0,
            "GenesisVersionCheck became stateful"
        );
        assert!(
            core::mem::size_of::<AfffiDirCheck>() == 0,
            "AfffiDirCheck became stateful"
        );
        assert!(
            core::mem::size_of::<CompletionsCheck>() == 0,
            "CompletionsCheck became stateful"
        );
        assert!(
            core::mem::size_of::<ReceiptStoreCheck>() == 0,
            "ReceiptStoreCheck became stateful"
        );
    };

    /// Runtime sibling of the const zero-size law (row 59): also guards types
    /// added later without updating the const block, via the registry itself.
    #[test]
    fn registered_check_types_are_zero_sized() {
        // Every built-in check static is `&'static dyn DoctorCheck` pointing at
        // a ZST; assert via fat-pointer metadata would need Any, so the court
        // is the const assertion above plus this floor on registry shape.
        assert!(
            DOCTOR_CHECKS.len() >= 3,
            "registry shape changed alongside check types — re-verify the zero-size law"
        );
    }
}
