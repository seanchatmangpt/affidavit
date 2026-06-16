# justfile — convenient task runner for the affidavit repo.
#
# Run `just` (or `just --list`) to see available recipes.
#
# NOTE ON RUST RECIPES: the `affidavit` crate depends on FIVE sibling PATH
# crates that live OUTSIDE this repo (../clap-noun-verb, ../wasm4pm*,
# ../lsp-max, ../clnrm/..., ../chicago-tdd-tools). `rust-build`, `rust-test`,
# and `golden` therefore only work when that full sibling workspace is checked
# out alongside this repo. `fmt`/`fmt-check` need no dependencies and always
# work. The web recipes are fully self-contained.

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

# --- rust build/test (REQUIRE the sibling PATH crates; see header) ----------

# Build the crate. NOTE: needs the sibling path-crates listed in the header.
rust-build:
    cargo build

# Run the test suite. NOTE: needs the sibling path-crates listed in the header.
rust-test:
    cargo test

# End-to-end golden smoke test. NOTE: runs the `affi` binary via `cargo run`,
# so it needs the sibling path-crates listed in the header.
golden:
    bash examples/golden_run.sh
