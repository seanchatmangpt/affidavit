//! Canonical serialization (JCS, RFC 8785 subset) + domain-separated BLAKE3 digests for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Implemented subset, stated honestly (RFC 8785):
//! - Strings, objects, arrays, `null`, `true`, `false`: fully conformant.
//!   Object keys sort by UTF-16 code units (RFC 8785 §3.2.3), NOT by UTF-8
//!   bytes — supplementary-plane keys sort before U+E000..=U+FFFF.
//!   Escaping per RFC 8785 §3.2.2.2: `"` `\` and C0 controls only, with the
//!   `\b \t \n \f \r` shorthands and lowercase `\u00xx` hex otherwise; every
//!   other character is emitted literally.
//! - Numbers (RFC 8785 §3.2.2.3): IEEE-754 doubles rendered by the ECMAScript
//!   `Number::toString` algorithm (fixed notation for decimal exponent n in
//!   -6 < n <= 21, `e+`/`e-` scientific notation otherwise), using Rust's
//!   shortest round-trip digits (`{:e}`). `-0` becomes `0`. Tested against the
//!   RFC 8785 Appendix B vector set below.
//! - Refusals (certify, never coerce): integer literals whose absolute value
//!   exceeds 2^53 are refused with [`CanonicalError::NonCanonicalNumber`] —
//!   JCS numbers are doubles, and such literals are not I-JSON; non-finite
//!   doubles (unrepresentable in canonical JSON) are refused with
//!   [`CanonicalError::Json`]. A `serde_json::Number` cannot hold a non-finite
//!   double through any public constructor, so that route is defensive depth;
//!   it fires on direct `format_double(f64)` calls.
//! - Parse precision note: this module canonicalizes exactly the IEEE-754
//!   double it receives. Which double a JSON literal becomes is decided by the
//!   caller's parser; serde_json's default (non-`float_roundtrip`) parser can
//!   land a 17-significant-digit literal on an adjacent double. The tests
//!   pin both behaviors honestly.

use serde_json::Value;

/// Domain-separation tag mixed into every digest (ctp:policy-v1 ctp:domainTag).
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";
/// Canonicalization identity (ctp:canon-JCS ctp:canonicalizationName).
pub const CANONICALIZATION: &str = "JCS-RFC8785";
/// Digest algorithm over the domain-separated bytes (ctp:canon-JCS ctp:digestAlgorithm).
pub const DIGEST_ALGORITHM: &str = "BLAKE3";
/// Signed-envelope version (ctp:policy-v1 ctp:envelopeVersion).
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// Largest integer exactly representable as an IEEE-754 double: 2^53.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_992;

/// Typed refusal of an uncanonicalizable value.
#[derive(Debug, thiserror::Error)]
pub enum CanonicalError {
    #[error("json canonicalization: {0}")]
    Json(String),
    #[error("number not canonical: {0}")]
    NonCanonicalNumber(String),
}

/// RFC 8785 JSON canonicalization (JCS) of a parsed `serde_json` value.
pub fn jcs(value: &Value) -> Result<String, CanonicalError> {
    let mut out = String::new();
    write_value(&mut out, value)?;
    Ok(out)
}

/// Canonical-byte sink: the one-pass target of [`write_value`]. Implemented
/// for `String` (the render path, [`jcs`]) and for [`blake3::Hasher`] via its
/// `io::Write` impl (the streaming digest path, [`jcs_digest`]) so both paths
/// share one canonical writer and cannot drift.
trait JcsSink {
    /// Neither sink can fail: `String::push_str` is infallible and
    /// `blake3::Hasher`'s `io::Write` impl is unconditionally successful.
    fn sink(&mut self, s: &str);
}

impl JcsSink for String {
    fn sink(&mut self, s: &str) {
        self.push_str(s);
    }
}

