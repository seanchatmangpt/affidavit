# AGENTS.md — `affidavit-wasm`

Invariants (each has a test that fails if you break it):

1. **Byte parity with `affi`.** `src/receipt.rs` is a port of the root crate's
   `canonical_bytes`, `recompute_chain` and `verify`. Do not "improve" wording,
   stage order, or hashing. Parity is proven by `tests/fixtures/*` (made by the
   real `affi`) via `tests/wasm_abi.rs`.
2. **`unsafe` lives only in `src/ffi.rs`** (wasm32 only). Everything else is
   `#![deny(unsafe_code)]` and native-testable. New behavior goes in `abi.rs`.
3. **The ABI is total**: `abi::call` must never panic. Failures are
   `{"ok":false,"error":{code,message}}`. Bump `ABI_VERSION` on any incompatible
   request/response change.
4. **Do not enable `serde_json/preserve_order`** — canonical JSON depends on
   sorted maps (`receipt::tests::canonical_json_…`).
5. **Version = `affi`'s version.** The genesis seed derives from this crate's
   version; the root `tests/release_identity.rs` enforces equality. On a bump,
   regenerate the fixtures (procedure in `docs/WASM.md`).
6. **Imports stay WASI-only** and buffers stay reclaimed (both tested).
7. Mining is `affidavit-core` used as a library; don't re-implement it here.

Validate loop (run all): `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo clippy --lib --target wasm32-wasip1 -- -D warnings`, `cargo test`,
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
