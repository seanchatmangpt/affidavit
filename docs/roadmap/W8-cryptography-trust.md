# W8 — Cryptography & Trust

**Workstream:** W8 (Cryptography & Trust)
**Owner role:** the cryptographic trust layer on top of BLAKE3 content-addressing
**Horizon:** 2026 H2 → 2030
**Status:** design / proposal · grounded against the tree at `26.6.17`
**Doctrine:** *certify, don't decide.* A signature attests **who** assembled or witnessed a receipt and **that its bytes are unaltered** — never that the work is honest, good, or virtuous.

> **Build caveat.** The private-registry `26.6` deps (`clap-noun-verb`, `clnrm-core`, `wasm4pm`, `lsp-max`, …) do not resolve in a lone checkout, so nothing here was `cargo build`/`test`-verified. All Rust is compilable-*style*: correct against the patterns in-tree, pending signature finalization against the sibling crates.

---

## 1. Mission & scope

### 1.1 Mission

Turn `affidavit`'s ad-hoc, stubbed signing verbs into a **coherent, sealed-invariant key + signature + transparency model**: every finalized receipt can carry a cryptographically verifiable statement of *authorship* (who assembled it) and *witnessing* (who countersigned it), backed by an append-only transparency log with inclusion proofs, and protected against a future cryptographically-relevant quantum computer (CRQC) via a hybrid classical+PQC migration that never strands old receipts.

The North Star: a receipt is a **non-forgeable carrier** today (the sealed `Receipt` + chain-rehash on deserialize, `src/types.rs:110-144`), and W8 extends that non-forgeability from *integrity* (BLAKE3) to *authenticated authorship + third-party attestation + long-horizon (100-year) survivability*.

### 1.2 In scope (W8 owns)

1. **Signing & attestation** — mature `sign` / `attest` / `notarize` / `assemble-with-signature` / `assemble-and-notarize` / `sbom-attest` into one key+signature model. Ed25519 first.
2. **Key management** — generation, rotation, on-disk encrypted storage, verification keys, trust roots/anchors, and a `keyring` source-of-truth.
3. **Post-quantum crypto** — fill the empty `pqc` Cargo feature (`Cargo.toml:167`) with real ML-DSA (Dilithium) / ML-KEM (Kyber), replacing the `mock_*` placeholders in `src/1000x_post_quantum_sealing.rs`, with a **hybrid** (Ed25519 + ML-DSA) signature and a documented migration path.
4. **Transparency log / notarization** — an append-only Merkle log of signed-receipt heads, RFC 3161 / RFC 9162-style timestamping, inclusion + consistency proofs, and **witness co-signing** (third-party countersignatures over log checkpoints).
5. **Supply-chain trust** — in-toto/SLSA-framed provenance attestations (the `attest` and `sbom-attest` predicates), signed and logged.

### 1.3 Boundaries (NOT W8)

| Concern | Owner | W8 relationship |
|---|---|---|
| Verification **mechanics / performance / Merkle-distribution** | **W7** | W8 *provides* the signature & inclusion-proof primitives W7's distributed verifier consumes; W8 defines the verify-stage contract, W7 schedules/parallelizes/shards it. |
| Turning attestations into **compliance reports** (SOC2/SLSA-level rollups) | **W10** | W8 emits the signed, logged provenance facts; W10 aggregates them into governance evidence. |
| **SBOM ingestion / normalization** (SPDX/CycloneDX → canonical `Sbom`) | **W9** | W8 consumes the canonical `Sbom` address (`src/sbom_supply_chain.rs`) and signs the resulting attestation. |
| Output contract, exit-code catalog, `--json` plumbing | **W3** | W8 verbs consume W3's `Out` handle; W8 never hand-rolls JSON (closes the same `format!`-JSON defect class as B2). |
| `keyring`/`trust-root` discoverability, "did you mean", guide pages | **W4** | W8 owns the keyring *model*; W4 surfaces it in onboarding. |

### 1.4 Non-negotiable invariants

- **I1 — Sealed construction.** Signatures **wrap** a finalized `Receipt`; they never bypass `ChainAssembler::finalize` (`src/chain.rs:135-137`) or reconstruct a `Receipt` via the private `_seal` field (`src/types.rs:93-104`). A `SignedReceipt` *contains* a `Receipt`; it is not a parallel receipt type.
- **I2 — Integrity precedes authenticity.** A signature is **always** computed over the receipt's existing `chain_hash` bytes (the rolling BLAKE3, `src/chain.rs:53-60`). Signature verification re-runs `recompute_chain` first (mirroring `src/types.rs:127-134` and `src/1000x_post_quantum_sealing.rs:135-141`); a valid signature over a broken chain is still a REJECT.
- **I3 — Certify, don't decide.** No W8 surface mints a verdict of "honest." A signature answers *"these bytes, this author, unaltered"* and stops there. `attest`/`sbom-attest` predicates state *structural* facts (`src/sbom_supply_chain.rs:18-26`), never quality judgments.
- **I4 — Determinism.** Signed envelopes serialize via `canonical_bytes` (sorted-key JSON, `src/types.rs:545-549`); the **signed message is canonical bytes**, so the same logical receipt + key yields a reproducible signing input (signature itself may be randomized per-algorithm, but the *message* is fixed).
- **I5 — Crypto-agility.** Every signature is tagged with an algorithm id and key id; nothing assumes a single suite. This is what makes the PQC migration (§3.4) non-breaking.

---

## 2. Current state (grounded) + gap

### 2.1 What exists today

