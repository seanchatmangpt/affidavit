#compdef affi
#
# affi.zsh — zsh completion for the `affi` CLI (affidavit Provenance Layer).
#
# Generated from src/registry.rs — 69 verbs across 10 groups.
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
                'quality[Quality monitoring]' \
                'guide[Help and discovery]'
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
                        'root_cause[Trace root cause of failure]' \
                        'audit[Full integrity scan]' \
                        'query[Filter and search events]' \
                        'model[Extract type schema]' \
                        'conformance[Check against profile rules]' \
                        'coverage_analysis[Event coverage completeness]' \
                        'tech_debt[Technical debt analysis]' \
                        'security_debt[Security vulnerability analysis]' \
                        'emit_batch[Bulk-emit events from JSON array file]' \
                        'emit_from_cicd[Emit from CI/CD pipeline]' \
                        'emit_from_cloud[Emit from cloud events]' \
                        'emit_from_github[Emit from GitHub webhook]' \
                        'emit_from_gitlab[Emit from GitLab webhook]' \
                        'emit_from_monitoring[Emit from monitoring alerts]' \
                        'emit_from_sbom[Emit from SBOM components]' \
                        'emit_from_security[Emit from security scan results]' \
                        'verify_compliance[Verify against compliance framework]' \
                        'verify_sla[Verify against SLA targets]' \
                        'verify_family[Verify multiple receipts for consistency]' \
                        'policy_enforce[Enforce custom policy rules]' \
                        'license_compliance[License compliance check]' \
                        'gdpr_proof[GDPR evidence generation]' \
                        'hipaa[HIPAA compliance check]' \
                        'pci_dss[PCI-DSS compliance check]' \
                        'sign[Sign receipt with a key]' \
                        'notarize[Notarize receipt with timestamp]' \
                        'attest[Attest receipt with identity]' \
                        'assemble_with_signature[Assemble and sign in one step]' \
                        'assemble_and_notarize[Assemble and notarize in one step]' \
                        'sbom_attest[Attest SBOM provenance]' \
                        'sbom_scan[Scan SBOM for vulnerabilities]' \
                        'sbom_blast_radius[SBOM dependency blast radius]' \
                        'sbom_compliance[SBOM compliance check]' \
                        'sbom_ntia[NTIA minimum elements check]' \
                        'anomaly_detect[Detect anomalous events]' \
                        'predict[Predict future events]' \
                        'trend_analysis[Analyze event trends over time]' \
                        'variance[Variance analysis across receipts]' \
                        'find_blast_radius[Find dependency blast radius]' \
                        'explain_incident[Explain incident from receipt chain]' \
                        'causality_chain[Build causality chain]' \
                        'bus_factor[Bus factor analysis]' \
                        'dora_metrics[DORA metrics from receipt chain]' \
                        'team_velocity[Team velocity from events]' \
                        'portfolio_health[Portfolio health across receipts]' \
                        'orphaned_code[Find orphaned code events]' \
                        'dependency_matrix[Build dependency matrix]' \
                        'catalog[List and search receipt fixtures]' \
                        'search[Fuzzy verb search]' \
                        'profile[Show profile schema]' \
                        'install_git_hook[Install git hook for auto-verify]' \
                        'monitor[Continuous quality monitoring]' \
                        'visualize[Export receipt graph]' \
                        'test[Verb dispatch smoke test]' \
                        'receipt_throughput[Throughput benchmark]'
                    ;;
                affi)
                    _values 'verb' \
                        'doctor[Environment and receipt-store health checks]'
                    ;;
                quality)
                    _values 'verb' \
                        'monitor[Continuous quality monitoring]'
                    ;;
                guide)
                    _values 'verb' \
                        'search[Fuzzy verb keyword search]' \
                        'tutorial[Interactive tutorial]' \
                        'examples[Show usage examples]' \
                        'man[Show man page for a verb]'
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
