# Definition of Done — Phase 1: Receipt Inspection
## affidavit DX/QOL 1000x Initiative

**Branch:** `claude/zen-cerf-oq87br`  
**Version:** 26.6.14  
**Phase:** 1 of 6  
**Theme:** Receipt Inspection & Visualization  
**Author:** Sean Chatman <xpointsh@gmail.com>  
**Created:** 2026-06-14  

---

## Phase 1 Overview

Phase 1 delivers the five foundational DX features in the Receipt Inspection cluster. All feature handlers live in `src/handlers.rs`. Each feature is 80% reuse from `chicago-tdd-tools` + `ggen` (already integrated) and 20% new glue code. No feature ships until every item in its DoD section is checked.

**Features:**

| # | Command | Handler | Status |
|---|---------|---------|--------|
| F1 | `affi receipt inspect` | `handlers::inspect` | 🔲 Not started |
| F2 | `affi receipt diff` | `handlers::diff_receipts` (new) | 🔲 Not started |
| F3 | `affi receipt visualize --format={dot,json}` | `handlers::visualize` (new) | 🔲 Not started |
| F4 | `affi receipt catalog` | `handlers::catalog` (new) | 🔲 Not started |
| F5 | Shell completion (bash/zsh/fish) | `clap-noun-verb` codegen | 🔲 Not started |

**Quality Gate (must pass before Phase 2):**

```
cargo test --all           → 0 failures, 0 ignored-for-CI
cargo clippy -- -D warnings  → 0 warnings
cargo fmt --check          → clean
```

---

## Feature 1: `affi receipt inspect`

### 1.1 Acceptance Criteria

> Detailed event/object type distribution report for any valid receipt.

| # | Given | When | Then | Pass/Fail |
|---|-------|------|------|-----------|
| AC-1 | A 3-event receipt with types `create`, `transform`, `release` | `affi receipt inspect r.json` is run | stderr contains `RECEIPT INSPECTION REPORT` header | 🔲 |
| AC-2 | A 3-event receipt with types `create`, `transform`, `release` | `affi receipt inspect r.json` is run | stderr contains each event type with its count (e.g., `create: 1 events`) | 🔲 |
| AC-3 | A receipt with `f:artifact` and `d:artifact` objects | `affi receipt inspect r.json` is run | stderr contains `artifact:` with a reference count ≥ 1 | 🔲 |
| AC-4 | A receipt with 0 object refs in any event | `affi receipt inspect r.json` is run | command exits 0, section `Object types:` is omitted or shows `(none)` | 🔲 |
| AC-5 | A receipt with duplicate event types (e.g., two `build` events) | `affi receipt inspect r.json` is run | the count for `build` is `2 events`, not `1 events` | 🔲 |
| AC-6 | A non-existent file path | `affi receipt inspect missing.json` is run | command exits non-zero; stderr contains a user-readable error mentioning the file | 🔲 |
| AC-7 | A tampered receipt file (chain hash mismatch) | `affi receipt inspect tampered.json` is run | command exits non-zero; stderr mentions `chain hash mismatch` | 🔲 |
| AC-8 | A valid 10-event receipt | `affi receipt inspect r.json` is run | `Total events: 10` appears in output | 🔲 |
| AC-9 | Any valid receipt | `affi receipt inspect r.json` is run | `Format: core/v1` and `Chain hash:` both appear in output | 🔲 |
| AC-10 | Any valid receipt | `affi receipt inspect r.json` is run | the `Inspection complete.` line appears at the end of the report | 🔲 |

### 1.2 Implementation Checklist

#### Ontology

- [ ] `ontology/affi-cli.ttl` declares `affi:inspect` as a `cnv:Verb` under the `receipt` noun
- [ ] `affi:inspect` has `cnv:hasVerbName "inspect"`
- [ ] `affi:inspect` has one positional argument `receipt` (type `xsd:string`, index 1)
- [ ] `ggen sync` passes all validation rules without error (`no-orphan-verb`, `no-flag-field-collision`, etc.)

#### Handler (`src/handlers.rs`)

- [ ] `pub fn inspect(receipt: String) -> Result<()>` exists and is the canonical entry point
- [ ] Handler calls `adapt(affidavit::cli::show(&receipt))?` to parse — no duplicate parsing logic
- [ ] Handler calls `crate::verbs::inspect_with_fixtures(&parsed)` for report generation
- [ ] Output is routed to `eprint!` / `eprintln!` (stderr), not `println!` (stdout) — per §6 guard
- [ ] Handler propagates file-not-found as a human-readable `NounVerbError::execution_error`
- [ ] Handler propagates chain-hash-mismatch (from `show` → deserialization) as a non-zero exit

#### Report Generator (`src/verbs/mod.rs` — `inspect_with_fixtures`)

- [ ] Output begins with `RECEIPT INSPECTION REPORT\n=========================\n`
- [ ] `Format: {format_version}` line present
- [ ] `Total events: {n}` line present
- [ ] `Chain hash: {hash}` line present
- [ ] `Event types:` section lists each unique type with count (`{type}: {n} events`)
- [ ] Event types are sorted deterministically (alphabetical or by first-occurrence order — pick one and document it)
- [ ] `Object types:` section lists each unique object `obj_type` with ref count
- [ ] Object types section is omitted entirely when no event has any object ref
- [ ] `Inspection complete.` terminator line
- [ ] Function is `pub fn inspect_with_fixtures(receipt: &Receipt) -> String`
- [ ] No `unwrap()` calls in non-test code paths

#### Verb Wrapper (`src/verbs/inspect.rs`)

- [ ] File generated by ggen OR hand-authored with identical shape to ggen output
- [ ] `#[verb("inspect", "receipt")]` attribute applied
- [ ] Delegates exclusively to `crate::handlers::inspect(receipt)`
- [ ] No business logic in wrapper

#### Verb Registration

- [ ] `pub mod inspect;` present in `src/verbs/mod.rs`
- [ ] `affi receipt inspect --help` prints the help text without error

### 1.3 Test Evidence Required

**Unit tests** (inline in `src/verbs/mod.rs`):

| Test name | File | What it checks |
|-----------|------|----------------|
| `inspect_generates_detailed_report` | `src/verbs/mod.rs` | 2-event receipt → report has event types + object types |
| `inspect_empty_objects_omits_object_section` | `src/verbs/mod.rs` | receipt with objectless events → `Object types:` section absent |
| `inspect_counts_duplicate_types` | `src/verbs/mod.rs` | two events with same type → count is 2 |
| `inspect_10_event_receipt` | `src/verbs/mod.rs` | 10-event receipt → `Total events: 10` in output |

**E2E tests** (`tests/dx_verbs_e2e.rs` — already partially exists):

| Test name | File | Expected output |
|-----------|------|-----------------|
| `inspect_verb_reports_event_distribution` | `tests/dx_verbs_e2e.rs` | `RECEIPT INSPECTION REPORT`, `create: 1 events`, `artifact:` |
| `inspect_rejects_missing_file` | `tests/e2e_inspection.rs` | exit non-zero, stderr contains file name |
| `inspect_rejects_tampered_receipt` | `tests/e2e_inspection.rs` | exit non-zero, `chain hash mismatch` in stderr |

**Minimum required command to pass:**
```bash
cargo test inspect -- --nocapture
# Expected: 7+ tests, 0 failures
```

### 1.4 Exit Criteria

