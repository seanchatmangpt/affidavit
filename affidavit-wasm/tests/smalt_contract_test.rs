//! Cross-repo contract conformance: the exact sequences smalt's Elixir fences
//! (wave v26.10.16, lanes W7/W8) will assert against the same artifact, run
//! here on the affidavit-wasm side so both repos prove the same contract.
//!
//! Sources of truth:
//! - JSON ABI + memory protocol: affidavit-wasm `src/ffi.rs` (ABI version 1)
//! - C-ABI gate: `affi_admit_and_seal` (ffi.rs, 352-byte CTRT token)
//! - smalt host contract: `lib/smalt/affidavit/wasm_engine.ex` (fail-closed;
//!   af_alloc/af_call/af_free; packed = (out_ptr << 32) | out_len)
#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::Host;
use serde_json::{json, Value};
use wasmi::{Engine, Instance, Linker, Module, Store};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

/// Handle to the raw C-ABI gate `affi_admit_and_seal`, which the shared
/// `common::Host` does not expose. Re-instantiates the same artifact in wasmi.
struct RawGate {
    store: Store<WasiCtx>,
    memory: wasmi::Memory,
    gate: wasmi::TypedFunc<(i32, i32, i64, i32, i32), u32>,
}

impl RawGate {
    fn new() -> Self {
        let bytes = std::fs::read(common::wasm_path()).expect("wasm artifact");
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes[..]).expect("valid wasm");
        let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
        let mut linker = <Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).expect("links WASI");
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiates");
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        let memory = instance.get_memory(&store, "memory").expect("exports memory");
        let gate = instance
            .get_typed_func::<(i32, i32, i64, i32, i32), u32>(&store, "affi_admit_and_seal")
            .expect("exports affi_admit_and_seal");
        RawGate {
            store,
            memory,
            gate,
        }
    }

    /// Write `delta` into module memory, run the gate, return (code, ctrt 352B).
    fn admit_and_seal(&mut self, delta: &[u8], tariff_cents: u64, modality: u8) -> (u32, [u8; 352]) {
        let delta_ptr = self.alloc_scratch(delta.len());
        self.memory
            .write(&mut self.store, delta_ptr, delta)
            .expect("writes delta");
        let out_ptr = self.alloc_scratch(352);
        let code = self
            .gate
            .call(
                &mut self.store,
                (
                    delta_ptr as i32,
                    delta.len() as i32,
                    tariff_cents as i64,
                    modality as i32,
                    out_ptr as i32,
                ),
            )
            .expect("gate call does not trap");
        let mut ctrt = [0u8; 352];
        self.memory
            .read(&self.store, out_ptr, &mut ctrt)
            .expect("reads CTRT");
        (code, ctrt)
    }

    /// Reserve scratch at the tail of linear memory: grow by one page (fresh
    /// pages are zeroed, so they are disjoint from anything the module's heap
    /// has handed out) and return the byte offset of the last `len` bytes.
    fn alloc_scratch(&mut self, len: usize) -> usize {
        self.memory
            .grow(&mut self.store, 1)
            .expect("grows memory");
        self.memory.data(&self.store).len() - len
    }
}

fn assemble_request() -> Value {
    json!({
        "op": "assemble",
        "events": [{
            "event_type": "smalt.contract.witness",
            "payload_hex": "00ff",
            "objects": ["smalt:witness:v1"]
        }]
    })
}

/// W7 happy path: ABI version handshake + assemble -> verify round trip over
/// the JSON ABI, exactly the sequence smalt's host drives through
/// af_alloc/af_call/af_free.
#[test]
fn json_happy_path_assemble_then_verify() {
    let mut host = Host::new();
    assert_eq!(host.abi_version.call(&mut host.store, ()).unwrap(), 1);

    let resp = host.call(assemble_request());
    assert_eq!(resp["ok"], Value::Bool(true), "assemble: {resp}");
    let receipt = resp["receipt"].clone();
    assert!(receipt.is_object(), "receipt object in {resp}");
    assert!(
        resp["content_address"].is_string() && resp["chain_hash"].is_string(),
        "content_address + chain_hash strings in {resp}"
    );

    let vresp = host.call(json!({"op": "verify", "receipt": receipt}));
    assert_eq!(vresp["ok"], Value::Bool(true), "verify: {vresp}");
    assert_eq!(vresp["accepted"], Value::Bool(true), "verify accepted: {vresp}");
    assert_eq!(vresp["profile"], json!("CoreV1"), "verify profile: {vresp}");
}

