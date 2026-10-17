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

## Cycle-3 additions (2026-10-16, 16-lane deep-dive round 3; rows 48-78)

| # | Proposal | Dep | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 48 | Typed blake3::Hash digest via serialize feature (malformed commitment refused at deserialize) | blake3 | `src/types.rs:27` | S/M | 63-hex/non-hex commitment rows flip verdict cells at stage-1 |
| 49 | Hasher: io::Write streaming seal (to_writer, delete intermediate canonical String) | blake3 | `crypto_trust_canonical.rs:222, wire_ctrt.rs` | S | writer vs render-then-hash digests byte-identical on 1MB corpus |
| 50 | validators seam for CLI admission gates (delete hand-rolled arg parsers) | clap-noun-verb | `src/verbs/*.rs` | S | old check vs validators:: diverge on one fixture -> court flips |
| 51 | policies wire-in to doctor (TargetPressurePolicy/ToolchainMismatchPolicy as real checks) | clap-noun-verb | `src/doctor_check.rs` | S | >max_gb size flips doctor verdict; removal drops count by 1 |
| 52 | SLH-DSA SPKI/PKCS8 export-import round-trip court | slh-dsa | `src/crypto_trust_pqc.rs tests` | S | tampered alg-id OID SPKI or truncated vk -> typed Err; honest round-trip byte-exact |
| 53 | SLH-DSA multipart domain-separation court (MultipartVerifier/Signer) | slh-dsa | `pqc.rs tests` | S | reordered/missing chunk must fail verify |
| 54 | X-Wing deterministic-encapsulation court (64B caller randomness) | x-wing | `crypto-trust tests` | S | same (ek,r) differing ct, or tampered ct same ss -> FAIL |
| 55 | X-Wing seed-identity + wire-budget pins (1216/32/1120) | x-wing | `crypto-trust tests` | S | 33-byte/truncated ct accepted or seed round-trip diverges -> FAIL |
| 56 | Ed25519 PKCS#8 key import/export round-trip | ed25519-dalek | `src/ed25519_witness.rs` | S | mutated DER byte -> typed refusal; round-trip restores pk |
| 57 | Ed25519 prehashed (phflag) strict-parity extension | ed25519-dalek | `ed25519_witness.rs` | S | malleated prehashed sig refused host+wasm; prehashed sig fails plain verify |
| 58 | wasm32 linkme premise probe (delete-class: fallback justification never tested) | linkme | `doctor_check.rs:153-163` | S | probe crate asserts len()==1 on wasm32-wasip1; survive -> delete fallback |
| 59 | Zero-size DoctorCheck law enforcement (size_of==0 compile court) | linkme | `doctor_check.rs` | S | 1-field check type fails to compile/register |
| 60 | JCS duplicate-key typed refusal (DEFECT: last-win silently canonicalizes) | serde_jcs | `parse layers both surfaces` | S | duplicate-key corpus row flips bad_json refusal vs today's silent accept |
| 61 | jcs_verify twin op (canonicalize/verify asymmetry) | serde_jcs | `crypto_trust_canonical.rs + wasm ABI` | S/M | swapped keys or reformatted number flips verdict with stage named |
| 62 | Aggregated range proof (prove_multi, O(log) joint proof) | bulletproofs | `src/zk_range.rs` | M | one out-of-range member refuses aggregate with index |
| 63 | Seeded deterministic blinding (byte-exact host/wasm differential courts) | bulletproofs | `src/zk_range.rs:87-89` | S | same seed+label+value -> cmp-identical proof bytes |
| 64 | otel plane BUILD_BROKEN adjudication (opentelemetry 0.32 sdk path removed) | opentelemetry | `tests/maximalist_tracing.rs` | S/M | cargo test --features otel --no-run exit 0 (repair) or grep-zero (delete) |
| 65 | KEM pinned-randomness encapsulation KAT (encapsulate_deterministic) | ml-kem | `affidavit-core/src/kem.rs tests` | S | same-m re-encapsulation diverges or different-m collides -> FAIL |
| 66 | KEM implicit-rejection (FO) semantics court | ml-kem | `kem.rs tests` | S | tampered-ct ss' unstable or == honest ss -> loud FAIL |
| 67 | ml-dsa ExpandedSigningKeyBytes wire round-trip court | ml-dsa | `src/crypto_trust_pqc.rs tests` | S | from_expanded(to_expanded(esk)) byte-identical sig; ±1-byte typed Err |
| 68 | ml-dsa SigningKey seed-identity court (as_seed/to_seed) | ml-dsa | `crypto_trust_pqc.rs tests` | S | to_seed(from_seed(s))==s; rebuilt key signs identically |
| 69 | EC JWK import seam (kty=EC P-256|secp256k1) + round-trip court | p256/k256 | `src/crypto_trust_jwks.rs` | S | non-CEM x refuses typed; dropping point-validation flips court |
| 70 | secp256k1/P-256 batch ECDSA verify (high-s member poisons batch) | k256 | `src/secp256k1_witness.rs` | M | swapped pair fails batch w/ index; lone high-s flips to refusal |
| 71 | Persistent SMT backend (monotree Sled/RocksDB) + reopen court | monotree | `src/smt.rs:88` | S/M | mutated DB byte flips inclusion verdict on reopen |
| 72 | Hasher-generic StateTree<H> + cross-hasher divergence court | monotree | `src/smt.rs` | S | verify_proof with swapped hasher still true = vacuous FAIL |
| 73 | DurableRefusal replay fixtures (refusal embeds machine-replayable fixture + blake3 integrity) | clnrm-core semantics | `src/diag.rs + new refusal_fixture.rs` | M | fixture byte mutation flips integrity hash; unreplayable refusal = FAIL |
| 74 | Latency-band cross-validation court (span vs receipt timing layers) | clnrm-core semantics | `otel feature + tracing.rs` | M | over-band span with no TimingViolation cell = FAIL |
| 75 | Streaming online/offline context-confusion court | wasm4pm-compat | `mining/ocel_streaming_context_court.rs` | S | online-vs-offline mislabeled evidence flips admission |
| 76 | Data-quality grounding gate before fitness/alignment claims | wasm4pm-compat | `src/mining.rs preflight` | S/M | ungrounded DQ -> typed refusal precedes any fitness number |
| 77 | bcinr double-mix tamper-propagation court (anti-vacuity) | none | `affidavit-core/src/bcinr.rs` | S | one-byte frame-t mutation flips chain_hash_t..n; non-propagating = FAIL |
| 78 | External-evidence boundary admission corpus (SPIFFE strictness, clock edges) | none | `affidavit-core/src/external_evidence.rs` | S | flipping one boundary (valid_until==now) flips a corpus cell |

