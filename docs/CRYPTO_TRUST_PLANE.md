# The Cryptographic Trust Plane

**Status**: v26.9.28 — as-built capability map (extended wave 3, 2026-09-28)
**Companion pack**: `ggen-marketplace/packs/affidavit-trust-plane-pack` (manufactured by `ggen sync`; the ontology is the source, the modules below are projections — never hand-edited)

---

## Boundary law: who owns what

Affidavit owns the ecosystem's **cryptographic trust plane**: keys, custody
semantics, canonicalization, post-quantum algorithms, signatures, verification,
revocation, and cryptographic standing. **SA2A owns what that standing is
allowed to authorize.**

No authority inversion. The loop runs one way:

```
            proof (CryptographicVerdict → CryptoStandingReceipt)
  ┌──────────────────────────────────────────────────────────────┐
  │                                                              │
  ▼                                                              │
AFFIDAVIT  ──────────proof──────────▶  SA2A  ──authority──▶  ACTUATOR
(keys, custody, JCS, PQC,                    (what the standing   (acts only
 signatures, verification,                    may authorize)       under authority)
 revocation, standing)
  ▲                                                              │
  └────────────────── evidence / receipt ────────────────────────┘
```

Affidavit certifies and refuses; it never decides. A
`CryptographicStanding` value is **evidence about bytes and keys** — that a
signature verified, that a key was live and unrevoked at the questioned
instant. Whether that evidence is *sufficient* to authorize an actuator is an
SA2A decision, made downstream, on top of the receipt — never inside the trust
plane. `certify-don't-decide` is the house law: refusals are typed values,
never panics, never silent coercion.

## Module map (9 rendered modules + the end-to-end court)

All rendered from the pack under `#[cfg(feature = "crypto-trust")]`
(`crypto_trust_enclave` additionally under `secure-enclave`). Every module
carries rendered facts (constants, serde wire forms, length laws) from the
`ctp:` graph and its own test court; the verification column names the
strongest witness each module carries.

| module | capability | verification |
|---|---|---|
| `crypto_trust_canonical` | JCS (RFC 8785) canonical serialization — UTF-16 key ordering, minimal escaping, ECMAScript number rendering — and domain-separated BLAKE3 digests | RFC 8785 Appendix B number vectors; typed refusal of integers beyond 2^53 (`NonCanonicalNumber`); domain divergence tests |
| `crypto_trust_keys` | `AlgorithmId`, `CryptoProfile`, `KeyId` (`afk1_` + 16 hex chars), `KeyFingerprint`, `KeyRecord`, custody (`CustodianIdentity`, `KeyOrigin`), `PublicKeyMaterial`, `InMemoryKeyRegistry` | duplicate-id and duplicate-fingerprint refusals; fingerprint divergence on algorithm/bytes/domain; serde wire forms |
| `crypto_trust_envelope` | `SignatureEnvelope` (12-field signed bytes, `CTP-ENVELOPE-v1`), `NonceJournal` replay evidence | field registry closure (`ENVELOPE_FIELDS` vs document keys); 12-field mutation divergence (every field moves the pre-image); window boundary law; replay window/boundary semantics; JCS 2^53 refusal |
| `crypto_trust_es256` | ES256 signing key (P-256, SHA-256, RFC 6979 deterministic) + `verify_es256` | RFC 6979 A.2.5 known-answer vector (exact DER); tamper falsifiers (message, signature structure, scalar, wrong key); zero-seed refusal |
| `crypto_trust_pqc` | ML-DSA-65 (FIPS 204), SLH-DSA-SHA2-128s (FIPS 205), hybrid ES256+ML-DSA-65 (both halves over the same bytes; BOTH must verify) | FIPS encoded-length assertions (1952/3309, 7856); explicit-rnd determinism; tamper and cross-key falsifiers per algorithm and per hybrid half; domain-bound hybrid randomizer |
| `crypto_trust_verify` | `VerificationEngine`, `TrustPolicy`, `CryptographicVerdict`, closed `CryptographicStanding` (order 0..7), `CryptoStandingReceipt`, `VerifyRefusal` | the ordered admission law (bytes → window → policy → registry → revocation → replay → signature); decided-negative asymmetry (`Invalid` is `Ok`); certification refuses every non-VALID outcome; receipt-hash tamper refusal on deserialize |
| `crypto_trust_lifecycle` | `KeyEpoch`, `RotationPolicy` (default: 2 epochs in flight, 90-day max age), `RevocationList` doubling as the revocation-epoch clock, staleness grace | epoch monotonicity; rotation caps (`EpochsExhausted`, `EpochExpired`); staleness-grace boundary law |
| `crypto_trust_seal` | `SealedReceipt` (`PQ-SEAL-v1`): envelope + signature bound to an affidavit `Receipt` via its content address | `subject_digest` binding refusal (`SubjectMismatch`); tampered base receipt refuses deserialization (inherited chain law); re-derivation of the digest from the base, never from the envelope's claim |
| `crypto_trust_enclave` | signing-provider dispatch: SOFTWARE / SECURE_ENCLAVE (macOS Security.framework, non-exportable P-256) / HSM refused (typed UNSUPPORTED, never faked) | deletion hygiene (every created key deleted; post-delete signing refuses); unknown-label typed refusals; `UnsupportedPlatform` on non-macOS; live KAT is `#[ignore]`-gated (see honest standing table) |
| `tests/crypto_trust_e2e.rs` | the end-to-end court: ontology → keys → envelope → sign → verify → standing → seal → tamper → refuse | **BUILD_BROKEN at v26.9.28 wave 3** — 10 `E0432` errors (`crate::` imports in an external integration test; the fixes are mechanical: `affidavit::` paths). Recorded honestly; standing below reflects this |

