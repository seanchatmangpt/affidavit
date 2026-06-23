//! Fitness evaluation: turning a [`Genome`] into a scored [`EvalResult`].
//!
//! An [`Evaluator`] answers one question about a configuration: *does it build,
//! and how well?* Two implementations ship:
//!
//! * [`CargoEvaluator`] — the real thing: shells out to `cargo build
//!   --message-format=json` in a target crate and counts errors/warnings.
//! * [`SyntheticEvaluator`] — a deterministic, **hermetic** model that never
//!   spawns a subprocess. It is what the test-suite and `--dry-run` use, so smoke
//!   tests are instantaneous and reproducible.
//!
//! Scoring is shared via [`score_from`]/[`ScoreWeights`]: a clean *build* is worth
//! a lot, dependency *resolution* and feature *breadth* a little, and compile
//! *errors* and wall-clock *time* are penalties — in that priority order.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crate::genome::Genome;
use crate::space::FeatureSpace;

/// The outcome of evaluating one configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct EvalResult {
    /// The genome's (literal, sorted) features.
    pub features: Vec<String>,
    /// Did dependency resolution succeed?
    pub resolves: bool,
    /// Did the crate build cleanly (exit code 0)?
    pub builds: bool,
    /// Number of compiler errors observed.
    pub error_count: u64,
    /// Number of compiler warnings observed.
    pub warn_count: u64,
    /// Wall-clock seconds the evaluation took.
    pub elapsed_s: f64,
    /// The scalar fitness (higher is better).
    pub score: f64,
    /// `true` if this result was served from a cache rather than recomputed.
    pub from_cache: bool,
}

/// Tunable weights for [`score_from`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreWeights {
    /// Reward for a clean build.
    pub build_bonus: f64,
    /// Reward for successful dependency resolution.
    pub resolve_bonus: f64,
    /// Reward per enabled feature (breadth).
    pub per_feature: f64,
    /// Penalty per compiler error.
    pub error_penalty: f64,
    /// Penalty per wall-clock second.
    pub time_penalty: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        // Mirrors the reference Python implementation's scoring constants.
        ScoreWeights {
            build_bonus: 1000.0,
            resolve_bonus: 50.0,
            per_feature: 2.0,
            error_penalty: 1.0,
            time_penalty: 0.1,
        }
    }
}

/// Compute a fitness score from raw evaluation signals.
///
/// Heavily rewards a clean build, lightly rewards resolution and feature breadth,
/// and penalizes compiler errors and wall-clock time.
pub fn score_from(
    weights: &ScoreWeights,
    builds: bool,
    resolves: bool,
    error_count: u64,
    n_features: usize,
    elapsed_s: f64,
) -> f64 {
    let mut s = if builds { weights.build_bonus } else { 0.0 };
    if resolves {
        s += weights.resolve_bonus;
    }
    s -= weights.error_penalty * (error_count as f64);
    s += weights.per_feature * (n_features as f64);
    s -= weights.time_penalty * elapsed_s;
    s
}

/// Something that can score a genome against a feature space.
///
/// Implementors only need to compute an [`EvalResult`]; caching and memoization
/// are handled by the GA driver ([`crate::run_ga`]), which dedups by the genome's
/// canonical key.
pub trait Evaluator {
    /// Evaluate `genome` interpreted against `space`.
    fn evaluate(&mut self, genome: &Genome, space: &FeatureSpace) -> EvalResult;
}

// ---------------------------------------------------------------------------
// Synthetic evaluator (hermetic; no subprocess)
// ---------------------------------------------------------------------------

/// A deterministic, dependency-free model of build outcomes.
///
/// No `cargo`, no subprocess, no I/O — perfect for tests, CI smoke runs, and
/// `--dry-run`. The model is honest about one thing: it reports `builds = true`
/// **only** when its modeled error count reaches zero, so a poisoned feature space
/// never fakes a green build.
#[derive(Debug, Clone)]
pub struct SyntheticEvaluator {
    weights: ScoreWeights,
    /// Errors charged before any feature-specific penalties.
    base_errors: u64,
    /// Per-feature error contributions, charged when a feature is in the
    /// *canonical* (closure-expanded) set.
    poison: BTreeMap<String, u64>,
}

