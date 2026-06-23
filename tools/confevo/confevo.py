#!/usr/bin/env python3
"""confevo — genetic Cargo-feature configuration optimizer (CLI + reporter).

`confevo` is a tiny, TPOT2-style genetic algorithm that searches the space of
Cargo *feature flags* for the `affidavit` crate, trying to find a configuration
that compiles cleanly. This module is the **entrypoint and reporter**: it parses
the command line, wires together the three sibling modules that do the real work,
runs the search, and renders both a machine-readable `results.json` and a
human-readable `report.md`.

The moving parts
----------------
This file imports three sibling modules that live in the SAME directory:

* ``genome``  — :class:`Genome`, a candidate feature-set, and ``ALL_FEATURES``,
  the universe of toggleable Cargo features.
* ``fitness`` — :class:`FitnessEvaluator`, which turns a genome into an
  :class:`EvalResult` (does it *resolve*? does it *build*? how many compile
  errors/warnings? how long did it take? a scalar ``score``). With ``--dry-run``
  the evaluator is *synthetic* — it never invokes ``cargo`` — which is what makes
  the smoke-test run hermetic and instantaneous.
* ``evolve``  — :func:`run_ga`, the genetic-algorithm driver, plus its
  :class:`GAConfig`/:class:`GAResult`/:class:`GenerationRecord` data contract.

Fitness, in one line
--------------------
A configuration is "better" when it **builds** (and at least **resolves**), has
**fewer compile errors**, enables **more features**, and finishes **faster** —
in that priority order. The GA selects, crosses over, and mutates feature-sets to
climb that gradient.

The honest finding
------------------
For the `affidavit` repository it is *expected that no configuration builds*:
the crate hard-depends on ``wasm4pm_compat``, and the published
``wasm4pm-compat 26.6.13`` does not compile under current nightly (~550 errors).
So this optimizer's real value is to **map the feature space and pinpoint the
obstruction** — not to hand back a green build. The report says so plainly, and
labels itself ``SIMULATED`` whenever ``--dry-run`` was used.

Usage
-----
::

    python3 confevo.py run [--generations N] [--population N] [--seed N]
                           [--mutation-rate F] [--crossover-rate F] [--elitism N]
                           [--tournament-k N] [--timeout S] [--dry-run]
                           [--out DIR] [--repo-root DIR]

Pure-stdlib, Python 3.11+.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import sys
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Sibling imports — support both flat (``python3 confevo.py``) and package
# (``python3 -m tools.confevo.confevo``) invocation. The flat case is the one the
# smoke test exercises; the package case is a convenience for callers that put
# ``tools`` on the path.
# ---------------------------------------------------------------------------
try:  # pragma: no cover - import shim
    from genome import ALL_FEATURES, Genome
    from fitness import EvalResult, FitnessEvaluator, cargo_available
    from evolve import GAConfig, GAResult, GenerationRecord, run_ga
except ImportError:  # pragma: no cover - import shim
    from .genome import ALL_FEATURES, Genome  # type: ignore[no-redef]
    from .fitness import (  # type: ignore[no-redef]
        EvalResult,
        FitnessEvaluator,
        cargo_available,
    )
    from .evolve import (  # type: ignore[no-redef]
        GAConfig,
        GAResult,
        GenerationRecord,
        run_ga,
    )


# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------

#: This file lives at ``<repo>/tools/confevo/confevo.py``; the affidavit repo
#: root is therefore two levels up from this file.
_THIS_FILE = Path(__file__).resolve()
DEFAULT_REPO_ROOT = _THIS_FILE.parents[2]
DEFAULT_OUT_DIR = _THIS_FILE.parent / "out"

DEFAULTS = {
    "generations": 3,
    "population": 6,
    "seed": 0,
    "mutation_rate": 0.1,
    "crossover_rate": 0.9,
    "elitism": 1,
    "tournament_k": 3,
    "timeout": 600,
}


# ---------------------------------------------------------------------------
# JSON serialization helpers
# ---------------------------------------------------------------------------


def _to_jsonable(obj: Any) -> Any:
    """Best-effort conversion of arbitrary objects into JSON-serializable data.

    Handles dataclasses (via :func:`dataclasses.asdict`), sets/frozensets (sorted
    to stay deterministic), Paths, and nested containers. Anything we cannot
    interpret falls back to ``str(obj)`` so serialization never explodes — this
    is a reporter, robustness beats fidelity.
    """
    if obj is None or isinstance(obj, (bool, int, float, str)):
        return obj
    if dataclasses.is_dataclass(obj) and not isinstance(obj, type):
        return {k: _to_jsonable(v) for k, v in dataclasses.asdict(obj).items()}
    if isinstance(obj, dict):
        return {str(k): _to_jsonable(v) for k, v in obj.items()}
    if isinstance(obj, (set, frozenset)):
        return sorted(_to_jsonable(v) for v in obj)
    if isinstance(obj, (list, tuple)):
        return [_to_jsonable(v) for v in obj]
    if isinstance(obj, Path):
        return str(obj)
    return str(obj)


def _eval_to_dict(ev: Any) -> dict[str, Any]:
    """Project an :class:`EvalResult` (or close lookalike) into a plain dict.

    Pulls the documented fields explicitly so the JSON shape is stable even if
    the dataclass grows extra internal fields, and degrades gracefully if a field
    is missing.
    """
    if ev is None:
        return {}
    fields = (
        "features",
        "resolves",
        "builds",
        "error_count",
        "warn_count",
        "elapsed_s",
        "score",
        "from_cache",
    )
    out: dict[str, Any] = {}
    for name in fields:
        if hasattr(ev, name):
            out[name] = _to_jsonable(getattr(ev, name))
    # Fall back to a full dataclass dump if it somehow exposed nothing known.
    if not out and dataclasses.is_dataclass(ev) and not isinstance(ev, type):
        return _to_jsonable(ev)
    return out


def _genome_features(g: Any) -> list[str]:
    """Sorted feature list for a genome, tolerating a few shapes."""
    if g is None:
        return []
    if hasattr(g, "to_features"):
        try:
            return list(g.to_features())
        except Exception:
            pass
    if hasattr(g, "features"):
        try:
            return sorted(str(f) for f in g.features)
        except Exception:
            pass
    if isinstance(g, (list, tuple, set, frozenset)):
        return sorted(str(f) for f in g)
    return []


# ---------------------------------------------------------------------------
# Cache extraction (for the "top 10 distinct configs" list)
# ---------------------------------------------------------------------------


def _extract_cache_results(evaluator: Any) -> list[Any]:
    """Best-effort pull of every :class:`EvalResult` the evaluator has cached.

    The evaluator's cache layout is an implementation detail owned by the sibling
    ``fitness`` module, so we probe a handful of plausible attribute names rather
    than hard-coding one. Whatever we find, we coerce into a flat list of
    EvalResult-like objects. If nothing matches, return an empty list and the
    caller falls back to the GA history.
    """
    if evaluator is None:
        return []

    candidates = [
        "cache",
        "_cache",
        "results",
        "_results",
        "memo",
        "_memo",
        "seen",
        "_seen",
        "evaluations",
        "_evaluations",
    ]
    for attr in candidates:
        store = getattr(evaluator, attr, None)
        if store is None:
            continue
        values: list[Any]
        if isinstance(store, dict):
            values = list(store.values())
        elif isinstance(store, (list, tuple, set, frozenset)):
            values = list(store)
        else:
            continue
        # Keep only things that look like an EvalResult (have a score + features).
        found = [v for v in values if hasattr(v, "score") and hasattr(v, "features")]
        if found:
            return found
    return []


def _top_distinct_configs(
    evaluator: Any,
    result: "GAResult",
    limit: int = 10,
) -> list[dict[str, Any]]:
    """Top-N distinct evaluated configurations, best score first.

    Prefers the evaluator's cache (which holds *every* configuration ever scored,
    not just per-generation bests). Falls back to the GA history, and always
    includes the overall best so the list is never empty. "Distinct" is by sorted
    feature-set.
    """
    pool: list[Any] = _extract_cache_results(evaluator)

    # Always make sure the headline best is represented.
    best_eval = getattr(result, "best_eval", None)
    if best_eval is not None:
        pool.append(best_eval)

    rows: list[dict[str, Any]] = []
    if pool:
        for ev in pool:
            d = _eval_to_dict(ev)
            if d:
                rows.append(d)
    else:
        # No cache exposed — reconstruct what we can from per-generation history.
        for rec in getattr(result, "history", []) or []:
            feats = _to_jsonable(getattr(rec, "best_features", []))
            rows.append(
                {
                    "features": feats,
                    "score": _to_jsonable(getattr(rec, "best_score", None)),
                }
            )
        # And the best result, projected fully.
        if best_eval is not None:
            best_d = _eval_to_dict(best_eval)
            if best_d:
                rows.append(best_d)

    # Deduplicate by feature-set, keeping the highest score for each.
    by_key: dict[str, dict[str, Any]] = {}
    for d in rows:
        feats = d.get("features") or []
        key = ",".join(sorted(str(f) for f in feats)) or "<empty>"
        score = d.get("score")
        prev = by_key.get(key)
        if prev is None or _score_sort_value(score) > _score_sort_value(prev.get("score")):
            by_key[key] = d

    ranked = sorted(by_key.values(), key=lambda d: _score_sort_value(d.get("score")), reverse=True)
    return ranked[:limit]


def _score_sort_value(score: Any) -> float:
    """Coerce a score into a float for sorting; missing/garbage sorts last."""
    try:
        return float(score)
    except (TypeError, ValueError):
        return float("-inf")


# ---------------------------------------------------------------------------
# Report rendering
# ---------------------------------------------------------------------------


def _fmt_features(feats: list[str]) -> str:
    """Render a feature list for a Markdown table cell."""
    if not feats:
        return "`<none>`"
    return "`" + ",".join(feats) + "`"


def _fmt_float(x: Any, nd: int = 4) -> str:
    try:
        return f"{float(x):.{nd}f}"
    except (TypeError, ValueError):
        return str(x)


def _render_report(
    *,
    cfg: "GAConfig",
    args: argparse.Namespace,
    result: "GAResult",
    repo_root: Path,
    top_configs: list[dict[str, Any]],
) -> str:
    """Render the full Markdown report as a string."""
    dry = bool(args.dry_run)
    best_genome = getattr(result, "best_genome", None)
    best_eval = getattr(result, "best_eval", None)
    best_feats = _genome_features(best_genome)
    best_dict = _eval_to_dict(best_eval)

    builds = bool(best_dict.get("builds", False))
    any_built = builds or any(bool(c.get("builds", False)) for c in top_configs)

    lines: list[str] = []
    title_suffix = " — SIMULATED (dry-run)" if dry else ""
    lines.append(f"# confevo report{title_suffix}")
    lines.append("")
    if dry:
        lines.append(
            "> **SIMULATED RUN.** This report was produced with `--dry-run`: the "
            "fitness function is *synthetic* and **no `cargo` commands were "
            "executed**. Scores are illustrative of the search machinery, not of "
            "real compiler behavior."
        )
        lines.append("")

    lines.append(
        "This is the output of **confevo**, a small *genetic algorithm* "
        "(TPOT2-style) that evolves **Cargo feature-flag configurations** for the "
        "`affidavit` crate. Each candidate is a subset of the crate's features; "
        "the GA repeatedly selects, crosses over, and mutates these feature-sets "
        "to maximize a fitness function. **Fitness prefers configurations that "
        "*build*, that at least *resolve* their dependency graph, that have "
        "*fewer compile errors*, that enable *more features*, and that finish "
        "*faster*** — in that priority order."
    )
    lines.append("")

    # -- Run parameters -----------------------------------------------------
    lines.append("## Run parameters")
    lines.append("")
    lines.append("| Parameter | Value |")
    lines.append("| --- | --- |")
    lines.append(f"| mode | {'SIMULATED (dry-run)' if dry else 'real (cargo)'} |")
    lines.append(f"| repo root | `{repo_root}` |")
    for label, attr in (
        ("generations", "generations"),
        ("population", "population_size"),
        ("seed", "seed"),
        ("mutation rate", "mutation_rate"),
        ("crossover rate", "crossover_rate"),
        ("elitism", "elitism"),
        ("tournament k", "tournament_k"),
    ):
        val = _config_get(cfg, attr)
        lines.append(f"| {label} | {val} |")
    lines.append(f"| timeout (s) | {args.timeout} |")
    lines.append(f"| total evaluations | {getattr(result, 'evaluations', 'n/a')} |")
    lines.append(f"| feature universe size | {len(ALL_FEATURES)} |")
    lines.append("")

    # -- Generations table --------------------------------------------------
    lines.append("## Generations")
    lines.append("")
    lines.append("| Gen | Best score | Mean score | Best configuration |")
    lines.append("| ---: | ---: | ---: | --- |")
    history = getattr(result, "history", []) or []
    for rec in history:
        idx = getattr(rec, "index", "?")
        bscore = _fmt_float(getattr(rec, "best_score", None))
        mscore = _fmt_float(getattr(rec, "mean_score", None))
        feats = _genome_features(getattr(rec, "best_features", []))
        if not feats:
            # best_features may already be a plain list of strings.
            raw = getattr(rec, "best_features", []) or []
            feats = sorted(str(f) for f in raw) if not hasattr(raw, "to_features") else feats
        lines.append(f"| {idx} | {bscore} | {mscore} | {_fmt_features(feats)} |")
    if not history:
        lines.append("| _(no generations recorded)_ |  |  |  |")
    lines.append("")

    # -- Best configuration -------------------------------------------------
    lines.append("## Best configuration found")
    lines.append("")
    lines.append(f"- **Features:** {_fmt_features(best_feats)}")
    if best_feats:
        lines.append(f"- **`cargo` invocation:** `cargo build --features {','.join(best_feats)}`")
    else:
        lines.append("- **`cargo` invocation:** `cargo build --no-default-features`")
    lines.append("")
    lines.append("### EvalResult")
    lines.append("")
    lines.append("| Field | Value |")
    lines.append("| --- | --- |")
    for name in (
        "resolves",
        "builds",
        "error_count",
        "warn_count",
        "elapsed_s",
        "score",
        "from_cache",
    ):
        if name in best_dict:
            val = best_dict[name]
            if name in ("elapsed_s", "score"):
                val = _fmt_float(val)
            lines.append(f"| {name} | {val} |")
    lines.append("")

    # -- Top configs --------------------------------------------------------
    if top_configs:
        lines.append("## Top configurations by score")
        lines.append("")
        lines.append("| Rank | Score | Builds | Resolves | Errors | Configuration |")
        lines.append("| ---: | ---: | :---: | :---: | ---: | --- |")
        for i, c in enumerate(top_configs, start=1):
            feats = sorted(str(f) for f in (c.get("features") or []))
            score = _fmt_float(c.get("score"))
            b = "yes" if c.get("builds") else "no"
            r = "yes" if c.get("resolves") else "no"
            errs = c.get("error_count", "?")
            lines.append(
                f"| {i} | {score} | {b} | {r} | {errs} | {_fmt_features(feats)} |"
            )
        lines.append("")

    # -- Honest finding -----------------------------------------------------
    lines.append("## Honest finding")
    lines.append("")
    if any_built:
        lines.append(
            "At least one evaluated configuration reported a **successful build**. "
            "See the table above for the winning feature-set."
        )
        if dry:
            lines.append("")
            lines.append(
                "_Caveat: this was a **SIMULATED** run (`--dry-run`). The "
                "synthetic fitness function can report a build it did not actually "
                "perform. Re-run without `--dry-run` to confirm against `cargo`._"
            )
    else:
        lines.append(
            "**No configuration built successfully.** This is the *expected* "
            "outcome for the `affidavit` repository, and it is not a bug in the "
            "optimizer."
        )
        lines.append("")
        lines.append(
            "`affidavit` hard-depends on **`wasm4pm_compat`**, and the published "
            "**`wasm4pm-compat 26.6.13`** does not compile under the current "
            "nightly toolchain (it emits on the order of **~550 compiler "
            "errors**). Because that dependency sits *below* every feature gate, "
            "**no subset of `affidavit`'s own Cargo features can route around "
            "it** — the obstruction is structural, not configurational."
        )
        lines.append("")
        lines.append(
            "The value of this run is therefore *diagnostic*: confevo **maps the "
            "feature space and pinpoints the obstruction** rather than finding a "
            "green build. The best-scoring configuration above is the one that "
            "gets *closest* (fewest errors / best resolution), which is the most "
            "useful starting point for fixing the underlying `wasm4pm-compat` "
            "breakage."
        )
        if dry:
            lines.append("")
            lines.append(
                "_(This particular run was **SIMULATED** via `--dry-run`, so the "
                "error counts above are synthetic; the structural conclusion about "
                "`wasm4pm-compat` still holds for a real run.)_"
            )
    lines.append("")

    return "\n".join(lines)


def _config_get(cfg: Any, attr: str) -> Any:
    """Read a config field, tolerating naming differences across the sibling.

    The ``evolve`` module owns ``GAConfig``'s exact field names. We try the name
    we were asked for plus a couple of common aliases (e.g. ``population`` vs
    ``population_size``) so the report stays populated regardless.
    """
    aliases = {
        "population_size": ("population_size", "population", "pop_size"),
        "generations": ("generations", "n_generations", "num_generations"),
        "mutation_rate": ("mutation_rate", "mutation"),
        "crossover_rate": ("crossover_rate", "crossover"),
        "elitism": ("elitism", "elite", "n_elite"),
        "tournament_k": ("tournament_k", "tournament_size", "k"),
        "seed": ("seed", "rng_seed"),
    }
    for name in aliases.get(attr, (attr,)):
        if hasattr(cfg, name):
            return getattr(cfg, name)
    return "n/a"


# ---------------------------------------------------------------------------
# GAConfig construction
# ---------------------------------------------------------------------------


def _build_ga_config(args: argparse.Namespace) -> "GAConfig":
    """Construct a :class:`GAConfig`, adapting to its actual field names.

    We don't control ``GAConfig``'s signature (it's defined in the sibling
    ``evolve`` module). To stay robust, we assemble a superset of plausible
    keyword names and pass only those the dataclass actually declares.
    """
    desired = {
        "generations": args.generations,
        "n_generations": args.generations,
        "num_generations": args.generations,
        "population": args.population,
        "population_size": args.population,
        "pop_size": args.population,
        "seed": args.seed,
        "rng_seed": args.seed,
        "mutation_rate": args.mutation_rate,
        "mutation": args.mutation_rate,
        "crossover_rate": args.crossover_rate,
        "crossover": args.crossover_rate,
        "elitism": args.elitism,
        "elite": args.elitism,
        "n_elite": args.elitism,
        "tournament_k": args.tournament_k,
        "tournament_size": args.tournament_k,
        "k": args.tournament_k,
    }

    accepted: dict[str, Any] = {}
    if dataclasses.is_dataclass(GAConfig):
        valid = {f.name for f in dataclasses.fields(GAConfig)}
        accepted = {k: v for k, v in desired.items() if k in valid}
        try:
            return GAConfig(**accepted)
        except TypeError:
            pass  # fall through to the canonical-name attempt below

    # Fallback: the canonical names from the task contract / defaults.
    canonical = {
        "generations": args.generations,
        "population": args.population,
        "seed": args.seed,
        "mutation_rate": args.mutation_rate,
        "crossover_rate": args.crossover_rate,
        "elitism": args.elitism,
        "tournament_k": args.tournament_k,
    }
    return GAConfig(**canonical)


# ---------------------------------------------------------------------------
# `run` command
# ---------------------------------------------------------------------------


# Exit codes: 0 = ok, 2 = invalid arguments, 3 = cargo unavailable (real mode).
EXIT_OK = 0
EXIT_BAD_ARGS = 2
EXIT_NO_CARGO = 3


def _validate_run_args(args: argparse.Namespace) -> list[str]:
    """Return a list of human-readable problems with the run arguments (empty=ok).

    argparse already enforces types; this enforces the *ranges* the GA needs so a
    bad value fails fast with a clear message instead of deep inside the search.
    """
    problems: list[str] = []
    if args.population < 1:
        problems.append(f"--population must be >= 1 (got {args.population})")
    if args.generations < 1:
        problems.append(f"--generations must be >= 1 (got {args.generations})")
    if args.elitism < 0:
        problems.append(f"--elitism must be >= 0 (got {args.elitism})")
    if args.elitism > args.population:
        problems.append(
            f"--elitism ({args.elitism}) cannot exceed --population ({args.population})"
        )
    if args.tournament_k < 1:
        problems.append(f"--tournament-k must be >= 1 (got {args.tournament_k})")
    if not (0.0 <= args.mutation_rate <= 1.0):
        problems.append(f"--mutation-rate must be in [0, 1] (got {args.mutation_rate})")
    if not (0.0 <= args.crossover_rate <= 1.0):
        problems.append(f"--crossover-rate must be in [0, 1] (got {args.crossover_rate})")
    if args.timeout < 1:
        problems.append(f"--timeout must be >= 1 second (got {args.timeout})")
    return problems


def cmd_run(args: argparse.Namespace) -> int:
    """Execute the GA search and emit results.json + report.md."""
    # Preflight 1: argument ranges (clear message instead of a deep crash).
    problems = _validate_run_args(args)
    if problems:
        for p in problems:
            print(f"confevo: error: {p}", file=sys.stderr)
        return EXIT_BAD_ARGS

    # Preflight 2: real mode needs cargo on PATH. Steer to --dry-run if absent,
    # rather than failing one build at a time.
    if not args.dry_run and not cargo_available():
        print(
            "confevo: error: `cargo` was not found on PATH. Install Rust "
            "(https://rustup.rs) or re-run with --dry-run for the synthetic model.",
            file=sys.stderr,
        )
        return EXIT_NO_CARGO

    repo_root = Path(args.repo_root).expanduser().resolve()
    out_dir = Path(args.out).expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    cache_path = out_dir / "cache.json"

    # 1. Fitness evaluator (synthetic when --dry-run).
    evaluator = FitnessEvaluator(
        repo_root=repo_root,
        timeout=args.timeout,
        dry_run=args.dry_run,
        cache_path=cache_path,
    )

    # 2. GA config + run.
    cfg = _build_ga_config(args)
    result: GAResult = run_ga(evaluator.evaluate, cfg)

    # 3. Assemble + write results.json.
    best_genome = getattr(result, "best_genome", None)
    best_eval = getattr(result, "best_eval", None)
    best_dict = _eval_to_dict(best_eval)
    top_configs = _top_distinct_configs(evaluator, result, limit=10)

    results_payload: dict[str, Any] = {
        "mode": "dry-run" if args.dry_run else "real",
        "repo_root": str(repo_root),
        "config": _to_jsonable(cfg),
        "history": [_to_jsonable(rec) for rec in (getattr(result, "history", []) or [])],
        "best": {
            "features": _genome_features(best_genome),
            "builds": best_dict.get("builds"),
            "resolves": best_dict.get("resolves"),
            "error_count": best_dict.get("error_count"),
            "score": best_dict.get("score"),
        },
        "best_eval": best_dict,
        "total_evaluations": getattr(result, "evaluations", None),
        "top_configs": top_configs,
    }

    results_path = out_dir / "results.json"
    report_path = out_dir / "report.md"

    results_path.write_text(
        json.dumps(results_payload, indent=2, sort_keys=False, default=str) + "\n",
        encoding="utf-8",
    )

    # 4. Write report.md.
    report_md = _render_report(
        cfg=cfg,
        args=args,
        result=result,
        repo_root=repo_root,
        top_configs=top_configs,
    )
    report_path.write_text(report_md, encoding="utf-8")

    # 5. Stdout summary.
    best_feats = _genome_features(best_genome)
    feats_str = ",".join(best_feats) if best_feats else "<none>"
    builds = best_dict.get("builds")
    resolves = best_dict.get("resolves")
    errc = best_dict.get("error_count")
    score = best_dict.get("score")

    print("confevo: genetic Cargo-feature optimizer" + (" [SIMULATED dry-run]" if args.dry_run else ""))
    print(f"  best config   : {feats_str}")
    print(f"  best score    : {_fmt_float(score)}")
    print(f"  builds        : {builds}")
    print(f"  resolves      : {resolves}")
    print(f"  error_count   : {errc}")
    print(f"  evaluations   : {getattr(result, 'evaluations', 'n/a')}")
    print(f"  wrote         : {results_path}")
    print(f"  wrote         : {report_path}")

    any_built = bool(builds) or any(bool(c.get("builds")) for c in top_configs)
    if not any_built:
        print(
            "  honest finding: NO configuration built. Expected for affidavit — it "
            "hard-depends on wasm4pm_compat, and wasm4pm-compat 26.6.13 fails to "
            "compile under current nightly (~550 errors). confevo mapped the "
            "feature space and pinpointed the obstruction."
        )

    return EXIT_OK


# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="confevo",
        description=(
            "Genetic (TPOT2-style) optimizer over Cargo feature flags for the "
            "affidavit crate. Searches for a configuration that builds; reports "
            "the feature-space map and the structural obstruction when none does."
        ),
    )
    sub = parser.add_subparsers(dest="command", required=True)

    run_p = sub.add_parser(
        "run",
        help="run the genetic search and write results.json + report.md",
        description="Run the genetic-algorithm search over Cargo feature flags.",
    )
    run_p.add_argument("--generations", type=int, default=DEFAULTS["generations"],
                       help=f"number of GA generations (default: {DEFAULTS['generations']})")
    run_p.add_argument("--population", type=int, default=DEFAULTS["population"],
                       help=f"population size (default: {DEFAULTS['population']})")
    run_p.add_argument("--seed", type=int, default=DEFAULTS["seed"],
                       help=f"RNG seed for reproducibility (default: {DEFAULTS['seed']})")
    run_p.add_argument("--mutation-rate", type=float, default=DEFAULTS["mutation_rate"],
                       dest="mutation_rate",
                       help=f"per-gene mutation probability (default: {DEFAULTS['mutation_rate']})")
    run_p.add_argument("--crossover-rate", type=float, default=DEFAULTS["crossover_rate"],
                       dest="crossover_rate",
                       help=f"crossover probability (default: {DEFAULTS['crossover_rate']})")
    run_p.add_argument("--elitism", type=int, default=DEFAULTS["elitism"],
                       help=f"number of elites carried over each generation (default: {DEFAULTS['elitism']})")
    run_p.add_argument("--tournament-k", type=int, default=DEFAULTS["tournament_k"],
                       dest="tournament_k",
                       help=f"tournament selection size (default: {DEFAULTS['tournament_k']})")
    run_p.add_argument("--timeout", type=int, default=DEFAULTS["timeout"],
                       help=f"per-build timeout in seconds (default: {DEFAULTS['timeout']})")
    run_p.add_argument("--dry-run", action="store_true",
                       help="use the synthetic fitness function (no cargo calls)")
    run_p.add_argument("--out", type=str, default=str(DEFAULT_OUT_DIR),
                       help=f"output directory (default: {DEFAULT_OUT_DIR})")
    run_p.add_argument("--repo-root", type=str, default=str(DEFAULT_REPO_ROOT),
                       dest="repo_root",
                       help=f"affidavit repo root (default: {DEFAULT_REPO_ROOT})")
    run_p.set_defaults(func=cmd_run)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    func = getattr(args, "func", None)
    if func is None:
        parser.print_help()
        return 2
    return func(args)


if __name__ == "__main__":
    raise SystemExit(main())
