#!/usr/bin/env python3
"""Unit tests for confevo — the genetic Cargo-feature configuration optimizer.

These tests exercise the three sibling modules (``genome``, ``fitness``,
``evolve``) WITHOUT ever invoking ``cargo`` or any subprocess: everything runs
against the deterministic *dry-run* synthetic fitness model and pure in-memory
stubs. That keeps the suite fast, hermetic, and runnable even though the actual
Rust build in this repo is broken (which is precisely the situation confevo was
built to map — see README.md).

Run from anywhere:

    python3 tools/confevo/test_confevo.py
    python3 -m unittest discover -s tools/confevo -p 'test_*.py'

STDLIB ONLY (uses ``unittest``).
"""

from __future__ import annotations

import os
import random
import sys
import tempfile
import unittest
from pathlib import Path

# ---------------------------------------------------------------------------
# Make the flat sibling modules importable regardless of the current working
# directory or how this test file is launched. We insert *this file's own
# directory* at the front of sys.path so `import genome, fitness, evolve`
# resolves the modules sitting next to us (tools/confevo/), not some shadowed
# copy elsewhere on the path.
# ---------------------------------------------------------------------------
_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

import genome  # noqa: E402  (path injection must precede these imports)
import fitness  # noqa: E402
import evolve  # noqa: E402

from genome import Genome, ALL_FEATURES, FEATURE_IMPLICATIONS  # noqa: E402
from fitness import EvalResult, FitnessEvaluator, score_from  # noqa: E402
from evolve import GAConfig, GenerationRecord, GAResult, run_ga  # noqa: E402


# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------

#: A repo_root for FitnessEvaluator. In dry-run mode it is never read (no cargo
#: is invoked), but the constructor requires it, so we point at the real repo
#: root two levels up from tools/confevo/.
REPO_ROOT = _HERE.parents[1]


def make_genome(features) -> Genome:
    """Build a Genome from any iterable of feature names (empty allowed)."""
    return Genome(frozenset(features))


def dry_run_evaluator(cache_path=None) -> FitnessEvaluator:
    """A FitnessEvaluator in dry-run mode (synthetic, no cargo / no subprocess)."""
    return FitnessEvaluator(
        repo_root=REPO_ROOT,
        dry_run=True,
        cache_path=cache_path,
    )


# ---------------------------------------------------------------------------
# 1. Canonicalization / implication-closure
# ---------------------------------------------------------------------------
class TestGenomeCanonical(unittest.TestCase):
    def test_discovery_pulls_in_core(self) -> None:
        canon = make_genome({"discovery"}).canonical()
        self.assertIn("core", canon.features)
        self.assertIn("discovery", canon.features)

    def test_predictive_closure_chain(self) -> None:
        # predictive -> conformance -> discovery -> core (transitive closure).
        canon = make_genome({"predictive"}).canonical()
        for feat in ("predictive", "conformance", "discovery", "core"):
            self.assertIn(feat, canon.features, f"{feat} missing from closure")

    def test_conformance_superset(self) -> None:
        canon = make_genome({"conformance"}).canonical()
        self.assertTrue(
            {"conformance", "discovery", "core"}.issubset(canon.features),
            f"expected superset of conformance/discovery/core, got {canon.to_features()}",
        )

    def test_canonical_is_idempotent(self) -> None:
        once = make_genome({"predictive"}).canonical()
        twice = once.canonical()
        self.assertEqual(once.features, twice.features)
        # key() is derived from the canonical form, so it is stable too.
        self.assertEqual(make_genome({"predictive"}).key(), once.key())

    def test_empty_genome_canonical_is_empty(self) -> None:
        self.assertEqual(make_genome([]).canonical().features, frozenset())
        self.assertEqual(make_genome([]).key(), "<empty>")


# ---------------------------------------------------------------------------
# 2. Encoding / views
# ---------------------------------------------------------------------------
class TestGenomeEncoding(unittest.TestCase):
    def test_to_features_sorted(self) -> None:
        g = make_genome({"ui", "core", "json-output"})
        feats = g.to_features()
        self.assertEqual(feats, sorted(feats))
        self.assertEqual(feats, ["core", "json-output", "ui"])

    def test_cargo_arg_comma_joined(self) -> None:
        g = make_genome({"ui", "core", "json-output"})
        self.assertEqual(g.to_cargo_features_arg(), "core,json-output,ui")

    def test_empty_cargo_arg_is_blank(self) -> None:
        self.assertEqual(make_genome([]).to_cargo_features_arg(), "")
        self.assertEqual(make_genome([]).to_features(), [])

    def test_unknown_feature_rejected(self) -> None:
        with self.assertRaises(ValueError):
            make_genome({"definitely-not-a-real-feature"})


