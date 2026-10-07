//! ABI metadata for the `affidavit-wasm` WASI JSON-ABI module.
//
// Consumed query columns (meta.rq): crate_name, export_prefix, abi_version,
// max_request_bytes, max_json_depth, ops, error_codes.
// Rendered by ggen (rust-wasi-wasmex-pack) from the wja: graph.
// Edit the ontology and re-render; never edit this file by hand.
#![allow(dead_code)]

/// Name of the crate hosting the module.
pub const CRATE_NAME: &str = "affidavit-wasm";

/// Prefix of the exported symbols (`<prefix>_abi_version/_alloc/_free/_call`).
pub const EXPORT_PREFIX: &str = "af";

/// Value returned by `<prefix>_abi_version`.
pub const ABI_VERSION: u32 = 1;

/// Upper bound on request size in bytes; alloc returns null above it.
pub const MAX_REQUEST_BYTES: usize = 16777216;

/// Upper bound on accepted JSON nesting depth.
pub const MAX_JSON_DEPTH: usize = 64;

/// Op table, ordered by `wja:opOrder`.
pub const OPS: &[&str] = &[
    "capabilities",
    "commit",
    "assemble",
    "verify",
    "mine",
    "conform",
    "verify_signature_input",
    "certify_authzen_evidence",
    "certify_spiffe_evidence",
    "jcs_canonicalize",
    "smt_absence_verify",
    "range_proof_verify",
    "derive_subject_digest",
    "verify_signature",
];

/// Typed error codes, ordered by `wja:codeOrder`.
pub const ERROR_CODES: &[&str] = &[
    "bad_json",
    "missing_field",
    "unknown_op",
    "bad_field",
    "too_large",
    "too_deep",
    "missing_buffer",
    "internal",
    "malformed",
];
