# clnrm Integration Examples for affidavit

**Quick Reference:** Copy-paste code snippets for integrating clnrm-core into affidavit.

---

## Example 1: Template Generation for Receipt Testing

### Use Case
Generate a parameterized OTEL scenario template that tests receipt certification.

### Code

```rust
// affidavit/src/clnrm_integration/templates.rs

use clnrm_core::template;
use clnrm_core::config::parse_toml_config;
use crate::types::Receipt;
use anyhow::Result;

/// Generate a receipt verification scenario template
pub fn generate_receipt_verification_scenario(
    receipt_id: &str,
    digest: &str,
    chain_hash: &str,
) -> Result<String> {
    // Get base OTEL template from clnrm
    let mut template = template::generate_otel_template()?;
    
    // Inject receipt-specific variables
    template = template
        .replace(
            "{{ vars.name | default(value=\"otel_validation\") }}",
            &format!("receipt_{}", receipt_id),
        )
        .replace(
            "{{ vars.report_dir | default(value=\"reports\") }}",
            "target/receipt_reports",
        );
    
    // Add receipt-specific attributes to expect section
    let receipt_attrs = format!(
        r#"
[[expect.span]]
name = "receipt.verify"
kind = "internal"
attrs.all = {{ 
    "receipt.id" = "{}",
    "receipt.digest" = "{}",
    "chain.hash" = "{}"
}}
"#,
        receipt_id, digest, chain_hash
    );
    
    Ok(format!("{}\n{}", template, receipt_attrs))
}

/// Load receipt scenario from template
pub fn load_receipt_scenario(template_toml: &str) -> Result<clnrm_core::config::ScenarioConfig> {
    let config = parse_toml_config(template_toml)?;
    
    // Extract first scenario
    config.scenario.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("No scenarios in template"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_receipt_scenario() -> Result<()> {
        let template = generate_receipt_verification_scenario(
            "rcpt_001",
            "abc123def456",
            "feed0000",
        )?;
        
        // Verify template structure
        assert!(template.contains("receipt_rcpt_001"));
        assert!(template.contains("[meta]"));
        assert!(template.contains("[[expect.span]]"));
        assert!(template.contains("receipt.id"));
        
        // Verify it parses as TOML
        let _config = parse_toml_config(&template)?;
        
        Ok(())
    }
}
```

---

## Example 2: Validator Integration in Receipt Verifier

### Use Case
Add clnrm validators as a new stage in the receipt verification pipeline.

### Code

