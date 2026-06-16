#[cfg(feature = "tera")]
use tera::{Context, Tera};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// Snippet Registry structure matching AC-3.1
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snippet {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub language: String,
    pub imports: Vec<String>,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnippetRegistry {
    pub snippets: Vec<Snippet>,
}

impl SnippetRegistry {
    pub fn new() -> Self {
        Self { snippets: Vec::new() }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| anyhow!("Failed to parse snippet registry: {}", e))
    }

    pub fn find_by_name(&self, pattern: &str) -> Vec<&Snippet> {
        let pattern = pattern.to_lowercase();
        self.snippets
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&pattern))
            .collect()
    }

    pub fn filter_by_tag(&self, tag: &str) -> Vec<&Snippet> {
        self.snippets
            .iter()
            .filter(|s| s.tags.iter().any(|t| t == tag))
            .collect()
    }

    pub fn format_snippet(&self, snippet: &Snippet) -> String {
        let mut out = String::new();
        for import in &snippet.imports {
            out.push_str(&format!("use {};\n", import));
        }
        if !snippet.imports.is_empty() {
            out.push('\n');
        }
        out.push_str(&snippet.code);
        out
    }
}

/// Default Snippets for the maximalist suite
pub const DEFAULT_SNIPPETS: &str = r#"{
  "snippets": [
    {
      "name": "chain-build-2-events",
      "tags": ["chain", "basic", "receipt"],
      "description": "Build a 2-event receipt using ChainAssembler",
      "language": "rust",
      "imports": [
        "crate::chain::ChainAssembler",
        "crate::ocel::{build_event, object_ref, SeqCounter}"
      ],
      "code": "let mut asm = ChainAssembler::new();\nlet mut counter = SeqCounter::new();\nlet e0 = build_event(\"build\", vec![object_ref(\"repo:main\", \"git\")], b\"payload-0\", &mut counter)?;\nasm.append(e0)?;\nlet receipt = asm.finalize();"
    },
    {
      "name": "verify-receipt",
      "tags": ["verify", "pipeline", "verdict"],
      "description": "Run the 7-stage certify pipeline on a receipt",
      "language": "rust",
      "imports": ["crate::verifier::verify"],
      "code": "let verdict = verify(&receipt);\nassert!(verdict.accepted, \"reason: {}\", verdict.reason);"
    },
    {
      "name": "tamper-detection",
      "tags": ["tamper", "chain-integrity", "mutation"],
      "description": "Demonstrate that tampering a commitment breaks chain integrity",
      "language": "rust",
      "imports": [
        "crate::types::Blake3Hash",
        "crate::verifier::verify"
      ],
      "code": "let mut tampered = receipt.clone();\ntampered.events[0].payload_commitment = Blake3Hash::from_bytes(b\"evil\");\nlet verdict = verify(&tampered);\nassert!(!verdict.accepted);\nassert_eq!(verdict.outcomes[2].stage, \"chain_integrity\");"
    },
    {
      "name": "object-reference",
      "tags": ["ocel", "object"],
      "description": "Create a reference to an object in an OCEL event",
      "language": "rust",
      "imports": ["crate::ocel::object_ref"],
      "code": "let obj = object_ref(\"user:123\", \"identity\");"
    },
    {
      "name": "seq-counter",
      "tags": ["ocel", "sequence"],
      "description": "Manage event sequence numbers",
      "language": "rust",
      "imports": ["crate::ocel::SeqCounter"],
      "code": "let mut counter = SeqCounter::new();\nlet seq = counter.next();"
    },
    {
        "name": "inspect-receipt",
        "tags": ["inspect", "debug"],
        "description": "Inspect a receipt using the default handler",
        "language": "rust",
        "imports": ["crate::handlers::inspect"],
        "code": "inspect(&receipt);"
    },
    {
        "name": "diff-receipts",
        "tags": ["diff", "debug"],
        "description": "Compare two receipts and print the diff",
        "language": "rust",
        "imports": ["crate::handlers::diff"],
        "code": "diff(&receipt_a, &receipt_b);"
    },
    {
        "name": "blake3-hash",
        "tags": ["hash", "crypto"],
        "description": "Create a Blake3Hash from bytes",
        "language": "rust",
        "imports": ["crate::types::Blake3Hash"],
        "code": "let hash = Blake3Hash::from_bytes(b\"data\");"
    },
    {
        "name": "discover-stages",
        "tags": ["discovery", "pipeline"],
        "description": "Discover all certification stages for a receipt",
        "language": "rust",
        "imports": ["crate::discovery::discover_stages"],
        "code": "let stages = discover_stages(&receipt);"
    }
  ]
}"#;

