#compdef affi
#
# affi.zsh — zsh completion for the `affi` CLI (affidavit Provenance Layer).
#
# AUTHORED, NOT GENERATED. The `affi` binary depends on five sibling PATH
# crates not present in this repo, so it cannot be built here to auto-generate
# completions. This file is hand-written from the documented CLI surface
# (README.md / CONTRIBUTING.md): the `receipt` noun with verbs emit, assemble,
# verify, show, plus the global flags. Keep it in sync with that surface.
#
# Install (pick one):
#   * Per-user:   mkdir -p ~/.zsh/completions
#                 cp completions/affi.zsh ~/.zsh/completions/_affi
#                 # ensure ~/.zshrc has, BEFORE `compinit`:
#                 #   fpath=(~/.zsh/completions $fpath)
#                 #   autoload -Uz compinit && compinit
#   * Quick test in the current shell:
#                 fpath=("$PWD/completions" $fpath); autoload -Uz compinit && compinit
#     (the file is found because of the leading `#compdef affi` tag)
#
# NOTE: when installing into an fpath directory, the file must be named `_affi`.

_affi() {
  local context state state_descr line
  typeset -A opt_args

  # Global flags available at every level.
  local -a global_flags
  global_flags=(
    '--version[print version and exit]'
    '--introspect[print JSON capability manifest]'
    '--format[output format]:format:(json)'
    '(-h --help)'{-h,--help}'[show help]'
  )

  _arguments -C \
    "${global_flags[@]}" \
    '1: :->noun' \
    '2: :->verb' \
    '*:: :->args' \
    && return 0

  case "$state" in
    noun)
      local -a nouns
      nouns=(
        'receipt:provenance receipt commands (emit, assemble, verify, show)'
      )
      _describe -t nouns 'noun' nouns
      ;;
    verb)
      if [[ "${line[1]}" == "receipt" ]]; then
        local -a verbs
        verbs=(
          'emit:append one operation-event to .affi/working.json'
          'assemble:finalize the working receipt into an immutable file'
          'verify:run the 7-stage certify pipeline (ACCEPT/REJECT)'
          'show:human-readable dump of a receipt'
        )
        _describe -t verbs 'receipt verb' verbs
      fi
      ;;
    args)
      if [[ "${line[1]}" == "receipt" ]]; then
        case "${line[2]}" in
          emit)
            _arguments \
              "${global_flags[@]}" \
              '--type[event type]:event_type:' \
              '--object[object as id:type\[:qualifier\]]:object:' \
              '--payload[payload file or - for stdin]:payload:_files'
            ;;
          assemble)
            _arguments \
              "${global_flags[@]}" \
              '--out[output receipt path]:out:_files'
            ;;
          verify|show)
            _arguments \
              "${global_flags[@]}" \
              '*:receipt:_files -g "*.json"'
            ;;
          *)
            _arguments "${global_flags[@]}"
            ;;
        esac
      fi
      ;;
  esac
}

_affi "$@"
