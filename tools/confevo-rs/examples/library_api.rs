//! **Library API**: using confevo as a library in your own Rust code.
//!
//! This shows how to:
//! - Load a feature space from Cargo.toml
//! - Construct queries programmatically
//! - Run semantic config generation
//! - Check results and act on them
//!
//! Use this pattern to integrate confevo into build systems, CI, or custom tools.
//!
//! Run with:
//!   cargo run --example library_api

use confevo::{feature_space_from_cargo_toml, generate_config, ConfigQuery, Engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== confevo as a Library ===\n");

    // Step 1: Load the feature space from this crate's own Cargo.toml
    println!("Step 1: Load feature space from Cargo.toml");
    let space = feature_space_from_cargo_toml("Cargo.toml", false)?;
    println!("  Loaded {} features", space.features().len());
    println!("  (Deterministic closure computed for all implications)\n");

    // Step 2: Build a query dynamically
    println!("Step 2: Build a configuration query");
    let query = ConfigQuery::new();

    // Note: confevo-rs is intentionally feature-free (zero dependencies).
    // This example shows the pattern; use on a real Cargo.toml with features.
    println!("  (Note: confevo-rs itself has no features for demo purposes)");
    println!("  In real usage, you'd call .require() and .forbid() here.\n");

    // Step 3: Solve with SAT breed
    println!("Step 3: Solve with SAT breed");
    if space.features().is_empty() {
        println!("  (No features in this crate; would solve on a real manifest)");
    } else {
        let result = generate_config(&space, &query, Engine::SatCdcl)?;
        println!("  Feasible: {}", result.feasible);

        if result.feasible {
            let genome = result.genome.unwrap();
            let feats = genome.features();
            println!("  Solution: {} features", feats.len());
            println!("  Feature set: {}", feats.join(", "));

            // Step 4: Use the result programmatically
            println!("\nStep 4: Act on the result");
            let cargo_flags = format!("--no-default-features --features {}", feats.join(","));
            println!("  To build this config, run:");
            println!("    cargo build {}", cargo_flags);
        } else {
            println!("  Configuration is IMPOSSIBLE");
            println!("  Obstruction: {:?}", result.clash);
        }

        // Step 5: Verify with CSP breed (optional second opinion)
        println!("\nStep 5: Verify result with CSP breed");
        let csp_result = generate_config(&space, &query, Engine::CspAc3)?;
        if result.feasible == csp_result.feasible {
            println!("  ✓ Both breeds agree on feasibility");
        } else {
            println!("  ⚠️  Breeds disagreed! One has a bug.");
        }
    }

    // Step 6: Show common query patterns (would work on a real manifest)
    println!("\n=== Query Construction Patterns ===");

    println!("\nPattern A: Minimal viable config");
    println!("  let q = ConfigQuery::new();");
    println!("  let result = generate_config(&space, &q, Engine::SatCdcl)?;");

    println!("\nPattern B: Require feature, forbid dependency");
    println!("  let q = ConfigQuery::new().require(\"feature\").forbid(\"dep\");");
    println!("  let result = generate_config(&space, &q, Engine::SatCdcl)?;");

    println!("\nPattern C: Check multiple scenarios");
    println!("  for feature in [\"a\", \"b\", \"c\"] {{");
    println!("    let q = ConfigQuery::new().require(feature);");
    println!("    let res = generate_config(&space, &q, Engine::SatCdcl)?;");
    println!("    println!(\"{{feature}}: {{}}\", res.feasible);");
    println!("  }}");

    println!("\n=== Library Use Cases ===");
    println!(
        "1. **Build system integration**: Call from your build script to auto-select features\n\
         2. **CI matrix generation**: Generate test combinations programmatically\n\
         3. **Feature auditing**: Check if deprecated features are still reachable\n\
         4. **Dependency analysis**: Find unused or always-enabled features\n\
         5. **Configuration solver**: Solve feature constraints in your application"
    );

    Ok(())
}
