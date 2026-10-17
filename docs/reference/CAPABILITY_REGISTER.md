# Capability Register — dependency-derivation wave 1 (2026-10-16, 16 lanes)

Coordinator-owned register. Next waves: dedupe against this file before dispatch.
Standing: all UNKNOWN until falsifier runs on the exact subject.

## Ranked queue (ERRC order; delete/reduce before generate)

| # | Proposal | Dep | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 1 | **Ed25519 strict-verify parity (defect)** — host uses `verify`, wasm uses `verify_strict`; malleated sigs pass host, fail wasm | ed25519-dalek | `src/ed25519_witness.rs:90` | S | malleated sig refused identically host+wasm in differential court |
| 2 | **SLH-DSA/ML-KEM genuine positive controls** | slh-dsa/ml-kem | core tests | S | DONE this wave (commit 74801fc) |
| 3 | **JCS cross-runtime parity court** — serde_jcs vs rendered `crypto_trust_canonical::jcs`, corpus of ~280 vectors | serde_jcs | new `affidavit-wasm/tests/jcs_parity.rs` | S | float-format divergence case flips court to divergence |
| 4 | **`smt_inclusion_verify` wasm op** — missing twin of absence op; closes prover/verifier asymmetry | monotree | `affidavit-wasm/src/advanced.rs` + ontology re-project | S/M | tampered proof step → `admitted:false`; wired-swap flips court |
| 5 | **Registry self-registration via linkme** — delete ~140-line hand REGISTRY literal, parity courts survive unchanged | linkme | `src/registry.rs` + macro wrapper | M | deleting one `#[verb]` drops count by exactly 1; courts still fail on deletion |
| 6 | **Framework-derived completions** — delete Python regex parser chain | clap-noun-verb | `src/verbs/completions.rs` | M | add a `#[verb]` stub → completions contain it with zero regen |
| 7 | **blake3 domain-separated KDF seam** (`new_derive_key`) — kem.rs:148 hashes unkeyed | blake3 | `affidavit-core/src/digest.rs` trait ext | S | context collision test: same bytes, two ctxs → different digests; dropping ctx flips digest |
| 8 | **ml-dsa context-string sealing** (`sign_deterministic(M, ctx)`) — replace tag-stuffing in sealed.rs | ml-dsa | core `sealed.rs` + wasm `pqc.rs` | M | ctx-signed bytes fail empty-ctx verify and vice versa |
| 9 | **KEM escrow envelope** (`judge_escrow`, bind key_id(pk)+blake3(ct)) | ml-kem | core `kem.rs` | M | bit-flipped ct still passes 7-stage verify but escrow verdict flips |
| 10 | **OCEL conformance court** over compat DFG fitness/precision (`admit_flat` grounded claims) | wasm4pm-compat | new `mining/ocel_compat_court.rs` | M | one activity mutation flips fitness + refusal path |
| 11 | **x-wing HNDL-resistant hybrid seal** | x-wing | new `src/hybrid_seal.rs` | M/L | stripping either leg refuses typed; single-leg accept = FAIL |
| 12 | **Wasm SLH/Dual ABI vacuity** | slh-dsa | `tests/pqc_abi.rs` | S | IN FLIGHT (write lane) |
| 13 | **Range-proof label `receipt-amount`** + cross-label court | bulletproofs | `advanced.rs:115` registry | S | prove-under-A, verify-under-B ⇒ refused |
| 14 | **Span-linked receipt lineage** (trace_verify + SpanRecord links) | opentelemetry | `src/tracing.rs` | S | parent chain-hash mutation changes link set |
| 15 | **OCEL v2 federation import** (foreign JSON → court surface) | wasm4cm-compat | `mining/ocel_compat_import.rs` | M | broken link table refused typed |
| 16 | **Batch witness verification** (verify_batch, ≥4x bench + poisoned-member refusal) | ed25519-dalek | `src/ed25519_witness.rs` | M | 1 swapped pair fails batch with index |
| 17 | **const-generic fixed-N batch verifier** (alloc-free) | core | `verifier.rs` | S | tampered N-th digest flips exactly cell N-1 |
| 18 | **DeterminismEngine vendored** (frozen clock + seeded RNG) | clnrm-core semantics | feature-gated adapter | M | same config → identical sequences; wall-clock build refused |
| 19 | **Alias parity from keywords[0]** (visible_alias per verb) | clap-noun-verb | `src/cli.rs` | S | `affi receipt certify` resolves to verify, exit 0 |
| 20 | **Auto-generated verb docs** from REGISTRY (kills 83/79/101 count drift class) | clap-noun-verb/ggen | docs renderer | M | summary edit without regen fails parity test |

