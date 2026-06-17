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
use crate::types::{Blake3Hash, ObjectRef, Receipt};
use crate::verifier;
use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::PathBuf;

/// Append one operation-event to the working receipt (`.affi/working.json`).
///
/// `objects` are `id:type` pairs (optionally `id:type:qualifier`). The payload
/// is read from `payload` (a file path, or `-` for stdin) and its BLAKE3 digest
/// becomes the commitment — the raw payload is never stored in the receipt.
/// Returns the new event's details.
pub fn emit(
    event_type: &str,
    objects: &[String],
    payload: &str,
) -> Result<crate::types::EmitOutput> {
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
    let commitment = event.payload_commitment.as_hex().to_string();

    events.push(event);
    chain::save_working(&events).context("saving working receipt")?;

    Ok(crate::types::EmitOutput {
        event_id: id,
        seq,
        event_type: event_type.to_string(),
        commitment,
    })
}

/// Finalize the working receipt into an immutable, content-addressed receipt
/// file (name = BLAKE3 of canonical bytes), or to `out` if provided.
///
/// Builds the receipt through [`chain::ChainAssembler`] so the rolling chain
/// hash and format version match exactly what [`crate::verifier`] expects, then
/// content-addresses it. Returns the receipt path and content address.
pub fn assemble(out: Option<&str>) -> Result<crate::types::AssembleOutput> {
    let events = chain::load_working().context("loading working receipt")?;
    let event_count = events.len();
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

    chain::save_receipt(&receipt, &path).with_context(|| format!("writing receipt to {path:?}"))?;

    Ok(crate::types::AssembleOutput {
        receipt_path: path.display().to_string(),
        content_address: address.as_hex().to_string(),
        event_count,
    })
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

/// Render an object ref for human display: `id:type` or `id:type/qualifier`.
fn format_object(o: &ObjectRef) -> String {
    match &o.qualifier {
        Some(q) => format!("{}:{}/{}", o.id, o.obj_type, q),
        None => format!("{}:{}", o.id, o.obj_type),
    }
}

/// First 12 hex chars of a hash, for compact human display.
fn short_hash(h: &Blake3Hash) -> String {
    let hex = h.as_hex();
    let end = hex.len().min(12);
    hex[..end].to_string()
}

/// Load and parse an immutable receipt file from `path`.
fn load_receipt(path: &str) -> Result<Receipt> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading receipt {path:?}"))?;
    let receipt: Receipt =
        serde_json::from_str(&text).with_context(|| format!("parsing receipt {path:?}"))?;
    Ok(receipt)
}

/// Convert a Markdown string to plain ASCII for terminal display.
///
/// Transformations applied in order:
/// 1. ATX headings (#, ##, ###) -> UPPERCASE text + underline (=, -, ~)
/// 2. Bold (**text** or __text__) -> text (strips markers)
/// 3. Italic (*text* or _text_) -> text (strips markers)
/// 4. Inline code (`text`) -> text (strips backticks)
/// 5. Fenced code blocks (``` ... ```) -> 4-space indented block
/// 6. Links [text](url) -> "text (url)"
/// 7. Reflow all paragraphs to max_width (default 80)
pub(crate) fn format_help_markdown(md: &str) -> String {
    format_help_markdown_width(md, 80)
}

pub(crate) fn format_help_markdown_width(md: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_paragraph = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if !current_paragraph.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&reflow_paragraph(&current_paragraph, max_width));
                result.push('\n');
                current_paragraph.clear();
            }
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            result.push_str("    ");
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if trimmed.starts_with('#') {
            if !current_paragraph.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&reflow_paragraph(&current_paragraph, max_width));
                result.push('\n');
                current_paragraph.clear();
            }
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed[level..].trim();
            let transformed = apply_inline_transforms(text);
            let uppercase = transformed.to_uppercase();
            let underline_char = match level {
                1 => '=',
                2 => '-',
                _ => '~',
            };
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&uppercase);
            result.push('\n');
            result.push_str(&underline_char.to_string().repeat(uppercase.len()));
            result.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            if !current_paragraph.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&reflow_paragraph(&current_paragraph, max_width));
                result.push('\n');
                current_paragraph.clear();
            }
        } else {
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }
    }

    if !current_paragraph.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&reflow_paragraph(&current_paragraph, max_width));
        result.push('\n');
    }

    result.trim_end_matches('\n').to_string()
}

fn apply_inline_transforms(text: &str) -> String {
    let mut s = text.to_string();

    // 6. Links [text](url) -> "text (url)"
    while let Some(start_bracket) = s.find('[') {
        if let Some(end_bracket) = s[start_bracket..].find(']') {
            let end_bracket = start_bracket + end_bracket;
            if let Some(start_paren) = s[end_bracket..].find('(') {
                let start_paren = end_bracket + start_paren;
                if let Some(end_paren) = s[start_paren..].find(')') {
                    let end_paren = start_paren + end_paren;
                    let link_text = &s[start_bracket + 1..end_bracket];
                    let url = &s[start_paren + 1..end_paren];
                    let replacement = format!("{} ({})", link_text, url);
                    s.replace_range(start_bracket..end_paren + 1, &replacement);
                    continue;
                }
            }
        }
        break;
    }

    // 2. Bold
    s = s.replace("**", "");
    s = s.replace("__", "");

    // 3. Italic
    s = s.replace("*", "");

    // 4. Inline code
    s = s.replace("`", "");

    s
}

fn reflow_paragraph(text: &str, max_width: usize) -> String {
    let text = apply_inline_transforms(text);
    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push_str(&current_line);
            result.push('\n');
            current_line = word.to_string();
        }
    }
    result.push_str(&current_line);
    result
}
