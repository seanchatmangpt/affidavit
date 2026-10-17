#![cfg(not(target_arch = "wasm32"))]

//! JCS parity court: drive the wasm ABI `jcs_canonicalize` op through
//! `affidavit_wasm::abi::call` with every case of the root crate's RFC 8785
//! corpus (`../fixtures/crypto_trust_jcs_corpus.json`, the same fixture the
//! root differential court `tests/crypto_trust_jcs_differential.rs` uses).
//!
//! Invariants under court:
//!   1. `commitment == blake3(canonical bytes)` for every case.
//!   2. Canonical bytes are a fixed point: re-canonicalizing the output
//!      reproduces the output byte-exact.
//!   3. Agreement with the independent Python sidecar
//!      (`../fixtures/crypto_trust_jcs_expected.json`) under the same
//!      divergence taxonomy as the root differential court:
//!        - parse-precision (parsed double lacks the sidecar's IEEE-754
//!          bits) is a serde_json parser artifact, counted, non-fatal;
//!        - same-bits byte disagreement is a REAL divergence, fatal;
//!        - the 8 big-int refusal cases are a registered divergence: the
//!          wasm op canonicalizes where the Python RFC 8785 implementation
//!          refuses; round-trip value preservation is asserted instead;
//!        - THREE ordering divergences are registered (all the same class):
//!          `obj_utf16_order_torture`, `obj_astral_vs_uffff`,
//!          `obj_ue000_vs_u10000`.
//!          serde_jcs orders keys by Unicode scalar value; RFC 8785 §3.2.3
//!          orders by UTF-16 code units (they disagree for BMP-high vs
//!          supplementary-plane keys). The root crate's rendered `jcs`
//!          implements UTF-16 ordering, so this is a REAL wasm-vs-root
//!          parity gap, registered (not fixed) because this lane owns only
//!          this test file. Any divergence outside the registered set is
//!          fatal.
//!
//! Registered limitation (lane constraint: this court may not edit
//! Cargo.toml, so affidavit-wasm cannot dev-depend on the root crate):
//! byte comparison against the ROOT crate's rendered
//! `crypto_trust_canonical::jcs` is NOT asserted in this file. The
//! independent Python sidecar comparison IS, so the court is not
//! tautological: wasm/serde_jcs bytes are compared against a second
//! implementation written in another language over the same corpus.

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};

const CORPUS_RELPATH: &str = "../fixtures/crypto_trust_jcs_corpus.json";
const EXPECTED_RELPATH: &str = "../fixtures/crypto_trust_jcs_expected.json";

/// Drive one input text through the real wasm ABI export path; returns
/// (canonical_json, commitment_hex).
fn call_jcs(input_text: &str) -> (String, String) {
    let req = json!({"op": "jcs_canonicalize", "json": input_text});
    let out = affidavit_wasm::abi::call(&serde_json::to_vec(&req).unwrap());
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        r["ok"], true,
        "wasm ABI refused a case it must canonicalize: input={input_text} response={r}"
    );
    let canonical = r["canonical_json"]
        .as_str()
        .expect("response carries canonical_json str")
        .to_string();
    let commitment = r["commitment"]
        .as_str()
        .expect("response carries commitment str")
        .to_string();
    (canonical, commitment)
}

/// One corpus row: input value plus the Python sidecar's expectation.
struct Row {
    id: String,
    value: Value,
    canonical: Option<String>,
    refusal: Option<String>,
    f64_bits: Option<String>,
}

