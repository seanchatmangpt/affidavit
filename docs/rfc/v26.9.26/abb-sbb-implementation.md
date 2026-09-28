# RFC v26.9.26 — architecture qualification standing seed

## Ownership
affidavit owns evidence/standing receipts for architecture qualification and realization. It does not grant consequential execution authority.

## Definition of done
1. Define deterministic ArchitectureQualificationReceipt and ArchitectureReceipt schemas.
2. Bind receipts to exact ABB, ArchitectureContract, SBB, qualification evidence, producer and artifact digests.
3. Preserve candidate != truth != authority != DO != standing.
4. Add replay verification and receipt-chain integrity.
5. Refuse forged evidence, changed contract, changed SBB digest, cross-subject reuse and UNKNOWN promotion.
6. Record SUPERSEDED standing after SBB replacement/migration.
7. Accept independent evidence from autofde-lab, xaas and runtime/process observations.
8. Provide machine-readable queries for current qualified SBB standing by ABB.

Affidavit establishes evidence standing only; BRCE remains the DO boundary.

## Standing of this seed (schema `affidavit.architecture-qualification.v2`)
| DoD | status | where |
|---|---|---|
| 1 | PARTIAL_ALIVE: qualification receipt only; no separate realization receipt | `src/architecture.rs` |
| 2 | ALIVE: digests re-validated by certify and by integrity/replay/parse | `verify_integrity` |
| 3 | ALIVE: `confers_do_authority` is always false and refused when true | `verify_integrity` |
| 4 | ALIVE: replay, chain link, supersession verification, ledger admission | `verify_replay`, `verify_chain`, `Supersession::verify` |
| 5 | PARTIAL_ALIVE: forged evidence refused against observed bytes (`verify_evidence`); the receipt digest is unkeyed, so producer authenticity is UNSUPPORTED here | `QualificationEvidence` |
| 6 | ALIVE: replacement yields a QUALIFIED successor and a SUPERSEDED retirement record of the replaced SBB | `supersede` |
| 7 | PARTIAL_ALIVE: typed intake for autofde-lab, xaas and runtime evidence bound to bytes; no transport from those repos yet | `EvidenceSource` |
| 8 | PARTIAL_ALIVE: in-memory ledger, `current_qualified(abb)` and JSON `query_json(abb)`; no persistence | `ArchitectureStandingLedger` |