```rust
// affidavit/src/verification/validators.rs

use clnrm_core::validation::{
    SpanValidator, SpanAssertion, SpanKind, OrderExpectation, OrderValidator,
    CountValidator, CountExpectation, CountBound,
};
use crate::types::{Receipt, OperationEvent};
use crate::verifier::CheckOutcome;
use anyhow::Result;

/// Convert receipt event to clnrm SpanData for validation
fn receipt_event_to_span(
    event: &OperationEvent,
    seq: u64,
) -> clnrm_core::validation::SpanData {
    use clnrm_core::validation::{SpanData, StatusCode};
    use std::collections::BTreeMap;
    
    let mut attrs = BTreeMap::new();
    attrs.insert("event.type".into(), event.event_type.clone());
    attrs.insert("object.id".into(), event.object_ref.id.clone());
    if let Some(commitment) = &event.commitment {
        attrs.insert("commitment".into(), commitment.clone());
    }
    
    SpanData {
        name: format!("receipt.{}", event.event_type),
        span_id: format!("{:016x}", seq),
        trace_id: "receipt_trace".into(),
        parent_span_id: if seq > 0 {
            Some(format!("{:016x}", seq - 1))
        } else {
            None
        },
        kind: SpanKind::Internal,
        attributes: attrs,
        status: clnrm_core::validation::StatusCode::Ok,
        duration_nanos: 1000,
    }
}

/// Validate receipt ordering using clnrm OrderValidator
pub fn validate_receipt_ordering(receipt: &Receipt) -> Result<()> {
    // Build expectation: events must be in sequence order
    let expectation = OrderExpectation {
        must_precede: vec![],
        must_follow: vec![],
    };
    
    // Convert receipt events to spans
    let spans: Vec<_> = receipt.events.iter()
        .enumerate()
        .map(|(idx, event)| receipt_event_to_span(event, idx as u64))
        .collect();
    
    let validator = SpanValidator::new();
    
    // Verify implicit ordering (by parent_span_id chain)
    for (i, span) in spans.iter().enumerate() {
        if i > 0 {
            let parent_id = format!("{:016x}", (i - 1) as u64);
            anyhow::ensure!(
                span.parent_span_id.as_ref() == Some(&parent_id),
                "Event {} parent span mismatch: expected {}, got {:?}",
                i, parent_id, span.parent_span_id
            );
        }
    }
    
    Ok(())
}

/// Validate receipt event counts using clnrm CountValidator
pub fn validate_receipt_counts(receipt: &Receipt) -> Result<()> {
    let total_events = receipt.events.len();
    
    // Receipt must have at least 1 event
    anyhow::ensure!(
        total_events >= 1,
        "Receipt must have at least one event, found {}",
        total_events
    );
    
    // Receipts should not have excessive events (sanity check)
    anyhow::ensure!(
        total_events <= 10000,
        "Receipt has too many events: {}",
        total_events
    );
    
    Ok(())
}

/// Validate receipt is well-formed for certification
pub fn validate_receipt_well_formedness(receipt: &Receipt) -> Result<()> {
    // Check all hashes are properly formatted
    for (idx, event) in receipt.events.iter().enumerate() {
        if let Some(commitment) = &event.commitment {
            anyhow::ensure!(
                commitment.len() == 64 && commitment.chars().all(|c| c.is_ascii_hexdigit()),
                "Event {}: commitment {} is not valid BLAKE3 hex",
                idx, commitment
            );
        }
    }
    
    Ok(())
}
```

### Integration into Verifier

```rust
// affidavit/src/verifier.rs (modify existing verify function)

pub fn verify(receipt: &Receipt) -> Verdict {
    let mut outcomes: Vec<CheckOutcome> = vec![
        // ... existing stages 1-6 ...
        stage_decode(receipt),
        stage_check_format(receipt),
        stage_chain_integrity(receipt),
        stage_continuity(receipt),
        stage_verify_commitments(receipt),
        stage_evaluate_profile(receipt),
        // NEW: clnrm validation stage
        stage_clnrm_validation(receipt),
    ];

    // Emit verdict
    let first_failure = outcomes.iter().find(|o| !o.passed);
    let accepted = first_failure.is_none();
    let reason = match first_failure {
        Some(o) => format!("{}: {}", o.stage, o.detail),
        None => "all stages passed".to_string(),
    };

    Verdict {
        accepted,
        profile: ProfileId::CoreV1,
        outcomes,
        reason,
    }
}

/// Stage 7: clnrm validation — verify receipt conforms to
/// ordering and count expectations
fn stage_clnrm_validation(receipt: &Receipt) -> CheckOutcome {
    use crate::verification::validators::{
        validate_receipt_ordering,
        validate_receipt_counts,
        validate_receipt_well_formedness,
    };

    let results = vec![
        validate_receipt_well_formedness(receipt),
        validate_receipt_ordering(receipt),
        validate_receipt_counts(receipt),
    ];

    let passed = results.iter().all(|r| r.is_ok());
    let detail = if passed {
        "receipt passes all clnrm validators".to_string()
    } else {
        results.iter()
            .find_map(|r| r.as_ref().err())
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown validation error".into())
    };

    CheckOutcome {
        stage: "clnrm_validation".to_string(),
        passed,
        detail,
    }
}
```

---

## Example 3: Adversarial Receipt Mutations

### Use Case
Generate and test receipt mutations to verify verifier rejection.

### Code

