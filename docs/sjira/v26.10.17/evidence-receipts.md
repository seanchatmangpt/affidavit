# Evidence Receipts — v26.10.17 Release Wave (2026-10-17)

Ledger of landed receipts for the v26.10.17 saturation wave. Every row is
`command -> exit -> count`, compiled from commit-message receipts in
`git log` (no invented numbers). Sources are named per section.

## 1. Saturation gate — two consecutive passes

Pass A and pass B are the wave saturation gate over the same tree (residue
commit `ac89671` "fmt/lock residue from the saturation gate passes"). Figures
below are the gate receipt figures carried by the wave dispatch.

| # | Command | Exit | Count |
|---|---------|------|-------|
| A1 | `just validate` | 0 | — |
| A2 | `examples/golden_run.sh` (GOLDEN_A) | 0 | ACCEPT(0) -> REJECT(2) |
| A3 | wasm full suite | 0 | 12/12 suites |
| A4 | affidavit-core suite | 0 | 8/8 suites |
| A5 | no-default-features build | 0 | — |
| A6 | `cargo clippy --all-targets -- -D warnings` | 0 | — |
| A7 | `cargo fmt --all -- --check` | 0 | — |
| A8 | `cargo doc` (doc build) | 0 | — |
| B1–B8 | Pass B: identical command set, identical exits/counts | 0 | identical |

## 2. Test-count ledger per plane

| # | Command | Exit | Count |
|---|---------|------|-------|
| T1 | `cargo test --lib` (root, default) | 0 | 452/0 |
| T2 | `cargo test --features crypto-trust --lib` (crypto-trust plane) | 0 | 909/0/1 ignored |
| T3 | `cargo test` (affidavit-core, full) | 0 | 193/0 |
| T4 | `cargo test` (affidavit-core, post-deletion re-run) | 0 | 8/8 suites ok |
| T5 | wasm full suite | 0 | 106/0 (`commit` receipts, `jcs_parity 4/4`) |
| T6 | wasm pqc_abi suite | 0 | 15/15 (`ea24b1c`) |
| T7 | `cargo test --features pqc --lib kem_dual_impl_kat` | 0 | 4/4 (`e79ef96`) |
| T8 | `cargo test --test e2e_error_cases` | 0 | 4/4 (`8624548`) |
| T9 | `cargo test --test cli_vacuity_tripwire` | 0 | 19/0, stable x2 (`5b8589b`, `63055dd`, `580c4dd`) |
| T10 | ag4 adversarial court | 0 | 19/0 (`6c14830`) |
| T11 | `cargo test --test crypto_trust_e2e --features crypto-trust` | 0 | 14/0 (`1a81553`) |

## 3. Vacuity tripwire (ERRC-CREATE)

| # | Command | Exit | Count |
|---|---------|------|-------|
| V1 | `cargo test --test cli_vacuity_tripwire` | 0 | 19 live cells, 19/0, stable on two consecutive runs (`5b8589b`) |
| V2 | Tripwire cells incl. 10 positive controls (byte-identity detector) | 0 | 10 positive controls, VACUOUS-six repair cells: legal-hold, license x3, audit x2, fre-902, restitution (`5b8589b`) |

## 4. Registry / ontology parity

| # | Command | Command detail | Exit | Count |
|---|---------|----------------|------|-------|
| R1 | `cargo test --lib registry` | parity courts | 0 | 18/0 |
| R2 | `docs_verb_parity` | census == verb_count() == 100 | 0 | 2/0 (count 100) |
| R3 | `completions_drift` | completions vs registry | 0 | 7/0 |

## 4b. release_identity court

| # | Command | Exit | Count |
|---|---------|------|-------|
| I1 | release_identity court (README verb-count sites vs src/registry.rs) | 0 | 8/1 -> 9/0 after `38a9ed4` + `2c3a6a0` (101 -> 100, both sites) |

## 5. Wasm artifact re-pins (two-target-dir invariant)

| # | Command | Exit | Count |
|---|---------|------|-------|
| P1 | two-target-dir builds + `cmp` | 0 | byte-identical; new pin c0ddf0cf…ef13a88, 713303 B (`d5ec6a3`) |
| P2 | `AFFIDAVIT_WASM=… cargo test --test registry_artifacts` | 0 | 3 passed, 0 skipped (`d5ec6a3`) |
| P3 | two-target-dir builds + `cmp` | 0 | byte-identical; new pin b3cf38ba…d1a967, 716492 B (`a8c3c2b`) |
| P4 | `AFFIDAVIT_WASM=… cargo test --test registry_artifacts` | 0 | exit 0 (`a8c3c2b`) |

## 6. Audit report reference

`docs/reference/CLI_VACUITY_AUDIT_26_10_17.md` — 101 registry verbs executed
across 7 lanes; verdict distribution 62 EXECUTES / 17 REFUSED-TYPED / 9 THIN /
6 VACUOUS / 6 UNWIRED / 4 FEATURE-GATED. All VACUOUS+UNWIRED+THIN cells closed
by the ERRC wave (commits `127b2c0`, `e9a89be`, `0dd25cb`, `d7f4443`,
`a48a5a7`, `1b256eb`, `8153451`, `05461c3`, `3aa41ea`). Final verb count 100.
