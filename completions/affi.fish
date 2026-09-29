# fish completion — install via `cp completions/affi.fish ~/.config/fish/completions/affi.fish`
#
# affi.fish — shell completion for the `affi` CLI (affidavit Provenance Layer).
#
# GENERATED FILE — DO NOT EDIT.
# Generator: python3 scripts/generate_completions.py
# Source: src/registry.rs — Registry verbs: 86, Registry nouns: 8
# Regenerate after every registry change; tests/completions_drift.rs enforces it.

# --- helper predicates -------------------------------------------------------

function __affi_no_noun
    set -l toks (commandline -opc)
    for t in $toks[2..-1]
        switch $t
            case affi ecosystem envelope errc guide keys receipt standing
                return 1
        end
    end
    return 0
end

function __affi_using_noun
    set -l toks (commandline -opc)
    for t in $toks[2..-1]
        if test "$t" = "$argv[1]"
            return 0
        end
    end
    return 1
end

function __affi_no_verb
    set -l toks (commandline -opc)
    set -l found_noun 0
    for t in $toks[2..-1]
        switch $t
            case affi ecosystem envelope errc guide keys receipt standing
            set found_noun 1
            case '*'
            if test $found_noun -eq 1
                return 1
            end
        end
    end
    return 0
end

# --- Nouns -------------------------------------------------------------------

complete -c affi -f -n '__affi_no_noun' -a affi -d '1 verbs'
complete -c affi -f -n '__affi_no_noun' -a ecosystem -d '2 verbs'
complete -c affi -f -n '__affi_no_noun' -a envelope -d '2 verbs'
complete -c affi -f -n '__affi_no_noun' -a errc -d '4 verbs'
complete -c affi -f -n '__affi_no_noun' -a guide -d '1 verbs'
complete -c affi -f -n '__affi_no_noun' -a keys -d '5 verbs'
complete -c affi -f -n '__affi_no_noun' -a receipt -d '69 verbs'
complete -c affi -f -n '__affi_no_noun' -a standing -d '2 verbs'

# --- affi verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun affi; and __affi_no_verb' -a doctor -d 'Run environment and receipt-store health checks; optionally apply safe fixes'

# --- ecosystem verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun ecosystem; and __affi_no_verb' -a certify -d 'Federate sealed member standing receipts into one quorum-scored receipt'
complete -c affi -f -n '__affi_using_noun ecosystem; and __affi_no_verb' -a verify -d 'Re-run the federation law over a sealed ecosystem receipt'

# --- envelope verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun envelope; and __affi_no_verb' -a sign -d 'Seal a receipt under an ES256 key: CTP envelope + signature as a PQ-SEAL-v1 document'
complete -c affi -f -n '__affi_using_noun envelope; and __affi_no_verb' -a verify -d 'Adjudicate a PQ-SEAL-v1 sealed receipt against the registered keys and print the standing VERDICT (exit 0=VALID, 2=decided otherwise)'

# --- errc verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun errc; and __affi_no_verb' -a assure -d 'Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt'
complete -c affi -f -n '__affi_using_noun errc; and __affi_no_verb' -a certify -d 'Seal a declared ERRC transformation (affidavit/errc/v1)'
complete -c affi -f -n '__affi_using_noun errc; and __affi_no_verb' -a verify -d 'Re-run the ERRC directional and preservation laws over a sealed receipt'
complete -c affi -f -n '__affi_using_noun errc; and __affi_no_verb' -a verify-assurance -d 'Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt'

# --- guide verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun guide; and __affi_no_verb' -a search -d 'Search the verb registry by keyword to discover relevant commands'

# --- keys verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun keys; and __affi_no_verb' -a generate -d 'Generate a real ES256 signing key and append its public record to the key store (the secret never touches disk)'
complete -c affi -f -n '__affi_using_noun keys; and __affi_no_verb' -a import -d 'Import an externally-held public key (hex) into the key store: fingerprint it and register it under a custodian (origin Imported)'
complete -c affi -f -n '__affi_using_noun keys; and __affi_no_verb' -a list -d 'List registered key records: kid, algorithm, fingerprint, custodian'
complete -c affi -f -n '__affi_using_noun keys; and __affi_no_verb' -a revoke -d 'Revoke a registered key: append a tamper-evident revocation entry to the checksummed sidecar beside the key store'
complete -c affi -f -n '__affi_using_noun keys; and __affi_no_verb' -a rotate -d 'Rotate a registered ES256 key to a freshly generated ES256 successor: the successor signs the rotation record and its public record joins the store'

