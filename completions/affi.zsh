#compdef affi
_affi() {
    local context state state_descr line
    typeset -A opt_args

    _arguments \
        '1: :->noun' \
        '2: :->verb' \
        '*: :->args'

    case $state in
        noun) _values 'noun' receipt ;;
        verb) _values 'verb' \
            emit:'append an operation-event' \
            assemble:'finalize into immutable receipt' \
            verify:'run 7-stage certify pipeline' \
            show:'display receipt without verdict' \
            inspect:'structural analysis' \
            model:'discover process model (wasm4pm)' \
            diagnose:'LSP-shaped diagnostics (lsp-max)' \
            conformance:'fitness + activity_coverage + simplicity' \
            replay:'step-by-step event trace' \
            graph:'directly-follows graph' \
            stats:'aggregate event/object/DFG counts' \
            mutate:'tamper-evidence demonstration' \
            bench:'inline performance check' ;;
        args) _files ;;
    esac
}
_affi
