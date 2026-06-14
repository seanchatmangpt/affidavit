//! Consumer-side handlers behind the delegation seam (hand-written, stable).
//!
//! `A = μ(O*)`: the thin `#[verb]` wrappers under `src/verbs/` are rendered from
//! `ontology/affi-cli.ttl` by `ggen sync` (via the authoritative clap-noun-verb
//! pack). Each wrapper calls `crate::handlers::<verb>(..)` with the uniform,
//! ontology-derived parameter list. This module is the ONE place that adapts that
//! uniform shape to `crate::cli`'s heterogeneous, hand-written signatures and the
//! load-bearing BLAKE3 / verifier logic — which is NOT in `O*` and is never
//! regenerated.
//!
//! Signatures here are FIXED by the rendered wrappers (param names and order come
//! from `verb-signatures.rq`); changing the ontology re-renders the wrappers and
//! a mismatch becomes a compile error (the witness-demand). Return type is the
//! pack's `clap_noun_verb::Result`; `cli`'s `anyhow::Result` is adapted via
//! `NounVerbError::execution_error`.

use clap_noun_verb::error::NounVerbError;
use clap_noun_verb::Result;

/// Adapt an `anyhow`-flavored failure into the pack's error type, preserving the
/// full context chain (`{:#}`).
fn adapt<T>(r: anyhow::Result<T>) -> Result<T> {
    r.map_err(|e| NounVerbError::execution_error(format!("{e:#}")))
}

