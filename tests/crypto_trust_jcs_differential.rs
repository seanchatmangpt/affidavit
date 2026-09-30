#![cfg(feature = "crypto-trust")]

//! Cross-runtime differential proof for the rendered JCS implementation
//! (lane W5-W1-L6, wave 5 wave 1).
//!
//! `src/crypto_trust_canonical::jcs` (ggen-rendered from
//! affidavit-trust-plane-pack; never edited here) is proven against a SECOND,
//! independent RFC 8785 implementation: `tools/jcs_differential.py`, pure
//! Python stdlib, which generated the corpus
//! `fixtures/crypto_trust_jcs_corpus.json` and the expected-output sidecar
//! `fixtures/crypto_trust_jcs_expected.json`.
//!
//! Corpus coverage: ~280 values across the three conformance axes — UTF-16
//! code-unit key ordering (supplementary plane vs BMP >= U+E000), ECMAScript
//! `Number::toString` boundaries (±0, 1e-7, 1e21, 2^53, 5e-324, 1.797e308,
//! shortest-roundtrip decimals, denormals, powers of ten), and §3.2.2.2
//! escaping (every C0 control, the six shorthands, lowercase `\u00xx`).
//!
//! Divergence taxonomy enforced here:
//! - **Parse-precision** (reported, non-fatal): scalar float cases carry the
//!   exact IEEE-754 bits Python parsed (`value_f64_bits`). serde_json's
//!   DEFAULT parser is not correctly rounded on long literals (documented in
//!   the rendered module header), so sides can legitimately hold adjacent
//!   doubles for 16-17-digit literals; canonicalizing different doubles is a
//!   parser artifact, NOT a jcs conformance bug.
//! - **Conformance divergence** (fatal, reported loudly with case ids and a
//!   classification — number-formatting vs escaping vs ordering): the sides
//!   held the SAME double/string/structure and produced different canonical
//!   bytes. Fix NOTHING rendered in this lane; the divergence is reported.
//!
//! Teeth: `harness_has_teeth` proves the checker is load-bearing by mutating
//! a Python-expected output in-test (digit flip, member-order swap, escape
//! flip) and asserting every mutation flips the verdict from agreement to
//! divergence — without touching the committed fixture.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use affidavit::crypto_trust_canonical::{jcs, CanonicalError};
use serde_json::Value;

const CORPUS_RELPATH: &str = "fixtures/crypto_trust_jcs_corpus.json";
const EXPECTED_RELPATH: &str = "fixtures/crypto_trust_jcs_expected.json";

/// One sidecar entry: what the independent Python implementation expects.
struct ExpectedCase {
    canonical: Option<String>,
    refusal: Option<String>,
    /// Hex IEEE-754 bits of the double Python parsed (scalar float cases).
    f64_bits: Option<String>,
}

/// Verdict of one corpus case against the Python expectation.
enum CaseOutcome {
    /// Sides agree (canonical bytes equal, or both refuse identically).
    Agree,
    /// Sides hold different IEEE-754 doubles for the literal: serde_json
    /// default-parser artifact (documented in the rendered module), not a
    /// canonicalization conformance bug.
    ParsePrecision,
    /// Same input double/value, different canonical bytes: REAL divergence.
    Diverge(String),
}

fn load_json(relpath: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relpath);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {relpath} must be readable at {path:?}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("fixture {relpath} must parse: {e}"))
}

fn expected_map(expected_doc: &Value) -> BTreeMap<String, ExpectedCase> {
    let cases = expected_doc["cases"]
        .as_array()
        .expect("sidecar holds a cases array");
    let mut map = BTreeMap::new();
    for case in cases {
        let id = case["id"].as_str().expect("sidecar case id").to_string();
        let entry = ExpectedCase {
            canonical: case["canonical"].as_str().map(str::to_string),
            refusal: case["refusal"].as_str().map(str::to_string),
            f64_bits: case["value_f64_bits"].as_str().map(str::to_string),
        };
        assert!(
            map.insert(id.clone(), entry).is_none(),
            "duplicate sidecar case id: {id}"
        );
    }
    map
}

/// Does the serde_json-parsed double carry the same bits the Python side saw?
fn bits_match(tag: &str, number: &serde_json::Number) -> bool {
    let Ok(expected_bits) = u64::from_str_radix(tag.trim_start_matches("0x"), 16) else {
        return false;
    };
    number.as_f64().map(|f| f.to_bits()) == Some(expected_bits)
}

