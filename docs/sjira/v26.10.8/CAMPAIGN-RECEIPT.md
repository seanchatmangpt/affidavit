# CAMPAIGN-RECEIPT — affidavit — v26.10.8

| field | value |
|---|---|
| repo | `/Users/sac/affidavit` |
| branch | `feat/advanced-witness-capability-set` |
| HEAD at receipt | this commit (parent `fd586a9d877ee3af8f48116805dce5e562b3203d`) |
| standing | PARTIAL_ALIVE (pin court 3/3 witnessed; crypto_trust workstream dirty) |

## Campaign commits this wave (v26.10.8..HEAD)

| SHA | one-line |
|---|---|
| `fd586a9` | chore(wasm): regenerate ARTIFACTS.sha256 header via pack template (attribution fix) |
| `109ccb4` | docs: fix dead relative links (fleet link sweep) |
| `6148126` | docs(wasm): correct pack attribution to rust-wasi-wasmex-pack |
| `77aca41` | docs: crypto provenance how-to (committed surfaces only) |
| `23bd25a` | docs: cross-reference ggen-marketplace (fleet campaign) |
| `7d1bbc3` | chore(repo): archive stale root docs, relocate one-off scripts, ignore mutants output |
| `932dd31` | docs: refresh STATUS/README to trust-plane era |

## Courts / gates witnessed

- Pin court: 3/3 passed (pinned surfaces match committed projections).

## Open residues

- `crypto_trust` workstream dirty in-tree: `benches/crypto_trust_bench.rs`,
  `src/crypto_trust_crl_file.rs`, `src/crypto_trust_envelope.rs` modified, plus
  `.ggen-v2/receipt-log.jsonl` / `receipt.json` churn — uncommitted, owned by the live
  witness-capability lane.