Before marking F1 complete, ALL of the following must be true:

- [ ] All 10 acceptance criteria pass
- [ ] All 4 unit tests pass
- [ ] All 3 E2E tests pass
- [ ] `cargo clippy -- -D warnings` passes with no warnings introduced by F1 code
- [ ] `cargo fmt --check` passes
- [ ] `affi receipt inspect --help` shows meaningful help text
- [ ] The existing test `inspect_generates_detailed_report` (already in codebase) continues to pass
- [ ] The `inspect_verb_reports_event_distribution` test in `tests/dx_verbs_e2e.rs` passes

### 1.5 Reviewer Sign-off

| Role | Reviewer | Signs off on |
|------|----------|--------------|
| **Author** | Sean Chatman | All code, all tests written |
| **Architecture** | — | Handler→CLI delegation chain; no business logic leak into verb wrapper |
| **Test** | — | Unit tests are genuinely failing-when-fake (not tautological) |
| **Integration** | — | ggen sync passes; verb appears in `affi receipt --help` |

### 1.6 Rollback Criteria

Revert F1 (remove `src/verbs/inspect.rs`, revert `src/handlers.rs` and `src/verbs/mod.rs`) if:

- `cargo test --all` introduces ≥1 new failure in previously-passing tests
- The handler leaks non-stderr output (violates §6 stdout guard)
- Deserialization of a tampered receipt silently returns a partial report instead of erroring
- ggen sync fails after ontology update

Rollback command:
```bash
git revert HEAD --no-commit  # or git checkout main -- src/verbs/inspect.rs src/handlers.rs
```

### 1.7 Documentation Requirements

- [ ] `FEATURES_DX_QOL.md` updated: `inspect` row shows "✅ implemented, tests passing"
- [ ] `CLAUDE.md` verbs table includes `inspect` with its description
- [ ] `examples/golden_run.sh` or a new `examples/inspection.sh` demonstrates `affi receipt inspect`
- [ ] Inline rustdoc on `inspect_with_fixtures` documents the output format

### 1.8 Performance Benchmarks

| Operation | Receipt size | Target | Measurement method |
|-----------|-------------|--------|-------------------|
| `affi receipt inspect` wall time | 1 event | < 50 ms | `hyperfine --warmup 3 'affi receipt inspect r.json'` |
| `affi receipt inspect` wall time | 10 events | < 100 ms | hyperfine |
| `affi receipt inspect` wall time | 100 events | < 200 ms | hyperfine |
| `inspect_with_fixtures` in-process | 100 events | < 5 ms | `cargo bench --bench receipt_operations` |
| Binary startup overhead | n/a | < 20 ms | measured in bench harness |

Bench target name: `bench_inspect_100_events` in `benches/receipt_operations.rs`.

---

## Feature 2: `affi receipt diff`

### 2.1 Acceptance Criteria

> Compare two receipts by event sequence and commitment changes.

| # | Given | When | Then | Pass/Fail |
|---|-------|------|------|-----------|
| AC-1 | Two identical receipts `a.json` and `b.json` | `affi receipt diff a.json b.json` | stderr contains `No differences found` or equivalent | 🔲 |
| AC-2 | Receipt `b.json` has one extra event appended vs `a.json` | `affi receipt diff a.json b.json` | stderr reports `1 added` event with its seq and type | 🔲 |
| AC-3 | Receipt `b.json` is `a.json` minus its last event | `affi receipt diff a.json b.json` | stderr reports `1 removed` event | 🔲 |
| AC-4 | `a.json` and `b.json` have same seq numbers but different `event_type` at seq 1 | `affi receipt diff a.json b.json` | stderr reports `1 modified` event at seq 1 with old and new type | 🔲 |
| AC-5 | `a.json` and `b.json` have same seq/type but different commitments at seq 0 | `affi receipt diff a.json b.json` | stderr reports `1 modified` event showing commitment change | 🔲 |
| AC-6 | `a.json` and `b.json` differ in multiple events | `affi receipt diff a.json b.json` | all differences are listed, not just the first | 🔲 |
| AC-7 | Either file is missing | `affi receipt diff missing.json b.json` | exit non-zero, stderr names the missing file | 🔲 |
| AC-8 | Either file is a tampered receipt | `affi receipt diff tampered.json b.json` | exit non-zero, `chain hash mismatch` in stderr | 🔲 |
| AC-9 | Two receipts with 0 events each | `affi receipt diff empty_a.json empty_b.json` | exit 0, `No differences found` | 🔲 |
| AC-10 | `a.json` has 5 events, `b.json` has 3 events | `affi receipt diff a.json b.json` | stderr reports exactly 2 removed events (seq 3 and 4) | 🔲 |

### 2.2 Implementation Checklist

#### Data Structures (`src/handlers.rs` or new `src/diff.rs`)

- [ ] `pub struct DiffResult` defined with fields:
  - `added: Vec<DiffEntry>` (events in b but not a)
  - `removed: Vec<DiffEntry>` (events in a but not b)
  - `modified: Vec<ModifiedEntry>` (same seq, different content)
- [ ] `pub struct DiffEntry` has `seq: u64`, `event_type: String`, `commitment_prefix: String` (first 12 hex chars)
- [ ] `pub struct ModifiedEntry` has `seq: u64`, `old: DiffEntry`, `new: DiffEntry`
- [ ] All structs are `#[derive(Debug, Clone)]`

#### Algorithm (`src/handlers.rs::diff_receipts` or `src/diff.rs::compute_diff`)

- [ ] Takes `(a: &Receipt, b: &Receipt) -> DiffResult`
- [ ] Iterates events by seq in O(n) using sorted index
- [ ] Events matched by `seq` field (not by position in array)
- [ ] A seq present in `a` but not `b` → `removed`
- [ ] A seq present in `b` but not `a` → `added`
- [ ] A seq present in both but with different `event_type` OR different `payload_commitment` → `modified`
- [ ] Two events with same seq and identical fields → not reported

#### Handler (`src/handlers.rs`)

- [ ] `pub fn diff_receipts(receipt_a: String, receipt_b: String) -> Result<()>` exists
- [ ] Loads both receipts via `adapt(affidavit::cli::show(...))?`
- [ ] Calls diff algorithm
- [ ] Output routed to stderr
- [ ] When `DiffResult` has 0 added + 0 removed + 0 modified: prints `No differences found.`
- [ ] Summary line printed: `{n_added} added, {n_removed} removed, {n_modified} modified`
- [ ] Each added event: `+ [seq] {event_type} (commit: {short_commit})`
- [ ] Each removed event: `- [seq] {event_type} (commit: {short_commit})`
- [ ] Each modified event: `~ [seq] {old_type} → {new_type}` and/or `commit {old_prefix} → {new_prefix}`
- [ ] Exit 0 when diff completes (even if differences found — diff is informational)

#### Ontology

- [ ] `affi:diff` declared as `cnv:Verb` under `receipt` noun in `ontology/affi-cli.ttl`
- [ ] Two positional arguments: `receipt-a` (index 1) and `receipt-b` (index 2)
- [ ] `ggen sync` passes all validation rules

#### Verb Wrapper (`src/verbs/diff.rs`)

- [ ] `#[verb("diff", "receipt")]` attribute
- [ ] Takes two positional string args
- [ ] Delegates to `crate::handlers::diff_receipts(receipt_a, receipt_b)`
- [ ] `pub mod diff;` added to `src/verbs/mod.rs`