/// Adapts [`blake3::Hasher`] to [`JcsSink`]: canonical bytes fold into the
/// digest as they are produced — one pass, no intermediate `String`.
struct HasherSink<'a>(&'a mut blake3::Hasher);

impl JcsSink for HasherSink<'_> {
    fn sink(&mut self, s: &str) {
        use std::io::Write;
        // blake3::Hasher's io::Write impl is write_all-trivially successful;
        // the bound exists for the trait contract, not a real failure mode.
        self.0
            .write_all(s.as_bytes())
            .expect("blake3::Hasher write is infallible");
    }
}

fn write_value<W: JcsSink>(out: &mut W, value: &Value) -> Result<(), CanonicalError> {
    match value {
        Value::Null => out.sink("null"),
        Value::Bool(true) => out.sink("true"),
        Value::Bool(false) => out.sink("false"),
        Value::Number(n) => out.sink(&canonical_number(n)?),
        Value::String(s) => write_escaped(out, s),
        Value::Array(items) => {
            out.sink("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.sink(",");
                }
                write_value(out, item)?;
            }
            out.sink("]");
        }
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
            out.sink("{");
            for (i, (key, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.sink(",");
                }
                write_escaped(out, key);
                out.sink(":");
                write_value(out, val)?;
            }
            out.sink("}");
        }
    }
    Ok(())
}

/// RFC 8785 §3.2.2.2 string serialization: minimal escaping, lowercase hex.
fn write_escaped<W: JcsSink>(out: &mut W, s: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.sink("\"");
    for c in s.chars() {
        match c {
            '"' => out.sink("\\\""),
            '\\' => out.sink("\\\\"),
            '\u{0008}' => out.sink("\\b"),
            '\u{0009}' => out.sink("\\t"),
            '\u{000A}' => out.sink("\\n"),
            '\u{000C}' => out.sink("\\f"),
            '\u{000D}' => out.sink("\\r"),
            ch if (ch as u32) < 0x20 => {
                let v = ch as u32;
                out.sink("\\u00");
                out.sink(
                    (HEX[((v >> 4) & 0xf) as usize] as char)
                        .to_string()
                        .as_str(),
                );
                out.sink((HEX[(v & 0xf) as usize] as char).to_string().as_str());
            }
            c => out.sink(c.to_string().as_str()),
        }
    }
    out.sink("\"");
}

/// RFC 8785 §3.2.2.3 number serialization: ECMAScript `Number::toString` over
/// IEEE-754 doubles, with typed refusal of integer literals beyond double
/// precision (not I-JSON; JCS cannot canonically express them without silently
/// coercing their value).
fn canonical_number(n: &serde_json::Number) -> Result<String, CanonicalError> {
    if let Some(i) = n.as_i64() {
        if i.unsigned_abs() <= MAX_SAFE_INTEGER {
            return Ok(i.to_string());
        }
        return Err(CanonicalError::NonCanonicalNumber(i.to_string()));
    }
    if let Some(u) = n.as_u64() {
        if u <= MAX_SAFE_INTEGER {
            return Ok(u.to_string());
        }
        return Err(CanonicalError::NonCanonicalNumber(u.to_string()));
    }
    let f = n
        .as_f64()
        .ok_or_else(|| CanonicalError::Json(format!("number not representable as f64: {n}")))?;
    if !f.is_finite() {
        return Err(CanonicalError::Json(format!("non-finite number: {n}")));
    }
    if f == 0.0 {
        return Ok("0".to_string());
    }
    if f.fract() == 0.0 && f.abs() < MAX_SAFE_INTEGER as f64 {
        return Ok((f as i64).to_string());
    }
    format_double(f)
}

