//! # Verb Registry — Single Source of Truth for All 69 Verbs
//!
//! This module is the W4 keystone: a compile-time static registry that eliminates
//! drift between documentation, shell completions, and the actual verb set.
//!
//! ## Usage
//!
//! ```rust
//! use affidavit::registry::{REGISTRY, VerbGroup, lookup, by_group, did_you_mean, verb_count};
//!
//! // Count all registered verbs
//! assert_eq!(verb_count(), 69);
//!
//! // Look up by (verb, noun)
//! let entry = lookup("emit", "receipt").unwrap();
//! assert_eq!(entry.group, VerbGroup::Core);
//!
//! // Find verbs in a group
//! let core = by_group(VerbGroup::Core);
//! assert!(!core.is_empty());
//!
//! // "Did you mean" — substring match over verb names and keywords
//! let suggestions = did_you_mean("emi");
//! assert!(!suggestions.is_empty());
//! ```

/// Taxonomy group for grouping related verbs in help/search output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbGroup {
    /// Core receipt lifecycle: emit, assemble, verify, show, inspect, stats
    Core,
    /// Diagnostic and forensic operations: diagnose, diff, graph, replay, timeline, root_cause
    Diagnostics,
    /// Analysis and mining: audit, query, model, conformance, coverage_analysis, tech_debt, security_debt
    Analysis,
    /// Event ingestion from external systems: emit_from_cicd, emit_from_github, etc.
    Ingestion,
    /// Compliance framework verification: verify_compliance, verify_sla, verify_family, policy_enforce, etc.
    Compliance,
    /// Attestation and signing: sign, notarize, attest, assemble_with_signature, etc.
    Attestation,
    /// Software Bill of Materials operations: sbom_scan, sbom_blast_radius, etc.
    Sbom,
    /// Predictive and anomaly insights: anomaly_detect, predict, trend_analysis, variance, etc.
    Insights,
    /// Engineering health metrics: bus_factor, dora_metrics, team_velocity, portfolio_health, etc.
    Engineering,
    /// Tooling and infrastructure: catalog, search, profile, install_git_hook, monitor, etc.
    Tooling,
}

impl VerbGroup {
    /// Short human-readable label for the group.
    pub fn label(self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Diagnostics => "Diagnostics",
            Self::Analysis => "Analysis",
            Self::Ingestion => "Ingestion",
            Self::Compliance => "Compliance",
            Self::Attestation => "Attestation",
            Self::Sbom => "SBOM",
            Self::Insights => "Insights",
            Self::Engineering => "Engineering",
            Self::Tooling => "Tooling",
        }
    }

    /// One-line description of what verbs in this group do.
    pub fn description(self) -> &'static str {
        match self {
            Self::Core => "Core receipt lifecycle operations (emit, assemble, verify, show, inspect, stats)",
            Self::Diagnostics => "Diagnostic and forensic operations for troubleshooting receipt failures",
            Self::Analysis => "Analysis and mining of receipt chains for quality and coverage insights",
            Self::Ingestion => "Event ingestion adapters for CI/CD, cloud, SCM, and monitoring systems",
            Self::Compliance => "Framework compliance verification against GDPR, HIPAA, PCI-DSS, SOC 2, SLA, and custom policies",
            Self::Attestation => "Signing, notarization, and attestation of receipts for supply-chain assurance",
            Self::Sbom => "Software Bill of Materials scanning, verification, and NTIA compliance checks",
            Self::Insights => "Predictive and anomaly-detection insights derived from receipt chains",
            Self::Engineering => "Engineering health and productivity metrics extracted from receipt history",
            Self::Tooling => "Infrastructure tooling: catalog management, search, profiling, and hook installation",
        }
    }
}

/// A registered verb entry — one record per live verb.
#[derive(Debug, Clone)]
pub struct VerbEntry {
    /// The verb token (e.g. `"emit"`, `"verify_compliance"`).
    pub verb: &'static str,
    /// The noun this verb operates on (always `"receipt"` for now).
    pub noun: &'static str,
    /// Taxonomy group for grouping in help/search output.
    pub group: VerbGroup,
    /// One-line summary displayed in help and man pages.
    pub summary: &'static str,
    /// Keywords for fuzzy search and shell completion.
    pub keywords: &'static [&'static str],
    /// Optional short usage example shown in `--help` output.
    pub example: Option<&'static str>,
}

