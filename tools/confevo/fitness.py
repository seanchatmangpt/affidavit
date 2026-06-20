"""Fitness evaluation for the genetic Cargo-feature configuration optimizer.

This module evaluates a :class:`Genome` (a candidate set of Cargo features) by
either actually invoking ``cargo build`` (real mode) or by computing a
deterministic synthetic score that models the real ``affidavit`` repo situation
(dry-run mode). Results are cached by the genome's canonical key.

Doctrine note: this evaluator *certifies* a configuration's build outcome; it
does not decide whether the configuration is "good" beyond the scoring formula.

STDLIB ONLY.
"""

from __future__ import annotations

import json
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

try:
    # Flat import: the CLI runs from inside tools/confevo/.
    from genome import Genome, ALL_FEATURES
except ImportError:  # pragma: no cover - package-style import fallback
    from .genome import Genome, ALL_FEATURES  # type: ignore


# ---------------------------------------------------------------------------
# Result type
# ---------------------------------------------------------------------------
@dataclass
class EvalResult:
    features: list[str]
    resolves: bool
    builds: bool
    error_count: int
    warn_count: int
    elapsed_s: float
    score: float
    from_cache: bool = False

    def to_dict(self) -> dict:
        """Serialize to a plain JSON-compatible dict (for cache persistence)."""
        return {
            "features": list(self.features),
            "resolves": bool(self.resolves),
            "builds": bool(self.builds),
            "error_count": int(self.error_count),
            "warn_count": int(self.warn_count),
            "elapsed_s": float(self.elapsed_s),
            "score": float(self.score),
            "from_cache": bool(self.from_cache),
        }

    @classmethod
    def from_dict(cls, d: dict) -> "EvalResult":
        return cls(
            features=list(d.get("features", [])),
            resolves=bool(d.get("resolves", False)),
            builds=bool(d.get("builds", False)),
            error_count=int(d.get("error_count", 0)),
            warn_count=int(d.get("warn_count", 0)),
            elapsed_s=float(d.get("elapsed_s", 0.0)),
            score=float(d.get("score", 0.0)),
            from_cache=bool(d.get("from_cache", False)),
        )


# ---------------------------------------------------------------------------
# Scoring
# ---------------------------------------------------------------------------
def score_from(
    builds: bool,
    resolves: bool,
    error_count: int,
    n_features: int,
    elapsed_s: float,
) -> float:
    """Compute the fitness score from raw evaluation signals.

    Heavily rewards a clean build, lightly rewards dependency resolution and
    feature breadth, and penalizes compiler errors and wall-clock time.
    """
    s = 1000.0 if builds else 0.0
    s += 50.0 if resolves else 0.0
    s -= float(error_count)
    s += 2.0 * n_features
    s -= 0.1 * elapsed_s
    return s


# ---------------------------------------------------------------------------
# Synthetic (dry-run) model
# ---------------------------------------------------------------------------
def compute_synthetic(features: set[str]) -> tuple[bool, bool, int, int, float]:
    """Deterministic model of the real ``affidavit`` build situation.

    The input ``features`` MUST be the *canonical* (closure-expanded) feature
    set, so that e.g. ``discovery`` already implies ``core``.

    Returns ``(resolves, builds, error_count, warn_count, elapsed_s)``.

    Model rationale:
      * affidavit's lib references ``wasm4pm_compat`` unconditionally, so even
        the empty feature set fails with a base of ~6 errors.
      * Enabling ``core`` pulls in the broken wasm4pm-compat crate: +550.
      * Each of discovery / conformance / predictive widens the wasm4pm
        surface: +20 apiece.
      * ``lsp`` and ``gpu`` add small synthetic penalties (+5 each) so the
        landscape has gradient.
    """
    error_count = 6  # base: unconditional wasm4pm_compat reference

    if "core" in features:
        error_count += 550  # broken wasm4pm-compat crate

    for f in ("discovery", "conformance", "predictive"):
        if f in features:
            error_count += 20  # more wasm4pm surface

    if "lsp" in features:
        error_count += 5
    if "gpu" in features:
        error_count += 5  # heavy optional graphs

    warn_count = len(features)  # cosmetic
    builds = error_count == 0  # effectively never in this repo
    resolves = True  # dependency resolution always succeeds in dry-run
    elapsed_s = 0.5 + 0.1 * len(features)
    return resolves, builds, error_count, warn_count, elapsed_s


