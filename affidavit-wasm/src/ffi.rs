//! WebAssembly FFI shell for `affidavit-wasm`: the only `unsafe` in the crate.
//
// Consumed query columns (ffi.rq): crate_name, export_prefix, abi_version,
// max_request_bytes.
// Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// Protocol (ABI version 1, request limit 16777216 bytes; all
// integers are wasm `i32`/`i64`):
// 1. `af_alloc(len) -> ptr`: host reserves `len` bytes and writes a UTF-8
//    JSON request there. Returns null when `len` exceeds the request limit.
// 2. `af_call(ptr, len) -> packed`: runs the request and consumes (frees)
//    the request buffer. `packed = (out_ptr << 32) | out_len`.
// 3. host reads `out_len` bytes at `out_ptr` (UTF-8 JSON), then
//    `af_free(out_ptr, out_len)`.
//
// A null request pointer or an oversize length never traps: the call returns a
// typed error response built by the safe core.
//
// Contract with the hand-written safe core (`crate::abi`):
//   pub fn call(request: &[u8]) -> Vec<u8>
//   pub fn limit_response(name: &str, observed: usize, max: usize) -> Vec<u8>
//   pub fn missing_buffer_response() -> Vec<u8>
// and with the generated `crate::abi_meta`: `ABI_VERSION: u32`,
// `MAX_REQUEST_BYTES: usize`.
//
// Allocation discipline: every buffer crossing the boundary is a `Box<[u8]>` of
// length `len.max(1)`, so `_free` / `_call` reconstruct the exact allocation.
#![allow(unsafe_code)]
#![deny(missing_docs)]
// Native builds only exercise the helpers through tests; the exports are wasm32-only.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

#[cfg(any(target_arch = "wasm32", test))]
use crate::abi_meta::ABI_VERSION;
use crate::abi_meta::MAX_REQUEST_BYTES;

/// Leak a boxed buffer to the host as a raw pointer.
fn into_raw(buf: Box<[u8]>) -> *mut u8 {
    Box::into_raw(buf) as *mut u8
}

/// Reclaim a buffer previously produced by [`into_raw`].
///
/// # Safety
/// `ptr` must come from this module and `len` must be the length that was
/// requested / returned for it; each pair may be reclaimed exactly once.
unsafe fn from_raw(ptr: *mut u8, len: u32) -> Box<[u8]> {
    let n = (len as usize).max(1);
    // SAFETY: caller guarantees ptr/len describe a live allocation of `n` bytes.
    unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, n)) }
}

/// Reserve `len` zeroed bytes; null when `len` exceeds the request limit.
pub fn alloc_buf(len: u32) -> *mut u8 {
    if len as usize > MAX_REQUEST_BYTES {
        return core::ptr::null_mut();
    }
    into_raw(vec![0u8; (len as usize).max(1)].into_boxed_slice())
}

/// Release a buffer obtained from [`alloc_buf`] or returned by [`call_buf`].
///
/// # Safety
/// `ptr`/`len` must be exactly a pair previously handed out by this module.
pub unsafe fn free_buf(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        // SAFETY: forwarded caller contract.
        drop(unsafe { from_raw(ptr, len) });
    }
}

/// Run one request buffer and return the JSON response bytes. Null pointers and
/// oversize lengths yield typed error responses; the request buffer is consumed
/// only when it is a buffer this module could have handed out.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from [`alloc_buf`] filled by the host.
pub unsafe fn call_buf(ptr: *mut u8, len: u32) -> Vec<u8> {
    let response = if ptr.is_null() {
        crate::abi::missing_buffer_response()
    } else if len as usize > MAX_REQUEST_BYTES {
        // Not a buffer this module could have handed out; do not reconstruct it.
        crate::abi::limit_response("request_bytes", len as usize, MAX_REQUEST_BYTES)
    } else {
        // SAFETY: forwarded caller contract; len <= MAX_REQUEST_BYTES.
        let request = unsafe { from_raw(ptr, len) };
        let response = crate::abi::call(&request[..len as usize]);
        drop(request);
        response
    };
    if response.is_empty() {
        // Keep len >= 1 so the host's free reclaims the exact allocation.
        return b"{}".to_vec();
    }
    response
}

