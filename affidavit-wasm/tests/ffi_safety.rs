//! Adversarial FFI guards exercised through the real wasm module in wasmi.
mod common;
use common::Host;
use serde_json::Value;

fn read(h: &mut Host, packed: u64) -> Value {
    let (p, l) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
    let mut buf = vec![0u8; l as usize];
    h.memory.read(&h.store, p as usize, &mut buf).unwrap();
    h.free.call(&mut h.store, (p, l)).unwrap();
    serde_json::from_slice(&buf).expect("typed JSON response")
}

#[test]
fn null_zero_len_and_extreme_lengths_never_trap() {
    let mut h = Host::new();
    // free(null, any) is a no-op.
    h.free.call(&mut h.store, (0, 0)).unwrap();
    h.free.call(&mut h.store, (0, u32::MAX)).unwrap();
    // null ptr with any length -> missing_buffer.
    for len in [0u32, 5, u32::MAX] {
        let p = h.call.call(&mut h.store, (0, len)).unwrap();
        assert_eq!(read(&mut h, p)["error"]["code"], "missing_buffer");
    }
    // alloc(0) is non-null; call(ptr,0) is a typed bad_json, buffer consumed.
    let ptr = h.alloc.call(&mut h.store, 0).unwrap();
    assert_ne!(ptr, 0);
    let p = h.call.call(&mut h.store, (ptr, 0)).unwrap();
    assert_eq!(read(&mut h, p)["error"]["code"], "bad_json");
    // u32::MAX alloc refused; u32::MAX len on non-null ptr is typed too_large.
    assert_eq!(h.alloc.call(&mut h.store, u32::MAX).unwrap(), 0);
    let p = h.call.call(&mut h.store, (8, u32::MAX)).unwrap();
    assert_eq!(read(&mut h, p)["error"]["code"], "too_large");
    // Alloc/free churn of an unused buffer (never called) reclaims cleanly.
    let ptr = h.alloc.call(&mut h.store, 1024).unwrap();
    h.free.call(&mut h.store, (ptr, 1024)).unwrap();
    assert_eq!(h.call(serde_json::json!({"op":"capabilities"}))["ok"], true);
}
