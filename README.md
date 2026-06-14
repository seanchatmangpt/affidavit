# affidavit

**The Provenance Layer.** `affi` CLI · v26.6.14

`affidavit` assembles and certifies **provenance receipts**: append-only,
content-addressed chains of operation-events that record what a process did.
The `affi` binary lets you emit events, finalize them into an immutable
receipt, and verify that receipt against a fixed format standard.

## Doctrine

1. **Certify, don't decide.** The verifier never decides whether work is
   honest — that question is undecidable. It *checks a witness* (the receipt)
   against a format standard, and every check is decidable.
2. **A receipt is an append-only BLAKE3 chain of operation-events.** Each link
   folds the previous chain hash with the canonical bytes of the next event, so
   any edit to any event propagates through every later link.
3. **Unverifiable work is rejected, not detected.** A tampered or malformed
   receipt simply fails a stage and yields `REJECT` — the verifier proves a
   lawful chain exists, it does not hunt for fraud.
4. **The bypass is unconstructable.** Receipt struct-literal construction fails
   at compile time (E0451: private field `_seal`). Only the canonical seam
   ([`crate::chain::ChainAssembler::finalize`]) can construct sealed receipts.

## Quick Start

The full provenance workflow in 5 steps:

```bash
# 1. emit — record an operation-event into the working receipt
affi emit --type create --object "file.txt:artifact" --payload -

# 2. assemble — seal the working events into an immutable receipt
affi assemble --out receipt.json

# 3. verify — run the 7-stage certify pipeline
affi verify receipt.json
# -> verdict: ACCEPT [core/v1] — all stages passed

# 4. show — human-readable dump without rendering verdict
affi show receipt.json

# 5. inspect — structured JSON inspection
affi inspect receipt.json
```

Or in Rust (library path):

```rust
use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::admission::admit;
use affidavit::discovery::quality_metrics_from_admitted;

let mut asm = ChainAssembler::new();
let mut counter = SeqCounter::new();
let ev = build_event("create", vec![object_ref("file.txt", "artifact")], b"data", &mut counter)?;
asm.append(ev)?;
let receipt = asm.finalize();

let admitted = admit(receipt)?;
let (fitness, activity_coverage, simplicity) = quality_metrics_from_admitted(&admitted);
```

## Install / build

Stable Rust, edition 2021. From the repo root:

```bash
cargo build           # builds the affi binary
cargo test            # runs all tests
```

Run the binary either as `cargo run --bin affi -- <args>` or directly from
`target/debug/affi`.

## CLI surface

### All Verbs

| Verb | Description | ARDPRD ref |
|------|-------------|-----------|
| `affi emit` | Append one operation-event to the working receipt | FR-1 |
| `affi assemble` | Finalize working events into an immutable sealed receipt | FR-2 |
| `affi verify` | Run the 7-stage certify pipeline; exit 0 on ACCEPT | FR-3 |
| `affi show` | Human-readable dump of the chain (no verdict) | FR-4 |
| `affi inspect` | Structured JSON inspection of a receipt | DX/QOL |
| `affi conform` | Run full conformance pipeline (admit + quality metrics) | §4 seam |
| `affi discover` | Run process discovery on an admitted receipt | wasm4pm |
| `affi model` | Show the discovered process model (DFG + process tree) | wasm4pm |
| `affi admit` | Run the admission gate (OCEL court + chain verifier) | §4 seam |
| `affi sign` | Sign a receipt with a key | NFR-2 |
| `affi publish` | Publish a receipt to a registry | FR-5 |
| `affi replay` | Replay a receipt trace through the process model | wasm4pm |
| `affi mutate` | Run mutation testing on a receipt | clnrm |
| `affi bench` | Run Criterion benchmarks on chain operations | Criterion |
| `affi lint` | Run the LSP diagnostics pipeline over a receipt | lsp-max |

### Core verb details