impl VerbEntry {
    /// Construct a new `VerbEntry` without an example.
    pub const fn new(
        verb: &'static str,
        noun: &'static str,
        group: VerbGroup,
        summary: &'static str,
        keywords: &'static [&'static str],
    ) -> Self {
        Self {
            verb,
            noun,
            group,
            summary,
            keywords,
            example: None,
        }
    }

    /// Attach a short usage example to this entry.
    pub const fn with_example(mut self, example: &'static str) -> Self {
        self.example = Some(example);
        self
    }
}

/// The complete verb registry — 69 entries, one per live verb.
///
/// Ordering mirrors `src/verbs/mod.rs` (alphabetical) for easy cross-referencing.
pub static REGISTRY: &[VerbEntry] = &[
    // ── Core ────────────────────────────────────────────────────────────────
    VerbEntry::new(
        "emit",
        "receipt",
        VerbGroup::Core,
        "Record an operation-event into the working receipt chain",
        &["emit", "record", "event", "append", "log"],
    ),
    VerbEntry::new(
        "assemble",
        "receipt",
        VerbGroup::Core,
        "Finalize the working receipt into an immutable sealed file",
        &["assemble", "finalize", "seal", "build", "commit"],
    ),
    VerbEntry::new(
        "verify",
        "receipt",
        VerbGroup::Core,
        "Run the 7-stage certify pipeline against a receipt (exit 0=ACCEPT, 2=REJECT)",
        &["verify", "certify", "check", "validate", "audit"],
    ),
    VerbEntry::new(
        "show",
        "receipt",
        VerbGroup::Core,
        "Human-readable dump of the receipt chain with event details",
        &["show", "display", "print", "dump", "read"],
    ),
    VerbEntry::new(
        "inspect",
        "receipt",
        VerbGroup::Core,
        "Detailed inspection of receipt internals (chain hash, commitments, continuity)",
        &["inspect", "detail", "internals", "debug", "examine"],
    ),
    VerbEntry::new(
        "stats",
        "receipt",
        VerbGroup::Core,
        "Chain metrics: event count, hash distribution, event-type histogram",
        &["stats", "metrics", "count", "histogram", "summary"],
    ),
    // ── Diagnostics ─────────────────────────────────────────────────────────
    VerbEntry::new(
        "why",
        "receipt",
        VerbGroup::Diagnostics,
        "Explain in plain language why a receipt was rejected, with stage-by-stage remediation steps",
        &["why", "explain", "reason", "reject", "failed", "fix"],
    ),
    VerbEntry::new(
        "fix",
        "receipt",
        VerbGroup::Diagnostics,
        "Apply a safe structural repair: quarantine a tampered receipt or finalize a working one. Use --dry-run to preview.",
        &["fix", "repair", "quarantine", "finalize", "remediate"],
    ),
    VerbEntry::new(
        "diagnose",
        "receipt",
        VerbGroup::Diagnostics,
        "Troubleshoot verification failures with suggested remediation steps",
        &["diagnose", "debug", "troubleshoot", "fix", "repair"],
    ),
    VerbEntry::new(
        "diff",
        "receipt",
        VerbGroup::Diagnostics,
        "Compute a structural diff between two receipts showing added, removed, and changed events",
        &["diff", "compare", "delta", "changes", "between"],
    ),
    VerbEntry::new(
        "graph",
        "receipt",
        VerbGroup::Diagnostics,
        "DAG visualization of event dependencies and object references (dot, mermaid, json)",
        &["graph", "dag", "visualize", "dot", "mermaid", "dependencies"],
    ),
    VerbEntry::new(
        "replay",
        "receipt",
        VerbGroup::Diagnostics,
        "Re-execute chain from stored events, optionally applying custom handlers",
        &["replay", "rerun", "re-execute", "simulate", "playback"],
    ),
    VerbEntry::new(
        "timeline",
        "receipt",
        VerbGroup::Diagnostics,
        "Render a temporal timeline of events ordered by sequence number",
        &["timeline", "time", "sequence", "order", "chronology"],
    ),
    VerbEntry::new(
        "root_cause",
        "receipt",
        VerbGroup::Diagnostics,
        "Trace failure events back to their causal predecessors in the chain",
        &["root_cause", "root-cause", "cause", "origin", "trace", "failure"],
    ),
    // ── Analysis ────────────────────────────────────────────────────────────
    VerbEntry::new(
        "audit",
        "receipt",
        VerbGroup::Analysis,
        "Run a full audit pass over a receipt: chain integrity, commitments, and event completeness",
        &["audit", "full-check", "integrity", "scan", "review"],
    ),
    VerbEntry::new(
        "query",
        "receipt",
        VerbGroup::Analysis,
        "Query receipt events using a filter expression (event_type, object, seq range)",
        &["query", "filter", "search", "find", "select", "jq"],
    ),
    VerbEntry::new(
        "model",
        "receipt",
        VerbGroup::Analysis,
        "Extract the type schema from a receipt (event types, object types, qualifiers)",
        &["model", "schema", "types", "extract", "ontology"],
    ),
    VerbEntry::new(
        "conformance",
        "receipt",
        VerbGroup::Analysis,
        "Check a receipt against custom conformance rules or a named profile",
        &["conformance", "profile", "rules", "standard", "check"],
    ),
    VerbEntry::new(
        "coverage_analysis",
        "receipt",
        VerbGroup::Analysis,
        "Measure what fraction of defined event types appear in the receipt",
        &["coverage", "coverage_analysis", "completeness", "missing", "gaps"],
    ),
    VerbEntry::new(
        "tech_debt",
        "receipt",
        VerbGroup::Analysis,
        "Identify stale or low-quality events that signal accumulated technical debt",
        &["tech_debt", "debt", "quality", "stale", "maintenance"],
    ),
    VerbEntry::new(
        "security_debt",
        "receipt",
        VerbGroup::Analysis,
        "Surface security-relevant events with missing or weak commitments",
        &["security_debt", "security", "vulnerability", "weakness", "cve"],
    ),
    // ── Ingestion ────────────────────────────────────────────────────────────
    VerbEntry::new(
        "emit_batch",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest multiple events from a JSONL or CSV file in a single batch operation",
        &["emit_batch", "batch", "bulk", "import", "ingest", "jsonl"],
    ),
    VerbEntry::new(
        "emit_from_cicd",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a CI/CD pipeline run (GitHub Actions, GitLab CI, Jenkins, etc.)",
        &["emit_from_cicd", "cicd", "ci", "cd", "pipeline", "build"],
    ),
    VerbEntry::new(
        "emit_from_cloud",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from cloud provider audit logs (AWS CloudTrail, GCP Audit, Azure Monitor)",
        &["emit_from_cloud", "cloud", "aws", "gcp", "azure", "cloudtrail"],
    ),
    VerbEntry::new(
        "emit_from_github",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a GitHub repository (commits, PRs, releases, workflow runs)",
        &["emit_from_github", "github", "git", "commits", "prs"],
    ),
    VerbEntry::new(
        "emit_from_gitlab",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a GitLab project (pipelines, MRs, tags, deployments)",
        &["emit_from_gitlab", "gitlab", "git", "pipelines", "merge-requests"],
    ),
    VerbEntry::new(
        "emit_from_monitoring",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from monitoring/observability systems (Datadog, Prometheus, PagerDuty)",
        &["emit_from_monitoring", "monitoring", "observability", "datadog", "prometheus", "alerts"],
    ),
    VerbEntry::new(
        "emit_from_sbom",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest SBOM document events (CycloneDX or SPDX) into the receipt chain",
        &["emit_from_sbom", "sbom", "cyclonedx", "spdx", "components"],
    ),
    VerbEntry::new(
        "emit_from_security",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest security scan events (Snyk, Trivy, Dependabot, SARIF) into the receipt chain",
        &["emit_from_security", "security", "scan", "snyk", "trivy", "sarif"],
    ),
    // ── Compliance ───────────────────────────────────────────────────────────
    VerbEntry::new(
        "verify_compliance",
        "receipt",
        VerbGroup::Compliance,
        "Check evidence presence for a compliance framework (evidence present/absent — not a legal determination)",
        &["verify_compliance", "compliance", "framework", "evidence", "gdpr", "hipaa", "soc2"],
    ),
    VerbEntry::new(
        "verify_sla",
        "receipt",
        VerbGroup::Compliance,
        "Verify that receipt events satisfy defined SLA thresholds and time windows",
        &["verify_sla", "sla", "service-level", "latency", "uptime", "availability"],
    ),
    VerbEntry::new(
        "verify_family",
        "receipt",
        VerbGroup::Compliance,
        "Verify a family of related receipts all satisfy shared constraints",
        &["verify_family", "family", "group", "batch-verify", "related"],
    ),
    VerbEntry::new(
        "policy_enforce",
        "receipt",
        VerbGroup::Compliance,
        "Evaluate receipt events against a Rego or CEL policy file",
        &["policy_enforce", "policy", "rego", "cel", "opa", "enforce"],
    ),
    VerbEntry::new(
        "license_compliance",
        "receipt",
        VerbGroup::Compliance,
        "Check that dependency licenses in the receipt satisfy the project's license policy",
        &["license_compliance", "license", "oss", "open-source", "spdx", "policy"],
    ),
    VerbEntry::new(
        "gdpr_proof",
        "receipt",
        VerbGroup::Compliance,
        "Generate GDPR data-processing evidence from receipt events for auditor review",
        &["gdpr_proof", "gdpr", "privacy", "data-protection", "dpa", "evidence"],
    ),
    VerbEntry::new(
        "hipaa",
        "receipt",
        VerbGroup::Compliance,
        "Collect HIPAA safeguard evidence from receipt events for auditor review",
        &["hipaa", "health", "phi", "safeguard", "audit", "healthcare"],
    ),
    VerbEntry::new(
        "pci_dss",
        "receipt",
        VerbGroup::Compliance,
        "Collect PCI-DSS control evidence from receipt events for auditor review",
        &["pci_dss", "pci", "dss", "payment", "card", "control"],
    ),
    VerbEntry::new(
        "soc2_audit",
        "receipt",
        VerbGroup::Compliance,
        "Collect SOC 2 trust-service-criteria evidence from receipts for auditor review",
        &["soc2_audit", "soc2", "soc", "trust", "service-criteria", "audit"],
    ),
    // ── Attestation ──────────────────────────────────────────────────────────
    VerbEntry::new(
        "sign",
        "receipt",
        VerbGroup::Attestation,
        "Cryptographically sign a sealed receipt with a private key (PEM or PKCS#11)",
        &["sign", "signature", "cryptography", "private-key", "pem"],
    ),
    VerbEntry::new(
        "notarize",
        "receipt",
        VerbGroup::Attestation,
        "Submit a receipt to a transparency log for timestamped notarization (Sigstore/Rekor)",
        &["notarize", "notarization", "transparency", "sigstore", "rekor", "timestamp"],
    ),
    VerbEntry::new(
        "attest",
        "receipt",
        VerbGroup::Attestation,
        "Create a signed SLSA attestation document from a sealed receipt",
        &["attest", "attestation", "slsa", "provenance", "in-toto"],
    ),
    VerbEntry::new(
        "assemble_with_signature",
        "receipt",
        VerbGroup::Attestation,
        "Assemble and immediately sign the receipt in a single atomic operation",
        &["assemble_with_signature", "sign", "assemble", "atomic", "seal"],
    ),
    VerbEntry::new(
        "assemble_and_notarize",
        "receipt",
        VerbGroup::Attestation,
        "Assemble, seal, and submit to a transparency log in one step",
        &["assemble_and_notarize", "notarize", "assemble", "atomic", "rekor"],
    ),
    VerbEntry::new(
        "sbom_attest",
        "receipt",
        VerbGroup::Attestation,
        "Attach a SLSA attestation to a previously generated SBOM document",
        &["sbom_attest", "sbom", "attestation", "slsa", "supply-chain"],
    ),
    // ── SBOM ─────────────────────────────────────────────────────────────────
    VerbEntry::new(
        "sbom_scan",
        "receipt",
        VerbGroup::Sbom,
        "Scan a project and emit an SBOM event capturing all detected components",
        &["sbom_scan", "sbom", "scan", "components", "dependencies", "inventory"],
    ),
    VerbEntry::new(
        "sbom_blast_radius",
        "receipt",
        VerbGroup::Sbom,
        "Compute the blast radius of a vulnerable component across the SBOM dependency graph",
        &["sbom_blast_radius", "blast-radius", "impact", "vulnerability", "transitive"],
    ),
    VerbEntry::new(
        "sbom_compliance",
        "receipt",
        VerbGroup::Sbom,
        "Verify that the SBOM satisfies a license or security policy",
        &["sbom_compliance", "sbom", "compliance", "license", "security", "policy"],
    ),
    VerbEntry::new(
        "sbom_ntia",
        "receipt",
        VerbGroup::Sbom,
        "Check an SBOM document against the NTIA minimum-element requirements",
        &["sbom_ntia", "ntia", "minimum-elements", "sbom", "executive-order"],
    ),
    // ── Insights ─────────────────────────────────────────────────────────────
    VerbEntry::new(
        "anomaly_detect",
        "receipt",
        VerbGroup::Insights,
        "Detect anomalous events in the receipt chain using statistical outlier analysis",
        &["anomaly_detect", "anomaly", "outlier", "detect", "unusual", "statistics"],
    ),
    VerbEntry::new(
        "predict",
        "receipt",
        VerbGroup::Insights,
        "Forecast future chain state or failure probability based on historical receipt patterns",
        &["predict", "forecast", "probability", "failure", "ml", "trend"],
    ),
    VerbEntry::new(
        "trend_analysis",
        "receipt",
        VerbGroup::Insights,
        "Compute rolling trends over event frequency, latency, and error rates across receipts",
        &["trend_analysis", "trend", "rolling", "frequency", "latency", "error-rate"],
    ),
    VerbEntry::new(
        "variance",
        "receipt",
        VerbGroup::Insights,
        "Measure variance in event timing and payload size across a collection of receipts",
        &["variance", "spread", "deviation", "timing", "payload-size"],
    ),
    VerbEntry::new(
        "find_blast_radius",
        "receipt",
        VerbGroup::Insights,
        "Find all receipt events transitively affected by a given failing event",
        &["find_blast_radius", "blast-radius", "impact", "transitive", "cascade"],
    ),
    VerbEntry::new(
        "explain_incident",
        "receipt",
        VerbGroup::Insights,
        "Generate a structured incident explanation from events surrounding a failure",
        &["explain_incident", "incident", "explain", "postmortem", "rca"],
    ),
    VerbEntry::new(
        "causality_chain",
        "receipt",
        VerbGroup::Insights,
        "Build a causality chain graph linking events by causal relationships",
        &["causality_chain", "causality", "cause", "effect", "graph", "chain"],
    ),
    // ── Engineering ──────────────────────────────────────────────────────────
    VerbEntry::new(
        "bus_factor",
        "receipt",
        VerbGroup::Engineering,
        "Compute the bus factor for each object in the chain based on contributor events",
        &["bus_factor", "bus-factor", "knowledge", "risk", "contributors", "single-point"],
    ),
    VerbEntry::new(
        "dora_metrics",
        "receipt",
        VerbGroup::Engineering,
        "Extract DORA metrics (deployment frequency, lead time, MTTR, change failure rate) from receipts",
        &["dora_metrics", "dora", "deployment", "lead-time", "mttr", "change-failure"],
    ),
    VerbEntry::new(
        "team_velocity",
        "receipt",
        VerbGroup::Engineering,
        "Measure team throughput and cycle time from emit/assemble event pairs",
        &["team_velocity", "velocity", "throughput", "cycle-time", "team"],
    ),
    VerbEntry::new(
        "portfolio_health",
        "receipt",
        VerbGroup::Engineering,
        "Aggregate health signals across a portfolio of receipts into a dashboard-ready report",
        &["portfolio_health", "portfolio", "health", "dashboard", "aggregate"],
    ),
    VerbEntry::new(
        "orphaned_code",
        "receipt",
        VerbGroup::Engineering,
        "Identify object references in the receipt that have no corresponding emit events",
        &["orphaned_code", "orphaned", "dead-code", "unreferenced", "objects"],
    ),
    VerbEntry::new(
        "dependency_matrix",
        "receipt",
        VerbGroup::Engineering,
        "Build an object-to-object dependency matrix from co-occurrence in receipt events",
        &["dependency_matrix", "dependency", "matrix", "coupling", "objects"],
    ),
    // ── Tooling ──────────────────────────────────────────────────────────────
    VerbEntry::new(
        "catalog",
        "receipt",
        VerbGroup::Tooling,
        "Index a directory of receipts into a local catalog for fast search and lookup",
        &["catalog", "index", "registry", "store", "database"],
    ),
    VerbEntry::new(
        "search",
        "receipt",
        VerbGroup::Tooling,
        "Search the local receipt catalog by event type, object, date range, or keyword",
        &["search", "find", "query", "lookup", "catalog"],
    ),
    VerbEntry::new(
        "profile",
        "receipt",
        VerbGroup::Tooling,
        "Profile the performance of verify and assemble operations on a receipt",
        &["profile", "benchmark", "perf", "performance", "timing"],
    ),
    VerbEntry::new(
        "install_git_hook",
        "receipt",
        VerbGroup::Tooling,
        "Install a git hook that auto-emits receipt events on commit, push, or tag",
        &["install_git_hook", "git", "hook", "pre-commit", "post-commit", "auto-emit"],
    ),
    VerbEntry::new(
        "monitor",
        "receipt",
        VerbGroup::Tooling,
        "Watch a working receipt file and stream events to stdout as they are appended",
        &["monitor", "watch", "stream", "tail", "live"],
    ),
    VerbEntry::new(
        "receipt_throughput",
        "receipt",
        VerbGroup::Tooling,
        "Measure how many receipts per second the local store can verify under load",
        &["receipt_throughput", "throughput", "load", "benchmark", "performance"],
    ),
    VerbEntry::new(
        "visualize",
        "receipt",
        VerbGroup::Tooling,
        "Generate an interactive HTML or SVG visualization of the receipt chain",
        &["visualize", "html", "svg", "interactive", "chart", "render"],
    ),
    VerbEntry::new(
        "test",
        "receipt",
        VerbGroup::Tooling,
        "Run the built-in receipt self-test suite to validate the local installation",
        &["test", "self-test", "smoke-test", "sanity", "validate"],
    ),
];

