# value-matrix.md

Rollout plan for the CLI value-matrix completeness court
(`tests/value_matrix_index.rs`, v26.10.17).

## Purpose

Every (noun, verb) pair in `src/registry.rs` REGISTRY (100 pairs, 15 nouns)
must appear as a test-annotated marker `// value-matrix: (noun, verb)` in at
least one `tests/cli_value_matrix/*.rs` file. The 17 domain lanes add those
markers next to the tests that exercise each pair. Until then, the court
accepts pairs listed in this doc.

## Court mechanics

- Registry parse: same regex family as `scripts/generate_completions.py`
  (`VerbEntry::new("verb", "noun", VerbGroup::..., "...", ...)`), with
  Rust-escape decoding. Asserts non-empty parse (100 pairs) and no duplicate
  (noun, verb) pairs — a broken regex is a loud FAIL, not a silent pass.
- Marker scan: every `tests/cli_value_matrix/*.rs` file, marker
  `// value-matrix: (noun, verb)`.
- Doc fallback: `- (noun, verb)` lines in this file (section below).
- Coverage table printed to stderr on every run (a green run carries its own
  evidence; a court that cannot fail is not a court).
- Gate: `cargo test --test value_matrix_index` exit 0.

## Soft/hard gate

- Now: `COMPLETE = false` in `tests/value_matrix_index.rs` — the court prints
  `WARNING VALUE_MATRIX_ROLLOUT` and passes while coverage < 100%, so CI stays
  green during the rollout.
- Flip: the coordinator sets `COMPLETE = true` in
  `tests/value_matrix_index.rs` when coverage reaches 100%; from then on any
  missing pair is a hard FAIL (`VALUE_MATRIX_INCOMPLETE (hard)`). The flip is
  a one-line change, documented here per the work order.

## Rollout plan (17 domain lanes)

1. Each lane owns a disjoint set of `tests/cli_value_matrix/*.rs` files and
   adds `// value-matrix: (noun, verb)` markers next to real CLI tests.
2. Lanes must not list pairs in this doc's covered-pairs section unless the
   lane also lands a real test in the same diff.
3. Coordinator tracks coverage via the stderr table per lane landing.
4. When the table shows 100/100, coordinator flips `COMPLETE = true` and
   appends the flip receipt here.

## Covered pairs

Lanes append `- (noun, verb)` lines here only alongside a real test landing.
Current coverage: 0/100 — all pairs currently missing; the court's stderr
table is the live per-noun view.

