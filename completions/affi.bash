# bash completion for affi
_affi() {
    local cur prev words cword
    _init_completion || return

    case $cword in
        1) COMPREPLY=($(compgen -W "receipt" -- "$cur")) ;;
        2) COMPREPLY=($(compgen -W "emit assemble verify show inspect model diagnose conformance replay graph stats mutate bench --introspect --help" -- "$cur")) ;;
        3)
            case $prev in
                emit)     COMPREPLY=($(compgen -W "--type --object --payload --help" -- "$cur")) ;;
                assemble) COMPREPLY=($(compgen -W "--out --help" -- "$cur")) ;;
                verify|show|inspect|model|diagnose|conformance|replay|graph|stats|mutate|bench)
                          COMPREPLY=($(compgen -f -- "$cur")) ;;
            esac ;;
    esac
}
complete -F _affi affi
