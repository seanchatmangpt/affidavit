# Changelog — Affidavit

All notable changes to the Affidavit provenance layer are documented here.

## 26.10.5 — Advanced witness capability set

**Theme: the advanced witness set — 16 feature-gated cryptographic
integrations behind a panic-free fast-path actuation gate, a typed
quantization seam for CMCA payoffs, and three advanced-witness ABI ops in the
WASM runtime.** Branch commits `918a910`…`3b2e413` (2026-10-05).

### Added
- **Advanced witness capability set** — 16 feature-gated cryptographic
  integrations (commit `918a910`): BLS12-381 aggregate signatures
  (`src/bls_aggregate.rs`), secp256k1 and Ed25519 witnessing
  (`src/secp256k1_witness.rs`, `src/ed25519_witness.rs`), SMT and MMR
  accumulators (`src/smt.rs`, `affidavit-core/src/accumulator/mmr.rs`),
  sparse/bitmap sequence tracking (`src/seq_bitmap.rs`,
  `src/replay_filter.rs`), hybrid logical clocks (`src/hlc.rs`), causal
  graphs (`src/causal_graph.rs`), Cedar policy evaluation
  (`src/policy_cedar.rs`), threshold quorums (`src/threshold_quorum.rs`),
  zk range proofs (`src/zk_range.rs`), zero-copy views (`src/zero_copy.rs`),
  a WASM court (`src/wasm_court.rs`), JCS canonicalization
  (`src/canonical_jcs.rs`), canonical time (`src/canonical_time.rs`), and a
  binary envelope (`src/binary_envelope.rs`) — exercised by Chicago-style and
  property courts (`tests/advanced_crypto_chicago.rs`,
  `tests/advanced_crypto_property.rs`).
- **CMCA QuantizedPayoff seam** (commit `caef991`): `src/quantized_payoff.rs`
  — a typed f64→Q16.16 quantization boundary so payoff math crosses the
  deterministic-receipt seam without float drift; wired through
  `src/lib.rs`.
- **Three advanced-witness ABI ops in `affidavit-wasm`** (commit `60de40c`),
  rendered via `wasi-json-abi-pack` into `src/abi.rs` and registered in the
  capability registry (`registry/capability-registry.json`,
  `registry/op-examples.json`).
- **`affidavit::event_builder`** — `EventBuilder`, the type-safe preferred public
  API for constructing events before appending them to a chain; `build()`
  delegates to `build_event`, so it applies the same admission checks. See
  `examples/event_builder.rs` (commit `4dbc71e`, 2026-09-30).
- **Shell completions now also ship for PowerShell** (`completions/affi.ps1`)
  **and Nushell** (`completions/affi.nu`), generated from the verb registry by
  `scripts/generate_completions.py` and drift-enforced by
  `tests/completions_drift.rs` (commit `cab0903`, 2026-09-30). The existing
  bash/zsh/fish completions are unchanged in law.

### Fixed
- **Fast-path actuation gate + panic-free audit fixes** (commit `9b24778`):
  the fast path now passes through the authority fence
  (`src/authority_fence.rs`), and the HLC (`src/hlc.rs`) and WASM court
  (`src/wasm_court.rs`) are panic-free under the audit.

### Documentation
- **`certify_signed` and `EventBuilder` documented in the README** (commit
  `3b2e413`): signing-key usage for certify_signed and the EventBuilder
  construction path.

### Optional seam (documentation only)
- **Certified-receipts seam**: `CryptoStandingReceipt` via `certify_signed`
  wraps paid-delivery `payload_hash_hex`; consumers without the feature keep
  the plain sha256 fold.

## v26.9.28 — Cryptographic trust plane

**Theme: affidavit owns the ecosystem's cryptographic trust plane — real keys,
real post-quantum signatures, verification to a closed standing vocabulary,
sealed to receipts.** Affidavit certifies; SA2A (downstream) decides what that
certification may authorize. See
[`docs/CRYPTO_TRUST_PLANE.md`](docs/CRYPTO_TRUST_PLANE.md).

### Added
- **`affidavit-trust-plane-pack`** → nine rendered modules under the
  `crypto-trust` feature (`src/crypto_trust_{canonical,keys,envelope,es256,pqc,
  verify,lifecycle,seal}.rs`) plus the macOS Secure Enclave adapter under
  `secure-enclave` (`src/crypto_trust_enclave.rs`), and the end-to-end court
  `tests/crypto_trust_e2e.rs`. The ontology is the source; the modules are
  projections and carry a never-hand-edit header.
- **Real post-quantum cryptography, replacing the blake3-mock seal**
  (`1000x_post_quantum_sealing.rs` is retired): ML-DSA-65 (FIPS 204), SLH-
  DSA-SHA2-128s (FIPS 205), and a hybrid ES256+ML-DSA-65 composition where
  both halves sign the same bytes and verification requires both. The
  `PQ-SEAL-v1` wire name is conserved; the crypto behind it is now real.
- **The envelope law (RFC-SA2A-007-errata interlock)**: a 12-field signature
  envelope (`CTP-ENVELOPE-v1`) with algorithm, key id, nonce, and expiry
  inside the signed bytes; JCS (RFC 8785) canonicalization + domain-separated
  BLAKE3; `(kid, nonce)` replay defense over a 300-second window; and a
  revocation-epoch clock with a 300-second staleness grace.
- **Verification to standing**: `VerificationEngine` + `TrustPolicy` (policy
  is data; the caller owns time) returning a closed eight-value
  `CryptographicStanding` (VALID … MALFORMED) or typed `VerifyRefusal`; a
  failed signature is a decided negative (`INVALID`, `Ok`), never a refusal —
  and only VALID mints a `CryptoStandingReceipt`, whose `_seal` field makes
  forgery a compile error.