## The envelope: 12 signed fields (RFC-SA2A-007-errata interlock)

The signed-bytes envelope is the contract surface between this trust plane and
SA2A. Exactly twelve fields, canonical order 1..12, all *inside* the signed
bytes (never trusted from a header). The field closure is a pack gate
(`020_envelope_field_closure.rq`); the envelope version is pinned
(`CTP-ENVELOPE-v1`, gate `060`).

| # | field | type | why it is signed |
|---|---|---|---|
| 1 | `version` | string (`CTP-ENVELOPE-v1`) | the envelope law itself; `from_bytes` refuses any other version typed (`WrongVersion`) |
| 2 | `algorithm` | `AlgorithmId` | a signature does not bind a key it does not name |
| 3 | `key_id` | `KeyId` | which registered key adjudicates |
| 4 | `profile` | `CryptoProfile` (CLASSICAL/HYBRID/PQC) | the assurance floor is enforced against a signed claim |
| 5 | `policy_epoch` | u64 | which policy generation the signer knew |
| 6 | `revocation_epoch` | u64 | the revocation-epoch stamp the staleness grace reads |
| 7 | `generation` | u32 | key-generation discriminator within a key id |
| 8 | `nonce` | 16 bytes | half of the replay tuple `(kid, nonce)` |
| 9 | `not_before` | u64 | validity-window start (inclusive) |
| 10 | `expires_at` | u64 | validity-window end (inclusive) |
| 11 | `subject_digest` | 32 bytes | domain-separated BLAKE3 binding to the attested object |
| 12 | `audience` | string | who may consume the verdict |

Signatures are computed over
`domain_separated("affidavit.crypto-trust-plane.v1", [jcs(envelope_document)])`
— JCS (RFC 8785) canonical JSON digested with BLAKE3 under the domain tag.
Both sides of the interlock must reproduce the exact same bytes; there is no
"approximately canonical". Two further interlock laws:

