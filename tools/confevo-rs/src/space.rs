//! The feature *space*: the universe of toggleable features plus the implication
//! edges between them.
//!
//! A [`FeatureSpace`] is the project-specific context every other part of confevo
//! is interpreted against. It holds an *ordered* list of feature names (the order
//! is load-bearing: it makes random genome generation, crossover, and mutation
//! deterministic) and a map of *implications* — `a -> {b, c}` means "enabling `a`
//! also enables `b` and `c`". Cargo's own `[features]` table is exactly this shape,
//! which is why [`crate::manifest`] can build a space directly from a `Cargo.toml`.

use std::collections::{BTreeMap, BTreeSet};

/// Error returned when a [`FeatureSpace`] cannot be constructed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceError {
    /// The universe contained the same feature name twice.
    DuplicateFeature(String),
    /// An implication referenced a feature that is not in the universe.
    ///
    /// Carries `(source_feature, missing_target)`.
    UnknownImplicationTarget(String, String),
    /// An implication was keyed on a feature that is not in the universe.
    UnknownImplicationSource(String),
}

impl std::fmt::Display for SpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpaceError::DuplicateFeature(name) => {
                write!(f, "duplicate feature in universe: {name}")
            }
            SpaceError::UnknownImplicationTarget(src, dst) => {
                write!(f, "feature {src:?} implies unknown feature {dst:?}")
            }
            SpaceError::UnknownImplicationSource(src) => {
                write!(f, "implication keyed on unknown feature {src:?}")
            }
        }
    }
}

impl std::error::Error for SpaceError {}

/// The universe of features and the implication edges between them.
#[derive(Debug, Clone)]
pub struct FeatureSpace {
    /// Ordered universe of feature names (deterministic iteration order).
    features: Vec<String>,
    /// `feature -> directly-implied features` (only edges within the universe).
    implications: BTreeMap<String, BTreeSet<String>>,
    /// Membership set, for O(log n) `contains`.
    member: BTreeSet<String>,
}

impl FeatureSpace {
    /// Build a space from a feature universe and a set of implication edges.
    ///
    /// `features` defines the toggleable universe *and its iteration order*.
    /// `implications` is a list of `(source, [targets])` pairs; every source and
    /// every target must appear in `features`, otherwise a [`SpaceError`] is
    /// returned. Sources may be repeated — their target sets are merged.
    ///
    /// ```
    /// use confevo::FeatureSpace;
    /// let space = FeatureSpace::new(["a", "b"], [("b", vec!["a"])]).unwrap();
    /// assert_eq!(space.len(), 2);
    /// ```
    pub fn new<F, S, I, T, U>(features: F, implications: I) -> Result<Self, SpaceError>
    where
        F: IntoIterator<Item = S>,
        S: Into<String>,
        I: IntoIterator<Item = (T, Vec<U>)>,
        T: Into<String>,
        U: Into<String>,
    {
        let features: Vec<String> = features.into_iter().map(Into::into).collect();

        let mut member = BTreeSet::new();
        for name in &features {
            if !member.insert(name.clone()) {
                return Err(SpaceError::DuplicateFeature(name.clone()));
            }
        }

        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (src, targets) in implications {
            let src = src.into();
            if !member.contains(&src) {
                return Err(SpaceError::UnknownImplicationSource(src));
            }
            let entry = edges.entry(src.clone()).or_default();
            for t in targets {
                let t = t.into();
                if !member.contains(&t) {
                    return Err(SpaceError::UnknownImplicationTarget(src, t));
                }
                entry.insert(t);
            }
        }

        Ok(FeatureSpace {
            features,
            implications: edges,
            member,
        })
    }

    /// The ordered feature universe.
    pub fn features(&self) -> &[String] {
        &self.features
    }

    /// Number of features in the universe.
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// `true` if the universe is empty.
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// `true` if `name` is part of this space.
    pub fn contains(&self, name: &str) -> bool {
        self.member.contains(name)
    }

    /// The directly-implied features of `name` (not transitive). Empty if none.
    pub fn implications_of(&self, name: &str) -> impl Iterator<Item = &String> {
        self.implications.get(name).into_iter().flatten()
    }

    /// Compute the **transitive closure** of `feats` under the implication edges.
    ///
    /// Repeatedly folds in implied features until a fixpoint is reached, so the
    /// returned set is the *effective* feature-set that Cargo would actually
    /// activate. Inputs outside the universe are ignored.
    pub fn closure(&self, feats: &BTreeSet<String>) -> BTreeSet<String> {
        let mut closure: BTreeSet<String> = feats
            .iter()
            .filter(|f| self.member.contains(*f))
            .cloned()
            .collect();
        loop {
            let mut additions: BTreeSet<String> = BTreeSet::new();
            for feat in &closure {
                if let Some(targets) = self.implications.get(feat) {
                    for t in targets {
                        if !closure.contains(t) {
                            additions.insert(t.clone());
                        }
                    }
                }
            }
            if additions.is_empty() {
                break;
            }
            closure.extend(additions);
        }
        closure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn rejects_duplicate_features() {
        let err = FeatureSpace::new(["a", "a"], Vec::<(&str, Vec<&str>)>::new()).unwrap_err();
        assert_eq!(err, SpaceError::DuplicateFeature("a".into()));
    }

    #[test]
    fn rejects_unknown_implication_target() {
        let err = FeatureSpace::new(["a"], [("a", vec!["ghost"])]).unwrap_err();
        assert_eq!(
            err,
            SpaceError::UnknownImplicationTarget("a".into(), "ghost".into())
        );
    }

    #[test]
    fn rejects_unknown_implication_source() {
        let err = FeatureSpace::new(["a"], [("ghost", vec!["a"])]).unwrap_err();
        assert_eq!(err, SpaceError::UnknownImplicationSource("ghost".into()));
    }

    #[test]
    fn closure_follows_transitive_chain() {
        // predictive -> conformance -> discovery -> core
        let space = FeatureSpace::new(
            ["core", "discovery", "conformance", "predictive"],
            [
                ("discovery", vec!["core"]),
                ("conformance", vec!["discovery"]),
                ("predictive", vec!["conformance"]),
            ],
        )
        .unwrap();

        let closure = space.closure(&set(&["predictive"]));
        assert_eq!(
            closure,
            set(&["predictive", "conformance", "discovery", "core"])
        );
    }

    #[test]
    fn closure_ignores_unknown_inputs() {
        let space = FeatureSpace::new(["a"], Vec::<(&str, Vec<&str>)>::new()).unwrap();
        let closure = space.closure(&set(&["a", "not-in-space"]));
        assert_eq!(closure, set(&["a"]));
    }

    #[test]
    fn closure_of_empty_is_empty() {
        let space = FeatureSpace::new(["a", "b"], [("a", vec!["b"])]).unwrap();
        assert!(space.closure(&BTreeSet::new()).is_empty());
    }

    #[test]
    fn merges_repeated_sources() {
        let space =
            FeatureSpace::new(["a", "b", "c"], [("a", vec!["b"]), ("a", vec!["c"])]).unwrap();
        let closure = space.closure(&set(&["a"]));
        assert_eq!(closure, set(&["a", "b", "c"]));
    }
}
