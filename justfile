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
validate: errc-court fmt-check build test doctest clippy
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

# The stdlib-only Python genetic Cargo-feature optimizer.
confevo-test:
    cd tools/confevo && python3 -m unittest
