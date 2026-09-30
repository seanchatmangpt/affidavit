# RELEASE-READINESS — v26.9.28 crypto-trust plane

Standing: **BUILD_BROKEN** — release REFUSED pending gap list G1–G5. Every check below was executed for real in this session; nothing is projected or assumed.

- Subject: `/Users/sac/affidavit`, branch `feat/v26.9.28-crypto-trust-plane`, HEAD `a24c9d3d8c95624d059533a50558a4fb78c56e06`
- Tree state at run: DIRTY — 31 uncommitted files (cargo's dirty gate enumerates them; includes 10 `src/crypto_trust_*.rs`, 7 `src/verbs/*.rs`, `src/lib.rs`, `src/registry.rs`, tests, completions, CI workflow)
- Proposals live under `/tmp/affidavit-release-patches/` (01–04). None applied. Zero repo bytes written except this file.

## Check matrix

| # | Check | Command | Exit | Finding | Proposal |
|---|---|---|---|---|---|
| 1a | Publish dirty gate | `cargo publish --dry-run --features crypto-trust` | non-zero (refusal; exact code not captured on this pass) | REFUSED: "31 files in the working directory contain changes that were not yet committed" — publish cannot run clean until the feature work is committed (operator git transition; agents never run git) | — |
| 1b | Publish verify (past dirty gate) | `cargo publish --dry-run --features crypto-trust --allow-dirty` | **101** | Index update, manifest validation, packaging (479 files) and dependency resolution all PASSED — incl. `slh-dsa 0.2.0-rc.5`, `ml-dsa 0.1.1`, `p256 0.13.2`, `signature 2.2` compiled from the packaged tree. Verify build FAILED: `src/registry.rs:774:69` — `expected expression, found ,` on `.with_example("affi envelope export sealed.json --format sa2a"),,`. Single unconditional parse error ⇒ blocks default build, docs, and publish identically. No exclude-list crypto leak surfaced; no metadata complaints from cargo. | 02 |
| 2 | What ships | `cargo package --list --allow-dirty` | 0 | 479 files. LEAKS: `.ggen/` 13 (receipts/ocel/cache), `.ggen-v2/` 2, `.github/` 14, `semconv/` 4 (zero `src/` consumers), `ecosystem/`, `profiles/`, `release/`, `justfile`, `typos.toml`, `deny.toml`, `rust-toolchain.toml` (pins `nightly-2026-08-12`), `V2030.1.1-PRD-ARD.md` (internal PRD). BLOAT: `fixtures/` ~3.4 MB, dominated by `fixtures/crypto_trust_wire_corpus.json` **3.1 MB** whose sole consumer is `tests/crypto_trust_wire_fuzz.rs`. CORRECT non-leaks: `stubs/` ships 0 files (path-dep auto-exclusion); the 17 orphaned sources stay excluded; all 24 `src/crypto_trust_*.rs` ship — correct, they are declared, compiled code, not orphans. | 01 |
| 3 | Supply-chain gates | `cargo deny check` (cargo-deny 0.20.2, `all-features` graph) | **5** | advisories FAILED, licenses FAILED, bans ok, sources ok. **6 vulnerabilities**: rustls 0.23.40 RUSTSEC-2026-0285 (fix ≥0.23.45; via reqwest/ureq), quick-xml 0.37.5 ×2 (fix ≥0.41.0; via oxigraph→lsp-max), h2 unbounded DATA frames, protobuf uncontrolled recursion, `atomic`-family fmt::Pointer invalid deref. **5 unmaintained**: atty, number_prefix, proc-macro-error, opentelemetry-jaeger, opentelemetry_api. **1 unsound** (Error::downcast_mut). **Licenses**: BSL-1.0 ×3 (clipboard-win, error-code, str-buf ← rustyline 12), CDLA-Permissive-2.0 ×3 (webpki-root-certs, webpki-roots 0.26/1.0 ← reqwest/ureq), `clnrm-core 1.3.0` stub UNLICENSED (no license field in stub manifest; stub does not ship — local/CI gate defect only). Crypto-trust deps (signature/p256/ml-dsa/slh-dsa) drew zero advisories. | 03, 04 + `cargo update -p rustls -p h2 -p protobuf`; quick-xml ≥0.41 may require an upstream lsp-max/oxigraph bump — possible residual BLOCKED |
| 4 | Docs | `cargo doc --no-deps --features crypto-trust` | **101** | Same single parse error; build dies before the rustdoc pass, so **warnings count not measurable this session** (0 rustdoc warnings observable ≠ clean). BLOCKED by check 1b. | 02 |
| 5 | Module presence + gating | static + `cargo build --lib` (default) | build **101** | **24** `crypto_trust_*.rs` on disk (brief said 22 — observed 24), all declared `lib.rs:155-202`: 23 gated `crypto-trust`, 1 (`crypto_trust_enclave`) gated `secure-enclave`. Default cfg compiles **zero** trust-plane modules (static evidence). Verb layer (envelope_sign/verify/export, keys_generate/import/list/revoke/rotate, evidence_journal/heads/crl_publish/crl_apply) is declared unconditionally and delegates to `handlers.rs`, which carries paired `#[cfg(feature = "crypto-trust")]` real impls and `#[cfg(not(...))]` typed-refusal impls — the default binary refuses trust verbs instead of missing them. Compile-level confirmation and rlib size delta (informational) NOT measurable: default build hits the same parse error. | 02 |
| 6 | Semver/metadata | manifest inspection | — | `version = "26.9.28"` valid; license/readme/repo/docs present; categories include `cryptography`. GAPS: description says nothing about the trust plane (signatures, PQC, custody); keywords saturated at 5/5 with no signature/post-quantum term; `all` meta-feature omits `crypto-trust` (must NOT gain `secure-enclave` — its `security-framework` dep is macOS target-gated; putting it in `all` breaks non-macOS builds). `slh-dsa = "=0.2.0-rc.5"` explicit prerelease pin: resolved + compiled, carried as risk note (rc = moving upstream API). | 01 |

## BLOCKED — enumerated gap list (fix order)

| Gap | Blocks | Fix | Ref |
|---|---|---|---|
| G1 `src/registry.rs:774` double comma | default build, docs, publish dry-run | delete one `,` | `/tmp/affidavit-release-patches/02-registry-rs-syntax-fix.patch` |
| G2 31 uncommitted files | publish (dirty gate) | commit the feature branch work (operator git transition) | — |
| G3a license policy (BSL-1.0, CDLA-Permissive-2.0, unlicensed stub) | CI deny gate | allow-list the two permissive licenses; add license to stub | `03-deny-toml-licenses.patch`, `04-stub-clnrm-core-license.patch` |
| G3b RustSec advisories (rustls, quick-xml, h2, protobuf) | CI deny gate | `cargo update -p rustls -p h2 -p protobuf`; quick-xml ≥0.41 may need lsp-max/oxigraph upstream bump | command-line, no patch |
| G4 package hygiene (3.1 MB wire corpus, `.ggen*`, `.github`, PRD doc, tooling files) | package size/purity | exclude block; wire corpus + its sole test excluded as a pair | `01-Cargo.toml-metadata-and-exclude.patch` |
| G5 metadata (description, keywords, `all` feature) | crates.io discoverability, feature completeness | manifest patch | `01-Cargo.toml-metadata-and-exclude.patch` |

After G1–G5: re-run checks 1b/3/4/5 and only then a real `cargo publish`.

## Receipt

- Commands + exits: `cargo publish --dry-run --features crypto-trust` (refused, non-zero), `--allow-dirty` → 101; `cargo package --list --allow-dirty` → 0 (479 files); `cargo deny check` → 5 (0.20.2); `cargo doc --no-deps --features crypto-trust` → 101; `cargo build --lib` → 101.
- Falsifiers attempted: publish both with and without the dirty bypass (both gates witnessed firing); default build attempted to test whether the defect was feature-scoped (it is not — unconditional parse error).
- Ledger deltas: zero repo source bytes written; 4 patch proposals + this document only.
- What the operator did NOT have to write: the entire gap enumeration, the deny/license taxonomy, the gating audit, and all four patches.
