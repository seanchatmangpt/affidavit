#![cfg(feature = "crypto-trust")]
//! W4-L7 differential wire-format fuzz court over the three wire surfaces of
//! the cryptographic trust plane:
//!
//! 1. `envelope` — `SignatureEnvelope::to_bytes` / `from_bytes` (canonical
//!    JCS signed bytes);
//! 2. `sealed` — the sealed-receipt JSON document (`SealedReceipt`
//!    deserialization, whose base receipt recomputes the chain law);
//! 3. `hybrid` — the hybrid signature JSON (`HybridSignature` serde).
//!
//! The corpus at `fixtures/crypto_trust_wire_corpus.json` is a STATIC,
//! deterministic corpus generated once at authoring time (provenance header
//! names the generator and algorithm; a re-run of the generator reproduces
//! the file byte-identically, so CI classification never drifts). Every case's
//! `expect` was OBSERVED on these exact bytes at generation time and is pinned
//! here.
//!
//! Court law: for EVERY case, on EVERY surface, the parser must
//! (a) never panic — malformed input is refused as a typed error, never a
//!     crash (checked under `catch_unwind`; any caught panic is a REAL BUG in
//!     the rendered module and fails this test loudly with the case id), and
//! (b) match its pinned expectation:
//!     - `refuse` — the parser returns Err;
//!     - `accept-different` — the parser returns Ok with a value DIFFERENT
//!       from the original document's parse (for the envelope surface the
//!       never-accept-and-equal law holds absolutely: a mutation that parses
//!       back to the equal envelope would mean the mutation was an identity,
//!       so the envelope corpus carries no `any` cases at all);
//!     - `any` — non-semantic mutation (whitespace, duplicate keys with the
//!       last value winning, unknown fields) that lawfully parses back equal;
//!       only the no-panic law is asserted.
//!
//! Originals for the difference checks come from the corpus provenance header
//! itself (`originals.envelope_bytes_b64` cross-checked against the in-test
//! constants, `originals.sealed_json`, `originals.hybrid_json`), so this court
//! and the generator cannot drift apart silently.

use affidavit::crypto_trust_envelope::{SignatureEnvelope, ENVELOPE_VERSION};
use affidavit::crypto_trust_keys::{AlgorithmId, CryptoProfile, KeyId};
use affidavit::crypto_trust_pqc::HybridSignature;
use affidavit::crypto_trust_seal::SealedReceipt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// The static corpus, compiled in (deterministic CI; no runtime generator).
const CORPUS: &str = include_str!("../fixtures/crypto_trust_wire_corpus.json");

/// The court instant the generator used (fixed; no wall-clock in this lane).
const NOW: u64 = 1_700_000_500;

#[derive(Deserialize, Debug)]
struct CorpusCase {
    id: String,
    surface: String,
    kind: String,
    #[serde(default)]
    bytes_b64: Option<String>,
    #[serde(default)]
    json: Option<String>,
    expect: String,
}

#[derive(Deserialize, Debug)]
struct CorpusOriginals {
    envelope_bytes_b64: String,
    sealed_json: String,
    hybrid_json: String,
}

#[derive(Deserialize, Debug)]
struct CorpusProvenance {
    generator: String,
    algorithm: String,
    counts: Value,
    originals: CorpusOriginals,
}

#[derive(Deserialize, Debug)]
struct Corpus {
    provenance: CorpusProvenance,
    cases: Vec<CorpusCase>,
}

/// What a surface parser did with one case.
enum Outcome {
    Refused,
    EnvelopeOk(SignatureEnvelope),
    SealedOk(Box<SealedReceipt>),
    HybridOk(HybridSignature),
}

/// The in-test mirror of the generator's envelope fixture. The tripwire pins
/// this to the corpus via `originals.envelope_bytes_b64`, so if either side
/// drifts the court refuses instead of comparing against a silent mismatch.
fn original_envelope() -> SignatureEnvelope {
    SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: KeyId("k-wirefuzz-1".to_string()),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [0x77; 16],
        not_before: NOW - 1_000,
        expires_at: NOW + 1_000,
        subject_digest: [0x22; 32],
        audience: "affidavit.wirefuzz".to_string(),
    }
}

