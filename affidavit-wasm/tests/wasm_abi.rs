//! Runs the *compiled* WebAssembly module in a real wasm runtime (wasmi) and
//! drives it through the JSON ABI exactly as a host such as Wasmex would. Set
//! `AFFIDAVIT_WASM` to test a prebuilt module; otherwise the module is built
//! once into `target/wasm-abi`.
//!
//! The fixtures under `tests/fixtures/` were produced by the real `affi` binary
//! (`affi receipt emit` x3 -> `assemble` -> `verify`), so passing here means the
//! wasm module agrees with `affi`, not merely with itself.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use serde_json::{json, Value};
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

const GOLDEN: &str = include_str!("fixtures/golden_receipt.json");
const TAMPERED: &str = include_str!("fixtures/tampered_receipt.json");

fn wasm_path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        if let Some(p) = std::env::var_os("AFFIDAVIT_WASM") {
            return p.into();
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target = root.join("target/wasm-abi");
        let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .current_dir(&root)
            .args([
                "build",
                "--lib",
                "--target",
                "wasm32-wasip1",
                "--profile",
                "wasm",
                "--target-dir",
            ])
            .arg(&target)
            .status()
            .expect("cargo runs");
        assert!(
            status.success(),
            "wasm build failed (is the wasm32-wasip1 target installed?)"
        );
        target.join("wasm32-wasip1/wasm/affidavit_wasm.wasm")
    })
}

struct Host {
    store: Store<WasiCtx>,
    memory: Memory,
    alloc: TypedFunc<u32, u32>,
    free: TypedFunc<(u32, u32), ()>,
    call: TypedFunc<(u32, u32), u64>,
    abi_version: TypedFunc<(), u32>,
}

impl Host {
    fn new() -> Self {
        let bytes = std::fs::read(wasm_path()).expect("wasm artifact");
        let engine = Engine::default();
        let module = Module::new(&engine, &bytes[..]).expect("valid wasm");
        // The only imports allowed are WASI's: no JavaScript glue, no custom host.
        for i in module.imports() {
            assert_eq!(
                i.module(),
                "wasi_snapshot_preview1",
                "unexpected host import {}::{}",
                i.module(),
                i.name()
            );
        }
        let mut store = Store::new(&engine, WasiCtxBuilder::new().build());
        let mut linker = <Linker<WasiCtx>>::new(&engine);
        wasmi_wasi::add_to_linker(&mut linker, |ctx| ctx).expect("links WASI");
        let instance: Instance = linker
            .instantiate_and_start(&mut store, &module)
            .expect("instantiates");
        // Reactor-style modules expose `_initialize`; hosts must call it once.
        if let Ok(init) = instance.get_typed_func::<(), ()>(&store, "_initialize") {
            init.call(&mut store, ()).expect("_initialize");
        }
        Host {
            memory: instance
                .get_memory(&store, "memory")
                .expect("exports memory"),
            alloc: instance.get_typed_func(&store, "af_alloc").unwrap(),
            free: instance.get_typed_func(&store, "af_free").unwrap(),
            call: instance.get_typed_func(&store, "af_call").unwrap(),
            abi_version: instance.get_typed_func(&store, "af_abi_version").unwrap(),
            store,
        }
    }

    /// One raw request/response round trip; returns the response bytes.
    fn call_raw(&mut self, request: &[u8]) -> Vec<u8> {
        let len = request.len() as u32;
        let ptr = self.alloc.call(&mut self.store, len).unwrap();
        self.memory
            .write(&mut self.store, ptr as usize, request)
            .unwrap();
        let packed = self.call.call(&mut self.store, (ptr, len)).unwrap();
        let (out_ptr, out_len) = ((packed >> 32) as u32, (packed & 0xffff_ffff) as u32);
        let mut out = vec![0u8; out_len as usize];
        self.memory
            .read(&self.store, out_ptr as usize, &mut out)
            .unwrap();
        self.free.call(&mut self.store, (out_ptr, out_len)).unwrap();
        out
    }

    fn call(&mut self, request: Value) -> Value {
        let out = self.call_raw(&serde_json::to_vec(&request).unwrap());
        serde_json::from_slice(&out).expect("response is JSON")
    }

    fn memory_pages(&self) -> u64 {
        self.memory.size(&self.store)
    }
}

fn golden() -> Value {
    serde_json::from_str(GOLDEN).unwrap()
}

