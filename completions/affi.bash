# affi.bash — bash completion for the `affi` CLI (affidavit Provenance Layer).
#
# Generated from src/registry.rs — 79 verbs across 11 groups.
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
    local receipt_verbs="anomaly-detect assemble assemble-and-notarize assemble-with-signature attest audit bus-factor catalog causality-chain conformance coverage-analysis dependency-matrix diagnose diff dora-metrics emit emit-batch emit-from-cicd emit-from-cloud emit-from-github emit-from-gitlab emit-from-monitoring emit-from-sbom emit-from-security explain-incident find-blast-radius fix gdpr-proof graph hipaa inspect install-git-hook license-compliance model monitor notarize orphaned-code pci-dss policy-enforce portfolio-health predict profile query receipt-throughput replay root-cause sbom-attest sbom-blast-radius sbom-compliance sbom-ntia sbom-scan search security-debt show sign soc2-audit stats team-velocity tech-debt test timeline trend-analysis variance verify verify-compliance verify-family verify-sla visualize why"

    local affi_verbs="doctor"
    local guide_verbs="search"

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
                    verify|show|inspect|diagnose|why|diff|graph|replay|timeline|audit|stats|model|conformance|sign|notarize|attest|assemble-with-signature|assemble-and-notarize|fix)
                        if [[ "${cur}" == -* ]]; then
                            COMPREPLY=( $(compgen -W "--format --json --help" -- "${cur}") )
                        else
                            COMPREPLY=( $(compgen -f -X '!*.json' -- "${cur}") )
                        fi
                        return 0
                        ;;
                    emit|emit-batch|emit-from-github|emit-from-gitlab|emit-from-cicd|emit-from-cloud|emit-from-monitoring|emit-from-sbom|emit-from-security)
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
