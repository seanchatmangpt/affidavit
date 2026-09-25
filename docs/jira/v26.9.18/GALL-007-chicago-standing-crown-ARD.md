# ARD v26.9.18 — GALL-007: Chicago Standing Crown

**Status:** FINAL_SPEC — closed for v26.9.24
**Implementation standing:** MERGED@a45ed48f9554ed62ef9f09b85fd3c4f4960d73c9 (PR #76; rust CI build-and-test, clippy -D warnings, rustfmt green on that exact head)
**Release:** v26.9.18
**Repository:** `seanchatmangpt/affidavit`
**Owner:** affidavit
**Dependencies:** GALL-001..006 exact receipts
**Authority ceiling:** EVIDENCE/STANDING only; never manufacture missing evidence

## Architecture objective

One content-addressed crown receipt states, gate by gate, exactly what the GALL-001..006 evidence proves and issues the strongest lawful standing for that exact composition subject.

## Components

- Rust `affidavit` receipt verification core
- new GALL crown manifest/receipt parser
- 12-gate evaluator
- evidence ceiling/standing issuer
- `affi` CLI surface for crown verify
- fixture corpus containing valid and mutated predecessor bundles

## Control/data flow

`GALL-001..006 receipts + immutable composition -> digest/signature/schema verification -> 12 independent gate verdicts -> evidence ceiling -> final crown receipt`

## Invariants

1. Admit exact GALL-001..006 repo SHAs, receipt digests and shared composition identity.
2. Evaluate all 12 Chicago gates independently; no aggregate green flag can hide an OPEN/REFUSED gate.
3. Require anti-vacuity witnesses: a gate PASS means the relevant forbidden attempt was actually exercised where applicable.
4. Never execute missing manufacture, planning, DO, observation or fresh-consumer work inside affidavit.
5. Preserve evidence classes: inspection, local execution, hosted CI, runtime, merge and publication.
6. Emit a deterministic final receipt and refusal diagnostics suitable for offline verification.

## Failure/refusal boundaries

- Any predecessor mismatch => REFUSED
- Missing required gate evidence => OPEN/PARTIAL, never inferred PASS
- Receipt parser cannot verify subject => UNSUPPORTED/BLOCKED
- Conflicting predecessor subjects => REFUSED

## Qualification court

- cargo fmt --all -- --check
- cargo test
- focused GALL crown integration test using real receipt bundle
- `affi` CLI verify against positive and mutation fixtures

## Evidence boundary

Every PASS binds exact producer and predecessor identities. A changed SHA, mapping, model, lockfile, observation projection or runtime subject is a changed subject. A gate is PASS only from observed execution and its required falsifier, never from absence of evidence.

## Authority law

[
Received \neq Admitted,\quad Candidate \neq Authority,\quad SELECT \neq CONSTRUCT \neq DO
]

No component may gain authority merely because it generated, predicted, observed, validated, replayed or parsed something.

## Closure

Architectural closure requires the positive witness plus each named negative witness on the exact v26.9.18 subject.
