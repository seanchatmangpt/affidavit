#![cfg(feature = "crypto-trust")]

//! Published cross-runtime KAT fixture for the cryptographic trust plane
//! (wave 4, lane W4-L8).
//!
//! `src/crypto_trust_kat.rs` (ggen-rendered) is the single source of truth.
//! This suite proves that the committed fixture
//! `fixtures/crypto_trust_kat.json` is exactly `export_json(generate_corpus())`
//! wrapped in a canonical JCS header object, that the committed corpus
//! verifies through the Rust verifier, and that export/import is a byte fixed
//! point. The drift tripwire (`regeneration_reproduces_committed_fixture`)
//! refuses any silent change to the derivation chain: a corpus algorithm
//! change must consciously update the fixture so every downstream runtime
//! re-pins the new bytes.
//!
//! Regeneration (the exact command carried in the fixture's
//! `generated_from` note):
//!
//! ```text
//! cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 \
//!   cargo test --features crypto-trust --test crypto_trust_kat_fixture \
//!   export_fixture -- --exact --ignored
//! ```
//!
//! The independent (non-Rust) counterpart is
//! `tools/verify_crypto_trust_kat.py`, which re-derives the whole seed and
//! message chain in pure Python (BLAKE3 + the domain-separated layout) and
//! verifies the ES256 signatures and public-key derivations with the system
//! openssl.

use std::fs;
use std::path::PathBuf;

use affidavit::crypto_trust_canonical::{jcs, DOMAIN_TAG, ENVELOPE_VERSION};
use affidavit::crypto_trust_kat::{
    export_json, generate_corpus, import_json, verify_corpus, KAT_ALGORITHM_REGISTRY,
    KAT_DOMAIN_TAG, KAT_ENVELOPE_VERSION, KAT_SCHEMA_VERSION,
};
use serde_json::{json, Value};

const FIXTURE_RELPATH: &str = "fixtures/crypto_trust_kat.json";
const EXPORT_ENV: &str = "AFFIDAVIT_KAT_EXPORT_FIXTURE";

/// The exact header keys, in JCS (lexicographic) order. First keys of the
/// published file: schema version + generated-from note beside the corpus.
const EXPECTED_HEADER_KEYS: [&str; 3] = ["corpus", "generated_from", "schema_version"];

/// The generated-from note embedded in the fixture: provenance plus the exact
/// regeneration command.
const GENERATED_FROM: &str = "affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_RELPATH)
}

fn read_committed_fixture() -> String {
    fs::read_to_string(fixture_path())
        .unwrap_or_else(|e| panic!("committed fixture {FIXTURE_RELPATH} must be readable: {e}"))
}

/// The canonical JCS envelope object published as the fixture: the corpus
/// bytes plus the schema-version and generated-from header.
fn canonical_envelope(corpus_json: &str) -> String {
    let corpus: Value =
        serde_json::from_str(corpus_json).expect("export_json output must be valid JSON");
    let envelope = json!({
        "corpus": corpus,
        "generated_from": GENERATED_FROM,
        "schema_version": KAT_SCHEMA_VERSION,
    });
    jcs(&envelope).expect("string-only envelope must canonicalize")
}

/// The committed corpus as canonical JCS bytes (exactly the embedded
/// `export_json` output).
fn committed_corpus_json() -> String {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let corpus = value
        .get("corpus")
        .cloned()
        .expect("committed fixture must carry a corpus array");
    jcs(&corpus).expect("corpus array must canonicalize")
}

/// Exporter: regenerates `fixtures/crypto_trust_kat.json` from the rendered
/// KAT module. Guarded by [`EXPORT_ENV`] so an accidental `--ignored` run
/// refuses loudly instead of silently rewriting the committed bytes.
#[test]
#[ignore]
fn export_fixture() {
    if std::env::var(EXPORT_ENV).as_deref() != Ok("1") {
        panic!("export_fixture rewrites the committed KAT fixture; set {EXPORT_ENV}=1 to admit the export");
    }
    let canonical = canonical_envelope(&export_json(&generate_corpus()));
    fs::write(fixture_path(), &canonical)
        .unwrap_or_else(|e| panic!("must write {FIXTURE_RELPATH}: {e}"));
    println!(
        "wrote {FIXTURE_RELPATH} ({} bytes, canonical JCS)",
        canonical.len()
    );
}

#[test]
fn committed_fixture_header_is_exact() {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let object = value.as_object().expect("fixture must be a JSON object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        keys, EXPECTED_HEADER_KEYS,
        "fixture header must be exactly corpus/generated_from/schema_version in JCS order"
    );
    assert!(
        matches!(value["corpus"], Value::Array(_)),
        "corpus header field must be an array"
    );
    let note = value["generated_from"]
        .as_str()
        .expect("generated_from must be a string");
    assert!(
        note.contains("generate_corpus"),
        "generated_from must name the generating function"
    );
    assert!(
        note.contains(EXPORT_ENV),
        "generated_from must carry the exact regeneration command"
    );
    assert_eq!(
        value["schema_version"].as_str(),
        Some(KAT_SCHEMA_VERSION),
        "schema_version header must equal the rendered KAT schema version"
    );
}

#[test]
fn committed_fixture_is_jcs_canonical() {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let canonical = jcs(&value).expect("fixture must canonicalize");
    assert_eq!(
        canonical, raw,
        "committed fixture bytes must already be canonical JCS (RFC 8785)"
    );
}

#[test]
fn fixture_header_agrees_with_rendered_facts() {
    assert_eq!(KAT_SCHEMA_VERSION, "CTP-KAT-v1");
    assert_eq!(
        KAT_DOMAIN_TAG, DOMAIN_TAG,
        "KAT domain tag must be the plane domain tag"
    );
    assert_eq!(
        KAT_ENVELOPE_VERSION, ENVELOPE_VERSION,
        "KAT envelope version must be the plane envelope version"
    );
}

#[test]
fn committed_corpus_imports_and_verifies() {
    let corpus_json = committed_corpus_json();
    let vectors = import_json(&corpus_json).expect("committed corpus must import");
    let expected = KAT_ALGORITHM_REGISTRY.split("@@").count();
    assert_eq!(
        vectors.len(),
        expected,
        "committed corpus must carry one vector per registry entry"
    );
    let report = verify_corpus(&vectors).expect("committed corpus must verify");
    assert_eq!(report.total, expected);
    assert_eq!(report.passed, report.total, "strict law: passed == total");
    assert!(report.total > 0);
}

#[test]
fn committed_corpus_is_byte_fixed_point() {
    let corpus_json = committed_corpus_json();
    let back = import_json(&corpus_json).expect("committed corpus must import");
    assert_eq!(
        export_json(&back),
        corpus_json,
        "export_json(import_json(fixture corpus)) must be byte-identical to the committed corpus"
    );
}

/// The drift tripwire: in-test regeneration must reproduce the committed
/// fixture byte-for-byte. A corpus algorithm change (or any rendered fact
/// feeding the derivation chain) breaks this loudly, forcing a conscious
/// fixture regeneration that every downstream runtime re-pins.
#[test]
fn regeneration_reproduces_committed_fixture() {
    let regenerated = export_json(&generate_corpus());
    assert_eq!(
        regenerated,
        committed_corpus_json(),
        "KAT corpus drift: the rendered derivation chain changed; regenerate the fixture \
         consciously (see the fixture's generated_from note) so downstream runtimes re-pin"
    );
    assert_eq!(
        canonical_envelope(&regenerated),
        read_committed_fixture(),
        "full envelope drift: header or corpus changed; regenerate the fixture consciously"
    );
}