### 2.3 Test Evidence Required

**Unit tests** (`src/handlers.rs` or `tests/diff_unit.rs`):

| Test name | What it checks |
|-----------|----------------|
| `diff_identical_receipts_no_differences` | Two identical receipts → empty DiffResult |
| `diff_added_event` | b has extra event → 1 added |
| `diff_removed_event` | b missing last event → 1 removed |
| `diff_modified_event_type` | same seq, different type → 1 modified |
| `diff_modified_commitment` | same seq+type, different commitment → 1 modified |
| `diff_multiple_changes` | 3 changes → all 3 reported |
| `diff_empty_receipts` | both empty → 0 differences |

**E2E tests** (`tests/e2e_inspection.rs`):

| Test name | Expected stderr |
|-----------|-----------------|
| `diff_verb_reports_added_event` | `1 added`, `+ [3]` |
| `diff_verb_reports_no_differences` | `No differences found` |
| `diff_verb_rejects_missing_file` | exit non-zero, file name in stderr |

**Minimum required command to pass:**
```bash
cargo test diff -- --nocapture
# Expected: 10+ tests, 0 failures
```

### 2.4 Exit Criteria

- [ ] All 10 acceptance criteria pass
- [ ] All 7 unit tests pass
- [ ] All 3 E2E tests pass
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `affi receipt diff --help` shows both positional arg names and descriptions

### 2.5 Reviewer Sign-off

| Role | Reviewer | Signs off on |
|------|----------|--------------|
| **Author** | Sean Chatman | Algorithm correctness; all test cases written |
| **Architecture** | — | DiffResult types; no duplication with existing `verifier.rs` |
| **Test** | — | E2E tests use real assembled receipts, not mocked JSON |

### 2.6 Rollback Criteria

Revert F2 if:

- Diff algorithm produces false positives (reports differences on identical receipts)
- Output leaks to stdout instead of stderr
- `diff_receipts` panics on receipts with 0 events
- Any previously-passing test in `tests/e2e.rs` or `tests/dx_verbs_e2e.rs` starts failing

### 2.7 Documentation Requirements

- [ ] `FEATURES_DX_QOL.md` updated with `diff` implementation status
- [ ] Rustdoc on `DiffResult`, `DiffEntry`, `ModifiedEntry` and `diff_receipts` handler
- [ ] `examples/golden_run.sh` or `examples/diff_receipts.sh` demonstrates `affi receipt diff`
- [ ] Output format documented in `CLAUDE.md` CLI Surface section

### 2.8 Performance Benchmarks

| Operation | Receipt size | Target | Measurement method |
|-----------|-------------|--------|-------------------|
| `affi receipt diff` wall time | 10 events each | < 100 ms | hyperfine |
| `affi receipt diff` wall time | 100 events each | < 250 ms | hyperfine |
| `compute_diff` in-process | 100 events each | < 10 ms | criterion bench `bench_diff_100` |
| `compute_diff` in-process | 1000 events each | < 100 ms | criterion bench `bench_diff_1000` |

---

## Feature 3: `affi receipt visualize --format={dot,json}`

### 3.1 Acceptance Criteria

> Emit a process graph (DOT or JSON) from the receipt's event sequence.

| # | Given | When | Then | Pass/Fail |
|---|-------|------|------|-----------|
| AC-1 | A 3-event receipt with types `create`, `transform`, `release` | `affi receipt visualize --format=json r.json` | stderr (or stdout) contains a JSON object with `"nodes"` and `"edges"` keys | 🔲 |
| AC-2 | A 3-event receipt | `affi receipt visualize --format=json r.json` | `nodes` array contains at least one entry for each distinct `event_type` | 🔲 |
| AC-3 | A 3-event receipt with linear flow `create → transform → release` | `affi receipt visualize --format=json r.json` | `edges` contains entries `create→transform` and `transform→release` | 🔲 |
| AC-4 | A 3-event receipt | `affi receipt visualize --format=dot r.json` | output begins with `digraph` and contains `->` edges | 🔲 |
| AC-5 | A 3-event receipt | `affi receipt visualize --format=dot r.json` | DOT output contains each event type as a node label | 🔲 |
| AC-6 | `--format` flag is omitted | `affi receipt visualize r.json` | command exits with a user-readable error: `--format` is required | 🔲 |
| AC-7 | An invalid format value | `affi receipt visualize --format=xml r.json` | command exits non-zero with `invalid value` or similar | 🔲 |
| AC-8 | A receipt with a repeated event type (e.g., two `build` events) | `affi receipt visualize --format=json r.json` | `build` node appears once in nodes; edge `build→build` or `build→next` appears as appropriate | 🔲 |
| AC-9 | A single-event receipt | `affi receipt visualize --format=json r.json` | `nodes` has 1 entry, `edges` has 0 entries | 🔲 |
| AC-10 | A tampered receipt | `affi receipt visualize --format=json tampered.json` | exit non-zero, `chain hash mismatch` in stderr | 🔲 |

### 3.2 Implementation Checklist

#### Graph Data Structures

- [ ] `pub struct ReceiptGraph` defined (in `src/handlers.rs` or new `src/graph_builder.rs`)
- [ ] `ReceiptGraph` has `nodes: Vec<GraphNode>`, `edges: Vec<GraphEdge>`
- [ ] `pub struct GraphNode` has `id: String` (the event_type or event_id), `label: String`, `event_count: usize`
- [ ] `pub struct GraphEdge` has `from: String`, `to: String`, `weight: usize`
- [ ] `ReceiptGraph` implements `Serialize` (for JSON output)
- [ ] All graph types are `#[derive(Debug, Clone, Serialize)]`

#### Graph Builder

- [ ] `pub fn build_graph(receipt: &Receipt) -> ReceiptGraph` function
- [ ] Nodes derived from distinct `event_type` values in the receipt
- [ ] Edges derived from consecutive event pairs: `events[i].event_type → events[i+1].event_type`
- [ ] Repeated edges (same from→to) increment `weight` instead of adding duplicates
- [ ] Builder has no side effects

#### DOT Formatter

- [ ] `pub fn to_dot(graph: &ReceiptGraph) -> String` function
- [ ] Output begins with `digraph receipt {`
- [ ] Each node: `"{label}" [label="{label} ({count})"];`
- [ ] Each edge: `"{from}" -> "{to}" [label="{weight}"];`
- [ ] Output ends with `}`
- [ ] Node and edge labels are double-quoted and have any internal `"` escaped

#### JSON Formatter

- [ ] `pub fn to_json(graph: &ReceiptGraph) -> Result<String>` function
- [ ] Uses `serde_json::to_string_pretty` for human-readable output
- [ ] Top-level keys are `"nodes"` and `"edges"`
- [ ] `jq .nodes` on the output returns an array

#### Handler (`src/handlers.rs`)

- [ ] `pub fn visualize(receipt: String, format: String) -> Result<()>` signature
- [ ] Validates `format` is one of `"dot"`, `"json"` — returns descriptive error otherwise
- [ ] Calls `build_graph(&parsed)` then branches on format
- [ ] Output routed to stderr (consistent with other handlers)
- [ ] File load errors propagated as non-zero exit

#### Ontology + Verb

- [ ] `affi:visualize` declared as `cnv:Verb` under `receipt` noun
- [ ] Has positional arg `receipt` (index 1) and flag arg `--format`
- [ ] `--format` has allowed values `dot`, `json` (or validated in handler)
- [ ] `ggen sync` passes
- [ ] `pub mod visualize;` added to `src/verbs/mod.rs`

