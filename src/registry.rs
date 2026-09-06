//! # Verb Registry — Single Source of Truth for All 79 Verbs
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
//! assert_eq!(verb_count(), 79);
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
    /// Evidence federation kernel: standing, ecosystem, and ERRC certification courts.
    Federation,
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
            Self::Federation => "Federation",
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
            Self::Federation => "Evidence federation courts: standing, ecosystem quorum, and ERRC transformation certification",
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

/// The complete verb registry — 79 entries, one per live verb.
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
        "root-cause",
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
        "coverage-analysis",
        "receipt",
        VerbGroup::Analysis,
        "Measure what fraction of defined event types appear in the receipt",
        &["coverage", "coverage_analysis", "completeness", "missing", "gaps"],
    ),
    VerbEntry::new(
        "tech-debt",
        "receipt",
        VerbGroup::Analysis,
        "Identify stale or low-quality events that signal accumulated technical debt",
        &["tech_debt", "debt", "quality", "stale", "maintenance"],
    ),
    VerbEntry::new(
        "security-debt",
        "receipt",
        VerbGroup::Analysis,
        "Surface security-relevant events with missing or weak commitments",
        &["security_debt", "security", "vulnerability", "weakness", "cve"],
    ),
    // ── Ingestion ────────────────────────────────────────────────────────────
    VerbEntry::new(
        "emit-batch",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest multiple events from a JSONL or CSV file in a single batch operation",
        &["emit_batch", "batch", "bulk", "import", "ingest", "jsonl"],
    ),
    VerbEntry::new(
        "emit-from-cicd",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a CI/CD pipeline run (GitHub Actions, GitLab CI, Jenkins, etc.)",
        &["emit_from_cicd", "cicd", "ci", "cd", "pipeline", "build"],
    ),
    VerbEntry::new(
        "emit-from-cloud",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from cloud provider audit logs (AWS CloudTrail, GCP Audit, Azure Monitor)",
        &["emit_from_cloud", "cloud", "aws", "gcp", "azure", "cloudtrail"],
    ),
    VerbEntry::new(
        "emit-from-github",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a GitHub repository (commits, PRs, releases, workflow runs)",
        &["emit_from_github", "github", "git", "commits", "prs"],
    ),
    VerbEntry::new(
        "emit-from-gitlab",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from a GitLab project (pipelines, MRs, tags, deployments)",
        &["emit_from_gitlab", "gitlab", "git", "pipelines", "merge-requests"],
    ),
    VerbEntry::new(
        "emit-from-monitoring",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest events from monitoring/observability systems (Datadog, Prometheus, PagerDuty)",
        &["emit_from_monitoring", "monitoring", "observability", "datadog", "prometheus", "alerts"],
    ),
    VerbEntry::new(
        "emit-from-sbom",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest SBOM document events (CycloneDX or SPDX) into the receipt chain",
        &["emit_from_sbom", "sbom", "cyclonedx", "spdx", "components"],
    ),
    VerbEntry::new(
        "emit-from-security",
        "receipt",
        VerbGroup::Ingestion,
        "Ingest security scan events (Snyk, Trivy, Dependabot, SARIF) into the receipt chain",
        &["emit_from_security", "security", "scan", "snyk", "trivy", "sarif"],
    ),
    // ── Compliance ───────────────────────────────────────────────────────────
    VerbEntry::new(
        "verify-compliance",
        "receipt",
        VerbGroup::Compliance,
        "Check evidence presence for a compliance framework (evidence present/absent — not a legal determination)",
        &["verify_compliance", "compliance", "framework", "evidence", "gdpr", "hipaa", "soc2"],
    ),
    VerbEntry::new(
        "verify-sla",
        "receipt",
        VerbGroup::Compliance,
        "Verify that receipt events satisfy defined SLA thresholds and time windows",
        &["verify_sla", "sla", "service-level", "latency", "uptime", "availability"],
    ),
    VerbEntry::new(
        "verify-family",
        "receipt",
        VerbGroup::Compliance,
        "Verify a family of related receipts all satisfy shared constraints",
        &["verify_family", "family", "group", "batch-verify", "related"],
    ),
    VerbEntry::new(
        "policy-enforce",
        "receipt",
        VerbGroup::Compliance,
        "Evaluate receipt events against a Rego or CEL policy file",
        &["policy_enforce", "policy", "rego", "cel", "opa", "enforce"],
    ),
    VerbEntry::new(
        "license-compliance",
        "receipt",
        VerbGroup::Compliance,
        "Check that dependency licenses in the receipt satisfy the project's license policy",
        &["license_compliance", "license", "oss", "open-source", "spdx", "policy"],
    ),
    VerbEntry::new(
        "gdpr-proof",
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
        "pci-dss",
        "receipt",
        VerbGroup::Compliance,
        "Collect PCI-DSS control evidence from receipt events for auditor review",
        &["pci_dss", "pci", "dss", "payment", "card", "control"],
    ),
    VerbEntry::new(
        "soc2-audit",
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
        "assemble-with-signature",
        "receipt",
        VerbGroup::Attestation,
        "Assemble and immediately sign the receipt in a single atomic operation",
        &["assemble_with_signature", "sign", "assemble", "atomic", "seal"],
    ),
    VerbEntry::new(
        "assemble-and-notarize",
        "receipt",
        VerbGroup::Attestation,
        "Assemble, seal, and submit to a transparency log in one step",
        &["assemble_and_notarize", "notarize", "assemble", "atomic", "rekor"],
    ),
    VerbEntry::new(
        "sbom-attest",
        "receipt",
        VerbGroup::Attestation,
        "Attach a SLSA attestation to a previously generated SBOM document",
        &["sbom_attest", "sbom", "attestation", "slsa", "supply-chain"],
    ),
    // ── SBOM ─────────────────────────────────────────────────────────────────
    VerbEntry::new(
        "sbom-scan",
        "receipt",
        VerbGroup::Sbom,
        "Scan a project and emit an SBOM event capturing all detected components",
        &["sbom_scan", "sbom", "scan", "components", "dependencies", "inventory"],
    ),
    VerbEntry::new(
        "sbom-blast-radius",
        "receipt",
        VerbGroup::Sbom,
        "Compute the blast radius of a vulnerable component across the SBOM dependency graph",
        &["sbom_blast_radius", "blast-radius", "impact", "vulnerability", "transitive"],
    ),
    VerbEntry::new(
        "sbom-compliance",
        "receipt",
        VerbGroup::Sbom,
        "Verify that the SBOM satisfies a license or security policy",
        &["sbom_compliance", "sbom", "compliance", "license", "security", "policy"],
    ),
    VerbEntry::new(
        "sbom-ntia",
        "receipt",
        VerbGroup::Sbom,
        "Check an SBOM document against the NTIA minimum-element requirements",
        &["sbom_ntia", "ntia", "minimum-elements", "sbom", "executive-order"],
    ),
    // ── Insights ─────────────────────────────────────────────────────────────
    VerbEntry::new(
        "anomaly-detect",
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
        "trend-analysis",
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
        "find-blast-radius",
        "receipt",
        VerbGroup::Insights,
        "Find all receipt events transitively affected by a given failing event",
        &["find_blast_radius", "blast-radius", "impact", "transitive", "cascade"],
    ),
    VerbEntry::new(
        "explain-incident",
        "receipt",
        VerbGroup::Insights,
        "Generate a structured incident explanation from events surrounding a failure",
        &["explain_incident", "incident", "explain", "postmortem", "rca"],
    ),
    VerbEntry::new(
        "causality-chain",
        "receipt",
        VerbGroup::Insights,
        "Build a causality chain graph linking events by causal relationships",
        &["causality_chain", "causality", "cause", "effect", "graph", "chain"],
    ),
    // ── Engineering ──────────────────────────────────────────────────────────
    VerbEntry::new(
        "bus-factor",
        "receipt",
        VerbGroup::Engineering,
        "Compute the bus factor for each object in the chain based on contributor events",
        &["bus_factor", "bus-factor", "knowledge", "risk", "contributors", "single-point"],
    ),
    VerbEntry::new(
        "dora-metrics",
        "receipt",
        VerbGroup::Engineering,
        "Extract DORA metrics (deployment frequency, lead time, MTTR, change failure rate) from receipts",
        &["dora_metrics", "dora", "deployment", "lead-time", "mttr", "change-failure"],
    ),
    VerbEntry::new(
        "team-velocity",
        "receipt",
        VerbGroup::Engineering,
        "Measure team throughput and cycle time from emit/assemble event pairs",
        &["team_velocity", "velocity", "throughput", "cycle-time", "team"],
    ),
    VerbEntry::new(
        "portfolio-health",
        "receipt",
        VerbGroup::Engineering,
        "Aggregate health signals across a portfolio of receipts into a dashboard-ready report",
        &["portfolio_health", "portfolio", "health", "dashboard", "aggregate"],
    ),
    VerbEntry::new(
        "orphaned-code",
        "receipt",
        VerbGroup::Engineering,
        "Identify object references in the receipt that have no corresponding emit events",
        &["orphaned_code", "orphaned", "dead-code", "unreferenced", "objects"],
    ),
    VerbEntry::new(
        "dependency-matrix",
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
        "install-git-hook",
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
        "receipt-throughput",
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
    // ── Tooling (non-receipt nouns) ─────────────────────────────────────────
    VerbEntry::new(
        "doctor",
        "affi",
        VerbGroup::Tooling,
        "Run environment and receipt-store health checks; optionally apply safe fixes",
        &["doctor", "health", "check", "diagnose", "environment", "fix"],
    )
    .with_example("affi affi doctor --receipts ./receipts --fix"),
    VerbEntry::new(
        "search",
        "guide",
        VerbGroup::Tooling,
        "Search the verb registry by keyword to discover relevant commands",
        &["search", "guide", "discover", "find", "keyword", "help"],
    )
    .with_example("affi guide search federation"),

    // ── Federation ──────────────────────────────────────────────────────────
    VerbEntry::new(
        "certify",
        "standing",
        VerbGroup::Federation,
        "Seal a standing claim over an admitted receipt (affidavit/standing/v2)",
        &["certify", "standing", "alive", "evidence", "federation", "kernel"],
    )
    .with_example(
        "affi standing certify --receipt r.json --observation o.json --scope repo:acme/app --out standing.json",
    ),
    VerbEntry::new(
        "verify",
        "standing",
        VerbGroup::Federation,
        "Re-run the standing law over a sealed standing receipt",
        &["verify", "standing", "certify", "seal", "federation"],
    )
    .with_example("affi standing verify --receipt standing.json"),
    VerbEntry::new(
        "certify",
        "ecosystem",
        VerbGroup::Federation,
        "Federate sealed member standing receipts into one quorum-scored receipt",
        &["certify", "ecosystem", "federation", "quorum", "role", "cross-repo"],
    )
    .with_example(
        "affi ecosystem certify --receipt r.json --observation federation.json --out eco.json",
    ),
    VerbEntry::new(
        "verify",
        "ecosystem",
        VerbGroup::Federation,
        "Re-run the federation law over a sealed ecosystem receipt",
        &["verify", "ecosystem", "federation", "quorum", "seal"],
    )
    .with_example("affi ecosystem verify --receipt ecosystem.json"),
    VerbEntry::new(
        "certify",
        "errc",
        VerbGroup::Federation,
        "Seal a declared ERRC transformation (affidavit/errc/v1)",
        &["certify", "errc", "eliminate", "reduce", "raise", "create", "transformation"],
    )
    .with_example("affi errc certify --receipt r.json --observation errc.json --out errc.json"),
    VerbEntry::new(
        "verify",
        "errc",
        VerbGroup::Federation,
        "Re-run the ERRC directional and preservation laws over a sealed receipt",
        &["verify", "errc", "directional", "preservation", "fence", "seal"],
    )
    .with_example("affi errc verify --receipt errc.json"),
    VerbEntry::new(
        "assure",
        "errc",
        VerbGroup::Federation,
        "Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt",
        &["assure", "errc", "claim", "witness", "ledger", "assurance", "bijection"],
    )
    .with_example(
        "affi errc assure --parent errc.json --witnesses w.json --out assurance.json",
    ),
    VerbEntry::new(
        "verify-assurance",
        "errc",
        VerbGroup::Federation,
        "Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt",
        &["verify_assurance", "errc", "claim", "assurance", "parent", "binding"],
    )
    .with_example(
        "affi errc verify-assurance --receipt assurance.json --parent errc.json",
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
        let expected = 79; // 67 original + why + fix + affi doctor + guide search + 8 federation courts
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

    /// Read the authoritative ontology.
    fn ontology_source() -> String {
        std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ontology/affi-cli.ttl"
        ))
        .expect("ontology/affi-cli.ttl is readable")
    }

    /// Pull the quoted literal that follows `property` on a line, if present.
    fn literal_after<'a>(line: &'a str, property: &str) -> Option<&'a str> {
        let rest = line.trim().strip_prefix(property)?.trim_start();
        rest.strip_prefix('"')?.split('"').next()
    }

    /// The `(verb, noun)` pairs the ontology declares.
    ///
    /// Names alone are not enough: `search` is declared under both the
    /// `receipt` noun (payload grep) and the `guide` noun (registry
    /// discovery), so a name-only check would call `guide search` declared
    /// purely because `receipt search` exists. This walks each `a cnv:Verb`
    /// block, pairs its `cnv:hasVerbName` with the `cnv:hasNounName` of the
    /// noun its `cnv:belongsToNoun` points at, and compares pairs.
    fn ontology_verb_noun_pairs() -> std::collections::HashSet<(String, String)> {
        let ttl = ontology_source();

        // Resource id (e.g. `affi:ReceiptNoun`) -> declared noun name.
        let mut noun_names = std::collections::HashMap::new();
        let mut current_resource: Option<String> = None;
        for line in ttl.lines() {
            let trimmed = line.trim();
            if let Some(id) = trimmed.strip_suffix(" a cnv:Noun ;") {
                current_resource = Some(id.trim().to_string());
            }
            if let (Some(resource), Some(name)) = (
                current_resource.as_ref(),
                literal_after(trimmed, "cnv:hasNounName"),
            ) {
                noun_names.insert(resource.clone(), name.to_string());
                current_resource = None;
            }
        }

        let mut pairs = std::collections::HashSet::new();
        let mut verb_name: Option<String> = None;
        let mut in_verb = false;
        for line in ttl.lines() {
            let trimmed = line.trim();
            if trimmed.contains(" a cnv:Verb ;") {
                in_verb = true;
                verb_name = None;
                continue;
            }
            if !in_verb {
                continue;
            }
            if let Some(name) = literal_after(trimmed, "cnv:hasVerbName") {
                verb_name = Some(name.to_string());
            }
            if let Some(rest) = trimmed.strip_prefix("cnv:belongsToNoun ") {
                let resource = rest.trim_end_matches([';', '.', ' ']).trim();
                if let (Some(verb), Some(noun)) = (verb_name.as_ref(), noun_names.get(resource)) {
                    pairs.insert((verb.clone(), noun.clone()));
                }
                in_verb = false;
            }
        }
        pairs
    }

    /// The ontology is the authoritative input for the CLI surface
    /// (AGENTS.md §5): `src/verbs/**` is a projection of it. ggen cannot run in
    /// every environment, so this test is what keeps the projection and its
    /// source from drifting — it is the reason four verbs that had shipped
    /// undeclared (`why`, `fix`, `install-git-hook`, `monitor`) were declared
    /// in v26.9.6. Registry names use `_`; the ontology uses `-`.
    #[test]
    fn every_registry_verb_is_declared_in_the_ontology() {
        let declared = ontology_verb_noun_pairs();
        assert!(
            !declared.is_empty(),
            "no (verb, noun) pairs parsed out of ontology/affi-cli.ttl"
        );

        let undeclared: Vec<String> = REGISTRY
            .iter()
            .filter(|entry| !declared.contains(&(entry.verb.to_string(), entry.noun.to_string())))
            .map(|entry| format!("{} {}", entry.noun, entry.verb))
            .collect();
        assert!(
            undeclared.is_empty(),
            "these commands exist in REGISTRY but are not declared in ontology/affi-cli.ttl: {undeclared:?}. \
             Declare them in the ontology — the projection is not the source of truth."
        );
    }

    /// Every registry entry must have a real `#[verb(...)]` projection **in a
    /// module that is actually declared**, or the registry is advertising a
    /// command the binary cannot run.
    ///
    /// Scanning `src/verbs/*.rs` alone is not enough: a file can sit in the
    /// directory with a perfectly good `#[verb]` on it and never be compiled
    /// because nobody added its `pub mod` line. That is exactly what happened
    /// to `receipt-throughput`, which REGISTRY advertised for two releases
    /// while `src/verbs/mod.rs` did not declare it. So this walks the module
    /// list first and only reads files it names.
    #[test]
    fn every_registry_entry_has_a_verb_projection() {
        let verbs_dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/verbs"));
        let mod_rs = std::fs::read_to_string(verbs_dir.join("mod.rs"))
            .expect("src/verbs/mod.rs is readable");
        let declared_modules: Vec<&str> = mod_rs
            .lines()
            .filter_map(|line| line.trim().strip_prefix("pub mod ")?.strip_suffix(';'))
            .collect();
        assert!(
            !declared_modules.is_empty(),
            "no `pub mod` declarations found in src/verbs/mod.rs"
        );

        let mut projected = std::collections::HashSet::new();
        for module in &declared_modules {
            let path = verbs_dir.join(format!("{module}.rs"));
            let src = std::fs::read_to_string(&path).unwrap_or_else(|_| {
                panic!("src/verbs/mod.rs declares `{module}` but {path:?} is unreadable")
            });
            for line in src.lines() {
                let Some(rest) = line.trim().strip_prefix("#[verb(\"") else {
                    continue;
                };
                let mut parts = rest.split('"');
                let (Some(verb), Some(_), Some(noun)) = (parts.next(), parts.next(), parts.next())
                else {
                    continue;
                };
                projected.insert((verb.to_string(), noun.to_string()));
            }
        }

        // Exact match, no normalisation. The CLI dispatches on the literal in
        // `#[verb(...)]`; a REGISTRY row spelled `verify_compliance` names a
        // command that does not exist, so `lookup` misses and `guide search`
        // prints something the operator cannot type.
        let unprojected: Vec<String> = REGISTRY
            .iter()
            .filter(|entry| !projected.contains(&(entry.verb.to_string(), entry.noun.to_string())))
            .map(|entry| format!("{} {}", entry.noun, entry.verb))
            .collect();
        assert!(
            unprojected.is_empty(),
            "REGISTRY advertises verbs with no compiled #[verb] projection: {unprojected:?}. \
             Check both the token spelling (the CLI is kebab-case) and the `pub mod` line."
        );

        // And the reverse: a compiled verb missing from REGISTRY is invisible to
        // `--help` grouping, `guide search`, and the completions, so operators
        // cannot discover it even though the binary answers it.
        let registered: std::collections::HashSet<(String, String)> = REGISTRY
            .iter()
            .map(|entry| (entry.verb.to_string(), entry.noun.to_string()))
            .collect();
        let unregistered: Vec<String> = projected
            .difference(&registered)
            .map(|(verb, noun)| format!("{noun} {verb}"))
            .collect();
        assert!(
            unregistered.is_empty(),
            "these verbs are compiled and dispatchable but absent from REGISTRY, so they are \
             undiscoverable: {unregistered:?}"
        );
    }

    /// Verb tokens are what the operator types. Snake_case is never dispatchable.
    #[test]
    fn no_registry_verb_token_uses_snake_case() {
        let snake: Vec<&str> = REGISTRY
            .iter()
            .map(|entry| entry.verb)
            .filter(|verb| verb.contains('_'))
            .collect();
        assert!(
            snake.is_empty(),
            "these REGISTRY verb tokens use `_` but the CLI dispatches kebab-case: {snake:?}"
        );
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
            VerbGroup::Federation,
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
