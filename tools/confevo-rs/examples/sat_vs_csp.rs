//! **SAT vs CSP breeds**: solve the same problem two different ways.
//!
//! The two semantic breeds use different algorithms but should agree on feasibility:
//! - **SAT (sat_cdcl)**: Boolean satisfiability, DPLL with unit propagation, finds *minimal* models
//! - **CSP (csp_ac3)**: Constraint satisfaction, arc consistency, finds *first-valid* models
//!
//! Both prove feasibility/infeasibility. The SAT breed tends to produce smaller configs
//! (fewer features enabled) because it tries false before true.
//!
//! Run with:
//!   cargo run --example sat_vs_csp

use confevo::{generate_config, ConfigQuery, Engine};
use confevo::manifest::feature_space_from_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = r#"
[package]
name = "demo"
version = "1.0.0"

[features]
default = ["api", "db"]

# Service tiers
api = ["http"]
http = []
websocket = []
grpc = []

# Data layers
db = ["storage"]
storage = []
cache = []

# Extras
admin = ["auth"]
auth = []
logging = []
"#;

    let space = feature_space_from_str(manifest, false)?;

    println!("=== Comparing SAT vs CSP Breeds ===\n");

    let test_queries = vec![
        (
            ConfigQuery::new().require("api"),
            "Simple: require 'api'",
        ),
        (
            ConfigQuery::new()
                .require("api")
                .require("cache"),
            "Moderate: require 'api' AND 'cache'",
        ),
        (
            ConfigQuery::new()
                .require("admin")
                .require("websocket"),
            "Complex: require 'admin' AND 'websocket'",
        ),
    ];

    for (query, description) in test_queries {
        println!("Query: {}", description);
        println!("  Requires: {:?}", query.require);
        println!("  Forbids: {:?}\n", query.forbid);

        let sat_result = generate_config(&space, &query, Engine::SatCdcl)?;
        let csp_result = generate_config(&space, &query, Engine::CspAc3)?;

        println!("  SAT (sat_cdcl):");
        println!("    Feasible: {}", sat_result.feasible);
        if let Some(ref g) = sat_result.genome {
            let feats = g.features();
            println!("    Features ({}): {}", feats.len(), feats.join(", "));
        }
        println!("    Steps: {}", sat_result.result.inference_step_count);

        println!("\n  CSP (csp_ac3):");
        println!("    Feasible: {}", csp_result.feasible);
        if let Some(ref g) = csp_result.genome {
            let feats = g.features();
            println!("    Features ({}): {}", feats.len(), feats.join(", "));
        }
        println!("    Steps: {}", csp_result.result.inference_step_count);

        if sat_result.feasible != csp_result.feasible {
            println!("\n  ⚠️  DISAGREEMENT! Breeds gave different verdicts.");
        } else if sat_result.feasible && csp_result.feasible {
            // Both found solutions, but they may differ in size
            let sat_count = sat_result
                .genome
                .as_ref()
                .map(|g| g.features().len())
                .unwrap_or(0);
            let csp_count = csp_result
                .genome
                .as_ref()
                .map(|g| g.features().len())
                .unwrap_or(0);
            if sat_count != csp_count {
                println!(
                    "\n  ℹ️  Both found solutions, but SAT's is {} features, CSP's is {} features.",
                    sat_count, csp_count
                );
                println!("      (SAT prefers minimal; CSP finds first-valid.)");
            } else {
                println!("\n  ✓ Both breeds agree on feasibility and size.");
            }
        }
        println!("\n");
    }

    println!("=== Key Insight ===");
    println!(
        "On any well-formed query, both breeds MUST agree on feasibility.\n\
         (If they disagree, one has a bug.)\n\
         But they may produce different solutions:\n\
         - SAT: finds the *smallest* valid config (false-first in DPLL)\n\
         - CSP: finds the *first* valid config found by domain pruning\n\
         Both are correct answers to 'what features can I enable?'"
    );

    Ok(())
}
