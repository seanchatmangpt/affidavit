//! `confevo` — CLI driver for Cargo feature-flag configuration generation.
//!
//! Three subcommands, two paradigms:
//!
//! ```text
//! confevo run       [OPTIONS]   # NUMERIC: genetic-algorithm search
//! confevo solve     [OPTIONS]   # SEMANTIC: cognitive-breed reasoning over a manifest
//! confevo cognition <run|list>  # raw wasm4pm-style breed contract runner
//! ```
//!
//! * `run` reads the `[features]` table and *evolves* a configuration with the GA,
//!   writing `results.json` + `report.md`. With `--dry-run` it uses the hermetic
//!   synthetic evaluator; otherwise it shells out to real `cargo build`.
//! * `solve` *derives* a provably valid configuration by encoding the feature graph
//!   as a SAT/CSP instance and running a cognitive **breed** (`sat_cdcl` /
//!   `csp_ac3`). It can prove a `--require`/`--forbid` query infeasible.
//! * `cognition` runs a breed directly on a wasm4pm `intent.json` contract — the
//!   `wpm cognition run` analog.

use std::path::PathBuf;
use std::process::ExitCode;

use confevo::breeds::{run_named, supported_breeds};
use confevo::report::{to_json, to_markdown, Mode};
use confevo::{
    feature_space_from_cargo_toml, generate_config, run_ga, CargoEvaluator, ConfigQuery, Engine,
    Evaluator, GaConfig, Genome, SyntheticEvaluator,
};

fn usage() -> &'static str {
    "\
confevo — Cargo feature-flag configuration generation (numeric + semantic)

USAGE:
    confevo run [OPTIONS]            evolve a configuration (genetic algorithm)
    confevo solve [OPTIONS]          derive a configuration (cognitive breeds: SAT/CSP)
    confevo cognition run [OPTIONS]  run a breed on a wasm4pm intent.json contract
    confevo cognition list           list the implemented cognitive breeds

Run `confevo <command> --help` for command-specific options.
"
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (c.as_str(), r),
        None => {
            print!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let result = match cmd {
        "run" => cmd_run(rest),
        "solve" => cmd_solve(rest),
        "cognition" => cmd_cognition(rest),
        "-h" | "--help" => {
            print!("{}", usage());
            return ExitCode::SUCCESS;
        }
        other => Err(format!(
            "unknown command {other:?} (expected `run`, `solve`, or `cognition`)"
        )),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e == "help" => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("confevo: error: {e}");
            ExitCode::from(2)
        }
    }
}

// ---------------------------------------------------------------------------
// Small shared argument helpers
// ---------------------------------------------------------------------------

fn parse_num<T: std::str::FromStr>(s: &str, flag: &str) -> Result<T, String> {
    s.parse::<T>()
        .map_err(|_| format!("{flag}: invalid value {s:?}"))
}

fn parse_float(s: &str, flag: &str) -> Result<f64, String> {
    s.parse::<f64>()
        .map_err(|_| format!("{flag}: invalid value {s:?}"))
}

fn best_label(features: &[String]) -> String {
    if features.is_empty() {
        "<none>".to_string()
    } else {
        features.join(",")
    }
}

// ---------------------------------------------------------------------------
// `run` — genetic algorithm (numeric search)
// ---------------------------------------------------------------------------

struct RunOptions {
    manifest: PathBuf,
    out: PathBuf,
    cfg: GaConfig,
    dry_run: bool,
    include_default: bool,
    release: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        RunOptions {
            manifest: PathBuf::from("Cargo.toml"),
            out: PathBuf::from("confevo-out"),
            cfg: GaConfig::default(),
            dry_run: false,
            include_default: false,
            release: false,
        }
    }
}

fn run_usage() -> &'static str {
    "\
confevo run — genetic optimizer for Cargo feature-flag configurations

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

