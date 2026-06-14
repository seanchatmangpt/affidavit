# affi.fish — fish completion for the `affi` CLI (affidavit Provenance Layer).
#
# AUTHORED, NOT GENERATED. The `affi` binary depends on five sibling PATH
# crates not present in this repo, so it cannot be built here to auto-generate
# completions. This file is hand-written from the documented CLI surface
# (README.md / CONTRIBUTING.md): the `receipt` noun with verbs emit, assemble,
# verify, show, plus the global flags. Keep it in sync with that surface.
#
# Install (pick one):
#   * Quick, current shell:   source completions/affi.fish
#   * Per-user, persistent:   cp completions/affi.fish ~/.config/fish/completions/affi.fish
# fish auto-loads completions from ~/.config/fish/completions/ in new shells.

# --- helper predicates -----------------------------------------------------

# True when no noun (receipt) has been typed yet.
function __affi_no_noun
    set -l toks (commandline -opc)
    for t in $toks[2..-1]
        switch $t
            case '-*'
                continue
            case '*'
                return 1
        end
    end
    return 0
end

# True when `receipt` is present but no verb has been typed yet.
function __affi_receipt_needs_verb
    set -l toks (commandline -opc)
    set -l seen_receipt 0
    for t in $toks[2..-1]
        switch $t
            case '-*'
                continue
            case receipt
                if test $seen_receipt -eq 1
                    return 1
                end
                set seen_receipt 1
            case '*'
                # some non-flag token after receipt => verb already present
                if test $seen_receipt -eq 1
                    return 1
                end
                return 1
        end
    end
    test $seen_receipt -eq 1
end

# True when the current command is `receipt <verb>`.
function __affi_receipt_verb_is
    set -l toks (commandline -opc)
    set -l want $argv[1]
    set -l seen_receipt 0
    for t in $toks[2..-1]
        switch $t
            case '-*'
                continue
            case receipt
                set seen_receipt 1
            case '*'
                if test $seen_receipt -eq 1
                    test "$t" = "$want"; and return 0
                    return 1
                end
        end
    end
    return 1
end

# Disable file completion by default; we opt back in where it makes sense.
complete -c affi -f

# --- global flags ----------------------------------------------------------
complete -c affi -l version    -d 'Print version and exit'
complete -c affi -l introspect -d 'Print JSON capability manifest'
complete -c affi -l format     -d 'Output format' -x -a 'json'
complete -c affi -s h -l help  -d 'Show help'

# --- noun ------------------------------------------------------------------
complete -c affi -n '__affi_no_noun' -a 'receipt' -d 'Provenance receipt commands'

# --- receipt verbs ---------------------------------------------------------
complete -c affi -n '__affi_receipt_needs_verb' -a 'emit'     -d 'Append one operation-event to .affi/working.json'
complete -c affi -n '__affi_receipt_needs_verb' -a 'assemble' -d 'Finalize the working receipt into an immutable file'
complete -c affi -n '__affi_receipt_needs_verb' -a 'verify'   -d 'Run the 7-stage certify pipeline (ACCEPT/REJECT)'
complete -c affi -n '__affi_receipt_needs_verb' -a 'show'     -d 'Human-readable dump of a receipt'

# --- receipt emit flags ----------------------------------------------------
complete -c affi -n '__affi_receipt_verb_is emit' -l type    -d 'Event type' -x
complete -c affi -n '__affi_receipt_verb_is emit' -l object  -d 'Object as id:type[:qualifier]' -x
complete -c affi -n '__affi_receipt_verb_is emit' -l payload -d 'Payload file or - for stdin' -r

# --- receipt assemble flags ------------------------------------------------
complete -c affi -n '__affi_receipt_verb_is assemble' -l out -d 'Output receipt path' -r

# --- receipt verify / show: positional *.json ------------------------------
complete -c affi -n '__affi_receipt_verb_is verify' -k -a "(__fish_complete_suffix .json)" -d 'Receipt JSON'
complete -c affi -n '__affi_receipt_verb_is show'   -k -a "(__fish_complete_suffix .json)" -d 'Receipt JSON'