impl SyntheticEvaluator {
    /// A model with an explicit base error count and per-feature penalties.
    ///
    /// `poison` maps a feature name to the number of synthetic errors it adds when
    /// present in the canonical feature-set. Use this to encode domain knowledge
    /// (e.g. "the `core` feature pulls in a broken dependency: +550").
    pub fn new<I, S>(base_errors: u64, poison: I) -> Self
    where
        I: IntoIterator<Item = (S, u64)>,
        S: Into<String>,
    {
        SyntheticEvaluator {
            weights: ScoreWeights::default(),
            base_errors,
            poison: poison.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }

    /// A generic model that needs no per-project knowledge.
    ///
    /// Charges a small base error count plus a deterministic per-feature penalty
    /// derived from a hash of each feature name. This gives the search landscape a
    /// reproducible gradient (some feature combinations score better than others)
    /// without modeling any specific crate. Nothing ever "builds" — exactly the
    /// expected shape when a feature space hides a structural obstruction.
    pub fn generic() -> Self {
        SyntheticEvaluator {
            weights: ScoreWeights::default(),
            base_errors: 1,
            poison: BTreeMap::new(),
        }
    }

    /// Override the scoring weights.
    pub fn with_weights(mut self, weights: ScoreWeights) -> Self {
        self.weights = weights;
        self
    }

    /// Deterministic pseudo-penalty for an unmodeled feature, derived from its
    /// name so the landscape is stable across runs.
    fn implicit_penalty(name: &str) -> u64 {
        // FNV-1a over the bytes, folded into a small [1, 8] range.
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in name.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01B3);
        }
        1 + (h % 8)
    }
}

impl Evaluator for SyntheticEvaluator {
    fn evaluate(&mut self, genome: &Genome, space: &FeatureSpace) -> EvalResult {
        let canonical = genome.canonical(space);
        let canon_feats = canonical.features();

        let mut error_count = self.base_errors;
        for f in &canon_feats {
            error_count += match self.poison.get(f) {
                Some(p) => *p,
                None => {
                    if self.poison.is_empty() {
                        // Generic mode: give every feature a small stable penalty.
                        Self::implicit_penalty(f)
                    } else {
                        0
                    }
                }
            };
        }

        let warn_count = canon_feats.len() as u64;
        let builds = error_count == 0;
        let resolves = true; // synthetic dependency resolution always succeeds
        let elapsed_s = 0.5 + 0.1 * (canon_feats.len() as f64);
        let n_features = genome.len();
        let score = score_from(
            &self.weights,
            builds,
            resolves,
            error_count,
            n_features,
            elapsed_s,
        );

        EvalResult {
            features: genome.features(),
            resolves,
            builds,
            error_count,
            warn_count,
            elapsed_s,
            score,
            from_cache: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Cargo evaluator (real builds)
// ---------------------------------------------------------------------------

/// Cargo-level dependency-resolution failure markers.
///
/// Deliberately excludes "failed to resolve", which is the wording of the E0433
/// *compile* error ("failed to resolve: cannot find module ...") and would
/// otherwise misreport a crate that resolved fine but failed to compile.
const RESOLVE_FAIL_MARKERS: &[&str] = &[
    "failed to select a version",
    "no matching package",
    "error: failed to get",
    "failed to load source",
    "unable to update",
];

/// Evaluates a genome by actually running `cargo build` in a target crate.
///
/// Runs `cargo build --no-default-features --lib --message-format=json --features
/// <genome>` in `manifest_dir`, counts compiler errors/warnings from the JSON
/// stream, and derives `builds` from the exit code. Error/warning counting is a
/// best-effort line scan (see [`count_compiler_messages`]); it needs no JSON
/// dependency and is robust to non-message lines.
#[derive(Debug, Clone)]
pub struct CargoEvaluator {
    manifest_dir: PathBuf,
    weights: ScoreWeights,
    extra_args: Vec<String>,
}

impl CargoEvaluator {
    /// Create an evaluator that builds the crate rooted at `manifest_dir`.
    pub fn new<P: Into<PathBuf>>(manifest_dir: P) -> Self {
        CargoEvaluator {
            manifest_dir: manifest_dir.into(),
            weights: ScoreWeights::default(),
            extra_args: Vec::new(),
        }
    }

    /// Override the scoring weights.
    pub fn with_weights(mut self, weights: ScoreWeights) -> Self {
        self.weights = weights;
        self
    }

    /// Append extra arguments to every `cargo build` invocation (e.g. `--release`).
    pub fn with_extra_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extra_args = args.into_iter().map(Into::into).collect();
        self
    }

    /// `true` if a `cargo` executable is discoverable (a cheap preflight).
    ///
    /// Real-mode evaluation shells out to `cargo`; callers should preflight with
    /// this and steer the user to the synthetic evaluator when it returns `false`,
    /// rather than discovering the problem one failed build at a time.
    pub fn cargo_available() -> bool {
        Command::new("cargo")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Evaluator for CargoEvaluator {
    fn evaluate(&mut self, genome: &Genome, _space: &FeatureSpace) -> EvalResult {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.manifest_dir)
            .arg("build")
            .arg("--no-default-features")
            .arg("--lib")
            .arg("--message-format=json");
        let feat_arg = genome.cargo_features_arg();
        if !feat_arg.is_empty() {
            cmd.arg("--features").arg(&feat_arg);
        }
        for a in &self.extra_args {
            cmd.arg(a);
        }

        let start = Instant::now();
        let output = cmd.output();
        let elapsed_s = start.elapsed().as_secs_f64();

        let (builds, resolves, error_count, warn_count) = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let (errors, warns) = count_compiler_messages(&stdout);
                let resolves = resolves_from_output(&format!("{stdout}\n{stderr}"));
                (out.status.success(), resolves, errors, warns)
            }
            Err(_) => {
                // cargo missing or unspawnable: strongly-penalized sentinel rather
                // than crashing a long search mid-run.
                (false, false, 1_000_000, 0)
            }
        };

        let features = genome.features();
        let n_features = features.len();
        let score = score_from(
            &self.weights,
            builds,
            resolves,
            error_count,
            n_features,
            elapsed_s,
        );

        EvalResult {
            features,
            resolves,
            builds,
            error_count,
            warn_count,
            elapsed_s,
            score,
            from_cache: false,
        }
    }
}

/// Heuristic: dependency resolution succeeded unless a known marker appears.
pub fn resolves_from_output(combined: &str) -> bool {
    !RESOLVE_FAIL_MARKERS.iter().any(|m| combined.contains(m))
}

/// Count compiler errors and warnings in a cargo `--message-format=json` stream.
///
/// Each non-empty line is one JSON object. We treat a line as a diagnostic when it
/// carries `"reason":"compiler-message"`, then classify it by the diagnostic level
/// substring it contains. This is a deliberate, dependency-free heuristic — it
/// trades a JSON parser for a substring scan — and is robust to the non-message
/// lines (`compiler-artifact`, `build-finished`, …) cargo interleaves.
pub fn count_compiler_messages(stdout: &str) -> (u64, u64) {
    let mut errors = 0u64;
    let mut warns = 0u64;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains("\"reason\":\"compiler-message\"") {
            continue;
        }
        if line.contains("\"level\":\"error\"") {
            errors += 1;
        } else if line.contains("\"level\":\"warning\"") {
            warns += 1;
        }
    }
    (errors, warns)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space() -> FeatureSpace {
        FeatureSpace::new(
            ["core", "discovery", "ui", "otel", "metrics"],
            [("discovery", vec!["core"]), ("metrics", vec!["otel"])],
        )
        .unwrap()
    }