- **Replay** — the replay key is the `(kid, nonce)` tuple with a 300-second
  acceptance window (half-open `[first_seen, first_seen + 300)`); a repeated
  tuple is refused as `CryptographicStanding::ReplayRejected`, a value SA2A
  can observe. At or beyond the window boundary the entry restamps and the
  record is admitted.
- **Revocation-epoch staleness** — every revocation advances the global
  revocation epoch (`max(revoked_at) + 1`); signatures stamp the epoch they
  were made under. A signature naming a stale epoch is refused as revoked once
  the key's own revocation is more than 300 seconds in the past — inside the
  grace, a lagging revocation may simply not have propagated yet.

## Algorithms and measured sizes

All sizes below are asserted by the rendered modules' tests (not transcribed
from spec sheets) and were re-verified this session against the test sources:

| algorithm (`AlgorithmId`) | profile | spec | public key | signature | seed | notes |
|---|---|---|---|---|---|---|
| `ES256` | CLASSICAL | RFC 6979 / SEC 2 | 65 B (SEC1 uncompressed `04 \|\| X \|\| Y`) | 70–72 B DER (X9.62; the A.2.5 vector yields 72) | 32 B scalar (`from_seed`, test/fixture derivation only) | deterministic: no RNG on the signing path |
| `ML-DSA-65` | PQC | FIPS 204 | 1952 B | 3309 B | 32 B | explicit-rnd `Sign_internal`; same seed+msg+rnd ⇒ byte-identical signature |
| `SLH-DSA-SHA2-128s` | PQC | FIPS 205 | 32 B (`pk_seed \|\| pk_root` = 2n, n=16) | 7856 B | 48 B (`sk_seed \|\| sk_prf \|\| pk_seed`) | deterministic by construction |
| `ES256+ML-DSA-65` (hybrid) | HYBRID | draft-ietf-lamps-pq-composite-sig | 65 + 1952 B (concatenated classical-then-PQ for fingerprinting) | JSON of both halves: ES256 DER + ML-DSA-65 encoded | 32 B + 32 B | both halves sign the SAME bytes; verification requires BOTH. The ML-DSA randomizer is the domain-separated BLAKE3 of the message, keeping the hybrid deterministic and domain-bound |

The graph does not declare fixed lengths for the hybrid and SLH public-key
rows (`AlgorithmId::public_key_len` returns `None`); the concrete byte counts
above come from `PublicKeyMaterial::encoded_len` and the length tests.

## The standing vocabulary (closed, graph order 0..7)

`CryptographicStanding` is a closed eight-value vocabulary; the serde wire
form is SCREAMING_SNAKE_CASE and equals the graph's `ctp:standingName`.

| order | standing | meaning for the verifier |
|---|---|---|
| 0 | `VALID` | the signature verified over reconstructable signed bytes, inside the window, under an admitted algorithm and profile, against a known unrevoked key, with a fresh `(kid, nonce)` |
| 1 | `INVALID` | a **decided negative**: the signature check ran and did not verify. Returned `Ok` — it is a verdict, not a refusal |
| 2 | `EXPIRED` | the verifier clock is past `expires_at` |
| 3 | `REVOKED` | the key carries a revocation record (flat law, or staleness grace elapsed) |
| 4 | `REPLAY_REJECTED` | the `(kid, nonce)` tuple was already journaled inside the 300-second window |
| 5 | `UNKNOWN_KEY` | the key id is absent from the registry, or the registered record disagrees with the envelope's claims |
| 6 | `PROFILE_REFUSED` | the algorithm is outside the policy's allowed set, or the declared profile is below the policy floor |
| 7 | `MALFORMED` | the envelope could not be parsed or its signed bytes could not be reconstructed |

The distinction between a verdict and a refusal is deliberate: values 1–7
above as *verdict/refusal outcomes* all prevent authorization, but only
`VALID` ever mints a `CryptoStandingReceipt`, and refusals that prevent
adjudication itself (window, policy, registry, replay, provider) are typed
`VerifyRefusal` values rather than standing verdicts.

