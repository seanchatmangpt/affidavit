# affi.fish — fish completion for the `affi` CLI (affidavit Provenance Layer).
#
# Generated from src/registry.rs — 69 verbs across 10 groups.
# Covers all verbs from the compile-time static registry.
#
# Install: cp completions/affi.fish ~/.config/fish/completions/affi.fish

# --- helper predicates -------------------------------------------------------

function __affi_no_noun
    set -l toks (commandline -opc)
    for t in $toks[2..-1]
        switch $t
            case receipt affi quality guide
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
            case receipt affi quality guide
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

complete -c affi -f -n '__affi_no_noun' -a receipt -d 'Receipt operations (67 verbs)'
complete -c affi -f -n '__affi_no_noun' -a affi    -d 'Tool-level operations'
complete -c affi -f -n '__affi_no_noun' -a quality -d 'Quality monitoring'
complete -c affi -f -n '__affi_no_noun' -a guide   -d 'Help and discovery'

# --- receipt verbs -----------------------------------------------------------
# Core
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit                -d 'Record an operation-event'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble            -d 'Finalize working receipt into immutable file'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify              -d 'Run 7-stage certify pipeline'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a show                -d 'Human-readable receipt dump'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a inspect             -d 'Detailed inspection of receipt internals'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a stats               -d 'Chain metrics and histograms'
# Diagnostics
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a why                 -d 'Explain rejection in plain language'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a fix                 -d 'Quarantine or finalize a receipt'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a diagnose            -d 'LSP-shaped diagnostics for failures'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a diff                -d 'Structural diff between two receipts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a graph               -d 'DAG visualization (dot/mermaid/json)'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a replay              -d 'Re-execute chain from events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a timeline            -d 'Chronological event sequence'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a root_cause          -d 'Trace root cause of failure'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a audit               -d 'Full integrity scan'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a query               -d 'Filter and search events'
# Analysis
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a model               -d 'Extract type schema'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a conformance         -d 'Check against profile rules'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a coverage_analysis   -d 'Event coverage completeness'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a tech_debt           -d 'Technical debt analysis'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a security_debt       -d 'Security vulnerability analysis'
# Ingestion
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_batch          -d 'Bulk-emit events from JSON array file'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_cicd      -d 'Emit from CI/CD pipeline'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_cloud     -d 'Emit from cloud events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_github    -d 'Emit from GitHub webhook'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_gitlab    -d 'Emit from GitLab webhook'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_monitoring -d 'Emit from monitoring alerts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_sbom      -d 'Emit from SBOM components'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a emit_from_security  -d 'Emit from security scan results'
# Compliance
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify_compliance   -d 'Verify against compliance framework'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify_sla          -d 'Verify against SLA targets'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a verify_family       -d 'Verify multiple receipts for consistency'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a policy_enforce      -d 'Enforce custom policy rules'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a license_compliance  -d 'License compliance check'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a gdpr_proof          -d 'GDPR evidence generation'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a hipaa               -d 'HIPAA compliance check'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a pci_dss             -d 'PCI-DSS compliance check'
# Attestation
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sign                -d 'Sign receipt with a key'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a notarize            -d 'Notarize receipt with timestamp'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a attest              -d 'Attest receipt with identity'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble_with_signature -d 'Assemble and sign in one step'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a assemble_and_notarize   -d 'Assemble and notarize in one step'
# SBOM
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom_attest         -d 'Attest SBOM provenance'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom_scan           -d 'Scan SBOM for vulnerabilities'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom_blast_radius   -d 'SBOM dependency blast radius'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom_compliance     -d 'SBOM compliance check'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a sbom_ntia           -d 'NTIA minimum elements check'
# Insights
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a anomaly_detect      -d 'Detect anomalous events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a predict             -d 'Predict future events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a trend_analysis      -d 'Analyze event trends over time'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a variance            -d 'Variance analysis across receipts'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a find_blast_radius   -d 'Find dependency blast radius'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a explain_incident    -d 'Explain incident from receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a causality_chain     -d 'Build causality chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a bus_factor          -d 'Bus factor analysis'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a dora_metrics        -d 'DORA metrics from receipt chain'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a team_velocity       -d 'Team velocity from events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a portfolio_health    -d 'Portfolio health across receipts'
# Engineering
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a orphaned_code       -d 'Find orphaned code events'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a dependency_matrix   -d 'Build dependency matrix'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a catalog             -d 'List and search receipt fixtures'
# Tooling
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a search              -d 'Fuzzy verb search'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a profile             -d 'Show profile schema'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a install_git_hook    -d 'Install git hook for auto-verify'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a monitor             -d 'Continuous quality monitoring'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a visualize           -d 'Export receipt graph'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a test                -d 'Verb dispatch smoke test'
complete -c affi -f -n '__affi_using_noun receipt; and __affi_no_verb' -a receipt_throughput  -d 'Throughput benchmark'

# --- affi verbs --------------------------------------------------------------
complete -c affi -f -n '__affi_using_noun affi; and __affi_no_verb' -a doctor -d 'Environment and receipt-store health checks'

# --- Flags for common verbs --------------------------------------------------
complete -c affi -n '__affi_using_noun receipt' -l format -d 'Output format' -a 'human json yaml'
complete -c affi -n '__affi_using_noun receipt' -l json   -d 'Shorthand for --format json'
complete -c affi -n '__affi_using_noun affi'    -l json   -d 'Shorthand for --format json'
complete -c affi -s h -l help    -d 'Show help'
complete -c affi -s V -l version -d 'Show version'
