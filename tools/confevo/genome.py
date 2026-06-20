"""confevo.genome — Genetic Cargo-feature configuration optimizer: the genome.

This module is the *foundation* of confevo, a tiny genetic algorithm that
searches the space of Cargo feature-sets for the `affidavit` crate. Four sibling
modules (population, fitness, mutation/crossover operators, and the driver)
import the contract defined here, so the names, signatures, and field layout
below are load-bearing — change them and the siblings break.

What is a "genome"?
-------------------
A **genome** is a single *candidate configuration*: a subset of the crate's
Cargo features (see `ALL_FEATURES`). Concretely it is a `frozenset[str]` wrapped
in a frozen dataclass, which makes it immutable, hashable, and usable as a key in
the `seen`/memoization tables the search keeps. Building a genome corresponds to
choosing "which `--features a,b,c` flags would I pass to `cargo`?" — though this
module never invokes cargo or touches the filesystem; it is pure and
deterministic.

Feature implications
--------------------
Cargo features can enable *other* features. `FEATURE_IMPLICATIONS` records, for
each affidavit feature, the set of affidavit-internal features it turns on. For
example `predictive` implies `conformance`, which implies `discovery`, which
implies `core`; and `metrics` implies `otel`. These chains matter because two
genomes that *look* different on the surface can describe the **same** effective
build once implications are applied. `Genome.canonical()` computes the
transitive closure (a fixpoint over `FEATURE_IMPLICATIONS`) so equivalent
configurations collapse to one representative. Callers that want to deduplicate
or compare configurations for *semantic* equality should compare canonical forms
(or their `.key()`), not raw genomes.

Determinism
-----------
Randomness only enters through an explicit `random.Random` instance passed by the
caller (e.g. `Genome.random(rng)`), so seeding the RNG makes every run
reproducible. There is no I/O, no subprocess, and no wall-clock dependence.
"""

from __future__ import annotations

import random
from dataclasses import dataclass

# ---------------------------------------------------------------------------
# Feature universe
# ---------------------------------------------------------------------------

#: Every Cargo feature confevo is allowed to toggle. A `Genome` is a subset of
#: this list; any feature outside it is rejected by validation.
ALL_FEATURES: list[str] = [
    "core", "inspection", "discovery", "conformance", "predictive", "lsp", "file-watch",
    "mutation", "fixture-db", "otel", "metrics", "json-output", "shell", "ui", "benchmarking",
    "profiling", "gpu", "remediation", "pqc", "quality-monitor", "webhook",
]

#: affidavit-internal feature implications: enabling a key feature also enables
#: each feature listed in its value (which may, transitively, enable more). This
#: drives `Genome.canonical()`. Every feature in `ALL_FEATURES` appears here so
#: the table is total; features that imply nothing map to an empty list.
FEATURE_IMPLICATIONS: dict[str, list[str]] = {
    "core": [], "discovery": ["core"], "conformance": ["discovery"],
    "predictive": ["conformance"], "metrics": ["otel"], "profiling": ["benchmarking"],
    "file-watch": ["shell"], "quality-monitor": ["shell"], "webhook": ["shell"],
    # all other features imply nothing internal:
    "inspection": [], "lsp": [], "mutation": [], "fixture-db": [], "otel": [],
    "json-output": [], "shell": [], "ui": [], "benchmarking": [], "gpu": [],
    "remediation": [], "pqc": [],
}

#: Set form of ALL_FEATURES, precomputed for O(1) membership checks.
_ALL_FEATURES_SET: frozenset[str] = frozenset(ALL_FEATURES)