/// Compare `jcs(value)` against one Python-expected outcome.
fn check_case(value: &Value, expected: &ExpectedCase) -> CaseOutcome {
    if let Some(refusal) = expected.refusal.as_deref() {
        return match jcs(value) {
            Ok(actual) => CaseOutcome::Diverge(format!(
                "refusal asymmetry: python refuses ({refusal}); rust canonicalized to {actual}"
            )),
            Err(CanonicalError::NonCanonicalNumber(_)) if refusal == "NonCanonicalNumber" => {
                CaseOutcome::Agree
            }
            Err(other) => CaseOutcome::Diverge(format!(
                "refusal kind asymmetry: python {refusal}; rust {other}"
            )),
        };
    }
    let want = expected
        .canonical
        .as_deref()
        .expect("sidecar case carries canonical or refusal");
    if let Value::Number(number) = value {
        if let Some(tag) = expected.f64_bits.as_deref() {
            if !bits_match(tag, number) {
                return CaseOutcome::ParsePrecision;
            }
        }
    }
    match jcs(value) {
        Ok(actual) if actual == want => CaseOutcome::Agree,
        Ok(actual) => CaseOutcome::Diverge(format!(
            "canonical bytes differ\n    python: {want}\n    rust:   {actual}"
        )),
        Err(e) => CaseOutcome::Diverge(format!(
            "rust refused where python canonicalized to {want}: {e}"
        )),
    }
}

/// Heuristic classification of a byte divergence for the failure report:
/// number-formatting vs escaping vs ordering vs unclassified.
fn classify_divergence(actual: &str, expected: &str, value: &Value) -> &'static str {
    let parsed = (
        serde_json::from_str::<Value>(actual),
        serde_json::from_str::<Value>(expected),
    );
    match parsed {
        (Ok(a), Ok(e)) => {
            if a == e {
                // Same JSON value; only token-level escaping differs.
                return "escaping";
            }
            if let (Ok(ca), Ok(ce)) = (jcs(&a), jcs(&e)) {
                if ca == ce {
                    // Re-canonicalization converges: member ORDER differs.
                    return "ordering";
                }
            }
            if value.is_number() || digits_removed(actual) == digits_removed(expected) {
                return "number-formatting";
            }
            "unclassified"
        }
        _ => "unparseable (an output is not JSON text)",
    }
}

fn digits_removed(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '0'..='9' | 'e' | 'E' | '+' | '.' | '-'))
        .collect()
}

/// Split a canonical object text into its top-level member substrings
/// (comma-aware of nesting and string escapes). None if not an object.
fn top_level_members(text: &str) -> Option<Vec<String>> {
    let chars: Vec<char> = text.chars().collect();
    if chars.first() != Some(&'{') || chars.last() != Some(&'}') {
        return None;
    }
    let mut members: Vec<String> = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 1usize;
    for i in 1..chars.len() - 1 {
        let ch = chars[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                members.push(chars[start..i].iter().collect());
                start = i + 1;
            }
            _ => {}
        }
    }
    members.push(chars[start..chars.len() - 1].iter().collect());
    Some(members)
}

fn corpus_pair() -> (Value, BTreeMap<String, ExpectedCase>) {
    let corpus = load_json(CORPUS_RELPATH);
    let expected = expected_map(&load_json(EXPECTED_RELPATH));
    (corpus, expected)
}

#[test]
fn jcs_matches_independent_python_across_corpus() {
    let (corpus, expected) = corpus_pair();
    let cases = corpus["cases"].as_array().expect("corpus cases array");
    assert_eq!(
        cases.len(),
        expected.len(),
        "corpus and sidecar must cover exactly the same case set"
    );

    let mut canonical_checked = 0usize;
    let mut refusal_checked = 0usize;
    let mut parse_precision: Vec<String> = Vec::new();
    let mut divergences: Vec<String> = Vec::new();

    for case in cases {
        let id = case["id"].as_str().unwrap_or("<missing id>");
        let value = &case["value"];
        let entry = expected
            .get(id)
            .unwrap_or_else(|| panic!("sidecar missing expected output for case {id}"));
        match check_case(value, entry) {
            CaseOutcome::Agree => {
                if entry.refusal.is_some() {
                    refusal_checked += 1;
                } else {
                    canonical_checked += 1;
                }
            }
            CaseOutcome::ParsePrecision => parse_precision.push(id.to_string()),
            CaseOutcome::Diverge(detail) => {
                let actual = jcs(value).unwrap_or_else(|e| format!("<refusal: {e}>"));
                let want = entry
                    .canonical
                    .as_deref()
                    .map(str::to_string)
                    .unwrap_or_else(|| entry.refusal.clone().unwrap_or_default());
                let classification = classify_divergence(&actual, &want, value);
                divergences.push(format!("  case {id} [{classification}]\n    {detail}"));
            }
        }
    }

    println!("JCS cross-runtime differential: rendered Rust jcs vs independent Python RFC 8785");
    println!("  corpus cases:              {}", cases.len());
    println!("  canonical agreements:      {canonical_checked}");
    println!("  refusal agreements:        {refusal_checked}");
    println!(
        "  parse-precision artifacts: {} (serde_json default parser; NOT jcs bugs): {:?}",
        parse_precision.len(),
        parse_precision
    );
    println!("  conformance divergences:   {}", divergences.len());

    assert!(
        divergences.is_empty(),
        "REAL RFC 8785 conformance divergences between the rendered jcs and the \
         independent Python implementation (fix NOTHING rendered in this lane; \
         report with case ids):\n{}",
        divergences.join("\n")
    );
}

