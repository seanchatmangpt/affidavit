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

Actually shells out to `cargo build --no-default-features --lib --features …` for
every genome it evaluates (results are cached by canonical key, so duplicates are
free). It builds the **library target** (`--lib`) — a consistent, binary-agnostic
proxy for "does this feature-set compile":

```bash
python3 tools/confevo/confevo.py run --generations 3 --population 6
```

A real run on this repo bottoms out exactly where the dry-run predicts:

```text
confevo: genetic Cargo-feature optimizer
  best config   : <none>
  best score    : -11.04
  builds        : False
  resolves      : True
  error_count   : 6        # parsed straight from real `cargo ... --message-format=json`
  ...
  honest finding: NO configuration built. ... wasm4pm-compat 26.6.13 fails to compile ...
```

### Invocation forms

Both of these are equivalent (the second runs the package entrypoint,
`__main__.py`):

```bash
python3 tools/confevo/confevo.py run --dry-run        # script form
python3 -m tools.confevo run --dry-run                # module form (from repo root)
```

### Outputs

Both modes write their artifacts to the `--out` directory (default
**`tools/confevo/out/`**, git-ignored):

- `results.json` — machine-readable champion + per-generation history + top configs
- `report.md` — human-readable summary (best config, scores, the honest finding)
- `cache.json` — per-canonical-key evaluation cache (real runs reuse it across invocations)

### Exit codes

| Code | Meaning |
|-----:|---------|
| `0` | run completed (note: "completed" ≠ "a config built" — see the honest finding) |
| `2` | invalid arguments (e.g. `--population 0`, `--mutation-rate 2.0`, `--elitism > --population`) |
| `3` | real mode but `cargo` is not on `PATH` — install Rust or use `--dry-run` |

---

## Robustness & limitations

Things confevo handles deliberately, and the edges it does **not** paper over:

- **No `cargo`?** Real mode preflights with `cargo_available()` and exits `3` with
  a clear message pointing at `--dry-run`; a mid-run spawn failure (`FileNotFoundError`/
  `OSError`) degrades to a strongly-penalized sentinel rather than crashing the search.
- **Build timeouts** (`--timeout`, default 600 s/candidate) count as a non-build with a
  huge error count, so the GA learns to avoid pathologically slow configurations.
- **Degenerate configs** (`population`/`generations` < 1, negative `elitism`, out-of-range
  rates) are rejected up front by both the CLI (exit `2`) and `run_ga` (`ValueError`),
  instead of failing opaquely deep in the loop.
- **The `resolves` signal is a heuristic.** It scans cargo output for genuine
  *dependency-resolution* failures (`failed to select a version`, `no matching package`,
  …) and deliberately ignores the E0433 *compile* error `failed to resolve: …` — so a
  crate that resolved fine but failed to compile is reported as `resolves = True`,
  `builds = False`.
- **`--dry-run` is a model, not a measurement.** The synthetic landscape mirrors the real
  repo (empty → 6 errors, `core` → +550) closely enough to exercise and demo the search,
  but only a real run reflects the actual compiler. Reports made with `--dry-run` are
  labelled **SIMULATED** so the two are never confused.
- **Scope.** confevo toggles only `affidavit`'s own Cargo features (not arbitrary
  dependency versions or `cfg` flags), and evaluates the `--lib` target.

---

## Module map

| File                | Role                                                                       |
|---------------------|----------------------------------------------------------------------------|
| `genome.py`         | The genome: `ALL_FEATURES`, `FEATURE_IMPLICATIONS`, `Genome` (canonical closure, encodings, `random`). |
| `fitness.py`        | `FitnessEvaluator` (real cargo build *or* dry-run synthetic model), `score_from`, `EvalResult`, caching by canonical key. |
| `evolve.py`         | The GA core: `GAConfig`, `run_ga` (tournament selection, uniform crossover, mutation, elitism), `GAResult`/`GenerationRecord`. |
| `confevo.py`        | CLI entrypoint — wires genome + fitness + evolve together, validates args, preflights `cargo`, writes `out/`. |
| `__main__.py`       | Package entrypoint so `python3 -m tools.confevo` runs the CLI. |
| `test_confevo.py`   | `unittest` suite (43 tests) covering all of the above (no cargo — dry-run/stubs/mocks only). |
| `.gitignore`        | Keeps generated `out/` and `__pycache__/` out of version control. |

### Running the tests

The suite is hermetic and fast (no `cargo`, no subprocess — it exercises the
dry-run synthetic model and pure stubs):

```bash
# from the repo root
python3 -m unittest discover -s tools/confevo -p 'test_*.py'

# or run the file directly (it puts its own directory on sys.path)
python3 tools/confevo/test_confevo.py
```
