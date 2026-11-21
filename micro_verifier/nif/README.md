# BEAM NIF stub — AOD Historical Witness

**Status:** scaffold (AFFI-SPEC-26.11.19 PR-8)  
**Authority:** `affidavit-core::aod::verify_witness` via `affidavit-micro-verifier`  
**Do not** reimplement BLAKE3 / ML-DSA / git in Elixir or a sibling NIF.

## Preferred mounting paths

| Path | When | Notes |
|------|------|-------|
| **Wasmex + `affidavit-wasm`** | sandboxed, shared with Node/edge | see `docs/tutorials/02_wasm_beam_mounting.md` |
| **C ABI / Rustler NIF** (`af_aod_witness_verify`) | sub-µs Historical Witness of fossils | this directory |

Live Gate (`map_live`) is **not** exported here — auditors must not map Region 3.

## Build the host NIF

The Rustler surface is **not** a feature of this crate (keeps the affidavit
validate ladder free of Erlang headers). Copy `nif/src/lib.rs` into the host
Mix app's `native/` crate, depend on `affidavit-micro-verifier` as a path/git
dep, and link the prebuilt C ABI:

```bash
cd micro_verifier && cargo build --release
# → target/release/libaffidavit_micro_verifier.{a,so,dylib}
```

Or call through Wasmex + `affidavit-wasm` when sandboxing is preferred.

## Elixir module contract

```elixir
# {:ok, attestation_map} | {:error, code :: integer}
Affidavit.Aod.Witness.verify(aod_binary)
Affidavit.Aod.Witness.abi_version()  # => 1
```

Refusal codes are the positive `0xE501..=0xE50D` machine codes from
`AodRefusal` (or `-1` for null/empty input on the C path).

## Files

| File | Role |
|------|------|
| `Elixir.Affidavit.Aod.Witness.ex` | documented host module (load NIF or fall back) |
| `src/lib.rs` | copy-paste Rustler skeleton (mirrors `src/nif.rs` behind `feature = "nif"`) |

## Fence

Sibling BEAM apps (`ash_a2a`, `fnox`, `strata`/stratus) must **consume** this
ABI (or `affi aod witness` / wasm), not grow `blake3` / `ml-dsa` / `git2`
dependencies. See [`docs/AOD_ECOSYSTEM.md`](../../docs/AOD_ECOSYSTEM.md).
