#compdef affi
#
# affi.zsh — zsh completion for the `affi` CLI (affidavit Provenance Layer).
#
# Generated from src/registry.rs — 79 verbs across 11 groups.
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
                'receipt[Receipt chain operations (69 verbs)]' \
                'affi[Tool-level operations]' \
                'guide[Help and discovery]' \
                'standing[Certify receipt-bound ecosystem standing]' \
                'ecosystem[Federate sealed member standing receipts]' \
                'errc[Certify formal ERRC transformation receipts]'
            ;;
        verb)
            case "${words[2]}" in
                receipt)
                    _values 'verb' \
                        'emit[Record an operation-event]' \
                        'assemble[Finalize working receipt into immutable file]' \
                        'verify[Run 7-stage certify pipeline]' \
                        'show[Human-readable receipt dump]' \
                        'inspect[Detailed inspection of receipt internals]' \
                        'stats[Chain metrics and histograms]' \
                        'why[Explain rejection in plain language]' \
                        'fix[Quarantine or finalize a receipt]' \
                        'diagnose[LSP-shaped diagnostics for failures]' \
                        'diff[Structural diff between two receipts]' \
                        'graph[DAG visualization (dot/mermaid/json)]' \
                        'replay[Re-execute chain from events]' \
                        'timeline[Chronological event sequence]' \
                        'root-cause[Trace root cause of failure]' \
                        'audit[Full integrity scan]' \
                        'query[Filter and search events]' \
                        'model[Extract type schema]' \
                        'conformance[Check against profile rules]' \
                        'coverage-analysis[Event coverage completeness]' \
                        'tech-debt[Technical debt analysis]' \
                        'security-debt[Security vulnerability analysis]' \
                        'emit-batch[Bulk-emit events from JSON array file]' \
                        'emit-from-cicd[Emit from CI/CD pipeline]' \
                        'emit-from-cloud[Emit from cloud events]' \
                        'emit-from-github[Emit from GitHub webhook]' \
                        'emit-from-gitlab[Emit from GitLab webhook]' \
                        'emit-from-monitoring[Emit from monitoring alerts]' \
                        'emit-from-sbom[Emit from SBOM components]' \
                        'emit-from-security[Emit from security scan results]' \
                        'verify-compliance[Verify against compliance framework]' \
                        'verify-sla[Verify against SLA targets]' \
                        'verify-family[Verify multiple receipts for consistency]' \
                        'policy-enforce[Enforce custom policy rules]' \
                        'license-compliance[License compliance check]' \
                        'gdpr-proof[GDPR evidence generation]' \
                        'hipaa[HIPAA compliance check]' \
                        'pci-dss[PCI-DSS compliance check]' \
                        'sign[Sign receipt with a key]' \
                        'notarize[Notarize receipt with timestamp]' \
                        'attest[Attest receipt with identity]' \
                        'assemble-with-signature[Assemble and sign in one step]' \
                        'assemble-and-notarize[Assemble and notarize in one step]' \
                        'sbom-attest[Attest SBOM provenance]' \
                        'sbom-scan[Scan SBOM for vulnerabilities]' \
                        'sbom-blast-radius[SBOM dependency blast radius]' \
                        'sbom-compliance[SBOM compliance check]' \
                        'sbom-ntia[NTIA minimum elements check]' \
                        'anomaly-detect[Detect anomalous events]' \
                        'predict[Predict future events]' \
                        'trend-analysis[Analyze event trends over time]' \
                        'variance[Variance analysis across receipts]' \
                        'find-blast-radius[Find dependency blast radius]' \
                        'explain-incident[Explain incident from receipt chain]' \
                        'causality-chain[Build causality chain]' \
                        'bus-factor[Bus factor analysis]' \
                        'dora-metrics[DORA metrics from receipt chain]' \
                        'team-velocity[Team velocity from events]' \
                        'portfolio-health[Portfolio health across receipts]' \
                        'orphaned-code[Find orphaned code events]' \
                        'dependency-matrix[Build dependency matrix]' \
                        'catalog[List and search receipt fixtures]' \
                        'search[Fuzzy verb search]' \
                        'profile[Show profile schema]' \
                        'install-git-hook[Install git hook for auto-verify]' \
                        'monitor[Continuous quality monitoring]' \
                        'visualize[Export receipt graph]' \
                        'test[Verb dispatch smoke test]' \
                        'receipt-throughput[Throughput benchmark]'
                    ;;
                affi)
                    _values 'verb' \
                        'doctor[Environment and receipt-store health checks]'
                    ;;
                standing)
                    _values 'verb' \
                        'certify[Seal a standing claim over an admitted receipt]' \
                        'verify[Re-run the standing law over a sealed receipt]'
                    ;;
                ecosystem)
                    _values 'verb' \
                        'certify[Federate sealed member standing receipts]' \
                        'verify[Re-run the federation law over a sealed receipt]'
                    ;;
                errc)
                    _values 'verb' \
                        'certify[Seal a declared ERRC transformation]' \
                        'verify[Re-run the ERRC laws over a sealed receipt]' \
                        'assure[Seal a one-witness-per-claim assurance ledger]' \
                        'verify-assurance[Re-run the claim-assurance law]'
                    ;;
                guide)
                    _values 'verb' \
                        'search[Search the verb registry by keyword]'
                    ;;
            esac
            ;;
        args)
            local noun="${words[2]}"
            local verb="${words[3]}"
            case "${noun}-${verb}" in
                receipt-emit)
                    _arguments \
                        '--type[Event type]:type:' \
                        '--object[Object ref (id\:type[\:qualifier])]:object:' \
                        '--payload[Payload file or - for stdin]:file:_files' \
                        '--working-dir[Working directory]:dir:_files -/' \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]'
                    ;;
                receipt-assemble)
                    _arguments \
                        '--out[Output path]:file:_files' \
                        '--working-dir[Working directory]:dir:_files -/' \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]'
                    ;;
                receipt-verify|receipt-show|receipt-inspect|receipt-stats|receipt-why|receipt-diagnose|receipt-graph|receipt-replay|receipt-timeline|receipt-audit|receipt-model|receipt-conformance)
                    _arguments \
                        '1:receipt:_files -g "*.json"' \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]'
                    ;;
                receipt-fix)
                    _arguments \
                        '1:receipt:_files -g "*.json"' \
                        '--action[Repair action]:action:(quarantine finalize auto)' \
                        '--dry-run[Preview without modifying files]' \
                        '--format[Output format]:format:(human json yaml)'
                    ;;
                receipt-verify_family|receipt-verify_compliance|receipt-verify_sla|receipt-query)
                    _arguments \
                        '1:path:_files' \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]'
                    ;;
                receipt-monitor)
                    _arguments \
                        '--watch[Path to watch]:dir:_files -/' \
                        '--metrics[Metrics to monitor]:metrics:' \
                        '--rules[WE rules to check]:rules:' \
                        '--interval[Poll interval seconds]:interval:' \
                        '--output[Output channels]:output:' \
                        '--format[Output format]:format:(human json yaml)'
                    ;;
                affi-doctor)
                    _arguments \
                        '--receipts[Receipt store path]:path:_files -/' \
                        '--fix[Apply safe automatic remediations]' \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]'
                    ;;
                *)
                    _arguments \
                        '--format[Output format]:format:(human json yaml)' \
                        '--json[Shorthand for --format json]' \
                        '--help[Show help]'
                    ;;
            esac
            ;;
    esac
}

_affi "$@"
