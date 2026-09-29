# RESOLUTIONS — trust-plane wave 2 (2026-09-28)

Pinned seams and interlock decisions. Lanes interlock on this text, not on each other.

1. **Wiring (proven live)**: consumer `ggen sync run` = DeclarativeRules manifest.
   Data binding = rule SELECT columns as plain Tera vars. Frontmatter `sparql:` is
   NOT evaluated on this path (reserved doc-only). Pack ontology reaches the
   consumer graph via `[ontology].imports`. Pack-local `ggen.toml` (project
   profile) renders `generated/` as the pack's own self-proof — coordinator runs
   it; lanes never run `ggen sync` (single-writer).
2. **Ontology IRIs** are fixed (see dispatch contract); L1 materializes exactly;
   every lane's SPARQL binds to those IRIs. Individuals, not literals, for
   algorithms/profiles/standings/field-orders.
3. **Module interlock**: no shared trait; `crypto_trust_verify` dispatches on
   `AlgorithmId` to concrete fns in `crypto_trust_es256` / `crypto_trust_enclave` /
   `crypto_trust_pqc`. Exact fn signatures pinned in the dispatch contract.
4. **Cargo seams** (coordinator, applied): optional deps `signature 2.2`,
   `p256 0.13.2`, `ml-dsa 0.1.1`, `slh-dsa =0.2.0-rc.5`, `rand_core 0.6(getrandom)`;
   `[target.'cfg(target_os = "macos")' ] security-framework 3.7` (optional);
   features `crypto-trust`, `secure-enclave`; `rust-version = "1.85"`.
   Lane-reported dep fixes go through the coordinator only.
5. **Verb-pipeline collision** (surfaced by ggen 26.9.28 FM-GEN-004): resolved by
   `cnv:fieldName` on the 9 colliding verbs + `output_file = {{ handler_name }}.rs`
   + pack COALESCE. Retires the legacy `generate_verbs.py` projection. Coordinator
   commit.
6. **Mock retirement**: `src/1000x_post_quantum_sealing.rs` is an admitted mock
   (substrate-map red flag). Coordinator retires it after grep-verification; the
   `PqcReceipt { base, pqc_seal }` wire shape is conserved as
   `SealedReceipt { base, envelope, signature }` (field `base` kept).
7. **Empty-file placeholders**: `src/crypto_trust_*.rs` + `tests/crypto_trust_e2e.rs`
   exist empty until first render (buildability, not stubs: no signatures, no
   claims). Integration re-renders everything.
8. **Standing honesty**: enclave live KAT is `#[ignore]`-gated (operator biometrics)
   → PARTIAL_ALIVE until witnessed. HSM/TPM/PKCS#11: typed UNSUPPORTED refusal, no
   fakes.
