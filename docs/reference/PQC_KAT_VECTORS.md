# Reference: Post-Quantum Known Answer Test (KAT) Vectors

**Authority:** FIPS 204 (ML-DSA), FIPS 205 (SLH-DSA), FIPS 203 (ML-KEM)  
**Test Suite:** [`tests/crypto_trust_kat_fixture.rs`](file:///Users/sac/affidavit/tests/crypto_trust_kat_fixture.rs)  

---

## 1. Cryptographic Algorithms & Parameters

Affidavit v26.10.17 implements pure-Rust post-quantum cryptographic primitives:

- **ML-DSA-65 (FIPS 204):** NIST Security Level 3 lattice-based digital signature algorithm. Public key length: 1,952 bytes; signature length: 3,309 bytes.
- **SLH-DSA-128s (FIPS 205):** Stateless hash-based digital signature algorithm.
- **ML-KEM-768 (FIPS 203):** Module lattice-based key encapsulation mechanism.
- **X-Wing:** Post-quantum hybrid key encapsulation combining X25519 and ML-KEM-768.

---

## 2. Invariance & Fixed-Point Verification

All KAT vectors are loaded from golden fixtures and verified against RFC 8785 JSON Canonicalization Scheme (JCS) bytes:
- Any 1-bit perturbation across public keys, seeds, or signatures results in deterministic, typed verification rejection.
- Zero floating-point drift or architecture-dependent discrepancies between native x86_64, aarch64, and wasm32-wasip1.
