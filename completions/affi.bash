# affi.bash — bash completion for the `affi` CLI (affidavit Provenance Layer).
#
# Generated from src/registry.rs — 77 verbs across 11 groups.
# Covers all verbs from the compile-time static registry.
#
# Install (pick one):
#   source completions/affi.bash                          # current shell
#   cp completions/affi.bash ~/.local/share/bash-completion/completions/affi
#   sudo cp completions/affi.bash /usr/share/bash-completion/completions/affi

_affi() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion || return
    else
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
    fi

    # Nouns (top-level subcommands)
    local nouns="receipt affi guide standing ecosystem errc"

    # All receipt verbs (from registry.rs — Core, Diagnostics, Analysis,
    # Ingestion, Compliance, Attestation, SBOM, Insights, Engineering, Tooling)
    local receipt_verbs="emit assemble verify show inspect stats why fix diagnose diff graph replay timeline root_cause audit query model conformance coverage_analysis tech_debt security_debt emit_batch emit_from_cicd emit_from_cloud emit_from_github emit_from_gitlab emit_from_monitoring emit_from_sbom emit_from_security verify_compliance verify_sla verify_family policy_enforce license_compliance gdpr_proof hipaa pci_dss sign notarize attest assemble_with_signature assemble_and_notarize sbom_attest sbom_scan sbom_blast_radius sbom_compliance sbom_ntia anomaly_detect predict trend_analysis variance find_blast_radius explain_incident causality_chain bus_factor dora_metrics team_velocity portfolio_health orphaned_code dependency_matrix catalog search profile install_git_hook monitor visualize test receipt_throughput"

    local affi_verbs="doctor"
    local guide_verbs="search tutorial examples man"

    # Federation courts (v26.9.6): the evidence kernel's CLI surface.
    local standing_verbs="certify verify"
    local ecosystem_verbs="certify verify"
    local errc_verbs="certify verify assure verify-assurance"

    case "${COMP_CWORD}" in
        1)
            COMPREPLY=( $(compgen -W "${nouns} --help --version" -- "${cur}") )
            return 0
            ;;
        2)
            case "${prev}" in
                receipt)
                    COMPREPLY=( $(compgen -W "${receipt_verbs}" -- "${cur}") )
                    return 0
                    ;;
                affi)
                    COMPREPLY=( $(compgen -W "${affi_verbs}" -- "${cur}") )
                    return 0
                    ;;
                standing)
                    COMPREPLY=( $(compgen -W "${standing_verbs}" -- "${cur}") )
                    return 0
                    ;;
                ecosystem)
                    COMPREPLY=( $(compgen -W "${ecosystem_verbs}" -- "${cur}") )
                    return 0
                    ;;
                errc)
                    COMPREPLY=( $(compgen -W "${errc_verbs}" -- "${cur}") )
                    return 0
                    ;;
                guide)
                    COMPREPLY=( $(compgen -W "${guide_verbs}" -- "${cur}") )
                    return 0
                    ;;
            esac
            ;;
        *)
            local noun="${COMP_WORDS[1]}"
            local verb="${COMP_WORDS[2]}"
            if [[ "${noun}" == "receipt" ]]; then
                case "${verb}" in
                    verify|show|inspect|diagnose|why|diff|graph|replay|timeline|audit|stats|model|conformance|sign|notarize|attest|assemble_with_signature|assemble_and_notarize|fix)
                        if [[ "${cur}" == -* ]]; then
                            COMPREPLY=( $(compgen -W "--format --json --help" -- "${cur}") )
                        else
                            COMPREPLY=( $(compgen -f -X '!*.json' -- "${cur}") )
                        fi
                        return 0
                        ;;
                    emit|emit_batch|emit_from_github|emit_from_gitlab|emit_from_cicd|emit_from_cloud|emit_from_monitoring|emit_from_sbom|emit_from_security)
                        COMPREPLY=( $(compgen -W "--type --object --payload --working-dir --format --json --help" -- "${cur}") )
                        return 0
                        ;;
                    assemble)
                        COMPREPLY=( $(compgen -W "--out --working-dir --format --json --help" -- "${cur}") )
                        return 0
                        ;;
                    monitor)
                        COMPREPLY=( $(compgen -W "--watch --metrics --rules --interval --output --format --json --help" -- "${cur}") )
                        return 0
                        ;;
                    *)
                        if [[ "${cur}" == -* ]]; then
                            COMPREPLY=( $(compgen -W "--format --json --help" -- "${cur}") )
                        else
                            COMPREPLY=( $(compgen -f -- "${cur}") )
                        fi
                        return 0
                        ;;
                esac
            elif [[ "${noun}" == "affi" && "${verb}" == "doctor" ]]; then
                COMPREPLY=( $(compgen -W "--receipts --fix --format --json --help" -- "${cur}") )
                return 0
            elif [[ "${noun}" == "standing" || "${noun}" == "ecosystem" || "${noun}" == "errc" ]]; then
                case "${verb}" in
                    certify)
                        COMPREPLY=( $(compgen -W "--receipt --observation --scope --out --format --help" -- "${cur}") )
                        return 0
                        ;;
                    assure)
                        COMPREPLY=( $(compgen -W "--parent --witnesses --out --format --help" -- "${cur}") )
                        return 0
                        ;;
                    verify-assurance)
                        COMPREPLY=( $(compgen -W "--receipt --parent --format --help" -- "${cur}") )
                        return 0
                        ;;
                    verify)
                        COMPREPLY=( $(compgen -W "--receipt --format --help" -- "${cur}") )
                        return 0
                        ;;
                esac
            fi
            ;;
    esac

    COMPREPLY=( $(compgen -W "--help --version --format --json" -- "${cur}") )
}

complete -F _affi affi
