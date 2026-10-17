# ONTOLOGY-PROJECTION — affidavit — v26.10.17

Work-order rows for the ontology/ggen integrity lane, branch `v26.10.16`
(HEAD at write time `b0325d4`). Repo `/Users/sac/affidavit`. Format follows the
v26.10.8 wave (`docs/sjira/v26.10.8/`).

Standing vocabulary: `UNKNOWN | PARTIAL_ALIVE | ALIVE | BLOCKED | BUILD_BROKEN | UNSUPPORTED`
plus typed `REFUSED_*`. Each row: change → projection pipeline step → court → commit.

## Work-order rows

| # | id | change | projection pipeline step | court | commit | standing |
|---|---|---|---|---|---|---|
| 1 | FM-LAW-020 | Orphan-verb closure: receipt verbs (`receipt_test` family) and dossier list flattening reconciled between ontology and CLI surface; `ggen sync run` exit 0 twice, byte-identical render | `ontology/affi-cli.ttl` → `ggen sync run` (x2, byte-identical) → `src/verbs/*.rs` wrappers | Parity courts in `src/registry.rs` (ontology-vs-registry count + wiring tests) green | `ea24b1c` | ALIVE |
| 2 | PQC-ABI-DISC | PQC ABI surface discovery: `pqc_assemble`/`pqc_verify` ops and the `pqc_refused` diag code existed in code but had never been graph-managed — landed as `wja:Op` facts in `ontology/affi-wasm.ttl` | `ontology/affi-wasm.ttl` → `ggen sync run` → `affidavit-wasm/src/{ffi.rs,abi.rs,abi_meta.rs}` + registry JSONs | Advertisement court caught the regression (graph-vs-artifact advertisement mismatch); wasm re-pin `d5ec6a3` | `a4b65c8` | ALIVE |
| 3 | WASM-REPIN | Two wasm artifact re-pins this wave. (a) Pre-bump re-pin `11663a66` after PQC surface drift. (b) Post-bump re-pin **IN FLIGHT / PENDING** after the CalVer bump (`568beb9`) shifted source identity — two-target-dir byte-identical runs queued, `ARTIFACTS.sha256`/`artifact-pin.json` dirty in-tree, uncommitted | `affidavit-wasm` two-target-dir build x2 → `cmp` byte-identical → pin `AFFIDAVIT_WASM` digest → `cargo test --test registry_artifacts` | `registry_artifacts` court; drift is typed `REFUSED_PIN_DRIFT` | `a8c3c2b` (pre-bump, content = `11663a66` digest) | PENDING (post-bump pin); pre-bump ALIVE |
| 4 | CTDD-GATE-RESTORE | ctdd self-gate restoration: the deletion lane's ggen re-render clobbered `#[cfg(feature = "ctdd")]` gates on four verb wrappers (template cannot emit gates) — restored by hand, documented ontology-lane procedure | post-ggen hand-restoration step (known irreducible residue; template limitation, not hand-edit of projections) | build exit 0; `cargo test --lib` 452/0 | `dfc80e6` | ALIVE |
| 5 | VERB-DELETE-FLOW | Verb deletion flow: `receipt test` dummy verb deleted; receipt audit made real. Receipt test: ontology → ggen → registry count 100 | `ontology/affi-cli.ttl` (verb removed) → `ggen sync run` → `src/registry.rs` count 100 → README 101→100 sweep (`38a9ed4`, `2c3a6a0`) | Parity courts green (registry-vs-ontology count, wiring, `release_identity` verb-count court) | `3aa41ea` | ALIVE |
| 6 | VERB-DOC-COMPLETIONS | Verb-doc + completion regeneration as projections: 101 auto-rendered reference pages + bash/zsh/fish completions regenerated from the registry/ontology, never hand-edited | `ontology/affi-cli.ttl` → registry → `docs/reference/verbs/` (101 pages, parity court row 20) + `completions/affi.{bash,zsh,fish}` | Parity court (row 20) green; completions_drift court (fix `9ab506c`, follow-up `5a94104`) | `89fc68a` | ALIVE |

## Receipts

- FM-LAW-020 row: `ggen sync run` exit 0 x2, renders byte-identical; parity
  courts green — witnessed in `ea24b1c` commit body.
- PQC row: advertisement court regression caught by the graph-vs-artifact
  comparison before pin; re-pin `d5ec6a3` carries the two-target-dir `cmp`
  receipt.
- ctdd row: `dfc80e6` commit message records build exit 0 and `--lib` 452/0.
- Verb deletion: `3aa41ea` receipt test walked ontology → ggen → registry 100;
  README sweeps `38a9ed4` + `2c3a6a0` cite the `release_identity` court.
- Completions: `9ab506c` (bash hyphen-identifier fix) + `5a94104` (drift
  court follow-up) are the standing receipts for the completions projection.

## Open residue

- Row 3 (WASM-REPIN): post-bump re-pin is PENDING. `affidavit-wasm/registry/`
  and `dist/affidavit_wasm.wasm` are dirty in-tree awaiting the second
  two-target-dir run and court pass before commit. Until then the wave's
  standing is capped at PARTIAL_ALIVE.
- `src/mining/data_quality_gate.rs` (untracked) and the crypto_trust plane
  remain other lanes' ownership — not rowed here.