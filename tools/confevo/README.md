# confevo — genetic Cargo-feature configuration optimizer

`confevo` is a small **genetic algorithm** (evolutionary search) that optimizes
the `affidavit` crate's **Cargo feature-flag configuration**. It treats a *set of
enabled Cargo features* as a genome, scores each candidate by how well it builds,
and breeds successive generations toward higher-scoring configurations.

It is the **honest analog of [TPOT2](https://github.com/EpistasisLab/tpot)**.
TPOT2 uses genetic programming to evolve machine-learning *pipelines*; confevo
uses the same evolutionary machinery to evolve Cargo *feature-sets*. Same spirit,
different search space:

| | TPOT2 | confevo |
|------------------|----------------------------------|-----------------------------------|
| Genome           | an ML pipeline (operators)       | a subset of Cargo features        |
| Fitness          | cross-validated model score      | build outcome of `cargo build`    |
| Search           | genetic programming              | genetic algorithm                 |
| Output           | the best pipeline                | the best feature configuration    |

This is a **real GA**, not a fixed script. It maintains a *population*, ranks
candidates by fitness, and breeds the next generation via **tournament
selection**, **uniform crossover**, **per-feature bit-flip mutation**, and
**elitism** (the top genomes survive unchanged so the best score never regresses).
Every source of randomness flows through a single seeded `random.Random`, so a run
is **bit-for-bit reproducible** given its `GAConfig`.

confevo is written in **Python (standard library only)**. That is deliberate: it
runs and reports *regardless of whether the Rust build succeeds* — exactly the
property that lets it map a broken build's feature space, and the same reason
TPOT2 is a Python harness around the thing it searches.

---

## The genome

A **genome** is one candidate configuration: a `frozenset` of feature names drawn
from the crate's 21 features (`core`, `discovery`, `conformance`, `predictive`,
`ui`, `json-output`, `shell`, `otel`, `metrics`, `lsp`, …). It corresponds
directly to the flags you would pass:

```
cargo build --no-default-features --features core,discovery,ui
```

Cargo features enable other features, so confevo models a **feature-implication
graph** (`discovery → core`, `conformance → discovery`, `predictive →
conformance`, `metrics → otel`, `profiling → benchmarking`,
`{file-watch, quality-monitor, webhook} → shell`). `Genome.canonical()` computes
the **transitive closure** of those implications, so two genomes that describe the
*same effective build* collapse to one representative (`.key()`). The GA and the
fitness cache deduplicate on that canonical key.

## The fitness function

Each genome is scored by attempting (or simulating) a build and folding the
signals into a single number — higher is better:

```
score =  +1000   if the crate BUILDS (compiles cleanly)
         +50     if dependencies RESOLVE
         -1      per compiler ERROR
         +2      per enabled FEATURE        (breadth bonus)
         -0.1    per SECOND of wall-clock    (cheaper is better)
```

Why this shape? Because **even when nothing compiles, the score still has a
gradient.** A configuration that resolves its dependencies and produces *fewer*
compiler errors scores higher than one that produces *more*. That lets the GA
climb a meaningful slope and localize *which* features make the build worse —
instead of seeing a flat "everything fails" landscape with no signal. The build
bonus (`+1000`) dominates if anything ever compiles, but the
`resolves / -errors / +features / -seconds` terms shape the search the rest of the
time.

## The honest finding

confevo's verdict on this repository is blunt: **no feature configuration builds.**

`affidavit` hard-depends on `wasm4pm-compat` — `core = ["wasm4pm-compat"]` in
`Cargo.toml`, and `default = ["core"]` — and the published
`wasm4pm-compat 26.6.13` **fails to compile under the current Rust nightly**
(~550 errors). Because `core` pulls in that broken crate, and because
`discovery`/`conformance`/`predictive` all imply `core` transitively, every
configuration that touches the core surface explodes. Configurations that avoid
`core` still don't build (the crate references `wasm4pm_compat` paths
unconditionally for a residual ~6 errors), they just fail *less*.

confevo does not hide this behind a green checkmark. It **maps the space and
localizes the obstruction**: the search consistently drives *away* from `core`
(lower error count → higher score), and the champion it reports is whichever
honest configuration fails least — while truthfully recording `builds = False`.
Certify, don't decide: confevo reports the build outcome; it does not pretend a
broken build is a working one.

Because it is pure-Python stdlib, confevo produces that map **even though the Rust
build is broken** — there is no chicken-and-egg dependency on the very thing it is
diagnosing.

---

## Usage

### Simulated run (instant, no cargo)

Uses the deterministic synthetic fitness model — perfect for demos, CI, and
understanding the landscape without waiting on the compiler:

```bash
python3 tools/confevo/confevo.py run --dry-run --generations 3 --population 6 --seed 1
```

### Real run (invokes cargo per candidate, slower)

Actually shells out to `cargo build --no-default-features --features …` for every
genome it evaluates (results are cached by canonical key, so duplicates are free):

```bash
python3 tools/confevo/confevo.py run --generations 3 --population 6
```

### Outputs

Both modes write their artifacts to **`tools/confevo/out/`**:

- `tools/confevo/out/results.json` — machine-readable champion + per-generation history
- `tools/confevo/out/report.md` — human-readable summary (best config, scores, the finding)

---

## Module map

| File                | Role                                                                       |
|---------------------|----------------------------------------------------------------------------|
| `genome.py`         | The genome: `ALL_FEATURES`, `FEATURE_IMPLICATIONS`, `Genome` (canonical closure, encodings, `random`). |
| `fitness.py`        | `FitnessEvaluator` (real cargo build *or* dry-run synthetic model), `score_from`, `EvalResult`, caching by canonical key. |
| `evolve.py`         | The GA core: `GAConfig`, `run_ga` (tournament selection, uniform crossover, mutation, elitism), `GAResult`/`GenerationRecord`. |
| `confevo.py`        | CLI entrypoint — wires genome + fitness + evolve together and writes `out/`. |
| `test_confevo.py`   | `unittest` suite covering all of the above (no cargo — dry-run/stubs only). |

### Running the tests

The suite is hermetic and fast (no `cargo`, no subprocess — it exercises the
dry-run synthetic model and pure stubs):

```bash
# from the repo root
python3 -m unittest discover -s tools/confevo -p 'test_*.py'

# or run the file directly (it puts its own directory on sys.path)
python3 tools/confevo/test_confevo.py
```
