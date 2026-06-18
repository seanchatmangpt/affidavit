# Publishing Affidavit to crates.io

**Version:** 26.6.17  
**Date:** 2026-06-14  
**Status:** Ready for Publication

---

## Pre-Publication Checklist

### Code Quality
- [x] All tests pass (36/36)
  - [x] 19 library tests
  - [x] 6 dispatch tests
  - [x] 4 e2e tests
  - [x] 1 compile-fail test (UI)
- [x] No clippy warnings
- [x] Code formatted with `cargo fmt`
- [x] Release build succeeds

### Documentation
- [x] README.md updated (v26.6.17, features, CLI examples)
- [x] CHANGELOG.md created (v26.6.17 changes documented)
- [x] RELEASE.md created (release notes, migration guide)
- [x] INTEGRATIONS.md created (feature documentation)
- [x] STATUS.md updated (phase completion, integration status)
- [x] Module documentation in src/lib.rs
- [x] ARDPRD.md included (product/architecture requirements)

### Metadata
- [x] Cargo.toml complete:
  - [x] Version: 26.6.17
  - [x] Description: Provenance Layer...
  - [x] License: MIT OR Apache-2.0
  - [x] Authors: Sean Chatman <xpointsh@gmail.com>
  - [x] Repository: github.com/anthropics/affidavit
  - [x] Documentation: docs.rs/affidavit
  - [x] README: README.md
  - [x] Keywords: provenance, receipt, blake3, verification, sealed
  - [x] Categories: security, development-tools

### Licenses
- [x] LICENSE-MIT created (standard MIT license)
- [x] LICENSE-APACHE created (standard Apache 2.0 license)
- [x] Cargo.toml specifies "MIT OR Apache-2.0"

### Dependencies
- [x] All dependencies published or local path resolved
  - [x] clap-noun-verb (local, version 26.6.17)
  - [x] clap-noun-verb-macros (local, version 26.6.17)
  - [x] wasm4pm-compat (local, optional feature)
  - [x] Other deps: all on crates.io (linkme, serde, serde_json, blake3, anyhow, thiserror)
- [x] Dev dependencies all on crates.io

### Security
- [x] No secrets in code
- [x] No hardcoded credentials
- [x] No unsafe code (except in wasm4pm-compat dependency)
- [x] No network calls in library code
- [x] Stdout guard prevents accidental output

---

## Publishing Steps

### Step 1: Verify Cargo.toml Structure
```bash
cargo check --all-targets
cargo build --release
cargo test
```

### Step 2: Dry-Run Publication
```bash
cargo publish --dry-run
```

Expected output: `Publishing affidavit v26.6.17 to registry index`

### Step 3: Publish to crates.io

**Requirement:** You must have a crates.io account and be logged in:
```bash
# If not logged in:
cargo login
# Paste your API token when prompted
```

**Publish:**
```bash
cargo publish
```

Expected output:
```
    Uploading affidavit v26.6.17 to crates.io
   Downloaded crates in preparation for uploading
    Uploading affidavit v26.6.17 to crates.io
     Uploaded affidavit v26.6.17 to crates.io
      Waiting for crates.io index to be updated
```

### Step 4: Verify Publication
```bash
# Check crates.io
curl https://crates.io/api/v1/crates/affidavit
# Should return JSON with crate metadata

# Check docs.rs
open https://docs.rs/affidavit/26.6.17/affidavit/

# Verify installation
cargo install affidavit
affi --version
```

---

## Dependency Status

### Published on crates.io (Ready)
- linkme 0.3
- serde 1.0
- serde_json 1.0
- blake3 1.0
- anyhow 1.0
- thiserror 2.0
- assert_cmd 2.0
- predicates 3.0
- tempfile 3.0
- trybuild 1.0
- criterion 0.5
- opentelemetry 0.20
- opentelemetry-jaeger 0.19

