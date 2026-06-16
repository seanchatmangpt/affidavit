# Affidavit Project Mandates (GEMINI.md)

## Core Architectural Rules
- **ADR-1: Typestate over Library.** Always use `Evidence<Receipt, Admitted, AffidavitReceiptChain>` to ensure receipts flow through a forced one-way door.
- **ADR-2: Value-level Sealing.** Enforce receipt immutability via private `_seal` fields and dedicated constructors (e.g., `Receipt::sealed`).
- **ADR-3: Non-forgeable Carriers.** The `Evidence` carrier must have private fields to prevent struct-literal construction in external crates (E0451).
- **ADR-5: Verify↔Show Distinction.** Maintain strict behavioral separation between `verify` (adjudicates) and `show` (displays). Always witness this via dispatch tests.
- **ADR-7: Ontological Derivation.** CLI verbs must be rendered from `ontology/affi-cli.ttl` using `ggen`. Business logic lives in hand-written handlers.

## Implementation Standards
- **No Bare Returns:** All operations must wrap output in `Evidence` at the boundary (NFR-3).
- **Stdout Safety:** Use `#![deny(clippy::print_stdout)]` and behavioral tests to prevent human logs from corrupting protocol frames.
- **Exhaustive Completeness:** Never use placeholders or mocks. Implementation must be production-ready and structurally complete.
- **Deterministic Sealing:** Ensure same evidence always produces the same identity (BLAKE3).

## Autonomous Governance (Feature 7.1)
- **Autonomous Governance Agent (AGA):** A localized AI loop within the CLI that scans `.ggen/receipts/` for rejections, analyzes violations against these rules, and proposes architecture changes in `wip/1000x_autonomous_governance.rs`.
- **AGA Audit Loop:** Run `affi governance audit` to identify structural drift and propose fixes based on these mandates.

## Quality Gates
- **1000x Suite:** All features must pass their corresponding E2E test suite.
- **Failing-when-fake:** An integration is admitted only if removing it breaks a test that exercises real capability.