# ---------------------------------------------------------------------------
# 3. Random genome determinism
# ---------------------------------------------------------------------------
class TestGenomeRandom(unittest.TestCase):
    def test_same_seed_same_genome(self) -> None:
        a = Genome.random(random.Random(7)).to_features()
        b = Genome.random(random.Random(7)).to_features()
        self.assertEqual(a, b)

    def test_random_features_are_known(self) -> None:
        known = set(ALL_FEATURES)
        g = Genome.random(random.Random(7))
        self.assertTrue(set(g.features).issubset(known))

    def test_p_zero_and_one_bounds(self) -> None:
        # p=0.0 -> empty; p=1.0 -> all features. Deterministic regardless of seed.
        self.assertEqual(Genome.random(random.Random(123), p=0.0).features, frozenset())
        self.assertEqual(
            Genome.random(random.Random(123), p=1.0).features,
            frozenset(ALL_FEATURES),
        )


# ---------------------------------------------------------------------------
# 4. Synthetic fitness reproducibility
# ---------------------------------------------------------------------------
class TestSyntheticFitness(unittest.TestCase):
    def test_empty_genome_six_errors_no_build(self) -> None:
        ev = dry_run_evaluator()
        res = ev.evaluate(make_genome([]))
        self.assertEqual(res.error_count, 6)
        self.assertFalse(res.builds)
        # Dependency resolution always "succeeds" in the dry-run model.
        self.assertTrue(res.resolves)

    def test_core_genome_explodes(self) -> None:
        ev = dry_run_evaluator()
        res = ev.evaluate(make_genome({"core"}))
        # base 6 + 550 (broken wasm4pm-compat pulled in by core).
        self.assertGreaterEqual(res.error_count, 556)
        self.assertFalse(res.builds)

    def test_noncore_genome_stays_at_base(self) -> None:
        ev = dry_run_evaluator()
        res = ev.evaluate(make_genome({"ui", "json-output"}))
        # No core in the canonical closure -> only the base 6 errors.
        self.assertEqual(res.error_count, 6)
        self.assertFalse(res.builds)

    def test_discovery_implies_core_explosion(self) -> None:
        # discovery canonicalizes to include core, so it should also explode,
        # demonstrating the closure feeds the synthetic model.
        ev = dry_run_evaluator()
        res = ev.evaluate(make_genome({"discovery"}))
        self.assertGreaterEqual(res.error_count, 556)
        self.assertFalse(res.builds)

    def test_reproducible_across_evaluators(self) -> None:
        # Same genome, two independent evaluators -> identical signals & score.
        g = make_genome({"ui", "shell"})
        r1 = dry_run_evaluator().evaluate(g)
        r2 = dry_run_evaluator().evaluate(g)
        self.assertEqual(r1.error_count, r2.error_count)
        self.assertEqual(r1.builds, r2.builds)
        self.assertAlmostEqual(r1.score, r2.score, places=9)


# ---------------------------------------------------------------------------
# 5. Caching
# ---------------------------------------------------------------------------
class TestFitnessCache(unittest.TestCase):
    def test_second_eval_is_cached(self) -> None:
        ev = dry_run_evaluator()
        g = make_genome({"ui", "json-output"})

        first = ev.evaluate(g)
        self.assertFalse(first.from_cache)

        second = ev.evaluate(g)
        self.assertTrue(second.from_cache)
        self.assertAlmostEqual(first.score, second.score, places=9)
        self.assertEqual(first.error_count, second.error_count)

    def test_cache_keyed_by_canonical(self) -> None:
        # discovery and {discovery, core} share a canonical key, so evaluating
        # one then the other should hit the cache on the second.
        ev = dry_run_evaluator()
        first = ev.evaluate(make_genome({"discovery"}))
        self.assertFalse(first.from_cache)
        second = ev.evaluate(make_genome({"discovery", "core"}))
        self.assertTrue(second.from_cache)
        self.assertAlmostEqual(first.score, second.score, places=9)

    def test_cache_persists_to_disk(self) -> None:
        # A fresh evaluator pointed at an existing cache file should read the
        # prior result back as a cache hit. (Pure file I/O, still no cargo.)
        with tempfile.TemporaryDirectory() as tmp:
            cache_file = Path(tmp) / "cache.json"
            g = make_genome({"ui", "json-output"})

            ev1 = dry_run_evaluator(cache_path=cache_file)
            r1 = ev1.evaluate(g)
            self.assertFalse(r1.from_cache)
            self.assertTrue(cache_file.exists())

            ev2 = dry_run_evaluator(cache_path=cache_file)
            r2 = ev2.evaluate(g)
            self.assertTrue(r2.from_cache)
            self.assertAlmostEqual(r1.score, r2.score, places=9)


# ---------------------------------------------------------------------------
# 6. score_from gradient
# ---------------------------------------------------------------------------
class TestScoreFrom(unittest.TestCase):
    def test_builds_beats_no_build(self) -> None:
        building = score_from(True, True, 0, 3, 1.0)
        broken = score_from(False, True, 0, 3, 1.0)
        self.assertGreater(building, broken)

    def test_more_features_adds_two_each(self) -> None:
        base = score_from(True, True, 0, 3, 1.0)
        more = score_from(True, True, 0, 5, 1.0)
        # +2.0 per feature: 2 extra features -> +4.0 (allow tiny float slack).
        self.assertAlmostEqual(more - base, 4.0, places=6)
        self.assertGreater(more, base)

    def test_more_errors_lowers_score(self) -> None:
        fewer = score_from(False, True, 1, 3, 1.0)
        worse = score_from(False, True, 50, 3, 1.0)
        self.assertLess(worse, fewer)

    def test_resolves_helps_and_time_hurts(self) -> None:
        self.assertGreater(
            score_from(False, True, 6, 0, 0.0),
            score_from(False, False, 6, 0, 0.0),
        )
        self.assertGreater(
            score_from(False, True, 6, 0, 0.0),
            score_from(False, True, 6, 0, 100.0),
        )


