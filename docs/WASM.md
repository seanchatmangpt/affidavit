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
| `af_alloc` | `(len: u32) -> ptr` | Reserve `len` zeroed bytes for a request. |
| `af_call` | `(ptr: u32, len: u32) -> u64` | Run the request; **consumes** the request buffer. Returns `(out_ptr << 32) \| out_len`. |
| `af_free` | `(ptr: u32, len: u32)` | Release a response buffer (or an unused request buffer). |
| `memory` | | Linear memory. |

Round trip: `alloc` → write UTF-8 JSON → `call` → read `out_len` bytes at
`out_ptr` → `free(out_ptr, out_len)`. Every response is JSON:
`{"ok":true,"op":…,…}` or `{"ok":false,"error":{"code","message"}}`. The call
never panics on any input.

Error codes: `bad_json`, `missing_field`, `unknown_op`, `bad_field`,
`too_large` (requests over 16 MiB), `internal`.

## Ops

| Op | Request | Response (besides `ok`, `op`) |
|---|---|---|
| `capabilities` | `{}` | `version`, `abi_version`, `format_version` (`core/v1`), `genesis_seed`, `hash`, `ops`, `limits` |
| `commit` | `{payload}` or `{payload_hex}` | `commitment` (BLAKE3 hex), `bytes` |
| `assemble` | `{events:[{event_type, payload\|payload_hex\|payload_commitment, id?, objects?}]}` | `receipt`, `content_address`, `chain_hash` |
| `verify` | `{receipt}` | `accepted`, `profile`, `outcomes[6]` (`stage`,`passed`,`detail`), `reason`, `content_address` |
| `mine` | `{receipts:[…]}` | `activities`, `edges`, `start`, `end`, `activity_frequency`, `variants`, `footprint`, `trace_count`, `event_count`, `all_accepted`, `rejected_receipts` |
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
`assemble`, and (4) answers identically native vs. wasm. `tests/release_identity.rs`
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
- **No signatures.** This release verifies the chain and format. It does not
  verify `affi sign`/notarization material; those remain CLI-only.
- **Structural only.** Commitments are checked for well-formedness, never against
  payloads (the verifier "reads only commitments").
- **Sandboxed, not sealed.** Unlike the Rust `Receipt`, which cannot be forged by
  struct literal (`E0451`), a host can hand `verify` any JSON. Verification, not
  construction, is the guarantee across this boundary.
