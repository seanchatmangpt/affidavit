# affidavit-micro-verifier

Embeddable **Historical Witness** surface for Zero-String Sealed `.aod`
artifacts (AFFI-SPEC-26.11.19 PR-8).

```
.host (BEAM / WASM / C)
        │
        ▼
  af_aod_witness_verify   ← this crate (thin FFI)
        │
        ▼
  affidavit_core::aod::verify_witness   ← authority
```

## What it is

| Item | Value |
|------|-------|
| Mode | Historical Witness only (envelope; **zero Region 3**) |
| ABI | C (`include/affidavit_aod.h`), revision `1` |
| NIF | Documented stub under [`nif/`](nif/) |
| Crypto | **None local** — delegates to `affidavit-core` |

Live Gate (`map_live`) is intentionally **not** exported. Auditors certify
fossils; they do not burn leases or map data pages.

## Build & test

```bash
cd micro_verifier
cargo test
cargo build --release
# artifacts: target/release/libaffidavit_micro_verifier.{a,so,dylib}
```

Header for foreign loaders: [`include/affidavit_aod.h`](include/affidavit_aod.h).

## BEAM

Prefer Wasmex + `affidavit-wasm` for sandboxed mounting
(`docs/tutorials/02_wasm_beam_mounting.md`). For a dirty-NIF path, copy
[`nif/`](nif/) into the host Mix app and link this cdylib — do **not** add
`blake3` / `ml-dsa` / `git2` to the host.

Ecosystem fence: [`docs/AOD_ECOSYSTEM.md`](../docs/AOD_ECOSYSTEM.md).

## AGENTS

See [`AGENTS.md`](AGENTS.md).