    #[test]
    fn score_rewards_build_over_no_build() {
        let w = ScoreWeights::default();
        let built = score_from(&w, true, true, 0, 3, 1.0);
        let not = score_from(&w, false, true, 0, 3, 1.0);
        assert!(built > not);
    }

    #[test]
    fn score_penalizes_errors_and_time() {
        let w = ScoreWeights::default();
        let clean = score_from(&w, false, true, 0, 0, 0.0);
        let many = score_from(&w, false, true, 10, 0, 0.0);
        let slow = score_from(&w, false, true, 0, 0, 100.0);
        assert!(clean > many);
        assert!(clean > slow);
    }

    #[test]
    fn synthetic_charges_poison_via_canonical_closure() {
        // `discovery` implies `core`; poison only `core`. Enabling discovery must
        // still incur the core penalty because canonicalization pulls it in.
        let mut eval = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let s = space();
        let r = eval.evaluate(&Genome::new(["discovery"]), &s);
        assert_eq!(r.error_count, 6 + 550);
        assert!(!r.builds);
    }

    #[test]
    fn synthetic_never_fakes_a_build_with_base_errors() {
        let mut eval = SyntheticEvaluator::new(6, Vec::<(&str, u64)>::new());
        let s = space();
        let r = eval.evaluate(&Genome::empty(), &s);
        assert_eq!(r.error_count, 6);
        assert!(!r.builds, "synthetic model must not fake a green build");
    }

    #[test]
    fn synthetic_empty_poison_zero_base_builds_empty() {
        // A genuinely clean space (base 0, no poison) SHOULD report a build for the
        // empty genome — the honest positive case.
        let mut eval = SyntheticEvaluator::new(0, [("core", 5u64)]);
        let s = space();
        let r = eval.evaluate(&Genome::empty(), &s);
        assert_eq!(r.error_count, 0);
        assert!(r.builds);
    }

    #[test]
    fn generic_model_is_deterministic_and_unbuilding() {
        let mut a = SyntheticEvaluator::generic();
        let mut b = SyntheticEvaluator::generic();
        let s = space();
        let g = Genome::new(["ui", "otel"]);
        let ra = a.evaluate(&g, &s);
        let rb = b.evaluate(&g, &s);
        assert_eq!(ra.error_count, rb.error_count);
        assert_eq!(ra.score, rb.score);
        assert!(!ra.builds);
    }

    #[test]
    fn count_messages_classifies_lines() {
        let stream = concat!(
            "{\"reason\":\"compiler-artifact\",\"x\":1}\n",
            "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\"}}\n",
            "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"warning\"}}\n",
            "{\"reason\":\"compiler-message\",\"message\":{\"level\":\"error\"}}\n",
            "{\"reason\":\"build-finished\",\"success\":false}\n",
            "\n",
        );
        assert_eq!(count_compiler_messages(stream), (2, 1));
    }

    #[test]
    fn resolves_detects_failure_markers() {
        assert!(resolves_from_output("warning: unused\nFinished"));
        assert!(!resolves_from_output(
            "error: no matching package named `x`"
        ));
    }
}