# ---------------------------------------------------------------------------
# Real-build resolution heuristic
# ---------------------------------------------------------------------------
# Cargo-level dependency-resolution failures ONLY. Deliberately excludes
# "failed to resolve", which is the wording of the E0433 *compile* error
# ("failed to resolve: cannot find module `wasm4pm_compat`") and would
# otherwise misreport a crate that resolved fine but failed to compile.
_RESOLVE_FAIL_MARKERS = (
    "failed to select a version",
    "no matching package",
    "error: failed to get",
    "failed to load source",
    "unable to update",
)


def _resolves_from_output(combined: str) -> bool:
    """Heuristic: dependency resolution succeeded unless a known marker appears."""
    for marker in _RESOLVE_FAIL_MARKERS:
        if marker in combined:
            return False
    return True


def _count_compiler_messages(stdout: str) -> tuple[int, int]:
    """Parse cargo JSON-lines stdout, counting compiler errors and warnings."""
    error_count = 0
    warn_count = 0
    for line in stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            obj = json.loads(line)
        except (json.JSONDecodeError, ValueError):
            continue
        if not isinstance(obj, dict):
            continue
        if obj.get("reason") != "compiler-message":
            continue
        message = obj.get("message")
        if not isinstance(message, dict):
            continue
        level = message.get("level")
        if level == "error":
            error_count += 1
        elif level == "warning":
            warn_count += 1
    return error_count, warn_count


