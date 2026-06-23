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

*   ⚡ **High-Performance:** Parallelized verification across multi-core architectures.
*   🔍 **Deep Introspection:** Auto-generate DFG/Petri models from receipts.
*   🛡️ **Chaos Engineering:** Built-in mutation testing to stress-test your verifiers.
*   🤖 **Intelligent CLI:** 65+ canonical verbs, ontology-driven help, and powerful ad-hoc querying.

---

## 🛠️ Installation & Quick Start

### Build from Source
Ensure you have the latest stable Rust toolchain installed.

```bash
git clone https://github.com/seanchatmangpt/affidavit
cd affidavit
cargo build --release --all-features
```

### The "Golden Run" in 30 Seconds
Run the end-to-end smoke test to see `affidavit` in action:

```bash
./examples/golden_run.sh
```

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

---

## 💻 CLI Surface

Affidavit v26.6.22 ships **69 canonical verbs** across 10 groups, backed by a compile-time static registry (`src/registry.rs`) that is the authoritative single source of truth for help, completions, and documentation.

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

**Health & Diagnostics:**
- `affi doctor` — Run environment and receipt-store health checks with structured exit codes.

*(Full list: `affi --help` or `affi shell`. Verb registry: `src/registry.rs`. Groups: Core · Diagnostics · Analysis · Ingestion · Compliance · Attestation · SBOM · Insights · Engineering · Tooling)*

---

## 🛡️ Security Model

`affidavit` is designed for high-stakes environments where provenance is non-negotiable:
- **Zero-Knowledge Payloads:** We store commitments, not raw data, protecting sensitive information.
- **Deterministic Hashing:** Canonical JSON serialization ensures hashes are stable across platforms.
- **Memory Safety:** Written in 100% `safe` Rust (enforced via `#![deny(unsafe_code)]`).

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to participate in the provenance revolution.

## 📄 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
