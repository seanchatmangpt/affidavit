# bash completion — source via `source completions/affi.bash`
#
# affi.bash — shell completion for the `affi` CLI (affidavit Provenance Layer).
#
# GENERATED FILE — DO NOT EDIT.
# Generator: python3 scripts/generate_completions.py
# Source: src/registry.rs — Registry verbs: 92, Registry nouns: 9
# Regenerate after every registry change; tests/completions_drift.rs enforces it.
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

    # Nouns (top-level subcommands), then verbs per noun — from src/registry.rs.
    local nouns="affi ecosystem envelope errc evidence guide keys receipt standing"

    local affi_verbs="doctor"
    local ecosystem_verbs="certify verify"
    local envelope_verbs="export list sign verify"
    local errc_verbs="assure certify verify verify-assurance"
    local evidence_verbs="crl-apply crl-publish heads journal"
    local guide_verbs="search"
    local keys_verbs="generate import list revoke rotate"
    local receipt_verbs="anomaly-detect assemble assemble-and-notarize assemble-with-signature attest audit bus-factor catalog causality-chain conformance coverage-analysis dependency-matrix diagnose diff dora-metrics emit emit-batch emit-from-cicd emit-from-cloud emit-from-github emit-from-gitlab emit-from-monitoring emit-from-sbom emit-from-security explain-incident find-blast-radius fix gdpr-proof graph hipaa inspect install-git-hook license-compliance model monitor notarize orphaned-code pci-dss policy-enforce portfolio-health predict profile query receipt-throughput replay root-cause sbom-attest sbom-blast-radius sbom-compliance sbom-ntia sbom-scan search security-debt show sign soc2-audit stats team-velocity tech-debt test timeline trend-analysis variance verify verify-compliance verify-family verify-sla visualize why"
    local standing_verbs="certify verify"

    case "${COMP_CWORD}" in
        1)
            COMPREPLY=( $(compgen -W "${nouns} --help --version" -- "${cur}") )
            return 0
            ;;
        2)
            local var="${prev}_verbs"
            local verbs="${!var}"
            if [[ -n "${verbs}" ]]; then
                COMPREPLY=( $(compgen -W "${verbs}" -- "${cur}") )
            fi
            return 0
            ;;
        *)
            if [[ "${cur}" == -* ]]; then
                COMPREPLY=( $(compgen -W "--help --version" -- "${cur}") )
            fi
            return 0
            ;;
    esac
}

complete -F _affi affi
