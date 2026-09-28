//! The only `unsafe` in the crate: linear-memory buffer exchange with the host.
//!
//! Every buffer handed across the boundary is a `Box<[u8]>` of length
//! `len.max(1)`, so `af_free`/`af_call` can reconstruct the exact allocation.
#![allow(unsafe_code)]

fn into_raw(buf: Box<[u8]>) -> *mut u8 {
    Box::into_raw(buf) as *mut u8
}

/// # Safety
/// `ptr` must come from [`af_alloc`] / [`af_call`] and `len` must be the length
/// that was requested / returned; each pair may be reclaimed exactly once.
unsafe fn from_raw(ptr: *mut u8, len: u32) -> Box<[u8]> {
    let n = (len as usize).max(1);
    Box::from_raw(core::ptr::slice_from_raw_parts_mut(ptr, n))
}

/// ABI revision; see [`crate::abi::ABI_VERSION`].
#[no_mangle]
pub extern "C" fn af_abi_version() -> u32 {
    crate::abi::ABI_VERSION
}

/// Reserve `len` zeroed bytes of linear memory for the host.
#[no_mangle]
pub extern "C" fn af_alloc(len: u32) -> *mut u8 {
    into_raw(vec![0u8; (len as usize).max(1)].into_boxed_slice())
}

/// Release a buffer obtained from `af_alloc` or returned by `af_call`.
///
/// # Safety
/// `ptr`/`len` must be exactly a pair previously handed out by this module.
#[no_mangle]
pub unsafe extern "C" fn af_free(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        drop(from_raw(ptr, len));
    }
}

/// Execute one JSON request; returns `(out_ptr << 32) | out_len`.
///
/// # Safety
/// `ptr`/`len` must describe a buffer from `af_alloc` filled by the host.
#[no_mangle]
pub unsafe extern "C" fn af_call(ptr: *mut u8, len: u32) -> u64 {
    let request = from_raw(ptr, len);
    let response = crate::abi::call(&request[..len as usize]);
    drop(request);
    let out_len = response.len() as u64;
    let out_ptr = into_raw(response.into_boxed_slice()) as u64;
    (out_ptr << 32) | out_len
}
