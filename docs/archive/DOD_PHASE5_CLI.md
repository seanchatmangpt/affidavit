# Definition of Done — Phase 5: CLI Ergonomics & Advanced Features

**Project:** affidavit (DX/QOL 1000x Initiative)  
**Phase:** 5 of 5  
**Branch:** `claude/zen-cerf-oq87br`  
**Version:** 26.6.14  
**Last Updated:** 2026-06-14  
**Status:** Pre-implementation — definition locked, implementation pending

---

## Table of Contents

1. [Phase Overview](#phase-overview)
2. [Feature 5.1 — Help Formatter](#feature-51--help-formatter)
3. [Feature 5.2 — Auto-Generated Examples](#feature-52--auto-generated-examples)
4. [Feature 5.3 — Command Aliases](#feature-53--command-aliases)
5. [Feature 5.4 — JSON Output](#feature-54--json-output)
6. [Feature 5.5 — Interactive Shell REPL](#feature-55--interactive-shell-repl)
7. [E2E Test Template](#e2e-test-template-testse2e_clirs)
8. [JSON Schema Reference](#json-schema-reference)
9. [REPL Command Reference](#repl-command-reference)
10. [Alias Table](#alias-table)
11. [Example Auto-Gen Pipeline Diagram](#example-auto-gen-pipeline-diagram)
12. [Phase 5 Exit Criteria](#phase-5-exit-criteria)

---

## Phase Overview

Phase 5 completes the DX/QOL 1000x initiative by making the `affi` CLI self-explanatory,
consistent, composable, and interactive. The five features in this phase address ergonomics
at the output layer (help text, JSON, examples) and at the interaction layer (aliases, REPL).

### Files Touched

| File | Action | Owner |
|------|--------|-------|
| `src/cli.rs` | MODIFY — add `format_help_markdown()`, `--format` flag | cli agent |
| `src/bin/affi-shell.rs` | NEW — interactive REPL binary | shell agent |
| `ontology/affi-cli.ttl` | MODIFY — add alias triples, inspect/diff verbs | ontology agent |
| `examples/*.sh` | NEW (auto-generated) — executable shell examples | ggen |
| `Cargo.toml` | MODIFY — add `rustyline`, feature gates | integration agent |

### Feature Dependency Order

```
5.1 Help Formatter
    └─ blocks: help text spec affects all other features' --help output

5.2 Auto-Generated Examples
    └─ depends on: 5.1 (help text format), ggen fixture rendering

5.3 Command Aliases
    └─ depends on: ontology triples (cnv:hasAlias)

5.4 JSON Output
    └─ blocks: 5.5 (REPL uses JSON output internally)
    └─ depends on: types::InspectionReport, DiffResult (Phase 1/2)

5.5 Interactive Shell REPL
    └─ depends on: 5.4 (JSON output as REPL's internal format)
```

---

## Feature 5.1 — Help Formatter

### Summary

Ontology-driven help text rendered through `ggen` and displayed by `clap-noun-verb`.
A new `format_help_markdown(md: &str) -> String` function in `src/cli.rs` converts
any Markdown string to plain ASCII suitable for terminal output. Help text for every
verb includes ARDPRD cross-references.

### Acceptance Criteria

**Given/When/Then (minimum 8)**

**AC-5.1.1**
```
Given: the function format_help_markdown is called with "**bold text**"
When:  the function executes
Then:  it returns "bold text" with no asterisks
```

**AC-5.1.2**
```
Given: format_help_markdown receives "`code span`"
When:  the function executes
Then:  it returns "code span" with no backtick characters
```

**AC-5.1.3**
```
Given: format_help_markdown receives a fenced code block
       (lines beginning with ```)
When:  the function executes
Then:  the fence delimiters are removed and the inner content is preserved,
       indented by 4 spaces
```

**AC-5.1.4**
```
Given: format_help_markdown receives "[link text](https://example.com)"
When:  the function executes
Then:  the output contains "link text" followed by
       "(https://example.com)" in plain ASCII, no brackets
```

**AC-5.1.5**
```
Given: "affi receipt verify --help" is executed in a shell
When:  the output is captured
Then:  no line in the output exceeds 80 characters
```

**AC-5.1.6**
```
Given: "affi receipt verify --help" is executed in a shell
When:  the output is captured
Then:  the output contains the text "See also: ARDPRD" followed by
       a section reference (e.g., "SS3" or "FR-3")
```

**AC-5.1.7**
```
Given: "affi receipt emit --help" is executed in a shell
When:  the output is captured
Then:  the output contains "See also: ARDPRD" and references FR-1
       (Receipt emission requirement)
```

**AC-5.1.8**
```
Given: "affi receipt assemble --help" is executed in a shell
When:  the output is captured
Then:  the output contains "See also: ARDPRD" and references FR-2
       (Chain assembly requirement)
```

**AC-5.1.9**
```
Given: format_help_markdown receives "# Heading" (h1 markdown)
When:  the function executes
Then:  the output contains the heading text in uppercase,
       underlined with "=" characters, with no "#" prefix
```

**AC-5.1.10**
```
Given: the ggen.toml verb-help template references cnv:verbAbout for each verb
When:  "ggen sync" is run successfully
Then:  the generated src/verbs/*.rs files contain #[doc] comments matching
       the verbAbout strings in ontology/affi-cli.ttl
```

### Help Text Spec

All help text rendered by `affi` must conform to the following specification:

**General Rules**
- Maximum line width: **80 characters** (hard limit, enforced by test)
- No raw Markdown syntax in output (no `**`, `` ` ``, `#`, `[`, `]`, `(`, `)`)
- Paragraph wrapping: reflowed at 80 columns with a single blank line between paragraphs
- Code examples: indented 4 spaces, no fences

**ARDPRD Cross-Reference Format**

Every verb's help text must end with a cross-reference block:

```
See also: ARDPRD <section>
```

where `<section>` is one of the codes in the table below. The line must be
preceded by a blank line and must not exceed 80 characters.

| Verb | ARDPRD Section | Requirement Text |
|------|---------------|-----------------|
| emit | FR-1 | Receipt emission |
| assemble | FR-2 | Chain assembly |
| verify | FR-3 | Verification |
| show | FR-4 | Inspection (non-adjudicating) |
| inspect | FR-4 | Inspection (non-adjudicating) |
| diff | FR-4 | Inspection (non-adjudicating) |
| stats | NFR-1 | Determinism |
| graph | FR-4 | Inspection (non-adjudicating) |
| replay | NFR-1 | Determinism |
| model | FR-5 | CLI surface |
| conformance | FR-5 | CLI surface |
| diagnose | FR-6 | Tamper teeth |

**Example: `affi receipt verify --help` output (exact format)**

```
affi receipt verify

Run the certify pipeline over a receipt and print the verdict.

The pipeline runs 7 stages in order: decode, check_format,
chain_integrity, continuity, verify_commitments, evaluate_profile,
emit_verdict. Each stage produces a pass/fail outcome. The verdict
is ACCEPT only when all stages pass.

Exit code 0 indicates ACCEPT. Exit code 2 indicates REJECT.

USAGE:
    affi receipt verify <RECEIPT>

ARGUMENTS:
    <RECEIPT>    Path to the receipt JSON to verify

OPTIONS:
    --format <FORMAT>    Output format: text (default) or json
    --profile <PROFILE>  Conformance profile [default: core/v1]
    --strict             Treat warnings as failures
    -h, --help           Print this help text

See also: ARDPRD FR-3
```

**`format_help_markdown` Function Specification**

```rust
// Location: src/cli.rs
// Visibility: pub(crate)

/// Convert a Markdown string to plain ASCII for terminal display.
///
/// Transformations applied in order:
/// 1. ATX headings (#, ##, ###) -> UPPERCASE text + underline (=, -, ~)
/// 2. Bold (**text** or __text__) -> text (strips markers)
/// 3. Italic (*text* or _text_) -> text (strips markers)
/// 4. Inline code (`text`) -> text (strips backticks)
/// 5. Fenced code blocks (``` ... ```) -> 4-space indented block
/// 6. Links [text](url) -> "text (url)"
/// 7. Reflow all paragraphs to max_width (default 80)
///
/// Does NOT transform: HTML entities, tables, footnotes (passed through as-is).
pub(crate) fn format_help_markdown(md: &str) -> String { ... }
pub(crate) fn format_help_markdown_width(md: &str, max_width: usize) -> String { ... }
```

### Implementation Notes

- `format_help_markdown` must not allocate more than 2x input length
- The function is pure: no I/O, no global state, deterministic
- ggen ontology template reads `cnv:verbAbout` and appends ARDPRD ref from a
  static mapping in `ggen.toml` under `[ardprd_refs]` keyed by verb name
- The ARDPRD mapping is maintained in `ggen.toml`, not in the TTL ontology
  (separation of concerns: ontology describes structure, config maps to docs)

---

## Feature 5.2 — Auto-Generated Examples

### Summary

ggen renders chicago-tdd fixture data into executable `.sh` scripts in `examples/`.
Every generated example script must pass a sandbox runner test (exit 0, no error output).
No example may bit-rot: tests run every generated script in CI.

### Acceptance Criteria

**AC-5.2.1**
```
Given: ggen.toml contains a [examples] section pointing at chicago-tdd fixtures
When:  "ggen sync" is run
Then:  files matching examples/ex_*.sh are created or updated
       and each file has execute permission (chmod +x)
```

**AC-5.2.2**
```
Given: an auto-generated example script exists at examples/ex_emit_assemble.sh
When:  the script is executed in a clean temporary directory
Then:  the script exits with code 0 and produces no output on stderr
```

**AC-5.2.3**
```
Given: an auto-generated example script asserts a specific receipt file exists
When:  the script is executed
Then:  the assertion passes and the receipt file is present after the script completes
```

**AC-5.2.4**
```
Given: a chicago-tdd fixture named "linear_3_event" exists
When:  ggen renders it as an example
Then:  the resulting script emits exactly 3 events (one per fixture event)
       and assembles into a single receipt
```

**AC-5.2.5**
```
Given: an auto-generated example is executed
When:  "affi receipt verify" is called on the assembled receipt inside the script
Then:  verify exits 0 (ACCEPT) without modification
```

**AC-5.2.6**
```
Given: the source chicago-tdd fixture is updated
When:  "ggen sync" is run again
Then:  the corresponding example script is regenerated to match the new fixture
       and the old version is overwritten, not appended
```

**AC-5.2.7**
```
Given: a generated example script is run in a CI environment with no affi binary
When:  the script is sourced
Then:  the script fails with a clear error message on the first missing binary,
       not with a cryptic shell error
```

**AC-5.2.8**
```
Given: the test suite runs "cargo test example_scripts_execute"
When:  any generated example script exits non-zero
Then:  the test fails and prints the script name and captured stderr
```

**AC-5.2.9**
```
Given: a generated example script
When:  it is inspected
Then:  it contains no hardcoded BLAKE3 hash values;
       all hash assertions use pattern matching (e.g., grep -E "[0-9a-f]{64}")
```

**AC-5.2.10**
```
Given: ggen renders a new example from a fixture that uses object qualifiers
When:  the script runs
Then:  the --object flag in the emit call uses the full "id:type:qualifier" format
```

### Example Script Template

All generated scripts must follow this exact structure:

```bash
#!/usr/bin/env bash
# ============================================================
# Example: <FIXTURE_NAME>
# Generated by: ggen from chicago-tdd fixture '<FIXTURE_ID>'
# DO NOT EDIT — regenerate with: ggen sync
# ============================================================

set -euo pipefail

# -- Preamble -------------------------------------------------
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT
cd "$WORKDIR"

# Verify affi binary is available
if ! command -v affi >/dev/null 2>&1; then
    echo "ERROR: affi binary not found in PATH" >&2
    exit 127
fi

# -- Events ---------------------------------------------------
# Event 1: <EVENT_TYPE> (seq 0)
echo '{"key":"value"}' | affi receipt emit \
    --type <EVENT_TYPE> \
    --object <OBJECT_ID>:<OBJECT_TYPE> \
    --payload -

# Event 2: <EVENT_TYPE> (seq 1)
echo '{"key":"value2"}' | affi receipt emit \
    --type <EVENT_TYPE_2> \
    --object <OBJECT_ID_2>:<OBJECT_TYPE_2> \
    --payload -

# -- Assembly -------------------------------------------------
affi receipt assemble --out receipt.json

# -- Assertions -----------------------------------------------
# Receipt file must exist
test -f receipt.json || { echo "FAIL: receipt.json not created" >&2; exit 1; }

# Receipt must verify ACCEPT
affi receipt verify receipt.json
echo "PASS: receipt verified ACCEPT"

# Chain hash must be a valid BLAKE3 hex digest (64 lowercase hex chars)
CHAIN_HASH=$(python3 -c "import json,sys; print(json.load(sys.stdin)['chain_hash'])" < receipt.json)
echo "$CHAIN_HASH" | grep -qE '^[0-9a-f]{64}$' \
    || { echo "FAIL: chain_hash is not a valid BLAKE3 digest" >&2; exit 1; }

echo "PASS: <FIXTURE_NAME> example completed successfully"
```

**Template Variables (ggen substitutions)**

| Variable | Source | Example |
|----------|--------|---------|
| `<FIXTURE_NAME>` | `fixture.name` | `linear_3_event` |
| `<FIXTURE_ID>` | `fixture.id` | `chicago_tdd:linear_3_event` |
| `<EVENT_TYPE>` | `fixture.events[n].event_type` | `build` |
| `<OBJECT_ID>` | `fixture.events[n].objects[0].id` | `repo` |
| `<OBJECT_TYPE>` | `fixture.events[n].objects[0].obj_type` | `git` |

### Anti-Bit-Rot Requirements

1. Generated scripts are committed to source control (`examples/ex_*.sh`)
2. CI runs `cargo test -- example_scripts_execute` on every PR
3. The test harness captures stderr and fails verbosely on any non-zero exit
4. Scripts must not reference absolute paths (use `$WORKDIR` or relative)
5. Scripts must not embed content-addressed filenames (use `receipt.json` alias)

### ggen Configuration (ggen.toml additions)

```toml
[[examples]]
fixture_source = "chicago-tdd-tools"
output_dir = "examples"
filename_pattern = "ex_{fixture_name}.sh"
template = "templates/example_script.sh.tera"
chmod_execute = true

[examples.fixture_filter]
# Only generate examples from fixtures tagged "phase5-approved"
tags = ["phase5-approved"]
```

---

## Feature 5.3 — Command Aliases

### Summary

Short aliases for the `receipt` noun and common verbs, declared as
`cnv:hasAlias` triples in `ontology/affi-cli.ttl`. The `clap-noun-verb` library
reads alias triples at initialization and registers clap `Arg::visible_alias`
entries. No runtime dispatch logic is added to `affi.rs`.

### Acceptance Criteria

**AC-5.3.1**
```
Given: "affi r inspect receipt.json" is executed
When:  the command runs
Then:  it produces identical output to "affi receipt inspect receipt.json"
       and exits 0
```

**AC-5.3.2**
```
Given: "affi r v receipt.json" is executed
When:  the command runs
Then:  it produces identical output to "affi receipt verify receipt.json"
       and exits 0
```

**AC-5.3.3**
```
Given: "affi r e --type build --object repo:git --payload -" is executed
When:  the command runs
Then:  it produces identical output to
       "affi receipt emit --type build --object repo:git --payload -"
       and exits 0
```

**AC-5.3.4**
```
Given: "affi r a" is executed (assemble alias)
When:  the command runs
Then:  it produces identical output to "affi receipt assemble"
       and exits 0
```

**AC-5.3.5**
```
Given: "affi r s receipt.json" is executed (show alias)
When:  the command runs
Then:  it produces identical output to "affi receipt show receipt.json"
       and exits 0
```

**AC-5.3.6**
```
Given: "affi r --help" is executed
When:  the output is captured
Then:  it lists all available verbs AND their aliases in the commands section
```

**AC-5.3.7**
```
Given: an alias "r" is declared for the receipt noun in affi-cli.ttl
When:  "ggen sync" is run
Then:  the generated clap configuration includes a visible_alias("r") call
       on the receipt subcommand
```

**AC-5.3.8**
```
Given: a shell completion script is generated for bash
When:  "affi r <TAB>" is typed in a bash shell with completion sourced
Then:  the completion suggestions include all verb names AND verb aliases
```

**AC-5.3.9**
```
Given: an unrecognized alias is typed (e.g., "affi receipt x")
When:  the command runs
Then:  affi exits non-zero and prints a clear "unknown command" error
       listing the available verbs and their aliases
```

**AC-5.3.10**
```
Given: the alias table in ontology/affi-cli.ttl is the single source of truth
When:  a new alias triple is added to the TTL file
Then:  running "ggen sync" regenerates the CLI without any manual code changes
```

### Alias Mapping Table (Canonical Form)

This table is the authoritative alias specification. Every row must be
implemented and tested.

| Canonical Form | Short Alias | Long Aliases | Notes |
|---------------|-------------|--------------|-------|
| `affi receipt` | `affi r` | (none) | Noun alias only |
| `affi receipt emit` | `affi r e` | `affi r emit` | Verb alias under noun alias |
| `affi receipt assemble` | `affi r a` | `affi r assemble` | Verb alias under noun alias |
| `affi receipt verify` | `affi r v` | `affi r verify` | Verb alias under noun alias |
| `affi receipt show` | `affi r s` | `affi r show` | Verb alias under noun alias |
| `affi receipt inspect` | `affi r i` | `affi r inspect` | New verb from Phase 1 |
| `affi receipt diff` | `affi r d` | `affi r diff` | New verb from Phase 1 |
| `affi receipt stats` | (none) | `affi r stats` | Stats verb, no single-char alias |
| `affi receipt graph` | (none) | `affi r graph` | Graph verb, no single-char alias |
| `affi receipt replay` | (none) | `affi r replay` | Replay verb, no single-char alias |
| `affi receipt model` | (none) | `affi r model` | Model verb, no single-char alias |
| `affi receipt conformance` | (none) | `affi r conformance` | Conformance verb, no single-char alias |
| `affi receipt diagnose` | (none) | `affi r diagnose` | Diagnose verb, no single-char alias |

**Why some verbs have no single-char alias:** Single-char aliases are only
assigned where the letter is unambiguous and the verb is in the high-frequency
path (emit, assemble, verify, show, inspect, diff). Stats, graph, replay, model,
conformance, and diagnose are lower-frequency operations where a typo would be
confusing.

### Ontology Additions (affi-cli.ttl)

```turtle
# Noun alias
affi:ReceiptNoun
    cnv:hasAlias "r" .

# Verb aliases — emit
affi:EmitVerb
    cnv:hasAlias "e" .

# Verb aliases — assemble
affi:AssembleVerb
    cnv:hasAlias "a" .

# Verb aliases — verify
affi:VerifyVerb
    cnv:hasAlias "v" .

# Verb aliases — show
affi:ShowVerb
    cnv:hasAlias "s" .

# Verb aliases — inspect (added in Phase 1, alias registered in Phase 5)
affi:InspectVerb
    cnv:hasAlias "i" .

# Verb aliases — diff (added in Phase 1, alias registered in Phase 5)
affi:DiffVerb
    cnv:hasAlias "d" .
```

---

## Feature 5.4 — JSON Output

### Summary

Every verb in the `receipt` noun group gains a `--format=json` flag. When
`--format=json` is passed, the verb's output is written as a single JSON
object to stdout (no human-readable decoration). The feature is gated behind
`feature = "json-output"` in `Cargo.toml`. The default remains `--format=text`.

### Acceptance Criteria

**AC-5.4.1**
```
Given: "affi receipt verify receipt.json --format=json" is executed
When:  the command runs
Then:  the stdout contains a single JSON object that can be parsed by
       "python3 -c 'import json,sys; json.load(sys.stdin)'"
       and the object has the keys: "accepted", "profile", "outcomes", "reason"
```

**AC-5.4.2**
```
Given: "affi receipt inspect receipt.json --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "event_count", "event_types", "object_types", "chain_hash",
       "format_version", "chain_integrity_valid"
```

**AC-5.4.3**
```
Given: "affi receipt show receipt.json --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "format_version", "events", "chain_hash"
       and "events" is an array of event objects
```

**AC-5.4.4**
```
Given: "affi receipt diff a.json b.json --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "added", "removed", "modified", "event_count_a", "event_count_b"
```

**AC-5.4.5**
```
Given: "affi receipt emit ... --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "event_id", "seq", "event_type", "commitment"
```

**AC-5.4.6**
```
Given: "affi receipt assemble --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "receipt_path", "content_address", "event_count"
```

**AC-5.4.7**
```
Given: "affi receipt stats receipt.json --format=json" is executed
When:  the command runs
Then:  stdout is a JSON object with keys:
       "event_count", "chain_depth", "event_type_histogram",
       "object_type_histogram", "chain_hash"
```

**AC-5.4.8**
```
Given: the json-output feature is NOT enabled at compile time
When:  "affi receipt verify receipt.json --format=json" is executed
Then:  affi exits non-zero with a message indicating the json-output feature
       is not compiled in; text output is still available
```

**AC-5.4.9**
```
Given: "affi receipt verify receipt.json --format=json" is executed
       on a REJECT receipt
When:  the command runs
Then:  the JSON object has "accepted": false and "reason" contains the
       first failure stage name; the exit code is still 2 (not 0)
```

**AC-5.4.10**
```
Given: "affi receipt verify receipt.json" is executed (no --format flag)
When:  the command runs
Then:  the output is the existing human-readable text format (not JSON),
       preserving backward compatibility with current behavior
```

**AC-5.4.11**
```
Given: JSON output is piped to jq
       ("affi receipt verify r.json --format=json | jq '.accepted'")
When:  the pipeline runs
Then:  jq prints "true" or "false" without error
```

**AC-5.4.12**
```
Given: --format=yaml is passed (unsupported format)
When:  the command runs
Then:  affi exits non-zero and prints the list of accepted format values
```

### Feature Gate (Cargo.toml)

```toml
[features]
default = []
otel = []
json-output = []          # Phase 5.4: --format=json for all verbs
shell = ["rustyline"]     # Phase 5.5: interactive REPL

[dependencies]
# existing deps ...
rustyline = { version = "13", optional = true }
```

### JSON Output Method Signatures

```rust
// Each type gets a to_json() method gated on the feature flag.

#[cfg(feature = "json-output")]
impl Verdict {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "json-output")]
impl InspectionReport {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(feature = "json-output")]
impl DiffResult {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
```

### `--format` Flag Registration (ontology/affi-cli.ttl additions)

```turtle
# Shared format argument — add to every verb that supports output formatting
affi:FormatArg a cnv:Argument ;
    cnv:hasArgumentName "format" ;
    cnv:argumentAbout "Output format: text (default) or json" ;
    cnv:valueType "String" ;
    cnv:required "false" ;
    cnv:defaultValue "text" .

# Wire to each verb:
affi:VerifyVerb cnv:hasArguments affi:FormatArg .
affi:ShowVerb   cnv:hasArguments affi:FormatArg .
affi:InspectVerb cnv:hasArguments affi:FormatArg .
affi:DiffVerb   cnv:hasArguments affi:FormatArg .
affi:StatsVerb  cnv:hasArguments affi:FormatArg .
affi:EmitVerb   cnv:hasArguments affi:FormatArg .
affi:AssembleVerb cnv:hasArguments affi:FormatArg .
```

---

## Feature 5.5 — Interactive Shell REPL

### Summary

`src/bin/affi-shell.rs` implements an interactive shell for `affi`. The binary
is gated behind `feature = "shell"`. Without the feature, `affi shell` prints
a message directing users to rebuild with `--features shell`. With the feature,
the shell provides readline-like history and tab completion via `rustyline`,
and a set of REPL commands that operate on a loaded receipt in memory.

### Acceptance Criteria

**AC-5.5.1**
```
Given: affi is built with "--features shell"
When:  "affi shell" is executed in a terminal
Then:  a prompt "affi> " appears and the shell accepts input
```

**AC-5.5.2**
```
Given: the affi shell is running
When:  the user types "load receipt.json"
Then:  the shell responds "loaded: receipt.json (N events)" where N is the
       event count, and the receipt is held in memory for subsequent commands
```

**AC-5.5.3**
```
Given: a receipt is loaded in the shell
When:  the user types "inspect"
Then:  the shell prints the InspectionReport for the loaded receipt,
       identical to what "affi receipt inspect <path>" would produce
```

**AC-5.5.4**
```
Given: a receipt is loaded in the shell
When:  the user types "verify"
Then:  the shell prints the 7-stage pipeline output and the verdict,
       identical to what "affi receipt verify <path>" would produce
```

**AC-5.5.5**
```
Given: the affi shell is running
When:  the user types "diff other.json"
Then:  the shell diffs the currently loaded receipt against other.json
       and prints the DiffResult
```

**AC-5.5.6**
```
Given: a receipt is loaded in the shell
When:  the user types "trace"
Then:  the shell emits one OTel span per verifier stage (requires otel feature)
       or prints "trace requires the otel feature" if otel is not compiled in
```

**AC-5.5.7**
```
Given: the affi shell is running
When:  the user types "help"
Then:  the shell prints the REPL command reference table
       (all commands, syntax, description, one-liner)
```

**AC-5.5.8**
```
Given: the affi shell is running
When:  the user types "quit" or "exit" or sends Ctrl-D
Then:  the shell exits with code 0 and no error output
```

**AC-5.5.9**
```
Given: affi is built WITHOUT "--features shell"
When:  "affi shell" is executed
Then:  affi exits non-zero with:
       "affi shell requires the 'shell' feature. Rebuild with:
        cargo build --features shell"
```

**AC-5.5.10**
```
Given: the affi shell is running with rustyline enabled
When:  the user presses the up-arrow key
Then:  the shell recalls the previous command from history,
       identical to standard readline behavior
```

**AC-5.5.11**
```
Given: the affi shell is running with rustyline enabled
When:  the user types "in" and presses Tab
Then:  the shell completes to "inspect" (or shows alternatives if ambiguous)
```

**AC-5.5.12**
```
Given: the affi shell is running
When:  the user types "mutate 5" (require mutate verb from Phase 3)
Then:  the shell applies 5 random mutations to the loaded receipt,
       prints each mutation type and whether verify rejects it,
       and reports the kill ratio "killed N/5"
```

**AC-5.5.13**
```
Given: no receipt is loaded and the user types "inspect"
When:  the command runs
Then:  the shell prints "no receipt loaded. Use: load <path>"
       and does not panic
```

**AC-5.5.14**
```
Given: the affi shell is running
When:  the user types a command that does not exist (e.g., "frobnicate")
Then:  the shell prints "unknown command: frobnicate. Type 'help' for commands."
       and continues accepting input (does not exit)
```

### REPL Command Table

| Command | Syntax | Description | Output Format |
|---------|--------|-------------|---------------|
| `load` | `load <path>` | Load a receipt into memory from `<path>` | `loaded: <path> (N events)` |
| `inspect` | `inspect` | Print InspectionReport for the loaded receipt | Human-readable or JSON if --json passed at shell start |
| `verify` | `verify` | Run the 7-stage certify pipeline on the loaded receipt | Per-stage outcomes + ACCEPT/REJECT verdict |
| `show` | `show` | Print human-readable dump of the loaded receipt | Event list with seq, type, objects, commitment (12 chars) |
| `diff` | `diff <path>` | Diff the loaded receipt against `<path>` | DiffResult: added/removed/modified events |
| `mutate` | `mutate <N>` | Apply N random mutations; report which are rejected | `killed N/N mutations` |
| `trace` | `trace` | Emit OTel spans for the loaded receipt's verifier run | Span IDs printed; requires otel feature |
| `stats` | `stats` | Print chain metrics for the loaded receipt | Event count, chain depth, hash distribution |
| `graph` | `graph [--format=dot\|json]` | Print DAG of the loaded receipt | DOT or JSON graph representation |
| `reload` | `reload` | Reload the currently-loaded receipt from disk | Same as load, reusing the last path |
| `path` | `path` | Print the path of the currently-loaded receipt | File path or "(none)" |
| `clear` | `clear` | Unload the current receipt from memory | `cleared` |
| `help` | `help [command]` | Print this command table, or help for one command | Formatted command reference |
| `quit` | `quit` or `exit` | Exit the shell with code 0 | (none) |

### Shell Feature Degradation Table

| Situation | Behavior |
|-----------|----------|
| `feature = "shell"` absent, `affi shell` called | Exit 1 with feature rebuild message |
| `feature = "shell"` present, `rustyline` absent from deps | Compile error; shell feature requires rustyline |
| Shell running, `feature = "otel"` absent, `trace` called | Print "trace requires the otel feature" |
| Shell running, `feature = "json-output"` absent, inspect in JSON mode | Print "json-output feature not compiled in; output is text" |
| Shell running, `feature = "shell"` present, no terminal (piped stdin) | Accept commands from stdin without readline; no history |
| Shell running, Ctrl-C received | Clear current input line; do not exit (standard readline behavior) |
| Shell running, Ctrl-D on empty line | Exit with code 0 (standard EOF behavior) |
| Shell running, `load` called with non-existent path | Print error; do not change currently-loaded receipt |
| Shell running, `load` called with a tampered receipt | Print "receipt failed deserialization: chain hash mismatch"; do not load |

### REPL Entry Point (src/bin/affi-shell.rs structure)

```rust
// src/bin/affi-shell.rs
// Feature-gated: only compiles when feature = "shell" is enabled.
// When the feature is absent, affi.rs detects the "shell" subcommand
// and prints the feature rebuild message without this binary needing to compile.

#[cfg(feature = "shell")]
fn main() -> anyhow::Result<()> {
    use rustyline::DefaultEditor;
    let mut rl = DefaultEditor::new()?;
    let mut state = ShellState::new();

    loop {
        match rl.readline("affi> ") {
            Ok(line) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    rl.add_history_entry(&line)?;
                    if let Err(e) = dispatch_command(&line, &mut state) {
                        eprintln!("error: {e}");
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(rustyline::error::ReadlineError::Interrupted) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

struct ShellState {
    loaded_receipt: Option<(String, affidavit::types::Receipt)>,
}
```

---

## E2E Test Template (`tests/e2e_cli.rs`)

The following is the complete, concrete E2E test file for Phase 5. All tests must
pass on `cargo test --features shell,json-output,otel` before Phase 5 is closed.

```rust
//! End-to-end tests for Phase 5: CLI Ergonomics & Advanced Features.
//!
//! Tests the full stack: binary invocation via assert_cmd, receipt construction
//! through CLI flags, and output parsing. Each test uses a real `affi` binary
//! in a temporary working directory.
//!
//! Run: cargo test --test e2e_cli --features shell,json-output

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Build a fresh temporary directory and return it.
/// All paths are absolute; the dir is removed on drop.
fn workdir() -> TempDir {
    tempfile::tempdir().expect("tmpdir")
}

/// Run `affi receipt emit` with the given args in `dir`.
fn emit_event(dir: &TempDir, event_type: &str, object: &str, payload: &[u8]) {
    let payload_path = dir.path().join("payload.bin");
    fs::write(&payload_path, payload).expect("write payload");

    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args([
            "receipt", "emit",
            "--type", event_type,
            "--object", object,
            "--payload", payload_path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// Run `affi receipt assemble` in `dir` and return the path to the receipt.
fn assemble(dir: &TempDir) -> std::path::PathBuf {
    let out = dir.path().join("receipt.json");

    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["receipt", "assemble", "--out", out.to_str().unwrap()])
        .assert()
        .success();

    out
}

// ─── Feature 5.1: Help Formatter ────────────────────────────────────────────

#[test]
fn test_help_verify_no_markdown_syntax() {
    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "verify", "--help"])
        .output()
        .expect("run help");

    let text = String::from_utf8_lossy(&output.stdout);

    // No raw Markdown bold markers
    assert!(
        !text.contains("**"),
        "help output must not contain Markdown bold (**): got:\n{text}"
    );

    // No raw backtick inline code
    assert!(
        !text.contains('`'),
        "help output must not contain backticks: got:\n{text}"
    );

    // No lines longer than 80 characters
    for (i, line) in text.lines().enumerate() {
        assert!(
            line.len() <= 80,
            "help line {} exceeds 80 chars ({} chars): {:?}",
            i + 1,
            line.len(),
            line
        );
    }
}

#[test]
fn test_help_verify_ardprd_reference() {
    Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "verify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARDPRD"))
        .stdout(predicate::str::contains("FR-3"));
}

#[test]
fn test_help_emit_ardprd_reference() {
    Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "emit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARDPRD"))
        .stdout(predicate::str::contains("FR-1"));
}

#[test]
fn test_help_assemble_ardprd_reference() {
    Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "assemble", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARDPRD"))
        .stdout(predicate::str::contains("FR-2"));
}

#[test]
fn test_format_help_markdown_unit() {
    // White-box unit test for the format_help_markdown function.
    // Tests are inline in src/cli.rs; this is a cross-check at the CLI level.
    // We invoke a verb that uses markdown in its verbAbout and check the output.
    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "--help"])
        .output()
        .expect("run help");

    let text = String::from_utf8_lossy(&output.stdout);

    // Should contain help text with no hash symbols at line start (ATX heading stripped)
    for line in text.lines() {
        assert!(
            !line.starts_with('#'),
            "help output must not start a line with '#' (raw ATX heading): {:?}",
            line
        );
    }
}

// ─── Feature 5.2: Auto-Generated Examples ───────────────────────────────────

#[test]
fn test_generated_examples_exist() {
    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples");

    let scripts: Vec<_> = fs::read_dir(&examples_dir)
        .expect("read examples dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("ex_")
                && e.path().extension().map_or(false, |ext| ext == "sh")
        })
        .collect();

    assert!(
        !scripts.is_empty(),
        "no auto-generated example scripts found in examples/ (expected ex_*.sh)"
    );
}

#[test]
fn test_generated_examples_are_executable() {
    use std::os::unix::fs::PermissionsExt;

    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples");

    let scripts: Vec<_> = fs::read_dir(&examples_dir)
        .expect("read examples dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("ex_")
        })
        .collect();

    for entry in scripts {
        let meta = fs::metadata(entry.path()).expect("metadata");
        let mode = meta.permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "example script is not executable: {:?}",
            entry.path()
        );
    }
}

#[test]
fn test_generated_example_emit_assemble_runs() {
    let examples_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples");

    let script = examples_dir.join("ex_emit_assemble.sh");
    if !script.exists() {
        // Skip if this specific example has not been generated yet
        eprintln!("SKIP: ex_emit_assemble.sh not found (run ggen sync first)");
        return;
    }

    let workdir = workdir();
    let status = std::process::Command::new("bash")
        .arg(&script)
        .current_dir(workdir.path())
        .status()
        .expect("run example script");

    assert!(
        status.success(),
        "example script ex_emit_assemble.sh exited non-zero: {:?}",
        status
    );
}

// ─── Feature 5.3: Command Aliases ───────────────────────────────────────────

#[test]
fn test_alias_r_routes_to_receipt() {
    // "affi r --help" should produce identical output to "affi receipt --help"
    let canonical = Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["receipt", "--help"])
        .output()
        .expect("run canonical");

    let alias = Command::cargo_bin("affi")
        .expect("affi binary")
        .args(["r", "--help"])
        .output()
        .expect("run alias");

    let canonical_text = String::from_utf8_lossy(&canonical.stdout);
    let alias_text = String::from_utf8_lossy(&alias.stdout);

    // Both should succeed
    assert!(canonical.status.success(), "canonical r --help failed");
    assert!(alias.status.success(), "alias r --help failed");

    // Help content should be equivalent (modulo alias list)
    // We check that the verb listing is present in both
    assert!(
        alias_text.contains("emit") && alias_text.contains("verify"),
        "alias help text missing expected verbs: {alias_text}"
    );
    drop(canonical_text); // suppress unused warning
}

#[test]
fn test_alias_r_v_verify() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"payload1");
    let receipt = assemble(&dir);

    // "affi r v receipt.json" should accept (exit 0)
    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["r", "v", receipt.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ACCEPT"));
}

#[test]
fn test_alias_r_e_emit() {
    let dir = workdir();
    let payload = dir.path().join("p.bin");
    fs::write(&payload, b"test-payload").unwrap();

    // "affi r e ..." should emit an event
    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args([
            "r", "e",
            "--type", "test",
            "--object", "obj:artifact",
            "--payload", payload.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event"));
}

#[test]
fn test_alias_r_a_assemble() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"build-payload");

    // "affi r a --out ..." should assemble the receipt
    let out = dir.path().join("r.json");
    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["r", "a", "--out", out.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt"));

    assert!(out.exists(), "receipt file not created by 'affi r a'");
}

#[test]
fn test_alias_r_s_show() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"payload");
    let receipt = assemble(&dir);

    Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["r", "s", receipt.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("build"));
}

// ─── Feature 5.4: JSON Output ───────────────────────────────────────────────

#[cfg(feature = "json-output")]
#[test]
fn test_verify_json_output_shape() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"build");
    let receipt = assemble(&dir);

    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["receipt", "verify", receipt.to_str().unwrap(), "--format=json"])
        .output()
        .expect("run verify --format=json");

    assert!(output.status.success(), "verify JSON exited non-zero");

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse verify JSON output");

    assert!(json["accepted"].is_boolean(), "accepted must be boolean");
    assert!(json["reason"].is_string(), "reason must be string");
    assert!(json["profile"].is_string(), "profile must be string");
    assert!(json["outcomes"].is_array(), "outcomes must be array");

    let outcomes = json["outcomes"].as_array().unwrap();
    assert_eq!(outcomes.len(), 6, "expected 6 stage outcomes (stages 1-6)");

    for outcome in outcomes {
        assert!(outcome["stage"].is_string(), "outcome.stage must be string");
        assert!(outcome["passed"].is_boolean(), "outcome.passed must be boolean");
        assert!(outcome["detail"].is_string(), "outcome.detail must be string");
    }
}

#[cfg(feature = "json-output")]
#[test]
fn test_verify_json_reject_exit_code() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"original");
    let receipt = assemble(&dir);

    // Tamper the receipt
    let content = fs::read_to_string(&receipt).unwrap();
    let tampered = content.replace("\"build\"", "\"tampered\"");
    let tampered_path = dir.path().join("tampered.json");
    fs::write(&tampered_path, tampered).unwrap();

    // JSON output must still exit 2 on REJECT
    let status = Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["receipt", "verify", tampered_path.to_str().unwrap(), "--format=json"])
        .status()
        .expect("run verify on tampered");

    assert_eq!(
        status.code(),
        Some(2),
        "REJECT must exit with code 2, even with --format=json"
    );
}

#[cfg(feature = "json-output")]
#[test]
fn test_show_json_output_shape() {
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"build");
    emit_event(&dir, "test", "suite:unit", b"test");
    let receipt = assemble(&dir);

    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["receipt", "show", receipt.to_str().unwrap(), "--format=json"])
        .output()
        .expect("run show --format=json");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse show JSON");

    assert!(json["format_version"].is_string());
    assert!(json["chain_hash"].is_string());
    let events = json["events"].as_array().expect("events must be array");
    assert_eq!(events.len(), 2, "expected 2 events");

    let event0 = &events[0];
    assert_eq!(event0["seq"], 0u64);
    assert_eq!(event0["event_type"], "build");
    assert!(event0["payload_commitment"].is_string());
}

#[cfg(feature = "json-output")]
#[test]
fn test_emit_json_output_shape() {
    let dir = workdir();
    let payload = dir.path().join("p.bin");
    fs::write(&payload, b"payload").unwrap();

    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args([
            "receipt", "emit",
            "--type", "build",
            "--object", "repo:git",
            "--payload", payload.to_str().unwrap(),
            "--format=json",
        ])
        .output()
        .expect("run emit --format=json");

    assert!(output.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emit JSON");

    assert!(json["event_id"].is_string());
    assert_eq!(json["seq"], 0u64);
    assert_eq!(json["event_type"], "build");
    assert!(json["commitment"].is_string());
    // commitment must be a 64-char lowercase hex string
    let commitment = json["commitment"].as_str().unwrap();
    assert_eq!(commitment.len(), 64, "commitment must be 64 hex chars");
    assert!(
        commitment.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "commitment must be lowercase hex"
    );
}

#[test]
fn test_format_text_is_default() {
    // Without --format=json, output should be the existing human-readable text
    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"payload");
    let receipt = assemble(&dir);

    let output = Command::cargo_bin("affi")
        .expect("affi binary")
        .current_dir(dir.path())
        .args(["receipt", "verify", receipt.to_str().unwrap()])
        .output()
        .expect("run verify without --format");

    // Should NOT be a JSON object (should be human text)
    let text = String::from_utf8_lossy(&output.stdout);
    let is_json = text.trim().starts_with('{');
    assert!(
        !is_json,
        "default output must be text, not JSON: {text}"
    );
    assert!(
        text.contains("ACCEPT") || text.contains("REJECT"),
        "text output must contain verdict: {text}"
    );
}

// ─── Feature 5.5: Shell REPL ─────────────────────────────────────────────────

#[test]
fn test_shell_not_available_without_feature_message() {
    // When the shell feature is absent, affi shell should print a clear message.
    // This test is always run; behavior differs by feature flag.
    #[cfg(not(feature = "shell"))]
    {
        Command::cargo_bin("affi")
            .expect("affi binary")
            .args(["shell"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("shell").and(predicate::str::contains("feature")));
    }

    #[cfg(feature = "shell")]
    {
        // When shell IS available, "affi shell --help" should succeed
        Command::cargo_bin("affi")
            .expect("affi binary")
            .args(["shell", "--help"])
            .assert()
            .success();
    }
}

#[cfg(feature = "shell")]
#[test]
fn test_shell_quit_command() {
    // Feed "quit\n" to the shell and expect clean exit
    use std::io::Write;

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_affi"))
        .arg("shell")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn affi shell");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"quit\n")
        .expect("write quit");

    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "shell quit must exit 0: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "shell")]
#[test]
fn test_shell_load_and_inspect() {
    use std::io::Write;

    let dir = workdir();
    emit_event(&dir, "build", "repo:git", b"build");
    let receipt = assemble(&dir);

    let commands = format!(
        "load {}\ninspect\nquit\n",
        receipt.to_str().unwrap()
    );

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_affi"))
        .arg("shell")
        .current_dir(dir.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn affi shell");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(commands.as_bytes())
        .expect("write commands");

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "shell exited non-zero: stdout={stdout}"
    );
    assert!(
        stdout.contains("loaded") || stdout.contains("1 event"),
        "shell must confirm load: {stdout}"
    );
    assert!(
        stdout.contains("build") || stdout.contains("event_type"),
        "inspect must show event type: {stdout}"
    );
}

#[cfg(feature = "shell")]
#[test]
fn test_shell_unknown_command_does_not_exit() {
    use std::io::Write;

    let commands = b"frobnicate\nquit\n";

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_affi"))
        .arg("shell")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn affi shell");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(commands)
        .expect("write commands");

    let output = child.wait_with_output().expect("wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must exit 0 (quit was accepted after unknown command)
    assert!(
        output.status.success(),
        "shell must exit 0 after unknown command + quit: stderr={stderr} stdout={stdout}"
    );

    // Must print some "unknown command" message
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("unknown") || combined.contains("frobnicate"),
        "shell must report unknown command: {combined}"
    );
}

#[cfg(feature = "shell")]
#[test]
fn test_shell_help_command() {
    use std::io::Write;

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_affi"))
        .arg("shell")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn affi shell");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"help\nquit\n")
        .expect("write commands");

    let output = child.wait_with_output().expect("wait");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    // Help output must mention all load-bearing commands
    for cmd in &["load", "inspect", "verify", "diff", "quit"] {
        assert!(
            stdout.contains(cmd),
            "help output must mention command '{}': {stdout}",
            cmd
        );
    }
}

#[cfg(feature = "shell")]
#[test]
fn test_shell_inspect_without_load_prints_guidance() {
    use std::io::Write;

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_affi"))
        .arg("shell")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn affi shell");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"inspect\nquit\n")
        .expect("write commands");

    let output = child.wait_with_output().expect("wait");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Must exit 0 (graceful error, not panic)
    assert!(output.status.success());

    // Must tell user to load first
    assert!(
        combined.contains("load") || combined.contains("no receipt"),
        "must guide user to load: {combined}"
    );
}
```

---

## JSON Schema Reference

### `Verdict` (verify --format=json)

```json
{
  "accepted": true,
  "profile": "core/v1",
  "reason": "all stages passed",
  "outcomes": [
    {
      "stage": "decode",
      "passed": true,
      "detail": "2 event(s), format_version present"
    },
    {
      "stage": "check_format",
      "passed": true,
      "detail": "format_version == core/v1"
    },
    {
      "stage": "chain_integrity",
      "passed": true,
      "detail": "chain hash matches recomputed value"
    },
    {
      "stage": "continuity",
      "passed": true,
      "detail": "seq 0..1 contiguous, 2 unique ids"
    },
    {
      "stage": "verify_commitments",
      "passed": true,
      "detail": "all 2 commitments are well-formed BLAKE3 digests"
    },
    {
      "stage": "evaluate_profile",
      "passed": true,
      "detail": "core/v1: all 2 events have event_type and commitment"
    }
  ]
}
```

**Field Constraints**

| Field | Type | Constraint |
|-------|------|-----------|
| `accepted` | boolean | true iff all outcomes have `passed: true` |
| `profile` | string | one of: `"core/v1"` |
| `reason` | string | `"all stages passed"` on ACCEPT; first failure detail on REJECT |
| `outcomes` | array | exactly 6 elements, in pipeline order |
| `outcomes[].stage` | string | one of: decode, check_format, chain_integrity, continuity, verify_commitments, evaluate_profile |
| `outcomes[].passed` | boolean | required |
| `outcomes[].detail` | string | non-empty, human-readable, no newlines |

### `InspectionReport` (inspect --format=json)

```json
{
  "event_count": 3,
  "format_version": "core/v1",
  "chain_hash": "af3e9b1c...",
  "chain_integrity_valid": true,
  "event_types": {
    "build": 1,
    "test": 1,
    "audit": 1
  },
  "object_types": {
    "git": 2,
    "test-suite": 1
  },
  "events": [
    {
      "seq": 0,
      "id": "evt-0",
      "event_type": "build",
      "object_count": 1,
      "commitment": "6ef47c82..."
    }
  ]
}
```

**Field Constraints**

| Field | Type | Constraint |
|-------|------|-----------|
| `event_count` | integer | >= 0 |
| `format_version` | string | same as `Receipt.format_version` |
| `chain_hash` | string | 64 lowercase hex chars |
| `chain_integrity_valid` | boolean | true iff chain hash recomputes correctly |
| `event_types` | object | keys: event_type strings; values: integer counts |
| `object_types` | object | keys: object type strings; values: integer counts |
| `events` | array | summary rows (not full events) |

### `DiffResult` (diff --format=json)

```json
{
  "event_count_a": 3,
  "event_count_b": 4,
  "added": [
    {
      "seq": 3,
      "id": "evt-3",
      "event_type": "sign",
      "objects": [{"id": "key", "obj_type": "signing-key"}],
      "payload_commitment": "a2d95f..."
    }
  ],
  "removed": [],
  "modified": [
    {
      "seq": 1,
      "id_a": "evt-1",
      "id_b": "evt-1",
      "event_type_changed": false,
      "commitment_changed": true,
      "objects_changed": false
    }
  ]
}
```

**Field Constraints**

| Field | Type | Constraint |
|-------|------|-----------|
| `event_count_a` | integer | event count in receipt A |
| `event_count_b` | integer | event count in receipt B |
| `added` | array | full OperationEvent objects present in B but not A |
| `removed` | array | full OperationEvent objects present in A but not B |
| `modified` | array | events with same seq but differing fields |

### `EmitOutput` (emit --format=json)

```json
{
  "event_id": "evt-0",
  "seq": 0,
  "event_type": "build",
  "commitment": "6ef47c8291a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8"
}
```

### `AssembleOutput` (assemble --format=json)

```json
{
  "receipt_path": "/tmp/work/receipt.json",
  "content_address": "203d3bbf9e4a7c1d...",
  "event_count": 2
}
```

### `StatsOutput` (stats --format=json)

```json
{
  "event_count": 5,
  "chain_depth": 5,
  "chain_hash": "af3e9b...",
  "event_type_histogram": {
    "build": 2,
    "test": 2,
    "audit": 1
  },
  "object_type_histogram": {
    "git": 3,
    "test-suite": 2
  }
}
```

---

## REPL Command Reference

The following table is the authoritative reference for all `affi shell` commands.
It must be printed verbatim (minus table formatting) when `help` is invoked.

```
COMMAND     SYNTAX                     DESCRIPTION
─────────────────────────────────────────────────────────────────────────────
load        load <path>                Load receipt from <path> into memory.
                                       Prints: "loaded: <path> (N events)"

inspect     inspect                    Inspect the loaded receipt.
                                       Prints event_types, object_types,
                                       chain validity, event count.

verify      verify                     Run the 7-stage certify pipeline.
                                       Prints per-stage outcomes + verdict.

show        show                       Print human-readable receipt dump.
                                       Lists events: seq, type, objects.

diff        diff <path>                Diff the loaded receipt vs <path>.
                                       Prints: added, removed, modified.

stats       stats                      Print chain metrics.
                                       Prints: event count, chain depth,
                                       hash distribution, type histograms.

graph       graph [--format=dot|json]  Visualize event DAG.
                                       Default format: json.

mutate      mutate <N>                 Apply N random mutations and verify each.
                                       Prints: killed M/N mutations (kill ratio).

trace       trace                      Emit OTel spans for verifier stages.
                                       Requires the otel feature.

reload      reload                     Reload receipt from last-used path.

path        path                       Print path of loaded receipt.

clear       clear                      Unload receipt from memory.

help        help [command]             Print this reference, or help for <command>.

quit        quit | exit                Exit the shell (code 0).
─────────────────────────────────────────────────────────────────────────────
Tip: Use Tab for command completion. Use Up/Down arrows for history.
```

---

## Alias Table

Complete alias specification. This table is the single source of truth and must
be reflected identically in `ontology/affi-cli.ttl` via `cnv:hasAlias` triples.

| Canonical Command | Single-Char Alias | Notes |
|------------------|-------------------|-------|
| `affi receipt` | `affi r` | Noun alias; all verbs work under `r` |
| `affi receipt emit` | `affi r e` | High-frequency; emit is the entry point |
| `affi receipt assemble` | `affi r a` | High-frequency; assembly finalizes the chain |
| `affi receipt verify` | `affi r v` | High-frequency; verify is the exit condition |
| `affi receipt show` | `affi r s` | High-frequency; non-adjudicating display |
| `affi receipt inspect` | `affi r i` | High-frequency in DX workflows |
| `affi receipt diff` | `affi r d` | High-frequency in comparison workflows |
| `affi receipt stats` | (none) | Low-frequency; no single-char alias assigned |
| `affi receipt graph` | (none) | Low-frequency; `g` would conflict with git conventions |
| `affi receipt replay` | (none) | Low-frequency; re-execution is intentional, not casual |
| `affi receipt model` | (none) | Low-frequency; process mining is a deliberate operation |
| `affi receipt conformance` | (none) | Low-frequency; same reasoning as model |
| `affi receipt diagnose` | (none) | Low-frequency; troubleshooting is explicit |

**Alias Conflict Policy**

- Single-char aliases must not conflict with any other alias at the same level
- The letters e, a, v, s, i, d are reserved for the verbs above
- No future verb should be assigned these single-char aliases without updating this table
- Long-form aliases (e.g., `affi r emit` vs `affi receipt emit`) are always available
  because the noun alias (`r`) passes through to the same verb dispatch

---

## Completion Spec

### Bash Completion

Generated by `affi --generate=bash` or `affi receipt generate-completion --shell=bash`.

```bash
# Auto-generated by affi from ontology/affi-cli.ttl
_affi_completion() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        affi)
            COMPREPLY=($(compgen -W "receipt r shell help" -- "$cur"))
            ;;
        receipt|r)
            COMPREPLY=($(compgen -W "emit e assemble a verify v show s inspect i diff d stats graph replay model conformance diagnose" -- "$cur"))
            ;;
        emit|e)
            COMPREPLY=($(compgen -W "--type --object --payload --format" -- "$cur"))
            ;;
        assemble|a)
            COMPREPLY=($(compgen -W "--out --format" -- "$cur"))
            ;;
        verify|v)
            COMPREPLY=($(compgen -W "--format --profile --strict" -- "$cur"))
            ;;
        show|s|inspect|i|diff|d|stats|graph)
            COMPREPLY=($(compgen -W "--format" -- "$cur"))
            # Complete receipt file paths
            COMPREPLY+=($(compgen -f -X '!*.json' -- "$cur"))
            ;;
        --format)
            COMPREPLY=($(compgen -W "text json" -- "$cur"))
            ;;
        --profile)
            COMPREPLY=($(compgen -W "core/v1" -- "$cur"))
            ;;
    esac
}
complete -F _affi_completion affi
```

### Zsh Completion

```zsh
# Auto-generated by affi from ontology/affi-cli.ttl
#compdef affi

_affi() {
    local -a commands
    _arguments \
        '1: :->nouns' \
        '*: :->args'

    case $state in
        nouns)
            commands=('receipt:Manage provenance receipts' 'r:Alias for receipt')
            _describe 'nouns' commands
            ;;
        args)
            case $words[2] in
                receipt|r)
                    local -a verbs
                    verbs=(
                        'emit:Append an operation-event'
                        'e:Alias for emit'
                        'assemble:Finalize the working receipt'
                        'a:Alias for assemble'
                        'verify:Run the certify pipeline'
                        'v:Alias for verify'
                        'show:Print a human-readable dump'
                        's:Alias for show'
                        'inspect:Detailed receipt analysis'
                        'i:Alias for inspect'
                        'diff:Compare two receipts'
                        'd:Alias for diff'
                        'stats:Chain metrics'
                        'graph:DAG visualization'
                    )
                    _describe 'verbs' verbs
                    ;;
            esac
            ;;
    esac
}

_affi
```

### Fish Completion

```fish
# Auto-generated by affi from ontology/affi-cli.ttl
complete -c affi -n "__fish_use_subcommand" -a "receipt" -d "Manage provenance receipts"
complete -c affi -n "__fish_use_subcommand" -a "r" -d "Alias for receipt"

complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "emit e" -d "Append an operation-event"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "assemble a" -d "Finalize the working receipt"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "verify v" -d "Run the certify pipeline"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "show s" -d "Print human-readable dump"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "inspect i" -d "Detailed receipt analysis"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "diff d" -d "Compare two receipts"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "stats" -d "Chain metrics"
complete -c affi -n "__fish_seen_subcommand_from receipt r" -a "graph" -d "DAG visualization"

complete -c affi -n "__fish_seen_subcommand_from emit e" -l type -d "Operation-event type" -r
complete -c affi -n "__fish_seen_subcommand_from emit e" -l object -d "Object ref (id:type[:qualifier])" -r
complete -c affi -n "__fish_seen_subcommand_from emit e" -l payload -d "Payload source (file or -)" -r -F
complete -c affi -n "__fish_seen_subcommand_from emit e" -l format -d "Output format" -r -a "text json"

complete -c affi -n "__fish_seen_subcommand_from verify v" -l format -r -a "text json"
complete -c affi -n "__fish_seen_subcommand_from verify v" -l profile -r -a "core/v1"
complete -c affi -n "__fish_seen_subcommand_from verify v" -l strict -d "Treat warnings as failures"
```

**Completion Test**

```bash
# Test that bash completion script is valid syntax
source <(affi receipt generate-completion --shell=bash 2>/dev/null || true)
# Expected: no error; _affi_completion is defined

# Test that zsh completion script is valid syntax
zsh -c "source <(affi receipt generate-completion --shell=zsh 2>/dev/null || true)"
# Expected: exit 0
```

---

## Example Auto-Gen Pipeline Diagram

```
chicago-tdd-tools
     │
     │  fixture metadata (name, events, objects)
     ▼
ggen.toml [examples] section
     │
     │  SELECT ?fixture WHERE { ?f ct:hasTag "phase5-approved" }
     ▼
templates/example_script.sh.tera
     │
     │  render: Tera template engine substitutes fixture vars
     ▼
examples/ex_<fixture_name>.sh
     │
     ├── set -euo pipefail
     ├── mktemp -d (WORKDIR)
     ├── [for each fixture event]
     │       affi receipt emit --type ... --object ... --payload -
     ├── affi receipt assemble --out receipt.json
     ├── affi receipt verify receipt.json
     └── assertions (file exists, hash pattern, exit 0)
     │
     ▼
CI: cargo test -- example_scripts_execute
     │
     ├── discover examples/ex_*.sh
     ├── for each script:
     │       bash script.sh  (in fresh tmpdir)
     │       assert exit 0
     │       capture stderr; fail loudly on error
     └── report: "N/N example scripts passed"
```

**ggen.toml template directive (example)**

```toml
# ggen.toml additions for Phase 5.2

[[packs]]
# existing pack entry ...

[[examples]]
fixture_source  = "chicago-tdd-tools"
output_dir      = "examples"
filename_prefix = "ex_"
filename_suffix = ".sh"
template        = "ggen-templates/example_script.sh.tera"
chmod_execute   = true

[examples.filter]
tags = ["phase5-approved"]

[examples.vars]
affi_binary = "affi"
```

---

## Phase 5 Exit Criteria

All of the following must be true before Phase 5 is marked complete and the
`claude/zen-cerf-oq87br` branch is eligible for merge.

### Mandatory Checks (blocking)

- [ ] `cargo build` passes with no errors and no warnings (deny(warnings) in lib.rs)
- [ ] `cargo build --features shell,json-output,otel` passes
- [ ] `cargo test` passes (all unit + integration + e2e tests)
- [ ] `cargo test --features shell,json-output` passes (includes feature-gated E2E tests)
- [ ] `cargo test --test e2e_cli` passes (all tests in the E2E template above)
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` reports zero issues
- [ ] `cargo fmt --check` reports no formatting differences

### Feature-Specific Checks (blocking)

**5.1 Help Formatter**
- [ ] `affi receipt verify --help` output has no line > 80 characters
- [ ] `affi receipt verify --help` output contains no Markdown syntax (no `**`, no `` ` ``, no `#` at line start)
- [ ] `affi receipt verify --help` output contains "See also: ARDPRD FR-3"
- [ ] `affi receipt emit --help` contains "See also: ARDPRD FR-1"
- [ ] `affi receipt assemble --help` contains "See also: ARDPRD FR-2"
- [ ] `format_help_markdown` has at least 5 unit tests covering each transformation

**5.2 Auto-Generated Examples**
- [ ] At least 3 `examples/ex_*.sh` files exist and are committed to git
- [ ] All `examples/ex_*.sh` files have execute permission (mode & 0o111 != 0)
- [ ] Each example script runs to exit 0 in a fresh temporary directory
- [ ] No example script contains hardcoded BLAKE3 hashes
- [ ] `cargo test -- example_scripts_execute` passes
- [ ] ggen.toml contains `[[examples]]` section with template path

**5.3 Command Aliases**
- [ ] `affi r --help` exits 0 and lists available verbs
- [ ] `affi r v <receipt>` produces identical output to `affi receipt verify <receipt>`
- [ ] `affi r e ...` produces identical output to `affi receipt emit ...`
- [ ] `affi r a` produces identical output to `affi receipt assemble`
- [ ] `affi r s <receipt>` produces identical output to `affi receipt show <receipt>`
- [ ] All aliases are declared in `ontology/affi-cli.ttl` as `cnv:hasAlias` triples
- [ ] Alias table above is reflected 1:1 in the ontology TTL

**5.4 JSON Output**
- [ ] `--format=json` flag is accepted by: emit, assemble, verify, show, inspect, diff, stats
- [ ] `verify --format=json` produces JSON parseable by `jq`
- [ ] `verify --format=json` on REJECT still exits with code 2
- [ ] `show --format=json` produces JSON with `format_version`, `events`, `chain_hash`
- [ ] `emit --format=json` produces JSON with `event_id`, `seq`, `event_type`, `commitment`
- [ ] `assemble --format=json` produces JSON with `receipt_path`, `content_address`, `event_count`
- [ ] Without `--features json-output`, `--format=json` produces a clear error (not a crash)
- [ ] `Verdict::to_json()`, `InspectionReport::to_json()`, `DiffResult::to_json()` are gated on `#[cfg(feature = "json-output")]`
- [ ] Default format (no `--format` flag) is text, unchanged from pre-Phase-5 behavior

**5.5 Interactive Shell REPL**
- [ ] `affi shell` without `--features shell` exits non-zero with rebuild guidance
- [ ] With `--features shell`: `affi shell` starts and shows `affi> ` prompt
- [ ] `load <path>` command loads a receipt and prints event count
- [ ] `inspect` after load prints InspectionReport
- [ ] `verify` after load prints certify pipeline stages and verdict
- [ ] `diff <path>` prints DiffResult comparing loaded vs `<path>`
- [ ] `help` prints the REPL command reference table
- [ ] `quit` exits with code 0
- [ ] Unknown command prints guidance and does not exit the shell
- [ ] `inspect` without load prints "no receipt loaded" guidance
- [ ] With rustyline: Up-arrow recalls history (integration test via pty)
- [ ] With rustyline: Tab completes command names

### Non-Blocking Checks (advisory)

- [ ] `cargo doc --all-features --no-deps` generates without broken links
- [ ] All public functions in `src/cli.rs` have `///` doc comments
- [ ] `src/bin/affi-shell.rs` has a module-level `//!` doc comment explaining the REPL
- [ ] Fish/Zsh/Bash completion scripts are syntactically valid (tested via shell)
- [ ] REPL history file is written to `~/.affi_shell_history` (persistent across sessions)
- [ ] `ggen sync` runs without errors on the updated `ontology/affi-cli.ttl`

### ARDPRD Traceability

| Phase 5 Feature | ARDPRD Requirement | Satisfied By |
|----------------|-------------------|--------------|
| Help Formatter | NFR-6 (Witnessed surface) | Help text and ARDPRD refs are testable witnesses |
| Auto-Generated Examples | NFR-6 (Witnessed surface) | Each example script is a runnable witness |
| Command Aliases | FR-5 (CLI surface) | Aliases expose canonical verbs via alternate names |
| JSON Output | FR-3, FR-4 (Verification, Inspection) | Machine-readable output enables composable pipelines |
| Interactive Shell REPL | FR-4, FR-5 (Inspection, CLI surface) | REPL is a stateful inspection surface over receipts |

---

*This document is the Definition of Done for Phase 5 of the DX/QOL 1000x initiative.
All checkboxes above are required for merge unless marked non-blocking.
Branch: `claude/zen-cerf-oq87br`. Maintained by: Sean Chatman <xpointsh@gmail.com>.*
