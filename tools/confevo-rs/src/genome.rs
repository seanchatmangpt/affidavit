//! The [`Genome`]: a single candidate feature configuration.
//!
//! A genome is an immutable, hashable subset of a [`FeatureSpace`]'s universe —
//! "which `--features a,b,c` would I pass to `cargo`?". It is pure: constructing,
//! crossing, and mutating genomes never invokes cargo or touches the filesystem.
//!
//! Two genomes that differ only by *implied* features are not literally equal, but
//! they share a [`Genome::key`] (derived from the canonical closure), which is how
//! the search deduplicates semantically-identical configurations.

use std::collections::BTreeSet;

use crate::rng::Rng;
use crate::space::FeatureSpace;

/// A candidate Cargo feature-set: an immutable subset of a [`FeatureSpace`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Genome {
    feats: BTreeSet<String>,
}

impl Genome {
    /// Build a genome from any iterable of feature names.
    ///
    /// Names are stored verbatim (deduplicated by the `BTreeSet`); validity
    /// against a space is the caller's concern — the GA only ever produces genomes
    /// drawn from the space, and [`Genome::canonical`]/[`Genome::key`] ignore
    /// out-of-space names.
    pub fn new<I, S>(feats: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Genome {
            feats: feats.into_iter().map(Into::into).collect(),
        }
    }

    /// The empty genome (`cargo build --no-default-features` with nothing added).
    pub fn empty() -> Self {
        Genome {
            feats: BTreeSet::new(),
        }
    }

    /// Build a random genome, including each feature in `space` independently with
    /// probability `p`.
    ///
    /// Iterates `space.features()` in order so the draw sequence — and therefore
    /// the genome — is reproducible for a seeded [`Rng`].
    pub fn random(rng: &mut Rng, space: &FeatureSpace, p: f64) -> Self {
        let mut feats = BTreeSet::new();
        for f in space.features() {
            if rng.gen_bool(p) {
                feats.insert(f.clone());
            }
        }
        Genome { feats }
    }

    /// The genome's features as a sorted `Vec`.
    pub fn features(&self) -> Vec<String> {
        self.feats.iter().cloned().collect()
    }

    /// Borrow the underlying feature set.
    pub fn feature_set(&self) -> &BTreeSet<String> {
        &self.feats
    }

    /// Number of features enabled in this genome (literal, not canonical).
    pub fn len(&self) -> usize {
        self.feats.len()
    }

    /// `true` if no features are enabled.
    pub fn is_empty(&self) -> bool {
        self.feats.is_empty()
    }

    /// `true` if `feature` is enabled in this genome (literal membership).
    pub fn contains(&self, feature: &str) -> bool {
        self.feats.contains(feature)
    }

    /// Render as a comma-joined sorted string for `cargo --features`.
    ///
    /// Returns the empty string for the empty genome.
    pub fn cargo_features_arg(&self) -> String {
        self.features().join(",")
    }

    /// Return an equivalent genome with all implied features added.
    ///
    /// Delegates to [`FeatureSpace::closure`], so the result is the genome's
    /// *effective* feature-set under `space`'s implication edges.
    pub fn canonical(&self, space: &FeatureSpace) -> Genome {
        Genome {
            feats: space.closure(&self.feats),
        }
    }

    /// Stable identity string derived from the **canonical** feature-set.
    ///
    /// Used as the dedup / memoization token: comma-joined sorted canonical
    /// features, or the literal `"<empty>"` when the canonical form is empty. Two
    /// genomes describing the same effective build share a key.
    pub fn key(&self, space: &FeatureSpace) -> String {
        let canon = self.canonical(space);
        if canon.feats.is_empty() {
            "<empty>".to_string()
        } else {
            canon.features().join(",")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_space() -> FeatureSpace {
        // predictive -> conformance -> discovery -> core ; metrics -> otel
        FeatureSpace::new(
            [
                "core",
                "discovery",
                "conformance",
                "predictive",
                "otel",
                "metrics",
                "ui",
            ],
            [
                ("discovery", vec!["core"]),
                ("conformance", vec!["discovery"]),
                ("predictive", vec!["conformance"]),
                ("metrics", vec!["otel"]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn cargo_arg_is_sorted_and_comma_joined() {
        let g = Genome::new(["ui", "core", "otel"]);
        assert_eq!(g.cargo_features_arg(), "core,otel,ui");
    }

    #[test]
    fn empty_genome_renders_blank_and_keys_as_empty() {
        let space = demo_space();
        let g = Genome::empty();
        assert_eq!(g.cargo_features_arg(), "");
        assert_eq!(g.key(&space), "<empty>");
    }

    #[test]
    fn canonical_pulls_in_implication_chain() {
        let space = demo_space();
        let g = Genome::new(["predictive"]);
        assert_eq!(
            g.canonical(&space).features(),
            vec!["conformance", "core", "discovery", "predictive"]
        );
    }

    #[test]
    fn genomes_differing_only_by_implications_share_a_key() {
        let space = demo_space();
        let bare = Genome::new(["predictive"]);
        let expanded = Genome::new(["predictive", "conformance", "discovery", "core"]);
        assert_ne!(bare, expanded); // literally different
        assert_eq!(bare.key(&space), expanded.key(&space)); // same effective build
    }

    #[test]
    fn random_is_reproducible_for_a_seed() {
        let space = demo_space();
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert_eq!(
            Genome::random(&mut a, &space, 0.5),
            Genome::random(&mut b, &space, 0.5)
        );
    }

    #[test]
    fn random_p_bounds_are_total_and_empty() {
        let space = demo_space();
        let mut rng = Rng::new(1);
        assert_eq!(Genome::random(&mut rng, &space, 1.0).len(), space.len());
        assert!(Genome::random(&mut rng, &space, 0.0).is_empty());
    }
}