### 3.3 Test Evidence Required

**Unit tests:**

| Test name | File | What it checks |
|-----------|------|----------------|
| `build_graph_single_event` | `src/handlers.rs` or `src/graph_builder.rs` | 1 node, 0 edges |
| `build_graph_linear_3_events` | same | 3 nodes, 2 edges, weights all 1 |
| `build_graph_repeated_type` | same | weight > 1 on self-edge or repeated edge |
| `to_dot_starts_with_digraph` | same | DOT output starts with `digraph` |
| `to_dot_contains_arrow_edges` | same | DOT output contains `->` |
| `to_json_has_nodes_and_edges_keys` | same | JSON parseable, top-level has `nodes` and `edges` |
| `to_json_valid_for_jq` | same | JSON is valid (round-trips through `serde_json::from_str`) |

**E2E tests** (`tests/e2e_inspection.rs`):

| Test name | Expected output |
|-----------|-----------------|
| `visualize_verb_json_has_nodes_and_edges` | stderr/stdout contains `"nodes"` and `"edges"` |
| `visualize_verb_dot_starts_with_digraph` | stderr/stdout starts with or contains `digraph` |
| `visualize_rejects_unknown_format` | exit non-zero, `xml` in error message |
| `visualize_rejects_missing_format_flag` | exit non-zero |

**Minimum required command to pass:**
```bash
cargo test visualize -- --nocapture
# Expected: 11+ tests, 0 failures
```

### 3.4 Exit Criteria

- [ ] All 10 acceptance criteria pass
- [ ] All 7 unit tests pass
- [ ] All 4 E2E tests pass
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] JSON output passes `jq .nodes` without error
- [ ] DOT output passes `dot -V` validation (if graphviz available in CI)

### 3.5 Reviewer Sign-off

| Role | Reviewer | Signs off on |
|------|----------|--------------|
| **Author** | Sean Chatman | Graph builder correctness; DOT validity; JSON schema |
| **Architecture** | — | No duplication with `src/verbs/graph.rs` (existing DFG command) |
| **Test** | — | JSON round-trip test is genuinely failing-when-fake |

**Note:** `affi receipt graph` (existing) emits a DFG summary; `affi receipt visualize` emits a full graph structure in DOT/JSON. These are distinct outputs. The DoD reviewer must verify they do not overlap.

### 3.6 Rollback Criteria

Revert F3 if:

- DOT output is not valid graphviz syntax (fails `dot -T png`)
- JSON output is not valid JSON (fails `jq .`)
- `--format` validation is missing, causing panic on unknown format values
- Any previously-passing test in `tests/dx_verbs_e2e.rs` regresses

### 3.7 Documentation Requirements

- [ ] `FEATURES_DX_QOL.md` updated with `visualize` implementation status
- [ ] Rustdoc on `ReceiptGraph`, `build_graph`, `to_dot`, `to_json`
- [ ] `examples/visualize.sh` shows:
  ```bash
  affi receipt visualize --format=json r.json
  affi receipt visualize --format=dot r.json | dot -Tpng -o receipt_graph.png
  ```
- [ ] `CLAUDE.md` CLI Surface section updated with `visualize` entry

### 3.8 Performance Benchmarks

| Operation | Receipt size | Target | Measurement method |
|-----------|-------------|--------|-------------------|
| `affi receipt visualize --format=json` | 10 events | < 100 ms | hyperfine |
| `affi receipt visualize --format=json` | 100 events | < 200 ms | hyperfine |
| `affi receipt visualize --format=dot` | 100 events | < 200 ms | hyperfine |
| `build_graph` in-process | 1000 events | < 20 ms | criterion bench `bench_visualize_1000` |
| `to_json` serialization | 100-node graph | < 5 ms | criterion bench |

---

## Feature 4: `affi receipt catalog`

### 4.1 Acceptance Criteria

> List and search chicago-tdd-tools fixtures available for use with `affi`.

| # | Given | When | Then | Pass/Fail |
|---|-------|------|------|-----------|
| AC-1 | chicago-tdd-tools fixtures are integrated | `affi receipt catalog` | stderr lists at least 1 fixture with its name and event count | 🔲 |
| AC-2 | chicago-tdd-tools fixtures are integrated | `affi receipt catalog` | output includes a `Name`, `Events`, and `Description` (or similar) column/field per fixture | 🔲 |
| AC-3 | A 3-event fixture exists in chicago-tdd-tools | `affi receipt catalog --filter-events=3` | only fixtures with exactly 3 events appear | 🔲 |
| AC-4 | A fixture named `linear-5-event` exists | `affi receipt catalog --filter-name=linear` | output includes at least 1 result matching `linear` | 🔲 |
| AC-5 | No fixture matches the given filter | `affi receipt catalog --filter-name=nonexistent_xyz` | exit 0, stderr contains `No fixtures match` or similar | 🔲 |
| AC-6 | chicago-tdd-tools is not available or no fixtures registered | `affi receipt catalog` | exit non-zero with actionable error message | 🔲 |
| AC-7 | Catalog is listed | `affi receipt catalog` | output is deterministic (same order on repeated calls) | 🔲 |
| AC-8 | Multiple fixtures exist | `affi receipt catalog` | each fixture appears exactly once (no duplicates) | 🔲 |
| AC-9 | A fixture is listed | `affi receipt catalog --filter-name=<name>` then `affi receipt inspect <fixture_path>` | the fixture path from catalog can be directly inspected | 🔲 |
| AC-10 | `--help` is requested | `affi receipt catalog --help` | help text mentions `--filter-events` and `--filter-name` flags | 🔲 |

### 4.2 Implementation Checklist

#### Fixture Registry Interface

- [ ] Define `pub struct FixtureMeta` with fields: `name: String`, `event_count: usize`, `description: String`, `path: Option<PathBuf>`
- [ ] `pub fn list_fixtures() -> Vec<FixtureMeta>` function that discovers all available chicago-tdd-tools fixtures
- [ ] Fixture discovery does not panic if chicago-tdd-tools has 0 registered fixtures
- [ ] Fixtures returned in deterministic order (sorted by `name` alphabetically)
- [ ] `list_fixtures` is unit-testable without a live filesystem (can return fixtures from a registry constant or chicago-tdd-tools crate)

#### Filtering

- [ ] `pub fn filter_fixtures(fixtures: &[FixtureMeta], name_filter: Option<&str>, event_count_filter: Option<usize>) -> Vec<&FixtureMeta>` function
- [ ] `--filter-name` matches case-insensitively against `FixtureMeta::name` (substring match)
- [ ] `--filter-events` matches exact `event_count`
- [ ] Both filters can be combined (AND semantics)
- [ ] Empty result set is not an error (exit 0)

#### Handler (`src/handlers.rs`)

- [ ] `pub fn catalog(filter_name: Option<String>, filter_events: Option<usize>) -> Result<()>` signature
- [ ] Calls `list_fixtures()` then `filter_fixtures()`
- [ ] Formats output to stderr as a table or list:
  ```
  RECEIPT FIXTURE CATALOG
  =======================
  Name                 Events  Description
  linear-3-event       3       Three linear events: create, transform, release
  ```
- [ ] When 0 results: prints `No fixtures match the given filter.` and exits 0
- [ ] When catalog is empty/unavailable: exits non-zero with message

