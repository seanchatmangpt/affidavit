# Gatekeeper `verify_witness` WASM packaging inventory

**CONSTRUCT lane · 2026-11-21 · measured pin · no commits · Affidavit evidence 9dc09b5a-6434-4d8e-acb3-0d1c005dfac6**

Product claim under prosecution (C4 / GTM / 2027 research plan): an air-gapped
**&lt; 250 KB** WASM **gatekeeper** verifier whose witness check is Affidavit
`verify_witness`, offline, zero PII.

Byte sizes below are **observed** from `stat` / `shasum` after
`./scripts/gatekeeper_wasm_build.sh` — never invented.

---

## Verdict

| Claim | Disk status |
|---|---|
| Native `verify_witness` (Ed25519 `verify_strict`) | **ALIVE** — `src/ed25519_witness.rs`, feature `ed25519` / `crypto-trust` tests |
| Named WASM twin (`verify_witness` export) | **ALIVE** — `affidavit-gatekeeper-wasm` cdylib export `verify_witness` |
| Standalone / gatekeeper-pack WASM ≤ 250 KiB | **ALIVE** — size court fail-closed ≤ 256000 |
| Pinned slim gatekeeper WASM size | **28673 bytes** (28.00 KiB) — **under** bound by **227327** bytes |
| Fat provenance WASI module (untouched) | **718736 bytes** — still the only productized full ABI; **not** slimmed |

---

## 1. Native-only surface (`verify_witness`)

| Item | Path / fact |
|---|---|
| API | `affidavit::ed25519_witness::verify_witness(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64])` |
| Source | `src/ed25519_witness.rs` (module gated `#[cfg(feature = "ed25519")]` in `src/lib.rs`) |
| Law | dalek `VerifyingKey::verify_strict` (parity with wasm Ed25519 lane) |
| Courts | `tests/ed25519_strict_parity.rs`, `tests/ed25519_strict_vectors.rs` |

---

## 2. Slim gatekeeper crate (this deliverable)

Dedicated standalone workspace root — **does not** link or rebuild `affidavit-wasm`.

| Item | Path / fact |
|---|---|
| Crate | `affidavit-gatekeeper-wasm/` |
| Target | `wasm32-unknown-unknown` (browser / air-gapped; **not** WASI) |
| Law | dalek `verify_strict` via `verify_witness_strict` + C export `verify_witness` |
| Profile | `[profile.wasm]` — `opt-level = "z"`, LTO, strip, `panic = "abort"`, `trim-paths` |
| Build | `just gatekeeper-wasm-build` → `./scripts/gatekeeper_wasm_build.sh` |
| Court | `just gatekeeper-wasm-size-court` → `tests/size_court.rs` (≤ 256000) |
| CI | `.github/workflows/affidavit-gatekeeper-wasm.yml` |
| Pin | `affidavit-gatekeeper-wasm/registry/ARTIFACTS.sha256` |

### Recipe (copy-paste)

```bash
# Preferred (repo root) — build, wasm-opt -Oz, pin, refuse if >256000
just gatekeeper-wasm-build
just gatekeeper-wasm-size-court

# Manual equivalent
cd affidavit-gatekeeper-wasm
rustup target add wasm32-unknown-unknown
cargo build --locked --lib --target wasm32-unknown-unknown --profile wasm
wasm-opt -Oz --enable-bulk-memory \
  -o dist/affidavit_gatekeeper_verify_witness.wasm \
  target/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm
# Fat affidavit-wasm (718736) is never rebuilt here.
```

### Measured artifact (pinned)

| Source | Bytes | SHA-256 |
|---|---:|---|
| `dist/affidavit_gatekeeper_verify_witness.wasm` (`wasm-opt -Oz`) | **28673** | `5faae4642ca23c503e96d2a2ab938c056caffdde14e77516054fe4492aed0c97` |
| cargo `…/wasm/affidavit_gatekeeper_wasm.wasm` (pre-opt) | **35509** | (not the pin; pin is the dist/opt artifact) |
| Court bound | **256000** | — |
| Bound − pin | **227327** | under |

### Exports

| Symbol | Role |
|---|---|
| `verify_witness` | C ABI `(pk*32, msg*, msg_len, sig*64) -> i32` |
| `gatekeeper_abi_version` | `u32` (`1`) |

Return codes: `0` ok; `-1` malformed key; `-3` verify failed; `-4` null ptr.

---

## 3. Fat provenance module (unchanged)

`affidavit-wasm` remains the WASI provenance ABI at **718736** bytes
(`registry/ARTIFACTS.sha256` / `dist/affidavit_wasm.wasm`). This CONSTRUCT lane
does **not** slim that module. Closest twin inside it is still op
`verify_signature` / `alg=Ed25519` (`verify_strict`), not named `verify_witness`.

---

## 4. Parallel scaffold (removed)

A concurrent sketch `affidavit-gatekeeper/` (`af_verify_witness`, ~48 KiB
pre-opt) was deleted after reconcile — canonical product path is only
**`affidavit-gatekeeper-wasm`** (recipes, CI, pin, wasm-opt dist). Root
`Cargo.toml` exclude lists that crate alone.

---

## 5. Evidence index

- `affidavit-gatekeeper-wasm/src/lib.rs` — `verify_witness` / `verify_witness_strict`
- `affidavit-gatekeeper-wasm/tests/size_court.rs` — ≤ 256000 fail-closed
- `affidavit-gatekeeper-wasm/registry/ARTIFACTS.sha256` — measured pin
- `affidavit-gatekeeper-wasm/dist/affidavit_gatekeeper_verify_witness.wasm` — 28673 B
- `scripts/gatekeeper_wasm_build.sh`, `just gatekeeper-wasm-build`
- `.github/workflows/affidavit-gatekeeper-wasm.yml`
- Fat SoT unchanged: `affidavit-wasm/registry/ARTIFACTS.sha256` (718736)

**Lane authority:** construct + measure + pin. No commit.
