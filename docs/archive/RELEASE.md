# Affidavit v26.6.17 Release Notes

**Release Date:** 2026-06-14  
**Version:** 26.6.17  
**Status:** Release Candidate - Ready for crates.io

---

## Release Summary

Affidavit v26.6.17 completes **Phase 1 of ARDPRD** (Artifact Provenance Document) and is ready for publication to crates.io. This release includes:

1. ✅ Receipt sealing mechanism (ADR-2/3)
2. ✅ Deterministic BLAKE3 chain assembly
3. ✅ 7-stage cryptographic verification pipeline
4. ✅ CLI generation from ontology (ggen + clap-noun-verb)
5. ✅ Comprehensive test coverage (36 tests)
6. ✅ Benchmarking infrastructure (Criterion)
7. ✅ OpenTelemetry integration skeleton

---

## What's New in 26.6.17

### Core Features
- **Receipt Sealing**: Private `_seal` field makes struct-literal construction unconstructable (E0451)
- **Deterministic Chain**: BLAKE3 rolling hash with sorted canonical JSON ensures reproducibility
- **7-Stage Verifier**: Decidable pipeline (decode → format → chain integrity → continuity → commitments → profile → verdict)
- **CLI from Ontology**: Generated via ggen from `ontology/affi-cli.ttl`

### Quality & Testing
- **36 Tests**: 19 unit + 6 dispatch + 4 e2e + 1 UI (compile-fail)
- **Compile-Fail Witness**: Proves E0451 on struct-literal Receipt construction
- **Tamper Detection**: Adversarial tests verify chain integrity across mutations
- **Behavioral Witness**: CLI dispatch tests prove verify↔show distinction

### Safety
- **Stdout Guard**: `#![deny(clippy::print_stdout)]` prevents accidental output
- **Sealed Seam**: Only `ChainAssembler::finalize()` produces valid receipts
- **Non-Forgeable Carrier**: Private `_seal` field makes external construction fail

### Integrations
- **wasm4pm-compat**: Optional (feature: `evidence`) for Phase 2
- **OpenTelemetry**: Tracing integration (feature: `otel`)
- **Criterion**: Benchmarking infrastructure

---

## Breaking Changes

None. This is the first stable release.

---

## Deprecations

None.

---

## Known Issues & Limitations

### Open Residuals (Per ARDPRD §8)
1. **Undecidability relocated**: Rice's theorem is not defeated, only moved to construction boundary
2. **Verifier root-of-trust open**: Correctness of structural laws (continuity, chain integrity) is assumed
3. **Verify↔show distinction is type-blind**: Behavioral convention distinguishes them; type system cannot
4. **Bounded fragment**: Total structural admission is intractable; Blue River Dam bounds guarantee to decidable subset
5. **Nightly pin deferred**: Would be required for Evidence<_, Admitted, W> typestate (Phase 2)

### Known Limitations
- Trailing "null" in JSON output (clap-noun-verb; awaiting directed suppression upstream)
- Phase 2 (reasoning provenance) is standing condition, not completable milestone
- No IDE support yet (lsp-max integration future work)
- No high-scale performance optimization (SIMD token replay deferred)

---

## Migration Guide

N/A - First release.

---

## Install & Build

### From crates.io (after publishing)
```toml
[dependencies]
affidavit = "26.6.17"
```

### From Source
```bash
git clone https://github.com/anthropics/affidavit
cd affidavit
cargo build --release
```

### With Optional Features
```bash
# With Evidence typestate (Phase 2 preparation)
cargo build --features evidence

# With OpenTelemetry tracing
cargo build --features otel

# With all features
cargo build --all-features
```

---

## Usage

### Emit Operation-Events
```bash
affi receipt emit --type init --object app:service --payload - <<< "app init"
affi receipt emit --type transform --object data:artifact --payload - <<< "transform"
```

### Assemble Receipt
```bash
affi receipt assemble --out my-receipt.json
```

### Verify Receipt
```bash
affi receipt verify my-receipt.json
```

### Display Receipt
```bash
affi receipt show my-receipt.json
```

---

## Testing

All tests pass:
```bash
cargo test              # 36 tests
cargo test --lib       # 19 unit tests
cargo test --test cli_dispatch    # 6 dispatch tests
cargo test --test adversarial     # 6 adversarial tests
cargo test --test e2e             # 4 end-to-end tests
cargo test --test ui              # 1 compile-fail test
```

---

## Publishing to crates.io

### Prerequisites
- Rust 1.70+ (edition 2021)
- `cargo publish` credentials configured (`~/.cargo/credentials`)
- GitHub repository public

### Steps
```bash
# Verify all checks pass
cargo build --all-targets
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Dry-run publish
cargo publish --dry-run

# Publish to crates.io
cargo publish

# Verify publication
curl https://crates.io/api/v1/crates/affidavit
```

---

## Documentation

- [README.md](README.md) — Getting started and CLI surface
- [ARDPRD.md](ARDPRD.md) — Product & architecture requirements (PRD/ARD)
- [CHANGELOG.md](CHANGELOG.md) — Version history
- [STATUS.md](STATUS.md) — Phase 1 completion summary
- [INTEGRATIONS.md](INTEGRATIONS.md) — Library integrations and features
- [src/lib.rs](src/lib.rs) — Module documentation

---

## Support & Feedback

- **Issues**: Report bugs on GitHub
- **Discussions**: GitHub Discussions for design questions
- **Security**: Email security@affidavit.dev for security issues (if registered)

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

## Acknowledgments

Built with:
- **clap-noun-verb** for CLI framework
- **ggen** for code generation  
- **blake3** for cryptographic hashing
- **serde** for serialization
- **Criterion** for benchmarking

---

## What's Next

### Phase 2 (Standing Condition)
- [ ] Evidence<Receipt, Admitted, W> typestate from wasm4pm-compat
- [ ] Boundary-trace witness (β) integration
- [ ] chicago-tdd-tools for process mining assertions
- [ ] lsp-max for IDE support

### Operations
- [ ] Publish to crates.io
- [ ] Set up crates.io documentation
- [ ] Configure CI/CD for automated publishing
- [ ] Benchmark performance on various scales

---

**Affidavit v26.6.17 is ready for production use within Phase 1 scope (artifact provenance).**

*Last Updated: 2026-06-14*