/// Look up a verb by `(verb, noun)` pair.
///
/// Returns `None` if the combination is not registered.
pub fn lookup(verb: &str, noun: &str) -> Option<&'static VerbEntry> {
    REGISTRY.iter().find(|e| e.verb == verb && e.noun == noun)
}

/// Return all verbs belonging to the given [`VerbGroup`].
pub fn by_group(group: VerbGroup) -> Vec<&'static VerbEntry> {
    REGISTRY.iter().filter(|e| e.group == group).collect()
}

/// Fuzzy "did you mean" — find up to 5 verbs whose name or keywords
/// contain `input` as a substring (case-insensitive).
///
/// Results are sorted so exact prefix matches on the verb name appear first.
pub fn did_you_mean(input: &str) -> Vec<&'static VerbEntry> {
    let input_lower = input.to_lowercase();
    let mut matches: Vec<&'static VerbEntry> = REGISTRY
        .iter()
        .filter(|e| {
            e.verb.contains(&*input_lower) || e.keywords.iter().any(|k| k.contains(&*input_lower))
        })
        .collect();
    matches.sort_by_key(|e| {
        if e.verb.starts_with(&*input_lower) {
            0u8
        } else {
            1u8
        }
    });
    matches.truncate(5);
    matches
}

/// The number of verbs registered in [`REGISTRY`].
///
/// Use this as the authoritative count instead of hard-coding a literal in
/// documentation or completions — call `registry::verb_count()` instead.
pub fn verb_count() -> usize {
    REGISTRY.len()
}

