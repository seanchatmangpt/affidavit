# affidavit 📜

**The Provenance Layer for High-Assurance Systems.**

[![Rust](https://img.shields.io/badge/rust-1.78%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![1000x Initiative](https://img.shields.io/badge/1000x-Initiative%20Complete-green.svg)](STATUS.md)

`affidavit` is a cryptographic provenance engine designed to make the unverifiable unconstructable. It assembles, seals, and certifies **provenance receipts**—append-only, content-addressed BLAKE3 chains of operation-events that provide an immutable record of what a process actually did.

---

## 🏛️ Doctrine: Certify, Don't Decide

In complex systems, "honesty" is often undecidable. `affidavit` shifts the burden from detection to certification:

1.  **Witness-Based Verification:** The verifier doesn't hunt for fraud; it checks a *witness* (the receipt) against a formal format standard.
2.  **Decidable Pipeline:** Every stage of the 7-stage certify pipeline is decidable, yielding a definitive `ACCEPT` or `REJECT` verdict.
3.  **Unconstructable Bypass:** Valid receipts cannot be "faked" or manually constructed. They must pass through canonical, sealed seams in the library.
4.  **Content-Addressed Integrity:** Every event is linked via a rolling BLAKE3 hash. A single bit flip in any historical event invalidates the entire chain.

---

## 🚀 The 1000x Initiative

`affidavit` has been supercharged with 30+ features focused on **Combinatorial Maximalism** and world-class DX:

*   🏛️ **Evidence Federation:** Certify receipt-bound standing, cross-repo quorums, and formal ERRC transformations — see [`docs/FEDERATION.md`](docs/FEDERATION.md).
*   🔍 **Deep Introspection:** Auto-generate DFG/Petri models from receipts *(behind the `discovery` feature; see Feature status below)*.
*   🛡️ **Chaos Engineering:** Built-in mutation testing to stress-test your verifiers *(behind the `mutation` feature; see Feature status below)*.
*   🤖 **Intelligent CLI:** 92 canonical verbs, ontology-driven help, and powerful ad-hoc querying.

---

## 🛠️ Installation & Quick Start

### Build from Source
The toolchain is date-pinned in `rust-toolchain.toml`; rustup will fetch it for you.

```bash
git clone https://github.com/seanchatmangpt/affidavit
cd affidavit
cargo build --release
```

Run the full verification ladder with `just validate` (formatting, build, tests,
doctests, clippy, and the ERRC fast court).

#### Feature status

The default feature set is what ships and what CI gates. Several optional
features do **not** currently compile, so `--all-features` fails:

Each row below was verified with `cargo check --lib --features <name>`; rows
were last re-verified across the v26.9.28 trust-plane release and remain the
gating feature matrix at the v26.10.8 fleet tag:

| Feature | State |
|---------|-------|
| `default` (`core`) | ✅ builds, tested, and linted in CI |
| `inspection`, `lsp`, `shell`, `quality-monitor`, `file-watch`, `webhook`, `otel`, `gpu`, `remediation` | ⚠️ build, but no CI job covers them (ROADMAP P1-8) |
| `pqc` | ✅ cryptographic trust-plane lane: real ML-DSA-65 verification through RustCrypto, provider-owned private keys, replay/revocation negative courts |
| `discovery`, `conformance`, `predictive` | ❌ do not compile — they need `wasm4pm` APIs (`ilp_discovery`, `process_tree`, `models::EventLog`) that the local stub at `stubs/wasm4pm` does not expose |
| `mutation` | ❌ does not compile — needs `clnrm-core` APIs (`determinism::rng`) the local stub does not expose |

`remediation` was in the failing group until v26.9.6; it was missing a
`tracing` dependency its own module imports.

`stubs/wasm4pm` and `stubs/clnrm-core` are deliberate capability boundaries
(see [`AGENTS.md`](AGENTS.md) §1), not oversights: broadening one without an
observed integration proof would manufacture a green that means nothing. Closing
these out is tracked as ROADMAP P1-7.

### The "Golden Run" in 30 Seconds
Run the end-to-end smoke test to see `affidavit` in action:

```bash
./examples/golden_run.sh
```

### Verify Anywhere: the WebAssembly module
`affidavit-wasm/` ships the verifier as a sandboxed `.wasm` (~230 KB, WASI-only
imports) so a host can `verify`, `assemble`, `mine` and `conform` receipts
without shelling out to `affi` — and get the same verdict, proven against
receipts the real binary produced. See [`docs/WASM.md`](docs/WASM.md).

### The Cryptographic Trust Plane (new in v26.9.28)

Affidavit owns the ecosystem's cryptographic trust plane: key identity and
custody, JCS canonicalization, classical and post-quantum signatures, replay
and revocation law, and verification to a closed standing vocabulary. The
boundary is `certify-don't-decide`: a `CryptographicStanding` is evidence
about bytes and keys; whether that evidence authorizes an act is a decision
for the (downstream) authorization layer, never the trust plane. Post-quantum
is real cryptography — ML-DSA-65 (FIPS 204), SLH-DSA-SHA2-128s (FIPS 205), and
a hybrid ES256+ML-DSA-65 — replacing the retired blake3-mock seal. Limited by
design: Secure Enclave signing needs an entitlement-signed host (typed
`PARTIAL_ALIVE`); HSM is typed `UNSUPPORTED`. See [`docs/CRYPTO_TRUST_PLANE.md`](docs/CRYPTO_TRUST_PLANE.md) for the as-built
capability map, and [`SECURITY.md`](SECURITY.md) for the crypto posture.

Enable it with the `crypto-trust` feature (the enclave adapter adds
`secure-enclave`). Sign an envelope and verify it to a standing:

```rust
use affidavit::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
};
use affidavit::crypto_trust_lifecycle::RevocationList;
use affidavit::crypto_trust_verify::{CryptographicStanding, TrustPolicy, VerificationEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. A signing key (RFC 6979 deterministic ES256 over P-256) and its registry record.
    let signing = Es256SigningKey::generate()?;
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let mut registry = InMemoryKeyRegistry::new();
    registry.register(KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm: AlgorithmId::Es256,
        fingerprint,
        custodian: CustodianIdentity { subject: "builder@example.test".into(), device: None, org: None },
        origin: KeyOrigin::Generated,
        public_key: public,
        created_epoch: 1_700_000_000,
    })?;

    // 2. The envelope: all twelve fields live inside the signed bytes.
    let env = SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: KeyId::from_fingerprint(&fingerprint),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [0x5A; 16],
        not_before: 1_700_000_400,
        expires_at: 1_700_000_600,
        subject_digest: [0x11; 32],
        audience: "affidavit.verifier".to_string(),
    };

    // 3. Sign the domain-separated canonical pre-image; verify to a standing.
    let signature = signing.sign(&env.signing_input());
    let engine = VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(1_700_000_500),
    );
    let verdict = engine.verify_envelope(&env, &signature)?;
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
    println!("standing = {}", verdict.standing.as_str());
    Ok(())
}
```

### Signing & Auth Surface (advanced witness set)

On top of the ES256 attestation path, the envelope now admits Ed25519 (RFC
8032) and ES256K (RFC 8812) variants behind the `ed25519`/`secp256k1`
features; both flow through the same `SignatureEnvelope`/`verify_envelope`/
`certify_signed` path, with key selection from the signed `kid` only (alg/kid
mismatch refuses `UnknownKey` before any signature runs). Post-quantum
material (ML-DSA-65, SLH-DSA-SHA2-128s, hybrid ES256+ML-DSA-65) stays
sign/verify-capable but is outside the envelope surface.

JWKS export (`crypto_trust_jwks`, RFC 7517/7518/8037/8812): SEC1 → EC/P-256,
raw → OKP/Ed25519, compressed SEC1 → EC/secp256k1 (curve decompression);
PQ/hybrid material is typed-refused (`JwksError::UnsupportedAlgorithm`, never
a silent skip). Sets are kid-sorted and byte-deterministic; a set with one
`kid` under two algorithms refuses `JwksError::DuplicateKid` wholesale.

Optional `certified-receipts` feature: `certify_paid_delivery_payload` binds
`"affidavit-paid-delivery/v1|<subject>|<payload_hash_hex>"` as the subject and
mints a standing receipt through `certify_signed`; `verify_certified_paid_delivery`
enforces the linkage law — the carried receipt's commitment must equal
`blake3(signing_input)` with subject/key/algorithm bound, and the receipt is
re-audited (`receipt.verify()`) before any key registers.

Key rotation is court-pinned: an old `kid` stays valid alongside the new one
until revoked, then only the old `kid` refuses `KeyRevoked`. Threshold quorum
`share_verifies` mirrors the envelope dispatch with feature-gated Ed25519 and
ES256K arms.

The adversarial court (`tests/ag4_adversarial_crypto.rs`, 19 tests, real
keys/engine/wire forms) executes real attacks: duplicate JSON keys in the
envelope parser refuse pre-parse; high-s ES256K mirror signatures refuse;
JWKS duplicate-`kid` sets refuse; certified-receipt envelope swaps refuse via
the linkage law; alg-confusion, kid swap/empty, JCS float epoch and NFC/NFD,
and quorum duplicate-signer/share-reorder attacks refuse as-is.

---

## 📖 Core Concepts

### The Provenance Receipt
A receipt is the primary unit of evidence. It consists of:
- **Events:** Discrete operation records with monotonic sequence numbers.
- **Commitments:** BLAKE3 digests of payload data (payloads are never stored in the receipt).
- **Chain Seal:** A rolling hash that binds the entire history together.

### The 7-Stage Certify Pipeline
Each receipt passes through a rigorous validation gauntlet:
1.  **Decode:** Structural presence and version parsing.
2.  **Format Check:** Verification against the `core/v1` standard.
3.  **Chain Integrity:** Cryptographic re-computation of the rolling hash.
4.  **Continuity:** Logical sequence and uniqueness validation.
5.  **Commitment Verify:** Structural validation of all payload digests.
6.  **Profile Evaluation:** Conformance scoring against business logic.
7.  **Final Verdict:** Atomic `ACCEPT` or `REJECT` output.

### EventBuilder
`affidavit::event_builder::EventBuilder` is the type-safe, preferred public API for constructing events before appending them to a chain — its `build()` delegates to `build_event`, so it applies the same admission checks (see `examples/event_builder.rs`).

---

## 💻 CLI Surface

Affidavit ships **92 canonical verbs** across 11 groups, backed by a compile-time static registry (`src/registry.rs`) that is the authoritative single source of truth for help, completions, and documentation. The registry, the `#[verb]` projections under `src/verbs/`, and the authoritative ontology (`ontology/affi-cli.ttl`) are held in agreement by parity tests, so none of the three can drift.

Shell completions ship for five shells — bash, zsh, fish, PowerShell (`completions/affi.ps1`), and Nushell (`completions/affi.nu`) — generated from the verb registry by `scripts/generate_completions.py` and drift-enforced by `tests/completions_drift.rs`.

**Core Verbs (The Provenance Loop):**
- `affi emit` — Record a new operation-event.
- `affi assemble` — Finalize and seal the current receipt.
- `affi verify` — Run the certify pipeline against a receipt.
- `affi show` — Inspect receipt details.

**Western Electric Quality (Real-Time Monitoring):**
- `affi quality monitor` — Start Western Electric live statistical process control monitoring.
- `affi quality portfolio` — Analyze portfolio health across repositories.
- `affi quality trend-analysis` — Display historical degradation metrics.

**SBOM & Supply Chain Provenance:**
- `affi sbom scan` — Generate SBOM representation (SPDX/CycloneDX).
- `affi sbom attest` — Sign and bind an SBOM to the cryptographic provenance chain.
- `affi sbom blast-radius` — Calculate vulnerability risk propagation in the dependency graph.
- `affi sbom compliance` — Run NTIA minimum-element compliance verification.

**Advanced Auditing:**
- `affi receipt model` — Generate architectural models from provenance.
- `affi causality-chain` — Track root cause and event lineage.
- `affi security-debt` — Calculate pending remediation metrics.

**Evidence Federation (v26.9.6; still current at v26.10.8):**
- `affi standing certify` / `verify` — Seal and re-check a receipt-bound standing claim. ALIVE is unconstructable without execution, verification, *and* replay evidence.
- `affi ecosystem certify` / `verify` — Federate exact, already-sealed member standing receipts against a declared per-role ALIVE quorum.
- `affi errc certify` / `verify` — Seal a formal ERRC transformation: each claim bound to one `(target, metric, unit)` coordinate under one directional law, behind a mandatory preservation fence.
- `affi errc assure` / `verify-assurance` — Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt.

See [`docs/FEDERATION.md`](docs/FEDERATION.md) for the full operator guide, the exit-code contract, and a worked example.

**Health & Diagnostics:**
- `affi doctor` — Run environment and receipt-store health checks with structured exit codes.
- `affi --version` — Reports the affidavit version. This matters: the chain genesis seed is bound to it, so a receipt only verifies under the exact binary version that assembled it.

*(Full list: `affi --help`, or `affi guide search <keyword>`. Verb registry: `src/registry.rs`. Groups: Core · Diagnostics · Analysis · Ingestion · Compliance · Attestation · SBOM · Insights · Engineering · Tooling · Federation)*

---

## 🛡️ Security Model

`affidavit` is designed for high-stakes environments where provenance is non-negotiable:
- **Zero-Knowledge Payloads:** We store commitments, not raw data, protecting sensitive information.
- **Deterministic Hashing:** Canonical JSON serialization ensures hashes are stable across platforms.
- **Memory Safety:** Written in 100% `safe` Rust (enforced via `#![deny(unsafe_code)]`).
- **Cryptographic Trust Plane:** `crypto_trust` owns key identity, public verification material, custody metadata, epochs, replay semantics, signatures, and verified cryptographic standing. Private signing capability remains behind provider boundaries such as Secure Enclave/HSM/KMS.
- **Proof ≠ Permission:** A verified signature is evidence only. Affidavit does not convert cryptographic standing into consequence authority; consumers such as SA2A/BRCE make that separate decision.

See [`docs/CRYPTO_TRUST.md`](docs/CRYPTO_TRUST.md) for the boundary and consumer contract.

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to participate in the provenance revolution.

## 📄 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
