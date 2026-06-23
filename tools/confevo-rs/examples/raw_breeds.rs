//! **Raw breed contracts**: working directly with SAT/CSP on generic problems.
//!
//! confevo's breeds aren't just for Cargo features. They're general-purpose
//! classical AI solvers compatible with wasm4pm's contract format:
//!
//! - **SAT (sat_cdcl)**: Solve any Boolean satisfiability problem
//! - **CSP (csp_ac3)**: Solve any constraint satisfaction problem
//!
//! This example shows:
//! 1. Graph coloring (CSP)
//! 2. Logic puzzle (SAT)
//! 3. Scheduling conflict (SAT)
//!
//! Run with:
//!   cargo run --example raw_breeds

use confevo::breeds::{run_named, Fact};
use confevo::Contract;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Using Breeds for Generic Problems ===\n");

    // Problem 1: Graph coloring with CSP
    println!("Problem 1: Graph Coloring (CSP)");
    println!("  Can we color a triangle with 3 colors such that no adjacent\n  nodes share a color?");
    println!();

    let coloring_contract = Contract::new(
        "color a triangle with 3 colors",
        vec![
            Fact::new("csp-var", "V1:Red,Green,Blue"),
            Fact::new("csp-var", "V2:Red,Green,Blue"),
            Fact::new("csp-var", "V3:Red,Green,Blue"),
            // Constraints: each pair must differ
            Fact::new("csp-constraint", "V1!=V2"),
            Fact::new("csp-constraint", "V2!=V3"),
            Fact::new("csp-constraint", "V1!=V3"),
        ],
    );

    let coloring_result = run_named("csp_ac3", &coloring_contract)?;
    println!("  Result: {}", coloring_result.selected);
    println!("  Explanation: {}", coloring_result.explanation);
    if coloring_result.selected == "sat" {
        for fact in coloring_result.facts.iter() {
            if fact.key == "assign" {
                println!("    {}", fact.value);
            }
        }
    }
    println!();

    // Problem 2: Boolean logic puzzle (SAT)
    println!("Problem 2: Logic Puzzle (SAT)");
    println!(
        "  Three friends: Alice (A), Bob (B), Carol (C).\n\
         Constraints:\n\
         - Exactly one likes pizza (at least one: A OR B OR C)\n\
         - If A likes pizza, B doesn't (NOT(A AND B))\n\
         - If B likes pizza, C doesn't (NOT(B AND C))\n\
         Find: Who can like pizza?"
    );
    println!();

    // Convert to SAT: x1=Alice, x2=Bob, x3=Carol
    // Constraint 1: at least one likes pizza: (x1 OR x2 OR x3)
    // Constraint 2: not both A and B: (NOT x1 OR NOT x2) = (-1 OR -2)
    // Constraint 3: not both B and C: (NOT x2 OR NOT x3) = (-2 OR -3)
    // Constraint 4: at most one... actually, let's check all cases
    let puzzle_contract = Contract::new(
        "pizza preference puzzle",
        vec![
            Fact::new("clause:0", "1 2 3"), // At least one likes pizza
            Fact::new("clause:1", "-1 -2"), // Not both A and B
            Fact::new("clause:2", "-2 -3"), // Not both B and C
        ],
    );

    let puzzle_result = run_named("sat_cdcl", &puzzle_contract)?;
    println!("  Result: {}", puzzle_result.selected);
    println!("  Explanation: {}", puzzle_result.explanation);
    if puzzle_result.selected == "sat" {
        println!("  Solution:");
        for fact in puzzle_result.facts.iter() {
            if fact.key == "assign" {
                let assign = &fact.value;
                if assign == "x1=1" {
                    println!("    Alice likes pizza");
                } else if assign == "x2=1" {
                    println!("    Bob likes pizza");
                } else if assign == "x3=1" {
                    println!("    Carol likes pizza");
                }
            }
        }
    }
    println!();

    // Problem 3: Scheduling conflict (SAT - unsatisfiable)
    println!("Problem 3: Scheduling Conflict (SAT - IMPOSSIBLE)");
    println!(
        "  Two meetings: M1 and M2.\n\
         Constraints:\n\
         - Alice must attend M1 (x1 = true)\n\
         - Bob must attend M2 (x2 = true)\n\
         - If Alice attends M1, then M1 is Tuesday (x3 = true)\n\
         - If Bob attends M2, then M2 is Tuesday (x4 = true)\n\
         - But only one event can be Tuesday: not both (NOT(x3 AND x4))\n\
         Can we schedule this?"
    );
    println!();

    let conflict_contract = Contract::new(
        "impossible meeting schedule",
        vec![
            Fact::new("clause:0", "1"), // Alice must attend M1
            Fact::new("clause:1", "2"), // Bob must attend M2
            Fact::new("clause:2", "-1 3"), // If Alice at M1, then M1 is Tue
            Fact::new("clause:3", "-2 4"), // If Bob at M2, then M2 is Tue
            Fact::new("clause:4", "-3 -4"), // Not both on Tuesday
        ],
    );

    let conflict_result = run_named("sat_cdcl", &conflict_contract)?;
    println!("  Result: {}", conflict_result.selected);
    println!("  Explanation: {}", conflict_result.explanation);
    if conflict_result.selected == "unsat" {
        println!("  Why it's impossible:");
        println!("    1. Alice at M1 → M1 is Tuesday");
        println!("    2. Bob at M2 → M2 is Tuesday");
        println!("    3. But both can't be Tuesday");
        println!("    4. Therefore: CONTRADICTION");
    }
    println!();

    println!("=== Key Insight ===");
    println!(
        "These breeds solve generic problems, not just Cargo features:\n\
         - Graph coloring ✓\n\
         - Scheduling ✓\n\
         - Puzzles ✓\n\
         - Constraint systems ✓\n\
         - Boolean logic ✓\n\
         \n\
         In affidavit, we use them for feature configuration.\n\
         In wasm4pm, they solve the same kinds of problems.\n\
         Use them wherever you have constraints to reason about."
    );

    Ok(())
}
