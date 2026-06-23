//! `confevo` — CLI driver for the genetic Cargo-feature optimizer.
//!
//! ```text
//! confevo run [--manifest PATH] [--generations N] [--population N] [--seed N]
//!             [--mutation-rate F] [--crossover-rate F] [--elitism N]
//!             [--tournament-k N] [--dry-run] [--include-default]
//!             [--out DIR] [--release]
//! ```
//!
//! Reads the `[features]` table from a crate's `Cargo.toml`, evolves a
//! configuration, and writes `results.json` + `report.md` to the output dir. With
//! `--dry-run` it uses the hermetic synthetic evaluator (no `cargo`); otherwise it
//! shells out to real `cargo build`.

use std::path::PathBuf;
use std::process::ExitCode;

use confevo::report::{to_json, to_markdown, Mode};
use confevo::{
    feature_space_from_cargo_toml, run_ga, CargoEvaluator, Evaluator, GaConfig, SyntheticEvaluator,
};

/// Parsed CLI options for the `run` subcommand.
struct Options {
    manifest: PathBuf,
    out: PathBuf,
    cfg: GaConfig,
    dry_run: bool,
    include_default: bool,
    release: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            manifest: PathBuf::from("Cargo.toml"),
            out: PathBuf::from("confevo-out"),
            cfg: GaConfig::default(),
            dry_run: false,
            include_default: false,
            release: false,
        }
    }
}

fn usage() -> &'static str {
    "\
confevo — genetic optimizer for Cargo feature-flag configurations

USAGE:
    confevo run [OPTIONS]

OPTIONS:
    --manifest PATH        Cargo.toml to read [features] from   (default: Cargo.toml)
    --out DIR              output directory for reports          (default: confevo-out)
    --generations N        number of GA generations             (default: 3)
    --population N         population size                       (default: 6)
    --seed N               RNG seed for reproducibility          (default: 0)
    --mutation-rate F      per-feature bit-flip probability      (default: 0.1)
    --crossover-rate F     crossover probability                 (default: 0.9)
    --elitism N            elites carried over each generation   (default: 1)
    --tournament-k N       tournament selection size             (default: 3)
    --include-default      keep the `default` feature in the search universe
    --release              pass --release to cargo (real mode only)
    --dry-run              use the synthetic evaluator (no cargo)
    -h, --help             print this help
"
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("run") => {}
        Some("-h") | Some("--help") | None => return Err("help".to_string()),
        Some(other) => return Err(format!("unknown command {other:?} (expected `run`)")),
    }

    let mut o = Options::default();
    while let Some(arg) = it.next() {
        let mut next = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--manifest" => o.manifest = PathBuf::from(next()?),
            "--out" => o.out = PathBuf::from(next()?),
            "--generations" => o.cfg.generations = parse_num(&next()?, arg)?,
            "--population" => o.cfg.population = parse_num(&next()?, arg)?,
            "--seed" => o.cfg.seed = parse_num(&next()?, arg)?,
            "--mutation-rate" => o.cfg.mutation_rate = parse_float(&next()?, arg)?,
            "--crossover-rate" => o.cfg.crossover_rate = parse_float(&next()?, arg)?,
            "--elitism" => o.cfg.elitism = parse_num(&next()?, arg)?,
            "--tournament-k" => o.cfg.tournament_k = parse_num(&next()?, arg)?,
            "--include-default" => o.include_default = true,
            "--release" => o.release = true,
            "--dry-run" => o.dry_run = true,
            "-h" | "--help" => return Err("help".to_string()),
            other => return Err(format!("unknown option {other:?}")),
        }
    }
    Ok(o)
}

fn parse_num<T: std::str::FromStr>(s: &str, flag: &str) -> Result<T, String> {
    s.parse::<T>()
        .map_err(|_| format!("{flag}: invalid value {s:?}"))
}

