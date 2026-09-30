//! Registry and artifact court.
//!
//! 1. `registry/capability-registry.json` (rendered from the ontology) equals
//!    what the module itself reports through `capabilities` and `abi_meta`.
//! 2. The compiled module exports exactly the rendered symbol set and imports
//!    WASI only.
//! 3. When `AFFIDAVIT_WASM` is set and `registry/ARTIFACTS.sha256` carries a real
//!    pin, the module's sha256 and size equal the pin. Real files, real wasmi.
#![cfg(not(target_arch = "wasm32"))]

use affidavit_wasm::abi::call;
use affidavit_wasm::abi_meta::{
    ABI_VERSION, CRATE_NAME, ERROR_CODES, EXPORT_PREFIX, MAX_JSON_DEPTH, MAX_REQUEST_BYTES, OPS,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

mod common;

const REGISTRY: &str = include_str!("../registry/capability-registry.json");
const ARTIFACTS: &str = include_str!("../registry/ARTIFACTS.sha256");

fn registry() -> Value {
    serde_json::from_str(REGISTRY).expect("registry parses")
}

#[test]
fn registry_equals_the_modules_own_capabilities() {
    let reg = registry();
    let caps: Value = serde_json::from_slice(&call(br#"{"op":"capabilities"}"#)).unwrap();
    assert_eq!(reg["schema"], "wasi-json-abi.capability-registry/1");
    assert_eq!(reg["crate"], CRATE_NAME);
    assert_eq!(reg["abi_version"], ABI_VERSION);
    assert_eq!(reg["abi_version"], caps["abi_version"]);
    let reg_ops: Vec<&str> = reg["ops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["name"].as_str().unwrap())
        .collect();
    assert_eq!(reg_ops, OPS);
    assert_eq!(caps["ops"], json!(OPS));
    assert_eq!(reg["error_codes"], json!(ERROR_CODES));
    assert_eq!(caps["error_codes"], json!(ERROR_CODES));
    assert_eq!(
        reg["limits"]["max_request_bytes"],
        caps["limits"]["max_request_bytes"]
    );
    assert_eq!(reg["limits"]["max_request_bytes"], MAX_REQUEST_BYTES);
    assert_eq!(
        reg["limits"]["max_json_depth"],
        caps["limits"]["max_json_depth"]
    );
    assert_eq!(reg["limits"]["max_json_depth"], MAX_JSON_DEPTH);
    assert_eq!(reg["imports_policy"], "wasi_snapshot_preview1");
}

#[test]
fn compiled_module_exports_the_registry_symbols_and_imports_wasi_only() {
    // Drives the real build (or AFFIDAVIT_WASM) so the rendered export set is
    // witnessed on the deployed subject, not on the source.
    let _host = common::Host::new();
    let bytes = std::fs::read(common::wasm_path()).unwrap();
    let engine = wasmi::Engine::default();
    let module = wasmi::Module::new(&engine, &bytes[..]).unwrap();
    let mut exports: Vec<String> = module
        .exports()
        .filter(|e| e.ty().func().is_some())
        .map(|e| e.name().to_string())
        .filter(|n| n.starts_with(&format!("{EXPORT_PREFIX}_")))
        .collect();
    exports.sort();
    let mut want: Vec<String> = registry()["exports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    want.sort();
    assert_eq!(exports, want);
    for i in module.imports() {
        assert_eq!(i.module(), "wasi_snapshot_preview1");
    }
}

fn pin() -> Option<(String, usize, String)> {
    ARTIFACTS.lines().find_map(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.len() == 4 && f[0] == "wasm" && f[1].len() == 64 {
            Some((f[1].to_string(), f[2].parse().ok()?, f[3].to_string()))
        } else {
            None
        }
    })
}

#[test]
fn pinned_artifact_matches_the_built_module() {
    let Some(path) = std::env::var_os("AFFIDAVIT_WASM") else {
        eprintln!("AFFIDAVIT_WASM unset: artifact pin not checked");
        return;
    };
    let Some((sha, size, name)) = pin() else {
        eprintln!("ARTIFACTS.sha256 holds no real pin yet: not checked");
        return;
    };
    let bytes = std::fs::read(&path).expect("read AFFIDAVIT_WASM");
    assert!(
        std::path::Path::new(&path).ends_with(&name),
        "pin names {name}"
    );
    assert_eq!(
        bytes.len(),
        size,
        "pinned size differs: rebuild changed the module"
    );
    let got: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        got, sha,
        "module sha256 differs from registry/ARTIFACTS.sha256; if the change is \
         intended, rebuild with --locked and update the pin"
    );
}
