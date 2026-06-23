//! Rendering a [`GaResult`] to machine-readable JSON and human-readable Markdown.
//!
//! Both renderers are dependency-free: JSON is emitted by a tiny hand-rolled
//! serializer (just enough to produce valid output for the fixed result shape) and
//! Markdown is plain string building. The "honest finding" section states plainly
//! whether *any* configuration built, and — when none did — frames the run as a
//! map of the obstruction rather than a failure.

use crate::evolve::{GaConfig, GaResult};
use crate::fitness::EvalResult;

/// Whether the run used the synthetic model or real cargo builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Synthetic, hermetic evaluation (`--dry-run`).
    DryRun,
    /// Real `cargo build` evaluation.
    Real,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::DryRun => "dry-run",
            Mode::Real => "real",
        }
    }
}

/// Escape a string for embedding in a JSON document.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Render a finite float as a JSON number (never `NaN`/`Infinity`, which are not
/// valid JSON). Non-finite values fall back to `0`.
fn num(x: f64) -> String {
    if x.is_finite() {
        // Ryu-free: Rust's default Display is shortest-round-trippable for f64.
        format!("{x}")
    } else {
        "0".to_string()
    }
}

fn json_str_array(items: &[String]) -> String {
    let parts: Vec<String> = items.iter().map(|s| format!("\"{}\"", esc(s))).collect();
    format!("[{}]", parts.join(", "))
}

fn eval_to_json(ev: &EvalResult) -> String {
    format!(
        "{{\"features\": {feats}, \"resolves\": {res}, \"builds\": {builds}, \
         \"error_count\": {ec}, \"warn_count\": {wc}, \"elapsed_s\": {el}, \
         \"score\": {sc}, \"from_cache\": {fc}}}",
        feats = json_str_array(&ev.features),
        res = ev.resolves,
        builds = ev.builds,
        ec = ev.error_count,
        wc = ev.warn_count,
        el = num(ev.elapsed_s),
        sc = num(ev.score),
        fc = ev.from_cache,
    )
}

/// Render the full result as a JSON document.
pub fn to_json(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"mode\": \"{}\",\n", mode.label()));
    s.push_str(&format!("  \"manifest\": \"{}\",\n", esc(manifest)));
    s.push_str(&format!("  \"feature_universe_size\": {universe},\n"));
    s.push_str("  \"config\": {\n");
    s.push_str(&format!("    \"population\": {},\n", cfg.population));
    s.push_str(&format!("    \"generations\": {},\n", cfg.generations));
    s.push_str(&format!("    \"seed\": {},\n", cfg.seed));
    s.push_str(&format!(
        "    \"mutation_rate\": {},\n",
        num(cfg.mutation_rate)
    ));
    s.push_str(&format!(
        "    \"crossover_rate\": {},\n",
        num(cfg.crossover_rate)
    ));
    s.push_str(&format!("    \"elitism\": {},\n", cfg.elitism));
    s.push_str(&format!("    \"tournament_k\": {}\n", cfg.tournament_k));
    s.push_str("  },\n");

    // History.
    s.push_str("  \"history\": [\n");
    for (i, rec) in result.history.iter().enumerate() {
        let comma = if i + 1 < result.history.len() {
            ","
        } else {
            ""
        };
        s.push_str(&format!(
            "    {{\"index\": {idx}, \"best_score\": {bs}, \"mean_score\": {ms}, \"best_features\": {bf}}}{comma}\n",
            idx = rec.index,
            bs = num(rec.best_score),
            ms = num(rec.mean_score),
            bf = json_str_array(&rec.best_features),
        ));
    }
    s.push_str("  ],\n");

    s.push_str(&format!(
        "  \"best\": {},\n",
        eval_to_json(&result.best_eval)
    ));
    s.push_str(&format!(
        "  \"total_evaluations\": {}\n",
        result.evaluations
    ));
    s.push_str("}\n");
    s
}

