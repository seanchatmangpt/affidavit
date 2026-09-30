# HANDWRITTEN.md — the 1% ledger

Law: hand-writing consumer code is permitted only when no admitted
pack/generator expresses the semantic element, the owning ontology carries an
`UNSUPPORTED(generator, element)` row, and the write is admitted here. This
ledger shrinks monotonically; growth requires a paydown plan in the same
change. No ledger entry = fabrication of the ratio.

Everything under `src/crypto_trust_*.rs` (the trust-plane modules themselves)
is **manufactured** — rendered by `ggen sync` from
`ggen-marketplace/packs/affidavit-trust-plane-pack`; it never appears here.
The rows below are the consumer-side wiring seams only: the glue that mounts
the pack's projections into this crate. No pack expresses the consumer-side
wiring of the trust-plane pack (a pack renders modules; it cannot own the
consumer's manifest, module tree, rule merge, or CLI verb table).

| path | semantic element | missing capability | intended owner pack | date |
|---|---|---|---|---|
| `Cargo.toml` | `[features] crypto-trust = ["dep:signature", "dep:p256", "dep:ml-dsa", "dep:slh-dsa", "dep:rand_core"]` + `secure-enclave` gate and crate dependencies | no pack expresses the consumer's Cargo manifest (dependency/feature wiring of rendered modules) | affidavit-trust-plane-pack (consumer-wiring extension) | 2026-09-28 |
| `src/lib.rs` | `#[cfg(feature = "crypto-trust")] pub mod crypto_trust_*` declarations (+ `secure-enclave` gate for the enclave module) | no pack expresses the consumer's module tree (cfg-gated `mod` decls mounting rendered files) | affidavit-trust-plane-pack (consumer-wiring extension) | 2026-09-28 |
| `ggen.toml` | `[ontology] imports` merge of the pack ontology + `[[generation.rules]]` table binding pack queries/templates to `src/crypto_trust_*.rs` outputs | no pack expresses the consumer's projection profile (which rules render where in the consumer tree) | affidavit-trust-plane-pack (consumer-wiring extension) | 2026-09-28 |
| `ggen.toml` (verb pipeline) | `cnv:fieldName` disambiguation for colliding CLI verbs (`verify` x4, `certify` x3, `search` x2) rendered from multiple packs | no pack expresses cross-pack CLI verb-collision resolution (fieldName disambiguation in the consumer's verb pipeline) | affidavit-trust-plane-pack (consumer-wiring extension) | 2026-09-28 |
| `src/handlers.rs` trust-plane CLI glue | file transport, hex secret custody decode, exit codes, doctor check registration | no pack expresses consumer-side CLI adaptation of rendered crypto laws | affidavit-trust-plane-pack (handler seam law) | 2026-09-28 |
| `src/handlers.rs` evidence cluster (waves 1-2) | `evidence journal`/`crl-publish`/`crl-apply`/`heads` + `keys import`/`revoke`/`rotate` + `envelope list`/`export` handler bodies: real-receipt assembly, custody resolution, checksummed revocation sidecar, RFC 9162 tree-head re-derivation, SA2A approval projection, exit codes, feature-gated stub twins | no pack expresses consumer-side *composition* of rendered crypto laws into operator workflows (the pack renders module laws; it cannot own the consumer's orchestration of them) | affidavit-trust-plane-pack (handler seam law) | 2026-09-28 |
| `src/registry.rs` verb rows | REGISTRY entries for keys/envelope/evidence nouns (92 verbs, 9 nouns) | registry is the declared hand seam for CLI discovery metadata | affidavit-trust-plane-pack | 2026-09-28 |
| `ontology/affi-cli.ttl` verb-pipeline repair | `cnv:fieldName` disambiguation, rdf:List arg order, pipe-free about strings, alias facts | legacy generator held order/disambiguation outside the RDF model; facts now explicit | clap-noun-verb pack (upstream owner) | 2026-09-28 |
| `.github/workflows/rust.yml` crypto-trust job | CI lane for feature-gated projections; wave 1-2 add the macOS Secure Enclave lane and the `perf-budget` job (10x recorded-median drift alarm over `scripts/perf_budget_check.sh`) | no existing lane runs `--features crypto-trust` / the enclave feature on its native platform, or the recorded benchmark medians as a gate | affidavit-trust-plane-pack (CI law) | 2026-09-28 |
| `affidavit-core/src/crypto_verify.rs` (+ wasm ABI adapter `affidavit-wasm/src/crypto.rs`) | zero-dependency, `no_std` port of the envelope law: 12-field document model, byte-level decode, JCS-subset canonical bytes, domain-separated signing pre-image; no EC/ML-DSA arithmetic by design | the rendered plane modules carry serde/blake3/p256 dependencies; no pack expresses a no_std/zero-dep projection of the envelope law for dependency-free hosts (browser, HSM, on-chain) | affidavit-trust-plane-pack (no_std projection) | 2026-09-28 |
| `completions/affi.{bash,zsh,fish}` | held in registry-lockstep for the wave-2 verb surface (92 verbs / 9 nouns); the owner is `scripts/generate_completions.py` — this row admits any hand sync of these projections, validated by the drift court | completion projection is a consumer projection of `src/registry.rs`, not of the RDF verb graph — no pack expresses it; `tests/completions_drift.rs` is the admission gate for any hand-written byte in these files | clap-noun-verb pack (completion law) | 2026-09-28 |

Paydown plan: each seam is a one-time consumer mount; none grows with pack
surface. If the marketplace admits a consumer-wiring capability (manifest +
module-tree projection), these rows retire to zero. The evidence-cluster row
shrinks if the pack admits a handler-composition law; the `crypto_verify` row
retires when the pack renders a no_std profile; the completions row retires
when completion projection binds to the ontology instead of the registry.
