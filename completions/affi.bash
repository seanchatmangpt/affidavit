# affi.bash — bash completion for the `affi` CLI (affidavit Provenance Layer).
#
# AUTHORED, NOT GENERATED. The `affi` binary depends on five sibling PATH
# crates that are not in this repo, so it cannot be built here to auto-generate
# completions. This file is hand-written from the documented CLI surface
# (README.md / CONTRIBUTING.md): the `receipt` noun with verbs emit, assemble,
# verify, show, plus the global flags. Keep it in sync with that surface.
#
# Install (pick one):
#   * Quick, current shell:   source completions/affi.bash
#   * Per-user, persistent:   cp completions/affi.bash ~/.local/share/bash-completion/completions/affi
#   * System-wide:            sudo cp completions/affi.bash /usr/share/bash-completion/completions/affi
# Then start a new shell (or re-source). Requires the bash-completion package.

_affi() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion || return
    else
        # Minimal fallback when bash-completion's helper is unavailable.
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    fi

    local global_flags='--version --introspect --format --help -h'
    local nouns='receipt'
    local receipt_verbs='emit assemble verify show'

    # --- value completion for flags that take an argument ------------------
    case "$prev" in
        --format)
            COMPREPLY=( $(compgen -W 'json' -- "$cur") )
            return 0
            ;;
        --type|--object)
            # Free-form values; no fixed completion list.
            COMPREPLY=()
            return 0
            ;;
        --payload|--out)
            # File paths (or '-' for stdin/stdout where supported).
            COMPREPLY=( $(compgen -f -- "$cur") )
            return 0
            ;;
    esac

    # --- locate the noun and verb already on the line ---------------------
    local noun="" verb="" i
    for (( i=1; i < ${#words[@]}; i++ )); do
        local w="${words[i]}"
        [[ "$w" == -* ]] && continue
        if [[ -z "$noun" ]]; then
            noun="$w"
        elif [[ -z "$verb" ]]; then
            verb="$w"
            break
        fi
    done

    # --- noun position -----------------------------------------------------
    if [[ -z "$noun" ]]; then
        if [[ "$cur" == -* ]]; then
            COMPREPLY=( $(compgen -W "$global_flags" -- "$cur") )
        else
            COMPREPLY=( $(compgen -W "$nouns $global_flags" -- "$cur") )
        fi
        return 0
    fi

    # --- verb position (only `receipt` is documented) ---------------------
    if [[ "$noun" == "receipt" && -z "$verb" ]]; then
        if [[ "$cur" == -* ]]; then
            COMPREPLY=( $(compgen -W "$global_flags" -- "$cur") )
        else
            COMPREPLY=( $(compgen -W "$receipt_verbs $global_flags" -- "$cur") )
        fi
        return 0
    fi

    # --- per-verb flags / positionals -------------------------------------
    if [[ "$noun" == "receipt" ]]; then
        case "$verb" in
            emit)
                COMPREPLY=( $(compgen -W "--type --object --payload --format --help" -- "$cur") )
                return 0
                ;;
            assemble)
                COMPREPLY=( $(compgen -W "--out --format --help" -- "$cur") )
                return 0
                ;;
            verify|show)
                # Positional <receipt.json>; offer *.json plus the verb flags.
                if [[ "$cur" == -* ]]; then
                    COMPREPLY=( $(compgen -W "--format --help" -- "$cur") )
                else
                    COMPREPLY=( $(compgen -f -X '!*.json' -- "$cur") )
                fi
                return 0
                ;;
        esac
    fi

    COMPREPLY=( $(compgen -W "$global_flags" -- "$cur") )
    return 0
}

complete -F _affi affi
