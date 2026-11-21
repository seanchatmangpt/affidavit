//! Application binding for the static AOD [`CompiledOntology`] (AFFI-SPEC-26.11.19 §4.1).
//!
//! Production `affi aod verify --live` admits against
//! [`affidavit_core::compiled_fixture`] by default. Court overrides remain
//! available via `--ontology` / `--strides`. Regenerated projection lives in
//! `affidavit-core/src/aod/fixture_ontology.rs` (source: `ontology/aod-fixture.ttl`).

pub use affidavit_core::{
    compiled_fixture, derive_ontology_hash, CompiledOntology, FIXTURE_ONTOLOGY_HASH,
    FIXTURE_ONTOLOGY_HASH_HEX, FIXTURE_SCHEMA_ID, FIXTURE_STRIDES,
};
