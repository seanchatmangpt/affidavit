//! # affidavit-wasm
//!
//! affidavit's provenance layer as a WebAssembly module. A host (Elixir/Wasmex,
//! Node, Python, a browser, an edge runtime) can verify, assemble and mine
//! receipt chains without shelling out to `affi` — and gets **the same verdict
//! `affi verify` gives**, because the chain construction, canonical JSON and
//! seven-stage pipeline are a byte-for-byte port, held to real `affi` output by
//! a golden fixture (`tests/fixtures/`).
//!
//! All behaviour lives in the safe, natively testable [`abi`] module. The only
//! `unsafe` in this crate is the `ffi` module: the pointer handling needed to exchange
//! buffers with the host through linear memory, compiled for `wasm32` only.
//!
//! Protocol (integers are wasm `i32`/`i64`):
//! 1. `af_alloc(len) -> ptr` — host reserves `len` bytes and writes a UTF-8 JSON
//!    request there.
//! 2. `af_call(ptr, len) -> packed` — runs the request and consumes (frees) the
//!    request buffer. `packed = (out_ptr << 32) | out_len`.
//! 3. host reads `out_len` bytes at `out_ptr`, then `af_free(out_ptr, out_len)`.
//!
//! A response is always JSON: `{"ok":true,...}` or `{"ok":false,"error":{...}}`.
//! `af_abi_version() -> u32` reports the ABI revision (currently `1`).
//!
//! Doctrine, preserved: **certify, don't decide.** Nothing in this module
//! judges whether recorded work was honest.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod abi;
pub mod crypto;
mod external_evidence;
pub mod receipt;

#[cfg(target_arch = "wasm32")]
mod ffi;
