#![cfg(feature = "otel")]
//! COMBINATORIAL MAXIMALISM: Features 4.1/4.3 (Tracing/Baggage)
//!
//! Full OTel tracer instrumentation across the 7-stage verifier,
//! including baggage propagation for receipt IDs and versions.
//!
//! This implementation demonstrates "maximalist" tracing where every stage
//! of the 7-stage pipeline is instrumented with its own span, and receipt
//! metadata is propagated via OTel baggage.

use affidavit::chain::{recompute_chain, FORMAT_VERSION};
use affidavit::types::{CheckOutcome, ProfileId, Receipt, Verdict};
use opentelemetry::{
    baggage::BaggageExt,
    trace::{Span, TraceContextExt, Tracer, TracerProvider},
    Context, KeyValue,
};
use std::collections::BTreeSet;

/// The format version this verifier knows how to certify.
const STANDARD_VERSION: &str = FORMAT_VERSION;

/// Expected hex length of a BLAKE3-256 digest (32 bytes → 64 hex chars).
const BLAKE3_HEX_LEN: usize = 64;

/// Whether a hex string is a well-formed lowercase BLAKE3-256 digest.
fn is_well_formed_hash(hex: &str) -> bool {
    hex.len() == BLAKE3_HEX_LEN
        && hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
}

/// Certify a receipt with full OTel instrumentation and baggage propagation.
///
/// This is the "maximalist" version of `affidavit::verifier::verify`.
pub fn verify_maximalist<T: Tracer>(receipt: &Receipt, tracer: &T) -> Verdict {
    // 1. Setup Baggage: Propagate receipt identity and version.
    let cx = Context::current().with_baggage(vec![
        KeyValue::new("receipt.id", receipt.chain_hash.to_string()),
        KeyValue::new("receipt.version", receipt.format_version.clone()),
    ]);

    // 2. Open parent verify span.
    let mut parent_span = tracer.start_with_context("verify", &cx);
    parent_span.set_attribute(KeyValue::new("event_count", receipt.events.len() as i64));
    let cx = cx.with_span(parent_span);

    // 3. Run the 7-stage pipeline, each with its own child span.
    let mut outcomes = Vec::new();

    // Stage 1: decode
    outcomes.push(trace_stage(tracer, &cx, "decode", || stage_decode(receipt)));

    // Stage 2: check_format
    outcomes.push(trace_stage(tracer, &cx, "check_format", || {
        stage_check_format(receipt)
    }));

    // Stage 3: chain_integrity
    outcomes.push(trace_stage(tracer, &cx, "chain_integrity", || {
        stage_chain_integrity(receipt)
    }));

    // Stage 4: continuity
    outcomes.push(trace_stage(tracer, &cx, "continuity", || {
        stage_continuity(receipt)
    }));

    // Stage 5: verify_commitments
    outcomes.push(trace_stage(tracer, &cx, "verify_commitments", || {
        stage_verify_commitments(receipt)
    }));

    // Stage 6: evaluate_profile
    outcomes.push(trace_stage(tracer, &cx, "evaluate_profile", || {
        stage_evaluate_profile(receipt)
    }));

    // Stage 7: emit_verdict
    let verdict = trace_stage(tracer, &cx, "emit_verdict", || {
        let first_failure = outcomes.iter().find(|o| !o.passed);
        let accepted = first_failure.is_none();
        let reason = match first_failure {
            Some(o) => format!("{}: {}", o.stage, o.detail),
            None => "all stages passed".to_string(),
        };

        Verdict {
            accepted,
            profile: ProfileId::CoreV1,
            outcomes: outcomes.clone(),
            reason,
        }
    });

    let mut parent_span = cx.span();
    parent_span.set_attribute(KeyValue::new("accepted", verdict.accepted));
    parent_span.add_event(
        "verdict_emitted",
        vec![KeyValue::new("reason", verdict.reason.clone())],
    );
    parent_span.end();

    verdict
}

