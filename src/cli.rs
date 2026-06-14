//! Command implementations for the `affi` CLI: emit, assemble, verify, show.
//!
//! Owned by the `cli` phase-2 agent; wired by the integration agent to the real
//! sibling modules. Codes against `crate::types` for shapes, and delegates the
//! load-bearing algorithms to their canonical owners:
//!
//! - object parsing and event building → [`crate::ocel`]
//! - rolling chain hash, content addressing, persistence → [`crate::chain`]
//! - the certify pipeline → [`crate::verifier`]
//!
//! Nothing here re-implements hashing or the chain rule; doing so previously
//! caused a format-version / genesis-seed split that made assembled receipts
//! un-verifiable.

use crate::chain;
use crate::ocel;
use crate::types::{ObjectRef, Receipt};
use crate::verifier;
use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::PathBuf;

/// Append one operation-event to the working receipt (`.affi/working.json`).
///
/// `objects` are `id:type` pairs (optionally `id:type:qualifier`). The payload
/// is read from `payload` (a file path, or `-` for stdin) and its BLAKE3 digest
/// becomes the commitment — the raw payload is never stored in the receipt.
/// Prints the new event's id and seq.
#[allow(clippy::print_stdout)]
pub fn emit(event_type: &str, objects: &[String], payload: &str) -> Result<()> {
    if event_type.trim().is_empty() {
        bail!("--type must be a non-empty event_type");
    }

    let payload_bytes =
        read_payload(payload).with_context(|| format!("reading payload from {payload:?}"))?;

    let object_refs = objects
        .iter()
        .map(|spec| ocel::parse_object_ref(spec))
        .collect::<Result<Vec<ObjectRef>, _>>()
        .context("parsing object specs (expected id:type or id:type:qualifier)")?;

    // Load-or-create the working list, then derive the next seq from its length
    // via the deterministic counter so the new event continues the sequence.
    let mut events = chain::load_working().context("loading working receipt")?;
    let mut counter = ocel::SeqCounter::starting_at(events.len() as u64);

    // `ocel::build_event` computes the commitment (blake3 of payload bytes),
    // assigns seq/id from the counter, and validates well-formedness.
    let event = ocel::build_event(event_type, object_refs, &payload_bytes, &mut counter)
        .context("building operation-event")?;

    let id = event.id.clone();
    let seq = event.seq;
    events.push(event);
    chain::save_working(&events).context("saving working receipt")?;

    println!("emitted event {id} (seq {seq})");
    Ok(())
}

/// Finalize the working receipt into an immutable, content-addressed receipt
/// file (name = BLAKE3 of canonical bytes), or to `out` if provided.
///
/// Builds the receipt through [`chain::ChainAssembler`] so the rolling chain
/// hash and format version match exactly what [`crate::verifier`] expects, then
/// content-addresses it. Prints the receipt path and content address.
#[allow(clippy::print_stdout)]
pub fn assemble(out: Option<&str>) -> Result<()> {
    let events = chain::load_working().context("loading working receipt")?;
    if events.is_empty() {
        bail!(
            "nothing to assemble: working receipt {} has no events (run `affi emit` first)",
            chain::WORKING_PATH
        );
    }

    let assembler =
        chain::ChainAssembler::from_events(events).context("reconstructing chain assembler")?;
    let receipt = assembler.finalize();

    let address = chain::content_address(&receipt).context("content-addressing receipt")?;

    let path: PathBuf = match out {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(format!("{address}.json")),
    };

    chain::save_receipt(&receipt, &path)
        .with_context(|| format!("writing receipt to {path:?}"))?;

    println!("assembled receipt -> {}", path.display());
    println!("content address: {address}");
    Ok(())
}

/// Run the certify pipeline over `receipt` and print per-stage outcomes plus
/// the final verdict. Returns the process exit code (0 ACCEPT, non-zero REJECT).
pub fn verify(receipt: &str) -> Result<(i32, crate::types::Verdict)> {
    // The verify work runs INSIDE the span — the span wraps the adjudication,
    // not a throwaway closure. The span is recorded observably (tracing.rs).
    crate::tracing::trace_verify(receipt, || {
        let parsed = load_receipt(receipt)?;

        // Adjudicate through the REAL Layer 2 gate: admission::admit runs BOTH
        // the wasm4pm-compat OCEL court AND the affidavit certify pipeline. This
        // is what makes the OCEL court load-bearing in production — a receipt
        // the verifier alone would accept (e.g. objectless) is REJECTED here
        // because admit() refuses it. The detailed per-stage Verdict is still
        // produced for display.
        let verdict = verifier::verify(&parsed);
        match crate::admission::admit(parsed) {
            Ok(_admitted) => Ok((0, verdict)),
            Err(refusal) => {
                // A named refusal → REJECT, non-zero exit. Surface the refusal
                // in the verdict reason so the operator sees WHY (e.g.
                // "ocel_law_violation: EmptyEventObjectLinks").
                let rejected = crate::types::Verdict {
                    accepted: false,
                    reason: format!("admission refused: {refusal}"),
                    ..verdict
                };
                Ok((2, rejected))
            }
        }
    })
}

/// Print a human-readable dump of the receipt chain at `receipt`.
///
/// Lists each event with its seq, type, object refs, and a short commitment,
/// then the final chain hash.
///
/// `show` is the NON-adjudicating half of the verify↔show type-blind pair
/// (FR-4, ADR-5): it displays a receipt **without** rendering a verdict and
/// **without** minting `Admitted`. Minting admission here would be a fiat cast —
/// stamping the court's seal on bytes that never passed the court. `show`
/// therefore returns a plain `Receipt`; the only path to `Admitted` is the real
/// `crate::admission::admit` transition, which re-runs the structural law.
pub fn show(receipt: &str) -> Result<Receipt> {
    load_receipt(receipt)
}

// --------------------------------------------------------------------------
// Private helpers — only display/IO concerns that no sibling module owns.
// --------------------------------------------------------------------------

/// Read payload bytes from a file path, or from stdin when `source` is `-`.
fn read_payload(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .context("reading payload from stdin")?;
        Ok(buf)
    } else {
        std::fs::read(source).with_context(|| format!("opening payload file {source:?}"))
    }
}

/// Load and parse an immutable receipt file from `path`.
fn load_receipt(path: &str) -> Result<Receipt> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading receipt {path:?}"))?;
    let receipt: Receipt =
        serde_json::from_str(&text).with_context(|| format!("parsing receipt {path:?}"))?;
    Ok(receipt)
}
