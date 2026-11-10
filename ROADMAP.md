# affidavit — Project Roadmap

**Version:** 26.10.17
**Status:** living document; superseded sections live in git history (pre-2026-11-09 body).

## Current state (verified 2026-11-09)

- 100 canonical verbs in 11 groups; registry parity is enforced by
  `tests/docs_verb_parity.rs` (passes) against `src/registry.rs`.
- 7-stage certify pipeline, sealed receipts, determinism guarantees: see `README.md`.
- Evidence federation (`standing`, `ecosystem`, `errc`): `docs/FEDERATION.md`.
- Crypto trust plane (ML-DSA-65, SLH-DSA, hybrid): `docs/CRYPTO_TRUST_PLANE.md`.
- `affidavit-wasm` JSON-ABI verifier; pin in `affidavit-wasm/registry/ARTIFACTS.sha256`.
- Release history and receipts: `CHANGELOG.md`.

## Open items (from CHANGELOG 26.10.17)

- **BLS rogue-key soundness claim FALSIFIED** (capability register row 82): registered
  divergence, no fix landed.
- Capability register: `docs/reference/CAPABILITY_REGISTER.md`; CLI vacuity audit:
  `docs/reference/CLI_VACUITY_AUDIT_26_10_17.md` — remaining VACUOUS/UNVERIFIED cells are the
  backlog.
- Secure Enclave live KATs remain `#[ignore]`-gated (PARTIAL_ALIVE) pending an
  entitlement-signed host binary.

## See Also

`STATUS.md` · `CHANGELOG.md` · `docs/README.md`
