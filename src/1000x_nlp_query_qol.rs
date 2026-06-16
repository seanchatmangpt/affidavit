//! 1000X COMBINATORIAL MAXIMALISM: QOL Innovation.
//!
//! # Natural Language Receipt Querying
//!
//! A local embedded NLP parser that converts natural language queries into exact
//! `OCPQ` (Object-Centric Process Query) ASTs. This enables operators to use
//! human phrases like "show me all failed auth events last week" which are
//! deterministically mapped to the formal OCPQ vocabulary for CLI execution.

use wasm4pm_compat::ocpq::{ObjectScope, OcpqQuery, Predicate, PredicateKind};

/// A local embedded NLP parser for OCPQ queries.
pub struct NlpQueryParser;

impl NlpQueryParser {
    /// Create a new instance of the NLP parser.
    pub fn new() -> Self {
        Self
    }

    /// Convert a natural language query string into an OCPQ AST.
    ///
    /// The parser uses a rule-based token decomposition strategy to identify
    /// activities (Event), states (Object), and time constraints (Temporal).
    pub fn parse(&self, input: &str) -> OcpqQuery {
        let input_lower = input.to_lowercase();
        let words: Vec<&str> = input_lower.split_whitespace().collect();

        // Initialize OCPQ query with an empty global scope.
        // OCPQ queries can be scoped to specific object types, but NLP defaults
        // to a cross-perspective search unless explicitly scoped.
        let mut query = OcpqQuery::new(ObjectScope::new(Vec::<String>::new()));

        // 1. Identify Activity (PredicateKind::Event)
        // Maps words like "auth", "payment", "login" to OCEL activity types.
        let common_activities = [
            "auth", "login", "payment", "order", "shipment", "deploy", 
            "check", "verify", "emit", "assemble"
        ];
        for activity in common_activities {
            if words.contains(&activity) {
                query.predicates.push(Predicate::new(PredicateKind::Event(activity.to_string())));
            }
        }

        // 2. Identify State/Outcome (PredicateKind::Object)
        // In the NLP mapping, adjectives like "failed" or "successful" are treated
        // as object-level predicates that constrain the results.
        let common_states = ["failed", "success", "successful", "pending", "active", "rejected", "accepted"];
        for state in common_states {
            if words.contains(&state) {
                // Map "successful" to "success" for canonical representation
                let canonical = match state {
                    "successful" => "success",
                    s => s
                };
                query.predicates.push(Predicate::new(PredicateKind::Object(canonical.to_string())));
            }
        }

        // 3. Identify Temporal Constraints (PredicateKind::Temporal)
        // Standardize natural time phrases into ISO-ish or OCPQ-stable tokens.
        if input_lower.contains("last week") {
            query.predicates.push(Predicate::new(PredicateKind::Temporal("P7D".into())));
        } else if input_lower.contains("yesterday") {
            query.predicates.push(Predicate::new(PredicateKind::Temporal("P1D".into())));
        } else if input_lower.contains("today") {
            query.predicates.push(Predicate::new(PredicateKind::Temporal("PT0S".into())));
        } else if input_lower.contains("last month") {
            query.predicates.push(Predicate::new(PredicateKind::Temporal("P1M".into())));
        }

        // 4. Identify Explicit Scope
        // "show me all auth events FOR users" -> scope to "user" object type.
        if let Some(pos) = words.iter().position(|&w| w == "for" || w == "on" || w == "over") {
            if pos + 1 < words.len() {
                let target = words[pos + 1].trim_matches(|c: char| !c.is_alphabetic());
                let common_types = ["user", "order", "item", "artifact", "file", "agent"];
                if common_types.contains(&target) {
                    query.scope.object_types.push(target.to_string());
                }
            }
        }

        // 5. Cardinality Hints
        // "at least 5 failed auths"
        if input_lower.contains("at least") {
            if let Some(pos) = words.iter().position(|&w| w == "least") {
                if pos + 1 < words.len() {
                    if let Ok(min) = words[pos + 1].parse::<u32>() {
                        query.predicates.push(Predicate::new(PredicateKind::Cardinality { min, max: u32::MAX }));
                    }
                }
            }
        }

        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_failed_auth_last_week() {
        let parser = NlpQueryParser::new();
        let q = parser.parse("show me all failed auth events last week");
        
        // Activity check
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Event(e) if e == "auth")));
        // State check
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Object(o) if o == "failed")));
        // Temporal check
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Temporal(t) if t == "P7D")));
    }

    #[test]
    fn test_parse_scoped_query_with_cardinality() {
        let parser = NlpQueryParser::new();
        let q = parser.parse("find at least 3 successful login attempts for user");
        
        assert_eq!(q.scope.object_types, vec!["user".to_string()]);
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Event(e) if e == "login")));
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Object(o) if o == "success")));
        assert!(q.predicates.iter().any(|p| matches!(&p.kind, PredicateKind::Cardinality { min: 3, .. })));
    }

    #[test]
    fn test_parse_empty_query_is_safe() {
        let parser = NlpQueryParser::new();
        let q = parser.parse("");
        assert!(q.predicates.is_empty());
        assert!(q.scope.is_empty());
    }
}
