# Reference: Numeric Error Codes

**Classification:** Machine Code Reference  
**Scope:** `src/wire_ctrt.rs`, `src/replay_nullifier.rs`, `affidavit-wasm/src/ffi.rs`  

---

## 1. Error Code Catalog

| Hex Code | Identifier | Description / Root Cause |
| :--- | :--- | :--- |
| `0x0000` | `SUCCESS` | Operation completed normally. |
| `0xE001` | `ERR_NULL_POINTER` | Null pointer passed across C-ABI boundary. |
| `0xE002` | `ERR_BUFFER_OVERFLOW` | Payload length exceeds allocated buffer size. |
| `0xE003` | `ERR_BAD_MAGIC` | Wire format header does not match ASCII "CTRT". |
| `0xE004` | `ERR_UNSUPPORTED_VERSION` | Format version is unsupported. |
| `0xE005` | `ERR_DIGEST_MISMATCH` | Computed payload or wire digest does not match commitment. |
| `0xE006` | `ERR_INVALID_MODALITY` | Deontic modality byte outside valid enum range (0x01..=0x03). |
| `0xE007` | `ERR_CORRUPT_NULLIFIER` | Nullifier format corrupted or ledger truncated. |
| `0xE008` | `ERR_SPOLIATION_DETECTED` | Historical sequence gap detected in nullifier registry. |
| `0xE999` | `ERR_UNHANDLED_TRAP` | Panic caught at C-ABI boundary; failed closed. |
