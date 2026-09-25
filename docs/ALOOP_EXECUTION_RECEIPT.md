# ALOOP ExecutionReceipt profile (lane 8, ALOOP-ZCODE-DOGFOOD-001)

Machine-readable binding of the ALOOP autonomous-execution contract onto the dfcm
receipt (R = identity, authority, consequence, replay, standing).

**Doctrine: the affidavit certifies what occurred; it NEVER decides planning policy.**

## Artifacts

| path | role |
|---|---|
| `schemas/dfcm-receipt.schema.json` | vendored byte-copy of the canonical dfcm R v2 schema (`~/.zcode/dfcm/receipt.schema.json`) |
| `schemas/aloop-execution-receipt.schema.json` | ALOOP profile: ExecutionReceipt = R v2 + execution fields + namespaced provider extensions; dfcm schema inlined at `$defs/dfcmReceipt` (self-contained, zero network refs) |
| `tools/check_dfcm_schema_sync.py` | byte-sync guard: vendored == `~/.zcode/dfcm` == `~/.claude/dfcm` twin; profile inline == vendored |
| `tools/aloop_falsifiers.py` | the eleven ZeroUnreceiptedActuation falsifiers + causal-DAG derivation + cycle/bridge detection + deterministic golden case |
| `fixtures/aloop/golden_case/` | committed golden case (2 receipts, 3-event sha256 chain) |

## Contract binding

`ExecutionRequest{work_order, capability_requirements, subject, authority,
evidence_requirements, resource_constraints}` is discharged by an
`ExecutionProvider{capabilities, transport, availability, cost, concurrency,
authority_ceiling, receipt_protocol}` and certified by the normalized
`ExecutionReceipt{work_order_id, origin_authority, provider, provider_execution_id,
subject_before, subject_after, commands, consequences, evidence, exit_status,
timestamps, replay_binding}` + namespaced ext.

Mapping onto R v2 (`~/.zcode/dfcm/receipt.schema.json`):

* new **required** top-level: `work_order_id`, `origin_authority`, `provider`,
  `provider_execution_id`; optional `subject_before`/`subject_after` digests
  (`sha256:<64hex>`);
* `commands` ↔ `replay.commands`, `consequences` ↔ `consequence`, verification
  ladder → `evidence`, chain claim → `replay_binding.{event_ids, chain_head_hash,
  predecessor_work_order_ids}`;
* extensions are legal **only** as `provider_ext.<provider>` objects. The dfcm schema
  top level stays open on purpose; the dfcm **validator** refuses un-namespaced
  extension keys with `R_missing_identity` (typed, in the gate, not the schema);
* `identity.subject_sha` keeps git-commit semantics and is verified by
  `git cat-file` against `identity.repo`. Non-commit subjects (sha256/blake3 pack
  digests) route through the explicit `identity.subject_digest` field and never
  through `subject_sha`. A repo that cannot be locally verified is REFUSED
  (fail-closed), never silently passed.

## The eleven falsifiers (each witnessed by `tools/aloop_falsifiers.py self-test`)

| # | falsifier | broken term |
|---|---|---|
| F01 | missing event (seq gap) | `R_missing_replay` |
| F02 | reordered event | `mu_on_O` |
| F03 | duplicate actuation | `R_missing_authority` |
| F04 | duplicate consequence | `admission_vacuous` |
| F05 | broken chain (hash/prev_hash) | `R_missing_identity` |
| F06 | receipt without consequence | `R_missing_consequence` |
| F07 | consequence without receipt (unreceipted actuation → **UAR count**) | `mu_unlawful` |
| F08 | provider identity rewrite | `R_missing_identity` |
| F09 | subject mismatch | `R_missing_identity` |
| F10 | authority mismatch | `R_missing_authority` |
| F11 | post-hoc fabricated event | `R_not_fed_back` |

An event marked `reconstructed: true` is not refused (honest post-hoc
reconstruction); it is counted separately in the report so the honest count —
including RECONSTRUCTED — always survives.

## Causal DAG

`tools/aloop_falsifiers.py dag <case_dir>` emits `{nodes, edges, cycles, bridges}`:
nodes are events (`E:<event_id>`) and receipts (`R:<work_order_id>`); edges are
`seq` (event → predecessor), `claimed_by` (event → receipt),
`predecessor` (receipt → prior receipt), `binds` (receipt → chain-head event).
Cycle detection runs over the directed causal edges (`seq`, `predecessor`,
`claimed_by`; Kahn). Bridge detection runs over the undirected causal spine
(`seq`, `predecessor`, `binds`; Tarjan) — a bridge is a single point of chain
failure. `claimed_by` edges are excluded there: they saturate every undirected
cycle and would hide real weak points.

## Dogfood

`tools/aloop_falsifiers.py check <case_dir>` over a directory of lane receipts +
event chains prints a JSON verdict with `uar_count` (F07 violations) and
`reconstructed_unclaimed`, exit 1 if refused.