| Command | Purpose |
| --- | --- |
| `affi emit --type <event_type> --object <id:type[:qualifier]> ... --payload <file\|->` | Append one operation-event to the working receipt (`.affi/working.json`). The commitment is `blake3(payload)`; the raw payload is never stored. |
| `affi assemble [--out <path>]` | Finalize the working receipt into an immutable receipt file. Default name is the content address (`blake3` of canonical bytes). |
| `affi verify <receipt.json>` | Run the certify pipeline. Prints per-stage outcomes and the final verdict. Exit `0` on ACCEPT, non-zero on REJECT. |
| `affi show <receipt.json>` | Human-readable dump of the chain. |

### Worked example (the golden run)

The script at [`examples/golden_run.sh`](examples/golden_run.sh) runs the real
binary through the full lifecycle in a temp dir, then corrupts the receipt with
`sed` and re-verifies. Abridged output:

```text
--- emit event 1 ---
emitted event evt-0 (seq 0)
--- emit event 2 ---
emitted event evt-1 (seq 1)

--- assemble ---
assembled receipt -> receipt.json
content address: 88047402d8a59ea1099b6c374a19e4a7cc0c5a01b247dfcd566cb4b01becf05f

--- verify (honest, expect ACCEPT / exit 0) ---
decode: PASS — 2 event(s), format_version present
check_format: PASS — format_version == core/v1
chain_integrity: PASS — recomputed chain hash matches stored chain_hash
continuity: PASS — 2 event(s) with contiguous seq and unique ids
verify_commitments: PASS — all commitments are well-formed BLAKE3 digests
evaluate_profile: PASS — profile core/v1 satisfied
verdict: ACCEPT [core/v1] — all stages passed
exit code: 0

--- verify (tampered, expect REJECT / non-zero exit) ---
chain_integrity: FAIL — chain hash mismatch: stored 203d3bbf… recomputed dd9b9980…
verdict: REJECT [core/v1] — chain_integrity: chain hash mismatch …
exit code: 2
```

Flipping one event's `event_type` re-routes the rolling chain hash, so
`chain_integrity` recomputes a different hash than the one stored in the
receipt and the verdict flips to `REJECT`.

## Architecture

```
Events (build_event + SeqCounter)
   │
   ▼
ChainAssembler           rolling BLAKE3 hash folded per event
   │ .append(ev)         O(1) incremental — no re-hash on append
   │ .finalize()         seals the Receipt (private _seal field)
   ▼
Receipt                  immutable, content-addressed chain
   │
   ├──► Verifier         7-stage certify pipeline (pure, deterministic)
   │         │           decode → check_format → chain_integrity →
   │         │           continuity → verify_commitments → evaluate_profile
   │         ▼           → emit_verdict
   │       Verdict
   │         │
   │         ├──► LSP Diagnostics  (lsp.rs — lsp-max integration)
   │         │    one Error diagnostic per failing stage
   │         │
   │         └──► OTel Span        (tracing.rs — observable spans)
   │
   └──► admit()          BOTH courts must pass:
            │            1. wasm4pm-compat OCEL structural law
            │            2. affidavit chain/continuity certify pipeline
            ▼
       AdmittedReceipt   Evidence<Receipt, Admitted, AffidavitReceiptChain>
            │            type-gated: only reachable via admit()
            │
            ├──► discover_from_admitted()    wasm4pm process tree
            └──► quality_metrics_from_admitted()
                         fitness (token replay) + activity_coverage + simplicity
```

## The verifier: a 7-stage certify pipeline

The verifier is a straight pipeline — no component decides honesty. It maps
directly onto the C4 Level-3 component view:

