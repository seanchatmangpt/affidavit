//! # confevo — automatic Cargo feature-flag configuration generation
//!
//! `confevo` is a **zero-dependency**, `#![forbid(unsafe_code)]` Rust library and
//! CLI that *automatically generates* good Cargo feature-flag configurations for
//! a crate. It is a small, TPOT2-style **genetic algorithm**: a candidate
//! configuration (a subset of the crate's `[features]`) is a [`Genome`], a
//! population of genomes is scored by a pluggable [`Evaluator`], and the fittest
//! are bred forward via tournament selection, uniform crossover, per-feature
//! bit-flip mutation, and elitism.
//!
//! ## Why it exists
//!
//! A modern crate can expose dozens of optional features whose interactions are
//! not obvious — some combinations fail to build, some pull in a broken upstream
//! dependency, some are redundant. confevo **searches that space for you** and
//! reports the configuration that gets *closest to a clean build* (or, when none
//! builds, pinpoints the obstruction). It was built alongside `affidavit`, whose
//! root crate cannot compile because of a broken upstream (`wasm4pm-compat
//! 26.6.13`); confevo's job is to *map* such a feature space rather than pretend
//! it is healthy.
//!
//! ## Where the feature space comes from
//!
//! The library is **project-agnostic**: it reads the feature universe and the
//! implication edges straight out of a real `Cargo.toml [features]` table (see
//! [`manifest::feature_space_from_cargo_toml`]). Cargo already records
//! feature→feature edges (`a = ["b"]` means *a implies b*), so confevo's
//! canonical-closure logic reuses the manifest as ground truth — no hand-written
//! implication table required.
//!
//! ## Determinism
//!
//! Every source of randomness flows through a single seeded [`rng::Rng`]
//! (SplitMix64), and every loop that consumes randomness iterates a *fixed-order*
//! sequence (the feature universe, list indices). Given the same [`GaConfig`] the
//! run is bit-for-bit reproducible.
//!
//! ## Example
//!
//! ```
//! use confevo::{FeatureSpace, GaConfig, SyntheticEvaluator, run_ga};
//!
//! // A tiny feature space: `b` implies `a`.
//! let space = FeatureSpace::new(
//!     ["a", "b", "c"],
//!     [("b", vec!["a"])],
//! )
//! .unwrap();
//!
//! let mut eval = SyntheticEvaluator::generic();
//! let cfg = GaConfig { seed: 42, population: 8, generations: 5, ..Default::default() };
//! let result = run_ga(&mut eval, &space, &cfg).unwrap();
//!
//! // Elitism guarantees the best-so-far never regresses.
//! let first = result.history.first().unwrap().best_score;
//! let last = result.history.last().unwrap().best_score;
//! assert!(last >= first);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod evolve;
pub mod fitness;
pub mod genome;
pub mod manifest;
pub mod report;
pub mod rng;
pub mod space;

#[doc(inline)]
pub use evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord};
#[doc(inline)]
pub use fitness::{
    score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator,
};
#[doc(inline)]
pub use genome::Genome;
#[doc(inline)]
pub use manifest::{feature_space_from_cargo_toml, ManifestError};
#[doc(inline)]
pub use rng::Rng;
#[doc(inline)]
pub use space::{FeatureSpace, SpaceError};
