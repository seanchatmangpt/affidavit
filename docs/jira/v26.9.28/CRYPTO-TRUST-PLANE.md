# CRYPTO-TRUST-PLANE (2026-09-28)

Boundary decision (operator): **Affidavit owns the ecosystem's cryptographic trust
plane** — keys, custody semantics, canonicalization, PQC, signatures, attestations,
verification, revocation, cryptographic standing. **SA2A owns what that standing is
allowed to authorize.** No authority inversion: `AFFIDAVIT --proof--> SA2A
--authority--> ACTUATOR`, evidence/receipt loops back.

Branch: `feat/v26.9.28-crypto-trust-plane` (at origin/main 7eaf86b).
Companion pack: `ggen-marketplace/packs/affidavit-trust-plane-pack` (manufactured,
not hand-written). Envelope interlock: RFC-SA2A-007-errata (alg/kid/nonce/expiry
inside signed bytes, JCS, revocation epochs, (kid,nonce) replay).

## History (append-only: ts | standing | branch+SHA | gates | remaining)

- 2026-09-28 | ALIVE | 7eaf86b | probe: ggen sync green; pack ontology merges via
  `[ontology].imports`; rule-query columns are the only data binding; verb collision
  (verify x4, certify x3, search x2) fixed by `cnv:fieldName` disambiguation + pack
  COALESCE (retires legacy generate_verbs.py projection) | lanes pending
- 2026-09-29 | ALIVE | feat/v26.9.28-crypto-trust-plane | waves 2-4 integrated:
  22 rendered modules + 4 CLI verbs; verb-pipeline collision fixed (FM-GEN-004 law);
  theater handlers rewired to real crypto; NIST ACVP vectors landed; marketplace
  qualification ok; falsifier: ontology mutation -> rendered diff non-vacuous |
  remaining: enclave live KAT (entitlement), completions regen per verb add
