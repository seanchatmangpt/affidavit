# Omniplatform: cgap + ash_pplan + affidavit

Three repos, one evidence path. Each keeps its own doctrine:

| Repo | Role | What it contributes |
|---|---|---|
| `seanchatmangpt/cgap` | sovereign substrate, judges itself with fail-closed gates | gate runs (exit 0/1/2/3) |
| `seanchatmangpt/ash_pplan` | P-PLAN/PROV-O process control plane over Reactor | `AshPPlan.ExecutionReceipt` observations |
| `seanchatmangpt/affidavit` | provenance layer: **certify, don't decide** | seals both into BLAKE3 receipt chains and verifies them |

Producers observe, affidavit certifies, nobody decides for anyone else. The
seam is a file format, not a library dependency: no repo links another, so the
`no_std` substrate, the BEAM control plane and the Rust CLI stay independent.

## The seam: omni-evidence/v1 bundles

```
<bundle>/batch.json         [{"event_type","objects","payload"}...]
<bundle>/payloads/<file>    raw payload bytes, one file per event
```

- Schema: [`schemas/omni-evidence-batch.v1.schema.json`](../../schemas/omni-evidence-batch.v1.schema.json).
- `payload` is a path relative to the bundle directory. `affi receipt
  emit-batch` resolves it against the batch file's directory when it does not
  exist from the cwd, so a bundle stays valid wherever it is copied.
- `objects` are affidavit refs `id:type[:qualifier]`. Producers derive a
  colon-free id and carry raw identity (IRIs, commands) in the qualifier.
- Producers never hash. affidavit computes `blake3(payload bytes)` at emit and
  folds the chain at assemble, so the commitment binds exactly the bytes the
  producer wrote.

| Producer | Emitter | event_type | payload |
|---|---|---|---|
| cgap | `python3 scripts/affidavit_bundle.py --out DIR [--gate NAME=CMD]...` | `cgap.gate.{pass,fail,missing,skip,unknown}` | canonical JSON: gate, command, exit, verdict, output, subject sha |
| ash_pplan | `AshPPlan.AffidavitBundle.entries(receipts) \|> write!(dir)` | `pplan.execution.{succeeded,halted,failed,unknown}` | `ExecutionReceipt.to_rdf/2` N-Triples (carries the outcome and release-binding digests) |

## Certify a bundle

```bash
mkdir work && cd work
affi receipt emit-batch --batch-file ../bundle/batch.json
affi receipt assemble --out receipt.json
affi receipt verify receipt.json     # exit 0 ACCEPT / 2 REJECT
```

## Courts

- affidavit `tests/omni_bundle_e2e.rs`: both fixture bundles
  (`fixtures/omni/`) seal and ACCEPT; a retyped event REJECTs (positive
  control); every commitment equals `blake3` of the producer's payload file.
- cgap `python3 scripts/affidavit_bundle.py --self-check`: verdict mapping,
  exit codes in payloads, qualifier identity, byte determinism.
- ash_pplan `test/affidavit_bundle_test.exs`: event types, colon-safe refs,
  payload = `to_rdf/2`, deterministic render, layout on disk.
