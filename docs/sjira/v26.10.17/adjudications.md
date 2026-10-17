# ADJUDICATIONS — affidavit — v26.10.17

Wave: v26.10.17, branch `v26.10.16` (as-of HEAD `b0325d4`). Lane-owned file:
`docs/sjira/v26.10.17/adjudications.md`. Status vocabulary per AGENTS.md.
All rows are **DECISION-REQUIRED**: they are not auto-landable and cannot be
closed by any verification lane — each needs an explicit operator ruling.

| field | value |
|---|---|
| repo | `/Users/sac/affidavit` |
| branch | `v26.10.16` |
| HEAD at write | `b0325d4` (plus in-tree dirty worktree, not lane-owned) |
| standing | DECISION-REQUIRED ×5 (rows A1–A5) |
| falsifier of this register | a row closes without an operator ruling, or a ruling is recorded with no evidence commit / falsifier cited |

## Pending operator adjudications

| id | item | options | evidence | recommendation | blocking scope | type |
|---|---|---|---|---|---|---|
| A1 | **BLS aggregate soundness — KD-BLS-1.** Rogue-key compensation forgery verifies; the all-identity aggregate verifies over arbitrary messages. Aggregate verification is unsound without proof-of-possession. | (a) land PoP (proof-of-possession) registration infrastructure in front of aggregation; (b) de-scope the BLS aggregate module entirely (ERRC: delete outranks generate). | KD-BLS-1 row in `docs/reference/CAPABILITY_REGISTER.md`; commit `84a430f`; `tests/bls_rogue_key.rs` (2 ignored DEFECT rows, must flip on fix or on de-scope). | De-scope (b) unless a concrete multi-signer consumer is named. PoP infra is generation; de-scope is a permanent entropy sink and the 2 DEFECT rows are deleted with it. | Blocks closing KD-BLS-1 and any consumer of aggregate BLS in the trust plane. | DECISION-REQUIRED |
| A2 | **Root-crate crypto-trust feature flag vs the 'no PQC feature flags' law.** Operator law: PQC surface ships unconditionally, no feature flag. `affidavit-core` and `affidavit-wasm` comply; the root crate gates `ml-dsa`/`slh-dsa`/`ml-kem` behind the `crypto-trust` feature (`Cargo.toml:409-413`, `required-features` at `103/610/623/626`). | (a) make `crypto-trust` a default feature (flag exists, ships on by default — minimal diff, no consumer break); (b) remove the gate entirely, PQC deps unconditional (strictest reading; larger dependency surface for all consumers); (c) scope the law in docs to core+wasm only (already drafted in README/CLAUDE.md — ratify the scoping). | `Cargo.toml` (crypto-trust feature block, optional deps ml-dsa/slh-dsa/ml-kem); README/CLAUDE.md scoped-law text; register adjudication section. | (a) default-on: satisfies the law's intent (no silent PQC-absent builds) without forcing ml-kem hazmat + slh-dsa rc deps onto every consumer, and without a destructive dep-graph change. If the law is meant literally, choose (b) and fund the dep-graph work. | Blocks the "no PQC feature flags" claim for the root crate in every receipt citing that law; blocks docs finalization of the scoped-law text. | DECISION-REQUIRED |
| A3 | **Envelope `Algorithm::Es256` wire labels vs the PQC-only surface law.** `Algorithm::{Es256, HybridEs256MlDsa65}` wire labels exist (`src/crypto_verify.rs:147-178`) and are load-bearing in the CTRT/KAT corpus; removing them breaks the corpus. Docs now describe them as labels-not-implementations. | (a) ratify labels-not-implementations reading (labels preserved, no classical implementation behind them); (b) order reduction: remove the labels, migrate the CTRT/KAT corpus, accept the wire break. | `src/crypto_verify.rs:147-178`; CTRT/KAT corpus (`docs/reference/PQC_KAT_VECTORS.md`, `WIRE_CTRT_352.md`); docs scoping text. | (a) ratify. The label is wire compat, not a crypto claim; removal is a destructive wire break to save zero attack surface (no implementation behind it). If ratified, close with a one-line law annotation ("labels exempt from the PQC-only surface law"). | Blocks any wire-compat claim about the envelope surface; blocks CTRT/KAT corpus sign-off. | DECISION-REQUIRED |
| A4 | **clnrm-core hand-rolled ARX sponge (`pqc/hash.rs`).** A hand-implemented ARX sponge hash exists inside the dependency tree — direct violation of the operator law "no crypto algorithm is implemented in this project; every algorithm is an external dependency". It is fenced and unconsumed by affidavit. | Only one lawful direction; adjudication is on the *register action*: (a) keep UNSUPPORTED(dep), flag upstream and never port/consume (current standing); (b) escalate to a fork/patch plan if clnrm-core will not fix. | Register: "clnrm-core contains a hand-implemented ARX sponge hash (`pqc/hash.rs`): UNSUPPORTED(dep)"; `stubs/` + `[patch.crates-io]` fencing in `Cargo.toml`. | (a) hold UNSUPPORTED(dep), do not port, do not consume, upstream flag stays the sole action. Any consumption of `pqc/hash.rs` is a typed `REFUSED_*` transition. | Blocks any feature that would consume clnrm-core's pqc/hash surface; blocks closing the delete-class register row. | DECISION-REQUIRED |
| A5 | **Classical ES256/ES256K/Ed25519 code in the root crate predating the law.** Classical signature code predates the PQC-only law; docs are reconciled to describe it; removal is destructive (would break witnesses and their courts). | (a) keep-gated: retain the code, gate behind crypto-trust / document as legacy-pre-law surface; (b) removal: delete the classical witnesses and their courts (destructive, wide blast radius). | Register adjudication item ("code predates the law; docs reconciled; removal is destructive → operator call"); witness modules `src/secp256k1_witness.rs`, `src/ed25519_witness.rs`, `src/crypto_trust_es256.rs`; related ES256/ES256K pk-serialization ADJUDICATION (register cycle-2). | (a) keep-gated, with a docs line naming it legacy-pre-law and a falsifier row per witness. Removal should be scheduled, not silent: if removed, it is an ERRC delete with its own receipt, never an accident of refactoring. | Blocks a final "PQC-only signature surface" claim for the root crate; blocks the pk-serialization-form law adjudication (same ruling family). | DECISION-REQUIRED |

## Row closure protocol

A row closes when the operator ruling is recorded here (ruling + date + one-line
rationale) AND its falsifier is green or its de-scope lands. Pending that, each
row's falsifier is:

- A1: `tests/bls_rogue_key.rs` DEFECT rows still ignored, or aggregate module gone.
- A2: `cargo build --no-default-features` shows PQC deps optional in the root crate while the law text remains unscoped.
- A3: label removal breaks CTRT/KAT courts while docs claim labels-not-implementations is unratified.
- A4: any import/consume of clnrm-core `pqc/hash.rs` fails the row by definition.
- A5: removal of classical witnesses without a scheduled delete receipt flips this row to violated.
