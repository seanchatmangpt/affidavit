# affidavit as WebAssembly (v26.9.28)

`affidavit-wasm/` builds the provenance layer's verifier as a sandboxed WASI
module. Hosts (Elixir/Wasmex, Node, Python, edge runtimes) verify, assemble and
mine receipt chains **without shelling out to `affi`** and get the verdict `affi
verify` gives — same canonical JSON, same rolling BLAKE3 chain, same seven
stages, same wording. The design mirrors graphlaw's `graphlaw-wasm`: a tiny
`unsafe` FFI shell over a safe, natively testable JSON ABI.

> Doctrine, preserved: **certify, don't decide.** The module checks a receipt
> against the format standard. It never judges whether the work was honest.

## Build

```bash
cd affidavit-wasm
rustup target add wasm32-wasip1        # once, on the toolchain rust-toolchain.toml pins
cargo build --lib --target wasm32-wasip1 --profile wasm
# -> target/wasm32-wasip1/wasm/affidavit_wasm.wasm   (~230 KB)
just wasm-test                          # build + drive it in a real wasm runtime
```

Imports are `wasi_snapshot_preview1` only. `wasm32-unknown-unknown` is not a
supported target (WASI hosts only). Call `_initialize` once if your host does
not.

## ABI

| Export | Signature | Meaning |
|---|---|---|
| `af_abi_version` | `() -> u32` | ABI revision (currently `1`). |
| `af_alloc` | `(len: u32) -> ptr` | Reserve `len` zeroed bytes for a request. Returns **null (0)** when `len` exceeds the request limit (16 MiB). |
| `af_call` | `(ptr: u32, len: u32) -> u64` | Run the request; **consumes** the request buffer. Returns `(out_ptr << 32) \| out_len`. |
| `af_free` | `(ptr: u32, len: u32)` | Release a response buffer (or an unused request buffer). |
| `memory` | | Linear memory. |

Round trip: `alloc` → write UTF-8 JSON → `call` → read `out_len` bytes at
`out_ptr` → `free(out_ptr, out_len)`. A null request pointer or an oversize
length never traps: `af_call` answers with a typed error. Every response is JSON:
`{"ok":true,"op":…,…}` or `{"ok":false,"error":{"code","message"}}`. The call
never panics on any input.

Error codes (`abi_meta::ERROR_CODES`): `bad_json`, `missing_field`,
`unknown_op`, `bad_field`, `too_large` (request over 16 MiB), `too_deep` (JSON
nesting over 64), `missing_buffer` (`af_call` on a null buffer), `internal`.
Limit failures carry `error.limit`, `error.observed`, `error.max`.

Unknown extra request fields are tolerated (forward compatibility); an unknown
`op` is a typed `unknown_op`.

## Generated vs hand-written

The ABI shell is **manufactured**, not written: `ontology/affi-wasm.ttl` (an
instance of the project-neutral `wja:` vocabulary of
`ggen-marketplace/packs/wasi-json-abi-pack`) is rendered by `ggen sync` into

| Output | Content |
|---|---|
| `affidavit-wasm/src/ffi.rs` | the only `unsafe`: `af_abi_version/_alloc/_free/_call`, null/oversize guards |
| `affidavit-wasm/src/abi_meta.rs` | `ABI_VERSION`, `MAX_REQUEST_BYTES`, `MAX_JSON_DEPTH`, `OPS`, `ERROR_CODES` |
| `affidavit-wasm/.cargo/config.toml` | 16 MiB stack for the wasm targets |
| `affidavit-wasm/registry/capability-registry.json`, `op-examples.json` | machine-readable capability registry and one example request per op |
| `affidavit-wasm/registry/ARTIFACTS.sha256` | create-only pin of the built module (`wasm <sha256> <bytes> <name>`) |

Never edit these by hand: change the ontology and re-render (CI runs a drift
court: `ggen sync run` then `git diff --exit-code`). The op bodies
(`abi.rs`), `receipt.rs` and `crypto.rs` stay hand-written (`HANDWRITTEN.md`).

**Re-pin after any change to `src/` or `Cargo.lock`:**
`cargo build --locked --lib --target wasm32-wasip1 --profile wasm`, build again
with `--target-dir <other>` and `cmp` the two (they must be identical; the
profile sets `trim-paths` so embedded paths are checkout-independent), then put
`sha256`/size of the module on the `wasm` line of `registry/ARTIFACTS.sha256`.
`AFFIDAVIT_WASM=<module> cargo test --test registry_artifacts` enforces the pin.

## Ops