Cycle-3 defect-class findings: JCS duplicate-key last-win (row 60); otel feature plane compile-broken (row 64, repair lane dispatched); `affi doctor` registered in the verb registry but no clap subcommand wired (cli.rs has zero doctor wiring); serde_json duplicate-key last-win also affects root parse paths.

## Cycle-4 additions (2026-10-17; rows 79-100) — appended late (cycle-4 reports were integrated as lanes before this append)

| # | Proposal | Dep | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 79 | XOF OutputReader seam (variable-length commitments, resumable hashing) | blake3 | `src/chain.rs + core digest.rs` | S/M | set_position diverges from fill-at-0; resume-from-count != fresh = FAIL |
| 80 | zeroize feature on keyed-MAC plane (wipe replay_nullifier key material) | blake3 zeroize | `src/replay_nullifier.rs` | S | feature off -> keyed state survives drop = FAIL |
| 81 | ml-dsa mu-path differential court (mu vs msg domain distinctness) | ml-dsa | `src/crypto_trust_pqc.rs tests` | S | mu-path sig verifying over wrong mu/msg = loud FAIL |
| 82 | ml-dsa pinned-rnd sign_internal divergence court | ml-dsa | `crypto_trust_pqc.rs tests` | S | same-rnd-different-M collision or cross-rnd identical sig = FAIL |
| 83 | SLH context-string domain-separation court (256B ctx cap) | slh-dsa | `crypto_trust_pqc.rs tests` | S | ctx-bound sig passing empty-ctx verify (either direction) = FAIL |
| 84 | SLH seed-role partition court (sk_seed/pk_seed/sk_prf roles) | slh-dsa | `crypto_trust_pqc.rs tests` | S | sk_prf swap leaves pk identical but verify flips = anti-vacuity control |
| 85 | SLH/blake3/ed25519 zeroize-feature gaps (dep lines enable no zeroize) | slh-dsa/ed25519-dalek | `Cargo.toml dep features` | S | key material survives Drop on every sign call = register decision |
| 86 | dk->ek recovery + KeyInit expansion parity court | x-wing | `tests/adversarial_pqc.rs` | S | dk.encapsulation_key() != original pk bytes = FAIL |
| 87 | X-Wing dual-mode divergence-set isolation court (CFRG transition law) | x-wing | `tests/adversarial_pqc.rs` | S | spurious or collapsed divergence between accept/reject modes = FAIL |
| 88 | Ed25519 secret-material hygiene court (redacted Debug, zero seed bytes) | ed25519-dalek | `src/ed25519_witness.rs:34` | S | Debug/serde output containing any seed byte = FAIL |
| 89 | verify_strict adversarial vector corpus (small-order A, non-canonical R/A) | ed25519-dalek | `ed25519_strict tests` | S | any vector accepted host-side or host/wasm divergence = FAIL |
| 90 | Multi-party (MPC) aggregation of receipt-bound range proof | bulletproofs | `src/zk_range.rs RangeMpcSession` | M | out-of-range party aborts typed; single-party shortcut = vacuous FAIL |
| 91 | Pedersen commitment-opening consistency court | bulletproofs | `src/zk_range.rs` | S | open(v±1,b) accepted, or acceptance without receipt-bound blinding = FAIL |
| 92 | ECDSA RFC-6979 nonce-determinism + low-s normalization differential court [ADJUDICATION: form policy] | p256/k256 | `secp256k1_witness + wasm twin` | S | same-seed re-sign diverges host/wasm, or high-s accept asymmetry = FAIL |
| 93 | ProcessDelta unresolved-evidence court (TelemetryGap never a conformance verdict) | wasm4pm-compat | `mining/ocel_process_delta_court.rs` | S/M | unresolved delta accepted as conformance claim -> typed refusal |
| 94 | DECLARE constraint-temper court (22 canonical templates) | wasm4pm-compat | `mining/ocel_declare_court.rs` | M | NotCoExistence violation ACCEPTs = FAIL; satisfying log must ACCEPT |
| 95 | DoctorCheck runtime-context seam run(ctx) + check_receipt_store migration | linkme | `src/doctor_check.rs + handlers.rs` | S | post-migration findings byte-identical; registration removal drops count by 1 |
| 96 | CUSTOM_HANDLERS plugin slice: documented but nonexistent (delete-or-land) | linkme | `src/handlers.rs + CLAUDE.md` | S/M | integration test registers handler, emit dispatches through it; or strike doc |
| 97 | JCS wasm commitment domain-separation (DEFECT: unkeyed vs DOMAIN_TAG planes) | blake3 | `advanced.rs:47 + jcs_parity.rs` | S | untagged blake3(canonical) acceptance = FAIL |
| 98 | Parse-precision class repair (serde_json float_roundtrip, delete artifact class) | serde_json | `Cargo.toml both + jcs_parity.rs` | S | corpus parse_precision count == 0 with feature; revert flips nonzero |
| 99 | SMT batch-op surface + root permutation-invariance court | monotree | `src/smt.rs` | S/M | shuffled batch or batch-vs-sequential root divergence, or dup-key last-win = FAIL |
| 100 | Proof-structure budget law for verify_proof (>256/0-step/cut-sum refuse typed) | monotree | `src/smt.rs verify path` | S | oversized/zero-step proof accepted past recomputation = FAIL |

Cycle-4 defect-class: SMT absence-witness key-binding gap (verify_absence never checks neighbor_proof.key==neighbor_key — repair lane in flight); SMT empty-proof vacuity probe; dead src/verbs/fix.rs (DELETED, duplicate verb outside mod tree); quarantine sidecar wall-clock (handlers.rs:1859, repair lane in flight); `affi doctor` verb registered but no clap subcommand wired (pre-existing, unowned).

## Cycle-5 additions (2026-10-17; rows 101+)

| # | Proposal | Dep | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 101 | BehaviorCoverage grounding court (coverage cell increments as admission metric) | clnrm-core | new tests/behavior_coverage_court.rs | S/M | deleting an increment leaves cell green = vacuous FAIL |
| 102 | Dark-matter synthesis corpus import (hermeticity-violation scenarios as seeded fixtures) | clnrm-core | new tests/synthesis_corpus_court.rs | M | synthesized violation admitted, or non-deterministic across runs = FAIL |
