# CONSOLIDATION — retire `trust-plane-legacy` (lane W1-L2, wave 1, v26.9.28)

Subject: `/Users/sac/affidavit` branch `feat/v26.9.28-crypto-trust-plane` @ `48563e4`.
Scope: consolidation of the parallel-session WIP `src/crypto_trust.rs` (845 lines, gated
behind `trust-plane-legacy`) into the pack-rendered plane
(`ggen-marketplace/packs/affidavit-trust-plane-pack` → `src/crypto_trust_*.rs`).

Ground truth: the rendered plane. No `src/` byte was edited by this lane; the
retirement ships as a coordinator-applied patch (`/tmp/w5-w1-lane2/retire.patch`).

## Why retire (compile-break receipt)

`cargo check --features trust-plane-legacy` (exit 101, log:
`/tmp/w5-w1-lane2/legacy-check.log`):

- `E0432`: unresolved import `crypto_trust::JCS_SAFE_INTEGER_MAX` (used at
  `crypto_trust.rs:553,685`, declared nowhere)
- `E0425`: cannot find function `canonical_bytes` (used at `crypto_trust.rs:494`)
- `E0425`: cannot find value `JCS_SAFE_INTEGER_MAX`
- `E0433`: cannot find module or crate `serde_jcs` (`crypto_trust.rs:223`; the
  crate depends on `serde_json`, not `serde_jcs`)

The WIP never compiled after gating; the rendered plane has carried every
capability since (24 executed witnesses, below).

## Coverage court — capability by capability