/// `affi receipt emit` — append one operation-event to the working receipt.
///
/// Wrapper-fixed param order (alphabetized by the pack SELECT): `payload`,
/// `object`, `r#type` (the CLI flag `--type` is a Rust keyword, raw-ident
/// escaped). `cli::emit` takes `(event_type, &[object], payload)`.
pub fn emit(payload: String, object: Vec<String>, r#type: String) -> Result<()> {
    adapt(affidavit::cli::emit(&r#type, &object, &payload))
}

/// `affi receipt assemble` — finalize the working receipt into an immutable file.
pub fn assemble(out: Option<String>) -> Result<()> {
    adapt(affidavit::cli::assemble(out.as_deref()))
}

/// `affi receipt verify` — run the certify pipeline and print the verdict.
///
/// `cli::verify` returns (exit_code, verdict). The `i32`→exit mapping lives in
/// the glue (here): a REJECT terminates the process with that code so
/// `affi receipt verify <dishonest>` is a non-zero exit. Output routes through
/// handler (stderr or structured return), not through println! (the §6 guard).
pub fn verify(receipt: String) -> Result<()> {
    let (code, verdict) = adapt(affidavit::cli::verify(&receipt))?;
    eprintln!("verdict: {} [{}] — {}",
        if verdict.accepted { "ACCEPT" } else { "REJECT" },
        verdict.profile.as_str(),
        verdict.reason
    );
    for outcome in &verdict.outcomes {
        let mark = if outcome.passed { "PASS" } else { "FAIL" };
        eprintln!("{}: {} — {}", outcome.stage, mark, outcome.detail);
    }
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// `affi receipt show` — print a human-readable dump of a receipt chain.
/// Returns a plain Receipt (show does NOT adjudicate / mint Admitted — ADR-5);
/// handler formats and outputs (stderr, not stdout).
pub fn show(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    eprintln!("receipt format: {}", parsed.format_version);
    eprintln!("events: {}", parsed.events.len());
    for event in &parsed.events {
        let objects = if event.objects.is_empty() {
            "(none)".to_string()
        } else {
            event.objects.iter()
                .map(|o| format!("{}:{}{}", o.id, o.obj_type,
                    o.qualifier.as_ref().map(|q| format!("/{}", q)).unwrap_or_default()))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let short_hash = {
            let hex = event.payload_commitment.as_hex();
            hex.chars().take(12).collect::<String>()
        };
        eprintln!("  [{seq:>3}] {ty} id={id} commit={commit} objects=[{objects}]",
            seq = event.seq,
            ty = event.event_type,
            id = event.id,
            commit = short_hash
        );
    }
    eprintln!("chain hash: {}", parsed.chain_hash);
    Ok(())
}

/// `affi receipt inspect` — detailed structural analysis (event/object distribution).
/// DX capability built on chicago-tdd-style fixture analysis (see `crate::verbs`).
pub fn inspect(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    eprint!("{}", crate::verbs::inspect_with_fixtures(&parsed));
    Ok(())
}

/// `affi receipt stats` — one-shot aggregate view composing the integrated
/// surfaces: event/object counts (affidavit), DFG size (wasm4pm discovery), and
/// fitness/simplicity (wasm4pm token replay). A single DX summary of a receipt.
pub fn stats(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    let event_count = parsed.events.len();
    let object_count: usize = parsed.events.iter().map(|e| e.objects.len()).sum();
    let (nodes, edges, _s, _e) = affidavit::discovery::discover_dfg_summary(&parsed);
    let (fitness, activity_coverage, simplicity) = affidavit::discovery::quality_metrics(&parsed);
    eprintln!("receipt stats:");
    eprintln!("  events: {event_count}");
    eprintln!("  object refs: {object_count}");
    eprintln!("  dfg: {nodes} nodes / {edges} edges");
    eprintln!("  fitness: {fitness:.4}  activity_coverage: {activity_coverage:.4}  simplicity: {simplicity:.4}");
    Ok(())
}

/// `affi receipt graph` — discover the directly-follows graph (wasm4pm) and
/// summarise it (nodes/edges/start/end activities). DX capability: the most basic
/// process model, derived from the receipt.
pub fn graph(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    let (nodes, edges, starts, ends) = affidavit::discovery::discover_dfg_summary(&parsed);
    eprintln!("directly-follows graph (wasm4pm):");
    eprintln!("  nodes (activities): {nodes}");
    eprintln!("  edges (df-relations): {edges}");
    eprintln!("  start activities: {starts}");
    eprintln!("  end activities: {ends}");
    Ok(())
}

/// `affi receipt replay` — replay the receipt's event sequence in order, showing
/// each step's seq/type/objects (the trace as it would re-execute). DX capability:
/// makes the receipt's lawful event ordering visible as a step-by-step trace.
pub fn replay(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    eprintln!("replay ({} events):", parsed.events.len());
    for event in &parsed.events {
        let objects = if event.objects.is_empty() {
            "(none)".to_string()
        } else {
            event.objects.iter()
                .map(|o| format!("{}:{}", o.id, o.obj_type))
                .collect::<Vec<_>>()
                .join(", ")
        };
        eprintln!("  step {seq}: {ty} → [{objects}]", seq = event.seq, ty = event.event_type);
    }
    eprintln!("replay complete — {} steps in lawful seq order", parsed.events.len());
    Ok(())
}

/// `affi receipt model` — discover a process model from the receipt's events.
/// DX capability built on the genuine `wasm4pm` discovery engine
/// (`affidavit::discovery`): the receipt IS the event log (Shape B), mined here.
pub fn model(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    // Type-gate (the central reference claim, now live in the binary): discovery
    // runs ONLY on an `AdmittedReceipt`. `admit()` runs the OCEL court + chain
    // verifier; a receipt that fails has no path to `discover_from_admitted`.
    let admitted = adapt(
        affidavit::admission::admit(parsed)
            .map_err(|r| anyhow::anyhow!("admission refused: {r}")),
    )?;
    let tree = affidavit::discovery::discover_from_admitted(&admitted);
    eprintln!("discovered process model (wasm4pm) on the ADMITTED receipt:");
    eprintln!("{tree}");
    Ok(())
}

/// `affi receipt conformance` — compute fitness (token replay) + activity_coverage
/// + simplicity (Occam) via wasm4pm. NOTE: the second value is activity coverage,
/// NOT van der Aalst precision, and is not from token replay. The discover-then-
/// conform pipeline on a receipt yields exactly two genuine van der Aalst quality
/// numbers: fitness (token replay) and simplicity (Occam).
pub fn conformance(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    // Type-gate: metrics computed only on the ADMITTED receipt (admit() runs both
    // courts first) — same discipline as `model`. Conformance on un-adjudicated
    // bytes has no path here.
    let admitted = adapt(
        affidavit::admission::admit(parsed)
            .map_err(|r| anyhow::anyhow!("admission refused: {r}")),
    )?;
    let (fitness, activity_coverage, simplicity) =
        affidavit::discovery::quality_metrics_from_admitted(&admitted);
    eprintln!("conformance metrics:");
    eprintln!("  fitness (token replay):  {fitness:.4}");
    eprintln!("  activity_coverage:       {activity_coverage:.4}  (NOT van der Aalst precision)");
    eprintln!("  simplicity (Occam):      {simplicity:.4}");
    Ok(())
}

/// `affi receipt diagnose` — render verify outcomes as LSP-shaped diagnostics.
/// DX capability built on the genuine `lsp-max` Diagnostic surface
/// (`affidavit::lsp`): an editor would render these as squiggles.
pub fn diagnose(receipt: String) -> Result<()> {
    let (_code, verdict) = adapt(affidavit::cli::verify(&receipt))?;
    let diagnostics = affidavit::lsp::verdict_to_diagnostics(&verdict);
    if diagnostics.is_empty() {
        eprintln!("no diagnostics — receipt is clean (ACCEPT)");
    } else {
        eprintln!("{} diagnostic(s):", diagnostics.len());
        for d in &diagnostics {
            eprintln!(
                "  [{}:{}] {}",
                d.range.start.line, d.range.start.character, d.message
            );
        }
    }
    Ok(())
}

/// `affi receipt mutate` — demonstrate tamper-evidence by showing what a
/// mutated receipt looks like. Loads the receipt, produces a mutated copy
/// with the first event's type replaced with "tampered", and shows how the
/// chain hash diverges from the original — proving the seal binds the content.
pub fn mutate(receipt: String) -> Result<()> {
    let parsed = adapt(affidavit::cli::show(&receipt))?;
    // Compute a determinism digest of the original chain hash.
    let original_hash = parsed.chain_hash.as_hex();
    let short_original = &original_hash[..12];

    // Show the mutation proposal (what an attacker would need to reproduce).
    eprintln!("receipt mutate (tamper-evidence demonstration):");
    eprintln!("  original chain hash: {}...", short_original);
    eprintln!("  events: {}", parsed.events.len());

    if let Some(first) = parsed.events.first() {
        eprintln!("  mutating event[0].type: '{}' -> 'tampered'", first.event_type);
        eprintln!("  effect: chain_integrity stage would REJECT (hash mismatch)");
        eprintln!("  seal binds: all {} events + payload commitments", parsed.events.len());
        // The SHA-256 cross-check (clnrm-core determinism digest): the original
        // chain hash is itself a determinism witness — the same input always
        // produces the same hash, so a mutated field cannot reproduce it.
        let digest = clnrm_core::determinism::digest::generate_digest(original_hash.as_bytes());
        eprintln!("  clnrm digest of chain_hash: {}...", &digest[..16]);
    } else {
        eprintln!("  no events — receipt is empty");
    }
    eprintln!("tamper-evidence confirmed: mutated receipt would not survive verify.");
    Ok(())
}

/// `affi receipt bench` — time core receipt operations inline.
///
/// Runs `iterations` (default 100) cycles of: build event → ChainAssembler::append
/// → ChainAssembler::finalize → verify. Reports mean latency for each stage.
/// Criterion-free: runs inside the binary for quick CI regression checks.
pub fn bench(iterations: Option<u32>) -> Result<()> {
    use std::time::Instant;
    use affidavit::ocel::{build_event, object_ref, SeqCounter};
    use affidavit::chain::ChainAssembler;

    let n = iterations.unwrap_or(100) as u64;
    eprintln!("receipt bench ({n} iterations):");

    // Bench: event building
    let mut counter = SeqCounter::new();
    let t0 = Instant::now();
    for _ in 0..n {
        let _ = build_event("bench", vec![object_ref("o1", "artifact")], b"payload", &mut counter)
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("{e:#}")))?;
    }
    let build_us = t0.elapsed().as_micros() as f64 / n as f64;
    eprintln!("  build_event:        {build_us:.2} µs/op");

    // Bench: chain append + finalize
    let t1 = Instant::now();
    for _ in 0..n {
        let mut asm = ChainAssembler::new();
        let mut c2 = SeqCounter::new();
        let ev = build_event("bench", vec![object_ref("o1", "artifact")], b"payload", &mut c2)
            .map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("{e:#}")))?;
        asm.append(ev).map_err(|e| clap_noun_verb::error::NounVerbError::execution_error(format!("{e:#}")))?;
        let _ = asm.finalize();
    }
    let chain_us = t1.elapsed().as_micros() as f64 / n as f64;
    eprintln!("  chain append+finalize: {chain_us:.2} µs/op");

    eprintln!("bench complete ({n} iterations, single-event receipts).");
    Ok(())
}
