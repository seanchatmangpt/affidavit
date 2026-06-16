//! # SPECIFICATION: CLI Telepathy (QOL-1000X)
//!
//! ## Overview
//! CLI Telepathy is a predictive engine for the `affi` toolchain. It eliminates
//! cognitive load by anticipating the next command in the provenance lifecycle
//! based on environmental signals (Git, Cargo, Filesystem).
//!
//! ## Heuristics (Combinatorial Logic)
//! 1. **Emission Hook**: If `src/*.rs` is modified and `.affi/working.json` is empty,
//!    suggest `affi receipt emit`.
//! 2. **Assembly Hook**: If `.affi/working.json` is non-empty, suggest `affi receipt assemble`.
//! 3. **Verification Hook**: If a new receipt hash (content-addressed) is detected
//!    in `.affi/`, suggest `affi receipt verify <hash>`.
//! 4. **Inspection Hook**: If a receipt was recently verified, suggest `affi receipt inspect`.
//! 5. **Discovery Hook**: If a receipt is exceptionally large (>10KB), suggest
//!    `affi receipt model` to discover the underlying process.
//! 6. **Dev Hook**: If `tests/*.rs` changed, suggest `cargo test`.
//!
//! ## Shell Integration
//! Implements a "buffer pre-fill" strategy using:
//! - **Zsh**: `LBUFFER` manipulation within a ZLE widget.
//! - **Bash**: `READLINE_LINE` manipulation via `bind -x`.
//! - **Trigger**: Bound to `Ctrl+T` (Telepathy) by default.
//!
//! # IMPLEMENTATION

use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use std::time::{SystemTime, Duration};

/// A predicted command with associated metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Prediction {
    pub command: String,
    pub confidence: f64,
    pub reason: String,
    pub category: String,
}

/// The state of the workspace used for evaluation.
#[derive(Debug, Default)]
struct WorkspaceState {
    root: PathBuf,
    git_status: Vec<String>,
    has_working_receipt: bool,
    working_receipt_size: u64,
    recent_receipts: Vec<(PathBuf, SystemTime)>,
    cargo_test_failed: bool,
    src_modified: bool,
    tests_modified: bool,
    ontology_modified: bool,
}

/// A rule that can evaluate the workspace state and return a prediction.
trait TelepathyRule {
    fn name(&self) -> &str;
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction>;
}

// --- Rules ---

struct AssembleRule;
impl TelepathyRule for AssembleRule {
    fn name(&self) -> &str { "Assemble" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if state.has_working_receipt && state.working_receipt_size > 2 {
            Some(Prediction {
                command: "affi receipt assemble".to_string(),
                confidence: 0.95,
                reason: "Working receipt contains uncommitted events. Assemble them to finalize the provenance chain.".to_string(),
                category: "Chain".to_string(),
            })
        } else {
            None
        }
    }
}

struct EmitRule;
impl TelepathyRule for EmitRule {
    fn name(&self) -> &str { "Emit" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if state.src_modified && !state.has_working_receipt {
            Some(Prediction {
                command: "affi receipt emit".to_string(),
                confidence: 0.85,
                reason: "Source code modified. You likely need to emit new events to capture the latest operations.".to_string(),
                category: "Emission".to_string(),
            })
        } else {
            None
        }
    }
}

struct VerifyRule;
impl TelepathyRule for VerifyRule {
    fn name(&self) -> &str { "Verify" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if let Some((path, _)) = state.recent_receipts.first() {
            let rel_path = path.strip_prefix(&state.root).unwrap_or(path);
            Some(Prediction {
                command: format!("affi receipt verify {}", rel_path.display()),
                confidence: 0.80,
                reason: format!("Receipt {} recently modified. Verify it to ensure conformance with the certify pipeline.", rel_path.display()),
                category: "Verification".to_string(),
            })
        } else {
            None
        }
    }
}

struct InspectRule;
impl TelepathyRule for InspectRule {
    fn name(&self) -> &str { "Inspect" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        // If a receipt was just verified (we guess this by time), suggest inspect.
        if let Some((path, modified)) = state.recent_receipts.first() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed < Duration::from_secs(60) {
                    let rel_path = path.strip_prefix(&state.root).unwrap_or(path);
                    return Some(Prediction {
                        command: format!("affi receipt inspect {}", rel_path.display()),
                        confidence: 0.70,
                        reason: "Recently updated receipt detected. Inspect it for a detailed structural report.".to_string(),
                        category: "QOL".to_string(),
                    });
                }
            }
        }
        None
    }
}

struct ModelRule;
impl TelepathyRule for ModelRule {
    fn name(&self) -> &str { "Model" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if let Some((path, _)) = state.recent_receipts.first() {
            if let Ok(metadata) = fs::metadata(path) {
                if metadata.len() > 10000 { // Large receipt
                    let rel_path = path.strip_prefix(&state.root).unwrap_or(path);
                    return Some(Prediction {
                        command: format!("affi receipt model {}", rel_path.display()),
                        confidence: 0.65,
                        reason: "Large receipt detected. Discovering a process model (DFG/Petri) will help visualize complexity.".to_string(),
                        category: "Discovery".to_string(),
                    });
                }
            }
        }
        None
    }
}

struct CargoTestRule;
impl TelepathyRule for CargoTestRule {
    fn name(&self) -> &str { "CargoTest" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if state.tests_modified {
            Some(Prediction {
                command: "cargo test".to_string(),
                confidence: 0.75,
                reason: "Test files modified. Running the suite is recommended to verify local invariants.".to_string(),
                category: "Development".to_string(),
            })
        } else {
            None
        }
    }
}