## Cycle-2 additions (2026-10-16, deep-dive wave, dedupe vs wave 1)

| # | Proposal | Dep | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 21 | Runtime registry court — clap `validate()` + `command_structure()` cross-check in doctor | clap-noun-verb | `src/verbs/doctor.rs` | S | duplicate `#[verb]` stub compiles, count parity passes, court flips InvalidStructure |
| 22 | Diag exit-code range for framework surface errors (CommandNotFound→1500s + Levenshtein hint) | clap-noun-verb | `src/bin/affi.rs` + `src/diag.rs` | S/M | `affi bogus-noun` exits 1500-range + Diagnostic{hint} |
| 23 | `Hasher::reset()` reuse seam (replay_nullifier keyed hashing) | blake3 | `src/replay_nullifier.rs:81,134,143` | S | no-reset mutation flips differential court |
| 24 | mmap+rayon parallel hash for large files (identical-or-refuse) | blake3 features | `src/sbom.rs`/`chain.rs` | M | 8MB fixture mmap vs stream byte-identical, else refuse |
| 25 | Cached ExpandedSigningKey signer seam + differential court | ml-dsa | `src/crypto_trust_pqc.rs` | S | cached vs per-call byte-identical sigs; screening control fires |
| 26 | pk encode/decode round-trip + verify error-taxonomy court | ml-dsa | `crypto_trust_pqc.rs` tests | S | taxonomy cell change flips court |
| 27 | SLH opt_rand pinned-randomizer determinism/divergence court | slh-dsa | core pqc.rs tests | S | swapped opt_rand without sig flip = FAIL |
| 28 | SLH vk/sig wire-budget pins (32/7856) + ±1-byte typed refusal boundary | slh-dsa | core/wasm pqc.rs | S | SigLen drift or ±1-byte accept flips court |
| 29 | KEM seed-identity court via to_seed() (expanded form → typed None/refusal) | ml-kem | core kem.rs | S | fabricated seed accepted or Some-on-expanded = FAIL |
| 30 | Deterministic KAT dual-implementation court (ml-kem vs libcrux, hazmat) | ml-kem | root crate | M | single-implementation agreement = vacuous FAIL |
| 31 | WitnessKeyPair seed() / from_seed() + Signer/Verifier composition | ed25519 | `src/ed25519_witness.rs` | S | from_seed(seed()).public()==public(); zero-seed clamps |
| 32 | JWKS OKP import point-validation (VerifyingKey::from_bytes at boundary) | ed25519 | `src/crypto_trust_jwks.rs` | S | 32×0xff x refused typed; real pk passes |
| 33 | Receipt-bound range witnesses (transcript append receipt-hash) | bulletproofs/merlin | `src/zk_range.rs` | S | prove H1/verify H2 → Err; drop append flips court |
| 34 | Batched range verification (verify_multiple, shared gens) | bulletproofs | `src/zk_range.rs` | S/M | one swapped member fails batch w/ index |
| 35 | Post-revocation absence court (remove→absence→re-insert restore) | monotree | `src/smt.rs` tests | S | stale shadow-index bug flips court |
| 36 | Generic StateTree<D: Database> (fault-injectable test DB reaches SmtError::Tree) | monotree | `src/smt.rs:87-91` | S/M | injected DB fault → typed refusal, fence fail-closed |
| 37 | JCS number typed-refusal parity corpus (i64::MAX etc: serde_jcs admits, jcs() refuses) | serde_jcs | jcs_parity.rs rows | S | i64::MAX row flips parity court to divergence |
| 38 | JCS string-escaping corpus (NFC/NFD, U+2028/29, DEL) | serde_jcs | corpus rows | S | NFC-normalizing impl flips court |
| 39 | Flattening-loss accounting court (LossChain, REFUSED_FLATTENING_LOSS) | wasm4pm-compat | `mining/ocel_flatten_loss_court.rs` | S/M | severed E2O relationship flips ACCEPT→refusal |
| 40 | Alignment-claim grounding court (admit_flat before fitness emitted) | wasm4pm-compat | `mining/ocel_alignment_court.rs` | S/M | zero-move alignment → typed refusal not fitness 0.0 |
| 41 | REFUSED_* Prometheus counter family (central refusal point) | opentelemetry | `metrics` feature | S/M | known refusal → series refusal="…" value 1; drop increment → absent |
| 42 | Receipt-size bucket histogram at verify | opentelemetry | `metrics` feature | S | 2 sizes → ≥2 bucket counts |
| 43 | WitnessKeyPair seed round-trip (dup of 31 — merge) | — | — | — | — |
| 44 | SjReceipt TTL projection parity court | clnrm-core | federation seam | M | out-of-vocab standing → typed fail |
| 45 | Seeded permutation adversarial corpus for 7-stage pipeline | clnrm-core | tests | S/M | seed/swap mutation flips a verdict cell |
| 46 | Deterministic doctor emission (sort by id; delete duplicate legacy check pushes; id-uniqueness court) | linkme | `src/doctor_check.rs:253` + `handlers.rs:4930,4949` | S | rebuild×2 diff of `affi doctor --format json`; duplicate id fails gate |
| 47 | Doctor vacuity positive-control (host ≥7 checks; wasm32 loud UNSUPPORTED_REGISTRATION, never empty-ok) | linkme | `src/doctor_check.rs:155` | S | removing one registration drops count and fails gate |

