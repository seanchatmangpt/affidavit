//! Logic for the `affi receipt catalog` verb (Feature 1.4).
//!
//! This module provides the filtering and listing logic for the receipt fixture
//! database, allowing users to discover and inspect available test cases.

use crate::fixture_db::{Fixture, FixtureDatabase, FixtureQuery};

/// Filter and list fixtures from the database based on name and event count.
///
/// This implements the core logic for the catalog verb, supporting case-insensitive
/// substring matching for names and exact matching for event counts.
pub fn list_fixtures(
    db: &FixtureDatabase,
    name_filter: Option<String>,
    events_filter: Option<usize>,
) -> Vec<Fixture> {
    let query = FixtureQuery {
        name_contains: name_filter,
        min_events: events_filter,
        max_events: events_filter,
        ..Default::default()
    };

    db.search(&query)
}

/// Format a list of fixtures as a human-readable table string.
pub fn format_catalog(fixtures: &[Fixture]) -> String {
    if fixtures.is_empty() {
        return "No fixtures match the specified filters.\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{:<20} {:<10} {:<30}\n",
        "Name", "Events", "Description"
    ));
    output.push_str(&format!("{:-<20} {:-<10} {:-<30}\n", "", "", ""));

    for f in fixtures {
        // Use tags as a makeshift description if no dedicated field exists
        let description = if f.tags.is_empty() {
            "(no description)".to_string()
        } else {
            f.tags.join(", ")
        };

        output.push_str(&format!(
            "{:<20} {:<10} {:<30}\n",
            f.name, f.event_count, description
        ));
    }

    output
}