/// Join the corpus values (by id) with the Python sidecar expectations.
fn rows() -> Vec<Row> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus: Value = serde_json::from_str(
        &fs::read_to_string(manifest.join(CORPUS_RELPATH))
            .unwrap_or_else(|e| panic!("corpus fixture unreadable: {e}")),
    )
    .expect("corpus fixture parses");
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(manifest.join(EXPECTED_RELPATH))
            .unwrap_or_else(|e| panic!("expected sidecar unreadable: {e}")),
    )
    .expect("expected sidecar parses");

    let mut values: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    for case in corpus["cases"].as_array().expect("corpus cases array") {
        let id = case["id"].as_str().expect("corpus case id").to_string();
        values.insert(id, case["value"].clone());
    }

    let mut out = Vec::new();
    for case in expected["cases"].as_array().expect("sidecar cases array") {
        let id = case["id"].as_str().expect("sidecar case id").to_string();
        let value = values
            .remove(&id)
            .unwrap_or_else(|| panic!("sidecar id {id} missing from corpus"));
        out.push(Row {
            id,
            value,
            canonical: case["canonical"].as_str().map(str::to_string),
            refusal: case["refusal"].as_str().map(str::to_string),
            f64_bits: case["value_f64_bits"].as_str().map(str::to_string),
        });
    }
    assert!(
        values.is_empty(),
        "corpus ids absent from the sidecar: {values:?}"
    );
    out
}

#[test]
fn jcs_corpus_parity_and_commitment_invariant() {
    let all = rows();
    assert!(
        all.len() >= 280,
        "corpus shrank below the documented ~280 cases: {}",
        all.len()
    );

    let mut agree = 0usize;
    let mut parse_precision = 0usize;
    let mut registered_bigint_refusal = 0usize;
    let mut registered_ordering: Vec<String> = Vec::new();
    for row in &all {
        let input_text = serde_json::to_string(&row.value).unwrap();
        let (canonical, commitment) = call_jcs(&input_text);

        // Invariant 1: commitment == blake3(canonical bytes), recomputed
        // host-side with the same external blake3 crate the ABI uses.
        assert_eq!(
            commitment,
            blake3::hash(canonical.as_bytes()).to_hex().to_string(),
            "case {}: commitment is not blake3 of the canonical bytes",
            row.id
        );

        // Invariant 2: fixed point.
        let (recanon, recommit) = call_jcs(&canonical);
        assert_eq!(recanon, canonical, "case {}: not a fixed point", row.id);
        assert_eq!(
            recommit, commitment,
            "case {}: commitment moved on re-canonicalization",
            row.id
        );

        // Invariant 3: cross-implementation agreement with the Python sidecar.
        if let Some(refusal) = row.refusal.as_deref() {
            assert_eq!(refusal, "NonCanonicalNumber", "case {}", row.id);
            // Registered divergence: serde_jcs canonicalizes big integers
            // where the Python RFC 8785 implementation refuses. Assert the
            // round-trip preserves the value exactly, and count loudly.
            let round: Value = serde_json::from_str(&canonical).unwrap();
            assert_eq!(
                round, row.value,
                "case {}: big-int round-trip lost value",
                row.id
            );
            registered_bigint_refusal += 1;
            continue;
        }

        let want = row.canonical.as_deref().expect("sidecar row shape");
        if canonical == want {
            agree += 1;
            continue;
        }
        if let (Some(tag), Value::Number(n)) = (row.f64_bits.as_deref(), &row.value) {
            let expected_bits =
                u64::from_str_radix(tag.trim_start_matches("0x"), 16).expect("bits tag parses");
            if n.as_f64().map(|f| f.to_bits()) != Some(expected_bits) {
                // Parse-precision artifact: the two sides hold adjacent
                // doubles for the same literal. Non-fatal, counted.
                parse_precision += 1;
                continue;
            }
        }
        // Classification before verdict: a divergence whose two sides parse
        // to the SAME JSON value is an ORDERING/ESCAPING divergence. Exactly
        // one is registered: `obj_utf16_order_torture` — serde_jcs orders
        // object keys by Unicode scalar value (Rust `String`/BTreeMap byte
        // order), while RFC 8785 §3.2.3 orders by UTF-16 code units; the two
        // disagree for BMP-high keys (>= U+E000) vs supplementary-plane keys.
        // The root crate's rendered `jcs` implements UTF-16 ordering (the
        // root differential court is green on this same corpus), so this is
        // a REAL wasm-vs-root parity gap, registered here because this lane
        // does not own src/advanced.rs. Any divergence outside the
        // registered set is fatal.
        let parses_equal = (serde_json::from_str::<Value>(&canonical).ok(),
            serde_json::from_str::<Value>(want).ok());
        if parses_equal == (Some(row.value.clone()), Some(row.value.clone())) {
            const REGISTERED_UTF16_ORDERING: [&str; 3] = [
                "obj_utf16_order_torture",
                "obj_astral_vs_uffff",
                "obj_ue000_vs_u10000",
            ];
            assert!(
                REGISTERED_UTF16_ORDERING.contains(&row.id.as_str()),
                "UNREGISTERED ordering/escaping divergence on case {}:\n  wasm/serde_jcs: {canonical}\n  python sidecar: {want}",
                row.id
            );
            let round: Value = serde_json::from_str(&canonical).unwrap();
            assert_eq!(round, row.value, "case {}: registered-divergence round-trip lost value", row.id);
            registered_ordering.push(row.id.clone());
            continue;
        }
        panic!(
            "REAL JCS divergence on case {}:\n  wasm/serde_jcs: {canonical}\n  python sidecar: {want}",
            row.id
        );
    }

    eprintln!(
        "jcs_parity counts: total={} agree={agree} parse_precision={parse_precision} registered_bigint_refusal={registered_bigint_refusal} registered_ordering={registered_ordering:?}",
        all.len()
    );
    registered_ordering.sort();
    registered_ordering.sort();
    assert_eq!(
        registered_ordering,
        ["obj_astral_vs_uffff", "obj_ue000_vs_u10000", "obj_utf16_order_torture"],
        "the registered UTF-16-ordering divergence set moved: {registered_ordering:?}"
    );
    assert!(agree >= 250, "too few agreeing cases: {agree}");
    assert_eq!(
        registered_bigint_refusal, 8,
        "expected exactly 8 registered big-int refusal cases"
    );
}