fn cmd_run(args: &[String]) -> Result<(), String> {
    let mut o = RunOptions::default();
    let mut it = args.iter();
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
            "-h" | "--help" => {
                print!("{}", run_usage());
                return Err("help".to_string());
            }
            other => return Err(format!("unknown option {other:?}\n\n{}", run_usage())),
        }
    }

    let space = feature_space_from_cargo_toml(&o.manifest, o.include_default)
        .map_err(|e| format!("{e} (manifest: {})", o.manifest.display()))?;

    let mode = if o.dry_run { Mode::DryRun } else { Mode::Real };

    let mut synthetic;
    let mut cargo;
    let evaluator: &mut dyn Evaluator = if o.dry_run {
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
        let manifest_dir = o
            .manifest
            .parent()
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        cargo = CargoEvaluator::new(manifest_dir);
        if o.release {
            cargo = cargo.clone().with_extra_args(["--release"]);
        }
        &mut cargo
    };

    let result = run_ga_dyn(evaluator, &space, &o.cfg).map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&o.out)
        .map_err(|e| format!("creating output dir {}: {e}", o.out.display()))?;
    let manifest_str = o.manifest.display().to_string();
    let json = to_json(&result, &o.cfg, mode, &manifest_str, space.len());
    let md = to_markdown(&result, &o.cfg, mode, &manifest_str, space.len());
    let json_path = o.out.join("results.json");
    let md_path = o.out.join("report.md");
    std::fs::write(&json_path, &json)
        .map_err(|e| format!("writing {}: {e}", json_path.display()))?;
    std::fs::write(&md_path, &md).map_err(|e| format!("writing {}: {e}", md_path.display()))?;

    let tag = if o.dry_run {
        " [SIMULATED dry-run]"
    } else {
        ""
    };
    println!("confevo run: genetic Cargo-feature optimizer{tag}");
    println!("  manifest      : {manifest_str}");
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

/// Run the GA against a `&mut dyn Evaluator` (the generic `run_ga` behind a trait
/// object so the CLI can pick the evaluator at runtime).
fn run_ga_dyn(
    eval: &mut dyn Evaluator,
    space: &confevo::FeatureSpace,
    cfg: &GaConfig,
) -> Result<confevo::GaResult, confevo::GaError> {
    struct Dyn<'a>(&'a mut dyn Evaluator);
    impl Evaluator for Dyn<'_> {
        fn evaluate(&mut self, g: &Genome, s: &confevo::FeatureSpace) -> confevo::EvalResult {
            self.0.evaluate(g, s)
        }
    }
    run_ga(&mut Dyn(eval), space, cfg)
}

// ---------------------------------------------------------------------------
// `solve` — cognitive breeds (semantic generation)
// ---------------------------------------------------------------------------

struct SolveOptions {
    manifest: PathBuf,
    out: Option<PathBuf>,
    engine: Engine,
    query: ConfigQuery,
    include_default: bool,
    evaluate: bool,
    dry_run: bool,
}

impl Default for SolveOptions {
    fn default() -> Self {
        SolveOptions {
            manifest: PathBuf::from("Cargo.toml"),
            out: None,
            engine: Engine::SatCdcl,
            query: ConfigQuery::new(),
            include_default: false,
            evaluate: false,
            dry_run: false,
        }
    }
}

fn solve_usage() -> &'static str {
    "\
confevo solve — derive a valid configuration by symbolic reasoning (cognitive breeds)

Encodes the crate's feature-implication graph as a SAT/CSP instance and runs a
wasm4pm cognitive breed to DERIVE a provably valid configuration — or to PROVE
that your require/forbid query is impossible.

OPTIONS:
    --manifest PATH    Cargo.toml to read [features] from        (default: Cargo.toml)
    --breed ID         sat_cdcl | csp_ac3 (aliases: sat, csp)    (default: sat_cdcl)
    --require FEATURE   feature that must be enabled  (repeatable)
    --forbid FEATURE    feature that must be disabled (repeatable)
    --include-default  keep the `default` feature in the universe
    --out DIR          write cognition.json + config.md here     (default: none)
    --evaluate         after deriving, build the config with cargo to confirm
    --dry-run          with --evaluate, use the synthetic evaluator (no cargo)
    -h, --help         print this help
"
}

