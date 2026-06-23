//! The genetic-algorithm core.
//!
//! [`run_ga`] maintains a population of [`Genome`]s, scores each with an injected
//! [`Evaluator`], and breeds the next generation via **tournament selection**,
//! **uniform crossover**, **per-feature bit-flip mutation**, and **elitism** (the
//! top genomes survive unchanged). Because elites carry forward, the best-so-far
//! score is monotonically non-decreasing across generations.
//!
//! ## Determinism
//!
//! All randomness flows through a single seeded [`Rng`], and every loop that
//! consumes it iterates a fixed-order sequence (the space's feature universe, list
//! indices). Identical [`GaConfig`] + [`FeatureSpace`] + [`Evaluator`] ⇒
//! bit-for-bit identical [`GaResult`].
//!
//! ## Caching
//!
//! Within a run, results are memoized by `genome.canonical().key()`, so two
//! genomes describing the *same effective build* are evaluated once.
//! [`GaResult::evaluations`] counts only the real evaluator calls that happened.

use std::collections::{BTreeSet, HashMap};

use crate::fitness::{EvalResult, Evaluator};
use crate::genome::Genome;
use crate::rng::Rng;
use crate::space::FeatureSpace;

/// Hyperparameters controlling a single GA run.
#[derive(Debug, Clone)]
pub struct GaConfig {
    /// Number of genomes per generation.
    pub population: usize,
    /// Number of generations to run (also the number of history records).
    pub generations: usize,
    /// Seed for the deterministic RNG.
    pub seed: u64,
    /// Per-feature bit-flip probability during mutation.
    pub mutation_rate: f64,
    /// Probability that crossover (vs. cloning parent A) occurs.
    pub crossover_rate: f64,
    /// Number of top genomes carried over unchanged each generation.
    pub elitism: usize,
    /// Tournament size for parent selection.
    pub tournament_k: usize,
}

impl Default for GaConfig {
    fn default() -> Self {
        GaConfig {
            population: 6,
            generations: 3,
            seed: 0,
            mutation_rate: 0.1,
            crossover_rate: 0.9,
            elitism: 1,
            tournament_k: 3,
        }
    }
}

/// A one-line summary of a single evaluated generation.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationRecord {
    /// Zero-based generation index.
    pub index: usize,
    /// Best score in this generation.
    pub best_score: f64,
    /// Mean score across this generation.
    pub mean_score: f64,
    /// Features of the best genome this generation (literal, sorted).
    pub best_features: Vec<String>,
}

/// The outcome of a GA run: the champion plus per-generation history.
#[derive(Debug, Clone)]
pub struct GaResult {
    /// The best genome found across all generations.
    pub best_genome: Genome,
    /// The evaluation of [`GaResult::best_genome`].
    pub best_eval: EvalResult,
    /// One [`GenerationRecord`] per generation, in order.
    pub history: Vec<GenerationRecord>,
    /// Number of real (uncached) evaluator calls made.
    pub evaluations: usize,
}

/// Error returned for a degenerate [`GaConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaError(pub String);

impl std::fmt::Display for GaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid GA configuration: {}", self.0)
    }
}

impl std::error::Error for GaError {}

fn validate(cfg: &GaConfig) -> Result<(), GaError> {
    if cfg.population < 1 {
        return Err(GaError(format!(
            "population must be >= 1, got {}",
            cfg.population
        )));
    }
    if cfg.generations < 1 {
        return Err(GaError(format!(
            "generations must be >= 1, got {}",
            cfg.generations
        )));
    }
    if cfg.tournament_k < 1 {
        return Err(GaError(format!(
            "tournament_k must be >= 1, got {}",
            cfg.tournament_k
        )));
    }
    if cfg.elitism > cfg.population {
        return Err(GaError(format!(
            "elitism ({}) cannot exceed population ({})",
            cfg.elitism, cfg.population
        )));
    }
    if !(0.0..=1.0).contains(&cfg.mutation_rate) {
        return Err(GaError(format!(
            "mutation_rate must be in [0, 1], got {}",
            cfg.mutation_rate
        )));
    }
    if !(0.0..=1.0).contains(&cfg.crossover_rate) {
        return Err(GaError(format!(
            "crossover_rate must be in [0, 1], got {}",
            cfg.crossover_rate
        )));
    }
    Ok(())
}

/// A genome paired with its evaluation and stable canonical key.
struct Scored {
    genome: Genome,
    eval: EvalResult,
    key: String,
}

/// Order two `(score, key)` pairs the way the search ranks them: by score, then
/// by key, both ascending. "Best" is the maximum under this order, which matches
/// the reference implementation's descending sort on `(score, key)`.
fn cmp_score_key(a: (f64, &str), b: (f64, &str)) -> std::cmp::Ordering {
    a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1))
}

