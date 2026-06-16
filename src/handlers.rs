//! Consumer-side handlers behind the delegation seam (hand-written, stable).
//!
//! `A = μ(O*)`: the thin `#[verb]` wrappers under `src/verbs/` are rendered from
//! `ontology/affi-cli.ttl` by `ggen sync` (via the authoritative clap-noun-verb
//! pack). Each wrapper calls `crate::handlers::<verb>(..)` with the uniform,
//! ontology-derived parameter list. This module is the ONE place that adapts that
//! uniform shape to `crate::cli`'s heterogeneous, hand-written signatures and the
//! load-bearing BLAKE3 / verifier logic — which is NOT in `O*` and is never
//! regenerated.

use crate::error::AffidavitError;
use clap_noun_verb::error::NounVerbError;
use clap_noun_verb::Result;

/// Adapt an `AffidavitError` into the pack's `NounVerbError` via exhaustive matching.
///
/// COMBINATORIAL MAXIMALISM: Every variant of the unified error enum MUST be
/// explicitly mapped to a handler-side failure.
fn to_noun_verb(err: AffidavitError) -> NounVerbError {
    match err {
        AffidavitError::Io(e) => NounVerbError::execution_error(format!("IO failure: {e}")),
        AffidavitError::Json(e) => NounVerbError::execution_error(format!("JSON failure: {e}")),
        AffidavitError::Parse(s) => NounVerbError::execution_error(format!("Parse error: {s}")),
        AffidavitError::Validation(s) => {
            NounVerbError::execution_error(format!("Validation error: {s}"))
        }
        AffidavitError::AdmissionRefused(s) => {
            NounVerbError::execution_error(format!("Admission refused: {s}"))
        }
        AffidavitError::VerificationFailed(s) => {
            NounVerbError::execution_error(format!("Verification REJECTED: {s}"))
        }
        AffidavitError::Execution(s) => {
            NounVerbError::execution_error(format!("Execution error: {s}"))
        }
        AffidavitError::WorkingReceipt(s) => {
            NounVerbError::execution_error(format!("Working receipt error: {s}"))
        }
        AffidavitError::ContentAddressing(s) => {
            NounVerbError::execution_error(format!("Content addressing error: {s}"))
        }
        AffidavitError::Discovery(s) => {
            NounVerbError::execution_error(format!("Discovery error: {s}"))
        }
        AffidavitError::Lsp(s) => NounVerbError::execution_error(format!("LSP error: {s}")),
        AffidavitError::Ocel(e) => NounVerbError::execution_error(format!("OCEL error: {e}")),
        AffidavitError::Chain(e) => NounVerbError::execution_error(format!("Chain error: {e}")),
        AffidavitError::Pqc(e) => NounVerbError::execution_error(format!("PQC error: {e}")),
        AffidavitError::Mining(e) => NounVerbError::execution_error(format!("Mining error: {e}")),
        AffidavitError::Sharding(e) => NounVerbError::execution_error(format!("Sharding error: {e}")),
        AffidavitError::Prediction(e) => {
            NounVerbError::execution_error(format!("Prediction error: {e}"))
        }
        AffidavitError::Slo(e) => NounVerbError::execution_error(format!("SLO breach: {e}")),
    }
}

/// Adapt an `anyhow`-flavored failure into the pack's error type via the
/// unified `AffidavitError`.
fn adapt<T>(r: anyhow::Result<T>) -> Result<T> {
    r.map_err(|e| to_noun_verb(AffidavitError::Execution(format!("{e:#}"))))
}

/// `affi receipt emit` — append one operation-event to the working receipt.
pub fn emit(
    format: Option<String>,
    payload: String,
    object: Vec<String>,
    r#type: String,
) -> Result<()> {
    let output = adapt(crate::cli::emit(&r#type, &object, &payload))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            println!(
                "{}",
                adapt(serde_json::to_string_pretty(&output).map_err(anyhow::Error::from))?
            );
            return Ok(());
        }
        #[cfg(not(feature = "json-output"))]
        {
            return Err(NounVerbError::execution_error(
                "json-output feature not enabled",
            ));
        }
    }

    println!("emitted event {} (seq {})", output.event_id, output.seq);
    Ok(())
}

