//! **Semantic configuration derivation**: derive a valid feature set from constraints.
//!
//! This demonstrates the key difference between **semantic** (breeds) and **numeric** (GA):
//! - Numeric: scores configurations blindly, hoping to find good ones
//! - Semantic: reasons about feature implications, derives valid configs or proves impossibility
//!
//! Run with:
//!   cargo run --example semantic_derive

use confevo::{generate_config, ConfigQuery, Engine};
use confevo::manifest::feature_space_from_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A realistic crate manifest with feature implications.
    let manifest = r#"
[package]
name = "myapp"
version = "1.0.0"

[features]
default = ["server", "json"]

# Core services
server = ["dep:tokio"]
client = ["dep:reqwest"]

# Data formats
json = []
protobuf = ["dep:prost"]
msgpack = ["dep:rmp-serde"]

# Observability
metrics = ["prometheus"]
prometheus = []
tracing = ["dep:tracing"]

# Feature combinations
full-observability = ["metrics", "tracing"]
data-formats = ["json", "protobuf", "msgpack"]
"#;

    let space = feature_space_from_str(manifest, false)?;

    println!("=== Semantic Configuration Derivation ===\n");
    println!("Feature space: {} features", space.features().len());
    println!("(Deterministic closure computed for all implications)\n");

    // Query 1: Derive a minimal config for "I want metrics"
    println!("Query 1: What's the minimal set for metrics?");
    let query1 = ConfigQuery::new().require("metrics");
    let result1 = generate_config(&space, &query1, Engine::SatCdcl)?;
    println!("  Feasible: {}", result1.feasible);
    if let Some(genome) = result1.genome {
        println!("  Config: {}", genome.features().join(", "));
    }
    println!("  Reason: metrics implies prometheus (per the [features] table)\n");

    // Query 2: Can I have protobuf without json?
    println!("Query 2: Can I have protobuf without json?");
    let query2 = ConfigQuery::new().require("protobuf").forbid("json");
    let result2 = generate_config(&space, &query2, Engine::SatCdcl)?;
    println!("  Feasible: {}", result2.feasible);
    if let Some(genome) = result2.genome {
        println!("  Config: {}", genome.features().join(", "));
    } else {
        println!("  Obstruction: {:?}", result2.clash);
    }
    println!("  Reason: protobuf and json are independent; both can be selected\n");

    // Query 3: I want full observability AND all data formats
    println!("Query 3: Full observability + all data formats?");
    let query3 = ConfigQuery::new()
        .require("full-observability")
        .require("data-formats");
    let result3 = generate_config(&space, &query3, Engine::SatCdcl)?;
    println!("  Feasible: {}", result3.feasible);
    if let Some(genome) = result3.genome {
        println!("  Config: {}", genome.features().join(", "));
    }
    println!("  Reason: both feature sets are compatible\n");

    // Query 4: Can I have client but NOT tokio (which server requires)?
    println!("Query 4: Can I have client but forbid server?");
    let query4 = ConfigQuery::new().require("client").forbid("server");
    let result4 = generate_config(&space, &query4, Engine::SatCdcl)?;
    println!("  Feasible: {}", result4.feasible);
    if let Some(genome) = result4.genome {
        println!("  Config: {}", genome.features().join(", "));
    }
    println!("  Reason: client and server have no implication between them\n");

    // Show the key insight: SAT PROVES these answers.
    println!("=== Key Insight ===");
    println!(
        "Each answer above is PROVED by the SAT breed, not guessed or scored.\n\
         If the breed says 'feasible: true', you can trust it.\n\
         If the breed says 'feasible: false', it has PROVED no such config exists."
    );

    Ok(())
}
