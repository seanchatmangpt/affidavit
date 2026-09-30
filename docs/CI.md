# CI architecture

Goal: minimize PR wall-clock and runner cost without lowering the verification boundary.

## Ownership

- `.github/workflows/rust.yml` owns root-crate Rust format, default tests, doctests, clippy, DfCM integration coverage, crypto-trust feature closure, performance budget, and macOS Secure Enclave courts.
- `affidavit-core.yml` owns the zero-dependency/no_std verifier.
- `affidavit-wasm.yml` owns the compiled WASI module and real-runtime parity tests, including AuthZEN/SPIFFE evidence operations.
- `web.yml`, `aloop.yml`, and `confevo.yml` remain path-scoped specialist courts.

## Control-plane rules

1. Branch pushes do not duplicate pull-request runs. Push-triggered verification is scoped to `main`; PRs verify the exact PR head.
2. PR runs may restore Cargo caches but only `main` saves them.
3. `cargo test --all-targets` is the root compile-and-test pass; a separate `cargo build --all-targets` would duplicate compilation.
4. Crypto dependencies are optimized in the dev/test profile while first-party Rust keeps normal development checks.
5. Superseded PR commits may be cancelled. Main/scheduled executions are not cancelled.
6. The former standalone `dfcm.yml` and `crypto-trust.yml` duplicated courts now executed by `rust.yml`, so they are retired rather than run twice.

## Projection drift

The historical `ggen sync run` drift step was not an executable court on hosted runners because the repository does not independently admit the required `ggen` binary plus sibling `clap-noun-verb` and `ggen-marketplace` subjects. It is therefore not represented as green CI. Generated trust-plane modules remain exercised by Rust, KAT, and WASM courts; projection replay belongs to the ecosystem composition lane where those exact dependencies are available.

## Standing

A workflow success certifies only its named exact subject and court. It does not transfer policy, authorization, deployment, release, or consequential DO authority.