/// Helper to wrap a stage in a child span with outcome attributes.
fn trace_stage<T: Tracer, R, F>(tracer: &T, cx: &Context, name: &'static str, f: F) -> R
where
    F: FnOnce() -> R,
    R: IntoOutcome + Clone,
{
    let mut span = tracer.start_with_context(name, cx);
    let result = f();
    let outcome = result.clone().into_outcome();
    span.set_attribute(KeyValue::new("stage.name", name));
    span.set_attribute(KeyValue::new("stage.passed", outcome.passed));
    span.set_attribute(KeyValue::new("stage.detail", outcome.detail));
    span.end();
    result
}

/// Trait to extract CheckOutcome from either CheckOutcome or Verdict.
trait IntoOutcome {
    fn into_outcome(self) -> CheckOutcome;
}

impl IntoOutcome for CheckOutcome {
    fn into_outcome(self) -> CheckOutcome {
        self
    }
}

impl IntoOutcome for Verdict {
    fn into_outcome(self) -> CheckOutcome {
        CheckOutcome {
            stage: "emit_verdict".to_string(),
            passed: self.accepted,
            detail: self.reason,
        }
    }
}

// --- Implementation of the 6 core stages (mirrored from src/verifier.rs) ---

fn stage_decode(receipt: &Receipt) -> CheckOutcome {
    let passed = !receipt.format_version.trim().is_empty();
    let detail = if passed {
        format!("{} event(s), format_version present", receipt.events.len())
    } else {
        "format_version is empty or unparseable".to_string()
    };
    CheckOutcome {
        stage: "decode".to_string(),
        passed,
        detail,
    }
}

fn stage_check_format(receipt: &Receipt) -> CheckOutcome {
    let passed = receipt.format_version == STANDARD_VERSION;
    let detail = if passed {
        format!("format_version == {STANDARD_VERSION}")
    } else {
        format!(
            "expected format_version {STANDARD_VERSION}, found {}",
            receipt.format_version
        )
    };
    CheckOutcome {
        stage: "check_format".to_string(),
        passed,
        detail,
    }
}

fn stage_chain_integrity(receipt: &Receipt) -> CheckOutcome {
    match recompute_chain(&receipt.events) {
        Ok(computed) => {
            let passed = computed == receipt.chain_hash;
            let detail = if passed {
                "recomputed chain hash matches stored chain_hash".to_string()
            } else {
                format!(
                    "chain hash mismatch: stored {}, recomputed {}",
                    receipt.chain_hash, computed
                )
            };
            CheckOutcome {
                stage: "chain_integrity".to_string(),
                passed,
                detail,
            }
        }
        Err(e) => CheckOutcome {
            stage: "chain_integrity".to_string(),
            passed: false,
            detail: format!("could not canonicalize an event: {e}"),
        },
    }
}

fn stage_continuity(receipt: &Receipt) -> CheckOutcome {
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for (index, event) in receipt.events.iter().enumerate() {
        let expected_seq = index as u64;
        if event.seq != expected_seq {
            return CheckOutcome {
                stage: "continuity".to_string(),
                passed: false,
                detail: format!(
                    "seq gap at position {index}: expected {expected_seq}, found {}",
                    event.seq
                ),
            };
        }
        if !seen_ids.insert(event.id.as_str()) {
            return CheckOutcome {
                stage: "continuity".to_string(),
                passed: false,
                detail: format!("duplicate event id: {}", event.id),
            };
        }
    }
    CheckOutcome {
        stage: "continuity".to_string(),
        passed: true,
        detail: format!(
            "{} event(s) with contiguous seq and unique ids",
            receipt.events.len()
        ),
    }
}

fn stage_verify_commitments(receipt: &Receipt) -> CheckOutcome {
    for event in &receipt.events {
        let hex = event.payload_commitment.as_hex();
        if !is_well_formed_hash(hex) {
            return CheckOutcome {
                stage: "verify_commitments".to_string(),
                passed: false,
                detail: format!(
                    "event {} has a malformed commitment (expected {BLAKE3_HEX_LEN} lowercase hex chars)",
                    event.id
                ),
            };
        }
    }
    CheckOutcome {
        stage: "verify_commitments".to_string(),
        passed: true,
        detail: "all commitments are well-formed BLAKE3 digests".to_string(),
    }
}

fn stage_evaluate_profile(receipt: &Receipt) -> CheckOutcome {
    for event in &receipt.events {
        if event.event_type.trim().is_empty() {
            return CheckOutcome {
                stage: "evaluate_profile".to_string(),
                passed: false,
                detail: format!("event {} has an empty event_type", event.id),
            };
        }
        if event.payload_commitment.as_hex().is_empty() {
            return CheckOutcome {
                stage: "evaluate_profile".to_string(),
                passed: false,
                detail: format!("event {} is missing a commitment", event.id),
            };
        }
    }
    CheckOutcome {
        stage: "evaluate_profile".to_string(),
        passed: true,
        detail: format!("profile {} satisfied", ProfileId::CoreV1.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use affidavit::chain::ChainAssembler;
    use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent};
    use opentelemetry::sdk::trace::{Config, Sampler, TracerProvider as SdkTracerProvider};

    fn event(id: &str, seq: u64, event_type: &str, payload: &[u8]) -> OperationEvent {
        OperationEvent {
            id: id.to_string(),
            seq,
            event_type: event_type.to_string(),
            objects: vec![ObjectRef {
                id: format!("obj-{id}"),
                obj_type: "artifact".to_string(),
                qualifier: None,
            }],
            payload_commitment: Blake3Hash::from_bytes(payload),
        }
    }

    fn valid_receipt() -> Receipt {
        let mut asm = ChainAssembler::new();
        asm.append(event("e0", 0, "emit", b"payload-zero")).unwrap();
        asm.finalize()
    }

    #[test]
    fn test_verify_maximalist_tracing() {
        // Setup a mock/test tracer provider.
        let provider = SdkTracerProvider::builder()
            .with_config(Config::default().with_sampler(Sampler::AlwaysOn))
            .build();
        let tracer = provider.tracer("test-maximalist");

        let receipt = valid_receipt();
        let verdict = verify_maximalist(&receipt, &tracer);

        assert!(verdict.accepted);
        assert_eq!(verdict.outcomes.len(), 6);
    }
}
