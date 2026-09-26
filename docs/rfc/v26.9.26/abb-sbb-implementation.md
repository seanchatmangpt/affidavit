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