fn cmd_solve(args: &[String]) -> Result<(), String> {
    let mut o = SolveOptions::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut next = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--manifest" => o.manifest = PathBuf::from(next()?),
            "--breed" => {
                let id = next()?;
                o.engine = Engine::from_id(&id).ok_or_else(|| {
                    format!(
                        "unknown breed {id:?}; choose from: {}",
                        supported_breeds().join(", ")
                    )
                })?;
            }
            "--require" => o.query.require.push(next()?),
            "--forbid" => o.query.forbid.push(next()?),
            "--include-default" => o.include_default = true,
            "--out" => o.out = Some(PathBuf::from(next()?)),
            "--evaluate" => o.evaluate = true,
            "--dry-run" => o.dry_run = true,
            "-h" | "--help" => {
                print!("{}", solve_usage());
                return Err("help".to_string());
            }
            other => return Err(format!("unknown option {other:?}\n\n{}", solve_usage())),
        }
    }

    let space = feature_space_from_cargo_toml(&o.manifest, o.include_default)
        .map_err(|e| format!("{e} (manifest: {})", o.manifest.display()))?;

    let cfg = generate_config(&space, &o.query, o.engine)?;

    println!("confevo solve: semantic configuration generation");
    println!("  manifest      : {}", o.manifest.display());
    println!("  breed         : {}", cfg.engine_id);
    println!("  feature space : {} toggleable features", space.len());
    println!("  require       : {}", best_label(&o.query.require));
    println!("  forbid        : {}", best_label(&o.query.forbid));
    println!("  verdict       : {}", cfg.result.selected);
    println!("  feasible      : {}", cfg.feasible);

    if let Some(g) = &cfg.genome {
        let feats = g.features();
        println!("  configuration : {}", best_label(&feats));
        if feats.is_empty() {
            println!("  cargo         : cargo build --no-default-features");
        } else {
            println!(
                "  cargo         : cargo build --no-default-features --features {}",
                feats.join(",")
            );
        }
    } else {
        println!("  configuration : <none — query is infeasible>");
        if !cfg.clash.is_empty() {
            println!("  obstruction   : {}", cfg.clash.join(","));
        }
    }
    println!("  explanation   : {}", cfg.explanation);
    println!(
        "  inference     : {} steps, {} rules",
        cfg.result.inference_step_count, cfg.result.rules_evaluated
    );

    // Optional: confirm the derived configuration actually builds.
    if o.evaluate {
        if let Some(g) = &cfg.genome {
            let ev = evaluate_genome(g, &space, &o.manifest, o.dry_run)?;
            println!(
                "  evaluated     : builds={} resolves={} errors={}{}",
                ev.builds,
                ev.resolves,
                ev.error_count,
                if o.dry_run { " [SIMULATED]" } else { "" }
            );
        } else {
            println!("  evaluated     : skipped (no configuration to build)");
        }
    }

    if let Some(dir) = &o.out {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("creating output dir {}: {e}", dir.display()))?;
        let json_path = dir.join("cognition.json");
        let md_path = dir.join("config.md");
        std::fs::write(&json_path, cfg.result.to_json())
            .map_err(|e| format!("writing {}: {e}", json_path.display()))?;
        std::fs::write(&md_path, solve_markdown(&o, &cfg))
            .map_err(|e| format!("writing {}: {e}", md_path.display()))?;
        println!("  wrote         : {}", json_path.display());
        println!("  wrote         : {}", md_path.display());
    }

    Ok(())
}

/// Evaluate a derived genome, either synthetically (`--dry-run`) or via cargo.
fn evaluate_genome(
    g: &Genome,
    space: &confevo::FeatureSpace,
    manifest: &std::path::Path,
    dry_run: bool,
) -> Result<confevo::EvalResult, String> {
    if dry_run {
        let mut e = SyntheticEvaluator::generic();
        Ok(e.evaluate(g, space))
    } else {
        if !CargoEvaluator::cargo_available() {
            return Err(
                "`cargo` not found on PATH for --evaluate; re-run with --dry-run.".to_string(),
            );
        }
        let dir = manifest
            .parent()
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut e = CargoEvaluator::new(dir);
        Ok(e.evaluate(g, space))
    }
}