#[test]
fn harness_has_teeth() {
    let (corpus, expected) = corpus_pair();
    let cases = corpus["cases"].as_array().expect("corpus cases array");

    // Locate three deterministic cases and prove each tooth: the shared
    // checker accepts the Python expectation unmutated, and a deliberate
    // in-test mutation of that expectation flips the verdict to Diverge.
    let mut number_tooth = None;
    let mut ordering_tooth = None;
    let mut escaping_tooth = None;

    for case in cases {
        let id = case["id"].as_str().expect("corpus case id");
        let value = &case["value"];
        let entry = expected
            .get(id)
            .unwrap_or_else(|| panic!("sidecar missing expected output for case {id}"));
        if number_tooth.is_none()
            && entry.refusal.is_none()
            && entry.f64_bits.is_none()
            && value.is_number()
        {
            number_tooth = Some((id, value, entry));
        }
        if ordering_tooth.is_none()
            && value.is_object()
            && entry
                .canonical
                .as_deref()
                .map(|c| top_level_members(c).is_some_and(|m| m.len() >= 2))
                .unwrap_or(false)
        {
            ordering_tooth = Some((id, value, entry));
        }
        if escaping_tooth.is_none()
            && value.is_string()
            && entry
                .canonical
                .as_deref()
                .map(|c| c.contains("\\n"))
                .unwrap_or(false)
        {
            escaping_tooth = Some((id, value, entry));
        }
    }

    // Tooth 1 — number: flip the first digit of a numeric expected output.
    let (id, value, entry) = number_tooth.expect("a scalar integer case exists");
    assert!(matches!(check_case(value, entry), CaseOutcome::Agree));
    let want = entry.canonical.as_deref().expect("canonical present");
    let mutated = flip_first_digit(want);
    assert!(mutated != want, "mutation must change the text");
    let mutated_entry = ExpectedCase {
        canonical: Some(mutated),
        refusal: None,
        f64_bits: None,
    };
    assert!(
        matches!(check_case(value, &mutated_entry), CaseOutcome::Diverge(_)),
        "tooth 1 (number) did not fire for case {id}"
    );

    // Tooth 2 — ordering: swap the first two members of an object expected.
    let (id, value, entry) = ordering_tooth.expect("an object case exists");
    assert!(matches!(check_case(value, entry), CaseOutcome::Agree));
    let want = entry.canonical.as_deref().expect("canonical present");
    let members = top_level_members(want).expect("object case splits");
    assert!(members.len() >= 2, "object case needs two members: {id}");
    let mut swapped = String::from("{");
    swapped.push_str(&members[1]);
    swapped.push(',');
    swapped.push_str(&members[0]);
    swapped.push('}');
    let swapped_entry = ExpectedCase {
        canonical: Some(swapped),
        refusal: None,
        f64_bits: None,
    };
    assert!(
        matches!(check_case(value, &swapped_entry), CaseOutcome::Diverge(_)),
        "tooth 2 (ordering) did not fire for case {id}"
    );

    // Tooth 3 — escaping: flip a \n shorthand into \t in a string expected.
    let (id, value, entry) = escaping_tooth.expect("a string case with a \\n escape exists");
    assert!(matches!(check_case(value, entry), CaseOutcome::Agree));
    let want = entry.canonical.as_deref().expect("canonical present");
    let flipped = want.replacen("\\n", "\\t", 1);
    let flipped_entry = ExpectedCase {
        canonical: Some(flipped),
        refusal: None,
        f64_bits: None,
    };
    assert!(
        matches!(check_case(value, &flipped_entry), CaseOutcome::Diverge(_)),
        "tooth 3 (escaping) did not fire for case {id}"
    );
}

fn flip_first_digit(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut flipped = false;
    for ch in text.chars() {
        if !flipped && ch.is_ascii_digit() {
            let next = (ch as u8 - b'0' + 1) % 10;
            out.push((b'0' + next) as char);
            flipped = true;
        } else {
            out.push(ch);
        }
    }
    out
}

#[test]
fn sidecar_covers_corpus_exactly() {
    let (corpus, expected) = corpus_pair();
    let cases = corpus["cases"].as_array().expect("corpus cases array");
    let corpus_ids: Vec<&str> = cases
        .iter()
        .map(|c| c["id"].as_str().expect("corpus case id"))
        .collect();
    assert_eq!(corpus_ids.len(), expected.len());
    for id in &corpus_ids {
        assert!(expected.contains_key(*id), "sidecar missing case {id}");
    }
    // Refusal entries must never carry a canonical form and vice versa.
    for entry in expected.values() {
        assert!(
            entry.canonical.is_some() || entry.refusal.is_some(),
            "sidecar entry with neither canonical nor refusal"
        );
        if entry.refusal.is_some() {
            assert!(entry.canonical.is_none(), "refusal entry carries canonical");
        }
    }
}