#### Ontology + Verb

- [ ] `affi:catalog` declared as `cnv:Verb` under `receipt` noun
- [ ] Optional flag `--filter-name` (type `xsd:string`, optional)
- [ ] Optional flag `--filter-events` (type `xsd:integer`, optional)
- [ ] `ggen sync` passes
- [ ] `pub mod catalog;` added to `src/verbs/mod.rs`

### 4.3 Test Evidence Required

**Unit tests:**

| Test name | File | What it checks |
|-----------|------|----------------|
| `filter_fixtures_by_name_case_insensitive` | `src/handlers.rs` | "LINEAR" matches "linear-5-event" |
| `filter_fixtures_by_event_count` | `src/handlers.rs` | count=3 returns only 3-event fixtures |
| `filter_fixtures_combined` | `src/handlers.rs` | name + count filter reduces set |
| `filter_fixtures_no_match` | `src/handlers.rs` | empty result, no panic |
| `list_fixtures_deterministic` | `src/handlers.rs` | two calls return same order |

**E2E tests** (`tests/e2e_inspection.rs`):

| Test name | Expected output |
|-----------|-----------------|
| `catalog_verb_lists_fixtures` | `RECEIPT FIXTURE CATALOG` in stderr |
| `catalog_verb_filter_no_match` | `No fixtures match` in stderr, exit 0 |
| `catalog_verb_help_mentions_filters` | `--filter-name` in help text |

**Minimum required command to pass:**
```bash
cargo test catalog -- --nocapture
# Expected: 8+ tests, 0 failures
```

### 4.4 Exit Criteria

- [ ] All 10 acceptance criteria pass
- [ ] All 5 unit tests pass
- [ ] All 3 E2E tests pass
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `affi receipt catalog --help` shows `--filter-name` and `--filter-events` in help

### 4.5 Reviewer Sign-off

| Role | Reviewer | Signs off on |
|------|----------|--------------|
| **Author** | Sean Chatman | Fixture registry interface; filter logic; output formatting |
| **Integration** | — | chicago-tdd-tools dependency used correctly; `FixtureMeta::path` usable with `affi receipt inspect` |
| **Test** | — | `list_fixtures_deterministic` passes on repeated cargo test runs |

### 4.6 Rollback Criteria

Revert F4 if:

- `list_fixtures()` panics when chicago-tdd-tools has no fixtures
- Output is non-deterministic between runs
- `catalog` exits non-zero when `--filter-name` produces an empty result set
- The fixture path surfaced by catalog does not work with `affi receipt inspect`

### 4.7 Documentation Requirements

- [ ] `FEATURES_DX_QOL.md` updated with `catalog` implementation status
- [ ] Rustdoc on `FixtureMeta`, `list_fixtures`, `filter_fixtures`, `catalog` handler
- [ ] `examples/catalog.sh` demonstrates:
  ```bash
  affi receipt catalog
  affi receipt catalog --filter-events=3
  affi receipt catalog --filter-name=linear
  ```
- [ ] `CLAUDE.md` CLI Surface section updated with `catalog` entry

### 4.8 Performance Benchmarks

| Operation | Fixture count | Target | Measurement method |
|-----------|--------------|--------|-------------------|
| `affi receipt catalog` wall time | ≤ 50 fixtures | < 100 ms | hyperfine |
| `list_fixtures` in-process | any | < 5 ms | criterion bench `bench_catalog_list` |
| `filter_fixtures` in-process | 50 fixtures | < 1 ms | criterion bench `bench_catalog_filter` |

---

## Feature 5: Shell Completion (bash/zsh/fish)

### 5.1 Acceptance Criteria

> Tab completion for all `affi` verbs and flags via clap-noun-verb codegen.

| # | Given | When | Then | Pass/Fail |
|---|-------|------|------|-----------|
| AC-1 | bash is active shell | `source <(affi --completion bash)` succeeds | command exits 0, no errors | 🔲 |
| AC-2 | zsh is active shell | `affi --completion zsh > _affi` then `compinit` | zsh completion script is valid (parseable by zsh) | 🔲 |
| AC-3 | fish is active shell | `affi --completion fish > affi.fish` | fish completion script is valid (parseable by fish) | 🔲 |
| AC-4 | bash completion sourced | type `affi receipt <TAB>` in bash | completion candidates include `inspect`, `diff`, `visualize`, `catalog` | 🔲 |
| AC-5 | bash completion sourced | type `affi receipt inspect <TAB>` | completion candidates include no unexpected flags; `--help` may appear | 🔲 |
| AC-6 | bash completion sourced | type `affi receipt emit --type <TAB>` | completion offers a hint or does not error | 🔲 |
| AC-7 | Any shell | `affi --completion unknown_shell` | exit non-zero with a clear error listing supported shells | 🔲 |
| AC-8 | bash completion sourced | type `affi <TAB>` | `receipt` noun appears in candidates | 🔲 |
| AC-9 | Completion script generated | the output of `affi --completion bash` | script is idempotent: sourcing it twice does not cause errors | 🔲 |
| AC-10 | All 5 Phase 1 verbs are registered | `affi --completion bash` output | `inspect`, `diff`, `visualize`, `catalog` all appear as string literals in the completion script | 🔲 |

### 5.2 Implementation Checklist

#### clap-noun-verb Completion Integration

- [ ] Confirm clap-noun-verb exposes a completion generation API (e.g., `generate_completion(shell: Shell, app: &mut clap::Command)`)
- [ ] `src/bin/affi.rs` handles `--completion <shell>` flag before normal dispatch
- [ ] Completion generation uses `clap_complete::generate` or the clap-noun-verb equivalent
- [ ] `affi --completion bash` writes completion script to stdout (not stderr)
- [ ] `affi --completion zsh` writes zsh completion to stdout
- [ ] `affi --completion fish` writes fish completion to stdout
- [ ] Unrecognized shell name → `anyhow::bail!` with list of valid shells

#### Verb Registration (prerequisite for completion)

- [ ] All 4 new verbs (`inspect`, `diff`, `visualize`, `catalog`) are registered in the clap app before completion is generated
- [ ] Verify `linkme` distributed slice includes all new verbs at binary startup
- [ ] `affi receipt --help` lists all 4 new verbs before completion is implemented (prerequisite check)

#### Ontology / Codegen Updates

- [ ] `ontology/affi-cli.ttl` has `cnv:Verb` entries for `inspect`, `diff`, `visualize`, `catalog`
- [ ] `ggen sync` re-renders all verb wrappers without error
- [ ] All rendered wrappers compile without warnings

#### Completion Script Quality

- [ ] Bash script begins with `_affi_completions()` function definition or equivalent
- [ ] Bash script ends with `complete -F _affi_completions affi`
- [ ] Zsh script uses `#compdef affi` header
- [ ] Fish script uses `complete -c affi` lines
- [ ] No hardcoded paths (scripts work regardless of `$PATH` order)

#### Testing Infrastructure

- [ ] `tests/e2e_inspection.rs` has a `completion_script_contains_new_verbs` test
- [ ] Test invokes `affi --completion bash` via `assert_cmd` and checks output contains `inspect`, `diff`, `visualize`, `catalog`
- [ ] Test for `affi --completion zsh` similarly
- [ ] Test for unknown shell exits non-zero

### 5.3 Test Evidence Required