/// Render the full result as a Markdown report.
pub fn to_markdown(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) -> String {
    let dry = mode == Mode::DryRun;
    let best = &result.best_eval;
    let any_built = best.builds;

    let mut l = String::new();
    let suffix = if dry { " — SIMULATED (dry-run)" } else { "" };
    l.push_str(&format!("# confevo report{suffix}\n\n"));

    if dry {
        l.push_str(
            "> **SIMULATED RUN.** Produced with the synthetic evaluator: the fitness \
             function is a model and **no `cargo` commands were executed**. Scores \
             illustrate the search machinery, not real compiler behavior.\n\n",
        );
    }

    l.push_str(
        "This is the output of **confevo**, a zero-dependency genetic algorithm that \
         evolves **Cargo feature-flag configurations**. Each candidate is a subset of \
         the crate's `[features]`; the GA selects, crosses over, and mutates these \
         feature-sets to maximize a fitness function that **prefers configurations \
         that *build*, that *resolve*, that have *fewer errors*, that enable *more \
         features*, and that finish *faster*** — in that priority order.\n\n",
    );

    // Parameters.
    l.push_str("## Run parameters\n\n");
    l.push_str("| Parameter | Value |\n| --- | --- |\n");
    l.push_str(&format!(
        "| mode | {} |\n",
        if dry {
            "SIMULATED (dry-run)"
        } else {
            "real (cargo)"
        }
    ));
    l.push_str(&format!("| manifest | `{manifest}` |\n"));
    l.push_str(&format!("| generations | {} |\n", cfg.generations));
    l.push_str(&format!("| population | {} |\n", cfg.population));
    l.push_str(&format!("| seed | {} |\n", cfg.seed));
    l.push_str(&format!("| mutation rate | {} |\n", cfg.mutation_rate));
    l.push_str(&format!("| crossover rate | {} |\n", cfg.crossover_rate));
    l.push_str(&format!("| elitism | {} |\n", cfg.elitism));
    l.push_str(&format!("| tournament k | {} |\n", cfg.tournament_k));
    l.push_str(&format!("| total evaluations | {} |\n", result.evaluations));
    l.push_str(&format!("| feature universe size | {universe} |\n\n"));

    // Generations.
    l.push_str("## Generations\n\n");
    l.push_str("| Gen | Best score | Mean score | Best configuration |\n");
    l.push_str("| ---: | ---: | ---: | --- |\n");
    for rec in &result.history {
        l.push_str(&format!(
            "| {} | {:.4} | {:.4} | {} |\n",
            rec.index,
            rec.best_score,
            rec.mean_score,
            fmt_feats(&rec.best_features),
        ));
    }
    l.push('\n');

    // Best configuration.
    l.push_str("## Best configuration found\n\n");
    let feats = &best.features;
    l.push_str(&format!("- **Features:** {}\n", fmt_feats(feats)));
    if feats.is_empty() {
        l.push_str("- **`cargo` invocation:** `cargo build --no-default-features`\n\n");
    } else {
        l.push_str(&format!(
            "- **`cargo` invocation:** `cargo build --no-default-features --features {}`\n\n",
            feats.join(","),
        ));
    }
    l.push_str("| Field | Value |\n| --- | --- |\n");
    l.push_str(&format!("| resolves | {} |\n", best.resolves));
    l.push_str(&format!("| builds | {} |\n", best.builds));
    l.push_str(&format!("| error_count | {} |\n", best.error_count));
    l.push_str(&format!("| warn_count | {} |\n", best.warn_count));
    l.push_str(&format!("| elapsed_s | {:.4} |\n", best.elapsed_s));
    l.push_str(&format!("| score | {:.4} |\n\n", best.score));

    // Honest finding.
    l.push_str("## Honest finding\n\n");
    if any_built {
        l.push_str(
            "At least one evaluated configuration reported a **successful build**; the \
             winning feature-set is above.\n",
        );
        if dry {
            l.push_str(
                "\n_Caveat: this was a **SIMULATED** run. Re-run against real `cargo` to \
                 confirm._\n",
            );
        }
    } else {
        l.push_str(
            "**No configuration built successfully.** When the obstruction is *below* \
             every feature gate (e.g. a broken upstream dependency), no subset of the \
             crate's own features can route around it — the blocker is structural, not \
             configurational. The value of this run is diagnostic: confevo **maps the \
             feature space and surfaces the configuration that gets closest** (fewest \
             errors / best resolution), which is the most useful starting point for a \
             fix.\n",
        );
        if dry {
            l.push_str(
                "\n_(This run was **SIMULATED**; the error counts are synthetic, but the \
                 structural conclusion holds for a real run.)_\n",
            );
        }
    }

    l
}

fn fmt_feats(feats: &[String]) -> String {
    if feats.is_empty() {
        "`<none>`".to_string()
    } else {
        format!("`{}`", feats.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evolve::{run_ga, GaConfig};
    use crate::fitness::SyntheticEvaluator;
    use crate::space::FeatureSpace;

    fn run() -> (GaResult, GaConfig) {
        let space = FeatureSpace::new(
            ["core", "discovery", "ui", "otel"],
            [("discovery", vec!["core"])],
        )
        .unwrap();
        let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let cfg = GaConfig {
            generations: 4,
            population: 5,
            seed: 1,
            ..Default::default()
        };
        (run_ga(&mut e, &space, &cfg).unwrap(), cfg)
    }

    #[test]
    fn json_is_wellformed_enough() {
        let (r, cfg) = run();
        let j = to_json(&r, &cfg, Mode::DryRun, "Cargo.toml", 4);
        // Balanced braces/brackets and key presence.
        assert_eq!(j.matches('{').count(), j.matches('}').count());
        assert_eq!(j.matches('[').count(), j.matches(']').count());
        for key in [
            "\"mode\"",
            "\"history\"",
            "\"best\"",
            "\"total_evaluations\"",
        ] {
            assert!(j.contains(key), "missing {key}");
        }
    }

    #[test]
    fn json_escapes_quotes_in_manifest_path() {
        let (r, cfg) = run();
        let j = to_json(&r, &cfg, Mode::Real, "weird\"path", 4);
        assert!(j.contains("weird\\\"path"));
    }

    #[test]
    fn markdown_reports_honest_no_build() {
        let (r, cfg) = run();
        let md = to_markdown(&r, &cfg, Mode::DryRun, "Cargo.toml", 4);
        assert!(md.contains("SIMULATED"));
        assert!(md.contains("No configuration built successfully"));
        assert!(md.contains("Best configuration found"));
    }
}