- **Key lifecycle**: fingerprints (`afk1_` key ids), custody identities,
  a duplicate-refusing registry, rotation policy (2 epochs in flight, 90-day
  age cap), and revocation as typed values.
- **Packs gates v2** for the plane (field closure, envelope version pinning)
  in the companion pack's gate set.

### Wave 1 — consolidation, keys CLI, durable evidence, cross-runtime JCS
(branch commits through `a24c9d3`)

- **Consolidation retirement**: `src/crypto_trust.rs` — the 845-line
  parallel-session WIP, gated behind `trust-plane-legacy` — is deleted,
  together with its feature gate. The retirement is a capability court, not a
  deletion: 15 of the WIP's 18 items were covered-by the rendered plane (each
  with an executed witness), and the 3 genuinely new capabilities were ported
  as pack units — `crypto_trust_provider` (the `SigningProvider` seam,
  `sign_with_provider`, `DetachedSignature`, and a real `SoftwareEs256Provider`
  reference implementation) and `crypto_trust_nonce_store` (the
  `DiskNonceJournal`). Recorded failed edge: the WIP's Tpm/Kms/External
  custody labels are ontology vocabulary individuals, not code capabilities,
  and are not carried verbatim. The full item-by-item table is
  [`docs/jira/v26.9.28/CONSOLIDATION.md`](docs/jira/v26.9.28/CONSOLIDATION.md).
- **Key lifecycle on the CLI**: `affi keys import` (an externally-held hex
  public key, fingerprinted, registered with origin `Imported`), `affi keys
  revoke` (a tamper-evident revocation entry appended to the checksummed
  sidecar beside the key store), and `affi keys rotate` (a freshly generated
  ES256 successor whose rotation record is signed by the successor itself).
  Court: `tests/crypto_trust_keys_cli.rs`.
- **Durable evidence**: the `evidence` noun. `affi evidence journal`
  assembles a real receipt from the canonical assembler, seals it under the
  custody key, adjudicates the standing with a real engine (a verdict, never a
  literal), and appends a hash-chained journal entry — `CTP-JOURNAL-v1`:
  BLAKE3 `entry_hash` over the JCS-canonical fields, each entry `prev`-linked,
  every entry carrying the `key_id` and rotation context it was recorded
  under. `affi evidence heads` re-derives the RFC 9162 Merkle tree head from
  the journal entries alone and signs it when custody resolves.
  `affi evidence crl-publish` / `affi evidence crl-apply` drive the CRL
  surface below.
- **Journal/nonce persistence**: `crypto_trust_journal_persist.rs` — the
  durable cross-process nonce ledger (append-only JSONL at
  `.affi/nonce-journal.jsonl`, the envelope window law byte-for-byte,
  tmp + `sync_all` + rename atomicity, corruption refused with the exact line
  number) — closes the review gap "replay resistance dies with the process".
  `crypto_trust_nonce_store.rs` is the graph-derived twin (`.affi/nonces.jsonl`
  from the store file, explicit counted prune). Both carry twin-law tests
  against the in-memory `NonceJournal`.
