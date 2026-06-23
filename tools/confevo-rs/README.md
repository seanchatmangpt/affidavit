# confevo (Rust)

**Automatic Cargo feature-flag configuration generation** — a zero-dependency,
`#![forbid(unsafe_code)]` Rust library and CLI that produces a good `--features`
configuration for any crate **two ways**:

- **Numerically**, by *evolving* one with a small genetic algorithm (`confevo run`).
- **Semantically**, by *deriving* one with [wasm4pm](https://github.com/seanchatmangpt/wasm4pm)
  **cognitive breeds** — classical-AI constraint solvers (`confevo solve`).

This is the Rust port of the Python `tools/confevo` reference implementation. It is
**project-agnostic**: it reads the feature universe and the implication edges
straight out of a crate's `Cargo.toml [features]` table — no hand-written feature
list. Like its sibling `affidavit-core`, it carries **zero dependencies**, so it
builds and tests green even though the root `affidavit` crate cannot (its build is
blocked by the broken upstream `wasm4pm-compat 26.6.13`). A tool whose job is to
*map* a broken feature space must itself be immune to that breakage.

## Numeric vs semantic

These are two genuinely different ways to answer "what features should I enable?":

| | Numeric — `confevo run` | Semantic — `confevo solve` |
| --- | --- | --- |
| Representation | a bit-vector genome | boolean variables + logical constraints |
| Method | genetic algorithm (mutate, cross over, score) | cognitive breed: `sat_cdcl` / `csp_ac3` |
| Knows what a feature *means*? | no — it only sees a fitness number | yes — it reasons over the implication graph |
| "Can I have `predictive` without `core`?" | keeps scoring low, never conclusive | returns a **proof of UNSAT** |
| Output | the best-scoring configuration found | a provably valid configuration, or a refutation |

The numeric search is great for *grading* configurations against a real `cargo
build`. The semantic engine is great for *deriving* a valid one and for **proving
impossibility** — the feature-implication graph `a = ["b"]` is literally the clause
`¬a ∨ b`, so generating a configuration is a SAT/CSP problem.

## Cognitive breeds

wasm4pm ships 39 "Old-AI" reasoning systems ("breeds") behind `wpm cognition`, each
invoked as a **contract**: a JSON *intent* in, a JSON *result* (verdict +
explanation + inference trace) out. confevo ports the two whose job is configuration
generation, faithful to that contract format:

- **`sat_cdcl`** — Boolean satisfiability (DPLL with unit propagation and
  conflict-driven backtracking). Implications become clauses, requirements become
  unit clauses; the breed finds a minimal satisfying model or proves none exists.
- **`csp_ac3`** — constraint satisfaction via AC-3 arc consistency plus
  backtracking. Each feature is a variable over `{off, on}`; implications become
  `a => b` constraints; an emptied domain is a proof of infeasibility.

```bash
# Derive a valid configuration that enables `metrics`:
cargo run -- solve --manifest Cargo.toml --breed sat_cdcl --require metrics

# PROVE you cannot enable `predictive` without the broken `core`:
cargo run -- solve --manifest Cargo.toml --require predictive --forbid core
#   verdict     : unsat
#   feasible    : false
#   obstruction : core
#   explanation : required features transitively enable forbidden core …

# Run a breed directly on a wasm4pm-format intent.json (the `wpm cognition` analog):
cargo run -- cognition run --breed csp_ac3 --input intent.json
cargo run -- cognition list
```

The library exposes the same thing:

```rust
use confevo::{feature_space_from_cargo_toml, generate_config, ConfigQuery, Engine};

let space = feature_space_from_cargo_toml("Cargo.toml", false)?;
let query = ConfigQuery::new().require("predictive").forbid("core");
let cfg = generate_config(&space, &query, Engine::SatCdcl)?;
assert!(!cfg.feasible);                     // proved impossible
assert_eq!(cfg.result.selected, "unsat");   // by the breed, with a trace
# Ok::<(), Box<dyn std::error::Error>>(())
```

Each `solve`/`cognition` run is also bit-for-bit deterministic: variable and value
orderings are fixed and the `run_id`/`output_hash` provenance fields are content
hashes (no wall-clock, no randomness).

## What it does

A candidate configuration is a `Genome` (a subset of the crate's features). A
population of genomes is scored by an `Evaluator` and bred forward with:

- **tournament selection**
- **uniform crossover** over the full feature universe
- **per-feature bit-flip mutation**
- **elitism** (the top genomes survive unchanged → best-so-far never regresses)

Fitness prefers configurations that **build**, that **resolve** their dependency
graph, that have **fewer compile errors**, that enable **more features**, and that
finish **faster** — in that priority order.

## Determinism

Every source of randomness flows through one seeded SplitMix64 RNG, and every loop
that consumes it iterates a fixed-order sequence. Same `GaConfig` + `FeatureSpace`
+ `Evaluator` ⇒ bit-for-bit identical results (and identical JSON/Markdown reports).

## CLI

```bash
cargo run --release -- run --manifest path/to/Cargo.toml --dry-run \
    --generations 8 --population 8 --seed 42 --out confevo-out
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--manifest PATH` | `Cargo.toml` | crate manifest to read `[features]` from |
| `--out DIR` | `confevo-out` | where `results.json` + `report.md` are written |
| `--generations N` | 3 | GA generations (also the history length) |
| `--population N` | 6 | population size |
| `--seed N` | 0 | RNG seed (reproducibility) |
| `--mutation-rate F` | 0.1 | per-feature bit-flip probability |
| `--crossover-rate F` | 0.9 | crossover probability |
| `--elitism N` | 1 | elites carried over each generation |
| `--tournament-k N` | 3 | tournament size |
| `--include-default` | off | keep the `default` feature in the search universe |
| `--release` | off | pass `--release` to cargo (real mode only) |
| `--dry-run` | off | use the hermetic synthetic evaluator (no `cargo`) |

`--dry-run` uses the synthetic evaluator: no subprocess, instantaneous, perfect for
CI smoke tests. Without it, `confevo` shells out to real `cargo build
--no-default-features --features <genome> --message-format=json` and counts the
compiler errors/warnings (it preflights that `cargo` is on `PATH`).

## Library

```rust
use confevo::{feature_space_from_cargo_toml, run_ga, GaConfig, SyntheticEvaluator};

let space = feature_space_from_cargo_toml("Cargo.toml", /* include_default = */ false)?;
let mut eval = SyntheticEvaluator::generic();           // or CargoEvaluator::new(dir)
let cfg = GaConfig { seed: 42, generations: 8, ..Default::default() };
let result = run_ga(&mut eval, &space, &cfg)?;

println!("best: {:?} (score {})", result.best_eval.features, result.best_eval.score);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Implement the `Evaluator` trait to plug in your own fitness function (e.g. score by
binary size, `cargo bloat` output, or a remote build service).

## Module map

| Module | Responsibility |
| --- | --- |
| `rng` | seeded, deterministic SplitMix64 RNG |
| `space` | `FeatureSpace`: feature universe + implication edges + transitive closure |
| `manifest` | parse `Cargo.toml [features]` into a `FeatureSpace` (dependency-free) |
| `genome` | `Genome`: an immutable candidate feature-set |
| `fitness` | `Evaluator` trait, scoring, `SyntheticEvaluator`, `CargoEvaluator` |
| `evolve` | `run_ga` — the genetic-algorithm driver (**numeric**) |
| `breeds` | cognitive breeds: `Contract`/`BreedResult`, `sat_cdcl`, `csp_ac3` (**semantic**) |
| `breeds::encode` | bridge a `FeatureSpace` + `ConfigQuery` to a breed and back to a `Genome` |
| `report` | JSON + Markdown rendering |

## The honest finding

For a crate with a structural obstruction below every feature gate (a broken
upstream dependency), **no subset of features can route around it** — and confevo
says so plainly rather than inventing a green build. The synthetic evaluator
reports `builds = true` *only* when its modeled error count reaches zero, so a
poisoned space is never faked. The value of such a run is diagnostic: confevo maps
the feature space and surfaces the configuration that gets *closest* to clean,
which is the best starting point for a fix.

## Tests

```bash
cargo test          # 72 unit + 12 integration + 2 doc tests, all hermetic
cargo clippy --all-targets
cargo fmt --check
```

The breed tests are hermetic too: SAT/CSP solving needs no `cargo` and no
subprocess, so the semantic engine is exercised entirely in-process.

## License

MIT OR Apache-2.0
