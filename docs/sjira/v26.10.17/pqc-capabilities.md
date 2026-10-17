# PQC Capability Landings — work-order register — v26.10.17

| field | value |
|---|---|
| repo | `/Users/sac/affidavit` |
| branch | `v26.10.16` (HEAD `b0325d4` at receipt) |
| standing | PARTIAL_ALIVE (all listed courts green at HEAD; root crypto-trust plane BUILD_BROKEN on the *dirty* tree only — another lane's in-flight `src/verbs/*` vs `src/handlers.rs` mismatch; HEAD in scratch is green, exit 0) |
| court replay | `cd affidavit-core && cargo test` (lib 143/0); `cargo test --features crypto-trust --test adversarial_pqc` (13/0); `cargo test --features pqc --test kem_dual_impl_kat` (4/0); `cargo test --features crypto-trust --lib` (909 pass / 0 fail / 1 ignored) |

# Operator law (anchor)

Every cryptographic algorithm is an external dependency; zero in-crate
cryptography. Pinned: `ml-dsa 0.1.1`, `slh-dsa =0.2.0-rc.5`, `ml-kem 0.3.2`,
`blake3`, plus `x-wing`, `libcrux-ml-dsa`, `libcrux-ml-kem` (dual-impl courts).
The PQC surface (ML-DSA-65, SLH-DSA-SHA2-128s, hybrid ES256+ML-DSA-65)
ships unconditionally — no PQC feature flag — and includes the WASM artifact
(`src/crypto_trust_pqc.rs:1-10`; `affidavit-core/src/pqc.rs`).

# Work-order rows

| # | capability | seam file:line | falsifier court | landed commit | court exit (receipt) |
|---|---|---|---|---|---|
| 1 | ChainKdf `derive_key` — blake3 derive_key domain separation | `affidavit-core/src/digest.rs:298` (via `Hasher::new_derive_key`, no in-crate rounds) | ChainKdf domain-separation courts + FIPS 180-4 self-court (`affidavit-core/src/digest.rs` inline tests) | `a05c9c1` | core lib `143 passed; 0 failed`, exit 0 (replayed 2026-10-17) |
| 2 | ChainKdf `derive_xof` — XOF output streaming; confusion detectable, never silent (XOF vs plain derive divergence asserted) | `affidavit-core/src/digest.rs:286-312` | XOF-vs-derive divergence assertion (`derive_xof(...) == derive(...)` must fail) | `dbac2da` | core lib `143 passed; 0 failed`, exit 0 |
| 3 | KEM seed-identity: seed round-trip byte-exact | `affidavit-core/src/kem.rs:538` (`seed_identity_roundtrip_is_byte_exact`) | seed-identity court (register #29) | `eba7b49` | core lib 143/0, exit 0 |
| 4 | KEM cross-parameter-set confusion court | `affidavit-core/src/kem.rs:798` (register #80, cycle 3/4) | cross-parameter-set confusion court | `75d009c` | core lib 143/0, exit 0 |
| 5 | KEM pinned-randomness encapsulation KAT (CAP-65): pinned ct/ss recovered via seed decapsulation | `affidavit-core/src/kem.rs:642` (`cap65_pinned_randomness_encapsulation_kat`) | CAP-65 KAT — pinned encapsulation under fixed randomness | `ebfa6c7` (KAT seed surface closed in SK3 fix-forward; test lands here in kem.rs) | core lib 143/0, attribution: commit `ebfa6c7` closes the seed surface; test body present at HEAD `b0325d4` |
| 6 | KEM implicit rejection: tampered ct is pseudorandom and stable (FO transform) | `affidavit-core/src/kem.rs:702` (`cap66_implicit_rejection_tampered_ct_is_pseudorandom_and_stable`); adversarial mirror `tests/adversarial_pqc.rs:82` | implicit-rejection court (CAP-66 + Σ_PQC case 1) | `eeda22b` (via `-S implicit_rejection`: `13c776a`, `eeda22b`) | adversarial_pqc `13 passed; 0 failed`, exit 0; core lib 143/0 |
| 7 | Dual-implementation ML-KEM-768 KAT (ml-kem vs libcrux-ml-kem) | `tests/kem_dual_impl_kat.rs:1-139` (feature `pqc`) | dual-impl KAT court (register #30, ALIVE) | `3ea786d` | `4 passed; 0 failed`, exit 0 (feature `pqc`) |
| 8 | SLH-DSA opt_rand determinism + wire-budget pins (register #27/#28) | `affidavit-core/tests/pqc_external_validation.rs` (109-line court added by commit) | opt_rand A=B byte-identical; A≠B both verify | `9b36a3d` | core `cargo test` exit 0 (all suites incl. pqc_external_validation) |
| 9 | SLH SPKI/PKCS8 byte-exact round-trip (row 52) | `src/crypto_trust_pqc.rs` (SLH consts `:37-49`; SPKI court in commit 5cd7eb9) | SPKI/PKCS8 round-trip: fixture + fresh keys, imported key still verifies | `5cd7eb9` | crypto-trust lib `909 passed; 0 failed; 1 ignored`, exit 0 (feature `crypto-trust`) |
| 10 | SLH ctx (framed deontic contexts) + seed-role (three-seed keygen sk_seed\|\|sk_prf\|\|pk_seed) | `src/crypto_trust_pqc.rs:39-49` (seed-role consts); ctx framing per `ebfa6c7` | framed-context divergence must diverge; seed-role split length pinned `SLH_DSA_128S_SEED_LEN=48` | `ebfa6c7` + `5cd7eb9` | crypto-trust lib 909/0/1, exit 0 |
| 11 | ml-dsa cached signer (MlDsa65Signer cached-expansion seam) | `src/crypto_trust_pqc.rs` (seam from commit c7bad41, 276 insertions / 360 deletions) | pk round-trip courts | `c7bad41` | crypto-trust lib 909/0/1, exit 0 |
| 12 | ml-dsa sk round-trip + seed-identity (rows 67/68) | `src/crypto_trust_pqc.rs` | sk round-trip + seed-identity courts (rows 52/67/68 batch) | `5cd7eb9` | crypto-trust lib 909/0/1, exit 0 |
| 13 | x-wing dk->ek recovery parity (seeded corpus) + dual-mode isolation | `tests/adversarial_pqc.rs:463` (`test_x_wing_dk_to_ek_recovery_parity_seeded_corpus`); dual-mode per `adversarial_pqc.rs` rows 79/80 | parity corpus + dual-mode divergence courts (rows 79/80) | `a16f4ba` | adversarial_pqc `13 passed; 0 failed`, exit 0 |
| 14 | zk-range receipt-bound witnesses + batch verification (registers #33/#34) | `src/zk_range.rs:326` (`verify_range`), `:411-491` (`RangeBatchMember`/`prove_range_batch[_bound]`), `:574` (`verify_range_batch`) | receipt-hash binding must fail on any receipt mutation; batch verify | `3c285d5` | crypto-trust lib 909/0/1 incl. zk_range unit courts; adversarial suites exit 0 |
| 15 | zk-range seeded deterministic blinding + Pedersen opening seam | `src/zk_range.rs:150` (`prove_range_seeded`), `:175` (`pedersen_open`), `:204` (`SeededRng`) | seeded determinism: same seed -> byte-identical proof; blinding divergence | `23c3774` | crypto-trust lib 909/0/1, exit 0 |
| 16 | zk-range receipt-amount domain label | `affidavit-wasm/src/advanced.rs:392` (`affidavit:zk-range:v1:receipt-amount`; positive control `:512-520`) | registry-gate positive control at the label registry | `82dbbe2` | wasm surface courts exit 0 at pin `a8c3c2b` |
| 17 | bcinr double-mix tamper-propagation court (register #77) | `affidavit-core/tests/bcinr_fm_chain_009.rs` (168-line court) | tamper anywhere in the double-mix chain must propagate to a verify REJECT | `6534c13` | core `cargo test` exit 0 (bcinr_fm_chain_009 8/0 witnessed in full-suite tail) |
| 18 | external-evidence boundary admission corpus (register #78) | `affidavit-core/src/external_evidence.rs:174,240,252` (`admit_workload_identity` / `admit_evidence_validity` / `admit_authzen_evidence`) | boundary corpus: expired/foreign/malformed evidence must be typed-refused | `f3871f5` | core lib 143/0, exit 0 |
| 19 | LatencyBands cross-validation court (row 74) | `src/tracing.rs:193-265` (`LatencyBands` + cross-validation) | cross-validation: recomputed bands vs reported must agree or refuse | `898b09f` | crypto-trust lib 909/0/1, exit 0 |
