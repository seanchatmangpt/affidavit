# ERRC Claim Assurance — Evidence Completeness Refinement

Status: normative for `affidavit/errc-claim-assurance/v1`.

## 1. Source and purpose

This refinement reconstitutes the mature widened-claim rule from the exact
archaeological source:

- repository: `seanchatmangpt/ggen-legacy`
- commit: `60d38265b8d1d94c43f04ca6bdb8537184e510a8`
- artifact: `governance/claims-register.md`

That register requires a widened claim to carry a named object, exact source
coordinate, verifier, observed result, evidence path, receipt, exclusions, and a
claim ceiling. `affidavit/errc/v1` already supplies the named factor coordinate,
exact subject, observed before/after result, parent receipt, and directional
claim ceiling. This profile adds the missing verifier/evidence/exclusion witness
and proves exact one-to-one coverage of the parent claim set.

## 2. Objects

Let a verified ERRC receipt contain the finite claim-id set

`C = {c_1, ..., c_n}`.

Let a claim-assurance witness be

`w = (claim_id, verifier, locator, observed_result, commitment, exclusions)`.

The witness set is `W = {w_1, ..., w_m}`. Certification requires a bijection

`π : W → C`

where `π(w) = w.claim_id`.

Operationally:

1. every `c ∈ C` has exactly one witness;
2. no witness refers to an id outside `C`;
3. duplicate witness ids are refused;
4. `verifier`, `locator`, and `observed_result` are non-empty;
5. `commitment` is a canonical BLAKE3 digest;
6. `exclusions` is non-empty, canonical, and duplicate-free.

Thus `|W| = |C|` is necessary but not sufficient; exact set equality and
uniqueness are checked.

## 3. Receipt identity

The assurance receipt seals:

- profile identity;
- exact ggen-legacy source coordinate;
- fixed claim ceiling;
- BLAKE3 hash of the verified parent ERRC receipt;
- canonical parent claim-id set;
- canonical witness set;
- canonical BLAKE3 assurance hash.

The parent join is checked by `verify_against(parent)`. Standalone deserialize
can establish internal integrity, but **cannot infer that an external parent
object is the referenced parent**. That requires the explicit join operation.

## 4. Claim ceiling

The fixed ceiling is:

`CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM`

A valid assurance receipt establishes that every declared ERRC claim has the
required evidence-shaped witness and that the witness ledger is bound to one
exact parent receipt. It does **not** establish:

- that the evidence is factually true beyond the verifier's admitted semantics;
- causal attribution;
- utility or global optimality;
- production readiness;
- security/compliance certification;
- business approval;
- authority to actuate.

Those exclusions are structural, not editorial caveats.

## 5. Morphisms and composition

The lawful refinement morphism is

`ErrcReceipt --certify_errc_claim_assurance--> ErrcClaimAssuranceReceipt`.

It is defined only when the parent verifies and the witness relation is a total
bijection over the parent's claim ids. The receipt binds the parent hash, so the
same witness ledger cannot silently migrate to a different candidate or receipt.

No inverse morphism is claimed: evidence completeness does not reconstruct raw
evidence bytes. No transitive truth morphism is claimed: two adjacent valid
receipts do not prove causality or dominance.

## 6. Falsifiers

Certification must refuse:

- missing parent claim witness;
- extra/invented witness id;
- duplicate witness id;
- empty verifier;
- empty evidence locator;
- empty observed result;
- malformed evidence commitment;
- empty exclusion set;
- empty or duplicate exclusion statement;
- non-canonical claim/witness/exclusion order on deserialize;
- wrong profile/source/ceiling;
- tampered serialized material;
- valid assurance checked against a different valid parent receipt.

## 7. Standing

This profile is a **CONSTRUCT** proof surface. It cannot crown repository
`ALIVE`. Execution standing still requires observed execution against the exact
admitted subject plus the repository's independent verification/replay courts.
