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
| `fixtures/aloop/golden_case/` | committed golden case (2 receipts, 3-event sha256 chain); a projection of `emit-golden`, never hand-edited |
| `tools/tests/test_aloop_falsifiers.py` | Chicago pytest suite: golden-projection drift guard, profile conformance, adversarial corpus, benchmark regression bound |
| `tools/bench/bench_aloop_falsifiers.py` | deterministic benchmark (synthetic conformant cases 1k/10k/50k events) |
| `tools/bench/aloop_falsifiers.bench.json` | committed bench receipt (wall + process-CPU µs/event) |

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

## The eleven falsifiers + F00 profile gate (each witnessed by `tools/aloop_falsifiers.py self-test`)

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

| F00 | receipt does not conform to `schemas/aloop-execution-receipt.schema.json` (requires `jsonschema`; otherwise `profile_validation: SKIPPED(jsonschema-missing)` is reported, never hidden) | `mu_on_O` |

Hardening (v26.9.26) widened the existing falsifiers to close false-accepts and
crashes found by an adversarial corpus:

* malformed input (non-JSON / non-object lines or receipt files, invalid UTF-8,
  non-integer or boolean `seq`) is a typed F05/F01 refusal, never a traceback;
* F01 also refuses a chain that does not start at seq 0 (truncated prefix re-chained)
  and a receipt that claims an event id absent from the chain (phantom claim);
* F02 is strictly monotonic (a duplicated seq is not an append order);
* F03 refuses an event with no `actuation_id` (unattributable actuation);
* F04 refuses one event claimed by several receipts and one work order receipted
  twice (duplicate delivery);
* F05 refuses duplicate `event_id`s, an unsupported `hash_algo` (explicitly, not as a
  silent mismatch) and a `replay_binding.chain_head_hash` that is not the hash of the
  receipt's last claimed event (replay mismatch);
* F08 also joins `provider_execution_id` between each receipt and the events it claims;
* F09 refuses a claim across work orders and a non-40-hex event `subject_sha`;
* F11 compares RFC 3339 instants (offset-normalized), refuses naive/unparseable
  timestamps, and only `reconstructed: true` (the boolean) is an honest marking.

Court round 2 (affidavit#85 at `01d17e40`) closed five false-accept classes and one
destructive-IO defect:

* **clause ids + anti-vacuity**: every violation carries `clause` (`F<nn>.<name>`).
  `tools/tests` pins every clause by id (a row whose refusal is that clause), and
  `test_every_refusal_clause_has_a_clause_pinned_witness` reads the clause set from the
  verifier's source, so a clause added without a witness fails the suite. A sweep that
  replaced each of the 39 refusal clauses (plus F00 and 5 guard mutants) with `pass`
  killed 45/45 (at `01d17e40` 10/30 clauses survived, including `F05.prev_link`).
* **consequence binding (F06)**: `consequences[].hash` must equal the set of the claimed
  events' `consequence_hash` (`F06.unbound_consequence` / `F06.unreceipted_consequence`),
  and `consequence.{commits,files_changed,remote_effects}` must equal the union of the
  claimed events' same-named lists (`F06.field_mismatch`). A receipt can no longer
  certify a consequence no event produced.
* **omission is refused**: every event field a falsifier reads is required and owned by
  that falsifier (`F<nn>.event_field_missing`): subject_sha/work_order_id -> F09,
  provider/provider_execution_id -> F08, actor/authority_grant -> F10,
  consequence_hash -> F06, ts/recorded_at -> F11, event_type/hash/hash_algo -> F05;
  `prev_hash` must be present (null only at the root). F08 also joins the event
  `provider` with the claiming receipt's `provider.name`; F09 compares subjects
  whenever an event is claimed (a missing receipt subject is a mismatch).
* **no self-waiver**: `reconstructed: true` waives only the F11 timing check. An
  unclaimed event is F07 however it is marked; the report counts `reconstructed`
  and `reconstructed_unclaimed` (the latter are also UAR).
* **chain binding**: `replay_binding.chain_head_hash` is required by the profile and by
  the checker (`F05.head_missing`) whenever a receipt claims events.
* **input**: duplicate JSON keys are a parse refusal (parser-dependent payload); a
  `predecessor_work_order_ids` entry naming an absent receipt is `F01.dangling_predecessor`
  (and listed in the DAG's `dropped`).
* **tail anchor**: a suffix truncation that also drops the tail's receipt is
  undetectable from inside a case (the chain has no internal anchor for its own end).
  `check <case> --anchor <tail-hash>` compares against an externally held tail hash
  (`F01.anchor`); without it the report states `anchor: NONE(...)`.
* **safe emit**: `emit-golden <dir>` removes only a previous case
  (`receipts/*.json`, `events.ndjson`); any other entry is
  `REFUSED(UNSAFE_OVERWRITE)` (exit 2) and nothing is deleted.

The committed golden receipts previously failed their own profile schema (missing
`commands`, `evidence`, `exit_status`, `timestamps`); `golden_case()` now emits
profile-valid receipts and the fixture was regenerated with `emit-golden`.

An event marked `reconstructed: true` is exempt from F11 only (honest post-hoc
reconstruction); it is counted (`reconstructed`) and must still be claimed by a
receipt, so the honest count — including RECONSTRUCTED — survives without letting the
subject waive ZeroUnreceiptedActuation.

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

`tools/aloop_falsifiers.py check <case_dir> [--anchor <tail-hash>]` over a directory of lane receipts +
event chains prints a JSON verdict with `uar_count` (F07 violations) and
`reconstructed_unclaimed`, exit 1 if refused.

## Verification

```bash
python3 -m pytest tools/tests -q           # needs pytest + jsonschema
python3 tools/aloop_falsifiers.py self-test
python3 tools/bench/bench_aloop_falsifiers.py --out tools/bench/aloop_falsifiers.bench.json
```
