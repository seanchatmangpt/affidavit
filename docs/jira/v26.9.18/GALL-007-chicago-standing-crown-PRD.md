# PRD v26.9.18 — GALL-007: Chicago Standing Crown

**Status:** FINAL_SPEC — closed for v26.9.24
**Implementation standing:** MERGED@a45ed48f9554ed62ef9f09b85fd3c4f4960d73c9 (PR #76; rust CI build-and-test, clippy -D warnings, rustfmt green on that exact head)
**Release:** v26.9.18
**Repository:** `seanchatmangpt/affidavit`
**Owner:** affidavit
**Dependencies:** GALL-001..006 exact receipts
**Authority ceiling:** EVIDENCE/STANDING only; never manufacture missing evidence

## Product outcome

One content-addressed crown receipt states, gate by gate, exactly what the GALL-001..006 evidence proves and issues the strongest lawful standing for that exact composition subject.

## Problem

Even six successful component checkpoints do not automatically imply cross-repository conformance. A separate evidence-only issuer must evaluate the immutable composition against the 12 Chicago gates without repairing, actuating or inferring missing proof.

## Functional requirements

1. Admit exact GALL-001..006 repo SHAs, receipt digests and shared composition identity.
2. Evaluate all 12 Chicago gates independently; no aggregate green flag can hide an OPEN/REFUSED gate.
3. Require anti-vacuity witnesses: a gate PASS means the relevant forbidden attempt was actually exercised where applicable.
4. Never execute missing manufacture, planning, DO, observation or fresh-consumer work inside affidavit.
5. Preserve evidence classes: inspection, local execution, hosted CI, runtime, merge and publication.
6. Emit a deterministic final receipt and refusal diagnostics suitable for offline verification.

## Acceptance criteria

1. Old receipt against moved SHA refuses.
2. Missing GALL-006 leaves Gate 11 OPEN and prevents final ALIVE crown.
3. Substituting actuator self-report for GALL-004 fails independent-postcondition gate.
4. Known replay without positive execution fails Gate 12.
5. All supplied gates and falsifiers are traceable to predecessor receipt digests.
6. Final standing is reproducible offline from the same bundle.

## Evidence product

The checkpoint MUST emit a content-addressed machine-readable receipt/artifact binding exact producer SHA, input/predecessor identities, executed court, falsifiers, outputs, standing and evidence ceiling. Source presence or prose is not standing.

## Non-functional requirements

- Deterministic identity for identical admitted inputs.
- Typed UNKNOWN/PARTIAL/REFUSED/BLOCKED states.
- No ambient authority or undeclared dependency.
- Exact-head subject fencing and replayable evidence.
- No promotion of observation, model output, parsing or configuration into stronger standing.

## Definition of done

One content-addressed crown receipt states, gate by gate, exactly what the GALL-001..006 evidence proves and issues the strongest lawful standing for that exact composition subject.