# ---------------------------------------------------------------------------
# Genome
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Genome:
    """A candidate Cargo feature-set.

    The genome is an immutable, hashable wrapper around a `frozenset[str]` of
    feature names. Because the dataclass is frozen and holds a frozenset, value
    equality and hashing come for free: two genomes are `==` iff they hold the
    same raw features.

    Note on equality semantics: raw `==` compares the *literal* feature sets.
    Two genomes that differ only by *implied* features are therefore NOT equal
    as written — but they ARE equal after calling `.canonical()` on both, which
    expands each to its transitive closure. Callers that care about semantic
    (effective-build) equality should compare `a.canonical() == b.canonical()`
    or `a.key() == b.key()`.
    """

    features: frozenset[str]

    def __post_init__(self) -> None:
        # Accept any iterable of feature names but normalize to a frozenset so
        # equality/hashing are well-defined regardless of how the caller built
        # it. Then validate against the known feature universe.
        if not isinstance(self.features, frozenset):
            object.__setattr__(self, "features", frozenset(self.features))
        unknown = self.features - _ALL_FEATURES_SET
        if unknown:
            raise ValueError(
                "unknown feature(s) not in ALL_FEATURES: "
                + ", ".join(sorted(unknown))
            )

    # -- construction -------------------------------------------------------

    @classmethod
    def random(cls, rng: random.Random, p: float = 0.5) -> "Genome":
        """Build a random genome, including each feature independently w.p. `p`.

        `rng` is an explicit `random.Random` so results are reproducible when the
        RNG is seeded. Iterates `ALL_FEATURES` in order so the sequence of draws
        is itself deterministic for a given seed.
        """
        chosen = frozenset(f for f in ALL_FEATURES if rng.random() < p)
        return cls(chosen)

    # -- views --------------------------------------------------------------

    def to_features(self) -> list[str]:
        """Return the genome's features as a sorted list."""
        return sorted(self.features)

    def to_cargo_features_arg(self) -> str:
        """Render as a comma-joined sorted string for `cargo --features`.

        Returns the empty string for the empty genome (i.e. `--no-default-features`
        with nothing added).
        """
        return ",".join(sorted(self.features))

    # -- canonicalization ---------------------------------------------------

    def canonical(self) -> "Genome":
        """Return an equivalent genome with all implied features added.

        Computes the transitive closure over `FEATURE_IMPLICATIONS` via a simple
        fixpoint: keep folding in implications until the set stops growing. The
        result is the genome's *effective* feature-set, so genomes that describe
        the same build canonicalize to equal (and hash-equal) values.
        """
        closure: set[str] = set(self.features)
        while True:
            additions: set[str] = set()
            for feat in closure:
                additions.update(FEATURE_IMPLICATIONS.get(feat, ()))
            if additions <= closure:
                break  # fixpoint reached: nothing new to add
            closure |= additions
        return Genome(frozenset(closure))

    def key(self) -> str:
        """Stable identity string derived from the CANONICAL feature-set.

        Useful as a dict key / dedup token for semantically equal genomes:
        comma-joined sorted canonical features, or the literal "<empty>" when the
        canonical form has no features.
        """
        canon = self.canonical()
        if not canon.features:
            return "<empty>"
        return ",".join(sorted(canon.features))


# ---------------------------------------------------------------------------
# Smoke test
# ---------------------------------------------------------------------------


if __name__ == "__main__":
    rng = random.Random(0)
    genomes = [Genome.random(rng) for _ in range(2)]

    # Also exercise an implication chain explicitly so canonicalization is
    # visible in the output: `predictive` should pull in conformance/discovery/core.
    genomes.append(Genome(frozenset({"predictive", "metrics", "profiling"})))

    for i, g in enumerate(genomes):
        print(f"genome[{i}]")
        print(f"  to_features()            = {g.to_features()}")
        print(f"  to_cargo_features_arg()  = {g.to_cargo_features_arg()!r}")
        print(f"  canonical().to_features()= {g.canonical().to_features()}")
        print(f"  key()                    = {g.key()!r}")

    # Demonstrate the equality contract: differs only by implied features.
    bare = Genome(frozenset({"predictive"}))
    expanded = Genome(frozenset({"predictive", "conformance", "discovery", "core"}))
    print()
    print(f"bare == expanded                         : {bare == expanded}")
    print(f"bare.canonical() == expanded.canonical() : {bare.canonical() == expanded.canonical()}")
    print(f"bare.key() == expanded.key()             : {bare.key() == expanded.key()}")
