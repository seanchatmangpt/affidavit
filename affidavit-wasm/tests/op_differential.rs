//! Differential court: every `registry/op-examples.json` request (the ontology's
//! example per op) answers **byte-identically** natively (rlib) and inside the
//! compiled wasm module, and no example is vacuous (each op is exercised).
//! Real wasmi, no mocks. Also pins forward-compat and typed limit behaviour.
#![cfg(not(target_arch = "wasm32"))]

mod common;

use affidavit_wasm::abi::call;
use affidavit_wasm::abi_meta::{ERROR_CODES, MAX_JSON_DEPTH, MAX_REQUEST_BYTES, OPS};
use common::Host;
use serde_json::{json, Value};

const EXAMPLES: &str = include_str!("../registry/op-examples.json");

fn examples() -> Vec<(String, Value)> {
    let doc: Value = serde_json::from_str(EXAMPLES).expect("op-examples.json parses");
    doc["examples"]
        .as_array()
        .expect("examples array")
        .iter()
        .map(|e| (e["op"].as_str().unwrap().to_string(), e["request"].clone()))
        .collect()
}

#[test]
fn every_op_has_exactly_one_example_and_it_dispatches_that_op() {
    let ex = examples();
    let names: Vec<&str> = ex.iter().map(|(o, _)| o.as_str()).collect();
    assert_eq!(names, OPS, "examples must cover OPS, in order");
    for (op, req) in &ex {
        assert_eq!(req["op"], json!(op), "example for {op} requests another op");
    }
}

#[test]
fn native_and_wasm_are_byte_identical_over_every_example() {
    let mut h = Host::new();
    for (op, req) in examples() {
        let raw = serde_json::to_vec(&req).unwrap();
        let wasm = h.call_raw(&raw);
        assert_eq!(wasm, call(&raw), "native vs wasm diverged on op `{op}`");
        let v: Value = serde_json::from_slice(&wasm).unwrap();
        assert_eq!(
            v["ok"], true,
            "example for `{op}` must be a working request: {v}"
        );
        assert_eq!(v["op"], json!(op));
    }
}

#[test]
fn forward_compat_unknown_fields_tolerated_unknown_op_typed() {
    let mut h = Host::new();
    let base = json!({"op": "commit", "payload": "x"});
    let mut extended = base.clone();
    extended["from_the_future"] = json!({"nested": [1, 2, 3]});
    let (a, b) = (h.call(base), h.call(extended));
    assert_eq!(a, b, "an unknown extra request field changed the answer");

    let r = h.call(json!({"op": "op_from_the_future"}));
    assert_eq!(r["ok"], false);
    assert_eq!(r["error"]["code"], "unknown_op");
}

#[test]
fn limits_are_typed_in_wasm_and_never_trap() {
    let mut h = Host::new();

    // Depth: refused with a typed error, identical to native.
    let deep = format!(
        "{}{}",
        "[".repeat(MAX_JSON_DEPTH + 1),
        "]".repeat(MAX_JSON_DEPTH + 1)
    );
    let out = h.call_raw(deep.as_bytes());
    assert_eq!(out, call(deep.as_bytes()));
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(r["error"]["code"], "too_deep");
    assert_eq!(r["error"]["limit"], "json_depth");
    assert_eq!(r["error"]["max"], MAX_JSON_DEPTH);

    // alloc over the request limit returns null (0), and calling on that null
    // buffer yields a typed response instead of a trap.
    let ptr = h
        .alloc
        .call(&mut h.store, (MAX_REQUEST_BYTES as u32) + 1)
        .unwrap();
    assert_eq!(ptr, 0, "alloc over the limit must return null");
    let packed = h.call.call(&mut h.store, (0, 0)).unwrap();
    let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
    let mut buf = vec![0u8; out_len as usize];
    h.memory.read(&h.store, out_ptr as usize, &mut buf).unwrap();
    h.free.call(&mut h.store, (out_ptr, out_len)).unwrap();
    let r: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(r["error"]["code"], "missing_buffer");

    // An oversize length on any pointer is refused typed, buffer untouched.
    let packed = h
        .call
        .call(&mut h.store, (8, (MAX_REQUEST_BYTES as u32) + 1))
        .unwrap();
    let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
    let mut buf = vec![0u8; out_len as usize];
    h.memory.read(&h.store, out_ptr as usize, &mut buf).unwrap();
    h.free.call(&mut h.store, (out_ptr, out_len)).unwrap();
    let r: Value = serde_json::from_slice(&buf).unwrap();
    assert_eq!(r["error"]["code"], "too_large");
    assert_eq!(r["error"]["observed"], MAX_REQUEST_BYTES + 1);

    // Still healthy afterwards.
    assert_eq!(h.call(json!({"op": "capabilities"}))["ok"], true);
}

#[test]
fn every_error_code_the_module_emits_is_declared() {
    let mut h = Host::new();
    for raw in [
        &b"junk"[..],
        b"{}",
        br#"{"op":"nope"}"#,
        br#"{"op":"verify"}"#,
        br#"{"op":"assemble","events":[{"event_type":"x"}]}"#,
    ] {
        let r: Value = serde_json::from_slice(&h.call_raw(raw)).unwrap();
        let code = r["error"]["code"].as_str().unwrap();
        assert!(ERROR_CODES.contains(&code), "undeclared code `{code}`");
    }
}
