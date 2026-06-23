//! **Proving Infeasibility**: the killer feature of semantic reasoning.
//!
//! This is where semantic breeds shine compared to the genetic algorithm:
//! - GA: "score went down, maybe impossible" (guessing)
//! - Breeds: "PROVED impossible via structural analysis" (certainty)
//!
//! If a breed says a config is impossible, it's not an opinion—it's a theorem.
//!
//! Run with:
//!   cargo run --example infeasibility_proof

use confevo::{generate_config, ConfigQuery, Engine};
use confevo::manifest::feature_space_from_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A realistic scenario: trying to use features while avoiding a broken dependency.
    let manifest = r#"
[package]
name = "broken-upstream"
version = "1.0.0"

[features]
default = ["core", "json"]

# Core functionality (depends on broken crate)
core = ["dep:broken-crate-26.6.13"]

# Higher-level features (all transitively depend on core)
discovery = ["core"]
conformance = ["discovery"]
predictive = ["conformance"]

# Analysis and visualization
metrics = ["core"]
performance = ["metrics"]

# Output formats
json = []
protobuf = []
"#;

    let space = feature_space_from_str(manifest, false)?;

    println!("=== Proving Infeasibility ===\n");
    println!(
        "Scenario: broken-crate-26.6.13 is broken. It will never compile.\n\
         We want to know if we can use any advanced features without it.\n"
    );

    // Test 1: Can we use predictive without the broken core?
    println!("Test 1: Can I use 'predictive' without 'core'?");
    let q1 = ConfigQuery::new()
        .require("predictive")
        .forbid("core");
    let r1 = generate_config(&space, &q1, Engine::SatCdcl)?;

    if !r1.feasible {
        println!("  ✗ IMPOSSIBLE");
        println!("  Reason: {}", r1.result.explanation);
        println!("  Obstruction: {:?}\n", r1.clash);
        println!("  Why: predictive → conformance → discovery → core (unavoidable)\n");
    } else {
        println!("  ✓ Possible: {}\n", r1.genome.unwrap().features().join(", "));
    }

    // Test 2: Can we use metrics without core?
    println!("Test 2: Can I use 'metrics' without 'core'?");
    let q2 = ConfigQuery::new()
        .require("metrics")
        .forbid("core");
    let r2 = generate_config(&space, &q2, Engine::SatCdcl)?;

    if !r2.feasible {
        println!("  ✗ IMPOSSIBLE");
        println!("  Reason: {}", r2.result.explanation);
        println!("  Obstruction: {:?}\n", r2.clash);
        println!("  Why: metrics directly implies core\n");
    } else {
        println!("  ✓ Possible: {}\n", r2.genome.unwrap().features().join(", "));
    }

    // Test 3: Can we use just JSON output without core?
    println!("Test 3: Can I use 'json' (just output format) without 'core'?");
    let q3 = ConfigQuery::new()
        .require("json")
        .forbid("core");
    let r3 = generate_config(&space, &q3, Engine::SatCdcl)?;

    if !r3.feasible {
        println!("  ✗ IMPOSSIBLE");
        println!("  Reason: {}", r3.result.explanation);
    } else {
        println!("  ✓ POSSIBLE: {:?}", r3.genome.unwrap().features());
        println!("  Why: json has no implication to core; they're independent\n");
    }

    println!("=== The Value Proposition ===");
    println!(
        "Without semantic reasoning:\n\
         - You'd run 'cargo build --no-default-features --features predictive'\n\
         - It fails due to broken-crate\n\
         - You try removing predictive... fails again\n\
         - You try different combinations... all fail\n\
         - You give up: 'Maybe it's impossible?'\n\
         \n\
         With semantic reasoning:\n\
         - One command: 'confevo solve --require predictive --forbid core'\n\
         - Answer: PROVED IMPOSSIBLE (because predictive → ... → core)\n\
         - Certainty: You know for sure, not guessing\n\
         \n\
         Result: Time saved, knowledge gained, decision made."
    );

    Ok(())
}