**E2E tests** (`tests/e2e_inspection.rs`):

| Test name | Expected output |
|-----------|-----------------|
| `completion_bash_contains_inspect` | `inspect` in stdout of `affi --completion bash` |
| `completion_bash_contains_diff` | `diff` in stdout |
| `completion_bash_contains_visualize` | `visualize` in stdout |
| `completion_bash_contains_catalog` | `catalog` in stdout |
| `completion_zsh_is_generated` | exit 0, stdout non-empty |
| `completion_fish_is_generated` | exit 0, stdout non-empty |
| `completion_unknown_shell_errors` | exit non-zero |

**Shell integration test** (manual, recorded in review):

```bash
# Must be performed by reviewer in a real bash session:
source <(cargo run --bin affi -- --completion bash)
affi receipt <TAB>
# Candidate list must include: emit assemble verify show inspect diff visualize catalog
```

Result is recorded in the reviewer sign-off (not automated).

**Minimum required command to pass:**
```bash
cargo test completion -- --nocapture
# Expected: 7+ tests, 0 failures
```

### 5.4 Exit Criteria

- [ ] All 10 acceptance criteria pass
- [ ] All 7 automated E2E tests pass
- [ ] Manual shell test recorded in reviewer sign-off
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `affi --help` still works after completion integration (no regression)
- [ ] All pre-existing verbs (`emit`, `assemble`, `verify`, `show`, `inspect`, `model`, `diagnose`, `conformance`, `replay`, `graph`, `stats`) appear in the bash completion output

### 5.5 Reviewer Sign-off

| Role | Reviewer | Signs off on |
|------|----------|--------------|
| **Author** | Sean Chatman | Completion generation code; shell script validity |
| **Shell** | — | Manual test: `affi receipt <TAB>` in bash and zsh shows correct candidates |
| **Integration** | — | All new verbs are registered with clap before completion is called |

### 5.6 Rollback Criteria

Revert F5 if:

- `affi --completion bash` panics or exits non-zero
- Sourcing the bash completion script causes `bash: syntax error`
- The completion `--completion` flag shadows any existing verb flag
- `cargo test --all` introduces regressions in any existing test

### 5.7 Documentation Requirements

- [ ] `CLAUDE.md` updated with shell completion setup instructions
- [ ] `README.md` updated with "Shell Completion" section
- [ ] `examples/golden_run.sh` or new `examples/completion_setup.sh` demonstrates:
  ```bash
  # bash
  source <(affi --completion bash)
  # zsh (~/.zshrc)
  affi --completion zsh > "${fpath[1]}/_affi"
  # fish (~/.config/fish/completions/)
  affi --completion fish > ~/.config/fish/completions/affi.fish
  ```

### 5.8 Performance Benchmarks

| Operation | Target | Measurement method |
|-----------|--------|-------------------|
| `affi --completion bash` generation time | < 50 ms | hyperfine |
| `affi --completion zsh` generation time | < 50 ms | hyperfine |
| `affi --completion fish` generation time | < 50 ms | hyperfine |
| Binary startup to first output (all verbs) | < 30 ms | hyperfine with empty receipt |

---

## Phase 1 Master Acceptance Test

### File: `tests/e2e_inspection.rs`

This is the single E2E test file covering all 5 Phase 1 features. It must be created as part of Phase 1 and is the canonical evidence that Phase 1 is complete.

**Preamble and helpers:**

```rust
// tests/e2e_inspection.rs
//
// Phase 1 master E2E test: Receipt Inspection (affidavit DX/QOL 1000x).
//
// Covers:
//   F1: affi receipt inspect   — detailed event/object type report
//   F2: affi receipt diff      — compare two receipts
//   F3: affi receipt visualize — process graph output
//   F4: affi receipt catalog   — list/search chicago-tdd-tools fixtures
//   F5: shell completion       — bash/zsh/fish via clap-noun-verb
//
// All tests run against the REAL affi binary (assert_cmd::Command::cargo_bin).
// No stubs. No mocks. Failing-when-fake.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// Build a receipt with N events of the given (type, object) pairs, assembled to `out`.
fn build_receipt_with(dir: &TempDir, events: &[(&str, &str)], out: &str) {
    for (ty, obj) in events {
        affi(dir)
            .args(["receipt", "emit", "--type", ty, "--object", obj, "--payload", "-"])
            .write_stdin(*ty)
            .assert()
            .success();
    }
    affi(dir)
        .args(["receipt", "assemble", "--out", out])
        .assert()
        .success();
}
```

**F1 tests:**

```rust
// ── F1: inspect ──────────────────────────────────────────────────────────────

#[test]
fn inspect_reports_event_distribution() { /* AC-1, AC-2, AC-3 */ }

#[test]
fn inspect_counts_duplicate_event_types() { /* AC-5 */ }

#[test]
fn inspect_omits_object_section_when_no_objects() { /* AC-4 */ }

#[test]
fn inspect_shows_total_event_count() { /* AC-8 */ }

#[test]
fn inspect_rejects_missing_file() { /* AC-6 */ }

#[test]
fn inspect_rejects_tampered_receipt() { /* AC-7 */ }
```

**F2 tests:**

```rust
// ── F2: diff ─────────────────────────────────────────────────────────────────

#[test]
fn diff_identical_receipts_reports_no_differences() { /* AC-1 */ }

#[test]
fn diff_reports_added_event_in_b() { /* AC-2 */ }

#[test]
fn diff_reports_removed_event_from_b() { /* AC-3 */ }

#[test]
fn diff_reports_modified_event_type() { /* AC-4 */ }

#[test]
fn diff_reports_modified_commitment() { /* AC-5 */ }

#[test]
fn diff_rejects_missing_file() { /* AC-7 */ }
```

**F3 tests:**

```rust
// ── F3: visualize ────────────────────────────────────────────────────────────

#[test]
fn visualize_json_has_nodes_and_edges() { /* AC-1, AC-2, AC-3 */ }

#[test]
fn visualize_dot_starts_with_digraph() { /* AC-4, AC-5 */ }

#[test]
fn visualize_single_event_zero_edges() { /* AC-9 */ }

#[test]
fn visualize_rejects_unknown_format() { /* AC-7 */ }

#[test]
fn visualize_rejects_tampered_receipt() { /* AC-10 */ }
```

**F4 tests:**

```rust
// ── F4: catalog ──────────────────────────────────────────────────────────────

#[test]
fn catalog_lists_available_fixtures() { /* AC-1, AC-2 */ }

#[test]
fn catalog_filter_no_match_exits_zero() { /* AC-5 */ }

#[test]
fn catalog_help_mentions_filter_flags() { /* AC-10 */ }
```

**F5 tests:**

```rust
// ── F5: shell completion ─────────────────────────────────────────────────────

#[test]
fn completion_bash_contains_all_phase1_verbs() { /* AC-10 */ }

#[test]
fn completion_zsh_is_generated() { /* AC-2 */ }

#[test]
fn completion_fish_is_generated() { /* AC-3 */ }

#[test]
fn completion_unknown_shell_exits_nonzero() { /* AC-7 */ }
```

**Master E2E test (all 5 features in one flow):**

