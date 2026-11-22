# affidavit-gatekeeper-wasm

Slim **verify_witness-only** WebAssembly cdylib for the air-gapped gatekeeper
claim. Links **Ed25519 `verify_strict` only** — not the fat WASI provenance
module in `../affidavit-wasm`.

## Build

```bash
cargo build --lib --target wasm32-unknown-unknown --profile wasm
# -> target/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm
```

Optional size pass (does not change the fat module):

```bash
wasm-opt -Oz --enable-bulk-memory -o dist/affidavit_gatekeeper_verify_witness.wasm \
  target/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm
```

Root recipes: `just gatekeeper-wasm-build`, `just gatekeeper-wasm-size-court`,
`just gatekeeper-wasm-test`.


## Exports

| Export | Signature | Meaning |
|---|---|---|
| `verify_witness` | `(pk*32, msg*, msg_len, sig*64) -> i32` | Strict Ed25519 check; `0` = ok |
| `gatekeeper_abi_version` | `() -> u32` | ABI revision (`1`) |

Return codes: `0` ok; `-1` malformed public key; `-3` verification failed; `-4` null ptr.

## Size court

Fail-closed bound: **≤ 256000** bytes (250 KiB). See
`tests/size_court.rs` and `docs/GATEKEEPER_VERIFY_WITNESS_WASM_INVENTORY.md`.

This crate does **not** re-pin or slim `affidavit-wasm` (718736 B WASI module).