## Migration policy: classical → hybrid → post-quantum

`ES256 → ES256+ML-DSA-65 → ML-DSA-65` is an **affidavit profile change**
(`CryptoProfile::Classical → Hybrid → Pqc`), not an SA2A protocol rewrite.
The envelope carries the algorithm and profile inside the signed bytes, so the
two sides of the interlock negotiate by reading fields that already exist —
the 12-field envelope, the JCS canonicalization, and the standing vocabulary
do not move. Verifiers admit algorithms by profile floor (`ctp:minProfile`,
rendered default floor `CLASSICAL`, rendered default profile for new seals
`HYBRID`); older signatures remain verifiable within their validity windows
under the profile they declare.

| transition | direction | law |
|---|---|---|
| CLASSICAL → HYBRID | upgrade (recommended next step; the rendered default profile for new seals) | signer adopts `AlgorithmId::HybridEs256MlDsa65` + `CryptoProfile::Hybrid`; verifiers whose floor is `CLASSICAL` admit it unchanged — the profile field is inside the signed bytes, so no verifier-side renegotiation exists |
| HYBRID → PQC | upgrade | signer drops the classical half (`ML-DSA-65` / `SLH-DSA-SHA2-128s` + `CryptoProfile::Pqc`); any verifier still pinning a HYBRID floor refuses typed (`ProfileFloor`) until its policy is raised deliberately |
| HYBRID → CLASSICAL | downgrade (refuse by default) | a verifier may only admit this when its policy floor is `CLASSICAL`; a floor of `HYBRID` or `PQC` refuses the envelope typed. Downgrade is a policy decision, never a silent fallback |

Policy, not code, moves across every transition: `TrustPolicy` is data
(`min_profile`, `allowed` algorithm set, staleness bound, verifier clock), and
the engine never reads wall-clock time or widens its own admission set.

## Honest standing table (plane itself, as of 2026-09-28)

Standing of the trust plane itself, stated with teeth (the same vocabulary it
certifies with). Every ALIVE claim below was observed in this session with the
repo-pinned toolchain (`nightly-2026-08-12`); observation commands are listed
so the claim can be re-executed.

| capability | standing | evidence law |
|---|---|---|
| library compiles under `crypto-trust` | **ALIVE** | `cargo check --features crypto-trust --lib` — exit 0, this session |
| library compiles under `secure-enclave` | **ALIVE** | `cargo check --features secure-enclave --lib` — exit 0, this session |
| sign → verify → `VALID` path (ES256, envelope, engine) | **ALIVE** | a program exercising `Es256SigningKey` + `SignatureEnvelope` + `VerificationEngine` compiled and executed this session and returned `standing = VALID` |
| rendered doctests | **ALIVE** | `cargo test --features crypto-trust --doc` — 37 passed, 0 failed, this session |
| software signing (ML-DSA-65 / SLH-DSA / hybrid) | **ALIVE (test-source level)** | rendered from FIPS 204/205 parameter sets; lengths, determinism, and tamper falsifiers are asserted in `crypto_trust_pqc.rs`'s test court — but see the BUILD_BROKEN row: the unit court does not currently compile, so those assertions are re-run from source only after the fix below |
| crypto unit-test court (`--lib`) | **BUILD_BROKEN** | 1 error: `src/crypto_trust_seal.rs` test module calls `.register` without `use crate::crypto_trust_keys::KeyRegistry;` in scope (`E0599`). One-line import fix; not repaired in this documentation lane |
| end-to-end court (`tests/crypto_trust_e2e.rs`) | **BUILD_BROKEN** | 10 errors, all `E0432`: `crate::` imports in an external integration test (e.g. `use crate::verifier::verify;` must be `use affidavit::verifier::verify;`). Mechanical fix; not repaired in this documentation lane |
| secure enclave signing (live) | **PARTIAL_ALIVE** | the live known-answer tests are `#[ignore]`-gated: they run only on enclave-capable hardware when the operator witnesses them. Witnessed finding (2026-09-28, wave 2): persisting a Secure Enclave key from an unsigned/ad-hoc CLI binary is refused by the keychain with **OSStatus -25308** (with the access-control object) / **-34018 `errSecMissingEntitlement`** (without) — the test is not faked. Unblock: run from an entitlement-signed host binary (`cargo test --features secure-enclave secure_enclave_full_cycle -- --ignored`, or `live_` for the KAT). Until a witnessed run exists, this stays PARTIAL_ALIVE — inspection is not execution |
| HSM / TPM signing | **UNSUPPORTED** (typed) | `ctp:provider-HSM` carries `ctp:admitted false`; consuming code must refuse with a typed value, never fake a signature |

