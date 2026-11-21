# AGENTS.md — `micro_verifier`

Embeddable AOD Historical Witness FFI. Preserve these invariants:

1. **No local crypto.** Do not add `blake3`, `ml-dsa`, `slh-dsa`, `ml-kem`,
   `openssl`, `ring`, `git2`, or any hash/sign crate. Authority is
   `affidavit-core` only.
2. **Historical Witness only.** Never export Live Gate / Region 3 mapping /
   lease burn across the FFI.
3. **`unsafe` only in `src/ffi.rs`.** Everything else stays `deny(unsafe_code)`.
4. **ABI stability.** Bump `ABI_VERSION` and `AF_AOD_ABI_VERSION` together on
   any layout or semantics change; keep `AfAodWitnessAttestation` at 152 bytes
   unless the version bumps.
5. **NIF stub is documentation.** `nif/` is not a Cargo member. Do not pull
   `rustler` into this crate's default validate path.

Validate:

```bash
cd micro_verifier
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```
