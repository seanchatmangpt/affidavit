# CLI Vacuity Audit + 80/20 ERRC — v26.10.17 wave (2026-10-17)

Method: all 101 registry verbs executed against the built `affi` binary across 7
lanes (receipt batches 1-6 + non-receipt batch 7), real invocations with real
fixtures, per-verb classification + evidence. Receipts: this file's tables +
`/tmp/affi-audit-b*` fixtures.

## Verdict distribution (101 registry verbs)

| Class | Count | Meaning |
|---|---|---|
| EXECUTES | 62 | real effect from real state, tamper-sensitive |
| REFUSED-TYPED | 17 | honest typed stop (`REFUSED_UNSUPPORTED … crypto-trust`) — lawful, feature plane |
| THIN | 9 | runs but unfalsifiable / overclaims (list below) |
| VACUOUS | 6 | success unrelated to any input — court cannot flip |
| UNWIRED | 6 | emit-from-* payload-as-path defect — registered, cannot succeed for any input |
| FEATURE-GATED | 4 | receipt validate/chain-verify, dossier seal/verify — ctdd-gated BY DESIGN (Cargo.toml:389, not in default); exposed under `--features ctdd`; batch-6 'unwired' reading of validate/chain-verify corrected by the fresh-binary re-check |

Cross-cutting defects (fixed or queued): stray trailing `null` on success
paths (output layer, all verbs — documented clap-noun-verb B13);
`verify-compliance` format-dependent exit (human=2, json=0); exit-0-on-
unknown-input (predict, causality-chain, find-blast-radius, explain-incident,
monitor --rules); `doctor` reachable only as `affi affi doctor`.

## VACUOUS six (the eliminate set)

1. `receipt audit` — stdout is literally `null`; canned progress to stderr.
2. `receipt license-compliance` — never compares extracted licenses against
   `allowed_licenses`; verdict independent of license content.
3. `receipt test` — self-described dummy verb for ontology validation.
4. `certificate fre-902` — canned affidavit, zero world-state input.
5. `legal-hold status` — certifies FRCP 37(e) hold over ZERO chains and
   fabricates `events_preserved: 1`.
6. `restitution calculate` — identical $1.25M/$625K for every tariff schedule;
   flag changes the label only.

## THIN nine (raise-or-descope)

`gdpr-proof`, `hipaa` (unfalsifiable prose proofs), `receipt attest`
(help says "signed", output unsigned), `predict` (exit 0 on unknown type,
1.0 success rate from empty corpus), `variance`, `explain-incident`,
`causality-chain` (no reachability predicate), `verify-sla` (only min_events
enforced; help overclaims MTTR/two-person), `install-git-hook`+`monitor`
(violations never flip an exit).

---

# 80/20 ERRC

## The 20% (verified load-bearing — protect)

The receipt lifecycle core (emit, assemble, verify, show, inspect, stats,
why, fix, diagnose, diff, graph, replay — all 12 EXECUTES with a working
tamper court: verify/why/diagnose/fix all flip on one mutation) + the
federation surface (standing certify/verify, errc certify/verify/
verify-assurance, ecosystem certify/verify — ACCEPT/REJECT roundtrips with
typed schema escalation) + sbom family + policy-enforce/verify-family.
~22 verbs deliver essentially all operator value witnessed. These are
genuinely non-vacuous.

## ELIMINATE (delete outranks fix)

- `receipt test` — dummy verb; delete from registry + ontology (2 sites).
- `receipt audit` — stdout `null`; delete or make it scan and report; today
  it is a canned string, which is worse than absence in a certify-don't-
  decide tool.
- VACUOUS statutory trio (certificate fre-902 / legal-hold status /
  restitution calculate) — fabricating evidentiary outputs is the single
  worst defect class in a provenance tool. Either wire each to a real
  falsifiable check (legal-hold: refuse REFUSED_NO_CHAINS on empty input;
  fre-902: recompute the supplied CTRT chain before certifying; restitution:
  derive figures from a supplied schedule or refuse) or cut them. Default:
  wire legal-hold + restitution to refusals (small), cut nothing until the
  fix lanes land.
- `license-compliance` unless the allowed-license comparison lands (one
  real check closes it).

## REDUCE

- 12 default-unreachable crypto-trust verbs (keys/envelope/evidence/dossier
  emit+inspect): reduce default-surface noise — help text should state the
  feature requirement at `--help` time, not only at exec; dossier
  seal/verify compiled out entirely (ctdd gate) should be visible in help
  as feature-scoped, not silently absent from the registry count.
- Format-enum drift (profile/receipt-throughput/monitor/catalog/audit ignore
  `--format`): one shared Output format enum at the noun layer; delete the
  per-verb ad-hoc flags.
- The trailing-`null` output-layer bug: one fix, kills a defect on ~40 verbs
  (documented upstream B13; federation courts exempt).

## RAISE

- emit-from-* (6 verbs): payload staging via `emit_with_payload` (repair
  lane in flight) — turns 6 UNWIRED into EXECUTES.
- Typed refusals on unknown input: predict/causality-chain/find-blast-radius/
  explain-incident/monitor must exit non-zero on unknown type/start-event/
  change-event/rule-set (currently silent-empty exit 0).
- `verify-compliance`: exit code must be format-independent (verdict decides,
  not the renderer).
- `monitor`/`install-git-hook`: CRITICAL violations must set the hook's
  enforcement exit (the `exit 1` is literally commented out).

## CREATE

- Top-level `affi doctor` routing (currently `affi affi doctor` only — wired, namespaced; batch-7 re-check corrected the 'unwired' belief).
- Feature-aware `--help` (`requires: crypto-trust` on the 17 REFUSED-TYPED).
- Vacuity tripwire court: a test that runs every registered verb with
  minimal fixtures and asserts NO verb returns success whose output is
  byte-identical across two different inputs (the VACUOUS-class detector,
  mechanical version of this audit). Highest-leverage single artifact from
  this audit — lands the whole class as a permanent court.

## Sequencing

1. Repair lanes already dispatched: emit-from-* payload staging.
2. Dispatch: legal-hold/refuse-empty + license-comparison + VACUOUS tripwire
   court + trailing-null + format-exit unification (each is small).
3. Then: registry/ontology removal of `receipt test` (ggen re-render) +
   help-time feature annotations.
4. Re-run this audit script post-fixes; target: 0 VACUOUS, 0 UNWIRED,
   THIN ≤ 4 before the 26.10.17 tag.