| # | Stage | C4 component | Decidable check |
| --- | --- | --- | --- |
| 1 | `decode` | decode | Receipt is structurally present and the version field parses. |
| 2 | `check_format` | check_format(version) | `format_version` equals the standard this verifier knows (`core/v1`). |
| 3 | `chain_integrity` | check_chain_integrity | Recompute the rolling BLAKE3 chain hash from event bytes and compare to the stored `chain_hash`. |
| 4 | `continuity` | resolve_continuity | `seq` is contiguous from 0 with no gaps; event ids are unique. |
| 5 | `verify_commitments` | verify_commitments | Every payload commitment is a well-formed BLAKE3 digest (commitments only — never raw payloads). |
| 6 | `evaluate_profile` | evaluate_profile | Profile `core/v1`: each event carries an `event_type` and a commitment. |
| 7 | `emit_verdict` | emit Verdict | ACCEPT iff every prior stage passed; otherwise REJECT with the first failing stage's reason. |

## Determinism guarantees

- **No wall-clock.** Events are ordered by a monotonic `seq` counter, not
  timestamps. The same inputs always produce the same receipt and the same
  verdict.
- **No RNG, no map iteration order.** Serialized output uses sorted/canonical
  JSON, so hashing is reproducible across runs and machines.
- **The verifier is pure over the receipt bytes.** Given the same receipt it
  always yields the same `Verdict`, and it reads commitments — never raw
  payloads.

## Examples

13 runnable examples cover every major code path:

| Example | Run | What it demonstrates |
|---------|-----|---------------------|
| `admission_gate` | `cargo run --example admission_gate` | Honest receipt admitted; forged receipt refused by name |
| `adversarial_proof` | `cargo run --example adversarial_proof` | Three attack vectors caught by the pipeline |
| `chain_build` | `cargo run --example chain_build` | ChainAssembler from new() to finalize() |
| `chain_growth` | `cargo run --example chain_growth` | Rolling BLAKE3 hash evolution per event |
| `conformance_report` | `cargo run --example conformance_report` | Full discover-then-conform with quality metrics |
| `discover_shapeb` | `cargo run --example discover_shapeb` | Admission-gated process discovery (Shape-B) |
| `full_pipeline` | `cargo run --example full_pipeline` | Cross-product coherence: 6 hops end-to-end |
| `multi_object_receipt` | `cargo run --example multi_object_receipt` | Multi-object events with qualified refs |
| `observable_spans` | `cargo run --example observable_spans` | OTel span emission from verify() |
| `ocel_events` | `cargo run --example ocel_events` | Building and validating OCEL events |
| `receipt_determinism` | `cargo run --example receipt_determinism` | Same events always → same receipt |
| `verdict_diagnostics` | `cargo run --example verdict_diagnostics` | Verdict → LSP Diagnostic mapping |
| `verify_stages` | `cargo run --example verify_stages` | Each of the 7 pipeline stages in detail |

## Benchmarks

The Criterion benchmark suite measures chain assembly performance:

```bash
cargo bench
```

Representative results:
- `chain_append` (single event fold): ~2.4 µs
- `chain_finalize/10` (10-event seal): ~20.3 µs

Benchmarks are deterministic and do not depend on wall-clock time.

## Layout

```
src/lib.rs          module decls + re-exports
src/main.rs         affi CLI entry (clap parsing + dispatch)
src/types.rs        shared types: OperationEvent, Receipt, Verdict, CheckOutcome, ProfileId, Blake3Hash
src/ocel.rs         OCEL event/object/relationship model + builders
src/chain.rs        receipt assembly: rolling BLAKE3 chain hash, serialize/deserialize, persistence
src/verifier.rs     the 7-stage certify pipeline
src/admission.rs    admission gate: OCEL court + chain verifier → AdmittedReceipt
src/discovery.rs    process discovery + conformance metrics (wasm4pm integration)
src/lsp.rs          verdict → LSP diagnostics (lsp-max integration)
src/tracing.rs      observable span emission (OTel integration)
src/cli.rs          command impls: emit, assemble, verify, show, inspect, ...
examples/           13 runnable examples (cargo run --example <name>)
benches/            Criterion benchmark suite (cargo bench)
tests/              integration + adversarial + UI (compile-fail) + witness tests
```