# ---------------------------------------------------------------------------
# Evaluator
# ---------------------------------------------------------------------------
class FitnessEvaluator:
    """Evaluates genomes via cargo build (or a synthetic model), with caching."""

    def __init__(
        self,
        repo_root: Path,
        timeout: int = 600,
        dry_run: bool = False,
        cache_path: Optional[Path] = None,
    ):
        self.repo_root = Path(repo_root)
        self.timeout = int(timeout)
        self.dry_run = bool(dry_run)
        self.cache_path = Path(cache_path) if cache_path is not None else None
        self._cache: dict[str, EvalResult] = {}
        if self.cache_path is not None:
            self._load_cache()

    # -- cache persistence --------------------------------------------------
    def _load_cache(self) -> None:
        """Best-effort load of the on-disk cache. Corrupt caches are ignored."""
        if self.cache_path is None or not self.cache_path.exists():
            return
        try:
            raw = self.cache_path.read_text(encoding="utf-8")
            data = json.loads(raw)
            if not isinstance(data, dict):
                return
            for key, entry in data.items():
                if isinstance(entry, dict):
                    try:
                        self._cache[key] = EvalResult.from_dict(entry)
                    except (TypeError, ValueError):
                        continue
        except (OSError, json.JSONDecodeError, ValueError):
            # Corrupt or unreadable cache: start fresh.
            return

    def _save_cache(self) -> None:
        """Best-effort persist of the cache to disk."""
        if self.cache_path is None:
            return
        try:
            self.cache_path.parent.mkdir(parents=True, exist_ok=True)
            payload = {key: res.to_dict() for key, res in self._cache.items()}
            self.cache_path.write_text(
                json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8"
            )
        except OSError:
            # Persistence is best-effort; never fail evaluation over it.
            return

    @staticmethod
    def _copy_as_cached(res: EvalResult) -> EvalResult:
        return EvalResult(
            features=list(res.features),
            resolves=res.resolves,
            builds=res.builds,
            error_count=res.error_count,
            warn_count=res.warn_count,
            elapsed_s=res.elapsed_s,
            score=res.score,
            from_cache=True,
        )

    # -- evaluation ---------------------------------------------------------
    def evaluate(self, g: Genome) -> EvalResult:
        """Evaluate a genome, returning a cached copy on a key hit."""
        key = g.canonical().key()
        cached = self._cache.get(key)
        if cached is not None:
            return self._copy_as_cached(cached)

        if self.dry_run:
            result = self._evaluate_synthetic(g)
        else:
            result = self._evaluate_real(g)

        self._cache[key] = result
        self._save_cache()
        # First computation: from_cache stays False. Subsequent hits go through
        # _copy_as_cached() (which yields a fresh object with from_cache=True),
        # so the stored entry is never mutated by callers.
        return result

    def _evaluate_synthetic(self, g: Genome) -> EvalResult:
        # Use the CANONICAL feature closure so discovery -> core -> +550, etc.
        canonical_features = g.canonical().to_features()
        feature_set = set(canonical_features)
        resolves, builds, error_count, warn_count, elapsed_s = compute_synthetic(
            feature_set
        )
        n_features = len(feature_set)
        score = score_from(builds, resolves, error_count, n_features, elapsed_s)
        return EvalResult(
            features=g.to_features(),
            resolves=resolves,
            builds=builds,
            error_count=error_count,
            warn_count=warn_count,
            elapsed_s=elapsed_s,
            score=score,
            from_cache=False,
        )

    def _evaluate_real(self, g: Genome) -> EvalResult:
        cmd = [
            "cargo",
            "build",
            "--no-default-features",
            "--lib",
            "--message-format=json",
        ]
        feat_arg = g.to_cargo_features_arg()
        if feat_arg:
            cmd += ["--features", feat_arg]

        start = time.monotonic()
        try:
            proc = subprocess.run(
                cmd,
                cwd=str(self.repo_root),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=self.timeout,
                text=True,
            )
            elapsed_s = time.monotonic() - start
            stdout = proc.stdout or ""
            stderr = proc.stderr or ""
            combined = stdout + "\n" + stderr

            error_count, warn_count = _count_compiler_messages(stdout)
            builds = proc.returncode == 0
            resolves = _resolves_from_output(combined)
        except subprocess.TimeoutExpired:
            # Timeouts are strongly penalized: count as a non-build with a huge
            # error count but assume resolution succeeded.
            elapsed_s = time.monotonic() - start
            builds = False
            resolves = True
            error_count = 10 ** 6
            warn_count = 0

        features = g.to_features()
        n_features = len(features)
        score = score_from(builds, resolves, error_count, n_features, elapsed_s)
        return EvalResult(
            features=features,
            resolves=resolves,
            builds=builds,
            error_count=error_count,
            warn_count=warn_count,
            elapsed_s=elapsed_s,
            score=score,
            from_cache=False,
        )


# ---------------------------------------------------------------------------
# Smoke test
# ---------------------------------------------------------------------------
if __name__ == "__main__":
    repo_root = Path(__file__).resolve().parents[2]
    ev = FitnessEvaluator(repo_root=repo_root, dry_run=True)

    cases = [
        ("empty", Genome(frozenset())),
        ("{core}", Genome(frozenset({"core"}))),
        ("{discovery}", Genome(frozenset({"discovery"}))),
        ("{ui, json-output}", Genome(frozenset({"ui", "json-output"}))),
    ]

    print("# confevo fitness smoke (dry_run=True)")
    print(f"# repo_root = {repo_root}")
    print(f"# ALL_FEATURES = {ALL_FEATURES}")
    for label, g in cases:
        res = ev.evaluate(g)
        print(
            f"{label:20s} -> resolves={res.resolves} builds={res.builds} "
            f"error_count={res.error_count} warn_count={res.warn_count} "
            f"elapsed_s={res.elapsed_s:.2f} score={res.score:.2f} "
            f"from_cache={res.from_cache} features={res.features}"
        )

    # Re-evaluate one case to demonstrate the cache hit path.
    again = ev.evaluate(cases[1][1])
    print(f"\n# cache check: re-evaluate {cases[1][0]} -> from_cache={again.from_cache}")