fn solve_markdown(o: &SolveOptions, cfg: &confevo::SemanticConfig) -> String {
    let mut m = String::new();
    m.push_str("# confevo solve — semantic configuration\n\n");
    m.push_str(
        "Derived by a **cognitive breed** (symbolic reasoning over the feature graph), \
         not by genetic search. The breed treats each feature as a logical variable and \
         each implication `a = [\"b\"]` as a constraint, then either derives a provably \
         valid configuration or proves the query impossible.\n\n",
    );
    m.push_str("| Field | Value |\n| --- | --- |\n");
    m.push_str(&format!("| manifest | `{}` |\n", o.manifest.display()));
    m.push_str(&format!("| breed | `{}` |\n", cfg.engine_id));
    m.push_str(&format!("| require | {} |\n", best_label(&o.query.require)));
    m.push_str(&format!("| forbid | {} |\n", best_label(&o.query.forbid)));
    m.push_str(&format!("| verdict | {} |\n", cfg.result.selected));
    m.push_str(&format!("| feasible | {} |\n", cfg.feasible));
    m.push_str(&format!(
        "| inference steps | {} |\n",
        cfg.result.inference_step_count
    ));
    m.push_str(&format!(
        "| rules evaluated | {} |\n\n",
        cfg.result.rules_evaluated
    ));

    m.push_str("## Outcome\n\n");
    if let Some(g) = &cfg.genome {
        m.push_str(&format!(
            "- **Configuration:** `{}`\n",
            best_label(&g.features())
        ));
    } else {
        m.push_str("- **Configuration:** _none — the query is infeasible_\n");
        if !cfg.clash.is_empty() {
            m.push_str(&format!(
                "- **Obstruction:** required features transitively enable forbidden `{}`\n",
                cfg.clash.join(",")
            ));
        }
    }
    m.push_str(&format!("- {}\n", cfg.explanation));
    m
}

// ---------------------------------------------------------------------------
// `cognition` — raw breed contract runner (wasm4pm-faithful)
// ---------------------------------------------------------------------------

fn cognition_usage() -> &'static str {
    "\
confevo cognition — run an Old-AI cognitive breed on a wasm4pm contract

USAGE:
    confevo cognition run --breed ID --input intent.json [--out result.json]
    confevo cognition list

The `run` form reads a wasm4pm-format intent.json (an object with `intent` and a
`facts` array of {key, value}) and prints the breed's result.json to stdout (or to
--out). `list` prints the implemented breeds.

OPTIONS (run):
    --breed ID     sat_cdcl | csp_ac3
    --input PATH   path to the intent.json contract
    --out PATH     write result.json here (default: stdout)
    -h, --help     print this help
"
}

fn cmd_cognition(args: &[String]) -> Result<(), String> {
    let (sub, rest) = match args.split_first() {
        Some((s, r)) => (s.as_str(), r),
        None => {
            print!("{}", cognition_usage());
            return Err("help".to_string());
        }
    };

    match sub {
        "list" => {
            println!("confevo implements these cognitive breeds:");
            for b in supported_breeds() {
                println!("  {b}");
            }
            Ok(())
        }
        "run" => cmd_cognition_run(rest),
        "-h" | "--help" => {
            print!("{}", cognition_usage());
            Err("help".to_string())
        }
        other => Err(format!(
            "unknown cognition subcommand {other:?} (expected `run` or `list`)"
        )),
    }
}

fn cmd_cognition_run(args: &[String]) -> Result<(), String> {
    let mut breed: Option<String> = None;
    let mut input: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut next = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--breed" => breed = Some(next()?),
            "--input" => input = Some(PathBuf::from(next()?)),
            "--out" => out = Some(PathBuf::from(next()?)),
            "-h" | "--help" => {
                print!("{}", cognition_usage());
                return Err("help".to_string());
            }
            other => return Err(format!("unknown option {other:?}\n\n{}", cognition_usage())),
        }
    }

    let breed = breed.ok_or("cognition run requires --breed")?;
    let input = input.ok_or("cognition run requires --input")?;

    let text =
        std::fs::read_to_string(&input).map_err(|e| format!("reading {}: {e}", input.display()))?;
    let contract = confevo::Contract::from_json(&text)
        .map_err(|e| format!("parsing {}: {e}", input.display()))?;
    let result = run_named(&breed, &contract)?;
    let json = result.to_json();

    match out {
        Some(path) => {
            std::fs::write(&path, &json).map_err(|e| format!("writing {}: {e}", path.display()))?;
            println!(
                "confevo cognition: breed={} selected={} -> {}",
                breed,
                result.selected,
                path.display()
            );
        }
        None => print!("{json}"),
    }
    Ok(())
}
