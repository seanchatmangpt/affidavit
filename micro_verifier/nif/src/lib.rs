//! Documented Rustler skeleton (copy into a Mix `native/` crate).
//!
//! This file is **not** part of the `affidavit-micro-verifier` Cargo package.
//! The compiling NIF lives in `micro_verifier/src/nif.rs` behind `feature = "nif"`.
//! Kept here so BEAM hosts can diff against a stable stub without enabling
//! Rustler in the affidavit validate ladder.
//!
//! ```ignore
//! use rustler::{Binary, Encoder, Env, NifResult, Term};
//!
//! #[rustler::nif]
//! fn aod_abi_version() -> u32 { 1 }
//!
//! #[rustler::nif]
//! fn aod_witness_verify<'a>(env: Env<'a>, raw: Binary<'a>) -> NifResult<Term<'a>> {
//!     // Delegate to affidavit_micro_verifier::witness_verify — never reimplement.
//!     let _ = (env, raw);
//!     unimplemented!("link affidavit-micro-verifier; see micro_verifier/src/nif.rs")
//! }
//!
//! rustler::init!("Elixir.Affidavit.Aod.Witness");
//! ```

#![allow(dead_code)]

/// Stub marker so `rustc --crate-type lib` on this file alone still typechecks
/// the documented contract without Rustler.
pub const DOCUMENTED_ABI_VERSION: u32 = 1;