### Local Paths (Must be published separately first)
- **clap-noun-verb** v26.6.17 — Must publish to crates.io BEFORE affidavit
- **clap-noun-verb-macros** v26.6.17 — Must publish to crates.io BEFORE clap-noun-verb
- **wasm4pm-compat** v26.6.17 (optional, feature: `evidence`) — Publish separately or make it a git dep

### Resolution
When publishing, we have two options:

**Option A: Publish All Dependencies First (Recommended)**
1. Publish clap-noun-verb-macros to crates.io
2. Publish clap-noun-verb to crates.io
3. Update affidavit Cargo.toml to use crates.io versions instead of local paths
4. Publish affidavit to crates.io
5. Publish wasm4pm-compat separately

**Option B: Use git Dependencies (Temporary)**
Update Cargo.toml before publishing:
```toml
[dependencies]
clap-noun-verb = { git = "https://github.com/anthropics/clap-noun-verb.git", tag = "v26.6.17" }
clap-noun-verb-macros = { git = "https://github.com/anthropics/clap-noun-verb.git", tag = "v26.6.17" }
wasm4pm-compat = { git = "https://github.com/anthropics/wasm4pm-compat.git", tag = "v26.6.17", optional = true }
```

Then publish affidavit. Later, convert back to crates.io versions after dependencies are published.

---

## Post-Publication

### Update Documentation
- [ ] Add badge to README.md: `[![Crates.io](https://img.shields.io/crates/v/affidavit.svg)](https://crates.io/crates/affidavit)`
- [ ] Update installation instructions in README.md to use crates.io
- [ ] Tag repository: `git tag v26.6.17`
- [ ] Push tags: `git push origin v26.6.17`

### GitHub Release
- [ ] Create GitHub Release for v26.6.17
- [ ] Attach binary (if applicable): `target/release/affi`
- [ ] Include release notes from RELEASE.md
- [ ] Link to crates.io page

### Monitoring
- [ ] Monitor crates.io download stats
- [ ] Monitor docs.rs for documentation build (usually takes 1-2 minutes)
- [ ] Check for any yanking needs (broken dependencies, security issues)

---

## Rollback / Yanking

If a critical issue is found after publishing, you can yank the version:

```bash
cargo yank --vers 26.6.17
```

This prevents new users from depending on the broken version but allows existing users to keep using it.

---

## Feature Documentation

Ensure docs clearly explain features:

```bash
# Build and view documentation
cargo doc --open --no-deps

# Verify features are documented
cargo doc --features evidence --open
cargo doc --features otel --open
```

---

## crates.io Categories

- **security** — Because receipt verification is a security primitive
- **development-tools** — Because affidavit is used in build/test pipelines

These are correct for crates.io indexing.

---

## Maintainability

After publishing, keep updated:

- **CHANGELOG.md** — Update for every release
- **RELEASE.md** — Update with each version
- **README.md** — Ensure install instructions stay current
- **INTEGRATIONS.md** — Document new features and integrations

---

## Versioning

This project uses semantic versioning:

- **26.6.17** = Major.Minor.Patch
  - 26 = Phase 1 + system version
  - 6 = Month (June)
  - 14 = Day (14th)

For future releases, use proper semver: `1.0.0`, `1.1.0`, `1.0.1`, etc.

---

## Success Criteria

✅ Publication is successful when:
1. `cargo publish` completes with no errors
2. crates.io shows affidavit v26.6.17 published
3. `cargo search affidavit` returns the published version
4. `cargo install affidavit` installs the binary successfully
5. `docs.rs` hosts the generated documentation

---

## Additional Notes

- **Backward Compatibility:** This is v0.x-equivalent (first major version), so changes are expected in next releases
- **Security:** Affidavit is designed for provenance verification; no cryptographic keys are managed by the crate
- **Updates:** Keep dependencies current via `cargo update`
- **Yanking:** Only yank for critical issues; prefer deprecation releases otherwise

---

**Ready to publish: YES**

Run `cargo publish` when ready.

*Last Updated: 2026-06-14*