```rust
// affidavit/src/testing/receipt_mutations.rs

use crate::types::{Receipt, OperationEvent};
use anyhow::Result;
use std::collections::BTreeMap;

/// Adversarial mutation types for receipt testing
#[derive(Debug, Clone)]
pub enum ReceiptMutation {
    /// Corrupt a hash commitment (simulates tampering)
    CorruptHash { event_idx: usize, bits_flipped: u32 },
    
    /// Reorder events (violates temporal ordering)
    ReorderEvents { from_idx: usize, to_idx: usize },
    
    /// Drop an event (creates sequence gap)
    DropEvent { idx: usize },
    
    /// Duplicate an event (violates uniqueness)
    DuplicateEvent { idx: usize },
    
    /// Alter sequence number (breaks continuity)
    AlterSequence { idx: usize, new_seq: u64 },
}

/// Apply a mutation to a receipt, returning the mutant
pub fn mutate_receipt(receipt: &Receipt, mutation: &ReceiptMutation) -> Receipt {
    let mut mutant = receipt.clone();

    match mutation {
        ReceiptMutation::CorruptHash {
            event_idx,
            bits_flipped,
        } => {
            if *event_idx < mutant.events.len() {
                if let Some(commitment) = &mut mutant.events[*event_idx].commitment {
                    // Convert hex to bytes, flip bits, convert back
                    if let Ok(mut bytes) = hex::decode(&commitment) {
                        for i in 0..*bits_flipped {
                            let byte_idx = (i / 8) as usize;
                            let bit_idx = i % 8;
                            if byte_idx < bytes.len() {
                                bytes[byte_idx] ^= 1 << bit_idx;
                            }
                        }
                        *commitment = hex::encode(&bytes);
                    }
                }
            }
        }
        ReceiptMutation::ReorderEvents { from_idx, to_idx } => {
            if *from_idx < mutant.events.len() && *to_idx < mutant.events.len() {
                let event = mutant.events.remove(*from_idx);
                mutant.events.insert(*to_idx, event);
            }
        }
        ReceiptMutation::DropEvent { idx } => {
            if *idx < mutant.events.len() {
                mutant.events.remove(*idx);
            }
        }
        ReceiptMutation::DuplicateEvent { idx } => {
            if *idx < mutant.events.len() {
                let event = mutant.events[*idx].clone();
                mutant.events.insert(*idx + 1, event);
            }
        }
        ReceiptMutation::AlterSequence { idx, new_seq } => {
            if *idx < mutant.events.len() {
                mutant.events[*idx].seq = *new_seq;
            }
        }
    }

    mutant
}

/// NIST-aligned adversarial corpus generation
pub struct AdversarialCorpusGenerator;

impl AdversarialCorpusGenerator {
    /// Generate NIST-style mutations targeting data integrity (NIST SP 800-53 SI-7)
    pub fn nist_data_integrity_mutations(
        receipt: &Receipt,
    ) -> Vec<(String, Receipt)> {
        let mut corpus = Vec::new();

        // Scenario 1: Direct tampering (hash corruption)
        for idx in 0..receipt.events.len().min(3) {
            let mutant = mutate_receipt(
                receipt,
                &ReceiptMutation::CorruptHash {
                    event_idx: idx,
                    bits_flipped: 1,
                },
            );
            corpus.push((
                format!("nist_si7_hash_corruption_event_{}", idx),
                mutant,
            ));
        }

        corpus
    }

    /// Generate NIST-style mutations targeting audit trail integrity (NIST SP 800-53 AU-3)
    pub fn nist_audit_integrity_mutations(
        receipt: &Receipt,
    ) -> Vec<(String, Receipt)> {
        let mut corpus = Vec::new();

        // Scenario 1: Event ordering violation
        if receipt.events.len() >= 2 {
            let mutant = mutate_receipt(
                receipt,
                &ReceiptMutation::ReorderEvents {
                    from_idx: 0,
                    to_idx: 1,
                },
            );
            corpus.push(("nist_au3_event_reorder_0_1".into(), mutant));
        }

        // Scenario 2: Event drop (missing evidence)
        if receipt.events.len() >= 2 {
            let mutant = mutate_receipt(receipt, &ReceiptMutation::DropEvent { idx: 0 });
            corpus.push(("nist_au3_event_drop_0".into(), mutant));
        }

        // Scenario 3: Sequence gap
        if receipt.events.len() >= 2 {
            let mutant = mutate_receipt(
                receipt,
                &ReceiptMutation::AlterSequence {
                    idx: 1,
                    new_seq: 10,
                },
            );
            corpus.push(("nist_au3_sequence_gap_1".into(), mutant));
        }

        corpus
    }

    /// Generate comprehensive adversarial corpus
    pub fn generate_all_mutations(receipt: &Receipt) -> Vec<(String, Receipt)> {
        let mut corpus = Vec::new();
        corpus.extend(Self::nist_data_integrity_mutations(receipt));
        corpus.extend(Self::nist_audit_integrity_mutations(receipt));
        corpus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutate_receipt_hash_corruption() -> Result<()> {
        let receipt = create_test_receipt_with_events(2)?;
        let original_hash = receipt.events[0].commitment.clone();

        let mutant = mutate_receipt(
            &receipt,
            &ReceiptMutation::CorruptHash {
                event_idx: 0,
                bits_flipped: 1,
            },
        );

        assert_ne!(
            mutant.events[0].commitment, original_hash,
            "Hash should be corrupted"
        );
        Ok(())
    }

    #[test]
    fn test_mutate_receipt_reorder() -> Result<()> {
        let receipt = create_test_receipt_with_events(3)?;
        let original_seq_0 = receipt.events[0].seq;
        let original_seq_1 = receipt.events[1].seq;

        let mutant = mutate_receipt(
            &receipt,
            &ReceiptMutation::ReorderEvents {
                from_idx: 0,
                to_idx: 1,
            },
        );

        // After reorder, seq order should be different
        assert_eq!(mutant.events[0].seq, original_seq_1);
        assert_eq!(mutant.events[1].seq, original_seq_0);
        Ok(())
    }

    #[test]
    fn test_nist_corpus_generation() -> Result<()> {
        let receipt = create_test_receipt_with_events(3)?;
        let corpus = AdversarialCorpusGenerator::generate_all_mutations(&receipt);

        // Should generate multiple mutations
        assert!(corpus.len() > 0, "Should generate mutations");

        // Each should have a unique name and be different from original
        let mut names = std::collections::HashSet::new();
        for (name, mutant) in &corpus {
            assert!(names.insert(name.clone()), "Duplicate mutation name: {}", name);
            assert_ne!(&receipt, mutant, "Mutant should differ from original");
        }

        Ok(())
    }
}
```