/// Pick `k` contestants at random (with replacement) and return the fittest.
fn tournament_select<'a>(rng: &mut Rng, scored: &'a [Scored], k: usize) -> &'a Genome {
    let n = scored.len();
    let k = k.max(1).min(n);
    let mut best: &Scored = &scored[rng.below(n)];
    for _ in 1..k {
        let cand = &scored[rng.below(n)];
        if cmp_score_key((cand.eval.score, &cand.key), (best.eval.score, &best.key))
            == std::cmp::Ordering::Greater
        {
            best = cand;
        }
    }
    &best.genome
}

/// Uniform crossover over the full gene set (the space's feature universe).
///
/// With probability `crossover_rate`, each feature is inherited from parent A or B
/// with equal probability; otherwise the child clones parent A.
fn uniform_crossover(
    rng: &mut Rng,
    a: &Genome,
    b: &Genome,
    space: &FeatureSpace,
    crossover_rate: f64,
) -> BTreeSet<String> {
    if !rng.gen_bool(crossover_rate) {
        return a.feature_set().clone();
    }
    let mut selected = BTreeSet::new();
    for feature in space.features() {
        let from_a = rng.gen_bool(0.5);
        let source = if from_a { a } else { b };
        if source.contains(feature) {
            selected.insert(feature.clone());
        }
    }
    selected
}

/// Per-feature bit-flip mutation over the full gene set.
fn mutate(rng: &mut Rng, feats: &mut BTreeSet<String>, space: &FeatureSpace, rate: f64) {
    for feature in space.features() {
        if rng.gen_bool(rate) {
            if feats.contains(feature) {
                feats.remove(feature);
            } else {
                feats.insert(feature.clone());
            }
        }
    }
}

