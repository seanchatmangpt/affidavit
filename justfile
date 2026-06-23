# justfile — convenient task runner for the affidavit repo.
#
# Run `just` (or `just --list`) to see available recipes.
#
# NOTE ON RUST RECIPES: `rust-build`, `rust-test`, and `golden` currently
# can't run because the upstream crate `wasm4pm-compat 26.6.13` (published on
# crates.io) does not compile under current Rust nightly — roughly 550 compiler
# errors in the upstream crate itself. `fmt`/`fmt-check` need no dependencies
# and always work. The web recipes are fully self-contained.

# Default: list all available recipes.
default:
    @just --list

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

# --- rust formatting (needs NO sibling crates — always works) ---------------

# Format all Rust code in place.
fmt:
    cargo fmt --all

# Check Rust formatting without modifying files (the CI gate).
fmt-check:
    cargo fmt --all -- --check

# --- rust build/test (BLOCKED by broken upstream dep; see header) -----------

# Build the crate. NOTE: currently fails due to wasm4pm-compat 26.6.13 (see header).
rust-build:
    cargo build

# Run the test suite. NOTE: currently fails due to wasm4pm-compat 26.6.13 (see header).
rust-test:
    cargo test

# End-to-end golden smoke test. NOTE: runs the `affi` binary via `cargo run`,
# currently fails due to wasm4pm-compat 26.6.13 (see header).
golden:
    bash examples/golden_run.sh
