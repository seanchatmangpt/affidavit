# affidavit 📜
### The Provenance Layer for High-Assurance Systems & The Adversarial Evidentiary Trust Plane

`affidavit` is a deterministic cryptographic provenance engine designed to make the unverifiable unconstructable. It assembles, seals, and certifies append-only, content-addressed BLAKE3 chains of operation-events that provide an immutable record of computational execution.

In v26.10.16, `affidavit` reifies the **Adversarial Legal Ratchet**: embedding post-quantum cryptographic primitives (FIPS 204 ML-DSA-65), statutory anchors (SOX § 1514A, FRCP 37(e), Constructive Trust Gain-Share), and liquid avoided-tariff accounting (Restatement of Restitution § 42) directly into a constant-time 352-byte binary token (CTRT).

---

## 🏛️ Doctrine: Certify, Don't Decide

In complex corporate and technical architectures, subjective honesty is undecidable. `affidavit` shifts the burden from detection to certification:

1. **Witness-Based Verification:** The verifier does not hunt for intent; it checks a witness (the receipt) against a formal mathematical standard.
2. **Decidable Pipeline:** Every stage of the 7-stage certify pipeline is decidable, yielding an atomic `ACCEPT` or `REJECT` verdict in constant time.
3. **Unconstructable Bypass:** Valid receipts cannot be fabricated. They must pass through canonical, sealed seams in silicon.
4. **Content-Addressed Integrity:** Every event is linked via a rolling BLAKE3 hash. A single bit flip or sequence excision invalidates the entire chain.
5. **Self-Authenticating Legal Standing (FRE 902(14)):** Certified event chains emit automated affidavits admissible in federal court without foundation testimony.
6. **Anti-Spoliation by Construction (FRCP 37(e)):** Persistent nullifier sequence gaps turn routine CI cache-wiping and log pruning into mathematical proofs of intentional spoliation.

---

## 🚀 The v26.10.16 Release: Core Invariants

- 🛡️ **352-Byte CTRT Token:** Fixed-size packed binary wire format encoding `statutory_anchor`, `tariff_basis_cents`, and `deontic_modality` in bytes `151..352`.
- 🔐 **FIPS 204/205 PQC signatures:** ML-DSA-65 and SLH-DSA-SHA2-128s (plus
  ML-KEM-768 key encapsulation) via Rust bindings over the external `ml-dsa`,
  `slh-dsa`, and `ml-kem` crates — zero hand-implemented cryptography anywhere
  in this project; KAT-validated through the external implementations. PQC
  ships unconditionally in `affidavit-core` and the WASM artifact (no PQC
  feature flag); in the root crate it is available under the `crypto-trust`
  feature.
- ⚡ **affidavit-wasm (660,847 bytes pinned; see `affidavit-wasm/registry/ARTIFACTS.sha256`):** Sandboxed WASI-only binary exporting the `af_*` JSON-ABI surface (`af_abi_version`/`af_alloc`/`af_free`/`af_call`) plus the `affi_admit_and_seal` gate — admit-and-seal targets sub-100µs in-process BEAM execution; null and oversize requests return typed errors instead of trapping.
- 🤖 **100 Canonical Verbs:** Governed by RDF ontology (`ontology/affi-cli.ttl`) with strict drift-parity enforcement across Bash, Zsh, Fish, PowerShell, and Nushell.
- 📜 **Statutory Verbs:**
  - `affi legal-hold status`: Verifies historical chain continuity and outputs FRCP 37(e) spoliation status.
  - `affi restitution calculate`: Computes liquidated avoided compute spend under Restatement of Restitution § 42.
  - `affi certificate fre-902`: Emits self-authenticating digital evidence certificates under FRE 902(14).

---

## 🛠️ Quick Start

### Build from Source
The toolchain is date-pinned in `rust-toolchain.toml`; rustup will fetch it automatically:

```bash
git clone https://github.com/seanchatmangpt/affidavit
cd affidavit
cargo build --release
```

Run the full verification ladder:

```bash
cargo test --test adversarial_wire_ctrt --test verifier_stages --test adversarial_nullifier --test completions_drift
```

### Run the Smoke Test

```bash
./examples/golden_run.sh
```

---

## 💻 Canonical CLI Surface (101 Verbs)

Affidavit ships 100 canonical verbs across 11 groups, backed by a compile-time static registry (`src/registry.rs`) held in byte-level agreement with `ontology/affi-cli.ttl`:

### Core Provenance Loop:
* `affi emit` — Record a new operation-event.
* `affi assemble` — Finalize and seal the current receipt.
* `affi verify` — Run the 7-stage certify pipeline against a receipt.
* `affi show` — Inspect receipt details and cryptographic commitments.

### Compliance & Statutory Governance:
* `affi legal-hold status` — Audit local OCEL/CTRT chains for pruning under FRCP 37(e).
* `affi restitution calculate` — Calculate liquidated avoided cloud compute spend.
* `affi certificate fre-902` — Generate self-authenticating evidence affidavit under FRE 902(14).
* `affi sbom compliance` — Run NTIA minimum-element compliance verification.

### Evidence Federation:
* `affi standing certify / verify` — Seal and re-check receipt-bound standing claims.
* `affi ecosystem certify / verify` — Federate member standing receipts against an ALIVE quorum.
* `affi errc certify / verify` — Seal formal ERRC transformations behind a preservation fence.

---

## 🛡️ The 7-Stage Certify Pipeline

Each receipt passes through a rigorous, branchless validation gauntlet:

1. **Decode:** Structural presence and parseable `format_version`. (CTRT-magic checking lives in the separate CTRT wire-format court, `src/wire_ctrt.rs`.)
2. **Format Check:** Verification against the `core/v1` schema standard.
3. **Chain Integrity:** Cryptographic re-computation of the rolling BLAKE3 hash.
4. **Continuity:** Sequence validation; missing sequence numbers trigger `REJECT: EVIDENTIARY_CHAIN_GAP_DETECTED (FRCP-37E)`.
5. **Commitment Verify:** Structural validation of all payload digests.
6. **Profile Evaluation:** Enforces deontic law, revenue tier thresholds, and unauthorized deletion checks (`REJECT: UNLAWFUL_RECORD_EXPUNCTION (ABA-3.4A)`).
7. **Final Verdict:** Atomic `ACCEPT` or `REJECT` output.

---

## 📄 License

Dual-licensed under MIT or Apache 2.0.