| Op | Request | Response (besides `ok`, `op`) |
|---|---|---|
| `capabilities` | `{}` | `version`, `abi_version`, `format_version` (`core/v1`), `genesis_seed`, `hash`, `ops`, `limits` (`max_request_bytes`, `max_json_depth`), `error_codes` |
| `commit` | `{payload}` or `{payload_hex}` | `commitment` (BLAKE3 hex), `bytes` |
| `assemble` | `{events:[{event_type, payload\|payload_hex\|payload_commitment, id?, objects?}]}` | `receipt`, `content_address`, `chain_hash` |
| `verify` | `{receipt}` | `accepted`, `profile`, `outcomes[6]` (`stage`,`passed`,`detail`), `reason`, `content_address` |
| `mine` | `{receipts:[…]}` | `activities`, `edges`, `start`, `end`, `activity_frequency`, `variants`, `footprint`, `trace_count`, `event_count`, `all_accepted`, `rejected_receipts` |
| `verify_signature_input` | `{envelope_json, expected_signing_input_hex}` | `verified`, `signing_input_hex` (envelope signing pre-image binding; no signature arithmetic) |
| `conform` | `{model:[receipts], trace: receipt}` | `verdict`, `fitness`, `legal_moves`, `total_moves`, `start_ok`, `end_ok`, `first_violation`, `trace_accepted`, `model_all_accepted` |

Notes:

- `assemble` assigns `seq` by position and defaults `id` to `evt-<i>`. `objects`
  accepts the CLI's `id:type[:qualifier]` strings or `{id,obj_type,qualifier}`.
  Given the same bytes it reproduces `affi receipt assemble` exactly.
- `verify` returns a **REJECT verdict**, not an ABI error, for a receipt that
  cannot be decoded (stage `decode`) — a verifier answers "no"; it doesn't throw.
  `profile` is spelled `CoreV1`, as in `affi verify --format json`.
- `mine`/`conform` treat each receipt as one trace of `event_type`s in order and
  report whether each receipt was itself accepted; they never silently drop or
  "repair" one. `footprint` is omitted (`footprint_omitted: true`) above 256
  activities, since it is quadratic.

## Host walkthrough (Node, WASI)

```js
import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";

const wasi = new WASI({ version: "preview1" });
const { instance } = await WebAssembly.instantiate(
  readFileSync("affidavit.wasm"),
  wasi.getImportObject(),
);
wasi.initialize(instance); // calls _initialize
const { af_alloc, af_call, af_free, memory } = instance.exports;

function call(request) {
  const body = new TextEncoder().encode(JSON.stringify(request));
  const ptr = af_alloc(body.length);
  new Uint8Array(memory.buffer, ptr, body.length).set(body);
  const packed = af_call(ptr, body.length); // BigInt
  const outPtr = Number(packed >> 32n), outLen = Number(packed & 0xffffffffn);
  const out = new TextDecoder().decode(new Uint8Array(memory.buffer, outPtr, outLen));
  af_free(outPtr, outLen);
  return JSON.parse(out);
}

console.log(call({ op: "verify", receipt: JSON.parse(readFileSync("receipt.json")) }).reason);
```

Re-create views over `memory.buffer` after every call that can allocate: the
buffer detaches when linear memory grows.

## What "parity with `affi`" means here

`affidavit-wasm/tests/fixtures/golden_receipt.json` was produced by the real
`affi` binary (`affi receipt emit` ×3 → `assemble`). The tests assert, in a real
wasm runtime (wasmi), that the module (1) ACCEPTs it, (2) REJECTs a tampered copy
with the exact reason `affi` prints, (3) reproduces it byte-for-byte via
`assemble`, and (4) answers identically native vs. wasm, over every op example in
`registry/op-examples.json` (`tests/op_differential.rs`, which also pins typed
limits and forward-compat), with the registry checked against the module's own
`capabilities` and the artifact pin verified (`tests/registry_artifacts.rs`). `tests/release_identity.rs`
in the root crate additionally requires the fixture to ACCEPT under the current
release's own verifier and the wasm crate's version to equal `affi`'s, so a
version bump cannot leave either behind.

**Regenerate the fixture on every version bump** (the genesis seed is
`affidavit-v<version>-genesis`): emit three events, assemble with the new `affi`,
copy to `golden_receipt.json`, and `sed 's/"test"/"tampered"/'` a copy to
`tampered_receipt.json`; then update the expected hashes in
`tests/wasm_abi.rs::rejects_a_tampered_receipt_with_the_exact_reason_affi_gives`.

## Trust-model boundaries (read before relying on it)

- **Version-bound.** A receipt only verifies under the exact release that
  assembled it; a 26.9.24 receipt REJECTs at `chain_integrity` here. That is the
  genesis seed working as designed, not a bug.
- **No signatures in the WASM surface.** Signatures exist in the root crate,
  feature-gated behind `crypto-trust`: ES256, ML-DSA-65, SLH-DSA-SHA2-128s,
  and the hybrid ES256+ML-DSA-65, verified by `VerificationEngine`
  (`affi envelope verify`) — CLI and library only. This wasm build depends on
  `affidavit-core` alone and contains no signature verification; `verify` here
  checks the chain and format exactly as before. The wasm ABI offers no
  signing and no signature verification.
- **Structural only.** Commitments are checked for well-formedness, never against
  payloads (the verifier "reads only commitments").
- **Sandboxed, not sealed.** Unlike the Rust `Receipt`, which cannot be forged by
  struct literal (`E0451`), a host can hand `verify` any JSON. Verification, not
  construction, is the guarantee across this boundary.
