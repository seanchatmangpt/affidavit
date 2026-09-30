#!/usr/bin/env python3
"""Generate shell completions (bash/zsh/fish/PowerShell/Nushell) for the `affi` CLI.

Single source of truth: the static REGISTRY in src/registry.rs. This script
parses the `VerbEntry::new(...)` rows out of it (same text format the
registry's own drift tests parse) and emits completions/affi.bash,
completions/affi.zsh, completions/affi.fish, completions/affi.ps1 and
completions/affi.nu covering EVERY registered (noun, verb) pair:

  - noun-level completion:   `affi <TAB>` offers every registry noun
  - verb-level per noun:     `affi <noun> <TAB>` offers that noun's verbs
  - descriptions:            the registry summary, in zsh and fish (bash
                             `compgen -W` carries no description channel)

Output is deterministic: same registry -> byte-identical files. Everything is
sorted; no timestamps. The emitted files carry a DO-NOT-EDIT header naming the
generator command and the registry verb count; tests/completions_drift.rs
fails when a verb is added to the registry without re-running:

    python3 scripts/generate_completions.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY_PATH = REPO_ROOT / "src" / "registry.rs"
COMPLETIONS_DIR = REPO_ROOT / "completions"

# One `VerbEntry::new("verb", "noun", VerbGroup::Group, "summary", &[...])`
# call, spread over multiple lines. DOTALL spans the newlines; the string
# bodies accept Rust escapes (\" and \\).
ENTRY_RE = re.compile(
    r'VerbEntry::new\(\s*'
    r'"((?:[^"\\]|\\.)*)"\s*,\s*'
    r'"((?:[^"\\]|\\.)*)"\s*,\s*'
    r'VerbGroup::(\w+)\s*,\s*'
    r'"((?:[^"\\]|\\.)*)"\s*,',
    re.DOTALL,
)

_ESCAPES = {"\\": "\\", '"': '"', "n": "\n", "t": "\t", "r": "\r"}


def rust_unescape(literal: str) -> str:
    """Decode the escape sequences Rust string literals support (subset)."""
    out: list[str] = []
    i = 0
    while i < len(literal):
        ch = literal[i]
        if ch == "\\" and i + 1 < len(literal):
            nxt = literal[i + 1]
            if nxt in _ESCAPES:
                out.append(_ESCAPES[nxt])
                i += 2
                continue
        out.append(ch)
        i += 1
    return "".join(out)


def parse_registry() -> list[dict[str, str]]:
    """Return one row per REGISTRY entry: verb, noun, group, summary."""
    src = REGISTRY_PATH.read_text(encoding="utf-8")
    rows = [
        {
            "verb": rust_unescape(verb),
            "noun": rust_unescape(noun),
            "group": group,
            "summary": rust_unescape(summary),
        }
        for verb, noun, group, summary in ENTRY_RE.findall(src)
    ]
    if not rows:
        sys.exit(
            f"REFUSED_NO_REGISTRY_ENTRIES: parsed 0 VerbEntry rows from "
            f"{REGISTRY_PATH} — the registry text format changed; fix ENTRY_RE."
        )
    pairs = [(r["verb"], r["noun"]) for r in rows]
    if len(pairs) != len(set(pairs)):
        sys.exit("REFUSED_DUPLICATE_VERB_NOUN: REGISTRY contains a duplicate (verb, noun) pair.")
    return rows


def by_noun(rows: list[dict[str, str]]) -> dict[str, list[dict[str, str]]]:
    """Group rows by noun; nouns and per-noun verbs sorted alphabetically."""
    grouped: dict[str, list[dict[str, str]]] = {}
    for row in rows:
        grouped.setdefault(row["noun"], []).append(row)
    return {noun: sorted(grouped[noun], key=lambda r: r["verb"]) for noun in sorted(grouped)}


def header(shell_filename: str, first_line: str, verb_count: int, noun_count: int) -> str:
    lines = [
        first_line,
        "#",
        f"# {shell_filename} — shell completion for the `affi` CLI (affidavit Provenance Layer).",
        "#",
        "# GENERATED FILE — DO NOT EDIT.",
        "# Generator: python3 scripts/generate_completions.py",
        f"# Source: src/registry.rs — Registry verbs: {verb_count}, Registry nouns: {noun_count}",
        "# Regenerate after every registry change; tests/completions_drift.rs enforces it.",
    ]
    return "\n".join(lines)


def shell_quote_single(text: str) -> str:
    """Escape for a single-quoted shell string (bash/zsh/fish)."""
    return text.replace("'", "'\\''")


def zsh_description(text: str) -> str:
    """Escape a summary for use inside a single-quoted zsh `spec[description]`.

    Apostrophes are escaped at the shell layer (`'\\''`); `[`/`]` would end the
    description early and `:` is a spec separator, so those three get a
    backslash that reaches the completion language inside the quotes (house
    style of the previous completions).
    """
    return (
        text.replace("'", "'\\''")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace(":", "\\:")
    )


def emit_bash(nouns: dict[str, list[dict[str, str]]]) -> str:
    vc, nc = sum(len(v) for v in nouns.values()), len(nouns)
    out = [
        header("affi.bash", "# bash completion — source via `source completions/affi.bash`", vc, nc),
        "#",
        "# Install (pick one):",
        "#   source completions/affi.bash                          # current shell",
        "#   cp completions/affi.bash ~/.local/share/bash-completion/completions/affi",
        "#   sudo cp completions/affi.bash /usr/share/bash-completion/completions/affi",
        "",
        "_affi() {",
        "    local cur prev words cword",
        "    if declare -F _init_completion >/dev/null 2>&1; then",
        "        _init_completion || return",
        "    else",
        "        COMPREPLY=()",
        '        cur="${COMP_WORDS[COMP_CWORD]}"',
        '        prev="${COMP_WORDS[COMP_CWORD-1]}"',
        "    fi",
        "",
        "    # Nouns (top-level subcommands), then verbs per noun — from src/registry.rs.",
        f'    local nouns="{shell_quote_single(" ".join(nouns))}"',
        "",
    ]
    for noun, rows in nouns.items():
        verbs = " ".join(row["verb"] for row in rows)
        out.append(f'    local {noun}_verbs="{verbs}"')
    out += [
        "",
        '    case "${COMP_CWORD}" in',
        "        1)",
        '            COMPREPLY=( $(compgen -W "${nouns} --help --version" -- "${cur}") )',
        "            return 0",
        "            ;;",
        "        2)",
        '            local var="${prev}_verbs"',
        '            local verbs="${!var}"',
        '            if [[ -n "${verbs}" ]]; then',
        '                COMPREPLY=( $(compgen -W "${verbs}" -- "${cur}") )',
        "            fi",
        "            return 0",
        "            ;;",
        "        *)",
        '            if [[ "${cur}" == -* ]]; then',
        '                COMPREPLY=( $(compgen -W "--help --version" -- "${cur}") )',
        "            fi",
        "            return 0",
        "            ;;",
        "    esac",
        "}",
        "",
        "complete -F _affi affi",
        "",
    ]
    return "\n".join(out)


def emit_zsh(nouns: dict[str, list[dict[str, str]]]) -> str:
    vc, nc = sum(len(v) for v in nouns.values()), len(nouns)
    out = [
        "#compdef affi",
        "",
        header("affi.zsh", "# zsh completion — install per `fpath` + `compinit` (see below)", vc, nc),
        "#",
        "# Install (per-user):",
        "#   mkdir -p ~/.zsh/completions",
        "#   cp completions/affi.zsh ~/.zsh/completions/_affi",
        "#   # In ~/.zshrc (before compinit):",
        "#   #   fpath=(~/.zsh/completions $fpath)",
        "#   #   autoload -Uz compinit && compinit",
        "",
        "_affi() {",
        "    local state",
        "",
        "    _arguments \\",
        "        '(-h --help)'{-h,--help}'[Show help]' \\",
        "        '(-V --version)'{-V,--version}'[Show version]' \\",
        "        '1: :->noun' \\",
        "        '2: :->verb' \\",
        "        '*: :->args' && return 0",
        "",
        '    case "${state}" in',
        "        noun)",
        "            _values 'noun' \\",
    ]
    noun_specs = [
        f"                '{shell_quote_single(noun)}[{zsh_description(f'{len(nouns[noun])} verbs')}]'"
        for noun in nouns
    ]
    out += [spec + " \\" for spec in noun_specs[:-1]] + [noun_specs[-1]]
    out += ["            ;;", "        verb)", '            case "${words[2]}" in']
    for noun, rows in nouns.items():
        out.append(f"                {noun})")
        out.append("                    _values 'verb' \\")
        specs = [
            f"'{shell_quote_single(row['verb'])}[{zsh_description(row['summary'])}]'"
            for row in rows
        ]
        out += [f"                        {spec} \\" for spec in specs[:-1]]
        out.append(f"                        {specs[-1]}")
        out.append("                    ;;")
    out += [
        "            esac",
        "            ;;",
        "        args)",
        "            _arguments \\",
        "                '--help[Show help]'",
        "            ;;",
        "    esac",
        "}",
        "",
        '_affi "$@"',
        "",
    ]
    return "\n".join(out)


def emit_fish(nouns: dict[str, list[dict[str, str]]]) -> str:
    vc, nc = sum(len(v) for v in nouns.values()), len(nouns)
    noun_switch = " ".join(nouns)
    out = [
        header("affi.fish", "# fish completion — install via `cp completions/affi.fish ~/.config/fish/completions/affi.fish`", vc, nc),
        "",
        "# --- helper predicates -------------------------------------------------------",
        "",
        "function __affi_no_noun",
        "    set -l toks (commandline -opc)",
        "    for t in $toks[2..-1]",
        "        switch $t",
        f"            case {noun_switch}",
        "                return 1",
        "        end",
        "    end",
        "    return 0",
        "end",
        "",
        "function __affi_using_noun",
        "    set -l toks (commandline -opc)",
        "    for t in $toks[2..-1]",
        '        if test "$t" = "$argv[1]"',
        "            return 0",
        "        end",
        "    end",
        "    return 1",
        "end",
        "",
        "function __affi_no_verb",
        "    set -l toks (commandline -opc)",
        "    set -l found_noun 0",
        "    for t in $toks[2..-1]",
        "        switch $t",
        f"            case {noun_switch}",
        "            set found_noun 1",
        "            case '*'",
        "            if test $found_noun -eq 1",
        "                return 1",
        "            end",
        "        end",
        "    end",
        "    return 0",
        "end",
        "",
        "# --- Nouns -------------------------------------------------------------------",
        "",
    ]
    for noun in nouns:
        out.append(
            f"complete -c affi -f -n '__affi_no_noun' -a {noun} -d '{len(nouns[noun])} verbs'"
        )
    out.append("")
    for noun, rows in nouns.items():
        out.append(f"# --- {noun} verbs ------------------------------------------------------")
        out.append("")
        guard = f"'__affi_using_noun {noun}; and __affi_no_verb'"
        for row in rows:
            out.append(
                f"complete -c affi -f -n {guard} -a {row['verb']} "
                f"-d '{shell_quote_single(row['summary'])}'"
            )
        out.append("")
    out += [
        "# --- top-level flags ---------------------------------------------------------",
        "",
        "complete -c affi -s h -l help    -d 'Show help'",
        "complete -c affi -s V -l version -d 'Show version'",
        "",
    ]
    return "\n".join(out)


def ps_single_quote(text: str) -> str:
    """Escape for a PowerShell single-quoted string (apostrophe is doubled)."""
    return text.replace("'", "''")


def emit_powershell(nouns: dict[str, list[dict[str, str]]]) -> str:
    vc, nc = sum(len(v) for v in nouns.values()), len(nouns)
    out = [
        header("affi.ps1", "# PowerShell completion — dot-source via `. ./completions/affi.ps1`", vc, nc),
        "#",
        "# Install: add `. /path/to/completions/affi.ps1` to your $PROFILE.",
        "",
        "# noun -> ordered (verb -> summary) table, from src/registry.rs.",
        "$script:AffiCompletions = [ordered]@{",
    ]
    for noun, rows in nouns.items():
        out.append(f"    '{ps_single_quote(noun)}' = [ordered]@{{")
        for row in rows:
            out.append(f"        '{ps_single_quote(row['verb'])}' = '{ps_single_quote(row['summary'])}'")
        out.append("    }")
    out += [
        "}",
        "",
        "Register-ArgumentCompleter -Native -CommandName 'affi' -ScriptBlock {",
        "    param($wordToComplete, $commandAst, $cursorPosition)",
        "",
        "    # Words after `affi`, minus the word currently being completed.",
        "    $words = @($commandAst.CommandElements | Select-Object -Skip 1 | ForEach-Object { $_.ToString() })",
        "    if ($wordToComplete -ne '' -and $words.Count -gt 0) {",
        "        $words = @($words | Select-Object -First ($words.Count - 1))",
        "    }",
        "    $positional = @($words | Where-Object { $_ -notlike '-*' })",
        "",
        "    $candidates = [ordered]@{}",
        "    if ($wordToComplete -like '-*') {",
        "        $candidates['--help'] = 'Show help'",
        "        $candidates['--version'] = 'Show version'",
        "    } elseif ($positional.Count -eq 0) {",
        "        foreach ($n in $script:AffiCompletions.Keys) {",
        "            $candidates[$n] = \"$($script:AffiCompletions[$n].Count) verbs\"",
        "        }",
        "    } elseif ($positional.Count -eq 1 -and $script:AffiCompletions.Contains($positional[0])) {",
        "        $verbs = $script:AffiCompletions[$positional[0]]",
        "        foreach ($v in $verbs.Keys) { $candidates[$v] = $verbs[$v] }",
        "    }",
        "",
        "    foreach ($name in $candidates.Keys) {",
        "        if ($name -like \"$wordToComplete*\") {",
        "            $kind = if ($name -like '-*') { 'ParameterName' } else { 'ParameterValue' }",
        "            [System.Management.Automation.CompletionResult]::new($name, $name, $kind, $candidates[$name])",
        "        }",
        "    }",
        "}",
        "",
    ]
    return "\n".join(out)


def nu_string(text: str) -> str:
    """Render a Nushell double-quoted string literal."""
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def emit_nushell(nouns: dict[str, list[dict[str, str]]]) -> str:
    vc, nc = sum(len(v) for v in nouns.values()), len(nouns)
    out = [
        header("affi.nu", "# Nushell completion — load via `source completions/affi.nu`", vc, nc),
        "#",
        "# Install: add `source /path/to/completions/affi.nu` to your config.nu.",
        "",
        'def "nu-complete affi nouns" [] {',
        "    [",
    ]
    for noun in nouns:
        out.append(f"        {{value: {nu_string(noun)}, description: {nu_string(f'{len(nouns[noun])} verbs')}}}")
    out += [
        "    ]",
        "}",
        "",
        'def "nu-complete affi verbs" [context: string] {',
        '    let noun = ($context | split row " " | where {|w| $w != ""} | skip 1 | first)',
        "    match $noun {",
    ]
    for noun, rows in nouns.items():
        out.append(f"        {nu_string(noun)} => [")
        for row in rows:
            out.append(
                f"            {{value: {nu_string(row['verb'])}, description: {nu_string(row['summary'])}}}"
            )
        out.append("        ]")
    out += [
        "        _ => []",
        "    }",
        "}",
        "",
        "# Top-level command: `affi <noun> <verb> ...`.",
        'export extern "affi" [',
        '    noun?: string@"nu-complete affi nouns"',
        '    verb?: string@"nu-complete affi verbs"',
        "    ...rest: string",
        "    --help(-h)      # Show help",
        "    --version(-V)   # Show version",
        "]",
        "",
    ]
    return "\n".join(out)


def main() -> None:
    rows = parse_registry()
    nouns = by_noun(rows)
    verb_count = len(rows)

    outputs = {
        "affi.bash": emit_bash(nouns),
        "affi.zsh": emit_zsh(nouns),
        "affi.fish": emit_fish(nouns),
        "affi.ps1": emit_powershell(nouns),
        "affi.nu": emit_nushell(nouns),
    }
    COMPLETIONS_DIR.mkdir(parents=True, exist_ok=True)
    for filename, content in outputs.items():
        path = COMPLETIONS_DIR / filename
        path.write_bytes(content.encode("utf-8"))  # LF-terminated, no platform newline translation
        print(f"wrote {path} ({len(content.splitlines())} lines)")

    print(f"registry: {verb_count} verbs across {len(nouns)} nouns: {', '.join(nouns)}")


if __name__ == "__main__":
    main()
