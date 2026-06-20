"""confevo.evolve — the genetic-algorithm core of the Cargo-feature optimizer.

This module is a tiny, TPOT2-style evolutionary search, but instead of evolving
machine-learning pipelines it evolves **Cargo feature-sets** for the `affidavit`
crate. A *genome* (see :mod:`genome`) is one candidate configuration — a subset of
``ALL_FEATURES``. The GA maintains a population of genomes, scores each with an
injected evaluator, and breeds the next generation via tournament selection,
uniform crossover, and per-feature bit-flip mutation, preserving the best few
genomes unchanged (elitism).

Design notes
------------
* **Evaluator injection.** The fitness function is passed in as ``eval_fn`` and is
  only ever duck-typed through the returned object's ``.score`` (float, higher is
  better) and ``.features`` (list[str]) attributes. We deliberately do *not* import
  ``fitness`` so the GA stays unit-testable with a synthetic stub — see the
  ``__main__`` smoke block at the bottom of this file.
* **Determinism.** All randomness flows through a single ``random.Random(cfg.seed)``
  instance, and every loop that consumes randomness iterates a *fixed-order*
  sequence (``ALL_FEATURES`` for crossover/mutation, list indices for selection).
  Given the same ``GAConfig`` the run is bit-for-bit reproducible.
* **Caching.** Within a single :func:`run_ga` call, evaluation results are memoized
  by ``genome.canonical().key()``, so two genomes that describe the *same effective
  build* are evaluated once. ``GAResult.evaluations`` counts only the real
  ``eval_fn`` calls that actually happened.
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from typing import Callable

try:  # normal flat-module import (running the file directly / `tools/confevo` on path)
    from genome import Genome, ALL_FEATURES
except ImportError:  # package-relative import (imported as `confevo.evolve`)
    from .genome import Genome, ALL_FEATURES  # type: ignore[no-redef]


# ---------------------------------------------------------------------------
# Configuration & result records
# ---------------------------------------------------------------------------


@dataclass
class GAConfig:
    """Hyperparameters controlling a single GA run."""

    population: int = 6
    generations: int = 3
    seed: int = 0
    mutation_rate: float = 0.1     # per-feature bit-flip probability
    crossover_rate: float = 0.9
    elitism: int = 1
    tournament_k: int = 3


@dataclass
class GenerationRecord:
    """A one-line summary of a single evaluated generation."""

    index: int
    best_score: float
    mean_score: float
    best_features: list[str]


@dataclass
class GAResult:
    """The outcome of a GA run: the champion plus per-generation history."""

    best_genome: Genome
    best_eval: object          # the EvalResult of best_genome
    history: list[GenerationRecord]
    evaluations: int           # total eval_fn calls actually made


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _tournament_select(
    rng: random.Random,
    scored: list[tuple[Genome, object]],
    k: int,
) -> Genome:
    """Pick ``k`` contestants at random (with replacement) and return the fittest.

    Ties are broken by ``genome.key()`` so selection is deterministic for a fixed
    RNG state. ``k`` is clamped to ``len(scored)`` so it never exceeds the pool.
    """
    n = len(scored)
    k = max(1, min(k, n))
    contestants = [scored[rng.randrange(n)] for _ in range(k)]
    genome, _ev = max(
        contestants,
        key=lambda pair: (pair[1].score, pair[0].key()),
    )
    return genome


def _uniform_crossover(
    rng: random.Random,
    parent_a: Genome,
    parent_b: Genome,
    crossover_rate: float,
) -> set[str]:
    """Uniform crossover over the *full* gene set ``ALL_FEATURES``.

    With probability ``crossover_rate`` each feature is inherited from parent A or
    parent B with equal probability; otherwise (no crossover) the child is a clone
    of parent A. Returns the selected feature names as a mutable set, ready for the
    mutation pass.
    """
    a_feats = parent_a.features
    if rng.random() >= crossover_rate:
        # No crossover this time: clone parent A.
        return set(a_feats)

    b_feats = parent_b.features
    selected: set[str] = set()
    for feature in ALL_FEATURES:
        source = a_feats if rng.random() < 0.5 else b_feats
        if feature in source:
            selected.add(feature)
    return selected


def _mutate(rng: random.Random, features: set[str], mutation_rate: float) -> set[str]:
    """Per-feature bit-flip mutation over the full gene set ``ALL_FEATURES``.

    For every feature in ``ALL_FEATURES`` (in fixed order), flip its membership with
    probability ``mutation_rate`` — so a child can gain *or* lose any gene. Mutates
    and returns the given set.
    """
    for feature in ALL_FEATURES:
        if rng.random() < mutation_rate:
            if feature in features:
                features.discard(feature)
            else:
                features.add(feature)
    return features


# ---------------------------------------------------------------------------
# The GA driver
# ---------------------------------------------------------------------------


def run_ga(eval_fn: Callable[[Genome], object], cfg: GAConfig) -> GAResult:
    """Run the genetic algorithm and return the best genome plus history.

    The initial random population counts as generation 0; the loop runs
    ``cfg.generations`` generations total and therefore produces exactly
    ``cfg.generations`` :class:`GenerationRecord` entries. Each genome is scored via
    the injected ``eval_fn`` (duck-typed through ``.score`` / ``.features``), with
    results cached by ``canonical().key()`` so identical effective builds are only
    evaluated once.
    """
    rng = random.Random(cfg.seed)

    # Memoization across the whole run: canonical key -> EvalResult.
    eval_cache: dict[str, object] = {}
    evaluations = 0

    def evaluate(genome: Genome) -> object:
        nonlocal evaluations
        cache_key = genome.canonical().key()
        cached = eval_cache.get(cache_key)
        if cached is not None:
            return cached
        result = eval_fn(genome)
        eval_cache[cache_key] = result
        evaluations += 1
        return result

    # --- initial population (generation 0) --------------------------------
    population: list[Genome] = [
        Genome.random(rng, p=0.5) for _ in range(cfg.population)
    ]

    history: list[GenerationRecord] = []

    # Track the global champion across every generation.
    best_genome: Genome | None = None
    best_eval: object | None = None

    for gen_index in range(cfg.generations):
        # Score the current population (cached).
        scored: list[tuple[Genome, object]] = [
            (genome, evaluate(genome)) for genome in population
        ]

        # Rank best-first; ties broken by stable key() for determinism.
        ranked = sorted(
            scored,
            key=lambda pair: (pair[1].score, pair[0].key()),
            reverse=True,
        )

        gen_best_genome, gen_best_eval = ranked[0]
        scores = [ev.score for _g, ev in scored]
        record = GenerationRecord(
            index=gen_index,
            best_score=gen_best_eval.score,
            mean_score=sum(scores) / len(scores),
            best_features=list(gen_best_eval.features),
        )
        history.append(record)

        # Update the global champion (strictly-better wins; ties keep the earlier
        # / lexicographically-smaller key for determinism).
        if best_eval is None or (gen_best_eval.score, gen_best_genome.key()) > (
            best_eval.score,
            best_genome.key(),  # type: ignore[union-attr]
        ):
            best_genome = gen_best_genome
            best_eval = gen_best_eval

        # The final generation is only scored/recorded — no breeding past it.
        if gen_index == cfg.generations - 1:
            break

        # --- breed the next generation ------------------------------------
        next_population: list[Genome] = []

        # Elitism: carry over the top-N genomes unchanged.
        elite_count = max(0, min(cfg.elitism, len(ranked)))
        next_population.extend(genome for genome, _ev in ranked[:elite_count])

        # Fill the remainder with offspring.
        while len(next_population) < cfg.population:
            parent_a = _tournament_select(rng, scored, cfg.tournament_k)
            parent_b = _tournament_select(rng, scored, cfg.tournament_k)
            child_features = _uniform_crossover(
                rng, parent_a, parent_b, cfg.crossover_rate
            )
            child_features = _mutate(rng, child_features, cfg.mutation_rate)
            next_population.append(Genome(frozenset(child_features)))

        population = next_population

    # `best_genome`/`best_eval` are always set: cfg.generations >= 1 guarantees at
    # least one scored generation. Guard for the degenerate generations==0 case.
    if best_genome is None or best_eval is None:
        raise ValueError("run_ga requires cfg.generations >= 1")

    return GAResult(
        best_genome=best_genome,
        best_eval=best_eval,
        history=history,
        evaluations=evaluations,
    )


# ---------------------------------------------------------------------------
# Smoke test — synthetic stub evaluator, NO cargo / NO subprocess / NO I/O
# ---------------------------------------------------------------------------


if __name__ == "__main__":

    @dataclass
    class _StubEval:
        """Minimal stand-in for fitness.EvalResult: just `.score` and `.features`."""

        score: float
        features: list[str] = field(default_factory=list)

    def _stub_eval(genome: Genome) -> _StubEval:
        """A toy fitness landscape (no cargo): reward a few features, punish `core`.

        score = 10 * |features ∩ {ui, json-output, shell}|
                - 50 if `core` is in the canonical (effective) feature-set.
        """
        effective = set(genome.canonical().to_features())
        rewarded = {"ui", "json-output", "shell"}
        score = len(genome.features & rewarded) * 10
        if "core" in effective:
            score -= 50
        return _StubEval(score=float(score), features=genome.to_features())

    cfg = GAConfig(seed=1, population=8, generations=5)
    result = run_ga(_stub_eval, cfg)

    print("=== confevo.evolve smoke test (stub evaluator, no cargo) ===")
    print(f"config: {cfg}")
    print()
    print("generation history:")
    for record in result.history:
        print(
            f"  gen {record.index}: "
            f"best={record.best_score:+.1f} "
            f"mean={record.mean_score:+.2f} "
            f"best_features={record.best_features}"
        )

    print()
    print(f"total eval_fn calls (real, uncached): {result.evaluations}")
    print(f"final best score   : {result.best_eval.score:+.1f}")
    print(f"final best genome  : {result.best_genome.to_features()}")
    print(f"final best canonical: {result.best_genome.canonical().to_features()}")

    first_best = result.history[0].best_score
    last_best = result.history[-1].best_score
    monotonic = last_best >= first_best
    print()
    print(
        f"monotonic-with-elitism check: last({last_best:+.1f}) >= "
        f"first({first_best:+.1f}) -> {'PASS' if monotonic else 'FAIL'}"
    )
    assert monotonic, (
        f"elitism violated: last best {last_best} < first best {first_best}"
    )