/// W8 forge rejection: tampered receipts are rejected with ok=true +
/// accepted=false (typed verdict, never a trap), and malformed requests get
/// typed error responses.
#[test]
fn forge_rejections_are_typed_not_traps() {
    let mut host = Host::new();

    let resp = host.call(json!({"op": "verify", "receipt": {"forged": true}}));
    assert_eq!(resp["ok"], Value::Bool(true), "verify responds: {resp}");
    assert_eq!(
        resp["accepted"], Value::Bool(false),
        "forged receipt rejected: {resp}"
    );

    let bad = host.call_raw(b"{not json");
    let parsed: Value = serde_json::from_slice(&bad).expect("typed error JSON");
    assert_eq!(parsed["ok"], Value::Bool(false), "bad_json: {parsed}");
    assert_eq!(parsed["error"]["code"], json!("bad_json"), "error code: {parsed}");

    let unk = host.call(json!({"op": "no_such_op"}));
    assert_eq!(unk["ok"], Value::Bool(false), "unknown op: {unk}");
    assert_eq!(unk["error"]["code"], json!("unknown_op"), "error code: {unk}");
}

/// C-ABI happy path: affi_admit_and_seal returns 0x0000 and seals the exact
/// 352-byte CTRT layout ffi.rs pins (magic, version, delta hash, ACCEPT,
/// GAIN_5050, tariff LE, modality byte, nullifier).
#[test]
fn ctrt_happy_path_seals_expected_layout() {
    let mut gate = RawGate::new();
    let delta = b"smalt state delta v1";
    let (code, ctrt) = gate.admit_and_seal(delta, 4242, 3);

    assert_eq!(code, 0x0000, "gate success code");
    assert_eq!(&ctrt[0..4], b"CTRT", "magic");
    assert_eq!(&ctrt[4..6], &1u16.to_le_bytes(), "version 1 LE");
    let expect_hash = blake3::hash(delta);
    assert_eq!(&ctrt[70..102], &expect_hash.as_bytes()[..], "delta hash");
    assert_eq!(ctrt[150], 1, "ACCEPT byte");
    assert_eq!(&ctrt[151..167], b"GAIN_5050\0\0\0\0\0\0\0", "gain label");
    assert_eq!(&ctrt[167..175], &4242u64.to_le_bytes(), "tariff LE u64");
    assert_eq!(ctrt[175], 3, "deontic modality byte");
    let expect_null = blake3::hash(expect_hash.as_bytes());
    assert_eq!(&ctrt[176..208], &expect_null.as_bytes()[..], "nullifier");
}

/// C-ABI forge rejection: null pointers fail closed with 0xE001 and the gate
/// never traps and never writes a token.
#[test]
fn ctrt_rejects_null_pointers_fail_closed() {
    let mut gate = RawGate::new();

    let null_out = gate.gate.call(
        &mut gate.store,
        (0i32, 4i32, 0i64, 0i32, 0i32),
    );
    assert!(null_out.is_ok(), "null pointers must not trap");
    assert_eq!(null_out.unwrap(), 0xE001, "ERR_NULL_POINTER");

    // Real delta pointer, null out pointer.
    let ptr = gate.alloc_scratch(16);
    let code = gate
        .gate
        .call(&mut gate.store, (ptr as i32, 16, 0, 0, 0))
        .expect("no trap");
    assert_eq!(code, 0xE001, "null out pointer fails closed");
}
