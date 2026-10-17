# justfile — convenient task runner for the affidavit repo.
#
# Run `just` (or `just --list`) to see available recipes.
#
# The root crate builds and tests. The real published wasm4pm-compat v26.8.7 is
# the admitted structural dependency; `wasm4pm` and `clnrm-core` remain fenced by
# local stubs via `[patch.crates-io]` (see `stubs/`). Do not delete the stubs or
# the patch block to "fix" dependencies — a stub is a named capability boundary.
#
# `validate` runs the AGENTS.md §6 verification ladder in cheapest-first order.

# Default: list all available recipes.
default:
    @just --list

# --- the verification ladder (AGENTS.md §6) ---------------------------------

# Run the full ladder, cheapest high-information court first. This is the gate.
validate: errc-court fmt-check build test doctest clippy completions core-validate golden wasm-build wasm-test
    @echo "validate: every court passed"

# Exact-head ERRC fast court unit tests (cheapest court, no Rust build).
errc-court:
    python3 -m unittest discover -s scripts/tests -p 'test_ci_errc.py'

# Check Rust formatting without modifying files.
fmt-check:
    cargo fmt --all -- --check

# Format all Rust code in place.
fmt:
    cargo fmt --all

# Build every target (lib, bins, tests, benches, examples).
build:
    cargo build --all-targets

# Run the test suite across every target.
test:
    cargo test --all-targets

# Run documentation tests.
doctest:
    cargo test --doc

# Lint with warnings denied. This is a blocking CI gate.
clippy:
    cargo clippy --all-targets -- -D warnings

# --- end-to-end smoke -------------------------------------------------------

# Receipt lifecycle smoke test: emit -> assemble -> verify (honest and tampered).
golden:
    bash examples/golden_run.sh

# Re-mint the affidavit-wasm golden/tampered fixtures for the current version
# (docs/WASM.md "Regenerate the fixture on every version bump"). Refuses with
# REFUSED_VERSION_MISMATCH when CHANGELOG.md lacks a `## [<version>]` section.
golden-mint:
    python3 scripts/golden_receipt.py mint

# Verification-only genesis-drift court: fails when the web lane's hand-typed
# genesis seed literals (web/lib/verify-client.ts, web/app/visualizer/model.ts)
# do not match the current version's `affidavit-v<version>-genesis`. Never
# edits web/.
check-genesis:
    python3 scripts/golden_receipt.py check-genesis

# Federation courts end-to-end: standing, ecosystem, and ERRC through `affi`.
federation:
    cargo test --test federation_cli

# --- web (self-contained Next.js app, Node 22) -----------------------------

# Start the Next.js dev server.
web-dev:
    cd web && npm run dev

# Clean install + production build of the web app.
web-build:
    cd web && npm ci && npm run build

# Type-check the web app (no ESLint config in this repo; tsc is the gate).
web-check:
    cd web && npx tsc --noEmit

# --- sibling evidence lanes -------------------------------------------------

# The zero-dependency no_std core crate (not a workspace member).
core-test:
    cd affidavit-core && cargo test

# Full core-crate gate: tests, feature planes, lint, fmt, doc build.
core-validate: core-test core-ocel-test core-no-default core-clippy core-fmt-check core-doc

# ocel-compat plane: mining module against wasm4pm-compat OCEL v2 types.
core-ocel-test:
    cd affidavit-core && cargo test --features ocel-compat --lib mining

# Prove the verifier works without an allocator.
core-no-default:
    cd affidavit-core && cargo build --no-default-features

# Lint both feature planes (core is outside the root workspace; root clippy cannot see it).
core-clippy:
    cd affidavit-core && cargo clippy --all-targets -- -D warnings
    cd affidavit-core && cargo clippy --no-default-features -- -D warnings

# Formatting for the non-workspace core crate (root fmt-check cannot see it).
core-fmt-check:
    cd affidavit-core && cargo fmt --all -- --check

# Core doc build must be warning-free.
core-doc:
    cd affidavit-core && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# The stdlib-only Python genetic Cargo-feature optimizer.
confevo-test:
    cd tools/confevo && python3 -m unittest

# Build the WebAssembly module (wasm32-wasip1) -> affidavit-wasm/target/.../affidavit_wasm.wasm
wasm-build:
    cd affidavit-wasm && cargo build --lib --target wasm32-wasip1 --profile wasm

# Native unit tests + the compiled module driven in a real wasm runtime.
wasm-test:
    cd affidavit-wasm && cargo test

# --- shell completions --------------------------------------------------------

# Regenerate completions/affi.{bash,zsh,fish} from src/registry.rs.
# Deterministic: same registry -> byte-identical files. tests/completions_drift
# fails when a verb is added without running this.
completions:
    python3 scripts/generate_completions.py

# --- mutation court (crypto trust surface) -----------------------------------

# Targeted cargo-mutants regression court over the crypto-critical files.
# Crypto is unconditional by law: ML-DSA-65 via the external `ml-dsa` crate
# (affidavit-core/src/pqc.rs) and BLAKE3 via the external `blake3` crate
# (affidavit-core/src/digest.rs, crypto_verify.rs). No feature flags, no
# hand-rolled primitives. Baseline (2026-10-05): 129 mutants, 116+ caught,
# <= 3 survivors, all classified equivalent (see mutations/BASELINE.json).
mutate-crypto:
    cd affidavit-core && cargo mutants \
        --file src/pqc.rs \
        --file src/crypto_verify.rs \
        --file src/digest.rs \
        --timeout 300 \
        -j 4 \
        -- --lib --test blake3_chain_court --test pqc_external_validation