/// ECMAScript `Number::toString` (ECMA-262 §6.1.6.1.20, incorporated by
/// reference from RFC 8785 §3.2.2.3) for a finite nonzero double.
fn format_double(f: f64) -> Result<String, CanonicalError> {
    let neg = f < 0.0;
    // Rust guarantees `{:e}` is the shortest round-trip form, normalized:
    // d[.ddd]e<exp>, one nonzero digit before the point.
    let exp_form = format!("{:e}", f.abs());
    let (mantissa, exp_text) = exp_form
        .split_once('e')
        .ok_or_else(|| CanonicalError::Json(format!("cannot canonicalize number {f}")))?;
    let exp: i32 = exp_text
        .parse()
        .map_err(|e| CanonicalError::Json(format!("cannot canonicalize number {f}: {e}")))?;
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i32;
    let n = exp + 1; // value = 0.<digits> × 10^n
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    if k <= n && n <= 21 {
        out.push_str(&digits);
        for _ in 0..(n - k) {
            out.push('0');
        }
    } else if n > 0 && n <= 21 {
        out.push_str(&digits[..n as usize]);
        out.push('.');
        out.push_str(&digits[n as usize..]);
    } else if n > -6 && n <= 0 {
        out.push_str("0.");
        for _ in 0..(-n) {
            out.push('0');
        }
        out.push_str(&digits);
    } else {
        out.push_str(&digits[..1]);
        if k > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        let e = n - 1;
        if e > 0 {
            out.push('+');
        } else {
            out.push('-');
        }
        out.push_str(&e.abs().to_string());
    }
    Ok(out)
}

/// Domain-separated pre-image: `DOMAIN_TAG || 0x00 || domain || 0x00 ||` for
/// each part its 8-byte big-endian length then the part bytes. The explicit
/// length prefixes make part boundaries unambiguous under concatenation.
pub fn domain_separated(domain: &str, parts: &[&[u8]]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(DOMAIN_TAG.as_bytes());
    buf.push(0x00);
    buf.extend_from_slice(domain.as_bytes());
    buf.push(0x00);
    for part in parts {
        buf.extend_from_slice(&(part.len() as u64).to_be_bytes());
        buf.extend_from_slice(part);
    }
    buf
}

/// BLAKE3 digest (32 bytes) of the domain-separated pre-image.
pub fn digest(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    blake3::hash(&domain_separated(domain, parts)).into()
}