/// Standard base64 (RFC 4648, padded) decode — exact inverse of the
/// generator's encoder. A malformed corpus cell fails the test naming the
/// case (corpus bug, never a silent skip).
fn b64_decode(input: &str, case_id: &str) -> Vec<u8> {
    fn val(b: u8) -> Option<u32> {
        match b {
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'a'..=b'z' => Some((b - b'a') as u32 + 26),
            b'0'..=b'9' => Some((b - b'0') as u32 + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    assert!(
        bytes.len() % 4 == 0,
        "case {case_id}: base64 length {} not a multiple of 4",
        bytes.len()
    );
    let padding = bytes.iter().rev().take_while(|&&b| b == b'=').count();
    let data_end = bytes.len() - padding;
    assert!(
        bytes[..data_end].iter().all(|&b| b != b'='),
        "case {case_id}: base64 padding must be trailing only"
    );
    assert!(
        padding < 3,
        "case {case_id}: base64 padding exceeds 2 chars"
    );
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let n_vals = chunk.iter().filter(|&&b| b != b'=').count();
        let mut acc: u32 = 0;
        for &b in chunk {
            let v = if b == b'=' {
                0
            } else {
                val(b).unwrap_or_else(|| panic!("case {case_id}: invalid base64 byte {b:#04x}"))
            };
            acc = (acc << 6) | v;
        }
        match n_vals {
            4 => out.extend_from_slice(&[(acc >> 16) as u8, (acc >> 8) as u8, acc as u8]),
            3 => out.extend_from_slice(&[(acc >> 16) as u8, (acc >> 8) as u8]),
            2 => out.push((acc >> 16) as u8),
            _ => panic!("case {case_id}: final base64 quantum has {n_vals} values"),
        }
    }
    out
}

/// The one payload of a case, decoded (exactly one of bytes_b64 / json).
enum Payload {
    Bytes(Vec<u8>),
    Json(String),
}

fn payload_of(case: &CorpusCase) -> Payload {
    match (case.bytes_b64.as_deref(), case.json.as_deref()) {
        (Some(b64), None) => Payload::Bytes(b64_decode(b64, &case.id)),
        (None, Some(json)) => Payload::Json(json.to_string()),
        (None, None) => panic!("case {}: carries no payload", case.id),
        (Some(_), Some(_)) => panic!("case {}: carries both payloads", case.id),
    }
}

fn panic_text(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// Run the case's surface parser under catch_unwind. THIS is the differential
/// executor: envelope bytes go to `from_bytes`, sealed/hybrid JSON to serde.
fn execute(case: &CorpusCase, payload: &Payload) -> Result<Outcome, String> {
    let run = AssertUnwindSafe(|| match (case.surface.as_str(), payload) {
        ("envelope", Payload::Bytes(bytes)) => match SignatureEnvelope::from_bytes(bytes) {
            Err(_) => Outcome::Refused,
            Ok(envelope) => Outcome::EnvelopeOk(envelope),
        },
        ("envelope", Payload::Json(_)) => {
            panic!("case {}: envelope surface must carry bytes_b64", case.id)
        }
        ("sealed", Payload::Bytes(bytes)) => match serde_json::from_slice::<SealedReceipt>(bytes) {
            Err(_) => Outcome::Refused,
            Ok(sealed) => Outcome::SealedOk(Box::new(sealed)),
        },
        ("sealed", Payload::Json(text)) => match serde_json::from_str::<SealedReceipt>(text) {
            Err(_) => Outcome::Refused,
            Ok(sealed) => Outcome::SealedOk(Box::new(sealed)),
        },
        ("hybrid", Payload::Bytes(bytes)) => match serde_json::from_slice::<HybridSignature>(bytes)
        {
            Err(_) => Outcome::Refused,
            Ok(hybrid) => Outcome::HybridOk(hybrid),
        },
        ("hybrid", Payload::Json(text)) => match serde_json::from_str::<HybridSignature>(text) {
            Err(_) => Outcome::Refused,
            Ok(hybrid) => Outcome::HybridOk(hybrid),
        },
        (other, _) => panic!("case {}: unknown surface {other:?}", case.id),
    });
    catch_unwind(run).map_err(|payload| {
        format!(
            "REAL BUG: surface={} id={} kind={} PANICKED on malformed input: {}",
            case.surface,
            case.id,
            case.kind,
            panic_text(payload)
        )
    })
}

fn load_corpus() -> Corpus {
    serde_json::from_str(CORPUS).expect("static wire corpus must parse")
}

/// Tripwire: the corpus file itself is lawful — loadable, >= 500 cases, all
/// three surfaces represented, every case well-formed and decodable, and the
/// provenance counts true. Also pins the in-test envelope fixture to the
/// corpus originals so generator and court cannot drift apart silently.
#[test]
fn corpus_is_loadable_and_meets_the_lane_contract() {
    let corpus = load_corpus();

    assert!(
        corpus.cases.len() >= 500,
        "corpus must hold >= 500 cases, holds {}",
        corpus.cases.len()
    );

    let mut surfaces: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &corpus.cases {
        *surfaces.entry(case.surface.as_str()).or_default() += 1;
    }
    for surface in ["envelope", "sealed", "hybrid"] {
        let count = surfaces.get(surface).copied().unwrap_or(0);
        assert!(count > 0, "surface {surface} absent from the corpus");
    }

    // Provenance counts must equal observed counts (the header is evidence,
    // not decoration).
    let counts = &corpus.provenance.counts;
    let total = counts["total"].as_u64().expect("counts.total") as usize;
    assert_eq!(total, corpus.cases.len(), "provenance counts.total drifted");
    for surface in ["envelope", "sealed", "hybrid"] {
        let claimed = counts[surface].as_u64().expect("counts key") as usize;
        assert_eq!(
            claimed,
            surfaces.get(surface).copied().unwrap_or(0),
            "provenance counts.{surface} drifted"
        );
    }
    assert!(
        !corpus.provenance.generator.is_empty() && !corpus.provenance.algorithm.is_empty(),
        "provenance header must name the generator and algorithm"
    );

    // Every case well-formed; the envelope surface obeys the
    // never-accept-and-equal law (no identity expectations, bytes only).
    let valid_expect = ["refuse", "accept-different", "any"];
    for case in &corpus.cases {
        assert!(
            valid_expect.contains(&case.expect.as_str()),
            "case {}: illegal expect {:?}",
            case.id,
            case.expect
        );
        match (case.bytes_b64.is_some(), case.json.is_some()) {
            (true, false) | (false, true) => {}
            other => panic!("case {}: payload cardinality {:?}", case.id, other),
        }
        if case.surface == "envelope" {
            assert!(
                case.bytes_b64.is_some(),
                "case {}: envelope surface must carry bytes_b64",
                case.id
            );
            assert_ne!(
                case.expect, "any",
                "case {}: envelope corpus must never carry an identity expectation",
                case.id
            );
        }
        if let Some(b64) = case.bytes_b64.as_deref() {
            b64_decode(b64, &case.id);
        }
    }

    // Originals pin: the corpus's own envelope bytes must parse back to
    // exactly the in-test fixture envelope.
    let originals = &corpus.provenance.originals;
    let original_bytes = b64_decode(&originals.envelope_bytes_b64, "<originals.envelope>");
    let reparsed = SignatureEnvelope::from_bytes(&original_bytes)
        .expect("originals.envelope_bytes_b64 must parse");
    assert_eq!(
        reparsed,
        original_envelope(),
        "corpus originals and in-test envelope fixture have drifted"
    );
    serde_json::from_str::<SealedReceipt>(&originals.sealed_json)
        .expect("originals.sealed_json must parse");
    serde_json::from_str::<HybridSignature>(&originals.hybrid_json)
        .expect("originals.hybrid_json must parse");

    println!(
        "corpus contract OK: {} cases, surfaces {surfaces:?}",
        corpus.cases.len()
    );
}

/// The differential court: every case, on its surface, must (a) never panic
/// and (b) match its pinned expectation, with accept-different checked against
/// the corpus originals. Summary counts printed at the end.
#[test]
fn wire_corpus_cases_never_panic_and_match_pinned_expectations() {
    let corpus = load_corpus();
    let originals = &corpus.provenance.originals;

    let original_envelope_value = original_envelope();
    let original_sealed: SealedReceipt =
        serde_json::from_str(&originals.sealed_json).expect("original sealed parses");
    let original_sealed_reser =
        serde_json::to_string(&original_sealed).expect("original sealed reserializes");
    let original_hybrid: HybridSignature =
        serde_json::from_str(&originals.hybrid_json).expect("original hybrid parses");

    let mut panics: Vec<String> = Vec::new();
    let mut mismatches: Vec<String> = Vec::new();
    let mut summary: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut ran = 0usize;

    for case in &corpus.cases {
        ran += 1;
        *kinds.entry(case.kind.clone()).or_default() += 1;
        let payload = payload_of(case);
        let outcome = match execute(case, &payload) {
            Ok(outcome) => outcome,
            Err(report) => {
                panics.push(report);
                continue;
            }
        };

        let refused = matches!(outcome, Outcome::Refused);
        let summary_key = (
            case.surface.clone(),
            if refused {
                "refuse".to_string()
            } else {
                "ok".to_string()
            },
        );
        *summary.entry(summary_key).or_default() += 1;

        match case.expect.as_str() {
            "refuse" => {
                if !refused {
                    mismatches.push(format!(
                        "case {}: expected refuse, parser ACCEPTED",
                        case.id
                    ));
                }
            }
            "accept-different" => {
                let differs = match &outcome {
                    Outcome::Refused => None,
                    Outcome::EnvelopeOk(envelope) => Some(envelope != &original_envelope_value),
                    Outcome::SealedOk(sealed) => {
                        let reser = serde_json::to_string(sealed.as_ref())
                            .unwrap_or_else(|_| "<unserializable>".to_string());
                        Some(reser != original_sealed_reser)
                    }
                    Outcome::HybridOk(hybrid) => Some(hybrid != &original_hybrid),
                };
                match differs {
                    None => mismatches.push(format!(
                        "case {}: expected accept-different, parser REFUSED",
                        case.id
                    )),
                    Some(false) => mismatches.push(format!(
                        "case {}: expected accept-different, parser ACCEPTED AN EQUAL DOCUMENT (identity mutation)",
                        case.id
                    )),
                    Some(true) => {}
                }
            }
            "any" => {
                // No-panic already witnessed; no further assertion.
            }
            other => mismatches.push(format!("case {}: illegal expect {other:?}", case.id)),
        }
    }

    assert!(
        panics.is_empty(),
        "REAL BUGS: {} case(s) PANICKED a surface parser on malformed input — \
         the rendered modules must refuse, never crash:\n{}",
        panics.len(),
        panics.join("\n")
    );
    assert!(
        mismatches.is_empty(),
        "PINNED EXPECTATIONS VIOLATED: {} case(s) behaved differently than observed at \
         generation time (determinism law: this corpus is static, so any divergence is a \
         module behavior change):\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    assert_eq!(ran, corpus.cases.len(), "court ran every case exactly once");

    println!("wire fuzz court: {ran} cases executed, 0 panics, 0 expectation mismatches");
    for (kind, count) in &kinds {
        println!("  kind {kind}: {count}");
    }
    for ((surface, verdict), count) in &summary {
        println!("  surface {surface} -> {verdict}: {count}");
    }
}
