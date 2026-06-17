# DOC_COVERAGE_LOG

## 2026-06-14 — Bijective doc<->example gap map (synthesis pass)

### Per-example exit-code table (real `cargo run --example <name>` exit codes)

| Example | Exit code | Witness |
|---|---|---|
| admission_gate | 0 | PASS |
| chain_build | 0 | PASS |
| discover_shapeb | 0 | PASS |
| full_pipeline | 0 | PASS |
| observable_spans | 0 | PASS |
| ocel_events | 0 | PASS |
| receipt_determinism | 0 | PASS |
| verdict_diagnostics | 0 | PASS |
| verify_stages | 0 | PASS |

All 9 examples PASS (exit 0). No failed witnesses.

### Counts

- Documented pub items (lib `src/*.rs`, excluding binary verbs `handlers.rs`/`cli.rs`): ~50 pub fn/struct/enum.
- Exercised by lib examples (asserted, end-to-end): 9 capabilities spanning admission, chain, discovery, lsp, ocel, verifier, types, tracing, plus the cross-product full_pipeline.
- Additionally exercised by the `affi` binary verbs (handlers.rs/cli.rs + golden_run.sh + tests/dx_*_e2e.rs), NOT by lib examples — honestly binary-exercised, not gaps: `chain::content_address`, `chain::save_receipt`, `chain::save_working`, `chain::load_working`, `discovery::discover_dfg_summary`, `discovery::quality_metrics`.

### Gap direction 1 — documented-but-unexercised (have `///`, no lib example AND no binary exercise)

1. `chain::ChainAssembler::from_events` (src/chain.rs:107)
2. `chain::ChainAssembler::events` (src/chain.rs:120)
3. `chain::serialize_receipt` (src/chain.rs:153)
4. `chain::deserialize_receipt` (src/chain.rs:158)
5. `ocel::SeqCounter::starting_at` (src/ocel.rs:31)
6. `ocel::SeqCounter::next_seq` (src/ocel.rs:36)
7. `discovery::conformance_metrics` (src/discovery.rs:78)
8. `tracing::trace_emit` (src/tracing.rs:59)
9. `tracing::trace_assemble` (src/tracing.rs:68)
10. `tracing::trace_show` (src/tracing.rs:88)

Note: `deserialize_receipt`/`from_events` are indirectly used inside forged-receipt construction in several examples but are not the asserted subject of any example; counted as gaps for strictness.

### Gap direction 2 — exercised-but-undocumented (API an example/binary calls whose pub item lacks `///`)

- `chain::ChainError` (src/chain.rs:29) — no `///` above declaration; surfaced as Result error type in exercised paths.
- `types::AffidavitReceiptChain` (src/types.rs:55) — no `///`; marker struct, not exercised either (dead-ish).

### Bidirectional-link audit

- Example -> doc (header `//!` cites src module/symbol): ALL 9 present and correct.
- Doc -> example (pub item `///` cites `examples/<name>.rs`): ALL 9 present (src/admission.rs, chain.rs, discovery.rs, lsp.rs, ocel.rs, verifier.rs, types.rs, tracing.rs, and crate-level src/lib.rs //!).
- No missing links.

### Doc discrepancies surfaced by authors

- src/discovery.rs: the narrative doc block at lines ~84-91 describing the admission type-gate is physically attached to `discover_dfg_summary`, not to `discover_from_admitted` (which had no own doc). The example link was attached directly to `discover_from_admitted`. Mis-aligned source doc, flagged not papered over.
- src/lib.rs: already had a crate-level `//!` doc (an author assignment assumed none); handled by appending a cross-product section rather than overwriting.

### GAP metric

GAP = 10 documented-but-unexercised pub items remaining.

---

## Iteration 2 — 2026-06-14 (gap closure: 10 -> 0 documented-but-unexercised)

The 10 documented-but-unexercised lib APIs were closed by EXTENDING existing
examples (no word-count inflation, no new files):

| API(s) | Now exercised by |
|---|---|
| `tracing::{trace_emit, trace_assemble, trace_show}` | `examples/observable_spans.rs` — all 4 wrappers, span operation+target asserted (emit->event_type, assemble->count, verify/show->path) |
| `ocel::SeqCounter::{starting_at, next_seq}` | `examples/ocel_events.rs` — resume numbering from offset 5; `next_seq` monotonic 5->6; `build_event` continues at 7 |
| `chain::{from_events, events, serialize_receipt, deserialize_receipt}` | `examples/chain_build.rs` — rehydrate + re-finalize reproduces chain_hash; canonical round-trip; corrupt hash REFUSED at decode |
| `discovery::conformance_metrics` | `examples/discover_shapeb.rs` — fitness/coverage agree with `quality_metrics_from_admitted` (same net) |

### Contract correction (verified against source, NOT papered over)

- `chain::deserialize_receipt`: the `Receipt` type has a custom `Deserialize` impl that
  RE-VERIFIES the chain hash at decode time. A first draft asserted the opposite
  ("structural decode, no verification"); running the example surfaced the real error
  `chain hash mismatch: receipt claims ... recomputed ...` and the assertion was
  corrected to the true contract (deserialize REJECTS a corrupt chain_hash). The
  failing-when-fake example caught an aspirational doc claim before it shipped.

### Prior-iteration findings resolved

- src/discovery.rs doc misalignment FIXED: the Shape-B admission-type-gate narrative
  was moved from `discover_dfg_summary` to `discover_from_admitted` (where it belongs).
- "Exercised-but-undocumented" `ChainError` / `AffidavitReceiptChain`: FALSE POSITIVE —
  both already carry multi-line `///` docs (a `#[derive]`/`#[error]` sits between the
  doc and the declaration, which the enumeration grep skipped). No edit made (adding
  docs would be the word-count inflation this goal forbids).

### GAP metric

GAP = 0 documented-but-unexercised lib pub items. (6 APIs — `content_address`,
`save_receipt`, `save_working`, `load_working`, `discover_dfg_summary`,
`quality_metrics` — remain honestly BINARY-exercised via the `affi` verbs +
`examples/golden_run.sh` + `tests/dx_*_e2e.rs`, not lib-example gaps.)
All 9 lib examples run (real exit 0) and assert their failure/edge mode; bidirectional
links complete in both directions. Full suite: 346 passing, 0 failing.