/// Lowercase-hex rendering of [`digest`].
pub fn digest_hex(domain: &str, parts: &[&[u8]]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let d = digest(domain, parts);
    let mut s = String::with_capacity(64);
    for b in d {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// One-pass streaming digest of the canonical (JCS) bytes of `value`: the
/// canonical writer feeds a [`blake3::Hasher`] directly, so no intermediate
/// canonical `String` is ever materialized. Byte-equivalent to
/// `blake3::hash(jcs(value)?.as_bytes())` — differential-tested below.
pub fn jcs_digest(value: &Value) -> Result<[u8; 32], CanonicalError> {
    let mut hasher = blake3::Hasher::new();
    write_value(&mut HasherSink(&mut hasher), value)?;
    Ok(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canon(input: &str) -> String {
        let value: Value = serde_json::from_str(input).expect("test json parses");
        jcs(&value).expect("test value canonicalizes")
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(CANONICALIZATION, "JCS-RFC8785");
        assert_eq!(DIGEST_ALGORITHM, "BLAKE3");
        assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
    }

    #[test]
    fn rfc8785_vectors() {
        let cases: &[(&str, &str)] = &[
            (r#"{}"#, r#"{}"#),
            (r#"[]"#, r#"[]"#),
            (r#""abc""#, r#""abc""#),
            (r#"null"#, r#"null"#),
            (r#"true"#, r#"true"#),
            (r#"{"b":1,"a":2}"#, r#"{"a":2,"b":1}"#),
            // RFC 8785 Appendix B: number array. The first literal is written
            // in its canonical form because serde_json's default parser is not
            // correctly rounded on 17-significant-digit literals (see the
            // exact-double test below, which pins the RFC's original input).
            (
                r#"{"numbers":[333333333.3333333,1E30,4.50,2e-3,0.000000000000000000000000001]}"#,
                r#"{"numbers":[333333333.3333333,1e+30,4.5,0.002,1e-27]}"#,
            ),
            // RFC 8785 Appendix B: string escaping (EUR sign, C0 control, LF,
            // quote, backslashes, solidus)
            (
                r#"{"string":"\u20ac$\u000F\u000aA'\u0042\u0022\u005c\\\"/"}"#,
                r#"{"string":"€$\u000f\nA'B\"\\\\\"/"}"#,
            ),
            // nested container ordering
            (
                r#"{"o":{"b":1,"a":[true,false,null,1.5]}}"#,
                r#"{"o":{"a":[true,false,null,1.5],"b":1}}"#,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(&canon(input), expected, "vector failed for {input}");
        }
    }

    #[test]
    fn rfc8785_number_forms() {
        let cases: &[(&str, &str)] = &[
            (r#"[1.0]"#, r#"[1]"#),
            (r#"[-0, -0.0]"#, r#"[0,0]"#),
            (r#"[0.000001]"#, r#"[0.000001]"#),
            (r#"[1e-6]"#, r#"[0.000001]"#),
            (r#"[1e-7]"#, r#"[1e-7]"#),
            (r#"[1e20]"#, r#"[100000000000000000000]"#),
            (r#"[1e21]"#, r#"[1e+21]"#),
            (r#"[1e16]"#, r#"[10000000000000000]"#),
            (r#"[9.007199254740992e15]"#, r#"[9007199254740992]"#),
            (r#"[9007199254740992]"#, r#"[9007199254740992]"#),
            (r#"[5e-324]"#, r#"[5e-324]"#),
            (
                r#"[1.7976931348623157e308]"#,
                r#"[1.7976931348623157e+308]"#,
            ),
            (r#"[-1.5]"#, r#"[-1.5]"#),
            (r#"[-5.0]"#, r#"[-5]"#),
            (r#"[2e-3]"#, r#"[0.002]"#),
        ];
        for (input, expected) in cases {
            assert_eq!(&canon(input), expected, "number vector failed for {input}");
        }
    }

    #[test]
    fn utf16_code_unit_key_order() {
        // U+10000 (UTF-16 units D800 DC00) must sort BEFORE U+FFFD (unit FFFD),
        // although UTF-8 byte order says the opposite. A naive BTreeMap pass
        // emits U+FFFD first; JCS requires U+10000 first (RFC 8785 §3.2.3).
        let value: Value =
            serde_json::from_str(r#"{"\ufffd":1,"\ud800\udc00":2}"#).expect("parses");
        let mut expected = String::new();
        expected.push('{');
        expected.push('"');
        expected.push('\u{10000}');
        expected.push('"');
        expected.push_str(":2,\"");
        expected.push('\u{FFFD}');
        expected.push('"');
        expected.push_str(":1}");
        assert_eq!(jcs(&value).expect("canonicalizes"), expected);
    }

    #[test]
    fn canonical_form_is_fixed_point() {
        let input = r#"{"b":1,"a":[1.0,-0.0],"s":"\u00e9\ufffd"}"#;
        let once = canon(input);
        let twice = canon(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn jcs_is_deterministic() {
        let input = r#"{"z":[1,{"k":"v"},"à"],"_":null,"A":1e30}"#;
        let a = canon(input);
        let b = canon(input);
        assert_eq!(a, b);
    }

    #[test]
    fn integers_beyond_double_precision_refused() {
        for input in [
            "18446744073709551615",
            "9223372036854775807",
            "-9223372036854775808",
        ] {
            let value: Value = serde_json::from_str(input).expect("test json parses");
            match jcs(&value) {
                Err(CanonicalError::NonCanonicalNumber(_)) => {}
                other => panic!("expected NonCanonicalNumber for {input}, got {other:?}"),
            }
        }
    }

    #[test]
    fn rfc8785_appendix_b_number_via_exact_double() {
        // The RFC's original input literal 333333333.33333329 and its expected
        // output 333333333.3333333 denote the SAME IEEE-754 double (std's
        // correctly-rounded parser confirms identical bits). Canonicalizing
        // that exact double must yield the RFC-expected string.
        let x: f64 = "333333333.33333329"
            .parse()
            .expect("std f64 parse is correctly rounded");
        let value = Value::Number(serde_json::Number::from_f64(x).expect("finite"));
        assert_eq!(jcs(&value).expect("canonicalizes"), "333333333.3333333");
    }

    #[test]
    fn canonicalizes_exactly_the_double_it_receives() {
        // serde_json's default parser maps the RFC's 17-digit literal to the
        // adjacent double (fast-path, not float_roundtrip). The module must
        // not compensate for parse imprecision: adjacent double in, faithful
        // JCS of that double out.
        let value: Value = serde_json::from_str("333333333.33333329").expect("parses");
        assert_eq!(jcs(&value).expect("canonicalizes"), "333333333.33333325");
    }

    #[test]
    fn non_finite_double_refused() {
        // serde_json's public constructors cannot place a non-finite double in
        // a Number (From<f64> maps NaN/inf to Null; the parser rejects
        // out-of-range literals), so this fires the defensive route inside
        // format_double directly; the route keeps it total over f64.
        for f in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            match format_double(f) {
                Err(CanonicalError::Json(_)) => {}
                other => panic!("expected Json refusal for {f}, got {other:?}"),
            }
        }
    }

    #[test]
    fn domain_separated_layout_is_exact() {
        let buf = domain_separated("dom", &[b"ab", b"c"]);
        let mut expected: Vec<u8> = Vec::new();
        expected.extend_from_slice(DOMAIN_TAG.as_bytes());
        expected.push(0x00);
        expected.extend_from_slice(b"dom");
        expected.push(0x00);
        expected.extend_from_slice(&2u64.to_be_bytes());
        expected.extend_from_slice(b"ab");
        expected.extend_from_slice(&1u64.to_be_bytes());
        expected.extend_from_slice(b"c");
        assert_eq!(buf, expected);
    }

    #[test]
    fn part_boundaries_are_unambiguous() {
        // Both lists concatenate to the same bytes; length prefixes must keep
        // the pre-images (and therefore the digests) distinct.
        assert_ne!(
            domain_separated("d", &[b"ab", b"c"]),
            domain_separated("d", &[b"a", b"bc"])
        );
        assert_ne!(digest("d", &[b"ab", b"c"]), digest("d", &[b"a", b"bc"]));
    }

    #[test]
    fn domain_separates_digests() {
        assert_ne!(digest(DOMAIN_TAG, &[b"x"]), digest("other.domain", &[b"x"]));
        assert_ne!(digest("", &[b"x"]), digest("\u{0}", &[b"x"]));
    }

    #[test]
    fn digest_is_32_bytes_and_hex_is_64_lowercase() {
        let d = digest("dom", &[b"payload"]);
        assert_eq!(d.len(), 32);
        let h = digest_hex("dom", &[b"payload"]);
        assert_eq!(h.len(), 64);
        assert!(
            h.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hex must be lowercase: {h}"
        );
    }

    #[test]
    fn digest_is_deterministic() {
        assert_eq!(
            digest_hex("d", &[b"a", b"b"]),
            digest_hex("d", &[b"a", b"b"])
        );
    }

    #[test]
    fn empty_parts_digest_is_blake3_of_header() {
        let d = digest("dom", &[]);
        let e: [u8; 32] = blake3::hash(&domain_separated("dom", &[])).into();
        assert_eq!(d, e);
        assert_ne!(d, [0u8; 32]);
    }

    // --- streaming digest court: differential equivalence + mutation control ---

    fn parsed(input: &str) -> Value {
        serde_json::from_str(input).expect("test json parses")
    }

    #[test]
    fn jcs_digest_matches_blake3_of_jcs_string_over_corpus() {
        let corpus: &[&str] = &[
            r#"null"#,
            r#"true"#,
            r#"false"#,
            r#"0"#,
            r#""abc""#,
            r#"[]"#,
            r#"{}"#,
            r#"{"b":1,"a":2}"#,
            r#"{"numbers":[333333333.3333333,1E30,4.50,2e-3,0.000000000000000000000000001]}"#,
            r#"{"string":"\u20ac$\u000F\u000aA'\u0042\u0022\u005c\\\"/"}"#,
            r#"{"o":{"b":1,"a":[true,false,null,1.5]}}"#,
            r#"[1e-7,1e20,1e21,9.007199254740992e15,5e-324,-0.0,-1.5]"#,
            r#"{"�":1,"𐀀":2}"#,
        ];
        for input in corpus {
            let value = parsed(input);
            let via_string: [u8; 32] =
                blake3::hash(jcs(&value).expect("canonicalizes").as_bytes()).into();
            let via_writer = jcs_digest(&value).expect("streams");
            assert_eq!(
                via_writer, via_string,
                "writer-path digest diverged from string path for {input}"
            );
        }
    }

    #[test]
    fn jcs_digest_handles_large_synthetic_record() {
        // 5,000 keys x nested arrays: large enough that per-chunk hashing
        // across many sink calls must still fold identically.
        let mut value = serde_json::Map::new();
        for i in 0..5_000u32 {
            let key = format!("field_{i:04}");
            let inner = serde_json::json!({
                "idx": i,
                "ratio": f64::from(i) / 7.5,
                "tags": [format!("t{i}"), null, i % 2 == 0],
            });
            value.insert(key, inner);
        }
        let big = Value::Object(value);
        let via_string: [u8; 32] =
            blake3::hash(jcs(&big).expect("canonicalizes").as_bytes()).into();
        assert_eq!(
            jcs_digest(&big).expect("streams"),
            via_string,
            "large-record writer digest diverged"
        );
    }

    #[test]
    fn jcs_digest_is_a_real_mutation_control() {
        // A dropped key and a reordered array must each flip the digest; a
        // semantically identical re-serialization must not.
        let base = parsed(r#"{"alpha":[1,2,3],"beta":{"k":"v"},"gamma":1.5}"#);
        let baseline = jcs_digest(&base).expect("streams");

        let dropped = parsed(r#"{"alpha":[1,2,3],"beta":{"k":"v"}}"#);
        assert_ne!(
            jcs_digest(&dropped).expect("streams"),
            baseline,
            "dropping a key did not flip the digest"
        );

        let reordered_array = parsed(r#"{"alpha":[3,2,1],"beta":{"k":"v"},"gamma":1.5}"#);
        assert_ne!(
            jcs_digest(&reordered_array).expect("streams"),
            baseline,
            "array reordering did not flip the digest"
        );

        // Same content, keys written in reverse insertion order: JCS sorts, so
        // the digest must be IDENTICAL (order-invariance, not a mutation).
        let reordered_keys = parsed(r#"{"gamma":1.5,"beta":{"k":"v"},"alpha":[1,2,3]}"#);
        assert_eq!(
            jcs_digest(&reordered_keys).expect("streams"),
            baseline,
            "key reordering changed the digest (JCS sort broke?)"
        );

        // And the writer path flips exactly when the string path flips.
        let flipped_string_path: [u8; 32] =
            blake3::hash(jcs(&dropped).expect("canonicalizes").as_bytes()).into();
        assert_ne!(flipped_string_path, baseline);
    }

    #[test]
    fn jcs_digest_propagates_non_canonical_number_refusal() {
        let value = parsed("18446744073709551615");
        match jcs_digest(&value) {
            Err(CanonicalError::NonCanonicalNumber(_)) => {}
            other => panic!("expected NonCanonicalNumber from streaming path, got {other:?}"),
        }
    }
}