**The verbs are thin shims → one big handler module.** Every W8 verb is an auto-generated `#[verb(...)]` wrapper that delegates to `crate::handlers::*`:

- `src/verbs/sign.rs:13-21` → `handlers::sign(receipt, key_path, out, format)`
- `src/verbs/attest.rs:13-21` → `handlers::attest(...)`
- `src/verbs/notarize.rs:13-16` → `handlers::notarize(...)`
- `src/verbs/assemble_with_signature.rs:13-20` → `handlers::assemble_with_signature(...)`
- `src/verbs/assemble_and_notarize.rs:13-20` → `handlers::assemble_and_notarize(...)`
- `src/verbs/sbom_attest.rs:13-20` → `handlers::sbom_attest(...)`

**What each handler actually does today:**

| Verb | Handler | Reality | Cite |
|---|---|---|---|
| `sign` | `handlers::sign` | Reads `key_path` as a **string only** — never opens/uses it. Emits a JSON blob `{"signature":{"algorithm":"ed25519","status":"signed","note":"Production: sign chain_hash bytes with key at key_path."}}`. **No bytes are signed.** | `src/handlers.rs:641-673` |
| `notarize` | `handlers::notarize` | Emits `{"notarization":{"type":"rfc3161","status":"timestamp_token_attached","note":"Production: submit chain_hash to a TSA…"}}`. **No TSA call, no token.** | `src/handlers.rs:611-638` |
| `attest` | `handlers::attest` | **Most real of the three:** builds a genuine SLSA/in-toto-shaped statement (`_type`, `subject[].digest.blake3`, `predicateType`, `predicate.materials[]` mapped from event commitments). But it is **unsigned** — a plaintext envelope. | `src/handlers.rs:568-609` |
| `assemble-with-signature` | `handlers::assemble_with_signature` | Calls real `cli::assemble`, then echoes `signing_method` (default `"sigstore"`) into a hand-built JSON string `{…"signed":true}`. **No signing; method ignored beyond echo.** Also a `format!`-built-JSON injection vector (defect class B2). | `src/handlers.rs:295-313` |
| `assemble-and-notarize` | `handlers::assemble_and_notarize` | Same shape; echoes `notary_provider` (default `"rfc3161"`), `{…"notarized":true}`. **No notarization.** | `src/handlers.rs:316-334` |
| `sbom-attest` | `handlers::sbom_attest` | **The reference for "done right":** builds a structural provenance attestation via `sbom_supply_chain::attest_provenance`, then **appends it into the chain as an OCEL event** (`emit_with_payload("sbom:attest", …)`). Real, deterministic, in-chain — but **unsigned**. | `src/handlers.rs:3824-3854`; `src/sbom_supply_chain.rs:18-26` |

**A PQC module already exists — but it is all mocks and is not feature-gated.** `src/1000x_post_quantum_sealing.rs` (re-exported as `pqc_sealing`, `src/lib.rs:120-121`) defines a fully-shaped hybrid design:

- `PqcSeal { signature: DilithiumSignature, ciphertext: KyberCiphertext, key_id }` (`:42-51`)
- `PqcReceipt { base: Receipt, pqc_seal: PqcSeal }` — correctly **wraps** a sealed `Receipt` (respects I1) (`:53-60`)
- `QuantumResistantAssembler` whose `finalize()` signs `BLAKE3(receipt) || KyberCiphertext` (`:99-127`) — message construction respects I2
- `verify_pqc_receipt` re-runs `recompute_chain` before checking the signature (`:131-152`) — respects I2

…but the crypto is `mock_dilithium_sign` = `blake3("DILITHIUM-SIGN-MOCK" || msg)` (`:157-164`), `mock_kyber_encapsulate` returns the literal `b"MOCK-KYBER-CIPHERTEXT"` (`:179-184`). The doc-comment even flags it: *"In a production environment, this would use a crate like `pqc_dilithium`/`pqc_kyber`"* (`:25,32`). Critically, **this module is NOT behind `#[cfg(feature = "pqc")]`** — it compiles unconditionally, and the `pqc` feature in `Cargo.toml:167` is **empty** (`pqc = []`), so the flag toggles nothing.

**The chain primitives W8 builds on are solid:**

- `chain_hash` rolling BLAKE3 with genesis seed (`src/chain.rs:47-74`) — the canonical bytes to sign.
- `Receipt::sealed` is `pub(crate)` behind a private `_seal` (`src/types.rs:93-104`) — external construction is `E0451`.
- Deserialize re-verifies the chain (`src/types.rs:110-144`) — the non-forgeable-carrier precedent W8 extends to signatures.
- `canonical_bytes` (`src/types.rs:545-549`) — the deterministic signing input.

### 2.2 The gap