# ---------------------------------------------------------------------------
# Deterministic stub fitness landscape used by the GA tests (NO cargo).
# ---------------------------------------------------------------------------
class _StubEval:
    """Minimal stand-in for EvalResult: only `.score` and `.features` are used
    by the GA. We also carry `.builds` so test 8-style assertions can read it,
    though the GA itself only duck-types score/features."""

    __slots__ = ("score", "features", "builds")

    def __init__(self, score: float, features, builds: bool = False) -> None:
        self.score = float(score)
        self.features = list(features)
        self.builds = bool(builds)


_REWARDED = frozenset({"ui", "json-output", "shell"})


def _stub_eval(g: Genome) -> _StubEval:
    """Toy landscape: +10 per rewarded feature present; -50 if `core` is in the
    canonical (effective) feature-set. Pure, deterministic, no I/O."""
    effective = set(g.canonical().to_features())
    score = 10 * len(g.features & _REWARDED)
    if "core" in effective:
        score -= 50
    return _StubEval(score=score, features=g.to_features())


# ---------------------------------------------------------------------------
# 7. GA determinism + non-decreasing best (with stub eval_fn)
# ---------------------------------------------------------------------------
class TestGADeterminism(unittest.TestCase):
    def _run(self) -> GAResult:
        return run_ga(_stub_eval, GAConfig(seed=3, population=8, generations=6))

    def test_same_seed_same_champion(self) -> None:
        r1 = self._run()
        r2 = self._run()
        self.assertEqual(r1.best_genome.key(), r2.best_genome.key())

    def test_same_seed_same_history_scores(self) -> None:
        r1 = self._run()
        r2 = self._run()
        h1 = [rec.best_score for rec in r1.history]
        h2 = [rec.best_score for rec in r2.history]
        self.assertEqual(h1, h2)

    def test_history_length_matches_generations(self) -> None:
        r = self._run()
        self.assertEqual(len(r.history), 6)
        self.assertIsInstance(r.history[0], GenerationRecord)

    def test_best_score_non_decreasing(self) -> None:
        # Elitism guarantees the per-generation best never regresses.
        r = self._run()
        self.assertGreaterEqual(r.history[-1].best_score, r.history[0].best_score)
        # And monotonic non-decreasing across every adjacent pair.
        scores = [rec.best_score for rec in r.history]
        for earlier, later in zip(scores, scores[1:]):
            self.assertGreaterEqual(later, earlier)

    def test_champion_matches_best_history_entry(self) -> None:
        r = self._run()
        # The reported champion's score equals the best score ever recorded.
        best_recorded = max(rec.best_score for rec in r.history)
        self.assertAlmostEqual(r.best_eval.score, best_recorded, places=6)


# ---------------------------------------------------------------------------
# 8. GA with the real dry-run FitnessEvaluator (still no cargo)
# ---------------------------------------------------------------------------
class TestGAWithDryRunFitness(unittest.TestCase):
    def test_ga_completes_and_nothing_builds(self) -> None:
        ev = dry_run_evaluator()
        result = run_ga(ev.evaluate, GAConfig(seed=1, population=6, generations=3))

        # The GA produced a full history and a champion.
        self.assertEqual(len(result.history), 3)
        self.assertIsInstance(result.best_genome, Genome)

        # Honest finding: nothing builds in this repo, dry-run or otherwise.
        self.assertFalse(result.best_eval.builds)

        # The fittest config tends to EXCLUDE core: enabling core (directly or
        # via the discovery/conformance/predictive closure) adds +550 errors,
        # which the score penalizes. So the champion's canonical closure should
        # not contain core.
        champion_effective = set(result.best_genome.canonical().to_features())
        self.assertNotIn("core", champion_effective)

        # Correspondingly its error_count should sit at the no-core base (6),
        # well below the core-laden ~556.
        self.assertLess(result.best_eval.error_count, 556)
        self.assertGreaterEqual(result.best_eval.error_count, 6)

    def test_ga_dry_run_is_deterministic(self) -> None:
        # Two runs of the dry-run-fitness GA with the same seed agree on the
        # champion. (Each run uses a fresh evaluator / fresh cache.)
        r1 = run_ga(dry_run_evaluator().evaluate, GAConfig(seed=1, population=6, generations=3))
        r2 = run_ga(dry_run_evaluator().evaluate, GAConfig(seed=1, population=6, generations=3))
        self.assertEqual(r1.best_genome.key(), r2.best_genome.key())
        self.assertAlmostEqual(r1.best_eval.score, r2.best_eval.score, places=6)


if __name__ == "__main__":
    unittest.main()
