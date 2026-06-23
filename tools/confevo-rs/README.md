# confevo (Rust)

**Automatic Cargo feature-flag configuration generation** — a zero-dependency,
`#![forbid(unsafe_code)]` Rust library and CLI that evolves a good `--features`
configuration for any crate using a small genetic algorithm.

This is the Rust port of the Python `tools/confevo` reference implementation. It is
**project-agnostic**: it reads the feature universe and the implication edges
straight out of a crate's `Cargo.toml [features]` table — no hand-written feature
list. Like its sibling `affidavit-core`, it carries **zero dependencies**, so it
builds and tests green even though the root `affidavit` crate cannot (its build is
blocked by the broken upstream `wasm4pm-compat 26.6.13`). A tool whose job is to
*map* a broken feature space must itself be immune to that breakage.

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
| `evolve` | `run_ga` — the genetic-algorithm driver |
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
cargo test          # 43 unit + 5 integration + 2 doc tests, all hermetic
cargo clippy --all-targets
cargo fmt --check
```

## License

MIT OR Apache-2.0