fn parse_float(s: &str, flag: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("{flag}: invalid value {s:?}"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let opts = match parse_args(&args) {
        Ok(o) => o,
        Err(e) if e == "help" => {
            print!("{}", usage());
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("confevo: error: {e}\n\n{}", usage());
            return ExitCode::from(2);
        }
    };

    match run(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("confevo: error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(opts: Options) -> Result<(), String> {
    // Build the feature space from the manifest.
    let space = feature_space_from_cargo_toml(&opts.manifest, opts.include_default)
        .map_err(|e| format!("{e} (manifest: {})", opts.manifest.display()))?;

    let mode = if opts.dry_run {
        Mode::DryRun
    } else {
        Mode::Real
    };

    // Pick an evaluator. Preflight cargo in real mode so we fail fast and clearly.
    let mut synthetic;
    let mut cargo;
    let evaluator: &mut dyn Evaluator = if opts.dry_run {
        synthetic = SyntheticEvaluator::generic();
        &mut synthetic
    } else {
        if !CargoEvaluator::cargo_available() {
            return Err(
                "`cargo` was not found on PATH. Install Rust (https://rustup.rs) or \
                 re-run with --dry-run for the synthetic model."
                    .to_string(),
            );
        }
        let manifest_dir = opts
            .manifest
            .parent()
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        cargo = CargoEvaluator::new(manifest_dir);
        if opts.release {
            cargo = cargo.clone().with_extra_args(["--release"]);
        }
        &mut cargo
    };

    let result = run_ga_dyn(evaluator, &space, &opts.cfg).map_err(|e| e.to_string())?;

    // Write reports.
    std::fs::create_dir_all(&opts.out)
        .map_err(|e| format!("creating output dir {}: {e}", opts.out.display()))?;
    let manifest_str = opts.manifest.display().to_string();
    let json = to_json(&result, &opts.cfg, mode, &manifest_str, space.len());
    let md = to_markdown(&result, &opts.cfg, mode, &manifest_str, space.len());
    let json_path = opts.out.join("results.json");
    let md_path = opts.out.join("report.md");
    std::fs::write(&json_path, &json)
        .map_err(|e| format!("writing {}: {e}", json_path.display()))?;
    std::fs::write(&md_path, &md).map_err(|e| format!("writing {}: {e}", md_path.display()))?;

    // Human summary to stdout.
    let tag = if opts.dry_run {
        " [SIMULATED dry-run]"
    } else {
        ""
    };
    println!("confevo: genetic Cargo-feature optimizer{tag}");
    println!("  manifest      : {}", manifest_str);
    println!("  feature space : {} toggleable features", space.len());
    println!(
        "  best config   : {}",
        best_label(&result.best_eval.features)
    );
    println!("  best score    : {:.4}", result.best_eval.score);
    println!("  builds        : {}", result.best_eval.builds);
    println!("  resolves      : {}", result.best_eval.resolves);
    println!("  error_count   : {}", result.best_eval.error_count);
    println!("  evaluations   : {}", result.evaluations);
    println!("  wrote         : {}", json_path.display());
    println!("  wrote         : {}", md_path.display());
    if !result.best_eval.builds {
        println!(
            "  honest finding: NO configuration built. confevo mapped the feature \
             space and surfaced the closest-to-green configuration above."
        );
    }
    Ok(())
}

fn best_label(features: &[String]) -> String {
    if features.is_empty() {
        "<none>".to_string()
    } else {
        features.join(",")
    }
}

/// Run the GA against a `&mut dyn Evaluator`.
///
/// [`run_ga`] is generic over `impl Evaluator`; this thin wrapper lets the CLI
/// pick the concrete evaluator at runtime behind a trait object.
fn run_ga_dyn(
    eval: &mut dyn Evaluator,
    space: &confevo::FeatureSpace,
    cfg: &GaConfig,
) -> Result<confevo::GaResult, confevo::GaError> {
    struct Dyn<'a>(&'a mut dyn Evaluator);
    impl Evaluator for Dyn<'_> {
        fn evaluate(
            &mut self,
            g: &confevo::Genome,
            s: &confevo::FeatureSpace,
        ) -> confevo::EvalResult {
            self.0.evaluate(g, s)
        }
    }
    run_ga(&mut Dyn(eval), space, cfg)
}
