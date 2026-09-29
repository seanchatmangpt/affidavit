# affidavit-wasm

affidavit's provenance layer as a WebAssembly module: verify, assemble and mine
`core/v1` receipt chains through a JSON-over-linear-memory ABI, byte-compatible
with `affi`. Full guide: [`docs/WASM.md`](../docs/WASM.md).

```bash
cargo build --lib --target wasm32-wasip1 --profile wasm   # -> affidavit_wasm.wasm
cargo test                                                # unit + real wasm runtime
```

Standalone crate (own `[workspace]`), like `../affidavit-core`.
