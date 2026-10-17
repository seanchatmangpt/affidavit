# WORK-ORDER — affidavit — v26.10.17 release certification

| field | value |
|---|---|
| id | AFFI-261017-01 |
| repo | `/Users/sac/affidavit` |
| base | `568beb9` — chore(release): bump CalVer to 26.10.17 in unison + changelog citing real exits |
| branch | `v26.10.16` (in-tree wave; coordinator owns transitions) |
| standing | UNKNOWN until the acceptance receipts below are witnessed on this base |

## Task

Certify the 26.10.17 release state of `/Users/sac/affidavit`: run the full AGENTS.md §6
verification ladder on the post-bump base `568beb9` and record real gate receipts, so the
release standing graduates from UNKNOWN to ALIVE (or is refused with a typed reason).

## Acceptance (each cell cites the command that must be witnessed)

| # | gate | command | required outcome | receipt |
|---|---|---|---|---|
| 1 | Full gate, pass A | `just validate` | exit 0 | PENDING — witness on base `568beb9`+ |
| 2 | Full gate, pass B (consecutive) | `just validate` (immediately after A, no tree change) | exit 0 | PENDING — must run after A with zero intervening edits |
| 3 | Golden lifecycle | `bash examples/golden_run.sh` | clean receipt ACCEPT (exit 0); tampered receipt REJECT (exit 2) | PENDING |
| 4 | Wasm court plane | `just wasm-build && just wasm-test` | 12/12 suites pass | PENDING |
| 5 | Core court plane | `cd affidavit-core && cargo test` | 8/8 suites pass | PENDING |
| 6 | Pin cell | `AFFIDAVIT_WASM=dist/affidavit_wasm.wasm cargo test --test registry_artifacts` | digest pin `b3cf38ba` matches post-bump rebuild | **PENDING-REPIN** — the working tree shows `affidavit-wasm/registry/ARTIFACTS.sha256` and `dist/affidavit_wasm.wasm` dirty against the pinned digest `b3cf38ba`; the pin predates the CalVer bump and must be re-derived post-bump before cell 6 closes. Pin drift without re-pin = `REFUSED_PIN_DRIFT`, never a silent overwrite. |

Cells 1–2 must run on the same tree state consecutively (reproducibility invariant, docs/WASM.md).
A gate that executes 0 tests (compile abort) is a loud FAIL, not a pass.

## Constraints (operator laws, verbatim)

- **No in-crate crypto.** No crypto algorithm is implemented in this project; every algorithm
  is an external dependency. Zero hand-implemented cryptography; KAT validation runs through
  the external implementation.
- **PQC-only signature surface, unconditional.** ML-DSA-65 (FIPS 204, RustCrypto); no SLH-DSA,
  no classical EC/ES256; ships unconditionally — no PQC feature flag — and includes the WASM
  artifact.
- **`ocel-compat` is the only optional feature.** No new feature planes beyond the existing
  clippy feature planes (`crypto-trust`, `pqc`).
- **No unsafe.** `#![forbid(unsafe_code)]` holds across the crate and subcrates.
- **Alloc-free verify.** The verifier path is allocation-free (`no_std`-compatible core,
  `affidavit-core` zero-dep).
- **Determinism.** No wall-clock in the chain; canonical JSON field ordering; same inputs →
  same receipt.
- **No tag without release-skill stop conditions.** The release is tagged only after the
  release skill's stop conditions are met — a green gate alone does not authorize a tag.

## Evidence discipline

Every claim in the closing receipt embeds real command output (command, exit code, test
counts). A receipt with no embedded output is SUSPECT; a claim with no receipt is not a
receipt. Standing vocabulary: `UNKNOWN | PARTIAL_ALIVE | ALIVE | BLOCKED | BUILD_BROKEN |
UNSUPPORTED` plus typed `REFUSED_*`.