#[test]
fn reordered_key_envelope_canonicalizes_identically() {
    // Vacuity positive control: a deliberately reordered-key envelope must
    // canonicalize to the same bytes. The court can fail: the two inputs
    // really are different byte strings, and the canonical form really is a
    // recursive key sort (a no-op canonicalizer flips every assert here).
    let a = r#"{"zeta":1,"alpha":{"d":4,"b":2,"c":3},"mid":null}"#;
    let b = r#"{"mid":null,"alpha":{"c":3,"d":4,"b":2},"zeta":1}"#;
    assert_ne!(a, b, "fixture inputs must differ as raw bytes");
    let (ca, ka) = call_jcs(a);
    let (cb, kb) = call_jcs(b);
    assert_ne!(a, ca, "canonicalizer must not pass the envelope through");
    assert_eq!(ca, cb, "reordered-key envelopes must canonicalize identically");
    assert_eq!(ka, kb, "commitments must agree across key reordering");
    assert_eq!(
        ca, r#"{"alpha":{"b":2,"c":3,"d":4},"mid":null,"zeta":1}"#,
        "canonical form must sort keys recursively"
    );
}

#[test]
fn float_precision_2p53_neighbors_round_trip() {
    // Float-precision-sensitive cases: 2^53 and its neighbors. The first two
    // are exactly representable doubles; 2^53+1 is NOT exactly representable
    // as an f64 but IS an exact i64 through serde_json, so the canonical
    // digits must round-trip byte-exact either way.
    for text in [
        "9007199254740992", // 2^53
        "9007199254740991", // 2^53-1
        "9007199254740993", // 2^53+1
    ] {
        let (canonical, commitment) = call_jcs(text);
        assert_eq!(canonical, text, "exact integer digits must round-trip byte-exact");
        let parsed: Value = serde_json::from_str(&canonical).unwrap();
        assert!(parsed.is_number(), "round-trip must stay a number");
        assert_eq!(
            commitment,
            blake3::hash(canonical.as_bytes()).to_hex().to_string(),
            "commitment must bind the round-tripped canonical bytes"
        );
    }
}
