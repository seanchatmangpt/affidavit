# AGENTS.md — `affidavit-wasm`

Invariants (each has a test that fails if you break it):

1. **Byte parity with `affi`.** `src/receipt.rs` is a port of the root crate's
   `canonical_bytes`, `recompute_chain` and `verify`. Do not "improve" wording,
   stage order, or hashing. Parity is proven by `tests/fixtures/*` (made by the
   real `affi`) via `tests/wasm_abi.rs`.
2. **`unsafe` lives only in `src/ffi.rs`**, which is **generated** (as are
   `src/abi_meta.rs`, `.cargo/config.toml`, `registry/{capability-registry,op-examples}.json`)
   by `ggen sync` from `../ontology/affi-wasm.ttl` via `rust-wasi-wasmex-pack`
   (successor of the deprecated `wasi-json-abi-pack`).
   Never hand-edit them; edit the ontology (ops, limits, error codes, ABI
   version, prefix) and re-render. Everything else is `#![deny(unsafe_code)]`
   and native-testable. New op behavior goes in `abi.rs`; new ops also need an
   ontology `wja:Op` (with `wja:exampleRequest`).
3. **The ABI is total**: `abi::call` must never panic. Failures are
   `{"ok":false,"error":{code,message}}`. Bump `ABI_VERSION` on any incompatible
   request/response change.
4. **Do not enable `serde_json/preserve_order`** — canonical JSON depends on
   sorted maps (`receipt::tests::canonical_json_…`).
5. **Version = `affi`'s version.** The genesis seed derives from this crate's
   version; the root `tests/release_identity.rs` enforces equality. On a bump,
   regenerate the fixtures (procedure in `docs/WASM.md`).
6. **Imports stay WASI-only** and buffers stay reclaimed (both tested).
   Limits are typed, never traps: `af_alloc` returns null over
   `MAX_REQUEST_BYTES`; `too_large` / `too_deep` / `missing_buffer` errors.
   Every emitted error code must be in `abi_meta::ERROR_CODES`.
8. **Pinned artifact.** `registry/ARTIFACTS.sha256` pins the built module
   (`--locked`, `trim-paths`, pinned toolchain). Any `src/` or `Cargo.lock`
   change requires: build twice into separate target dirs, `cmp`, update the pin
   (procedure in `docs/WASM.md`); `tests/registry_artifacts.rs` enforces it under
   `AFFIDAVIT_WASM`.
9. **Differential court.** `tests/op_differential.rs` runs every
   `registry/op-examples.json` request natively and in wasmi and demands
   byte-identical answers.
7. Mining is `affidavit-core` used as a library; don't re-implement it here.

Validate loop (run all): `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo clippy --lib --target wasm32-wasip1 -- -D warnings`, `cargo test`,
`cargo build --lib --target wasm32-wasip1 --profile wasm`,
`AFFIDAVIT_WASM=target/wasm32-wasip1/wasm/affidavit_wasm.wasm cargo test --test wasm_abi --test op_differential --test registry_artifacts`,
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