/// Default Tera Templates
pub const TEST_FN_TEMPLATE: &str = r#"
#[test]
fn test_{{ pattern_name | replace(from="-", to="_") | replace(from=".", to="_") }}() {
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::verifier::verify;

    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    {% for event in events %}
    let event_{{ loop.index0 }} = build_event(
        "{{ event.event_type }}",
        vec![{% for obj in event.objects %}object_ref("{{ obj.id }}", "{{ obj.obj_type }}"){% if not loop.last %}, {% endif %}{% endfor %}],
        b"{{ event.payload }}",
        &mut counter,
    ).expect("build event {{ loop.index0 }}");
    asm.append(event_{{ loop.index0 }}).expect("append event {{ loop.index0 }}");
    {% endfor %}

    let receipt = asm.finalize();
    let verdict = verify(&receipt);

    {% if expected_verdict == "ACCEPT" %}
    assert!(verdict.accepted, "pattern {{ pattern_name }} must ACCEPT; reason: {}", verdict.reason);
    assert_eq!(verdict.reason, "all stages passed");
    {% else %}
    assert!(!verdict.accepted, "pattern {{ pattern_name }} must REJECT");
    assert!(verdict.reason.contains("{{ expected_failure_stage }}"),
        "expected failure at stage '{{ expected_failure_stage }}', got: {}", verdict.reason);
    {% endif %}
}
"#;

pub const TEST_MODULE_TEMPLATE: &str = r#"
//! Auto-generated test module from chicago-tdd fixtures.
//! Source pattern: {{ fixture_set_name }}
//! Generated by: affi generate test
//! Do not edit manually — re-run `affi generate test` to regenerate.

{% for test in tests %}
{{ test }}
{% endfor %}
"#;

#[cfg(feature = "tera")]
pub struct CodegenEngine {
    tera: Tera,
}

#[cfg(feature = "tera")]
impl CodegenEngine {
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("test_fn.tera", TEST_FN_TEMPLATE)?;
        tera.add_raw_template("test_module.tera", TEST_MODULE_TEMPLATE)?;
        Ok(Self { tera })
    }

    pub fn generate_test_function(&self, 
        pattern_name: &str, 
        events: Vec<serde_json::Value>, 
        expected_verdict: &str,
        expected_failure_stage: Option<&str>
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("pattern_name", pattern_name);
        context.insert("events", &events);
        context.insert("expected_verdict", expected_verdict);
        if let Some(stage) = expected_failure_stage {
            context.insert("expected_failure_stage", stage);
        } else {
            context.insert("expected_failure_stage", "");
        }

        self.tera.render("test_fn.tera", &context).map_err(|e| anyhow!("Template error: {}", e))
    }

    pub fn generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) -> Result<String> {
        let mut context = Context::new();
        context.insert("fixture_set_name", fixture_set_name);
        context.insert("tests", &tests);

        self.tera.render("test_module.tera", &context).map_err(|e| anyhow!("Template error: {}", e))
    }
}

#[cfg(not(feature = "tera"))]
pub struct CodegenEngine;

#[cfg(not(feature = "tera"))]
impl CodegenEngine {
    pub fn new() -> Result<Self> {
        Err(anyhow!("feature 'tera' is required for test generation"))
    }
}

pub fn main() -> Result<()> {
    #[cfg(feature = "tera")]
    {
        // Demonstration of the generation logic
        let engine = CodegenEngine::new()?;
        
        let event = serde_json::json!({
            "event_type": "git.commit",
            "objects": [{"id": "repo:1", "obj_type": "repository"}],
            "payload": "initial commit"
        });

        let test_fn = engine.generate_test_function(
            "basic-chain",
            vec![event],
            "ACCEPT",
            None
        )?;

        println!("--- Generated Test Function ---");
        println!("{}", test_fn);

        let module = engine.generate_test_module("golden-suite", vec![test_fn])?;
        println!("--- Generated Test Module ---");
        println!("{}", module);
    }

    let registry = SnippetRegistry::from_json(DEFAULT_SNIPPETS)?;
    let matches = registry.find_by_name("chain");
    
    println!("--- Snippet Search Results ---");
    for snippet in matches {
        println!("Name: {}", snippet.name);
        println!("Description: {}", snippet.description);
        println!("Code:\n{}", registry.format_snippet(snippet));
        println!("---");
    }

    Ok(())
}