/// Run the genetic algorithm and return the best genome plus per-generation
/// history.
///
/// The initial random population is generation 0; the loop runs `cfg.generations`
/// generations total, producing exactly that many [`GenerationRecord`]s. Each
/// genome is scored via `eval`, memoized by canonical key so identical effective
/// builds are evaluated once.
///
/// Returns [`GaError`] for degenerate configurations (population/generations < 1,
/// elitism > population, out-of-range rates).
pub fn run_ga(
    eval: &mut impl Evaluator,
    space: &FeatureSpace,
    cfg: &GaConfig,
) -> Result<GaResult, GaError> {
    validate(cfg)?;

    let mut rng = Rng::new(cfg.seed);
    let mut cache: HashMap<String, EvalResult> = HashMap::new();
    let mut evaluations = 0usize;

    // Score a genome, going through the per-run memoization cache.
    macro_rules! score_genome {
        ($genome:expr) => {{
            let g: &Genome = $genome;
            let key = g.key(space);
            match cache.get(&key) {
                Some(cached) => {
                    let mut c = cached.clone();
                    c.from_cache = true;
                    Scored {
                        genome: g.clone(),
                        eval: c,
                        key,
                    }
                }
                None => {
                    let ev = eval.evaluate(g, space);
                    cache.insert(key.clone(), ev.clone());
                    evaluations += 1;
                    Scored {
                        genome: g.clone(),
                        eval: ev,
                        key,
                    }
                }
            }
        }};
    }

    // Initial population (generation 0).
    let mut population: Vec<Genome> = (0..cfg.population)
        .map(|_| Genome::random(&mut rng, space, 0.5))
        .collect();

    let mut history: Vec<GenerationRecord> = Vec::with_capacity(cfg.generations);
    let mut best: Option<Scored> = None;

    for gen_index in 0..cfg.generations {
        // Score the current population.
        let mut scored: Vec<Scored> = population.iter().map(|g| score_genome!(g)).collect();

        // Rank best-first (descending by score, then key).
        scored.sort_by(|x, y| cmp_score_key((y.eval.score, &y.key), (x.eval.score, &x.key)));

        let gen_best = &scored[0];
        let sum: f64 = scored.iter().map(|s| s.eval.score).sum();
        let mean = sum / (scored.len() as f64);
        history.push(GenerationRecord {
            index: gen_index,
            best_score: gen_best.eval.score,
            mean_score: mean,
            best_features: gen_best.eval.features.clone(),
        });

        // Update the global champion (strictly-better wins; ties keep the
        // lexicographically-smaller key, matching the reference implementation).
        let is_new_best = match &best {
            None => true,
            Some(b) => {
                cmp_score_key((gen_best.eval.score, &gen_best.key), (b.eval.score, &b.key))
                    == std::cmp::Ordering::Greater
            }
        };
        if is_new_best {
            best = Some(Scored {
                genome: gen_best.genome.clone(),
                eval: gen_best.eval.clone(),
                key: gen_best.key.clone(),
            });
        }

        // The final generation is only scored/recorded — no breeding past it.
        if gen_index == cfg.generations - 1 {
            break;
        }

        // Breed the next generation.
        let mut next: Vec<Genome> = Vec::with_capacity(cfg.population);
        let elite_count = cfg.elitism.min(scored.len());
        next.extend(scored[..elite_count].iter().map(|s| s.genome.clone()));

        while next.len() < cfg.population {
            let pa = tournament_select(&mut rng, &scored, cfg.tournament_k).clone();
            let pb = tournament_select(&mut rng, &scored, cfg.tournament_k).clone();
            let mut child = uniform_crossover(&mut rng, &pa, &pb, space, cfg.crossover_rate);
            mutate(&mut rng, &mut child, space, cfg.mutation_rate);
            next.push(Genome::new(child));
        }

        population = next;
    }

    let champion = best.expect("generations >= 1 guarantees a champion");
    Ok(GaResult {
        best_genome: champion.genome,
        best_eval: champion.eval,
        history,
        evaluations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitness::SyntheticEvaluator;

    fn space() -> FeatureSpace {
        FeatureSpace::new(
            [
                "core",
                "discovery",
                "conformance",
                "ui",
                "otel",
                "metrics",
                "lsp",
            ],
            [
                ("discovery", vec!["core"]),
                ("conformance", vec!["discovery"]),
                ("metrics", vec!["otel"]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn rejects_degenerate_configs() {
        let s = space();
        let mut e = SyntheticEvaluator::generic();
        let bad = [
            GaConfig {
                population: 0,
                ..Default::default()
            },
            GaConfig {
                generations: 0,
                ..Default::default()
            },
            GaConfig {
                tournament_k: 0,
                ..Default::default()
            },
            GaConfig {
                elitism: 5,
                population: 3,
                ..Default::default()
            },
            GaConfig {
                mutation_rate: 2.0,
                ..Default::default()
            },
            GaConfig {
                crossover_rate: -0.1,
                ..Default::default()
            },
        ];
        for cfg in bad {
            assert!(run_ga(&mut e, &s, &cfg).is_err());
        }
    }

    #[test]
    fn history_length_matches_generations() {
        let s = space();
        let mut e = SyntheticEvaluator::generic();
        let cfg = GaConfig {
            generations: 5,
            population: 6,
            seed: 1,
            ..Default::default()
        };
        let r = run_ga(&mut e, &s, &cfg).unwrap();
        assert_eq!(r.history.len(), 5);
        for (i, rec) in r.history.iter().enumerate() {
            assert_eq!(rec.index, i);
        }
    }

    #[test]
    fn best_so_far_is_monotonic_under_elitism() {
        let s = space();
        let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let cfg = GaConfig {
            generations: 8,
            population: 8,
            seed: 99,
            elitism: 2,
            ..Default::default()
        };
        let r = run_ga(&mut e, &s, &cfg).unwrap();
        let scores: Vec<f64> = r.history.iter().map(|h| h.best_score).collect();
        for w in scores.windows(2) {
            assert!(w[1] >= w[0], "best regressed: {scores:?}");
        }
    }

    #[test]
    fn same_seed_same_champion_and_history() {
        let s = space();
        let cfg = GaConfig {
            generations: 6,
            population: 6,
            seed: 42,
            ..Default::default()
        };

        let mut e1 = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let r1 = run_ga(&mut e1, &s, &cfg).unwrap();
        let mut e2 = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let r2 = run_ga(&mut e2, &s, &cfg).unwrap();

        assert_eq!(r1.best_genome, r2.best_genome);
        assert_eq!(r1.best_eval, r2.best_eval);
        let h1: Vec<f64> = r1.history.iter().map(|h| h.best_score).collect();
        let h2: Vec<f64> = r2.history.iter().map(|h| h.best_score).collect();
        assert_eq!(h1, h2);
    }

    #[test]
    fn champion_matches_a_recorded_generation_best() {
        let s = space();
        let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let cfg = GaConfig {
            generations: 5,
            population: 6,
            seed: 7,
            ..Default::default()
        };
        let r = run_ga(&mut e, &s, &cfg).unwrap();
        let best = r
            .history
            .iter()
            .map(|h| h.best_score)
            .fold(f64::MIN, f64::max);
        assert_eq!(r.best_eval.score, best);
    }

    #[test]
    fn no_config_fakes_a_build_for_poisoned_space() {
        let s = space();
        let mut e = SyntheticEvaluator::new(6, [("core", 550u64)]);
        let cfg = GaConfig {
            generations: 6,
            population: 10,
            seed: 3,
            ..Default::default()
        };
        let r = run_ga(&mut e, &s, &cfg).unwrap();
        assert!(!r.best_eval.builds);
    }

    #[test]
    fn caching_keeps_evaluations_at_or_below_brute_force() {
        let s = space();
        let mut e = SyntheticEvaluator::generic();
        let cfg = GaConfig {
            generations: 5,
            population: 6,
            seed: 5,
            ..Default::default()
        };
        let r = run_ga(&mut e, &s, &cfg).unwrap();
        // Memoization can only reduce work below generations*population.
        assert!(r.evaluations <= 5 * 6);
        assert!(r.evaluations >= 1);
    }
}
