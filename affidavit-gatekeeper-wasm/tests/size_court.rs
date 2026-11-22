//! Fail-closed size court for the gatekeeper verify_witness WASM.
//!
//! Bound: ≤ 256000 bytes (250 KiB marketing / GTM claim).
//! Oversize → REFUSED with measured bytes (never invent a pin under the bound).
//! When `registry/ARTIFACTS.sha256` carries a real wasm pin, sha256 + bytes
//! must match the resolved artifact.
//!
//! Artifact resolution (first hit wins):
//! 1. `AFFIDAVIT_GATEKEEPER_WASM` env path
//! 2. `dist/affidavit_gatekeeper_verify_witness.wasm` (post wasm-opt pin path)
//! 3. `target/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm`

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 256_000;
const ARTIFACTS: &str = include_str!("../registry/ARTIFACTS.sha256");

fn candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(p) = env::var("AFFIDAVIT_GATEKEEPER_WASM") {
        out.push(PathBuf::from(p));
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    out.push(manifest.join("dist/affidavit_gatekeeper_verify_witness.wasm"));
    out.push(manifest.join("target/wasm32-unknown-unknown/wasm/affidavit_gatekeeper_wasm.wasm"));
    out
}

fn resolve_artifact() -> Option<PathBuf> {
    candidate_paths().into_iter().find(|p| p.is_file())
}

fn pin_line() -> Option<(String, u64, String)> {
    ARTIFACTS.lines().find_map(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        if f.len() == 4 && f[0] == "wasm" && f[1].len() == 64 && !f[1].contains("PLACEHOLDER") {
            let bytes: u64 = f[2].parse().ok()?;
            Some((f[1].to_string(), bytes, f[3].to_string()))
        } else {
            None
        }
    })
}

#[test]
fn gatekeeper_wasm_size_court_le_256000() {
    let path = resolve_artifact().unwrap_or_else(|| {
        panic!(
            "REFUSED:MISSING_GATEKEEPER_WASM — build first:\n  \
             ./scripts/gatekeeper_wasm_build.sh\n  \
             or set AFFIDAVIT_GATEKEEPER_WASM=/path/to.wasm"
        );
    });
    let bytes_blob = fs::read(&path).unwrap_or_else(|e| {
        panic!("REFUSED:READ_FAILED path={} err={e}", path.display());
    });
    let bytes = bytes_blob.len() as u64;
    assert!(
        bytes <= MAX_BYTES,
        "REFUSED:GATEKEEPER_WASM_OVERSIZE measured={bytes} bound={MAX_BYTES} path={}",
        path.display()
    );
    assert!(
        Path::new(&path).extension().is_some_and(|e| e == "wasm"),
        "artifact must be a .wasm file"
    );

    if let Some((sha, pinned_bytes, name)) = pin_line() {
        assert!(
            pinned_bytes <= MAX_BYTES,
            "REFUSED:GATEKEEPER_WASM_OVERSIZE pinned={pinned_bytes} bound={MAX_BYTES}"
        );
        assert!(
            path.ends_with(&name) || path.file_name().is_some_and(|n| n == name.as_str()),
            "REFUSED:PIN_NAME_MISMATCH path={} pin_name={name}",
            path.display()
        );
        assert_eq!(
            bytes, pinned_bytes,
            "REFUSED:PIN_SIZE_MISMATCH measured={bytes} pinned={pinned_bytes}"
        );
        let got: String = Sha256::digest(&bytes_blob)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            got, sha,
            "REFUSED:PIN_SHA256_MISMATCH — rebuild with ./scripts/gatekeeper_wasm_build.sh"
        );
    }

    eprintln!(
        "ALIVE:GATEKEEPER_WASM_SIZE measured={bytes} bound={MAX_BYTES} path={}",
        path.display()
    );
}
