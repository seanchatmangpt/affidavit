#compdef affi

# zsh completion — install per `fpath` + `compinit` (see below)
#
# affi.zsh — shell completion for the `affi` CLI (affidavit Provenance Layer).
#
# GENERATED FILE — DO NOT EDIT.
# Generator: python3 scripts/generate_completions.py
# Source: src/registry.rs — Registry verbs: 86, Registry nouns: 8
# Regenerate after every registry change; tests/completions_drift.rs enforces it.
#
# Install (per-user):
#   mkdir -p ~/.zsh/completions
#   cp completions/affi.zsh ~/.zsh/completions/_affi
#   # In ~/.zshrc (before compinit):
#   #   fpath=(~/.zsh/completions $fpath)
#   #   autoload -Uz compinit && compinit

_affi() {
    local state

    _arguments \
        '(-h --help)'{-h,--help}'[Show help]' \
        '(-V --version)'{-V,--version}'[Show version]' \
        '1: :->noun' \
        '2: :->verb' \
        '*: :->args' && return 0

    case "${state}" in
        noun)
            _values 'noun' \
                'affi[1 verbs]' \
                'ecosystem[2 verbs]' \
                'envelope[2 verbs]' \
                'errc[4 verbs]' \
                'guide[1 verbs]' \
                'keys[5 verbs]' \
                'receipt[69 verbs]' \
                'standing[2 verbs]'
            ;;
        verb)
            case "${words[2]}" in
                affi)
                    _values 'verb' \
                        'doctor[Run environment and receipt-store health checks; optionally apply safe fixes]'
                    ;;
                ecosystem)
                    _values 'verb' \
                        'certify[Federate sealed member standing receipts into one quorum-scored receipt]' \
                        'verify[Re-run the federation law over a sealed ecosystem receipt]'
                    ;;
                envelope)
                    _values 'verb' \
                        'sign[Seal a receipt under an ES256 key\: CTP envelope + signature as a PQ-SEAL-v1 document]' \
                        'verify[Adjudicate a PQ-SEAL-v1 sealed receipt against the registered keys and print the standing VERDICT (exit 0=VALID, 2=decided otherwise)]'
                    ;;
                errc)
                    _values 'verb' \
                        'assure[Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt]' \
                        'certify[Seal a declared ERRC transformation (affidavit/errc/v1)]' \
                        'verify[Re-run the ERRC directional and preservation laws over a sealed receipt]' \
                        'verify-assurance[Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt]'
                    ;;
                guide)
                    _values 'verb' \
                        'search[Search the verb registry by keyword to discover relevant commands]'
                    ;;
                keys)
                    _values 'verb' \
                        'generate[Generate a real ES256 signing key and append its public record to the key store (the secret never touches disk)]' \
                        'import[Import an externally-held public key (hex) into the key store\: fingerprint it and register it under a custodian (origin Imported)]' \
                        'list[List registered key records\: kid, algorithm, fingerprint, custodian]' \
                        'revoke[Revoke a registered key\: append a tamper-evident revocation entry to the checksummed sidecar beside the key store]' \
                        'rotate[Rotate a registered ES256 key to a freshly generated ES256 successor\: the successor signs the rotation record and its public record joins the store]'
                    ;;
                receipt)
                    _values 'verb' \
                        'anomaly-detect[Detect anomalous events in the receipt chain using statistical outlier analysis]' \
                        'assemble[Finalize the working receipt into an immutable sealed file]' \
                        'assemble-and-notarize[Assemble and attach a local trust-plane notarization in one step (no external TSA)]' \
                        'assemble-with-signature[Assemble and immediately sign the receipt in a single atomic operation]' \
                        'attest[Create a signed SLSA attestation document from a sealed receipt]' \
                        'audit[Run a full audit pass over a receipt\: chain integrity, commitments, and event completeness]' \
                        'bus-factor[Compute the bus factor for each object in the chain based on contributor events]' \
                        'catalog[Index a directory of receipts into a local catalog for fast search and lookup]' \
                        'causality-chain[Build a causality chain graph linking events by causal relationships]' \
                        'conformance[Check a receipt against custom conformance rules or a named profile]' \
                        'coverage-analysis[Measure what fraction of defined event types appear in the receipt]' \
                        'dependency-matrix[Build an object-to-object dependency matrix from co-occurrence in receipt events]' \
                        'diagnose[Troubleshoot verification failures with suggested remediation steps]' \
                        'diff[Compute a structural diff between two receipts showing added, removed, and changed events]' \
                        'dora-metrics[Extract DORA metrics (deployment frequency, lead time, MTTR, change failure rate) from receipts]' \
                        'emit[Record an operation-event into the working receipt chain]' \
                        'emit-batch[Ingest multiple events from a JSONL or CSV file in a single batch operation]' \
                        'emit-from-cicd[Ingest events from a CI/CD pipeline run (GitHub Actions, GitLab CI, Jenkins, etc.)]' \
                        'emit-from-cloud[Ingest events from cloud provider audit logs (AWS CloudTrail, GCP Audit, Azure Monitor)]' \
                        'emit-from-github[Ingest events from a GitHub repository (commits, PRs, releases, workflow runs)]' \
                        'emit-from-gitlab[Ingest events from a GitLab project (pipelines, MRs, tags, deployments)]' \
                        'emit-from-monitoring[Ingest events from monitoring/observability systems (Datadog, Prometheus, PagerDuty)]' \
                        'emit-from-sbom[Ingest SBOM document events (CycloneDX or SPDX) into the receipt chain]' \
                        'emit-from-security[Ingest security scan events (Snyk, Trivy, Dependabot, SARIF) into the receipt chain]' \
                        'explain-incident[Generate a structured incident explanation from events surrounding a failure]' \
                        'find-blast-radius[Find all receipt events transitively affected by a given failing event]' \
                        'fix[Apply a safe structural repair\: quarantine a tampered receipt or finalize a working one. Use --dry-run to preview.]' \
                        'gdpr-proof[Generate GDPR data-processing evidence from receipt events for auditor review]' \
                        'graph[DAG visualization of event dependencies and object references (dot, mermaid, json)]' \
                        'hipaa[Collect HIPAA safeguard evidence from receipt events for auditor review]' \
                        'inspect[Detailed inspection of receipt internals (chain hash, commitments, continuity)]' \
                        'install-git-hook[Install a git hook that auto-emits receipt events on commit, push, or tag]' \
                        'license-compliance[Check that dependency licenses in the receipt satisfy the project'\''s license policy]' \
                        'model[Extract the type schema from a receipt (event types, object types, qualifiers)]' \
                        'monitor[Watch a working receipt file and stream events to stdout as they are appended]' \
                        'notarize[Attach a local trust-plane notarization to a receipt (unsigned attestation request, or a real ES256 attestation with AFFI_NOTARY_KEY)]' \
                        'orphaned-code[Identify object references in the receipt that have no corresponding emit events]' \
                        'pci-dss[Collect PCI-DSS control evidence from receipt events for auditor review]' \
                        'policy-enforce[Evaluate receipt events against a Rego or CEL policy file]' \
                        'portfolio-health[Aggregate health signals across a portfolio of receipts into a dashboard-ready report]' \
                        'predict[Forecast future chain state or failure probability based on historical receipt patterns]' \
                        'profile[Profile the performance of verify and assemble operations on a receipt]' \
                        'query[Query receipt events using a filter expression (event_type, object, seq range)]' \
                        'receipt-throughput[Measure how many receipts per second the local store can verify under load]' \
                        'replay[Re-execute chain from stored events, optionally applying custom handlers]' \
                        'root-cause[Trace failure events back to their causal predecessors in the chain]' \
                        'sbom-attest[Attach a SLSA attestation to a previously generated SBOM document]' \
                        'sbom-blast-radius[Compute the blast radius of a vulnerable component across the SBOM dependency graph]' \
                        'sbom-compliance[Verify that the SBOM satisfies a license or security policy]' \
                        'sbom-ntia[Check an SBOM document against the NTIA minimum-element requirements]' \
                        'sbom-scan[Scan a project and emit an SBOM event capturing all detected components]' \
                        'search[Search the local receipt catalog by event type, object, date range, or keyword]' \
                        'security-debt[Surface security-relevant events with missing or weak commitments]' \
                        'show[Human-readable dump of the receipt chain with event details]' \
                        'sign[Cryptographically sign a sealed receipt with an ES256 key (raw-hex key file; production custody via HSM/Secure Enclave)]' \
                        'soc2-audit[Collect SOC 2 trust-service-criteria evidence from receipts for auditor review]' \
                        'stats[Chain metrics\: event count, hash distribution, event-type histogram]' \
                        'team-velocity[Measure team throughput and cycle time from emit/assemble event pairs]' \
                        'tech-debt[Identify stale or low-quality events that signal accumulated technical debt]' \
                        'test[Run the built-in receipt self-test suite to validate the local installation]' \
                        'timeline[Render a temporal timeline of events ordered by sequence number]' \
                        'trend-analysis[Compute rolling trends over event frequency, latency, and error rates across receipts]' \
                        'variance[Measure variance in event timing and payload size across a collection of receipts]' \
                        'verify[Run the 7-stage certify pipeline against a receipt (exit 0=ACCEPT, 2=REJECT)]' \
                        'verify-compliance[Check evidence presence for a compliance framework (evidence present/absent — not a legal determination)]' \
                        'verify-family[Verify a family of related receipts all satisfy shared constraints]' \
                        'verify-sla[Verify that receipt events satisfy defined SLA thresholds and time windows]' \
                        'visualize[Generate an interactive HTML or SVG visualization of the receipt chain]' \
                        'why[Explain in plain language why a receipt was rejected, with stage-by-stage remediation steps]'
                    ;;
                standing)
                    _values 'verb' \
                        'certify[Seal a standing claim over an admitted receipt (affidavit/standing/v2)]' \
                        'verify[Re-run the standing law over a sealed standing receipt]'
                    ;;
            esac
            ;;
        args)
            _arguments \
                '--help[Show help]'
            ;;
    esac
}

_affi "$@"
