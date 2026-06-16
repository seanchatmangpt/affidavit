# affidavit

**The Provenance Layer.** `affi` CLI · v26.6.14 · **1000x Initiative Complete**

`affidavit` assembles and certifies **provenance receipts**: append-only,
content-addressed chains of operation-events that record what a process did.
The `affi` binary lets you emit events, finalize them into an immutable
receipt, and verify that receipt against a fixed format standard.

## 🚀 1000x Initiative: Combinatorial Maximalism

The project has successfully completed the **1000x Initiative**, integrating 30+ new features across 6 library areas to provide a world-class developer experience.

- **10x Faster Tests:** Fixture-driven receipt generation.
- **10x Faster Feedback:** Automated mutation testing.
- **10x Easier Adoption:** Shell completion & ontology-driven help.
- **10x More Confidence:** Process discovery & conformance scoring.

**See [wip/documentation_maximalist.md](wip/documentation_maximalist.md) for the full tutorial suite of all 30 features.**

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

## Install / build

Stable Rust, edition 2021. From the repo root:

```bash
cargo build --all-features           # builds the affi binary with 1000x features
cargo test                           # 60+ tests (30 core + 30 e2e)
```

Run the binary either as `cargo run --bin affi -- <args>` or directly from
`target/debug/affi`.

## CLI surface (1000x Expanded)

| Command | Purpose |
| --- | --- |
| `affi emit` | Append an operation-event to the working receipt. |
| `affi assemble` | Finalize into an immutable receipt file. |
| `affi verify` | Run the 7-stage certify pipeline. |
| `affi show` | Human-readable dump of the chain. |
| `affi receipt inspect` | **NEW** Detailed structural analysis. |
| `affi receipt model` | **NEW** Auto-generate DFG/Petri model. |
| `affi receipt conform` | **NEW** Score against process laws. |
| `affi mutate receipt` | **NEW** Stress-test verifier with chaos. |
| `affi bench throughput` | **NEW** Scaling & regression detection. |
| `affi shell` | **NEW** Interactive provenance REPL. |

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

## Layout

```
src/lib.rs        module decls + re-exports
src/main.rs       affi CLI entry (clap parsing + dispatch)
src/types.rs      shared types: OperationEvent, Receipt, Verdict, CheckOutcome, ProfileId, Blake3Hash
src/ocel.rs       OCEL event/object/relationship model + builders
src/chain.rs      receipt assembly: rolling BLAKE3 chain hash, serialize/deserialize, persistence
src/verifier.rs   the 7-stage certify pipeline
src/cli.rs        command impls: emit, assemble, verify, show
examples/golden_run.sh   end-to-end smoke (ACCEPT then REJECT)
```
