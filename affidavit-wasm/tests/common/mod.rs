//! Shared real-runtime harness: locates (or builds) the compiled module and
//! drives it through the JSON ABI in wasmi, exactly as a host such as Wasmex
//! would. Set `AFFIDAVIT_WASM` to test a prebuilt module; otherwise the module
//! is built once into `target/wasm-abi`.
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use serde_json::Value;
use wasmi::{Engine, Instance, Linker, Memory, Module, Store, TypedFunc};
use wasmi_wasi::{WasiCtx, WasiCtxBuilder};

pub fn wasm_path() -> &'static PathBuf {
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

pub struct Host {
    pub store: Store<WasiCtx>,
    pub memory: Memory,
    pub alloc: TypedFunc<u32, u32>,
    pub free: TypedFunc<(u32, u32), ()>,
    pub call: TypedFunc<(u32, u32), u64>,
    pub abi_version: TypedFunc<(), u32>,
}

impl Host {
    pub fn new() -> Self {
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
    pub fn call_raw(&mut self, request: &[u8]) -> Vec<u8> {
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

    pub fn call(&mut self, request: Value) -> Value {
        let out = self.call_raw(&serde_json::to_vec(&request).unwrap());
        serde_json::from_slice(&out).expect("response is JSON")
    }

    pub fn memory_pages(&self) -> u64 {
        self.memory.size(&self.store)
    }
}