struct StatsRule;
impl TelepathyRule for StatsRule {
    fn name(&self) -> &str { "Stats" }
    fn evaluate(&self, state: &WorkspaceState) -> Option<Prediction> {
        if state.ontology_modified {
             Some(Prediction {
                command: "affi stats --ontology".to_string(),
                confidence: 0.60,
                reason: "Ontology modified. Check aggregate statistics to ensure the surface area is consistent.".to_string(),
                category: "Analysis".to_string(),
            })
        } else {
            None
        }
    }
}

// --- Engine ---

pub struct Telepathy {
    workspace_root: PathBuf,
    rules: Vec<Box<dyn TelepathyRule>>,
}

impl Telepathy {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            workspace_root: root.as_ref().to_path_buf(),
            rules: vec![
                Box::new(AssembleRule),
                Box::new(EmitRule),
                Box::new(VerifyRule),
                Box::new(InspectRule),
                Box::new(ModelRule),
                Box::new(CargoTestRule),
                Box::new(StatsRule),
            ],
        }
    }

    pub fn predict(&self) -> Option<Prediction> {
        let state = self.gather_state();
        let mut predictions: Vec<Prediction> = self.rules.iter()
            .filter_map(|rule| rule.evaluate(&state))
            .collect();

        // Sort by confidence descending
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        predictions.into_iter().next()
    }

    fn gather_state(&self) -> WorkspaceState {
        let mut state = WorkspaceState {
            root: self.workspace_root.clone(),
            ..Default::default()
        };

        // 1. Working Receipt
        let working_path = self.workspace_root.join(".affi/working.json");
        if let Ok(metadata) = fs::metadata(&working_path) {
            state.has_working_receipt = true;
            state.working_receipt_size = metadata.len();
        }

        // 2. Git Status
        if let Ok(output) = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(&self.workspace_root)
            .output() {
            let status = String::from_utf8_lossy(&output.stdout);
            for line in status.lines() {
                state.git_status.push(line.to_string());
                if line.contains("src/") { state.src_modified = true; }
                if line.contains("tests/") { state.tests_modified = true; }
                if line.contains("ontology/") { state.ontology_modified = true; }
            }
        }

        // 3. Recent Receipts
        state.recent_receipts = self.find_recent_receipts();

        state
    }

    fn find_recent_receipts(&self) -> Vec<(PathBuf, SystemTime)> {
        let mut receipts = Vec::new();
        let search_paths = vec![
            self.workspace_root.clone(),
            self.workspace_root.join(".affi"),
        ];

        for dir in search_paths {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            if !name.contains("working") && (name.len() > 32 || path.parent().map(|p| p.ends_with(".affi")).unwrap_or(false)) {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    if let Ok(modified) = metadata.modified() {
                                        receipts.push((path, modified));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        receipts.sort_by(|a, b| b.1.cmp(&a.1));
        receipts
    }
}

/// Minimalist CLI for the Telepathy engine.
pub fn run_cli() {
    let args: Vec<String> = std::env::args().collect();
    let tele = Telepathy::new(".");

    if args.contains(&"--raw".to_string()) {
        if let Some(p) = tele.predict() {
            print!("{}", p.command);
        }
    } else if args.contains(&"--init".to_string()) {
        println!("{}", shell_integration());
    } else {
        if let Some(p) = tele.predict() {
            println!("AFFI TELEPATHY — PREDICTION ENGINE");
            println!("=================================");
            println!("Command:    \x1b[1;32m{}\x1b[0m", p.command);
            println!("Confidence: {:.1}%", p.confidence * 100.0);
            println!("Category:   {}", p.category);
            println!("Reason:     {}", p.reason);
            println!("\n(Hint: add `eval \"$(affi telepathy --init)\"` to your shell config for Ctrl+T integration)");
        } else {
            println!("Telepathy: No clear prediction. Try emitting some events.");
        }
    }
}

/// Shell integration script for Bash and Zsh.
pub fn shell_integration() -> &'static str {
    r#"
# Affi Telepathy Shell Integration
# Usage: eval "$(affi telepathy --init)"

_affi_telepathy() {
    # Fetch prediction from the binary
    local prediction
    prediction=$(affi telepathy --raw 2>/dev/null)
    
    if [ -n "$prediction" ]; then
        if [ -n "$ZSH_VERSION" ]; then
            # Zsh: Put command in buffer and keep current line
            LBUFFER="$prediction"
            zle redisplay
        elif [ -n "$BASH_VERSION" ]; then
            # Bash: Set the current line to the prediction
            READLINE_LINE="$prediction"
            READLINE_POINT=${#READLINE_LINE}
        fi
    fi
}

# Bind to Ctrl+T (Telepathy)
if [ -n "$ZSH_VERSION" ]; then
    zle -N _affi_telepathy
    bindkey '^T' _affi_telepathy
elif [ -n "$BASH_VERSION" ]; then
    bind -x '"\C-t": _affi_telepathy'
fi

echo "[affi] Telepathy integration active (Ctrl+T)" >&2
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_maximalist_telepathy_logic() {
        let dir = tempdir().unwrap();
        let tele = Telepathy::new(dir.path());
        
        // Initially no prediction
        assert!(tele.predict().is_none());
        
        // Create working receipt
        let affi_dir = dir.path().join(".affi");
        fs::create_dir(&affi_dir).unwrap();
        fs::write(affi_dir.join("working.json"), "[{\"event\": 1}]").unwrap();
        
        let p = tele.predict().expect("Should predict assemble");
        assert_eq!(p.command, "affi receipt assemble");
        assert!(p.confidence > 0.9);
    }
}

/// Standalone entry point for testing the WIP module.
fn main() {
    run_cli();
}