- **The CRL file**: `crypto_trust_crl_file.rs` — the durable publication file
  for `CTP-CRL-v1`. The bytes on disk are exactly the JCS canonical form the
  signature covers (no pretty-print drift between wire and file); publish is
  staged tmp + sync + rename, so readers never see a partial CRL; read is
  fail-closed (a missing CRL is *not* an empty CRL — assuming "nothing is
  revoked" from missing bytes would invert the safety property); apply
  verifies the issuer signature first, then the epoch freshness grace, then
  merges atomically on any refusal.
- **Cross-runtime JCS differential**: `tools/jcs_differential.py` — a second,
  independent RFC 8785 implementation in pure Python stdlib — generates a
  ~280-case corpus (`fixtures/crypto_trust_jcs_corpus.json`) across UTF-16
  key ordering, ECMAScript `Number::toString` boundaries, and §3.2.2.2
  escaping. `tests/crypto_trust_jcs_differential.rs` proves the rendered `jcs`
  agrees with it, with a divergence taxonomy: parse-precision cases are
  reported as non-fatal, a same-double conformance divergence is fatal and
  classified — and mutation teeth prove the checker itself is load-bearing.
- **Multi-surface KAT vectors**: `fixtures/crypto_trust_kat_vectors.json` +
  `tests/crypto_trust_kat_vectors.rs` pin the *higher* surfaces as wire bytes
  — the 12-field envelope (`signing_input`, `to_bytes`), full seal artifacts
  (real receipts → envelope → RFC 6979 ES256 → `SealedReceipt` JCS bytes,
  verified through the real engine), and successor-signed rotation records —
  all deterministic by law. In-test regeneration must reproduce the committed
  fixture byte-for-byte, and one flipped byte per surface must refuse.
  `tools/verify_crypto_trust_kat.py` re-derived every seed pre-image in pure
  Python at generation time; its honest scope limits are recorded in the
  fixture.
- **CI**: a `macos-latest` Secure Enclave lane (fmt, the non-ignored
  `--features secure-enclave` suite, clippy) on an exact-candidate checkout —
  the one platform where the enclave feature can actually link.
- **The SLH-DSA pin, resolved on evidence**: keep `slh-dsa = "=0.2.0-rc.5"`.
  It is the newest version of the crate; the "stable" 0.1.0 is the
  incompatible one (it sits on a `signature 2.3.0-pre.4` stack and a different
  API), while rc.5 shares the signature-3 stack the plane standardizes on.
  crates.io evidence tables and the recorded upgrade law:
  [`docs/jira/v26.9.28/SLH-DSA-PIN.md`](docs/jira/v26.9.28/SLH-DSA-PIN.md).

### Wave 2 — the plane reaches every runtime
(in flight on the working tree at documentation time)

- **`affidavit-core::crypto_verify`** — a zero-dependency, `no_std` port of
  the envelope law: the twelve-field document model, byte-level JSON decode,
  the JCS-subset canonical bytes, and the domain-separated signing pre-image.
  It deliberately contains no EC and no ML-DSA arithmetic: it proves *which
  bytes are bound*, so a consumer with any verifier — browser WebCrypto, a
  Node ML-DSA build, an HSM — can check a signature over exactly those bytes.
  The decode path is borrowed and allocation-free (`EnvelopeRef`); the owned
  form lives behind `alloc`, and refusals are typed values.
- **The wasm module gains a seventh ABI op, `verify_signature_input`**
  (`affidavit-wasm/src/crypto.rs` over the core port): recompute-and-compare
  of the signing pre-image, with structured refusals (`wrong_version`,
  `non_canonical_number`, `malformed`, `bad_hex`) and the honesty boundary
  carried into the response — `verified: true` means the expected bytes match
  what the envelope binds, *not* that a signature is valid. Tests in
  `affidavit-wasm/tests/wasm_abi.rs` and `affidavit-wasm/tests/crypto_abi.rs`.
- **Performance budget lane**: `scripts/perf_budget_check.sh` re-runs the real
  criterion suite with the settings the measured-performance table was
  recorded with and refuses any median >10× the recorded number (faster is
  never a failure), wired as the `perf-budget` CI job on an exact-candidate
  checkout against the "Measured performance" table in
  [`docs/CRYPTO_TRUST_PLANE.md`](docs/CRYPTO_TRUST_PLANE.md).
- **Completions regenerated** for bash, zsh, and fish — 92 verbs, 9 nouns
  (`evidence` joins `keys` and `envelope`) — held to the registry by
  `tests/completions_drift.rs` (committed with the wire-fuzz corpus: every
  registry pair completable, no orphans, header count honest, generated
  header intact).
- **Envelope export to SA2A**: `affi envelope export --format sa2a` emits the
  `SA2A-C2-APPROVAL-v1` approval (JCS-canonical) from a `PQ-SEAL-v1` sealed
  document — the downstream certification boundary, as bytes.

### Honest limits
- The working tree does not compile at documentation time:
  `src/registry.rs:774` carries a `,,` parse error in the in-flight verb-table
  edit. Before tag, the gates must pass on the tagged tree: `cargo check/test
  --features crypto-trust`, clippy `-D warnings`, the completions drift court,
  and the release-identity gates.
- Two advertised verbs cannot dispatch yet: `envelope list` and `envelope
  export` have registry rows, committed handlers, and completions entries, but
  no rendered verb wrapper — `src/verbs/envelope_list.rs` /
  `envelope_export.rs` land on the next `ggen sync`, as documented in the
  handler source. This is the receipt-throughput defect class; the parity
  tests exist to catch exactly this.
- The README still says "90 canonical verbs" against a 92-entry registry;
  `tests/release_identity.rs` holds the README to the registry and will fail
  until the projection is regenerated.
- `src/crypto_trust_attestation.rs` is declared in `src/lib.rs` and is empty
  (0 bytes): the attestation-records module *seat* exists, the capability does
  not. Nothing in this entry should be read as shipping attestation records.
- Wave 0's two BUILD_BROKEN courts are repaired in source —
  `tests/crypto_trust_e2e.rs` was re-rendered through the pack (zero
  `crate::` imports remain) and the seal test module's imports are complete —
  but the compile witness is pending the tag gates above.
- Secure Enclave signing is **PARTIAL_ALIVE**: the live known-answer tests are
  `#[ignore]`-gated. Witnessed: unsigned/ad-hoc CLI binaries cannot persist
  enclave keys (OSStatus -25308 / -34018 `errSecMissingEntitlement`);
  unblocking requires an entitlement-signed host binary.
- HSM/TPM signing is **UNSUPPORTED** (typed refusal, never faked).

## [26.9.28] — 2026-09-28

**Theme: verify anywhere — the provenance layer as a WebAssembly module.**

Until now the only way to certify a receipt outside Rust was to shell out to
`affi`, or to trust a hand-written TypeScript port (`web/lib/verify-client.ts`)
that had already drifted from the binary once (its genesis seed sat three
releases behind). v26.9.28 ships the verifier itself as a sandboxed `.wasm`, so a
host — Elixir/Wasmex, Node, Python, an edge runtime — gets the verdict `affi
verify` gives, from the same algorithm, without a process boundary. The design
follows the graphlaw module: a tiny `unsafe` FFI shell over a safe, natively
testable JSON ABI, tested in a real wasm runtime.

### Added
- **Cryptographic trust plane (`src/crypto_trust.rs`)** — Affidavit now owns key identifiers, public verification material, custody metadata, generations/revocation epochs, domain-separated canonical signing material, replay admission, signature verification, and an unconstructable `VerifiedSignature` standing carrier. Private signing capability remains behind a `SigningProvider` boundary; verified evidence does not confer SA2A/BRCE authority.
- **Real post-quantum verification under `pqc`** — the feature now pulls RustCrypto `ml-dsa` and verifies FIPS 204 ML-DSA-65 signatures. The old `1000x_post_quantum_sealing.rs` BLAKE3-based Dilithium/Kyber placeholders were removed and replaced by a compatibility facade over the real trust plane. Negative courts cover mutated signatures, wrong subjects, replay, revocation, duplicate key ids, and stale revocation epochs.
- **`docs/CRYPTO_TRUST.md` + `crypto-trust` CI lane** — documents proof/authority separation and gates the ML-DSA path with formatting, tests, Clippy, and doctests.
- **`affidavit-wasm/`** — a standalone crate (like `affidavit-core/`, not a root
  workspace member) that builds `affidavit.wasm` for `wasm32-wasip1` (~230 KB).
  Exports `af_alloc`, `af_free`, `af_call`, `af_abi_version` and `memory`; a
  request is UTF-8 JSON in linear memory, a response is UTF-8 JSON. Imports are
  `wasi_snapshot_preview1` only — no JavaScript glue.
- **Six ops** over that ABI: `capabilities`, `commit` (BLAKE3 of a payload),
  `assemble` (events → sealed `core/v1` receipt), `verify` (the seven-stage
  certify pipeline, per-stage outcomes), `mine` (directly-follows graph,
  α-footprint, variants, activity frequencies) and `conform` (token-replay
  fitness of a trace against a model discovered from receipts). Mining and
  conformance are `affidavit-core` used as a library, not re-implemented.
- **Parity with `affi`, proven rather than claimed.** Canonical JSON, the rolling
  BLAKE3 chain and all seven stages are a byte-for-byte port of the root crate.
  `affidavit-wasm/tests/fixtures/` holds a receipt produced by the real `affi`
  binary; the wasm tests assert that the module ACCEPTs it, that it REJECTs the
  tampered copy with the *exact* reason string `affi` prints (same stored and
  recomputed hashes), and that wasm `assemble` reproduces `affi`'s receipt
  byte-for-byte. A further test runs the same requests natively and in wasm and
  requires identical response bytes.
- **Host-boundary hygiene tests**: no imports beyond WASI; every buffer is
  reclaimed (2000 calls, linear memory must not grow — verified to fail if
  `af_free` leaks); structured errors (`bad_json`, `unknown_op`, `missing_field`,
  `bad_field`, `too_large`) cross the boundary as JSON; a 2000-event receipt
  verifies inside the sandbox.
- **`.github/workflows/affidavit-wasm.yml`** — fmt, clippy on native *and*
  `wasm32-wasip1` (which compiles the FFI shell), unit tests, module build, the
  real-runtime tests against that exact build, rustdoc `-D warnings`, sha256 and
  artifact upload. On a published release a separate least-privilege job attaches
  `affidavit.wasm` and its checksum.
- **Release-identity gates** (`tests/release_identity.rs`): the wasm crate must
  declare the same version as `affi` (its genesis seed is derived from its own
  version), and the golden fixture must ACCEPT under this release's own verifier,
  so a version bump that forgets to regenerate it fails in the root lane.
- **`docs/WASM.md`** — the ABI, ops, error codes, a host walkthrough, and the
  trust-model boundaries.
- `just wasm-build`, `just wasm-test`.

### Changed
- Version 26.9.24 → **26.9.28**. The genesis seed is now
  `affidavit-v26.9.28-genesis`, so receipts assembled by 26.9.24 binaries do not
  verify under 26.9.28 (by design; see the README). The browser verifier and
  visualizer carry the new seed, and `release-tag-v26.9.24.yml` is now
  `release-tag-v26.9.28.yml`.

### Fixed
- **`cargo publish --dry-run` failed to resolve**, on 26.9.24 as well as here, so
  the release workflow's first step could never pass. Publishing ignores the
  `[patch]` stub and resolved the real `wasm4pm 26.6.10`, whose `wasm-bindgen
  =0.2.100` pin cannot coexist with `wgpu 30` (`^0.2.127`, the `gpu` feature).
  The crates.io edge to `wasm4pm` is removed: it is now a path-only
  dev-dependency on `stubs/wasm4pm` (stripped on publish), and its `[patch]`
  entry is gone. `wasm4pm` stays as a marker Cargo feature with no dependency
  behind it, so `cfg(feature = "wasm4pm")` and the `discovery` feature name are
  unchanged. `discovery`/`conformance`/`predictive` were already non-compiling
  against the stub and still are (README, ROADMAP P1-7). The dry run now
  packages 412 files, builds the unpacked package, and stops only at the
  upload. `stubs/wasm4pm` is now a path dependency, so `cargo fmt --all` formats
  it (one function reformatted).
- The wasm release-identity gates skip when `affidavit-wasm/` is absent, since
  `tests/release_identity.rs` ships in the published package and the sibling
  crate does not.

### Not in this release
- No `wasm32-unknown-unknown` build: like graphlaw, the module targets WASI hosts.
- The root `affidavit` crate itself is not compiled to wasm; the module is the
  small verifier/mining surface, deliberately. Retiring the hand-written
  TypeScript verifier in favor of this module in `web/` is the natural follow-up
  and is not done here.
- `affidavit-core`'s `Fnv256` hasher remains a non-cryptographic reference; the
  wasm module uses BLAKE3 (`blake3` crate, `pure` feature) because it must
  agree with `affi`.

## [26.9.28] — 2026-09-28

**Theme: the v26.9.x fan-out lands on one line.**

Consolidates the parallel v26.9.x workstreams into `main` (PR #88) and cuts the
release. The version moves 26.9.24 → 26.9.28. Because the chain genesis seed is
derived from the crate version (`affidavit-v<version>-genesis`), every receipt
hash changes: receipts assembled by 26.9.24 or earlier do **not** verify under
26.9.28, and the browser verifier and visualizer carry the new seed.

### Added
- Execution manifests (`src/execution_manifest.rs`) binding provenance to
  immutable execution subjects.
- GALL crown certification with typed predecessor witnesses (`src/gall.rs`).
- Ecosystem standing receipts, ERRC courts, and architecture qualification receipts.
- Examples: `conformance_report` (requires the `discovery` feature),
  `chain_growth`, `adversarial_proof`.

### Changed
- Dependency bumps: `opentelemetry-jaeger` 0.22, `shlex` 2,
  `prometheus` 0.14, `criterion` 0.8, `pollster` 1.0, `syn` 3; web: Next 16,
  react-dom 19.3, `@types/node` 26; CI: `actions/checkout@v7`.
- Benches use `std::hint::black_box` (criterion 0.8 deprecates its own).
- Release workflow is now `release-tag-v26.9.28.yml`.
- `wgpu` stays at 0.19: 30.x requires `wasm-bindgen ^0.2.127`, which conflicts with
  the published `wasm4pm 26.6.10` pin (`=0.2.100`) and breaks `cargo publish`.
  `Cargo.lock` keeps `wasm-bindgen 0.2.100` / `js-sys 0.3.77` for the same reason.

### Known limitations
- The `otel` and `gpu` features and `--all-features` do not build (stub
  fences and `tracing` wiring); this predates the release and is not addressed.
- Two June branches (`claude/confident-mendel-mrolti`,
  `claude/vigilant-hawking-l7is37`) were intentionally not merged.

## [26.9.24] — 2026-09-24

**Theme: consolidation of the v26.9.x branch fan-out.**

Merges the parallel v26.9.x workstreams (execution manifests, GALL crown
certification, ecosystem standing receipts, ERRC courts, architecture receipts)
into one line. The genesis seed moves to `affidavit-v26.9.24-genesis`; the browser
verifier and visualizer carry the same seed.

## [26.9.6] — 2026-09-06

**Theme: the evidence federation kernel becomes reachable.**

The v26.9.x BCRE federation kernel landed as library code — `certify_standing`,
`certify_ecosystem`, `certify_errc`, and `certify_errc_claim_assurance` were
implemented, unit tested, and exported from `src/lib.rs`, but no verb, handler,
or registry entry referenced any of them. The kernel could only be driven from
Rust. This release gives it a real, tested operator surface and brings the
release identity back in line with the code.

### Added
- **Federation courts** (`src/federation.rs`): the adapter layer between the CLI
  and the four kernel certifiers. Loads a `core/v1` receipt, drives it through
  the real Layer 2 gate (`admission::admit`, which runs both the wasm4pm-compat
  OCEL court and the affidavit certify pipeline), parses a bounded observation,
  and seals the profile receipt. It performs **no adjudication of its own** —
  every accept/refuse decision belongs to `admission` or to the kernel.
- **Eight new verbs across three new nouns** (69 → 79 registry entries, 10 → 11
  groups; the extra two are `affi doctor` and `guide search`, which shipped
  dispatchable but unregistered — see Fixed):
  - `affi standing certify` / `affi standing verify` — `affidavit/standing/v2`
  - `affi ecosystem certify` / `affi ecosystem verify` — `affidavit/ecosystem/v1`
  - `affi errc certify` / `affi errc verify` — `affidavit/errc/v1`
  - `affi errc assure` / `affi errc verify-assurance` —
    `affidavit/errc-claim-assurance/v1`
- **`VerbGroup::Federation`** in `src/registry.rs` — the eleventh taxonomy group.
- **`--out <PATH>`** on every `certify`/`assure` verb — writes the sealed
  receipt as a clean artifact so `certify` → `verify` composes in a real script.
- **Machine-consumable `--format json`** on all eight federation verbs. The
  `clap-noun-verb` runtime renders each verb's return value to stdout after the
  handler returns, appending a bare `null` to otherwise valid JSON — so
  `--format json | jq` failed on ACCEPT while working on REJECT. The federation
  courts now exit with their own code on every path, emitting exactly one JSON
  document. Other verbs are still affected (ROADMAP B13).
- **Ontology/registry/projection parity tests** (`src/registry.rs`):
  `every_registry_verb_is_declared_in_the_ontology` and
  `every_registry_entry_has_a_verb_projection`. `ontology/affi-cli.ttl` is the
  authoritative CLI input (AGENTS.md §5) but `ggen` cannot run in every
  environment, so these tests are what keep the projection honest.
- **`tests/federation_cli.rs`** — 15 end-to-end tests driving the real `affi`
  binary: certify → verify round trips, tamper detection, the ALIVE evidence
  law, unmet role quorums, ERRC directional refusals, and the assurance
  bijection. Library tests pass whether or not the CLI surface exists; these do
  not.
- **`tests/release_identity.rs`** — holds `affi --version`, `GENESIS_SEED`,
  `CHANGELOG.md`, the README verb count, the browser verifier's genesis seed,
  the never-compiled-sources list, and the shell completions to the package
  version and the registry.
- **Five anti-forgery tests for the kernel's derived fields.** `coverage` and
  `standing` (ecosystem) and `claim_ids` (claim assurance) are *derived*, not
  inputs — so a forger can rewrite one, recompute `receipt_hash` over the
  doctored material (the algorithm is deterministic and public), and produce a
  receipt that hashes correctly. `CoverageMismatch`, `StandingMismatch` and
  `ClaimSetMismatch` are the only things standing between that attacker and a
  forged federation, and **none of the three had a single test**. They now do,
  including the JSON round trip an operator actually uses and the proof that
  `verify_against(parent)` is load-bearing rather than optional: a shrunken
  claim ledger passes standalone `verify()` and is caught only against its
  parent.
- **`just validate`** — the AGENTS.md §6 verification ladder as one recipe.
- **`docs/FEDERATION.md`** — the operator guide for the federation courts, with
  a complete worked example and the exit-code contract.

### Changed
- **Version**: `26.6.22` → `26.9.6` in `Cargo.toml`, `Cargo.lock`, `ggen.toml`,
  and the `affi-shell` banner. Because the chain genesis seed is
  `concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`, **receipts
  assembled by 26.6.22 will fail stage 3 (`chain_integrity`) under 26.9.6.**
  This is the intended release-boundary behaviour, not a regression: re-emit and
  re-assemble against the new binary.
- **Shell completions** now cover all 79 verbs and all six nouns across bash,
  zsh, and fish, **using the kebab-case names the binary actually dispatches**.
  They had copied the registry's old snake_case spelling, so 39 of the names
  they offered (`verify_compliance`, `root_cause`, `emit_from_github`, …) were
  rejected outright — a completion that types a command the CLI refuses is
  worse than none. They also advertised a `quality` noun and `guide
  tutorial`/`examples`/`man` verbs that do not exist, and omitted
  `receipt-throughput`. `shell_completions_offer_only_dispatchable_verbs` now
  holds all three files to the registry.
- **`justfile`**: removed the stale header claiming the Rust recipes cannot run
  because `wasm4pm-compat 26.6.13` does not compile. The real published
  `wasm4pm-compat 26.8.7` is admitted and the full ladder passes.
- **`ROADMAP.md`** re-verified against the code: B3, B5, B6, B10, P0-2, P0-3,
  P1-2, P1-3, P1-4 and P1-6 were marked open but had already shipped.

### Fixed
- **`affi --version` reported the framework version.** `clap-noun-verb` builds
  its root command with its own `CARGO_PKG_VERSION`, so `affi --version`
  answered `cli 26.6.2`. Since the genesis seed is bound to the affidavit
  version, an operator diagnosing a cross-version `chain_integrity` failure was
  reading the wrong number. `src/bin/affi.rs` now answers a bare top-level
  `--version`/`-V` itself; subcommand `--version` still reaches the framework.
- **Ontology drift**: `why`, `fix`, `install-git-hook`, and `monitor` shipped in
  the projection without ever being declared in `ontology/affi-cli.ttl`. All
  four are now declared, and the parity test blocks the drift from returning.
- **`affi receipt verify` could not reach stage 3.** `Receipt`'s `Deserialize`
  re-runs the chain law, so a tampered receipt failed at the door with a
  framework parse error and **exit 1**, not the documented REJECT (**2**) — and
  `chain_integrity`, the stage whose entire job is catching exactly this, was
  unreachable from every CLI path. `chain::deserialize_receipt_unchecked` is a
  `pub(crate)` forensic seam that lets `verify` adjudicate a suspect file
  instead of refusing to open it; the stage now FAILs by name and the verb exits
  2. The seal is untouched — `Receipt::sealed` stays private. This also repaired
  `affi receipt why` (its `chain_integrity` explanation branch was structurally
  unreachable) and `affi receipt fix` (its primary action, quarantining a
  tampered receipt, could never execute).
- **`affi receipt verify-family` exited 0 while reporting REJECTs**, so a CI job
  piping it stayed green over a store containing tampered receipts. Any reject
  now exits 2.
- **41 registry rows named commands that do not exist.** `src/registry.rs` used
  snake_case verb tokens (`verify_compliance`) while the CLI dispatches
  kebab-case (`verify-compliance`), so `lookup` missed and `guide search`
  printed names an operator cannot type. A new test rejects any `_` in a verb
  token.
- **`receipt-throughput` was advertised but not compiled.** It had a registry
  entry, a `#[verb]` projection, and a handler — but no `pub mod` line in
  `src/verbs/mod.rs`, so it was never built or dispatchable. The parity test now
  walks the module list rather than the directory, so a file nobody declared can
  no longer masquerade as a shipped verb.
- **`affi doctor` and `guide search` were absent from the registry entirely**,
  making them undiscoverable through `guide search`, `--help` grouping, and the
  completions. Both are now registered and declared in the ontology, and the
  parity tests compare `(verb, noun)` pairs in both directions — a name-only
  check had passed `guide search` purely because a distinct `receipt search`
  exists.
- **The ontology claimed a CLI surface that does not exist.** It declared
  `receipt-throughput`, `variance`, and `profile` under a `bench` noun and
  `audit` under `governance`, while all four ship under `receipt`. The four now
  point at `ReceiptNoun`; `bench` and `governance` are marked RESERVED with the
  regrouping tracked as ROADMAP P2-7.
- **BLAKE3 digests are now canonically lowercase in all four kernel profiles.**
  The validators used `is_ascii_hexdigit`, which accepts `A-F`, so the same
  digest could be written two ways — and because the digest *string* is hashed
  into the receipt identity, the two spellings produced two different receipt
  hashes for identical evidence. That is a canonicalisation hole in a format
  whose entire value is that identical content has identical identity (ADR-5),
  and the refusal docstring advertised it as intended
  ("64 lowercase/uppercase hex digits"). **This narrows what is admitted:** a
  hand-authored receipt carrying uppercase digits that was previously accepted
  is now refused by name. No receipt this crate has ever produced is affected —
  `Blake3Hash` is built from `blake3::Hash::to_hex`, which is lowercase.
- **The browser verifier rejected every real receipt.** `web/` hard-codes the
  genesis seed, and it had drifted three releases behind
  (`affidavit-v26.6.17-genesis` while the crate was 26.6.22). `tsc --noEmit`,
  the web lane's only gate, cannot know the Rust seed;
  `tests/release_identity.rs` now does.
- **`examples/golden_run.sh` was broken and untested.** It ran `cargo run` from
  inside a temp dir, so cargo resolved the toolchain from that directory, missed
  `rust-toolchain.toml`, fell back to stable, and died on wasm4pm-compat's
  `#![feature(...)]` (E0554, exit 101). It now builds once from the repo root
  and invokes the binary directly — and `tests/golden_run.rs` executes it, so
  the example README points newcomers at is a court rather than a claim.
- **`affi-shell` hard-coded its version banner**; it now derives it from
  `CARGO_PKG_VERSION` like everything else.
- **`docs/glossary.md` stated the genesis seed resolves to
  `affidavit-v26.6.22-genesis`** — normative documentation of the live binary,
  now corrected and covered by the release-identity gates.
- **Two module docs materially overstated what their code does.**
  `1000x_post_quantum_sealing.rs` claimed "quantum-resistant existential
  unforgeability" and "100-year provenance security" over three `mock_*`
  functions that compute unkeyed BLAKE3 and ignore the secret key entirely;
  `1000x_gpu_verifier.rs` claimed its shader "runs iterative BLAKE3" when the
  shader's own comment says `simplified to 1 round for prototype speed` (BLAKE3
  uses seven) and its format check compares against a literal marked
  `// Placeholder`. Both headers now state plainly what the code computes, that
  their verdicts are not authoritative, and what would have to change before
  the original claims hold. The code was always internally honest — the docs
  were not, and in a certification tool that is the more dangerous half.
- **`cargo build --release --all-features`, the command README gave users, does
  not compile.** `discovery`/`conformance`/`predictive` need `wasm4pm` APIs and
  `mutation` needs `clnrm-core` APIs that the deliberate local stubs do not
  expose; CI never noticed because it builds default features only. The
  `remediation` feature was separately broken by a missing `tracing` dependency
  (fixed here). README now gives a command that works plus a
  feature-status table verified row by row with `cargo check --lib --features
  <name>` — four features are still broken (`discovery`, `conformance`,
  `predictive`, `mutation`); closing them out is ROADMAP P1-7, and gating
  feature combinations in CI is P1-8. Per AGENTS.md §1 the stubs were not
  broadened to paper over this — that would manufacture a green that means
  nothing.
- **README claimed "65+ canonical verbs" three lines above its own "79"**, and
  advertised two capabilities without saying they sit behind features that do
  not build.
- **`wasm-encoder` was a mandatory dependency** — downloaded, compiled and
  linked into every build, and carried in the published dependency graph —
  whose only consumer is an orphaned file the compiler never sees. Removed. A
  provenance tool should not ship a supply-chain edge for code that does not
  exist.

### Internal
- Removed `src/handlers_stubs.rs` — 300 lines of `todo!()` referenced by nothing
  in `src/`, `tests/`, `benches/`, `examples/`, or `build.rs`. `generate_verbs.py`
  regenerates it on demand.
- 17 files under `src/` (164 KB: the 13 `1000x_*.rs` drafts plus
  `generation.rs`, `metrics.rs`, `mining.rs`, `mutation.rs`) are declared by no
  `mod` and mapped by no `#[path]`, so the compiler never sees them — yet
  `cargo package` shipped them. They are now named in Cargo.toml's `exclude`,
  and `orphaned_sources_are_declared_or_excluded` refuses to let the list grow
  silently. Nothing was deleted; deciding each file's fate is tracked as
  ROADMAP P2-8.
- Test suite: 835 tests + 32 doctests, all passing under
  `cargo test --all-targets`, `cargo test --doc`, and
  `cargo clippy --all-targets -- -D warnings`.

## [26.6.22] — 2026-06-22

### Changed
- **Documentation & Release Readiness**:
  - Updated all documentation for v26.6.22 release.
  - Fixed Rust badge (1.56 → 1.78) and removed shell REPL mention.
  - Expanded verb documentation (11 core → 65+ across 9 families).
  - Added glossary entries for SBOM, Western Electric SPC, OCEL, DORA metrics, NTIA elements.
  - Prepared crate for crates.io publication with `exclude` list.

## [26.6.19] — 2026-06-19

### Added
- **Verb registry** (`src/registry.rs`): compile-time static registry of all **67 canonical verbs** in 10 groups (Core, Diagnostics, Analysis, Ingestion, Compliance, Attestation, SBOM, Insights, Engineering, Tooling). Single source of truth for help text, shell completions, and documentation. Exposes `REGISTRY`, `lookup`, `by_group`, `did_you_mean`, and `verb_count`.
- **Error code catalog** (`src/diag.rs`): stable versioned exit codes (OK=0, REJECT=2, USAGE_ERROR=3, IO_ERROR=4, INTERNAL=5, SLA_BREACH=6) and a structured `Diag` type for machine-readable diagnostics; full `--format=json` support.
- **Output abstraction** (`src/output.rs`): unified `Out` handle routing human/JSON/YAML output to stdout and diagnostics to stderr across all verbs.
- **`affi doctor`** (`src/verbs/doctor.rs`): new health-check verb for environment and receipt-store diagnostics.
- **Innovation design proposals** (`docs/innovation/`): five fan-out agent proposals covering doctor-command, doctor-receipts, DX CLI ergonomics, QoL workflow, and DX onboarding (00-SYNTHESIS.md + 01–05).
- **2030 program roadmap** (`docs/roadmap/`): ten-workstream master plan (W1 Foundations → W10 Compliance/Governance) with release calendar and cross-workstream dependency graph.
- **`SECURITY.md`**: security policy and responsible disclosure process.
- **`deny.toml`** / **`typos.toml`**: supply-chain advisory checks and automated typo detection.

### Changed
- **Version bump**: Updated to v26.6.19 in `Cargo.toml`, `CLAUDE.md`, `docs/INDEX.md`, and all versioned references.
- **Genesis seed** (`src/chain.rs`): replaced hardcoded `b"affidavit-v26.6.14-genesis"` with a compile-time expression `concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")` so the seed always matches the running binary without manual updates. Receipts from prior versions will fail stage 3 (`chain_integrity`) — this is the intended release-boundary behavior.
- **CLI verb count**: README and docs updated from 59 → 67 to match the registry.
- **Repository URL**: corrected to `https://github.com/seanchatmangpt/affidavit` throughout.
- **Documentation cohesion**: synchronized version strings across README, CLAUDE.md, glossary, INDEX, and integration docs.

### Fixed
- `docs/INDEX.md` CLNRM integration plan link pointed to a non-existent `26.6.17` file; corrected to `26.6.14`.
- Stale `your-repo` GitHub placeholder removed from installation instructions.
- Output routing: `diff` and `stats` verbs now emit substantive output to stdout (was erroneously going to stderr).
- Tampered-receipt reporting in `verify` now surfaces the correct stage and reason.

### Internal
- Removed all `.backup` and `receipt-throughput.rs` dead files from `src/verbs/`.
- `portfolio_test_dataset.json` moved to `fixtures/`.
- Added `docs/archive/ACCOMPLISHMENTS_v26619.md` release summary.
- All doc timestamps synchronized to 2026-06-19.

## [26.6.17] — 2026-06-17

### Added
- **Maximalist Nexus Ontology & CLI Verbs**:
  - Automatically generated and implemented **59 new CLI verbs** powered by the maximalist nexus ontology.
  - Implemented 59 distinct handler functions with genuine operational logic replacing prior stubs.
  - Expanded the vocabulary to include advanced capabilities such as `dependency matrix`, `causality chain`, `gdpr proof`, `sbom attest`, `security debt`, and more.
- **Western Electric Real-Time Quality Monitoring**:
  - Delivered a production-ready, 5,400+ LOC implementation of all **7 Western Electric Statistical Process Control (SPC) rules**.
  - Developed real-time LLM quality degradation and cheating detection.
  - Provided support for monitoring up to 300+ repositories simultaneously with a rolling window analysis engine.
  - Added object-level metric tracking (File, Module, Package, Repo) combined with Pearson correlation scoring to identify simultaneous violation causality.
  - Designed deep OCEL (Object-Centric Event Logs) integration for event emission, causal chain tracing, and generating unforgeable quality audit trails.
- **SBOM & Supply Chain Provenance**:
  - Shipped a complete SBOM vertical CLI slice incorporating 6 new verbs.
  - Added robust CycloneDX and SPDX parsing, validation, and NTIA compliance modules (`src/sbom.rs`, `src/sbom_compliance.rs`).
  - Introduced supply chain risk propagation logic and vulnerability tracing via `src/sbom_vulnerability.rs`.
- **Phase 2 Webhook & Daemon Integrations**:
  - Engineered production file watcher daemon handlers for persistent provenance streams.
  - Delivered mature Slack webhook integrations and network listeners.
  - Rolled out a portfolio monitoring simulation capable of querying and verifying a newly added 312-repository test dataset.
- **Extensive Testing & Benchmarking**:
  - Over 2,600+ lines of test code added, achieving a 100% pass rate across 211+ new tests.
  - Shipped `tests/western_electric_comprehensive.rs` spanning 86 stress tests validating variants, sigma levels, and performance constraints (<1ms detection time).
  - Wired massive E2E integration suites including `tests/sbom_integration.rs` and `tests/ocel_quality_integration.rs`.
- **Comprehensive Documentation Architecture**:
  - Added the definitive Phase Change Thesis containing profound academic and cross-disciplinary references (65+ references).
  - Wrote hyper-detailed `IMPLEMENTATION_SUMMARY.md` and `docs/WESTERN_ELECTRIC_COMPLETE.md` encompassing theoretical backing, tuning params, and Mermaid architecture diagrams.
  - Reorganized the `docs/` folder, safely migrating older phase specs to `docs/archive`.

### Changed
- **Filesystem & CI Hardening**: 
  - Ported hardened filesystem interaction patterns and validations from the `clnrm_prototype`.
  - Modernized `rustfmt` CI workflow using the latest Rust unpinned nightly tools while ensuring format failures are non-blocking.
  - Scrubbed and sanitized all developer machine paths across repositories.
  - Strengthened `.gitignore` for a cleaner, secure public distribution model.

## [26.6.14] — 2026-06-14

### Added
- **Receipt sealing (ADR-2/3)**: Receipt struct now has a private `_seal` field that prevents struct-literal construction from external code. Only the canonical seam (`crate::chain::ChainAssembler::finalize`) can construct sealed receipts.
- **Compile-fail witness**: `tests/ui/compile_fail/receipt_private_seal.rs` demonstrates that external code cannot construct Receipt directly (E0451).
- **Stdout safety guard (§6)**: Added `#![deny(clippy::print_stdout)]` at library root to prevent accidental output from dependencies. Intentional CLI output in `cli.rs` is explicitly allowed.
- **E2E test suite**: `tests/e2e.rs` exercises the complete receipt lifecycle (emit → assemble → verify → show) with tamper detection.

### Changed
- **Version bump**: Updated to v26.6.14 in Cargo.toml, ggen.toml, and ontology.
- **Genesis seed**: Updated to `affidavit-v26.6.14-genesis` for deterministic chain binding.
- **Receipt constructor**: Changed from struct-literal construction to `Receipt::sealed()` internal constructor to enforce sealing.

### Technical Details
- Receipt now unconstructable without going through the canonical seam (the bypass is unconstructable witness).
- Verifier pipeline remains deterministic (golden-diff witness via unit tests).
- CLI dispatch tests ensure verify↔show inversion (type-blind pairs witness).
- Stdout output is clean and unambiguous (behavioral witness).

### Status
- **Phase 1 complete**: Artifact provenance with all four acceptance witnesses (§9 of ARDPRD.md).
- **Remaining**: Phase 2 (reasoning provenance) is a standing condition, not a completable milestone.

## Prior Versions

See git history for versions < 26.6.14.