| # | Gap | Consequence |
|---|---|---|
| G1 | **No real signatures anywhere.** `sign` never touches the key; `assemble-with-signature` ignores its method. | The headline trust feature is a string literal. A "signed" receipt is indistinguishable from an unsigned one. |
| G2 | **No `SignedReceipt` type, no envelope, no verify path.** Signatures (when they appear) are loose JSON blobs disjoint from the receipt and from `verify`. | Nothing carries authorship through the type system; the 7-stage verifier (`src/verifier.rs`) has no signature stage. |
| G3 | **No key management.** No keygen, no keyring, no trust roots, no rotation, no revocation. `key_path` is an opaque unread string. | Can't answer "is this signer trusted?" — only W8 can supply that primitive for W7/W10. |
| G4 | **PQC is mock-only and ungated.** `mock_*` BLAKE3 stands in for Dilithium/Kyber; `pqc` feature is empty; module compiles always. | "100-year provenance security" (`:5`) is currently a BLAKE3 keyed hash — *not* quantum-resistant and not even gated. |
| G5 | **No transparency log.** `notarize` emits a fake RFC 3161 token; no append-only log, no inclusion/consistency proofs, no witnesses. | No third party can prove a receipt existed at a time; no detection of a forked/equivocating log. |
| G6 | **Attestations are unsigned & off-chain (except `sbom-attest`).** `attest` builds a real in-toto statement but signs nothing; `sbom-attest` is in-chain but unsigned. | Supply-chain provenance is *claimed*, not *attested* in the cryptographic sense. |
| G7 | **Hand-built JSON in the W8 path** (`assemble_with_signature`, `assemble_and_notarize`) — `format!` interpolation. | Injection / invalid-JSON risk (the project's own B2 defect); must move to W3's `Out`/serde. |

---

## 3. Phased plan (2026 H2 → 2030)

Sizing: **S** ≤ ½ day · **M** ~1–2 days · **L** ~3–5 days · **XL** ~1–2 wk (estimates, unverified against a real build). Each phase ends ACCEPT-clean against the existing verifier; no phase weakens any invariant in §1.4.

### Phase 0 — 2026 H2 · "Real Ed25519, a SignedReceipt envelope, a keyring"

**Objective.** Replace the signing stubs with genuine Ed25519 over `chain_hash`, behind a single sealed-respecting `SignedReceipt` envelope, with a minimal but real keyring and trust-root model. This is the foundation everything else hangs off.

**Deliverables.**
- New crate dep (gated): `ed25519-dalek` under a `sign` feature; `sign` joins the `all` aggregate. `hex` (already optional, `Cargo.toml:88`) promoted to a `sign` dep.
- `src/trust/mod.rs` — `Signer`/`Verifier` traits, `SignatureSuite` enum, `KeyId`, `SignedReceipt` envelope, `SigningError`.
- `src/trust/keyring.rs` — `Keyring` (generate / load / save), encrypted-at-rest secret keys (passphrase-wrapped), `TrustRoot` set of accepted verification keys.
- Rewrite `handlers::sign` to actually read `key_path`, sign `receipt.chain_hash` canonical bytes, emit a `SignedReceipt`. Rewrite `assemble_with_signature` to assemble-then-sign in one call (`signing_method` = `ed25519` | `sigstore-keyless` (deferred to P2)).
- New verb shim `keyring` (`generate` / `list` / `export-pub` / `trust-add`) following the `#[verb]` pattern (`src/verbs/sign.rs:13` shape).
- W7 contract: a `SignatureCheck` verify-stage *spec* (stage 8, optional) so W7 can wire it into the pipeline; W8 ships the pure `verify_signature(&SignedReceipt, &TrustRoot) -> SignatureOutcome` function.

**Compilable-style sketch — the core trait + envelope:**

```rust
//! src/trust/mod.rs — W8 cryptographic trust layer.
//! Doctrine: a signature attests authorship + integrity, never honesty.

use serde::{Deserialize, Serialize};
use crate::types::{Blake3Hash, Receipt, canonical_bytes};
use crate::chain::recompute_chain;

/// Stable identifier for a signature algorithm. Crypto-agility (I5):
/// every signature self-describes its suite so verifiers never assume one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureSuite {
    Ed25519,
    /// Hybrid: Ed25519 + ML-DSA-65 concatenated (lands Phase 2, §3.4).
    Ed25519MlDsa65,
    /// Pure post-quantum ML-DSA-65 (Dilithium). Phase 3+.
    MlDsa65,
}

/// A key fingerprint: blake3 of the public-key bytes, hex. Reuses the
/// project's content-addressing convention (Blake3Hash, src/types.rs:27).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KeyId(pub String);

impl KeyId {
    pub fn of_public_key(pubkey: &[u8]) -> Self {
        KeyId(Blake3Hash::from_bytes(pubkey).as_hex().to_string())
    }
}

/// A detached signature over a receipt's canonical chain-hash bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub suite: SignatureSuite,
    pub key_id: KeyId,
    /// Raw signature bytes, hex-encoded for canonical, diffable JSON.
    pub sig_hex: String,
    /// What role this signer plays: who assembled vs. who witnessed.
    pub role: SignerRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignerRole {
    /// Asserts "I assembled these bytes" (authorship).
    Author,
    /// Asserts "I observed these bytes at log time" (witnessing).
    Witness,
}

/// A finalized Receipt wrapped with one or more signatures.
///
/// INVARIANT I1: this *contains* a sealed `Receipt`; it never reconstructs one.
/// The receipt arrives only from `ChainAssembler::finalize` (src/chain.rs:135).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedReceipt {
    /// The sealed receipt, unchanged. Its `chain_hash` is the signing message.
    pub receipt: Receipt,
    /// Author/witness signatures, in deterministic (key_id, role) order.
    pub signatures: Vec<Signature>,
}

#[derive(Debug, thiserror::Error)]
pub enum SigningError {
    #[error("signing failed: {0}")] Sign(String),
    #[error("signature invalid: {0}")] Verify(String),
    #[error("chain integrity failed before signature check: {0}")] Chain(String),
    #[error("no trusted key for signature {0:?}")] Untrusted(KeyId),
    #[error("key error: {0}")] Key(String),
}

/// The message every signature commits to: the receipt's chain_hash bytes.
/// INVARIANT I2/I4: integrity-first, deterministic, canonical.
pub fn signing_message(receipt: &Receipt) -> Result<Vec<u8>, SigningError> {
    // Bind the signature to the rolling BLAKE3 head, hex-encoded (matches the
    // chain-fold convention in src/chain.rs:53-60 and pqc_sealing :109-112).
    let _ = canonical_bytes(receipt) // touch to assert serializability
        .map_err(|e| SigningError::Sign(e.to_string()))?;
    Ok(receipt.chain_hash.as_hex().as_bytes().to_vec())
}

/// Produces a signature over a finalized receipt. One impl per suite.
pub trait Signer {
    fn suite(&self) -> SignatureSuite;
    fn key_id(&self) -> KeyId;
    fn sign(&self, receipt: &Receipt, role: SignerRole)
        -> Result<Signature, SigningError>;
}

/// Verifies a signature against a known verification key.
pub trait Verifier {
    fn suite(&self) -> SignatureSuite;
    fn verify(&self, receipt: &Receipt, sig: &Signature)
        -> Result<(), SigningError>;
}

/// Outcome shape consumed by W7's optional SignatureCheck verify-stage.
/// (W8 owns the primitive; W7 owns scheduling it in the 7→8 stage pipeline.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureOutcome {
    pub checked: usize,
    pub valid: usize,
    pub untrusted: Vec<KeyId>,
    /// True iff chain integrity held AND >=1 trusted Author signature verified.
    pub authored_by_trusted: bool,
}

/// Top-level verify: integrity FIRST (I2), then signatures, against a trust root.
pub fn verify_signed(
    signed: &SignedReceipt,
    roots: &TrustRoot,
) -> Result<SignatureOutcome, SigningError> {
    // I2: a valid signature over a broken chain is still a REJECT.
    let recomputed = recompute_chain(&signed.receipt.events)
        .map_err(|e| SigningError::Chain(e.to_string()))?;
    if recomputed != signed.receipt.chain_hash {
        return Err(SigningError::Chain("chain hash mismatch".into()));
    }
    let mut out = SignatureOutcome {
        checked: 0, valid: 0, untrusted: vec![], authored_by_trusted: false,
    };
    for sig in &signed.signatures {
        out.checked += 1;
        match roots.verifier_for(&sig.key_id) {
            None => out.untrusted.push(sig.key_id.clone()),
            Some(v) => {
                v.verify(&signed.receipt, sig)?;
                out.valid += 1;
                if matches!(sig.role, SignerRole::Author) {
                    out.authored_by_trusted = true;
                }
            }
        }
    }
    Ok(out)
}

/// The set of verification keys a verifier is willing to accept (the anchor).
pub struct TrustRoot { /* key_id -> boxed Verifier; loaded from keyring */ }
impl TrustRoot {
    pub fn verifier_for(&self, _id: &KeyId) -> Option<&dyn Verifier> { todo!() }
}
```

**Acceptance criteria.**
- `affi receipt sign r.json --key-path k.sk` produces a `SignedReceipt` whose Ed25519 signature **actually verifies** with the matching public key, and **fails** if any event byte changes (regression-tests the I2 path, mirroring `chain_tamper_changes_hash`, `src/chain.rs:249-261`).
- `affi keyring generate` writes a passphrase-encrypted secret key + a public key whose `KeyId` is its BLAKE3.
- `verify_signed` returns `Err(Chain…)` on a tampered receipt **before** reporting signature status (proves I2 ordering).
- Round-trip: `SignedReceipt` → canonical JSON → back is byte-identical (I4), and the inner `Receipt` still re-verifies its chain on deserialize (`src/types.rs:110-144` unchanged).
- No `Receipt` is ever constructed except via `finalize` (I1 holds — grep shows zero new `Receipt { … }` literals).

**Cross-workstream deps.** W3 (consume `Out` for output, kills G7/B2 in this path) · W7 (receives the `SignatureCheck` stage-8 spec + `verify_signed`).

---

### Phase 1 — 2027 · "Transparency log, real notarization, signed attestations"

**Objective.** Stand up an append-only Merkle transparency log of signed-receipt heads with inclusion + consistency proofs and real RFC 3161 timestamping; sign the `attest`/`sbom-attest` provenance statements.

**Deliverables.**
- `src/trust/translog.rs` — `LogEntry`, `MerkleLog` (RFC 9162-style), `InclusionProof`, `ConsistencyProof`, `Checkpoint` (signed tree head / STH).
- Real `notarize`: submit `chain_hash` to a configurable TSA (RFC 3161) **and** append a `LogEntry` to the local/remote transparency log; emit the inclusion proof + timestamp token alongside the `SignedReceipt`.
- `assemble-and-notarize`: assemble → sign → log → return inclusion proof in one shot (replacing the echo stub, `src/handlers.rs:316-334`).
- Signed attestations: wrap the in-toto statement (`src/handlers.rs:577-594`) in a DSSE-style envelope and sign with the keyring; `sbom-attest`'s in-chain event (`src/handlers.rs:3835`) gains a signature object so the provenance fact is cryptographically attributable.
- W7 contract: an `InclusionCheck` verify-stage spec — given a `SignedReceipt` + `InclusionProof` + a trusted `Checkpoint`, W8 supplies `verify_inclusion(...)`; W7 schedules/distributes it.

**Compilable-style sketch — transparency-log entry + inclusion proof:**

```rust
//! src/trust/translog.rs — append-only Merkle log of signed-receipt heads.
//! Doctrine: the log proves *existence & order*, never *honesty*.

use serde::{Deserialize, Serialize};
use crate::types::Blake3Hash;
use crate::trust::{SignedReceipt, Signature, signing_message};

/// One leaf in the transparency log: a commitment to a signed receipt head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Zero-based position of this leaf in the log (monotone, like event `seq`).
    pub index: u64,
    /// Leaf hash = blake3(0x00 || chain_hash_bytes || author_key_ids).
    /// Domain-separated (RFC 9162 §2.1) so leaves never collide with nodes.
    pub leaf_hash: Blake3Hash,
    /// The chain head this leaf commits to (the receipt's chain_hash).
    pub receipt_head: Blake3Hash,
    /// Optional RFC 3161 timestamp token bytes (hex), if a TSA was used.
    pub timestamp_token_hex: Option<String>,
}

impl LogEntry {
    /// Domain-separated leaf hashing (0x00 prefix distinguishes leaf vs. node).
    pub fn compute_leaf(index: u64, signed: &SignedReceipt) -> LogEntry {
        let head = signed.receipt.chain_hash.clone();
        let mut buf = vec![0x00u8];
        buf.extend_from_slice(head.as_hex().as_bytes());
        for s in &signed.signatures {
            buf.extend_from_slice(s.key_id.0.as_bytes());
        }
        LogEntry {
            index,
            leaf_hash: Blake3Hash::from_bytes(&buf),
            receipt_head: head,
            timestamp_token_hex: None,
        }
    }
}

/// A signed tree head (checkpoint / STH): the log's commitment at size `n`.
/// Witnesses (third parties) co-sign this to defeat log equivocation (I3:
/// it certifies the log's state, it does not judge the receipts within).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub tree_size: u64,
    pub root_hash: Blake3Hash,
    /// Log operator + witness co-signatures over (tree_size || root_hash).
    pub signatures: Vec<Signature>,
}

/// Merkle audit path proving leaf `index` is in the tree of size `tree_size`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_index: u64,
    pub tree_size: u64,
    /// Sibling hashes bottom-up; replayed against the leaf to reach the root.
    pub audit_path: Vec<Blake3Hash>,
    pub root_hash: Blake3Hash,
}

/// Internal Merkle node hashing, domain-separated with 0x01 (RFC 9162 §2.1).
fn node_hash(left: &Blake3Hash, right: &Blake3Hash) -> Blake3Hash {
    let mut buf = vec![0x01u8];
    buf.extend_from_slice(left.as_hex().as_bytes());
    buf.extend_from_slice(right.as_hex().as_bytes());
    Blake3Hash::from_bytes(&buf)
}

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("inclusion proof failed: {0}")] Inclusion(String),
    #[error("checkpoint signature untrusted")] UntrustedCheckpoint,
}

/// Replay the audit path; the receipt is included iff it reproduces root_hash.
/// W8 provides this primitive; W7's distributed verifier *schedules* it.
pub fn verify_inclusion(
    entry: &LogEntry,
    proof: &InclusionProof,
    trusted_root: &Blake3Hash,
) -> Result<(), LogError> {
    if &proof.root_hash != trusted_root {
        return Err(LogError::Inclusion("root mismatch vs trusted checkpoint".into()));
    }
    let mut idx = proof.leaf_index;
    let mut acc = entry.leaf_hash.clone();
    for sibling in &proof.audit_path {
        acc = if idx % 2 == 0 { node_hash(&acc, sibling) }
              else            { node_hash(sibling, &acc) };
        idx /= 2;
    }
    if acc == proof.root_hash { Ok(()) }
    else { Err(LogError::Inclusion("recomputed root != proof root".into())) }
}
```

**Acceptance criteria.**
- A receipt notarized via `assemble-and-notarize` yields an `InclusionProof` that `verify_inclusion` accepts against the published `Checkpoint`, and that **fails** if the leaf or any sibling is altered.
- A `Checkpoint` carries ≥1 log-operator signature and (when configured) ≥1 witness co-signature; `verify_signed` over the checkpoint rejects an untrusted witness.
- `attest`/`sbom-attest` outputs are DSSE-signed; the signature verifies against the keyring and the `subject.digest.blake3` still equals the receipt `chain_hash` (`src/handlers.rs:581`).
- Determinism: same receipt set + same append order ⇒ byte-identical root hash (mirrors `chain_is_deterministic`, `src/chain.rs:239-246`).

**Cross-workstream deps.** W7 (consumes `verify_inclusion` / `verify_signed` checkpoint primitives; owns distribution) · W9 (supplies canonical `Sbom` for `sbom-attest`) · W10 (consumes signed attestations + proofs as governance evidence) · W3 (output).

---

### Phase 2 — 2028 · "PQC lands: real ML-DSA/ML-KEM, hybrid signatures, migration path"

**Objective.** **This is when PQC becomes real.** Replace the `mock_*` placeholders in `src/1000x_post_quantum_sealing.rs` with FIPS-standardized ML-DSA (Dilithium) and ML-KEM (Kyber), **gate them behind the now-meaningful `pqc` feature** (`Cargo.toml:167`), and ship the **hybrid** `Ed25519MlDsa65` suite as the default for new signatures — with a migration story that strands no existing receipt.

**Deliverables.**
- `pqc` feature populated: add `ml-dsa` + `ml-kem` (or `pqcrypto-dilithium`/`pqcrypto-kyber`, or `fips204`/`fips203`) deps, all `optional = true`, pulled in only by `pqc` (and transitively by `all`). `rand` (already optional, `Cargo.toml:91`) promoted to a `pqc`/`sign` dep for keygen.
- **Gate the existing module:** move `src/1000x_post_quantum_sealing.rs` behind `#[cfg(feature = "pqc")]` at the `pub mod pqc_sealing` site (`src/lib.rs:120-121`) — closes G4's "compiles always" half. (We do not *modify* it during design; this is the Phase-2 implementation step.)
- Replace `mock_dilithium_sign`/`mock_dilithium_verify`/`mock_kyber_encapsulate` (`:157-184`) with real calls; keep the **exact** message-construction (`BLAKE3(receipt) || ciphertext`, `:108-112`) so PQ-SEAL-v1 semantics and tests (`:205-240`) carry over.
- A real `MlDsaSigner`/`MlDsaVerifier` implementing the §3.1 `Signer`/`Verifier` traits, plus a `HybridSigner` that emits **two** signatures (Ed25519 + ML-DSA) under one `Ed25519MlDsa65`-tagged envelope entry.
- **Migration path doc + tooling** (see §3.4) — `affi keyring migrate` (issue PQC keys, re-sign existing `SignedReceipt`s with a hybrid co-signature without re-finalizing the chain), and a verifier policy knob `--require {classical|hybrid|pqc}`.

**Compilable-style sketch — hybrid signer + suite-dispatched verify:**

```rust
//! src/trust/pqc.rs (feature = "pqc") — hybrid classical+PQ signatures.
//! Lands 2028. Reuses the PqcSeal message rule from
//! src/1000x_post_quantum_sealing.rs:108-112 (BLAKE3 head || ciphertext).

use crate::trust::{Signer, Verifier, Signature, SignatureSuite,
                   SignerRole, KeyId, SigningError, signing_message};
use crate::types::Receipt;

/// Emits Ed25519 AND ML-DSA-65 over the same message, tagged hybrid.
/// Migration property: a hybrid signature verifies under a *classical-only*
/// policy (Ed25519 half) AND a *pqc* policy (ML-DSA half) — so flipping the
/// required level never invalidates already-issued hybrids.
pub struct HybridSigner {
    classical: Box<dyn Signer>, // Ed25519 (Phase 0)
    pq: Box<dyn Signer>,        // ML-DSA-65 (Phase 2)
}

impl Signer for HybridSigner {
    fn suite(&self) -> SignatureSuite { SignatureSuite::Ed25519MlDsa65 }
    fn key_id(&self) -> KeyId { self.classical.key_id() } // anchor on classical id
    fn sign(&self, receipt: &Receipt, role: SignerRole)
        -> Result<Signature, SigningError>
    {
        let _msg = signing_message(receipt)?; // identical message for both halves
        let c = self.classical.sign(receipt, role)?;
        let q = self.pq.sign(receipt, role)?;
        // Concatenate hex halves with a separator the verifier splits on.
        Ok(Signature {
            suite: SignatureSuite::Ed25519MlDsa65,
            key_id: self.classical.key_id(),
            sig_hex: format!("{}:{}", c.sig_hex, q.sig_hex),
            role,
        })
    }
}

/// Policy for how strong a signature a verifier insists on (migration knob).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredAssurance { Classical, Hybrid, Pqc }

/// Suite-aware verification gate. Old receipts (Ed25519) still pass under
/// `Classical`; the fleet can ratchet to `Hybrid` then `Pqc` over years
/// WITHOUT a flag-day, because hybrids satisfy all three.
pub fn verify_with_policy(
    receipt: &Receipt,
    sig: &Signature,
    classical: &dyn Verifier,
    pq: &dyn Verifier,
    need: RequiredAssurance,
) -> Result<(), SigningError> {
    match (sig.suite, need) {
        (SignatureSuite::Ed25519, RequiredAssurance::Classical) =>
            classical.verify(receipt, sig),
        (SignatureSuite::Ed25519, RequiredAssurance::Hybrid | RequiredAssurance::Pqc) =>
            Err(SigningError::Verify("classical-only sig below required assurance".into())),
        (SignatureSuite::MlDsa65, _) => pq.verify(receipt, sig),
        (SignatureSuite::Ed25519MlDsa65, _) => {
            // Split "c_hex:q_hex"; verify the half(s) the policy demands.
            let (c_hex, q_hex) = sig.sig_hex.split_once(':')
                .ok_or_else(|| SigningError::Verify("malformed hybrid sig".into()))?;
            let cs = Signature { suite: SignatureSuite::Ed25519,
                                 sig_hex: c_hex.into(), ..sig.clone() };
            let qs = Signature { suite: SignatureSuite::MlDsa65,
                                 sig_hex: q_hex.into(), ..sig.clone() };
            match need {
                RequiredAssurance::Classical => classical.verify(receipt, &cs),
                RequiredAssurance::Pqc       => pq.verify(receipt, &qs),
                RequiredAssurance::Hybrid    => {
                    classical.verify(receipt, &cs)?; pq.verify(receipt, &qs)
                }
            }
        }
    }
}
```

**Acceptance criteria.**
- With `--features pqc`, `QuantumResistantAssembler::finalize` uses **real ML-DSA/ML-KEM** (no `mock_*` symbol reachable); existing tests `test_quantum_resistant_flow` / `test_tamper_detection` (`src/1000x_post_quantum_sealing.rs:205-240`) still pass against the real backend.
- Without `--features pqc`, the `pqc_sealing` module does **not** compile in (the `cfg` gate proves the feature now means something — G4 closed).
- A receipt signed `Ed25519MlDsa65` verifies under `Classical`, `Hybrid`, **and** `Pqc` policies; an old `Ed25519`-only receipt verifies under `Classical` and is cleanly **rejected** (not crashed) under `Pqc`.
- `affi keyring migrate` re-signs an existing `SignedReceipt` with a hybrid co-signature **without** touching `receipt.chain_hash` (I1/I2 hold — the inner sealed receipt is byte-identical before/after).

**Cross-workstream deps.** W7 (verify policy threading into distributed verification; suite-aware proof primitives) · W10 (records crypto-assurance level as governance metadata) · W1 (feature-matrix / CI: a `pqc` build-and-test lane).

---

### Phase 3 — 2029 · "Trust at scale: rotation, revocation, federated logs, witness gossip"

**Objective.** Make the trust layer operable for a fleet: key rotation/revocation with transparency, multi-witness gossip to defeat split-view attacks, and federation across multiple transparency logs.

**Deliverables.**
- **Key lifecycle as logged events:** key issuance, rotation, and revocation are themselves `OperationEvent`s appended into a dedicated *trust receipt* (dog-fooding the chain — a key's history is itself a verifiable provenance chain). Revocation = a signed, logged tombstone; `TrustRoot` consults it.
- **Witness gossip:** witnesses exchange `Checkpoint`s; a `verify_consistency(old_cp, new_cp, proof)` primitive (RFC 9162 consistency proof) proves the log is append-only and non-equivocating across witnesses.
- **Federation:** a `SignedReceipt` may carry inclusion proofs from *multiple* logs; verifier policy `--quorum N` requires N independent log inclusions (resilience if one log is compromised).
- **Threshold / multi-author signing:** k-of-n author signatures for high-assurance receipts (e.g., release artifacts).
- W7 contract: `verify_consistency` + multi-log quorum evaluation primitives.

**Compilable-style sketch — consistency + quorum (signature shown; bodies elided):**

```rust
//! src/trust/federation.rs (2029) — multi-log quorum + append-only proofs.

use crate::trust::translog::{Checkpoint, InclusionProof, LogEntry, LogError, verify_inclusion};
use crate::types::Blake3Hash;

/// Proof that a log of size m is a prefix of the same log at size n (m <= n).
/// Replayed to show no leaf was ever rewritten (RFC 9162 §2.1.2). Defeats a
/// log that tries to fork history between two observers.
pub fn verify_consistency(
    _old: &Checkpoint, _new: &Checkpoint, _proof: &[Blake3Hash],
) -> Result<(), LogError> { /* replay old root from new tree */ Ok(()) }

/// A receipt's membership evidence across independent transparency logs.
pub struct FederatedInclusion {
    pub per_log: Vec<(LogEntry, InclusionProof, /*trusted root*/ Blake3Hash)>,
}

/// Accept iff the receipt is independently included in >= `quorum` logs.
/// Certifies "this existed, witnessed by >=N independent logs" — not honesty.
pub fn verify_quorum(fi: &FederatedInclusion, quorum: usize) -> Result<(), LogError> {
    let ok = fi.per_log.iter()
        .filter(|(e, p, root)| verify_inclusion(e, p, root).is_ok())
        .count();
    if ok >= quorum { Ok(()) }
    else { Err(LogError::Inclusion(format!("quorum not met: {ok}/{quorum}"))) }
}
```

**Acceptance criteria.**
- A rotated key's old signatures still verify (historical receipts stay valid); a revoked key's signatures are flagged `untrusted` post-revocation but the receipt's *integrity* verdict is unaffected (I3: revocation ≠ "dishonest").
- `verify_consistency` rejects a log that rewrote any historical leaf.
- `--quorum 2` rejects a receipt present in only one of two required logs; accepts when present in both.
- k-of-n: a receipt with k-1 valid author signatures fails the threshold; k passes.

**Cross-workstream deps.** W7 (quorum/consistency scheduling across shards) · W10 (revocation + rotation as compliance-relevant lifecycle events) · W4 (operator UX for keyring/witness config).

---

### Phase 4 — 2030 · "Default-secure, PQC-primary, hardware-anchored, audited"

**Objective.** PQC-primary by default, hardware-rooted keys, and an independently auditable trust layer — the steady state.

**Deliverables.**
- **PQC-primary default:** new keys default to hybrid; verifier fleet default policy ratchets to `Hybrid` (with `Pqc`-only available for high-assurance domains). Classical-only signing is deprecated to opt-in legacy.
- **Hardware/HSM & remote signing:** `Signer` impls backed by PKCS#11 / cloud KMS / TPM so author secret keys need never touch disk; keyless (Sigstore/Fulcio-style ephemeral-key + OIDC) becomes a first-class `Signer`.
- **Formal-ish assurance:** a published threat model + an external crypto review of the suite implementations and the log; property tests (quickcheck, already a dev-dep, `Cargo.toml:111`) for sign/verify/inclusion/consistency round-trips.
- **Crypto-agility headroom:** a second PQ suite slot (e.g., SLH-DSA / SPHINCS+ as a hash-based fallback independent of lattice assumptions) wired through the `SignatureSuite` enum — proving the agility design (I5) holds.

**Acceptance criteria @2030.** See §4.

**Cross-workstream deps.** W7 (PQC-primary distributed verification at scale) · W10 (audit evidence, assurance-level reporting) · W1 (release/CI hardening, reproducible PQC builds) · W9 (signed SBOM provenance is the default supply-chain artifact).

---

## 4. Definition of done @2030

A receipt's trust story is **complete, default-secure, and quantum-ready** when all hold:

1. **Authorship is real and typed.** Every `affi receipt sign` / `assemble-with-signature` emits a `SignedReceipt` (§3.1) whose signature genuinely verifies and genuinely fails on a one-byte edit. The string-stub era (`src/handlers.rs:641-673`) is gone. (G1, G2 closed.)
2. **Keys are managed.** `affi keyring` covers generate / rotate / revoke / export-pub / trust-add, secret keys are encrypted-at-rest or HSM/KMS/keyless-anchored, and a `TrustRoot` answers "is this signer trusted?" deterministically. (G3 closed.)
3. **PQC is real, gated, and default-hybrid.** The `pqc` feature (`Cargo.toml:167`) pulls in standardized ML-DSA/ML-KEM; the `mock_*` functions (`src/1000x_post_quantum_sealing.rs:157-184`) are gone; new signatures are hybrid `Ed25519MlDsa65` by default; a hybrid receipt verifies under classical, hybrid, **and** pqc policies, so **no receipt ever issued was stranded by the migration**. (G4 closed; PQC shipped in **Phase 2 / 2028**.)
4. **Transparency is provable.** Every notarized receipt has a Merkle inclusion proof against a witness-co-signed checkpoint; consistency proofs prove the log append-only; `--quorum N` federates across independent logs. `notarize`'s fake token (`src/handlers.rs:611-638`) is gone. (G5 closed.)
5. **Provenance is attested, not just claimed.** `attest` and `sbom-attest` outputs are DSSE-signed, logged, and (for `sbom-attest`) in-chain, with `subject.digest.blake3` bound to the receipt head. (G6 closed.)
6. **Invariants intact.** Across all of the above: no `Receipt` is constructed outside `ChainAssembler::finalize` (I1); signature/inclusion verification always re-checks chain integrity first (I2); no surface ever emits a verdict of "honest" (I3); signed envelopes are canonical/deterministic (I4); every signature self-describes its suite (I5). No hand-built JSON remains in the W8 path (G7 closed; W3 `Out` adopted).
7. **W7 is fully supplied.** W8 exposes stable primitives — `verify_signed`, `verify_inclusion`, `verify_consistency`, `verify_quorum`, `verify_with_policy` — that W7's distributed/Merkle verifier consumes without owning any crypto itself.

---

## 5. Cross-workstream dependencies (summary)

| Dep | Direction | What crosses the seam | Phase |
|---|---|---|---|
| **W7 — Verification Engine** | W8 → W7 | Signature & proof **primitives** (`verify_signed`, `verify_inclusion`, `verify_consistency`, `verify_quorum`, `verify_with_policy`) + the optional stage-8 `SignatureCheck`/`InclusionCheck` specs. W7 owns *mechanics, performance, Merkle distribution, scheduling*; W8 owns the *crypto*. | All |
| **W9 — Ecosystem & Standards** | W9 → W8 | Canonical `Sbom` (`src/sbom_supply_chain.rs`) that `sbom-attest` signs; in-toto/SLSA & RFC 9162 / RFC 3161 standard alignment. W8 signs/logs what W9 ingests. | P1, P2, P4 |
| **W10 — Compliance & Governance** | W8 → W10 | Signed, logged attestations + inclusion/consistency proofs + crypto-assurance level become governance **evidence**; key rotation/revocation are compliance lifecycle events. W8 emits facts; W10 reports. | P1–P4 |
| **W3 — CLI Ergonomics** | W3 → W8 | The `Out` handle / exit-code catalog. W8 verbs consume it (kills the `format!`-JSON defect class, B2/G7, in `assemble_with_signature`/`assemble_and_notarize`). | P0+ |
| **W4 — Onboarding/Registry** | W4 → W8 | Surfaces `keyring`/`trust-root`/witness config in guide & registry; "did you mean" for the new verbs. W8 owns the *model*; W4 the *discoverability*. | P0, P3 |
| **W1 — Foundations** | W1 ↔ W8 | Feature-matrix & CI: a `--features pqc` build-and-test lane; reproducible PQC builds; the genesis-seed/version hygiene (`src/chain.rs:22`) that signed receipts also depend on. | P2, P4 |

---

### Appendix A — File map (grounding index, read-only)

| Path | Role in W8 |
|---|---|
| `src/chain.rs:47-74,135-137` | Rolling BLAKE3 + `ChainAssembler::finalize` — the bytes W8 signs; the sealed seam W8 must not bypass (I1/I2). |
| `src/types.rs:93-104,110-144,545-549` | `Receipt::sealed` (private `_seal`), deserialize chain-recheck (the non-forgeable-carrier precedent), `canonical_bytes` (deterministic signing input). |
| `src/handlers.rs:295-334` | `assemble_with_signature` / `assemble_and_notarize` stubs (echo-only, hand-built JSON — G1/G7). |
| `src/handlers.rs:568-609` | `attest` — real in-toto statement, **unsigned** (G6). |
| `src/handlers.rs:611-638` | `notarize` — fake RFC 3161 token (G5). |
| `src/handlers.rs:641-673` | `sign` — pure JSON stub, key never read (G1). |
| `src/handlers.rs:3824-3854` | `sbom-attest` — the "done right" reference: structural attestation appended in-chain, but unsigned (G6). |
| `src/1000x_post_quantum_sealing.rs` (`pub mod pqc_sealing`, `src/lib.rs:120-121`) | Full hybrid PQC *design* — `PqcSeal`/`PqcReceipt`/`QuantumResistantAssembler`/`verify_pqc_receipt` — but `mock_*` crypto (`:157-184`) and **not** `#[cfg(feature="pqc")]`-gated (G4). |
| `Cargo.toml:167` | `pqc = []` — empty feature; populated in Phase 2 / 2028. |
| `src/sbom_supply_chain.rs:18-26` | `attest_provenance` + the "certify, don't decide" doctrine note W8 inherits (I3). |
