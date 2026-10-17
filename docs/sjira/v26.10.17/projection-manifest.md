# PROJECTION-MANIFEST — affidavit — v26.10.17

How wave order AFFI-261017-01 (`work-order.md`, same directory) is projected into
code and receipts through the ggen_igniter semantic-jira pack, what the mapped
outputs are, and what is NOT yet automatable with the pack as it exists on disk
(pack v26.9.19; survey of `/Users/sac/ggen_igniter`, read-only, 2026-10-17).

## 0. Surfaces read (receipts, not memory)

- `/Users/sac/ggen_igniter/priv/ggen/semantic-jira-pack/pack.toml` — `name = "semantic-jira-pack"`, `version = "26.9.19"`
- `priv/ggen/semantic-jira-pack/ontology.ttl` — the `sj:` vocabulary
- `priv/ggen/semantic-jira-pack/shapes/work-order.shacl.ttl` — admission shapes
- `priv/ggen/semantic-jira-pack/gates/*.rq` (010–066) and `verify/*.unbound.rq`
- `priv/ggen/semantic-jira-pack/templates/*.eex` and `runtime-templates/descriptor.json.eex`
- `/Users/sac/ggen_igniter/lib/ggen_igniter/semantic_jira.ex` (1533 lines) — `admit_work_order/1`, digest, closed projection/receipt-class lists
- `/Users/sac/ggen_igniter/lib/mix/tasks/semantic_jira.{bootstrap,execute,frontier,reconcile,xaas_receipt,prov,court_map,descriptor}.ex`
- `/Users/sac/ggen_igniter/lib/mix/tasks/ggen_igniter.sync.ex` — `ggen sync run` reads `ggen.toml` off disk
- `/Users/sac/affidavit/docs/sjira/v26.10.17/work-order.md` — the wave order, currently Markdown only

## 1. What the pack consumes (RDF work-order shape)

Class: `sj:WorkOrder`, namespace `https://ggen-igniter.dev/ontology/semantic-jira#`.

Kernel-required fields (`semantic_jira.ex` `@required`): `identity, title,
description, subject, repository, base_sha, standing, evidence_ceiling,
promotion_rule, replay_identity, required_courts, required_evidence, acceptance,
falsifiers, projections, origin_authority`.

TTL-side law (ontology.ttl + work-order.shacl.ttl, as read):

- Exactly one `sj:originAuthority` → an admitted `sj:CodeWorkAuthority`
  individual (`sj:WorkOrderOriginShape`: `sj:StrategicObjective` or
  `sj:GoalCheckpoint`) carrying `sj:admissionDigest "sha256:<64 lowercase hex>"`.
  Prose artifacts (`sj:Proposition`, `sj:ProseObservation`) can never originate an
  order. `GgenIgniter.SemanticJira.Authority.verify_origin/3` recomputes the
  sha256 over the admitted source bytes before manufacture.
- `sj:baseSha` matches 40 lowercase hex; `sj:replayIdentity` unique across orders;
  ALIVE standing requires `sj:candidateSha` + `sj:subjectSha` + `sj:receipt`
  (SHACL SPARQL: "ALIVE requires exact candidate/subject SHA and a durable receipt").
- `sj:provider` is FORBIDDEN on an order (gate 047 fires on any use); provider
  identity only via `sj:capabilityId "provider:id"`.
- Dependency edges: `sj:DependencyEdge` with `sj:dependencyType` in
  `requires{SemanticIdentity,Projection,Capability,Receipt,Runtime,Verifier,Observation,Postcondition,Publication}`
  and `sj:upstreamWorkOrder`.
- `sj:ProjectionSpec` must carry `sj:authorityClaim "NONE"` (SHACL).
- Optional: `sj:requiresGitGroundTruth true` makes baseSha verify against a real
  git work tree even without the sync task's `--verify-base-sha` opt-in.

## 2. ggen.toml manifest entries + commands

Step 1 is authoring the RDF — the order is currently Markdown only. New file:
`/Users/sac/affidavit/ontology/affi-work-order-261017.ttl`, one `sj:WorkOrder`
individual `affi:AFFI-261017-01` carrying: `dcterms:identifier "AFFI-261017-01"`,
title "Certify the 26.10.17 release state", `sj:repository
"seanchatmangpt/affidavit"`, `sj:baseSha` (full 40-hex of the wave base
`568beb9`, resolved at authoring time), `sj:standing "UNKNOWN"`,
`sj:evidenceCeiling`, `sj:authorityCeiling "CONSTRUCT"`, `sj:promotionRule`
(from work-order.md), `sj:replayIdentity "affidavit:26.10.17:AFFI-261017-01"`,
court/evidence/acceptance/falsifier individuals for acceptance rows 1–6, and
`sj:projections` from the 15 closed classes (`jira wbpr prd ard vision fond hddl
sa2a a2a_agent_card worker verification executive machine receipt replay` —
`projection_types/0`), plus one `sj:originAuthority` → a new `sj:GoalCheckpoint`
individual for the v26.10.17 wave whose `sj:admissionDigest` is `sha256:` over
the wave's admitted source bytes.

`ggen.toml` additions:

```toml
[ontology]
path = "ontology/affi-work-order-261017.ttl"

[packs.semantic-jira-pack]
# sj:ProjectionSpec carries authorityClaim "NONE" (SHACL law)
```

Commands, in order:

1. `mix ggen_igniter.shacl --ontology ontology/affi-work-order-261017.ttl` —
   SHACL admission court before any render (origin authority, ALIVE-crown,
   uniqueness, dependency-edge lexical law).
2. `ggen sync run` (or `mix ggen_igniter.sync --pack semantic-jira-pack:jira`) —
   binds the pack's gate queries (010 subjects, 020 work orders, 030 properties,
   040 nodes) as template bindings (`work_orders`, `work_order_subjects`,
   `properties`, `nodes`); the pack's `verify/*.unbound.rq` companions are
   offender-reporting — any row is a refusal
   (`REFUSED:SEMANTIC_JIRA_INVALID_WORK_ORDER`), enforced inside the templates
   themselves (jira.md.eex raises on alive-crown and description-integrity rows).
3. `mix ggen_igniter.verify` — the pack gate set (045 description integrity,
   046 ALIVE receipt crown, 047 provider neutrality, 050 frontier, 055 standing
   projection, 060–066 A2A/monotonic-evolution) as the admission court plane.

## 3. Mapped outputs (read from the templates' front matter)

| source | output path | output |
|---|---|---|
| `templates/jira.md.eex` (front matter: `to: docs/jira/<%= id %>.md`, `for_each: work_orders`) | `docs/jira/AFFI-261017-01.md` | Jira-shaped Markdown ticket |
| `templates/ocel.json.eex` (`to: docs/semantic-jira/ocel.json`, no for_each) | one OCEL log document per manufacture run, covering every WorkOrder | object-centric event log |
| `templates/plan.hddl.eex` + `plan-domain.hddl.eex` | HDDL domain + plan for the wave | planning artifacts |
| `templates/wbpr.md.eex`, `prd.md.eex`, `vision.md.eex`, `ard.md.eex`, `projection.md.eex` | WBPR / PRD / vision / ARD / projection Markdown | documentation projections |
| `templates/a2a_agent_card.json.eex` + `a2a_manufacture.ex.eex` | A2A agent card JSON + manufacture module | protocol surface |
| `runtime-templates/descriptor.json.eex` | execution descriptor JSON | runtime descriptor |
| `mix semantic_jira.reconcile` + `mix semantic_jira.xaas_receipt` | reconciler state; maps a sealed XaaS receipt onto the reconciler-consumable shape | receipt-class state |
| `mix semantic_jira.prov` | PROV event log | provenance events |
| `SemanticJira.digest/1` | canonical JSON sha256 of the normalized order | projection-run digest |

The last desired output for this wave — per-gate receipt JSONs (just validate
exit/test counts, wasm pin receipt, golden_run.sh ACCEPT/REJECT pair) landing in
`docs/sjira/v26.10.17/` — is NOT in this table because no template emits it. See §4.

## 4. NOT automatable — UNSUPPORTED(projection-capability)

Explicit list, with the pack extension each needs:

1. **Court-receipt JSONs from the gate plane.** No template consumes real gate
   output (`just validate` exit/test counts, `registry_artifacts` pin receipt,
   `examples/golden_run.sh` ACCEPT/REJECT) and renders a receipt JSON into
   `docs/sjira/v26.10.17/`. Receipts today are emitted by affidavit's own chain
   (`affi emit` → `affi assemble` → `affi verify`, BLAKE3 chain) or by hand into
   `evidence-receipts.md`. Pack extension needed: a `court-receipt.json.eex`
   template + a gate asserting every acceptance cell has a receipt file whose
   mtime postdates the baseSha commit time (the KD-COVERS/SIM-PASS gating
   pattern).
2. **Standing promotion.** Advancing `sj:standing` UNKNOWN → ALIVE requires hand
   authoring the ALIVE-crown triples; the `sj:StandingTransition` SHACL shape
   validates from/to standing but nothing in the pack generates transition nodes
   from court output. Pack extension: a standing-transition template consuming
   court receipts.
3. **Lease/worker/authority binding.** SHACL requires `sj:authority` +
   `sj:lease` + `sj:worker` when `sj:authorityRequirement != "NONE"`; there is
   no generator for `sj:lease` objects — the kernel moduledoc states it does not
   issue runtime leases (XaaS/Ultracode-owned; consequential DO is BRCE-owned).
   This wave is in-tree with coordinator-owned git, so the hand-written section
   is: the order ships with `authorityRequirement "NONE"`; any future lease
   binding stays hand-authored.
4. **ALIVE-crown SHA minting.** `sj:candidateSha`/`sj:subjectSha`/`sj:receipt`
   must name real heads/receipts; the pack verifies shape and uniqueness, it does
   not mint them. Pack extension: a witness-minting task (analogous to the
   existing `mix semantic_jira.prov` PROV-log task).
5. **Generated-vs-handwritten classification (recorded fact, not a gap).** The
   pack's templates render docs/HDDL/JSON/module outputs only — no `.rs` template
   exists; all affidavit `src/` is irreducible hand-written residue by pack
   design. No pack extension needed; recorded so the closing receipt can cite it.

## 5. Summary

Automatable today with pack v26.9.19 as-read: RDF → Jira Markdown / HDDL / OCEL /
agent card / descriptor projections; the SHACL + gate admission court set; the
reconciler, PROV event log, and canonical digest. NOT automatable: court-receipt
JSONs from the gate plane, standing promotion, lease binding, and ALIVE-crown SHA
minting — each named above with the pack extension required, standing as
UNSUPPORTED(projection-capability) until those templates/tasks exist.