```rust
// ── Phase 1 Master: all 5 features in one receipt lifecycle ──────────────────

#[test]
fn phase1_master_e2e_all_features() {
    let dir = TempDir::new().expect("tempdir");

    // Build receipt A (3 events)
    build_receipt_with(&dir, &[
        ("create",    "file:artifact"),
        ("transform", "data:artifact"),
        ("release",   "file:artifact"),
    ], "a.json");

    // Build receipt B (4 events — A plus one more)
    build_receipt_with(&dir, &[
        ("create",    "file:artifact"),
        ("transform", "data:artifact"),
        ("release",   "file:artifact"),
        ("audit",     "log:audit"),
    ], "b.json");

    // F1: inspect a.json
    affi(&dir)
        .args(["receipt", "inspect", "a.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("RECEIPT INSPECTION REPORT"))
        .stderr(predicate::str::contains("Total events: 3"))
        .stderr(predicate::str::contains("create: 1 events"))
        .stderr(predicate::str::contains("Inspection complete."));

    // F2: diff a.json b.json
    affi(&dir)
        .args(["receipt", "diff", "a.json", "b.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("1 added"));

    // F2: diff identical (a.json a.json)
    affi(&dir)
        .args(["receipt", "diff", "a.json", "a.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No differences found"));

    // F3: visualize json
    affi(&dir)
        .args(["receipt", "visualize", "--format=json", "a.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("nodes"))
        .stderr(predicate::str::contains("edges"));

    // F3: visualize dot
    affi(&dir)
        .args(["receipt", "visualize", "--format=dot", "a.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("digraph"));

    // F4: catalog (listing)
    affi(&dir)
        .args(["receipt", "catalog"])
        .assert()
        .success()
        .stderr(predicate::str::contains("RECEIPT FIXTURE CATALOG"));

    // F5: bash completion contains phase 1 verbs
    affi(&dir)
        .args(["--completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("inspect"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("visualize"))
        .stdout(predicate::str::contains("catalog"));
}
```

---

## Feature Matrix

> Which tests cover which features and acceptance criteria.

### Unit Tests

| Test | F1 | F2 | F3 | F4 | F5 |
|------|----|----|----|----|-----|
| `inspect_generates_detailed_report` | ✓ | | | | |
| `inspect_empty_objects_omits_object_section` | ✓ | | | | |
| `inspect_counts_duplicate_types` | ✓ | | | | |
| `inspect_10_event_receipt` | ✓ | | | | |
| `diff_identical_receipts_no_differences` | | ✓ | | | |
| `diff_added_event` | | ✓ | | | |
| `diff_removed_event` | | ✓ | | | |
| `diff_modified_event_type` | | ✓ | | | |
| `diff_modified_commitment` | | ✓ | | | |
| `diff_multiple_changes` | | ✓ | | | |
| `diff_empty_receipts` | | ✓ | | | |
| `build_graph_single_event` | | | ✓ | | |
| `build_graph_linear_3_events` | | | ✓ | | |
| `build_graph_repeated_type` | | | ✓ | | |
| `to_dot_starts_with_digraph` | | | ✓ | | |
| `to_dot_contains_arrow_edges` | | | ✓ | | |
| `to_json_has_nodes_and_edges_keys` | | | ✓ | | |
| `to_json_valid_for_jq` | | | ✓ | | |
| `filter_fixtures_by_name_case_insensitive` | | | | ✓ | |
| `filter_fixtures_by_event_count` | | | | ✓ | |
| `filter_fixtures_combined` | | | | ✓ | |
| `filter_fixtures_no_match` | | | | ✓ | |
| `list_fixtures_deterministic` | | | | ✓ | |

### E2E Tests (`tests/e2e_inspection.rs`)

| Test | F1 | F2 | F3 | F4 | F5 | Master |
|------|----|----|----|----|-----|--------|
| `inspect_reports_event_distribution` | ✓ | | | | | ✓ |
| `inspect_counts_duplicate_event_types` | ✓ | | | | | |
| `inspect_omits_object_section_when_no_objects` | ✓ | | | | | |
| `inspect_shows_total_event_count` | ✓ | | | | | ✓ |
| `inspect_rejects_missing_file` | ✓ | | | | | |
| `inspect_rejects_tampered_receipt` | ✓ | | | | | |
| `diff_identical_receipts_reports_no_differences` | | ✓ | | | | ✓ |
| `diff_reports_added_event_in_b` | | ✓ | | | | ✓ |
| `diff_reports_removed_event_from_b` | | ✓ | | | | |
| `diff_reports_modified_event_type` | | ✓ | | | | |
| `diff_reports_modified_commitment` | | ✓ | | | | |
| `diff_rejects_missing_file` | | ✓ | | | | |
| `visualize_json_has_nodes_and_edges` | | | ✓ | | | ✓ |
| `visualize_dot_starts_with_digraph` | | | ✓ | | | ✓ |
| `visualize_single_event_zero_edges` | | | ✓ | | | |
| `visualize_rejects_unknown_format` | | | ✓ | | | |
| `visualize_rejects_tampered_receipt` | | | ✓ | | | |
| `catalog_lists_available_fixtures` | | | | ✓ | | ✓ |
| `catalog_filter_no_match_exits_zero` | | | | ✓ | | |
| `catalog_help_mentions_filter_flags` | | | | ✓ | | |
| `completion_bash_contains_all_phase1_verbs` | | | | | ✓ | ✓ |
| `completion_zsh_is_generated` | | | | | ✓ | |
| `completion_fish_is_generated` | | | | | ✓ | |
| `completion_unknown_shell_exits_nonzero` | | | | | ✓ | |
| `phase1_master_e2e_all_features` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

### Acceptance Criteria Coverage

| Feature | Total ACs | ACs covered by unit tests | ACs covered by E2E | Both |
|---------|-----------|--------------------------|-------------------|------|
| F1 inspect | 10 | AC-1,2,3,4,5,8,9,10 | AC-1,2,3,4,5,6,7,8,9,10 | All |
| F2 diff | 10 | AC-1,2,3,4,5,6,9 | AC-1,2,3,4,5,6,7,8,9,10 | All |
| F3 visualize | 10 | AC-1,2,3,4,5,8,9 | AC-1,2,3,4,5,7,9,10 | All |
| F4 catalog | 10 | AC-3,4,5,7,8 | AC-1,2,3,4,5,6,7,8,10 | All |
| F5 completion | 10 | — | AC-1,2,3,4,7,8,10 | All (manual for AC-4,5,6,8,9) |

---

## Phase 1 Quality Gate

### Required before any Phase 2 work begins

```bash
# 1. All tests green
cargo test --all 2>&1 | tail -5
# Expected: "test result: ok. N passed; 0 failed; 0 ignored"

# 2. Zero clippy warnings
cargo clippy -- -D warnings 2>&1 | tail -5
# Expected: no output (or only "Finished")

# 3. Format check
cargo fmt --check 2>&1
# Expected: no output (exit 0)

# 4. New test count (Phase 1 must add ≥ 25 tests vs baseline)
cargo test --all 2>&1 | grep "test result"
# Expected: total tests ≥ (baseline + 25)

# 5. Master E2E test passes specifically
cargo test phase1_master_e2e_all_features -- --nocapture
# Expected: "test phase1_master_e2e_all_features ... ok"
```

### Quality Gate Checklist