# --- receipt verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a anomaly-detect -d 'Detect anomalous events in the receipt chain using statistical outlier analysis'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble -d 'Finalize the working receipt into an immutable sealed file'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble-and-notarize -d 'Assemble and attach a local trust-plane notarization in one step (no external TSA)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble-with-signature -d 'Assemble and immediately sign the receipt in a single atomic operation'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a attest -d 'Create a signed SLSA attestation document from a sealed receipt'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a audit -d 'Run a full audit pass over a receipt: chain integrity, commitments, and event completeness'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a bus-factor -d 'Compute the bus factor for each object in the chain based on contributor events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a catalog -d 'Index a directory of receipts into a local catalog for fast search and lookup'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a causality-chain -d 'Build a causality chain graph linking events by causal relationships'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a conformance -d 'Check a receipt against custom conformance rules or a named profile'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a coverage-analysis -d 'Measure what fraction of defined event types appear in the receipt'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a dependency-matrix -d 'Build an object-to-object dependency matrix from co-occurrence in receipt events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a diagnose -d 'Troubleshoot verification failures with suggested remediation steps'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a diff -d 'Compute a structural diff between two receipts showing added, removed, and changed events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a dora-metrics -d 'Extract DORA metrics (deployment frequency, lead time, MTTR, change failure rate) from receipts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit -d 'Record an operation-event into the working receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-batch -d 'Ingest multiple events from a JSONL or CSV file in a single batch operation'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-cicd -d 'Ingest events from a CI/CD pipeline run (GitHub Actions, GitLab CI, Jenkins, etc.)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-cloud -d 'Ingest events from cloud provider audit logs (AWS CloudTrail, GCP Audit, Azure Monitor)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-github -d 'Ingest events from a GitHub repository (commits, PRs, releases, workflow runs)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-gitlab -d 'Ingest events from a GitLab project (pipelines, MRs, tags, deployments)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-monitoring -d 'Ingest events from monitoring/observability systems (Datadog, Prometheus, PagerDuty)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-sbom -d 'Ingest SBOM document events (CycloneDX or SPDX) into the receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit-from-security -d 'Ingest security scan events (Snyk, Trivy, Dependabot, SARIF) into the receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a explain-incident -d 'Generate a structured incident explanation from events surrounding a failure'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a find-blast-radius -d 'Find all receipt events transitively affected by a given failing event'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a fix -d 'Apply a safe structural repair: quarantine a tampered receipt or finalize a working one. Use --dry-run to preview.'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a gdpr-proof -d 'Generate GDPR data-processing evidence from receipt events for auditor review'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a graph -d 'DAG visualization of event dependencies and object references (dot, mermaid, json)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a hipaa -d 'Collect HIPAA safeguard evidence from receipt events for auditor review'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a inspect -d 'Detailed inspection of receipt internals (chain hash, commitments, continuity)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a install-git-hook -d 'Install a git hook that auto-emits receipt events on commit, push, or tag'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a license-compliance -d 'Check that dependency licenses in the receipt satisfy the project'\''s license policy'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a model -d 'Extract the type schema from a receipt (event types, object types, qualifiers)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a monitor -d 'Watch a working receipt file and stream events to stdout as they are appended'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a notarize -d 'Attach a local trust-plane notarization to a receipt (unsigned attestation request, or a real ES256 attestation with AFFI_NOTARY_KEY)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a orphaned-code -d 'Identify object references in the receipt that have no corresponding emit events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a pci-dss -d 'Collect PCI-DSS control evidence from receipt events for auditor review'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a policy-enforce -d 'Evaluate receipt events against a Rego or CEL policy file'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a portfolio-health -d 'Aggregate health signals across a portfolio of receipts into a dashboard-ready report'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a predict -d 'Forecast future chain state or failure probability based on historical receipt patterns'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a profile -d 'Profile the performance of verify and assemble operations on a receipt'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a query -d 'Query receipt events using a filter expression (event_type, object, seq range)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a receipt-throughput -d 'Measure how many receipts per second the local store can verify under load'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a replay -d 'Re-execute chain from stored events, optionally applying custom handlers'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a root-cause -d 'Trace failure events back to their causal predecessors in the chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom-attest -d 'Attach a SLSA attestation to a previously generated SBOM document'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom-blast-radius -d 'Compute the blast radius of a vulnerable component across the SBOM dependency graph'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom-compliance -d 'Verify that the SBOM satisfies a license or security policy'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom-ntia -d 'Check an SBOM document against the NTIA minimum-element requirements'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom-scan -d 'Scan a project and emit an SBOM event capturing all detected components'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a search -d 'Search the local receipt catalog by event type, object, date range, or keyword'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a security-debt -d 'Surface security-relevant events with missing or weak commitments'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a show -d 'Human-readable dump of the receipt chain with event details'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sign -d 'Cryptographically sign a sealed receipt with an ES256 key (raw-hex key file; production custody via HSM/Secure Enclave)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a soc2-audit -d 'Collect SOC 2 trust-service-criteria evidence from receipts for auditor review'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a stats -d 'Chain metrics: event count, hash distribution, event-type histogram'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a team-velocity -d 'Measure team throughput and cycle time from emit/assemble event pairs'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a tech-debt -d 'Identify stale or low-quality events that signal accumulated technical debt'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a test -d 'Run the built-in receipt self-test suite to validate the local installation'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a timeline -d 'Render a temporal timeline of events ordered by sequence number'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a trend-analysis -d 'Compute rolling trends over event frequency, latency, and error rates across receipts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a variance -d 'Measure variance in event timing and payload size across a collection of receipts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify -d 'Run the 7-stage certify pipeline against a receipt (exit 0=ACCEPT, 2=REJECT)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify-compliance -d 'Check evidence presence for a compliance framework (evidence present/absent — not a legal determination)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify-family -d 'Verify a family of related receipts all satisfy shared constraints'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify-sla -d 'Verify that receipt events satisfy defined SLA thresholds and time windows'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a visualize -d 'Generate an interactive HTML or SVG visualization of the receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a why -d 'Explain in plain language why a receipt was rejected, with stage-by-stage remediation steps'

# --- standing verbs ------------------------------------------------------

complete -c affi -f -n '__affi_using_noun standing; and __affi_no_verb' -a certify -d 'Seal a standing claim over an admitted receipt (affidavit/standing/v2)'
complete -c affi -f -n '__affi_using_noun standing; and __affi_no_verb' -a verify -d 'Re-run the standing law over a sealed standing receipt'

# --- top-level flags ---------------------------------------------------------

complete -c affi -s h -l help    -d 'Show help'
complete -c affi -s V -l version -d 'Show version'
