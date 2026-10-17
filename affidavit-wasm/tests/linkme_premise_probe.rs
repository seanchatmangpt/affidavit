//! Register row 58 premise probe (delete-class): the doctor wasm32 fallback
//! (src/doctor_check.rs `#[cfg(target_arch = "wasm32")] DOCTOR_CHECKS`
//! empty-slice branch) claims "linkme `distributed_slice` needs linker
//! sections that wasm-ld does not provide". This court measures the premise
//! against linkme 0.3.36 on both wasm targets, with a flip leg that fires if
//! linkme ever gains wasm support.
//!
//! Measured 2026-10-17 (linkme 0.3.36, wasm32-wasip1 and wasm32-unknown-unknown):
//! the linkme probe does NOT compile — linkme's own proc-macro gates by
//! `target_os` (linkme-impl 0.3.36 declaration.rs allow-lists only
//! none/linux/macos/ios/tvos/android/fuchsia/illumos/freebsd/openbsd/psp plus
//! a windows/uefi branch) and emits "distributed_slice is not implemented for
//! this platform" as a compile_error!. wasm-ld never sees a linker section
//! from linkme. Separately, a hand-rolled `#[used] #[link_section]` probe with
//! boundary statics builds cleanly for wasm32 and the custom section survives
//! into the artifact — the raw mechanism IS available; the missing piece is
//! linkme's platform allow-list, not wasm-ld sections.
//!
//! Verdict grounding: row 47's UNSUPPORTED stands, but the fallback comment's
//! stated mechanism ("wasm-ld does not provide linker sections") is wrong.
//! The fallback (empty wasm32 slice) is still required today. Deletion becomes
//! lawful only if linkme gains wasm support (then the flip leg asserts
//! len()==1 under wasmi) or the registry is re-implemented on raw
//! #[link_section] + boundary symbols.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;
use std::process::Command;

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

/// Write a throwaway probe crate into a fresh temp dir; removed at test end.
fn write_probe_crate(name: &str, lib_rs: &str, deps: &[(&str, &str)]) -> PathBuf {
    use std::fs;
    let dir = std::env::temp_dir().join(format!("{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    let mut toml = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n"
    );
    for (n, v) in deps {
        toml.push_str(&format!("{n} = \"{v}\"\n"));
    }
    toml.push_str("\n[profile.release]\npanic = \"abort\"\nlto = true\n");
    fs::write(dir.join("Cargo.toml"), toml).unwrap();
    fs::write(dir.join("src/lib.rs"), lib_rs).unwrap();
    dir
}

/// Real cargo build for a wasm target; returns (success, combined output).
fn cargo_build(dir: &PathBuf, target: &str) -> (bool, String) {
    let out = Command::new(cargo())
        .current_dir(dir)
        .args(["build", "--release", "--target", target, "--offline"])
        .output()
        .expect("cargo spawns");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn wasm_artifact(dir: &PathBuf, target: &str, name: &str) -> PathBuf {
    dir.join("target")
        .join(target)
        .join("release")
        .join(format!("{name}.wasm"))
}

/// Positive control: wasm-ld accepts custom `#[used] #[link_section]` statics
/// with boundary markers on both wasm planes, the section bytes survive into
/// the artifact, and the module validates in the real wasmi runtime. This is
/// the leg that grounds the corrected mechanism claim.
#[test]
fn wasm_ld_keeps_custom_link_sections_and_the_module_validates() {
    let lib = r#"
#[used]
#[link_section = ".linkme_Start"]
static START: [u8; 0] = [];

#[used]
#[link_section = ".linkme_Stop"]
static STOP: [u8; 0] = [];

#[used]
#[link_section = ".linkme_elems"]
static ELEM: u32 = 7;

#[no_mangle]
pub extern "C" fn probe_len() -> u32 {
    1
}
"#;
    for target in ["wasm32-wasip1", "wasm32-unknown-unknown"] {
        let dir = write_probe_crate("lsmeta_probe", lib, &[]);
        let (ok, out) = cargo_build(&dir, target);
        assert!(
            ok,
            "hand-rolled #[link_section] probe failed to build for {target}:\n{out}"
        );
        let bytes = std::fs::read(wasm_artifact(&dir, target, "lsmeta_probe")).unwrap();
        assert!(
            bytes.windows(13).any(|w| w == b".linkme_elems"),
            "custom section name missing from emitted wasm for {target} ({} bytes)",
            bytes.len()
        );
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, &bytes[..])
            .unwrap_or_else(|e| panic!("wasmi rejects probe module for {target}: {e}"));
        let mut store = wasmi::Store::new(&engine, ());
        let instance = wasmi::Linker::<()>::new(&engine)
            .instantiate_and_start(&mut store, &module)
            .expect("probe module instantiates and starts");
        let f: wasmi::TypedFunc<(), u32> = instance
            .get_typed_func(&store, "probe_len")
            .expect("probe_len export present");
        let len = f.call(&mut store, ()).expect("probe_len runs");
        assert_eq!(len, 1, "positive control must return 1");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}

/// The premise probe: one #[distributed_slice], one element. If it compiles
/// for a wasm target, assert len()==1 at runtime through wasmi (this is the
/// DELETE-RECOMMENDED flip leg). Today linkme 0.3.36 refuses at macro
/// expansion with its platform-gate compile error, and this court asserts
/// exactly that refusal — the receipt grounding row 47's UNSUPPORTED.
#[test]
fn linkme_distributed_slice_on_wasm32() {
    let lib = r#"
use linkme::distributed_slice;

#[distributed_slice]
pub static PROBE: [&'static str];

#[distributed_slice(PROBE)]
static ELEM: &str = "probe-elem";

#[no_mangle]
pub extern "C" fn probe_len() -> u32 {
    PROBE.len() as u32
}
"#;
    for target in ["wasm32-wasip1", "wasm32-unknown-unknown"] {
        let dir = write_probe_crate("linkme_probe", lib, &[("linkme", "=0.3.36")]);
        let (ok, out) = cargo_build(&dir, target);
        if ok {
            // Flip leg: linkme now supports wasm — demand runtime proof.
            let bytes = std::fs::read(wasm_artifact(&dir, target, "linkme_probe")).unwrap();
            let engine = wasmi::Engine::default();
            let module = wasmi::Module::new(&engine, &bytes[..]).expect("module validates");
            let mut store = wasmi::Store::new(&engine, ());
            let instance = wasmi::Linker::<()>::new(&engine)
                .instantiate_and_start(&mut store, &module)
                .expect("instantiates and starts");
            let f: wasmi::TypedFunc<(), u32> = instance
                .get_typed_func(&store, "probe_len")
                .expect("probe_len export");
            let len = f.call(&mut store, ()).expect("probe_len runs");
            assert_eq!(len, 1, "distributed slice did not materialize on {target}");
        } else {
            assert!(
                out.contains("not implemented for this platform"),
                "linkme probe failed for {target} but NOT via the platform gate:\n{out}"
            );
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