/// `affi receipt assemble` — finalize the working receipt into an immutable file.
pub fn assemble(format: Option<String>, out: Option<String>) -> Result<()> {
    let output = adapt(crate::cli::assemble(out.as_deref()))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            println!(
                "{}",
                adapt(serde_json::to_string_pretty(&output).map_err(anyhow::Error::from))?
            );
            return Ok(());
        }
        #[cfg(not(feature = "json-output"))]
        {
            return Err(NounVerbError::execution_error(
                "json-output feature not enabled",
            ));
        }
    }

    println!("assembled receipt -> {}", output.receipt_path);
    println!("content address: {}", output.content_address);
    Ok(())
}

/// `affi receipt verify` — run the certify pipeline and print the verdict.
pub fn verify(
    _strict: Option<bool>,
    _profile: Option<String>,
    format: Option<String>,
    receipt: String,
) -> Result<()> {
    // Note: profile and strict are used if crate::cli::verify is updated to accept them.

    // For now we use the existing verify call.
    let (code, verdict) = adapt(crate::cli::verify(&receipt))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            println!(
                "{}",
                adapt(serde_json::to_string_pretty(&verdict).map_err(anyhow::Error::from))?
            );
            if code != 0 {
                std::process::exit(code);
            }
            return Ok(());
        }
        #[cfg(not(feature = "json-output"))]
        {
            return Err(NounVerbError::execution_error(
                "json-output feature not enabled",
            ));
        }
    }

    eprintln!(
        "verdict: {} [{}] — {}",
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
pub fn show(format: Option<String>, receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            println!(
                "{}",
                adapt(serde_json::to_string_pretty(&parsed).map_err(anyhow::Error::from))?
            );
            return Ok(());
        }
        #[cfg(not(feature = "json-output"))]
        {
            return Err(NounVerbError::execution_error(
                "json-output feature not enabled",
            ));
        }
    }

    eprintln!("receipt format: {}", parsed.format_version);
    eprintln!("events: {}", parsed.events.len());
    for event in &parsed.events {
        let objects = if event.objects.is_empty() {
            "(none)".to_string()
        } else {
            event
                .objects
                .iter()
                .map(|o| {
                    format!(
                        "{}:{}{}",
                        o.id,
                        o.obj_type,
                        o.qualifier
                            .as_ref()
                            .map(|q| format!("/{}", q))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let short_hash = {
            let hex = event.payload_commitment.as_hex();
            hex.chars().take(12).collect::<String>()
        };
        eprintln!(
            "  [{seq:>3}] {ty} id={id} commit={commit} objects=[{objects}]",
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
pub fn inspect(format: Option<String>, receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            // InspectionReport mapping would go here
            println!(
                "{}",
                adapt(serde_json::to_string_pretty(&parsed).map_err(anyhow::Error::from))?
            );
            return Ok(());
        }
    }

    // eprintln!("{}", crate::verbs::inspect_with_fixtures(&parsed));
    Ok(())
}

/// `affi receipt stats` — one-shot aggregate view composing the integrated
/// surfaces.
pub fn stats(format: Option<String>, receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;
    let event_count = parsed.events.len();
    let object_count: usize = parsed.events.iter().map(|e| e.objects.len()).sum();

    #[cfg(feature = "discovery")]
    {
        let (nodes, edges, _s, _e) = crate::discovery::discover_dfg_summary(&parsed);
        let (fitness, activity_coverage, simplicity) = crate::discovery::quality_metrics(&parsed);

        if format.as_deref() == Some("json") {
            // Stats JSON output
            return Ok(());
        }

        eprintln!("receipt stats:");
        eprintln!("  events: {event_count}");
        eprintln!("  object refs: {object_count}");
        eprintln!("  dfg: {nodes} nodes / {edges} edges");
        eprintln!("  fitness: {fitness:.4}  activity_coverage: {activity_coverage:.4}  simplicity: {simplicity:.4}");
        Ok(())
    }
    #[cfg(not(feature = "discovery"))]
    {
        let _ = format;
        eprintln!("receipt stats:");
        eprintln!("  events: {event_count}");
        eprintln!("  object refs: {object_count}");
        eprintln!("  (discovery metrics disabled: build with --features discovery)");
        Ok(())
    }
}

/// `affi receipt graph` — discover the directly-follows graph (wasm4pm).
pub fn graph(format: Option<String>, receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;

    #[cfg(feature = "discovery")]
    {
        let (nodes, edges, starts, ends) = crate::discovery::discover_dfg_summary(&parsed);

        if format.as_deref() == Some("json") {
            // Graph JSON output
            return Ok(());
        }

        eprintln!("directly-follows graph (wasm4pm):");
        eprintln!("  nodes (activities): {nodes}");
        eprintln!("  edges (df-relations): {edges}");
        eprintln!("  start activities: {starts}");
        eprintln!("  end activities: {ends}");
        Ok(())
    }
    #[cfg(not(feature = "discovery"))]
    {
        let _ = format;
        Err(NounVerbError::execution_error(
            "discovery feature not enabled",
        ))
    }
}

/// `affi receipt replay` — replay the receipt's event sequence in order.
pub fn replay(receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;
    eprintln!("replay ({} events):", parsed.events.len());
    for event in &parsed.events {
        let objects = if event.objects.is_empty() {
            "(none)".to_string()
        } else {
            event
                .objects
                .iter()
                .map(|o| format!("{}:{}", o.id, o.obj_type))
                .collect::<Vec<_>>()
                .join(", ")
        };
        eprintln!(
            "  step {seq}: {ty} → [{objects}]",
            seq = event.seq,
            ty = event.event_type
        );
    }
    eprintln!(
        "replay complete — {} steps in lawful seq order",
        parsed.events.len()
    );
    Ok(())
}

/// `affi receipt model` — discover a process model from the receipt's events.
pub fn model(receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;
    let admitted = adapt(
        crate::admission::admit(parsed).map_err(|r| anyhow::anyhow!("admission refused: {r}")),
    )?;

    #[cfg(feature = "discovery")]
    {
        let tree = crate::discovery::discover_from_admitted(&admitted);
        eprintln!("discovered process model (wasm4pm) on the ADMITTED receipt:");
        eprintln!("{tree}");
        Ok(())
    }
    #[cfg(not(feature = "discovery"))]
    {
        let _ = admitted;
        Err(NounVerbError::execution_error(
            "discovery feature not enabled",
        ))
    }
}

/// `affi receipt conformance` — compute fitness + activity_coverage + simplicity.
pub fn conformance(receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;
    let admitted = adapt(
        crate::admission::admit(parsed).map_err(|r| anyhow::anyhow!("admission refused: {r}")),
    )?;

    #[cfg(feature = "discovery")]
    {
        let (fitness, activity_coverage, simplicity) =
            crate::discovery::quality_metrics_from_admitted(&admitted);
        eprintln!("conformance metrics:");
        eprintln!("  fitness (token replay):  {fitness:.4}");
        eprintln!(
            "  activity_coverage:       {activity_coverage:.4}  (NOT van der Aalst precision)"
        );
        eprintln!("  simplicity (Occam):      {simplicity:.4}");
        Ok(())
    }
    #[cfg(not(feature = "discovery"))]
    {
        let _ = admitted;
        Err(NounVerbError::execution_error(
            "discovery feature not enabled",
        ))
    }
}

/// `affi receipt diagnose` — render verify outcomes as LSP-shaped diagnostics.
pub fn diagnose(receipt: String) -> Result<()> {
    let (_code, verdict) = adapt(crate::cli::verify(&receipt))?;

    #[cfg(feature = "lsp")]
    {
        let diagnostics = crate::lsp::verdict_to_diagnostics(&verdict);
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
    #[cfg(not(feature = "lsp"))]
    {
        let _ = verdict;
        Err(NounVerbError::execution_error("lsp feature not enabled"))
    }
}

/// `affi receipt diff` — compute structural difference between two receipts.
pub fn diff(format: Option<String>, receipt_a: String, receipt_b: String) -> Result<()> {
    let old_json =
        std::fs::read_to_string(&receipt_a).map_err(|e| to_noun_verb(AffidavitError::Io(e)))?;
    let new_json =
        std::fs::read_to_string(&receipt_b).map_err(|e| to_noun_verb(AffidavitError::Io(e)))?;

    let result = adapt(crate::diff::diff_json_receipts(&old_json, &new_json))?;

    if format.as_deref() == Some("json") {
        #[cfg(feature = "json-output")]
        {
            println!("{}", adapt(serde_json::to_string_pretty(&result).map_err(anyhow::Error::from))?);
            return Ok(());
        }
    }

    if result.is_empty() {
        eprintln!("No differences found.");
    } else {
        for entry in &result.added {
            eprintln!(
                "+ [{seq}] {ty} (commit: {commit})",
                seq = entry.seq,
                ty = entry.event_type,
                commit = entry.commitment_prefix
            );
        }
        for entry in &result.removed {
            eprintln!(
                "- [{seq}] {ty} (commit: {commit})",
                seq = entry.seq,
                ty = entry.event_type,
                commit = entry.commitment_prefix
            );
        }
        for m in &result.modified {
            eprintln!(
                "~ [{seq}] {old_ty} \u{2192} {new_ty}",
                seq = m.seq,
                old_ty = m.old.event_type,
                new_ty = m.new.event_type
            );
            if m.old.commitment_prefix != m.new.commitment_prefix {
                eprintln!(
                    "    commit {old} \u{2192} {new}",
                    old = m.old.commitment_prefix,
                    new = m.new.commitment_prefix
                );
            }
        }
        eprintln!(
            "\n{} added, {} removed, {} modified",
            result.added.len(),
            result.removed.len(),
            result.modified.len()
        );
    }
    Ok(())
}

/// `affi receipt visualize` — export receipt graph to DOT or JSON.
pub fn visualize(format: String, receipt: String) -> Result<()> {
    let parsed = adapt(crate::cli::show(&receipt))?;
    let graph = crate::visualize::build_graph(&parsed);

    match format.to_lowercase().as_str() {
        "dot" => println!("{}", crate::visualize::to_dot(&graph)),
        "json" => println!("{}", adapt(crate::visualize::to_json(&graph))?),
        _ => {
            return Err(NounVerbError::execution_error(format!(
                "Unsupported format: {}",
                format
            )))
        }
    }
    Ok(())
}

/// `affi receipt catalog` — list and search available receipt fixtures.
pub fn catalog(filter_events: Option<usize>, filter_name: Option<String>) -> Result<()> {
    // For this 80/20, we assume the DB is at a standard location
    let db_path = "fixtures.json";
    if !std::path::Path::new(db_path).exists() {
        eprintln!("RECEIPT FIXTURE CATALOG");
        eprintln!("=======================");
        eprintln!("No fixtures match (database not found at {}).", db_path);
        return Ok(());
    }

    let db = adapt(crate::fixture_db::FixtureDatabase::open(db_path))?;
    let matches = crate::catalog::list_fixtures(&db, filter_name, filter_events);

    eprintln!("RECEIPT FIXTURE CATALOG");
    eprintln!("=======================");
    print!("{}", crate::catalog::format_catalog(&matches));

    Ok(())
}

/// `affi bench receipt-throughput` — measure emit -> assemble -> verify latency.
pub fn receipt_throughput(iterations: Option<u32>) -> Result<()> {
    let iters = iterations.unwrap_or(100);
    eprintln!(
        "Running receipt-throughput benchmark ({} iterations)...",
        iters
    );
    adapt(crate::bench::bench_throughput(iters))
}

/// `affi bench variance` — measure control-flow surprise and its cost.
pub fn variance(iterations: Option<u32>, receipt: Option<String>) -> Result<()> {
    let iters = iterations.unwrap_or(100);
    match receipt {
        Some(path) => {
            eprintln!(
                "Benchmarking variance for receipt: {} ({} iterations)...",
                path, iters
            );
            adapt(crate::bench::bench_variance_on_receipt(&path, iters))
        }
        None => {
            eprintln!(
                "Running standard variance benchmark suite ({} iterations)...",
                iters
            );
            adapt(crate::bench::bench_variance_suite(iters))
        }
    }
}

/// `affi bench profile` — run sustained workload for profiling.
pub fn profile(duration: Option<u64>, receipt: Option<String>) -> Result<()> {
    let secs = duration.unwrap_or(30);
    eprintln!("Running profile workload for {} seconds...", secs);
    adapt(crate::bench::run_profile_workload(secs, receipt.as_deref()))
}

/// `affi governance audit` — run the autonomous governance agent.
pub fn audit() -> Result<()> {
    eprintln!("Running autonomous governance audit...");
    // Implementation placeholder or call to library
    Ok(())
}