### Cycle-2 delete-class findings (ERRC — outrank generation)

- **Dead `verify_bip340`** (prehashed variant, zero call sites/tests): delete (secp256k1_witness.rs:69); companion defect — BIP-340 court covers only vector 0, add vectors 1-3 + two failure vectors.
- **`metrics`/opentelemetry-prometheus optional deps have ZERO consumers**: wire them (row 41) or delete the dep plane.
- **clnrm-core contains a hand-implemented ARX sponge hash** (`pqc/hash.rs`): law violation inside the dependency tree; do not port/consume — flag upstream.
- **ES256/ES256K pk-serialization form law inconsistent** (compressed admissible-but-unexportable for ES256; reverse for ES256K): adjudication item.

## Adjudication items (operator decision, not auto-landable)

- **Root-crate PQC feature flag** (`crypto-trust` gates ml-dsa/slh-dsa/ml-kem): operator law says "no PQC feature flags"; core+wasm comply, root does not. Options: make crypto-trust default, or scope the law to core+wasm in docs (README/CLAUDE.md now scoped this way).
- **Envelope `Algorithm::{Es256, HybridEs256MlDsa65}` wire labels** (crypto_verify.rs:147-178): wire labels vs "PQC-only surface" law. Docs now state labels-not-implementations; removal would break the CTRT/KAT corpus.
- **ES256/ES256K/Ed25519 classical code in root crate** vs operator "PQC-only signature surface" law: code predates the law; docs reconciled; removal is destructive → operator call.
- **`recovering`-based ECDSA signer recovery (CAP-1)**: verifier-only, no new dep; trust-model note — recovery output must be checked against stored key fingerprint before admission.

## Lane hygiene

- bash completions `legal-hold` hyphen bug: IN FLIGHT (generator sanitize lane).
- docs/WASM.md 16-op/10-error-code drift: IN FLIGHT (doc lane).
- core zero-dep docstring drift: IN FLIGHT (doc lane).
- core AGENTS/README four-algorithm law: IN FLIGHT (doc lane).
- justfile validate gaps: DONE (1f7103a).
- CHANGELOG 26.10.16 entry: DONE (fd04c66).