| # | WIP item (`crypto_trust.rs`) | disposition | evidence (module::item + executed witness) |
|---|---|---|---|
| 1 | `SIGNATURE_PROFILE` ("affidavit/crypto-standing/v1") | covered-by | `crypto_trust_verify::CRYPTO_STANDING_PROFILE` — same string; witness `w1` |
| 2 | `SIGNING_DOMAIN` (domain-separated signed bytes) | covered-by | `crypto_trust_canonical::{DOMAIN_TAG, domain_separated}` — domain-bound byte input; witness `w2` |
| 3 | `SignatureAlgorithm` {Es256, MlDsa65, SlhDsaSha2_128s} | covered-by (superset) | `crypto_trust_keys::AlgorithmId` (+ `HybridEs256MlDsa65`); witness `w3` |
| 4 | `KeyCustody` {SecureEnclave, Tpm, Hsm, Kms, Software, External} | covered-by | `crypto_trust_keys::KeyProviderKind` + `KeyOrigin` (+ `crypto_trust_enclave::PROVIDER_KIND`, real Secure Enclave); Tpm/Kms/External labels are ontology vocabulary facts (`ctp:providerKind` individuals), not code capability — the non-exportable custody law is carried (`key_material_exportable()`); witness `w4`. Failed edge recorded: label-breadth of the WIP custody enum is NOT carried verbatim; admission of Tpm/Kms individuals is a graph change, out of lane scope. |
| 5 | `KeyState` {Active, Revoked} | covered-by | `crypto_trust_lifecycle` (`KeyEpoch::active_at`, `RevocationList::is_revoked/revoke`) + `VerifyRefusal::KeyRevoked`; witness `w5` |
| 6 | `KeyRecord` (public material only) | covered-by | `crypto_trust_keys::KeyRecord` (id/algorithm/fingerprint/custodian/origin/public_key/created_epoch); no private field; witness `w6` |
| 7 | `KeyRegistry` (register/get/revoke/iter, dup refusal) | covered-by (superset) | `crypto_trust_keys::KeyRegistry` TRAIT + `InMemoryKeyRegistry` + `crypto_trust_store::FileKeyStore` (restart-durable, checksummed — a capability the WIP type explicitly lacked); witnesses `w7` |
| 8 | `SigningMaterial` + `signing_bytes()` (JCS + domain) | covered-by | `crypto_trust_envelope::SignatureEnvelope` + `signing_input_checked()` over the 12 graph-declared fields (`ENVELOPE_FIELDS`); binding of every security coordinate witnessed by mutation; witness `w8` |
| 9 | `SignatureEnvelope` | covered-by | `crypto_trust_envelope::SignatureEnvelope` (signature detached on the wire; `verify_envelope(&env, sig)` consumes them separately) |
| 10 | `SigningProvider` trait | **NEW — ported** | rendered plane is static-dispatch (`Es256SigningKey::sign`, `ml_dsa65_sign`, `enclave_sign` free fns); no provider-polymorphic seam existed → ported-as `crypto_trust_provider` |
| 11 | `sign_with_provider` | **NEW — ported** | ported-as `crypto_trust_provider::sign_with_provider` (detached-signature form, wired to the plane's real signing input law) |
| 12 | `VerificationContext` (now/audience/subject/min-epoch) | covered-by | `crypto_trust_verify::TrustPolicy` (+ `CryptographicVerdict.subject_digest` as the consumer-side subject binding); witnesses `w9` |
| 13 | `NonceLedger` (in-memory) | covered-by | `crypto_trust_envelope::NonceJournal` (window law strictly superior: in-window reject, boundary restamp, counted prune); witness: twin-law test inside `crypto_trust_nonce_store` |
| 14 | `NonceLedger` persistence ("persist this state when replay resistance must survive restarts") | **NEW — ported** | no rendered module persisted nonce state (grep over store/journal/log: absent) → ported-as `crypto_trust_nonce_store` (`DiskNonceJournal`, JSONL, `sync_data` on every admission, atomic prune rewrite) |
| 15 | `VerifiedSignature` (unconstructable standing) | covered-by | `crypto_trust_verify::{CryptographicVerdict, CryptoStandingReceipt}` (+ `pub(crate) seal` = unconstructable outside the plane); witness `w10` |
| 16 | `verify_and_record` | covered-by (superset) | `crypto_trust_verify::VerificationEngine::{verify_envelope, certify}` — replay+window+revocation+profile adjudication, decided-negative semantics; witness `w9` |
| 17 | `CryptoRefusal` (26 variants) | covered-by (distributed) | typed refusal families: `VerifyRefusal`, `RegistryError`, `EnvelopeError`, `LifecycleRefusal`, `StoreError`, `ProviderRefusal` (new); every WIP refusal maps to a variant in one of these (subject/audience/epoch/replay/verifier-unavailable/provider-mismatch) |
| 18 | `JCS_SAFE_INTEGER_MAX` (2^53 integer law) | covered-by | `crypto_trust_canonical::jcs` refuses non-I-JSON integers (2^53+1 → typed error, 2^53 exact); witness `w11` |

Court result: 15 covered-by (with executed witnesses for every claim that names
behavior), 3 genuinely-new capabilities detected (items 10, 11, 14 — the
provider seam counting as one capability with its entrypoint).

## Ported-as (pack units, real code, full tests)

1. **`crypto_trust_provider`** — provider-polymorphic signing seam:
   `SigningProvider` trait (key_id/algorithm/sign), `sign_with_provider`
   (identity checks BEFORE private capability; empty-signature refusal;
   uncanonicalizable-envelope refusal), `DetachedSignature`
   (envelope + detached signature, the plane's wire form),
   `SoftwareEs256Provider` (real reference implementation over
   `crypto_trust_es256::Es256SigningKey`; Debug omits the key). Pack files:
   `queries/provider.rq` + `templates/crypto_trust_provider.rs.tmpl`, bound as
   rule `provider` in the pack `ggen.toml`. Renders from `ctp:policy-v1
   ctp:domainTag/ctp:envelopeVersion`; conformance-tested equal to
   `crypto_trust_canonical::DOMAIN_TAG` and `crypto_trust_envelope::ENVELOPE_VERSION`.
2. **`crypto_trust_nonce_store`** — disk-backed replay ledger:
   `DiskNonceJournal` (append-only JSONL, header format `CTP-NONCE-JOURNAL-v1`,
   typed `Io/WrongFormat/Corrupt{line}/ReplayRejected` refusals, `sync_data`
   per admission, atomic tmp+rename prune, last-line-wins restamp across
   restarts), `default_journal_path()` derived from `ctp:store-v1
   ctp:storeFile` (`.affi/keys.json` → `.affi/nonces.jsonl`). Pack files:
   `queries/nonce-store.rq` + `templates/crypto_trust_nonce_store.rs.tmpl`,
   bound as rule `nonce-store`. Twin test proves it implements
   `crypto_trust_envelope::NonceJournal`'s window law exactly.

Verification (scratch `/tmp/w5-w1-lane2/verify`, `CARGO_TARGET_DIR=/tmp/ctp-shared-target`,
toolchain `nightly-2026-08-12` per consumer pin):
`cargo test` → 24/24 ok (12 in-module + 12 coverage-court); `cargo fmt -- --check` clean;
`cargo clippy --all-targets -- -D warnings` clean. Render = template + `sed`
graph-const substitution (domain_tag `affidavit.crypto-trust-plane.v1`,
envelope_version `CTP-ENVELOPE-v1`, window `300`, replay_key `kid,nonce`,
store_file `.affi/keys.json`); rendered bytes at `/tmp/w5-w1-lane2/crypto_trust_{provider,nonce_store}.rs`.

## Retirement (patch for the coordinator — NOT applied by this lane)

`/tmp/w5-w1-lane2/retire.patch` (unified diff, applies clean:
`patch --dry-run -p0` verified). Removes, in order:

1. `src/crypto_trust.rs` (845 lines, whole file)
2. `src/lib.rs`: the `#[cfg(feature = "trust-plane-legacy")] pub mod crypto_trust;`
   declaration + comment; the gated `pub use crypto_trust::{...}` re-export block
3. `Cargo.toml`: the `trust-plane-legacy = []` feature + its comment

The same patch WIRES the two new modules (`pub mod crypto_trust_nonce_store;`,
`pub mod crypto_trust_provider;` under `#[cfg(feature = "crypto-trust")]`,
alphabetical). Apply order: run `ggen sync` FIRST so the consumer render
materializes `src/crypto_trust_provider.rs` + `src/crypto_trust_nonce_store.rs`
(and their lib.rs wiring if sync manages it), THEN apply the patch (its mod
insertions are idempotent-checkable: skip if sync already wired them), THEN
`cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings`.

## Exclusions / refused

- No `src/` edits by this lane (contract: diff proposals only).
- No `ggen sync` run (forbidden for lanes); render proven by sed substitution
  against the same graph facts the queries select.
- No ontology.ttl edit (not in lane ownership); both new queries bind EXISTING
  graph individuals/predicates by pinned predicate, zero graph changes.
- Tpm/Kms custody labels: recorded failed edge (row 4), pack-vocabulary
  admission path noted, not silently pruned.
- Agents ran no git state commands; patch application left to the coordinator.
