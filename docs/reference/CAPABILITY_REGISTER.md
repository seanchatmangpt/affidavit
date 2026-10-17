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
