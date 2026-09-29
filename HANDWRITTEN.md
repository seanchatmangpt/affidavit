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

Paydown plan: each seam is a one-time consumer mount; none grows with pack
surface. If the marketplace admits a consumer-wiring capability (manifest +
module-tree projection), these rows retire to zero.
| `src/handlers.rs` trust-plane CLI glue | file transport, hex secret custody decode, exit codes, doctor check registration | no pack expresses consumer-side CLI adaptation of rendered crypto laws | affidavit-trust-plane-pack (handler seam law) | 2026-09-28 |
| `src/registry.rs` verb rows | REGISTRY entries for keys/envelope nouns | registry is the declared hand seam for CLI discovery metadata | affidavit-trust-plane-pack | 2026-09-28 |
| `ontology/affi-cli.ttl` verb-pipeline repair | `cnv:fieldName` disambiguation, rdf:List arg order, pipe-free about strings, alias facts | legacy generator held order/disambiguation outside the RDF model; facts now explicit | clap-noun-verb pack (upstream owner) | 2026-09-28 |
| `.github/workflows/rust.yml` crypto-trust job | CI lane for feature-gated projections | no existing lane runs `--features crypto-trust` | affidavit-trust-plane-pack (CI law) | 2026-09-28 |
