# affidavit v26.9.16 — RFC Closure Contract

Status: DRAFT IMPLEMENTATION PR.

## Canonical Jira tickets

- A2A-2607 — Blue River Dam cross-repo closure
- A2A-2612 — machine-experience compile-back qualification

## RFC ownership

This repo owns evidence standing, tamper-resistant receipt verification, and the qualification boundary for cross-repo DME/SA2A claims.

## Required closure

1. Define a v26.9.16 evidence profile that can bind exact repository revision, semantic subject, manufacturer/projection identity, authority evidence, resource-allocation receipt, execution receipt and post-state attestation.
2. Distinguish local verification, hosted CI, merge, publication, deployment and runtime ALIVE standing.
3. Make cross-repo closure compositional: a higher-level crown may reference lower-level exact-subject receipts but must fail if any required subject is missing, moved or unqualified.
4. Qualify machine-experience promotion separately from successful exploratory inference; only admitted/qualified reusable machinery earns KNOWN standing.
5. Preserve tamper detection and deterministic verdict bytes.

## Chicago falsifiers

- substitute one repository revision while preserving the aggregate standing receipt;
- omit one required lower-level subject and still obtain closure;
- claim ALIVE from compile/test evidence alone;
- promote model output to machine experience without admission/qualification evidence;
- mutate a receipt field without invalidating verification;
- replay evidence in a way that re-actuates consequence.

## Definition of done

Exact-head adversarial courts demonstrate fail-closed cross-repo composition, revision binding, machine-experience qualification, tamper detection and explicit standing ceilings. This repo certifies evidence; it does not manufacture semantic truth or DO authority.