## Consumption

```rust
use affidavit::crypto_trust_seal::{seal_receipt, verify_sealed, SealedReceipt, SEALED_RECEIPT_FORMAT};

// receipt: affidavit::types::Receipt (chain-sealed, content-addressed)
let sealed: SealedReceipt = seal_receipt(&receipt, envelope, signature)?;
let verdict = verify_sealed(&sealed, &engine)?;
```

The base receipt's own law is untouched: `Receipt` deserialization re-verifies
the BLAKE3 chain, so a tampered base cannot even become a `Receipt` value. The
seal adds a second, independent binding — **who attests** the receipt, under
which key, epoch, and policy — via the envelope's `subject_digest`, which must
equal the domain-separated BLAKE3 digest of the receipt's content address.
A compile-and-run example of the full sign → verify path is in the
[README](../README.md#the-cryptographic-trust-plane-new-in-v26928).

## Measured performance (observed 2026-09-28)

Every number below is **OBSERVED on this machine** (darwin/arm64,
repo-pinned toolchain `nightly-2026-08-12`) by running the real criterion
suite against the real crypto — none is an estimate, a datasheet figure, or a
claim about any other hardware. Criterion settings: `--sample-size 10
--warm-up-time 1 --measurement-time 2`; the median column is criterion's
`median.point_estimate` from `target/criterion/<bench>/new/estimates.json`.
With 10 samples the confidence intervals are wide; treat each row as one
witnessed observation, not a floor or a guarantee. The pack-side gate
`gates/120_performance_budget.py` (affidavit-trust-plane-pack) re-runs the
`ctp_es256_verify` bench and refuses if this table's median for it drifts
outside one order of magnitude of a fresh measurement on the same machine.

Reproduce with:

```text
CARGO_TARGET_DIR=/tmp/ctp-shared-target \
cargo bench --features crypto-trust --bench crypto_trust_bench -- \
    --sample-size 10 --warm-up-time 1 --measurement-time 2
```

| bench id | operation | median (observed) | machine |
|---|---|---|---|
| `ctp_es256_sign` | ES256 sign | 131.47 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_es256_verify` | ES256 verify | 216.63 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_ml_dsa65_sign` | ML-DSA-65 sign | 526.61 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_ml_dsa65_verify` | ML-DSA-65 verify | 96.85 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_slh_dsa128s_sign` | SLH-DSA sign | 106.84 ms | darwin/arm64, nightly-2026-08-12 |
| `ctp_slh_dsa128s_verify` | SLH-DSA verify | 105.49 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_hybrid_sign` | Hybrid ES256+ML-DSA-65 sign | 812.91 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_hybrid_verify` | Hybrid ES256+ML-DSA-65 verify | 313.97 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_jcs_1kb_object` | JCS canonicalize, 1250-byte JSON object | 3.4891 µs | darwin/arm64, nightly-2026-08-12 |
| `ctp_envelope_signing_input` | envelope signing pre-image | 4.2261 µs | darwin/arm64, nightly-2026-08-12 |

