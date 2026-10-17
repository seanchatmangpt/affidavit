# VERIFICATION-COURTS — affidavit — v26.10.17

| field | value |
|---|---|
| id | AFFI-261017-01 (court set for `work-order.md`) |
| repo | `/Users/sac/affidavit` |
| base | `568beb9` (26.10.17 post-bump) |
| lane file ownership | `docs/sjira/v26.10.17/verification-courts.md` (this file only) |
| standing | UNKNOWN — each expected count below is the court size on disk at base `568beb9` (test fns counted with `grep -c '#\[test\]'`); the release receipt must replace these with witnessed counts from a real run before standing graduates |

The court set below certifies AFFI-261017-01. For each family: the law it certifies,
its mutation-sensitivity (what flips it), and its command with the expected exit/count.

## Court set (19 families)

| # | court family | law certified | mutation-sensitivity (what flips it) | command | expected exit / count |
|---|---|---|---|---|---|
| 1 | vacuity tripwire | Every one of the 100 CLI verbs executes a real code path; the 19 live vacuity cells stay closed; no byte-identity stub verbs. | Re-landing a no-op dummy verb (the deleted `fix.rs` class) or deleting a corpus cell flips it FAIL. A 0-test run (compile abort) is a loud FAIL, not a pass. | `cargo test --test cli_vacuity_tripwire` | exit 0; 10 tests, 0 failed |
| 2 | release_identity | Version identity across the release surface: Cargo.toml, CHANGELOG `## [26.10.17]`, wasm ABI meta, and the version-matched genesis seed agree on one CalVer. | Any single-site bump (one file moved to 26.10.18 alone) or CHANGELOG section rename flips it FAIL on version mismatch. | `cargo test --test release_identity` | exit 0; 9 tests, 0 failed |
| 3 | registry/ontology parity | The CLI surface is a faithful projection of `ontology/affi-cli.ttl`: registry `VerbEntry` rows match the ontology, entry count matches the compile-time constant, final verb count 100. | Adding/removing a verb without ontology + registry + count constant in step (CLAUDE.md §Add-a-New-Verb steps 1/4 skipped) flips the parity tests FAIL. | `cargo test --lib registry` | exit 0; 14 tests, 0 failed |
| 4 | completions drift | Shell completions (`completions/affi.{bash,zsh,fish}`) are a projection of the registry — no verb added, renamed, or deleted without regenerating completions. | Adding a verb without regenerating completions, or hand-editing a completion file without a registry change, flips the drift court FAIL. | `cargo test --test completions_drift` | exit 0; 7 tests, 0 failed |
| 5 | docs_verb_parity | Documentation claims about the verb surface match the registry (verb-count sites, verb lists in README/CLAUDE.md/AGENTS.md). | Bumping the registry without the doc verb-count sites (the `38a9ed4` class of drift) flips it FAIL. | `cargo test --test docs_verb_parity` | exit 0; 2 tests, 0 failed |
| 6 | ag4 adversarial | The ag4 quartet of adversarial attacks stays closed: the four published attack paths against the crypto surface are refused with typed errors, court 19/19. | Reintroducing any one of the four closed attack paths (the `c2119a5` mutations) flips its court case FAIL; the court must stay at 19/19. | `cargo test --test ag4_adversarial_crypto` | exit 0; 19 tests, 0 failed |
| 7 | frost_nonce | FROST threshold-quorum nonces are single-use: nonce reuse is refused with `NonceReuse` (row 81c CLOSED). | Reusing a FROST nonce across two signing rounds (the `2e9a18d` mutation) flips the court FAIL via the typed refusal path. | `cargo test --test frost_nonce_court` | exit 0; 4 tests, 0 failed |
| 8 | pqc_external_validation | No in-crate crypto: ML-DSA-65 KAT validation runs through the external RustCrypto implementation (`ml-dsa`), FIPS 204 vectors, PQC-only surface, unconditional (no feature flag). | Any hand-rolled or stubbed signer/verifier, a classical EC/ES256 path, or a KAT fixture checked against itself (fixture == computation) flips it FAIL. | `cargo test --test crypto_trust_kat_vectors --test crypto_trust_kat_fixture --test crypto_trust_acvp` | exit 0; 13+7+7 = 27 tests, 0 failed |
| 9 | kem / SLH / x-wing / zk-range | KEM dual-implementation ML-KEM-768 KAT agreement; KEM cross-parameter-set confusion refused; seed identity; SLH-DSA SPKI/PKCS8 + opt_rand determinism + wire-budget pins; x-wing dk->ek parity and dual-mode divergence; zk-range bulletproof bounds and Pedersen opening law. | Swapping one KEM implementation for a divergent one (KAT disagreement), cross-wiring parameter sets, mutating an SLH wire budget, breaking x-wing derived-key parity, or emitting an out-of-range zk-range witness flips the owning case FAIL. | `cargo test --features pqc --test kem_dual_impl_kat --test adversarial_pqc` and `cargo test --features zk-range --lib zk_range` (pqc / zk-range feature planes; kem_dual_impl_kat is `required-features = ["pqc"]` — without the flag cargo silently skips the target, which reads as a vacuous pass and is forbidden) | exit 0; kem 5 + adversarial_pqc 13 = 18, plus 29 zk_range lib tests, 0 failed |
| 10 | bcinr | Double-mix tamper propagation: any tamper in a bcinr double-mix composition propagates to a detectable digest divergence (the `6534c13` court). | Neutralizing the second mix (single-mix mutation) so a tampered input verifies clean flips it FAIL. | `cargo test --test dfcm_chicago` | exit 0; 9 tests, 0 failed |
| 11 | external-evidence | External evidence crosses the boundary only through the admission corpus: out-of-boundary evidence is refused typed, in-boundary evidence is admitted with a receipt (the `f3871f5` corpus). | Admitting unsigned/unattributed external evidence, or refusing well-formed in-boundary evidence, flips the boundary cases FAIL. | `cargo test --test crypto_trust_evidence_cli` | exit 0; 6 tests, 0 failed |
| 12 | permutation corpus | Verify is permutation-robust: seeded event-order permutations produce the canonical verdict/root (row 45, `9813c44`; SMT roots permutation-invariant, row 101). | An order-dependent verifier (verdict or canonical root changes under a seeded permutation with identical content) flips it FAIL. | `cargo test --test verifier_permutation_corpus` | exit 0; 5 tests, 0 failed |
| 13 | SjReceipt parity | SjReceipt TTL projection parity: the affidavit chain bridges to a Semantic-Jira receipt with 1:1 field identity and the standing vocabulary maps 1:1, out-of-vocabulary standings refuse typed (the `3215f69` class). | Dropping/renaming a projected field, or adding an out-of-vocab standing that silently maps, flips the parity cases FAIL. | `cargo test --test sj_projection_parity` | exit 0; 5 tests, 0 failed |
| 14 | dual-impl KAT | Two independent implementations agree on ML-KEM-768 known-answer vectors — the generator is not its own oracle. | Collapsing to a single implementation, or editing a KAT vector so only one impl agrees, flips the disagreement case FAIL. | `cargo test --features pqc --test kem_dual_impl_kat` | exit 0; 5 tests, 0 failed (pqc plane; skipped target = vacuous pass, forbidden) |
| 15 | quarantine_determinism | Doctor quarantine sidecars are wall-clock-free and reproducible (row 79, `f9a5c60`): same inputs, same quarantine bytes. | Reintroducing a wall-clock timestamp into the quarantine sidecar path flips it FAIL on two-run byte comparison. | `cargo test --test quarantine_determinism` | exit 0; 2 tests, 0 failed |
| 16 | output_no_trailing_null | B13: no success path emits a stray trailing null byte on any output format (the `05461c3` kill stays killed). | Reintroducing a trailing `\0` on any writer path flips the byte-level assertions FAIL. | `cargo test --test output_no_trailing_null` | exit 0; 3 tests, 0 failed |
| 17 | cli_exit_codes | Verdict-determined, stable exit codes from the diag catalog: ACCEPT=0 / REJECT=2, typed refusals map to catalog exits (the `1b256eb`/`8624548` courts). | Changing any verb to a constant exit, or reordering the diag catalog, flips the mapping cases FAIL. | `cargo test --test cli_exit_codes` | exit 0; 6 tests, 0 failed |
| 18 | cli_emit_from_courts | All six `emit-from-*` verbs are wired through `emit_with_payload` and produce real chain events (the `0dd25cb` wiring). | Un-wiring any one emit-from verb to a stub passthrough flips its court case FAIL. | `cargo test --test cli_emit_from_courts` | exit 0; 4 tests, 0 failed |
| 19 | registry_artifacts pin | The wasm artifact is admitted only on two-target-dir byte-identity; the pinned digest in `affidavit-wasm/registry/ARTIFACTS.sha256` matches the committed module; drift is `REFUSED_PIN_DRIFT`, never a silent overwrite. | Any wasm source drift that changes `dist/affidavit_wasm.wasm` without the re-pin procedure (or a hand-edited pin) flips it FAIL with `REFUSED_PIN_DRIFT`. | `AFFIDAVIT_WASM=dist/affidavit_wasm.wasm cargo test --test registry_artifacts` | exit 0; 3 tests, 0 failed |