#[test]
fn exports_the_abi_and_reports_its_release_identity() {
    let mut h = Host::new();
    assert_eq!(h.abi_version.call(&mut h.store, ()).unwrap(), 1);
    let caps = h.call(json!({"op": "capabilities"}));
    assert_eq!(caps["ok"], true);
    assert_eq!(caps["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        caps["genesis_seed"],
        format!("affidavit-v{}-genesis", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn accepts_a_real_affi_receipt() {
    let mut h = Host::new();
    let v = h.call(json!({"op": "verify", "receipt": golden()}));
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["accepted"], true, "{v}");
    assert_eq!(v["profile"], "CoreV1");
    assert_eq!(v["reason"], "all stages passed");
    let stages: Vec<&str> = v["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["stage"].as_str().unwrap())
        .collect();
    assert_eq!(
        stages,
        [
            "decode",
            "check_format",
            "chain_integrity",
            "continuity",
            "verify_commitments",
            "evaluate_profile"
        ]
    );
}

#[test]
fn rejects_a_tampered_receipt_with_the_exact_reason_affi_gives() {
    // Copied from `affi receipt verify tampered_receipt.json` on the same
    // fixture: same stored and recomputed hashes, same wording.
    let mut h = Host::new();
    let receipt: Value = serde_json::from_str(TAMPERED).unwrap();
    let v = h.call(json!({"op": "verify", "receipt": receipt}));
    assert_eq!(v["accepted"], false);
    assert_eq!(
        v["reason"],
        "chain_integrity: chain hash mismatch: stored d6cd5e0c07671707d34d5e65c42964a82f06776b93d0e33457b5a6dcf7765be0, recomputed 1f1d040c94289281daa2c13b18d4cda9b1a28c59ba323285febc20971f461d7a"
    );
}

#[test]
fn assemble_reproduces_affi_byte_for_byte() {
    // The golden receipt came from three `affi receipt emit` calls whose payload
    // files held these exact bytes (with trailing newline).
    let mut h = Host::new();
    let r = h.call(json!({"op": "assemble", "events": [
        {"event_type": "build",  "objects": ["repo:git:main"],             "payload": "compile step\n"},
        {"event_type": "test",   "objects": ["suite:test-suite:unit"],     "payload": "unit tests\n"},
        {"event_type": "deploy", "objects": ["svc:service"],               "payload": "ship it\n"},
    ]}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(
        r["receipt"],
        golden(),
        "wasm assemble diverged from real affi output"
    );
    assert_eq!(r["chain_hash"], golden()["chain_hash"]);
}

#[test]
fn native_and_wasm_answer_byte_identically() {
    // Same request, two targets (native rlib vs compiled wasm32): responses must
    // be identical bytes. Catches float, hasher or ordering drift across targets.
    let mut h = Host::new();
    let requests = [
        json!({"op": "capabilities"}),
        json!({"op": "verify", "receipt": golden()}),
        json!({"op": "commit", "payload": "héllo wörld ✓"}),
        json!({"op": "mine", "receipts": [golden(), golden()]}),
        json!({"op": "conform", "model": [golden()], "trace": golden()}),
        json!({"op": "nope"}),
    ];
    for req in requests {
        let raw = serde_json::to_vec(&req).unwrap();
        assert_eq!(
            h.call_raw(&raw),
            affidavit_wasm::abi::call(&raw),
            "target divergence on {req}"
        );
    }
}

#[test]
fn mines_and_conforms_over_a_real_receipt() {
    let mut h = Host::new();
    let m = h.call(json!({"op": "mine", "receipts": [golden()]}));
    assert_eq!(m["activities"], json!(["build", "deploy", "test"]));
    assert_eq!(m["all_accepted"], true);
    let c = h.call(json!({"op": "conform", "model": [golden()], "trace": golden()}));
    assert_eq!(c["verdict"], "conformant");
    assert_eq!(c["fitness"], 1.0);
}

#[test]
fn errors_cross_the_boundary_as_structured_json() {
    let mut h = Host::new();
    for (raw, code) in [
        (&b"garbage"[..], "bad_json"),
        (b"{}", "missing_field"),
        (br#"{"op":"nope"}"#, "unknown_op"),
        (b"", "bad_json"),
    ] {
        let r: Value = serde_json::from_slice(&h.call_raw(raw)).unwrap();
        assert_eq!(r["ok"], false);
        assert_eq!(r["error"]["code"], code);
    }
    // The module is still healthy after a run of failures.
    assert_eq!(h.call(json!({"op": "capabilities"}))["ok"], true);
}

#[test]
fn buffers_are_reclaimed_so_memory_does_not_grow_per_call() {
    let mut h = Host::new();
    let req = json!({"op": "verify", "receipt": golden()});
    for _ in 0..50 {
        h.call(req.clone()); // warm up allocator arenas
    }
    let warm = h.memory_pages();
    for _ in 0..2000 {
        assert_eq!(h.call(req.clone())["accepted"], true);
    }
    assert_eq!(
        h.memory_pages(),
        warm,
        "linear memory grew across 2000 identical calls: a buffer is leaking"
    );
}

#[test]
fn large_receipts_verify_inside_the_sandbox() {
    let mut h = Host::new();
    let events: Vec<Value> = (0..2000)
        .map(|i| {
            let ty = ["build", "test", "deploy"][i % 3];
            json!({"event_type": ty, "payload": format!("p{i}")})
        })
        .collect();
    let sealed = h.call(json!({"op": "assemble", "events": events}));
    assert_eq!(sealed["ok"], true);
    let v = h.call(json!({"op": "verify", "receipt": sealed["receipt"]}));
    assert_eq!(v["accepted"], true, "{}", v["reason"]);
}