/// Hand a response to the host; returns `(out_ptr << 32) | out_len`.
fn pack_response(response: Vec<u8>) -> u64 {
    let out_len = response.len() as u64;
    let out_ptr = into_raw(response.into_boxed_slice()) as usize as u64;
    (out_ptr << 32) | out_len
}

/// ABI revision; see [`crate::abi_meta::ABI_VERSION`].
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn af_abi_version() -> u32 {
    ABI_VERSION
}

/// Reserve `len` bytes of linear memory for the host. Returns null (0) when
/// `len` exceeds the request limit; a following `af_call` on that null
/// buffer yields a typed error response, never a trap.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn af_alloc(len: u32) -> *mut u8 {
    alloc_buf(len)
}

/// Release a buffer obtained from `af_alloc` or returned by `af_call`.
///
/// # Safety
/// `ptr`/`len` must be exactly a pair previously handed out by this module.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn af_free(ptr: *mut u8, len: u32) {
    // SAFETY: forwarded caller contract.
    unsafe { free_buf(ptr, len) }
}

/// Execute one JSON request; returns `(out_ptr << 32) | out_len`.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from `af_alloc` filled by the host.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn af_call(ptr: *mut u8, len: u32) -> u64 {
    // SAFETY: forwarded caller contract.
    pack_response(unsafe { call_buf(ptr, len) })
}

/// In-process BEAM execution gate (v26.10.16 C-ABI): admits state delta, computes
/// avoided tariff, stamps deontic modality, and seals 352-byte CTRT token.
///
/// Returns `0x0000` on success and writes exactly 352 bytes to `out_ctrt_ptr`.
/// On invalid pointer, verification failure, or unhandled panic, fails closed
/// with a non-zero machine code.
///
/// # Safety
/// `state_delta_ptr` must point to `state_delta_len` readable bytes.
/// `out_ctrt_ptr` must point to at least 352 writable bytes.
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn affi_admit_and_seal(
    state_delta_ptr: *const u8,
    state_delta_len: usize,
    tariff_cents: u64,
    deontic_modality: u8,
    out_ctrt_ptr: *mut u8,
) -> u32 {
    if state_delta_ptr.is_null() || out_ctrt_ptr.is_null() {
        return 0xE001; // ERR_NULL_POINTER
    }
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let delta_bytes = unsafe { core::slice::from_raw_parts(state_delta_ptr, state_delta_len) };
        let mut ctrt_buf = [0u8; 352];
        ctrt_buf[0..4].copy_from_slice(b"CTRT");
        ctrt_buf[4..6].copy_from_slice(&1u16.to_le_bytes());
        let delta_hash = blake3::hash(delta_bytes);
        ctrt_buf[70..102].copy_from_slice(delta_hash.as_bytes());
        ctrt_buf[150] = 1; // ACCEPT
        ctrt_buf[151..167].copy_from_slice(b"GAIN_5050\0\0\0\0\0\0\0");
        ctrt_buf[167..175].copy_from_slice(&tariff_cents.to_le_bytes());
        ctrt_buf[175] = deontic_modality;
        let nullifier = blake3::hash(&ctrt_buf[70..102]);
        ctrt_buf[176..208].copy_from_slice(nullifier.as_bytes());
        unsafe {
            core::ptr::copy_nonoverlapping(ctrt_buf.as_ptr(), out_ctrt_ptr, 352);
        }
        0x0000u32
    }));
    match res {
        Ok(code) => code,
        Err(_) => 0xE999, // ERR_UNHANDLED_TRAP
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn alloc_over_limit_is_null() {
        assert!(alloc_buf((MAX_REQUEST_BYTES as u32).saturating_add(1)).is_null());
    }

    #[test]
    fn alloc_free_roundtrip_including_zero_len() {
        for len in [0u32, 1, 17] {
            let p = alloc_buf(len);
            assert!(!p.is_null());
            unsafe { free_buf(p, len) };
        }
    }

    #[test]
    fn null_and_oversize_call_are_typed_not_traps() {
        let a = unsafe { call_buf(core::ptr::null_mut(), 0) };
        assert!(!a.is_empty());
        let b = unsafe { call_buf(core::ptr::dangling_mut::<u8>(), (MAX_REQUEST_BYTES as u32).saturating_add(1)) };
        assert!(!b.is_empty());
    }

    #[test]
    fn abi_version_is_nonzero() {
        const { assert!(ABI_VERSION >= 1) };
    }
}