## Feature-plane courts (the plane above does not see these by default)

| court family | law | command | expected |
|---|---|---|---|
| crypto-trust adversarial plane | crypto_trust mutation/adversarial corpus, ct timing invariance, ed25519-strict vectors, bls rogue-key registered divergence (row 82) | `cargo test --features "crypto-trust pqc ct_timing_invariance" --test adversarial --test adversarial_deontic --test adversarial_folding --test adversarial_git_merkle --test adversarial_nullifier --test adversarial_pqc --test adversarial_wire_ctrt --test ct_timing_invariance --test crypto_trust_adversarial --test crypto_trust_mutation_court` (feature flags are load-bearing: `ct_timing_invariance` and the ed25519-strict targets carry `required-features`; a skipped target is a vacuous pass, forbidden) | exit 0; 0 failed (corpus sizes: adversarial 6, deontic 11, folding 3, git_merkle 3, nullifier 5, adversarial_pqc 13, wire_ctrt 7, ct_timing 3, ct_adversarial 29, mutation_court per-file) |
| clippy feature planes | lint law on both feature planes | `cargo clippy --all-targets --features "crypto-trust pqc" -- -D warnings` | exit 0; 0 warnings |

## Vacuity-sweep law

**Every court in this file has a positive control that must fire.** A court that cannot
fail is not a court: each family above is admitted only because a named mutation
(column 4) flips it to FAIL, and a run that executes 0 tests (compile abort) reads as
a loud FAIL, never a pass. A skip must be loud (`[SKIP] <reason>`), never a faked
pass. Any family whose positive control cannot be exhibited is not a receipt and does
not count toward the release standing.

## Receipt discipline

Expected counts above are read from the tree (`grep -c '#\[test\]'` at base `568beb9`)
— they are court sizes, not witnessed counts. The closing release receipt must embed
real command output (command, exit code, test counts) for every family; a receipt with
no embedded output is SUSPECT, and a claim with no receipt is not a receipt.