/// Full-text search across verb name, summary, and keywords.
///
/// Returns all entries that match any of the whitespace-split query tokens,
/// ranked by hit count descending, then verb name ascending.
pub fn search(query: &str) -> Vec<&'static VerbEntry> {
    let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
    if tokens.is_empty() {
        return REGISTRY.iter().collect();
    }

    let mut scored: Vec<(usize, &'static VerbEntry)> = REGISTRY
        .iter()
        .filter_map(|e| {
            let haystack = format!(
                "{} {} {} {}",
                e.verb,
                e.noun,
                e.summary,
                e.keywords.join(" ")
            )
            .to_lowercase();
            let hits = tokens
                .iter()
                .filter(|t| haystack.contains(t.as_str()))
                .count();
            if hits > 0 {
                Some((hits, e))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.verb.cmp(b.1.verb)));
    scored.into_iter().map(|(_, e)| e).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_entry_count_matches_constant() {
        // Update this number whenever you add or remove verbs from REGISTRY.
        let expected = 69; // 67 original + why + fix
        assert_eq!(
            verb_count(),
            expected,
            "REGISTRY has {} entries but expected {}. Update this test and any docs that hard-code the verb count.",
            verb_count(), expected
        );
    }

    #[test]
    fn search_finds_verify_for_certify() {
        let results = search("certify");
        assert!(
            !results.is_empty(),
            "search for 'certify' should return results"
        );
        assert!(
            results.iter().any(|e| e.verb == "verify"),
            "verify should match 'certify'"
        );
    }

    #[test]
    fn all_entries_have_non_empty_fields() {
        for entry in REGISTRY {
            assert!(!entry.verb.is_empty(), "verb is empty");
            assert!(
                !entry.noun.is_empty(),
                "noun is empty for verb {}",
                entry.verb
            );
            assert!(
                !entry.summary.is_empty(),
                "summary is empty for verb {}",
                entry.verb
            );
            assert!(
                !entry.keywords.is_empty(),
                "keywords is empty for verb {}",
                entry.verb
            );
        }
    }

    #[test]
    fn lookup_core_verbs() {
        for verb in &["emit", "assemble", "verify", "show", "inspect", "stats"] {
            let entry = lookup(verb, "receipt")
                .unwrap_or_else(|| panic!("verb '{}' not found in registry", verb));
            assert_eq!(entry.group, VerbGroup::Core);
        }
    }

    #[test]
    fn lookup_missing_returns_none() {
        assert!(lookup("nonexistent", "receipt").is_none());
        assert!(lookup("emit", "nonexistent").is_none());
    }

    #[test]
    fn by_group_returns_correct_group() {
        let core = by_group(VerbGroup::Core);
        assert!(!core.is_empty());
        for entry in &core {
            assert_eq!(entry.group, VerbGroup::Core);
        }
    }

    #[test]
    fn did_you_mean_returns_suggestions() {
        let suggestions = did_you_mean("emit");
        assert!(!suggestions.is_empty());
        // "emit" itself should appear first as an exact prefix match
        assert_eq!(suggestions[0].verb, "emit");
    }

    #[test]
    fn did_you_mean_max_five_results() {
        // "e" matches many verbs; result must be capped at 5
        let suggestions = did_you_mean("e");
        assert!(suggestions.len() <= 5);
    }

    #[test]
    fn all_groups_have_at_least_one_entry() {
        let groups = [
            VerbGroup::Core,
            VerbGroup::Diagnostics,
            VerbGroup::Analysis,
            VerbGroup::Ingestion,
            VerbGroup::Compliance,
            VerbGroup::Attestation,
            VerbGroup::Sbom,
            VerbGroup::Insights,
            VerbGroup::Engineering,
            VerbGroup::Tooling,
        ];
        for group in groups {
            let entries = by_group(group);
            assert!(
                !entries.is_empty(),
                "group {:?} has no entries in REGISTRY",
                group
            );
        }
    }
}
