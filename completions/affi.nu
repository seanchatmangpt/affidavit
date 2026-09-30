# Nushell completion — load via `source completions/affi.nu`
#
# affi.nu — shell completion for the `affi` CLI (affidavit Provenance Layer).
#
# GENERATED FILE — DO NOT EDIT.
# Generator: python3 scripts/generate_completions.py
# Source: src/registry.rs — Registry verbs: 92, Registry nouns: 9
# Regenerate after every registry change; tests/completions_drift.rs enforces it.
#
# Install: add `source /path/to/completions/affi.nu` to your config.nu.

def "nu-complete affi nouns" [] {
    [
        {value: "affi", description: "1 verbs"}
        {value: "ecosystem", description: "2 verbs"}
        {value: "envelope", description: "4 verbs"}
        {value: "errc", description: "4 verbs"}
        {value: "evidence", description: "4 verbs"}
        {value: "guide", description: "1 verbs"}
        {value: "keys", description: "5 verbs"}
        {value: "receipt", description: "69 verbs"}
        {value: "standing", description: "2 verbs"}
    ]
}

def "nu-complete affi verbs" [context: string] {
    let noun = ($context | split row " " | where {|w| $w != ""} | skip 1 | first)
    match $noun {
        "affi" => [
            {value: "doctor", description: "Run environment and receipt-store health checks; optionally apply safe fixes"}
        ]
        "ecosystem" => [
            {value: "certify", description: "Federate sealed member standing receipts into one quorum-scored receipt"}
            {value: "verify", description: "Re-run the federation law over a sealed ecosystem receipt"}
        ]
        "envelope" => [
            {value: "export", description: "Export the attestation envelope of a PQ-SEAL-v1 sealed document: json = the CTP-ENVELOPE-v1 document (default), sa2a = the SA2A-C2-APPROVAL-v1 approval (JCS-canonical)"}
            {value: "list", description: "List key-store records an envelope may bind to: kid, algorithm, profile, fingerprint, custodian (read-only)"}
            {value: "sign", description: "Seal a receipt under an ES256 key: CTP envelope + signature as a PQ-SEAL-v1 document"}
            {value: "verify", description: "Adjudicate a PQ-SEAL-v1 sealed receipt against the registered keys and print the standing VERDICT (exit 0=VALID, 2=decided otherwise)"}
        ]
        "errc" => [
            {value: "assure", description: "Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt"}
            {value: "certify", description: "Seal a declared ERRC transformation (affidavit/errc/v1)"}
            {value: "verify", description: "Re-run the ERRC directional and preservation laws over a sealed receipt"}
            {value: "verify-assurance", description: "Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt"}
        ]
        "evidence" => [
            {value: "crl-apply", description: "Admit a published CRL file into a fresh revocation list: issuer signature first, then epoch freshness, then merge (atomic on any refusal)"}
            {value: "crl-publish", description: "Publish the signed revocation list (CTP-CRL-v1) for the store's recorded revocations under the registered issuer key"}
            {value: "heads", description: "Audit the standing journal and report the RFC 9162 tree head re-derived from the journal entries alone (signed when custody resolves)"}
            {value: "journal", description: "Record a receipt's cryptographic standing as durable journal evidence: assemble a real receipt, seal it under the custody key, adjudicate the standing, append the hash-chained journal entry"}
        ]
        "guide" => [
            {value: "search", description: "Search the verb registry by keyword to discover relevant commands"}
        ]
        "keys" => [
            {value: "generate", description: "Generate a real ES256 signing key and append its public record to the key store (the secret never touches disk)"}
            {value: "import", description: "Import an externally-held public key (hex) into the key store: fingerprint it and register it under a custodian (origin Imported)"}
            {value: "list", description: "List registered key records: kid, algorithm, fingerprint, custodian"}
            {value: "revoke", description: "Revoke a registered key: append a tamper-evident revocation entry to the checksummed sidecar beside the key store"}
            {value: "rotate", description: "Rotate a registered ES256 key to a freshly generated ES256 successor: the successor signs the rotation record and its public record joins the store"}
        ]
        "receipt" => [
            {value: "anomaly-detect", description: "Detect anomalous events in the receipt chain using statistical outlier analysis"}
            {value: "assemble", description: "Finalize the working receipt into an immutable sealed file"}
            {value: "assemble-and-notarize", description: "Assemble and attach a local trust-plane notarization in one step (no external TSA)"}
            {value: "assemble-with-signature", description: "Assemble and immediately sign the receipt in a single atomic operation"}
            {value: "attest", description: "Create a signed SLSA attestation document from a sealed receipt"}
            {value: "audit", description: "Run a full audit pass over a receipt: chain integrity, commitments, and event completeness"}
            {value: "bus-factor", description: "Compute the bus factor for each object in the chain based on contributor events"}
            {value: "catalog", description: "Index a directory of receipts into a local catalog for fast search and lookup"}
            {value: "causality-chain", description: "Build a causality chain graph linking events by causal relationships"}
            {value: "conformance", description: "Check a receipt against custom conformance rules or a named profile"}
            {value: "coverage-analysis", description: "Measure what fraction of defined event types appear in the receipt"}
            {value: "dependency-matrix", description: "Build an object-to-object dependency matrix from co-occurrence in receipt events"}
            {value: "diagnose", description: "Troubleshoot verification failures with suggested remediation steps"}
            {value: "diff", description: "Compute a structural diff between two receipts showing added, removed, and changed events"}
            {value: "dora-metrics", description: "Extract DORA metrics (deployment frequency, lead time, MTTR, change failure rate) from receipts"}
            {value: "emit", description: "Record an operation-event into the working receipt chain"}
            {value: "emit-batch", description: "Ingest multiple events from a JSONL or CSV file in a single batch operation"}
            {value: "emit-from-cicd", description: "Ingest events from a CI/CD pipeline run (GitHub Actions, GitLab CI, Jenkins, etc.)"}
            {value: "emit-from-cloud", description: "Ingest events from cloud provider audit logs (AWS CloudTrail, GCP Audit, Azure Monitor)"}
            {value: "emit-from-github", description: "Ingest events from a GitHub repository (commits, PRs, releases, workflow runs)"}
            {value: "emit-from-gitlab", description: "Ingest events from a GitLab project (pipelines, MRs, tags, deployments)"}
            {value: "emit-from-monitoring", description: "Ingest events from monitoring/observability systems (Datadog, Prometheus, PagerDuty)"}
            {value: "emit-from-sbom", description: "Ingest SBOM document events (CycloneDX or SPDX) into the receipt chain"}
            {value: "emit-from-security", description: "Ingest security scan events (Snyk, Trivy, Dependabot, SARIF) into the receipt chain"}
            {value: "explain-incident", description: "Generate a structured incident explanation from events surrounding a failure"}
            {value: "find-blast-radius", description: "Find all receipt events transitively affected by a given failing event"}
            {value: "fix", description: "Apply a safe structural repair: quarantine a tampered receipt or finalize a working one. Use --dry-run to preview."}
            {value: "gdpr-proof", description: "Generate GDPR data-processing evidence from receipt events for auditor review"}
            {value: "graph", description: "DAG visualization of event dependencies and object references (dot, mermaid, json)"}
            {value: "hipaa", description: "Collect HIPAA safeguard evidence from receipt events for auditor review"}
            {value: "inspect", description: "Detailed inspection of receipt internals (chain hash, commitments, continuity)"}
            {value: "install-git-hook", description: "Install a git hook that auto-emits receipt events on commit, push, or tag"}
            {value: "license-compliance", description: "Check that dependency licenses in the receipt satisfy the project's license policy"}
            {value: "model", description: "Extract the type schema from a receipt (event types, object types, qualifiers)"}
            {value: "monitor", description: "Watch a working receipt file and stream events to stdout as they are appended"}
            {value: "notarize", description: "Attach a local trust-plane notarization to a receipt (unsigned attestation request, or a real ES256 attestation with AFFI_NOTARY_KEY)"}
            {value: "orphaned-code", description: "Identify object references in the receipt that have no corresponding emit events"}
            {value: "pci-dss", description: "Collect PCI-DSS control evidence from receipt events for auditor review"}
            {value: "policy-enforce", description: "Evaluate receipt events against a Rego or CEL policy file"}
            {value: "portfolio-health", description: "Aggregate health signals across a portfolio of receipts into a dashboard-ready report"}
            {value: "predict", description: "Forecast future chain state or failure probability based on historical receipt patterns"}
            {value: "profile", description: "Profile the performance of verify and assemble operations on a receipt"}
            {value: "query", description: "Query receipt events using a filter expression (event_type, object, seq range)"}
            {value: "receipt-throughput", description: "Measure how many receipts per second the local store can verify under load"}
            {value: "replay", description: "Re-execute chain from stored events, optionally applying custom handlers"}
            {value: "root-cause", description: "Trace failure events back to their causal predecessors in the chain"}
            {value: "sbom-attest", description: "Attach a SLSA attestation to a previously generated SBOM document"}
            {value: "sbom-blast-radius", description: "Compute the blast radius of a vulnerable component across the SBOM dependency graph"}
            {value: "sbom-compliance", description: "Verify that the SBOM satisfies a license or security policy"}
            {value: "sbom-ntia", description: "Check an SBOM document against the NTIA minimum-element requirements"}
            {value: "sbom-scan", description: "Scan a project and emit an SBOM event capturing all detected components"}
            {value: "search", description: "Search the local receipt catalog by event type, object, date range, or keyword"}
            {value: "security-debt", description: "Surface security-relevant events with missing or weak commitments"}
            {value: "show", description: "Human-readable dump of the receipt chain with event details"}
            {value: "sign", description: "Cryptographically sign a sealed receipt with an ES256 key (raw-hex key file; production custody via HSM/Secure Enclave)"}
            {value: "soc2-audit", description: "Collect SOC 2 trust-service-criteria evidence from receipts for auditor review"}
            {value: "stats", description: "Chain metrics: event count, hash distribution, event-type histogram"}
            {value: "team-velocity", description: "Measure team throughput and cycle time from emit/assemble event pairs"}
            {value: "tech-debt", description: "Identify stale or low-quality events that signal accumulated technical debt"}
            {value: "test", description: "Run the built-in receipt self-test suite to validate the local installation"}
            {value: "timeline", description: "Render a temporal timeline of events ordered by sequence number"}
            {value: "trend-analysis", description: "Compute rolling trends over event frequency, latency, and error rates across receipts"}
            {value: "variance", description: "Measure variance in event timing and payload size across a collection of receipts"}
            {value: "verify", description: "Run the 7-stage certify pipeline against a receipt (exit 0=ACCEPT, 2=REJECT)"}
            {value: "verify-compliance", description: "Check evidence presence for a compliance framework (evidence present/absent — not a legal determination)"}
            {value: "verify-family", description: "Verify a family of related receipts all satisfy shared constraints"}
            {value: "verify-sla", description: "Verify that receipt events satisfy defined SLA thresholds and time windows"}
            {value: "visualize", description: "Generate an interactive HTML or SVG visualization of the receipt chain"}
            {value: "why", description: "Explain in plain language why a receipt was rejected, with stage-by-stage remediation steps"}
        ]
        "standing" => [
            {value: "certify", description: "Seal a standing claim over an admitted receipt (affidavit/standing/v2)"}
            {value: "verify", description: "Re-run the standing law over a sealed standing receipt"}
        ]
        _ => []
    }
}

# Top-level command: `affi <noun> <verb> ...`.
export extern "affi" [
    noun?: string@"nu-complete affi nouns"
    verb?: string@"nu-complete affi verbs"
    ...rest: string
    --help(-h)      # Show help
    --version(-V)   # Show version
]