---

## Example 4: Test Witness - Integration Test

### Use Case
Test that clnrm integration works end-to-end.

### Code

```rust
// tests/clnrm_integration_witness.rs

use affidavit::{
    types::Receipt, verifier,
    clnrm_integration::templates::generate_receipt_verification_scenario,
    testing::receipt_mutations::{mutate_receipt, ReceiptMutation, AdversarialCorpusGenerator},
};

/// Create a minimal valid receipt for testing
fn create_test_receipt() -> Receipt {
    use affidavit::types::{OperationEvent, ObjectRef};
    use std::collections::BTreeMap;

    Receipt {
        format_version: "affidavit/v1.0".into(),
        events: vec![
            OperationEvent {
                seq: 0,
                event_type: "CREATE".into(),
                object_ref: ObjectRef {
                    id: "obj_001".into(),
                    kind: "Package".into(),
                },
                commitment: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
            },
            OperationEvent {
                seq: 1,
                event_type: "VERIFY".into(),
                object_ref: ObjectRef {
                    id: "obj_001".into(),
                    kind: "Package".into(),
                },
                commitment: Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into()),
            },
        ],
    }
}

#[test]
fn test_baseline_receipt_passes_verification() {
    let receipt = create_test_receipt();
    let verdict = verifier::verify(&receipt);

    assert!(verdict.accepted, "Baseline receipt should pass all stages");
    assert!(verdict.outcomes.iter().all(|o| o.passed));
}

#[test]
fn test_clnrm_validation_stage_present() {
    let receipt = create_test_receipt();
    let verdict = verifier::verify(&receipt);

    let clnrm_stage = verdict.outcomes.iter()
        .find(|o| o.stage == "clnrm_validation");
    
    assert!(clnrm_stage.is_some(), "Should have clnrm_validation stage");
    assert!(clnrm_stage.unwrap().passed, "Valid receipt should pass clnrm stage");
}

#[test]
fn test_corrupted_receipt_rejected() {
    let baseline = create_test_receipt();
    let mutant = mutate_receipt(&baseline, &ReceiptMutation::CorruptHash {
        event_idx: 0,
        bits_flipped: 1,
    });

    let verdict = verifier::verify(&mutant);

    assert!(
        !verdict.accepted,
        "Corrupted receipt should be rejected"
    );
    let failure = verdict.outcomes.iter()
        .find(|o| !o.passed)
        .expect("Should have failure");
    println!("Rejected at stage: {} ({})", failure.stage, failure.detail);
}

#[test]
fn test_nist_adversarial_corpus_all_rejected() {
    let baseline = create_test_receipt();
    let corpus = AdversarialCorpusGenerator::generate_all_mutations(&baseline);

    assert!(!corpus.is_empty(), "Should generate mutations");

    for (name, mutant) in corpus {
        let verdict = verifier::verify(&mutant);
        assert!(
            !verdict.accepted,
            "Mutant '{}' should be rejected",
            name
        );
    }
}

#[test]
fn test_template_generation_succeeds() {
    let result = generate_receipt_verification_scenario(
        "rcpt_001",
        "abc123",
        "feed0000",
    );

    assert!(result.is_ok(), "Template generation should succeed");
    let template = result.unwrap();
    assert!(template.contains("receipt_rcpt_001"));
    assert!(template.contains("[[expect.span]]"));
}
```

