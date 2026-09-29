# LANES — affidavit-trust-plane-pack wave 2 (2026-09-28)

Two canonical checkouts, disjoint file ownership, agents never run git.
Coordinator: single-writer for `ggen sync run` (both repos), git transitions,
Cargo.toml/lib.rs/ggen.toml/CI seams.

| Lane | Repo | Owned files (nothing else) |
|---|---|---|
| L1 | marketplace | `packs/affidavit-trust-plane-pack/{pack.toml, ontology.ttl, targets.toml, ggen.toml, package.toml, HANDWRITTEN.md}` |
| L2 | marketplace | `packs/affidavit-trust-plane-pack/{gates/*, gate-court.toml, witnesses/**, qualification/consumer.ttl}` |
| L3 | marketplace | `packs/affidavit-trust-plane-pack/{queries/canonical.rq, templates/crypto_trust_canonical.rs.tmpl}` |
| L4 | marketplace | `packs/affidavit-trust-plane-pack/{queries/keys.rq, templates/crypto_trust_keys.rs.tmpl}` |
| L5 | marketplace | `packs/affidavit-trust-plane-pack/{queries/lifecycle.rq, templates/crypto_trust_lifecycle.rs.tmpl}` |
| L6 | marketplace | `packs/affidavit-trust-plane-pack/{queries/envelope.rq, templates/crypto_trust_envelope.rs.tmpl}` |
| L7 | marketplace | `packs/affidavit-trust-plane-pack/{queries/es256.rq, queries/enclave.rq, templates/crypto_trust_es256.rs.tmpl, templates/crypto_trust_enclave.rs.tmpl}` |
| L8 | marketplace | `packs/affidavit-trust-plane-pack/{queries/pqc.rq, templates/crypto_trust_pqc.rs.tmpl}` |
| L9 | marketplace | `packs/affidavit-trust-plane-pack/{queries/verify.rq, templates/crypto_trust_verify.rs.tmpl}` |
| L10 | marketplace+affidavit | `packs/.../{queries/seal.rq, queries/e2e.rq, templates/crypto_trust_seal.rs.tmpl, templates/crypto_trust_e2e.rs.tmpl}`; affidavit `docs/CRYPTO_TRUST_PLANE.md`, `docs/INDEX.md` row, `HANDWRITTEN.md` |

Shared seams (coordinator-only): affidavit `Cargo.toml`, `src/lib.rs`,
`ggen.toml`, `.github/workflows/`, `src/1000x_post_quantum_sealing.rs` retirement,
clap-noun-verb `queries/*.rq` (fieldName COALESCE, already applied).
