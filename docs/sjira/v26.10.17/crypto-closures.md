# CRYPTO-CLOSURES — affidavit — v26.10.17

Wave v26.10.17, write lane (docs/sjira/v26.10.17/crypto-closures.md). Repo
`/Users/sac/affidavit`, branch `v26.10.16`. Exemplar format:
`docs/sjira/v26.10.8/CAMPAIGN-RECEIPT.md` (receipt-table register).

Register of record for the divergence row: KD-BLS-1 in
`docs/reference/CAPABILITY_REGISTER.md` (registered in commit `11f2b59`).

Disposition summary: **5 rows — 4 CLOSURES, 1 REGISTERED DIVERGENCE.**
The BLS row is a divergence, not a closure: the soundness claim was
falsified and no fix is claimed; the row stays open until the operator
adjudicates PoP infrastructure vs de-scope.

## Closure rows

### R1 — FROST nonce-reuse (row 81c) — CLOSED

| field | value |
|---|---|
| attack | One round-1 nonce draw reused across two signing packages over different messages was ACCEPTED (both aggregates verified) — an algebraic key-extraction oracle for every participant share; `round2_sign` had no reuse guard (`SigningNonces` move-by-value, no state). |
| falsifier test | `tests/frost_nonce_court.rs` (commit `3bb560c` pinned the DEFECT with flip instructions: stale-package refusal, 256-draw uniqueness, reuse ACCEPTED, honest/tampered positive control) |
| landed commit | repair `2e9a18d` — `SingleUseNonces` consuming newtype (`RefCell<Option<SigningNonces>>`, non-Clone); `round2_sign_once` returns typed `Err(QuorumError::NonceReuse)` before any Schnorr response; court cell (c) flipped per pinning instruction. Court commit `3bb560c`. |
| court exit | `frost_nonce_court` 4/0 exit 0 (`--features threshold-quorum`); `threshold_quorum` lib 7/0; `--lib` 449/0 (no features) / 456/0 (feature plane) |

### R2 — SMT absence-witness key binding (row 34-class) — CLOSED

| field | value |
|---|---|---|
| attack | Forged absence witness: `verify_absence` folded only leaf value + step cuts — the neighbor key field was decorative — so a valid inclusion proof from an unrelated subtree passed the cryptographic check while the prefix law was argued from `neighbor_key`. |
| falsifier test | `src/smt.rs` unit cells: `absence_witness_refuses_neighbor_proof_key_mismatch`, `absence_witness_prefix_tamper_still_refused`, plus vacuity pins `absence_witness_verifies_for_uncommitted_key` / `absence_witness_refuses_committed_key_and_empty_tree` |
| landed commit | `0ac7485` — `verify_absence` now requires `neighbor_proof.key == neighbor_key` AND `proof_routes_key()`: a cut parser extracting each step's consumed path range + 32-aligned path bits from the monotree node serialization, ranges tiling 0..256 in order with every bit matching `neighbor_key`. Empty-proof vacuity ALREADY-SAFE-PINNED. |
| court exit | `smt` 11/0 exit 0 (`--features smt`); `--lib` 449/0 |

### R3 — AG4 quartet (JCS dup-field / alg-confusion / quorum share counts) — CLOSED

| field | value |
|---|---|
| attack | Four executed adversarial attacks over the crypto-trust plane: (a) envelope kept the LAST duplicate top-level JSON member (serde semantics), (b) foreign-algorithm signature bytes hit the Registry catchall (`error UnknownKey`) instead of deciding `Invalid`, (c) honest Ed25519/ES256K quorum shares were falsely refused by the same catchall — share-count/algorithm arms missing in `VerificationEngine::verify_signature_bytes` and `QuorumEngine::share_verifies`, (d) `kid`-swap / duplicate-`kid` registry gaps covered by the same court. |
| falsifier test | `tests/ag4_adversarial_crypto.rs` — `jcs_duplicate_field_refused`, `alg_confusion_ed_key_under_es256k_envelope_refused`, `alg_confusion_es256k_sig_bytes_under_ed25519_envelope_decided_invalid`, `quorum_ed25519_share_counts`, `quorum_es256k_share_counts`, `quorum_duplicate_signer_refused` |
| landed commit | `c2119a5` — envelope `from_bytes` duplicate-member refusal (raw-byte, depth/escape-aware); Ed25519/ES256K matching-material arms so foreign-algorithm bytes DECIDE `Invalid`; `share_verifies` same arms; `EcdsaVerificationFailed` decides `Ok(false)`, malformed key stays `InvalidShare`. |
| court exit | `ag4_adversarial_crypto` 19/19 exit 0 (baseline 15/4), positive controls intact; `--lib --features crypto-trust` 849/0/1 ignored; `--lib` 409/409; fmt clean; clippy plane clean |

### R4 — e2e SK3 degenerate-seed fixtures — CLOSED

| field | value |
|---|---|
| attack | Not an external attack: e2e fixtures used all-identical seeds (`[tag;32]`, impostor `[0xBB;32]`) — refused by the SK3 degenerate-seed screening law in integration builds (`cfg!(test)` false there, screening active), so the suite could not run against the real screening law. |
| falsifier test | `tests/crypto_trust_e2e.rs` — `es256_full_cycle_from_receipt_to_standing_to_seal`, `wrong_signer_yields_a_decided_invalid_verdict`; falsifier exercised mid-lane: `[0xBB;32]` impostor fixture refused by screening — law live, not decorative. |
| landed commit | `1a81553` — fixtures derive varied seeds (`tag ^ i*const`); impostor seed derived the same way. |
| court exit | `crypto_trust_e2e` 14/0 exit 0 (`--features crypto-trust`) |

## Registered divergence (not a closure)

### D1 — BLS rogue-key + all-identity aggregate — REGISTERED DIVERGENCE KD-BLS-1

| field | value |
|---|---|
| attack | Registered keys `[pk_honest, pk_u+pk_v−pk_honest]` with sigs `[u·H(B), v·H(B)]` pass the single pairing check over a digest the honest committee never approved; second defect: an all-identity aggregate (identity key + identity sig) verifies over ANY message. The module doc's "sound aggregation mode without proof-of-possession infrastructure" is false. |
| falsifier test | `tests/bls_rogue_key.rs` — 2 `#[ignore]` DEFECT rows pinning both forgeries, plus positive control (court can flip); linearity witness added to `src/bls_aggregate.rs` |
| landed commit | `84a430f` — court only; NO fix claimed. The module doc's soundness claim is falsified; disposition REGISTERED DIVERGENCE. |
| court exit | `bls_rogue_key` 3 passed / 2 ignored (DEFECT) exit 0; `bls_aggregate` lib 5/5 |
| close condition | PoP (proof-of-possession) infrastructure lands, or the BLS aggregate module is de-scoped — operator decision (KD-BLS-1, `docs/reference/CAPABILITY_REGISTER.md`). |

## Standing

Wave register: 4 CLOSURES + 1 REGISTERED DIVERGENCE (KD-BLS-1). All court
exits are as-of-commit receipts cited from the respective commit messages on
branch `v26.10.16`; the divergence remains open until operator adjudication.