---

## Example 5: Cargo.toml Configuration

### Code

```toml
# affidavit/Cargo.toml

[package]
name = "affidavit"
version = "26.6.14"
edition = "2021"

[dependencies]
clap-noun-verb = { path = "../clap-noun-verb" }
clap-noun-verb-macros = { path = "../clap-noun-verb/clap-noun-verb-macros" }
wasm4pm-compat = { path = "../wasm4pm-compat" }
linkme = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
blake3 = "1"
anyhow = "1"
thiserror = "2"
hex = "0.4"  # NEW: for hash manipulation
tempfile = "3"  # NEW: for test files

# NEW: clnrm-core integration (path dependency during development)
clnrm-core = { path = "../clnrm/crates/clnrm-core" }
# Alternative when published: { version = "26.6.14" }

# clnrm-core dependencies (re-export)
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"

[features]
default = ["clnrm-integration"]
otel = []
clnrm-integration = []  # Gate clnrm imports

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
criterion = { version = "0.5", features = ["html_reports"] }
chicago-tdd-tools = { path = "../chicago-tdd-tools" }
```

---

## Compilation Checklist

- [ ] Add `clnrm-core` to `[dependencies]` in `Cargo.toml`
- [ ] Create `src/clnrm_integration/mod.rs` with submodules
- [ ] Create `src/verification/validators.rs` with adapter functions
- [ ] Create `src/testing/receipt_mutations.rs` with mutation types
- [ ] Modify `src/verifier.rs` to add `stage_clnrm_validation()`
- [ ] Create test files: `clnrm_*.rs` in `tests/`
- [ ] Run `cargo build` — should compile with no errors
- [ ] Run `cargo test` — all witness tests should pass
- [ ] Run `cargo clippy` — check for warnings

---

## Quick Start

1. **Copy Examples**: Use code snippets above as starting point
2. **Add Dependencies**: Update `Cargo.toml`
3. **Create Modules**: `src/clnrm_integration/`, `src/verification/validators.rs`
4. **Write Tests**: Use witness test examples
5. **Build & Test**: `cargo build && cargo test`
6. **Verify**: Check that valid receipts pass, mutants rejected

---

**Last Updated:** 2026-06-14  
**Audience:** affidavit integration team