- [ ] `cargo test --all` → 0 failures, 0 ignored
- [ ] `cargo clippy -- -D warnings` → clean
- [ ] `cargo fmt --check` → clean
- [ ] `tests/e2e_inspection.rs` exists and all tests in it pass
- [ ] `phase1_master_e2e_all_features` test passes
- [ ] No existing test in `tests/e2e.rs`, `tests/dx_verbs_e2e.rs`, or `tests/dx_full_pipeline_e2e.rs` regresses
- [ ] Total test count increased by ≥ 25 compared to pre-Phase-1 baseline
- [ ] `benches/receipt_operations.rs` has at least one new bench per feature (5 new benches minimum)
- [ ] `cargo bench --bench receipt_operations` runs without panic

### Baseline (pre-Phase-1)

Record baseline test count before starting Phase 1:

```bash
cargo test --all 2>&1 | grep "test result"
```

> **Baseline recorded:** _____ tests (fill in before Phase 1 begins)

---

## Phase 1 Exit Criteria (summary)

Phase 1 is complete when ALL of the following are checked:

### Per-Feature

- [ ] F1 `inspect`: all 10 ACs pass, 4 unit + 3 E2E tests green, docs updated
- [ ] F2 `diff`: all 10 ACs pass, 7 unit + 3 E2E tests green, docs updated
- [ ] F3 `visualize`: all 10 ACs pass, 7 unit + 4 E2E tests green, docs updated
- [ ] F4 `catalog`: all 10 ACs pass, 5 unit + 3 E2E tests green, docs updated
- [ ] F5 `completion`: all 10 ACs pass, 7 E2E tests green, manual shell test recorded

### Global

- [ ] `tests/e2e_inspection.rs` created with all 25 test functions
- [ ] `phase1_master_e2e_all_features` test passes end-to-end
- [ ] Quality gate: `cargo test --all`, `cargo clippy`, `cargo fmt --check` all pass
- [ ] `ggen sync` passes with all new verbs in `ontology/affi-cli.ttl`
- [ ] All 5 verbs appear in `affi receipt --help` output
- [ ] All 5 verbs appear in `affi --completion bash` output
- [ ] `FEATURES_DX_QOL.md` shows all 5 features as "✅ implemented"
- [ ] `CLAUDE.md` CLI Surface section updated with all 5 new commands
- [ ] No `unwrap()` in non-test production code introduced by Phase 1
- [ ] No `println!` (stdout) in handlers — all handler output via `eprint!`/`eprintln!`

### Performance

- [ ] `affi receipt inspect` on 100-event receipt: < 200 ms (measured)
- [ ] `affi receipt diff` on two 100-event receipts: < 250 ms (measured)
- [ ] `affi receipt visualize --format=json` on 100-event receipt: < 200 ms (measured)
- [ ] `affi receipt catalog` (listing): < 100 ms (measured)
- [ ] `affi --completion bash`: < 50 ms (measured)

---

## Approvals

Phase 1 is closed when the following sign-off block is completed:

| Sign-off | Name | Date | Notes |
|----------|------|------|-------|
| **Author** | Sean Chatman | | All code written and tested |
| **Architecture review** | | | Handler delegation correct; no business logic in verb wrappers |
| **Test review** | | | Tests are failing-when-fake; no tautological assertions |
| **Integration review** | | | ggen sync passes; all verbs registered; completion works |
| **Manual shell test** | | | Tab completion verified in bash and zsh by reviewer |
| **Performance** | | | All 5 bench targets measured and within budget |

---

## Appendix A: File Change Map

Files expected to be created or modified in Phase 1:

| File | Action | Feature |
|------|--------|---------|
| `ontology/affi-cli.ttl` | **Modified** — add 4 new verb declarations | F1, F2, F3, F4 |
| `src/handlers.rs` | **Modified** — add `diff_receipts`, `visualize`, `catalog` | F2, F3, F4 |
| `src/verbs/mod.rs` | **Modified** — add `pub mod diff/visualize/catalog`; new helper functions | F1, F2, F3, F4 |
| `src/verbs/diff.rs` | **Created** | F2 |
| `src/verbs/visualize.rs` | **Created** | F3 |
| `src/verbs/catalog.rs` | **Created** | F4 |
| `src/bin/affi.rs` | **Modified** — add `--completion` flag handling | F5 |
| `tests/e2e_inspection.rs` | **Created** | F1–F5 |
| `benches/receipt_operations.rs` | **Modified** — add 5 new bench cases | F1–F5 |
| `examples/inspection.sh` | **Created** | F1 |
| `examples/diff_receipts.sh` | **Created** | F2 |
| `examples/visualize.sh` | **Created** | F3 |
| `examples/catalog.sh` | **Created** | F4 |
| `examples/completion_setup.sh` | **Created** | F5 |
| `FEATURES_DX_QOL.md` | **Modified** — mark all 5 as implemented | All |
| `CLAUDE.md` | **Modified** — update CLI Surface table | All |

---

## Appendix B: Edge Cases Reference

| Edge case | Expected behavior | Relevant ACs |
|-----------|------------------|--------------|
| Receipt with 0 events | Not possible post-assemble (assemble requires ≥1 event) | — |
| Receipt with 1 event | inspect: Total events: 1; diff: compares normally; visualize: 1 node 0 edges | F1-AC-8, F3-AC-9 |
| Receipt with duplicate event IDs | Blocked by verifier `continuity` stage — receipt will not deserialize | AC-7 across all |
| Object with qualifier (`file:artifact:main`) | `obj_type` is `artifact`; qualifier appears in `show` but inspect groups by `obj_type` | F1-AC-3 |
| Very long event type string (>256 chars) | Admission gate (`src/admission.rs`) should reject at emit time | Not tested in Phase 1 |
| Receipt file with extra JSON fields | `serde_json` uses `deny_unknown_fields`? Check `types.rs` Receipt derive. | F1-AC-6 |
| Binary (non-UTF-8) receipt file | `std::fs::read_to_string` returns error → propagated as non-zero exit | F1-AC-6, F2-AC-7 |
| `diff` where a has seq gaps (not contiguous) | Should not happen post-verify; if it occurs, diff by seq still works | F2 |
| `catalog` with no chicago-tdd-tools fixtures registered | Print `No fixtures available.`, exit 0 | F4-AC-6 |
| Completion for shell with special chars in verb name | Not applicable — verb names are ASCII alphanumeric + dash | F5 |

---

## Appendix C: Glossary

| Term | Definition |
|------|-----------|
| **80/20** | 80% reused library code (chicago-tdd-tools, ggen), 20% new glue code |
| **Failing-when-fake** | A test that would fail if its target code were stubbed or unregistered |
| **Handler** | A function in `src/handlers.rs` that implements a verb's business logic |
| **Verb wrapper** | A thin `#[verb]` annotated function in `src/verbs/*.rs`, generated by ggen |
| **DiffResult** | The structured output of F2's comparison algorithm |
| **FixtureMeta** | Metadata struct describing a chicago-tdd-tools fixture |
| **ReceiptGraph** | The graph data structure produced by F3's visualize command |
| **DoD** | Definition of Done |
| **AC** | Acceptance Criterion |
| **ggen sync** | The code-generation step that re-renders verb wrappers from the ontology |
| **§6 guard** | The project rule that handler output goes to stderr, not stdout |
| **Chain hash mismatch** | The error emitted when a receipt's stored `chain_hash` ≠ recomputed hash |
| **Tampered receipt** | A receipt whose bytes were modified after assembly (detected via BLAKE3) |

---

*Definition of Done — Phase 1: Receipt Inspection*  
*affidavit v26.6.14 | branch: `claude/zen-cerf-oq87br` | 2026-06-14*
