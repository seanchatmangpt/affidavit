#!/usr/bin/env python3
"""JCS (RFC 8785) cross-runtime differential harness -- Python side.

Lane W5-W1-L6. This tool is the INDEPENDENT second implementation of RFC 8785
JSON canonicalization used to differentially prove the ggen-rendered
``affidavit::crypto_trust_canonical::jcs`` (Rust side). Neither side is
authoritative; disagreement on the same IEEE-754 double / same parsed string
is a real conformance bug in exactly one of them.

What it does (single invocation, no network, stdlib only):

1. Generates a structured corpus of ~250 JSON values exercising the three
   conformance axes:
     - key ordering by UTF-16 code units (RFC 8785 section 3.2.3), including
       supplementary-plane keys vs BMP keys >= U+E000;
     - ECMAScript ``Number::toString`` number rendering (section 3.2.2.3)
       across the boundary set (+-0, 1e-7, 1e21, 2^53, 5e-324, 1.797e308,
       shortest-roundtrip decimals, denormals, powers of ten);
     - string escaping (section 3.2.2.2): every C0 control, the six
       shorthands, lowercase ``\\u00xx``, minimal escaping otherwise.
2. Writes ``fixtures/crypto_trust_jcs_corpus.json``
   (schema: ``{"meta": ..., "cases": [{"id": ..., "value": <JSON>}]}``).
3. Canonicalizes every case with the Python RFC 8785 implementation below and
   writes the sidecar ``fixtures/crypto_trust_jcs_expected.json`` holding the
   Python-expected output per case id. Scalar float cases additionally carry
   ``value_f64_bits`` (hex of the exact IEEE-754 double Python parsed) so the
   Rust test can distinguish a REAL canonicalization divergence from a
   serde_json parse-precision artifact (serde_json's default parser is not
   correctly rounded on long literals; Python's ``float()`` is).
4. Refusal symmetry: integer literals with |v| > 2^53 are not I-JSON; the
   rendered Rust jcs refuses them (``NonCanonicalNumber``) and this
   implementation refuses them identically. The sidecar records the refusal.

Composite (object/array) cases carry only numbers whose shortest-roundtrip
digit count is <= 15, which both runtimes parse exactly (serde_json's default
parser is exact in that range, Python's float() is always correctly rounded);
any composite divergence is therefore a genuine conformance divergence.

Usage:  python3 tools/jcs_differential.py
Output is deterministic (no timestamps, insertion-ordered, no set iteration).
"""

import json
import math
import os
import struct
import sys

MAX_SAFE_INTEGER = 2 ** 53

CORPUS_RELPATH = os.path.join("fixtures", "crypto_trust_jcs_corpus.json")
EXPECTED_RELPATH = os.path.join("fixtures", "crypto_trust_jcs_expected.json")


class NonCanonicalNumber(Exception):
    """Mirrors CanonicalError::NonCanonicalNumber on the Rust side."""


# ---------------------------------------------------------------------------
# Python RFC 8785 canonicalization (independent implementation)
# ---------------------------------------------------------------------------

_SHORTHANDS = {
    '"': '\\"',
    "\\": "\\\\",
    "\b": "\\b",
    "\t": "\\t",
    "\n": "\\n",
    "\f": "\\f",
    "\r": "\\r",
}


def serialize_string(s):
    """RFC 8785 section 3.2.2.2: minimal escaping, lowercase hex."""
    out = ['"']
    for ch in s:
        esc = _SHORTHANDS.get(ch)
        if esc is not None:
            out.append(esc)
        elif ord(ch) < 0x20:
            out.append("\\u%04x" % ord(ch))
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def serialize_integer(i):
    """I-JSON policy mirrored from the rendered Rust side: integer literals
    beyond double precision are refused, never coerced."""
    if abs(i) > MAX_SAFE_INTEGER:
        raise NonCanonicalNumber("integer beyond double precision: %d" % i)
    return str(i)


def es_digits(f):
    """Shortest round-trip digits + decimal exponent n of a finite nonzero
    float, in the RFC 8785 normalization  value = 0.<digits> x 10^n.

    Python's repr() is shortest-roundtrip (same digit selection as Rust's
    ``{:e}``; both pick the shortest, closest candidate). Handles repr's fixed
    ('123.5', '9007199254740992.0') and scientific ('1e-07', '5e-324') forms.
    """
    s = repr(abs(f))
    mant, _, exp_text = s.partition("e")
    exp = int(exp_text) if exp_text else 0
    if "." in mant:
        intpart, fracpart = mant.split(".")
    else:
        intpart, fracpart = mant, ""
    raw = intpart + fracpart
    pos = len(intpart) + exp  # decimal point position within raw digits
    # Drop trailing zeros that live after the decimal point (repr's ".0"
    # integral marker); zeros inside the integer magnitude are significant.
    while len(raw) > pos and raw.endswith("0"):
        raw = raw[:-1]
    stripped = raw.lstrip("0")
    lead = len(raw) - len(stripped)
    return (stripped if stripped else "0"), pos - lead


def serialize_double(f):
    """ECMAScript Number::toString (ECMA-262 6.1.6.1.20) for a finite double,
    same branch structure as the rendered Rust ``format_double``."""
    if math.isnan(f) or math.isinf(f):
        raise NonCanonicalNumber("non-finite double")
    if f == 0.0:  # catches -0.0 as well
        return "0"
    if f.is_integer() and abs(f) < MAX_SAFE_INTEGER:
        return str(int(f))
    digits, n = es_digits(f)
    k = len(digits)
    if k <= n <= 21:
        body = digits + "0" * (n - k)
    elif 0 < n <= 21:
        body = digits[:n] + "." + digits[n:]
    elif -6 < n <= 0:
        body = "0." + "0" * (-n) + digits
    else:
        body = digits[0]
        if k > 1:
            body += "." + digits[1:]
        e = n - 1
        body += "e" + ("+" if e > 0 else "-") + str(abs(e))
    return ("-" if f < 0 else "") + body


def canonicalize(value):
    """RFC 8785 canonical JSON text of a parsed-Python value. Raises
    NonCanonicalNumber where the rendered Rust side refuses."""
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        return serialize_string(value)
    if isinstance(value, int):
        return serialize_integer(value)
    if isinstance(value, float):
        return serialize_double(value)
    if isinstance(value, list):
        return "[" + ",".join(canonicalize(item) for item in value) + "]"
    if isinstance(value, dict):
        keys = sorted(value.keys(), key=lambda k: k.encode("utf-16-be"))
        members = [
            serialize_string(key) + ":" + canonicalize(value[key]) for key in keys
        ]
        return "{" + ",".join(members) + "}"
    raise TypeError("not a JSON value: %r" % (value,))


# ---------------------------------------------------------------------------
# Corpus generation
# ---------------------------------------------------------------------------

def shortest_digit_count(f):
    digits, _ = es_digits(f)
    return len(digits)


def assert_composite_parse_safe(value, case_id):
    """Composite cases must hold only numbers both runtimes parse identically:
    exact integers (|v| <= 2^53; refusals are scalar-only cases) and floats
    whose shortest digit count is <= 15 (serde_json's default parser is exact
    there; Python's float() is correctly rounded everywhere)."""
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return
    if isinstance(value, int):
        if abs(value) > MAX_SAFE_INTEGER:
            raise AssertionError(
                "composite case %s holds a refusal-class integer" % case_id
            )
        return
    if isinstance(value, float):
        if shortest_digit_count(value) > 15:
            raise AssertionError(
                "composite case %s holds a >15-digit float (parse-unsafe)"
                % case_id
            )
        return
    if isinstance(value, list):
        for item in value:
            assert_composite_parse_safe(item, case_id)
        return
    if isinstance(value, dict):
        for key, item in value.items():
            assert_composite_parse_safe(item, case_id)
        return
    raise AssertionError("composite case %s holds a non-JSON leaf" % case_id)


def build_cases():
    cases = []

    def add(case_id, value, composite=False):
        if any(existing["id"] == case_id for existing in cases):
            raise AssertionError("duplicate case id: %s" % case_id)
        if composite:
            assert_composite_parse_safe(value, case_id)
        cases.append({"id": case_id, "value": value})

    # -- scalars: null / bool ------------------------------------------------
    add("scalar_null", None)
    add("scalar_true", True)
    add("scalar_false", False)

    # -- integers inside the safe range --------------------------------------
    ints_in_range = [
        0, 1, -1, 42, -42, 255, -255, 65536, 2 ** 31 - 1, -(2 ** 31),
        10 ** 15, 999999999999999, 123456789012345,
        MAX_SAFE_INTEGER - 1, MAX_SAFE_INTEGER, -MAX_SAFE_INTEGER,
        -MAX_SAFE_INTEGER + 1,
    ]
    for i in ints_in_range:
        add("int_%d" % i, i)

    # -- integer refusals (|v| > 2^53, still within i64/u64 literal range) ----
    refusals = [
        MAX_SAFE_INTEGER + 1,
        -(MAX_SAFE_INTEGER + 1),
        MAX_SAFE_INTEGER + 2,
        2 ** 62,
        2 ** 63 - 1,
        -(2 ** 63),
        2 ** 64 - 1,
        12345678901234567890,
    ]
    for i in refusals:
        add("int_refusal_%d" % i, i)

    # -- float boundary set ---------------------------------------------------
    float_boundary = [
        0.0, -0.0,
        0.5, -0.5, 1.5, -1.5, 0.1, -0.1, 0.2, 0.3, 1.0 / 3.0, 2.0 / 3.0,
        1e-7, -1e-7, 1e-6, 9.999999999999999e-7, 1e-5, 1e-4, 0.001, 0.002,
        1e20, 1e21, -1e21, 1e22, 9.999999999999998e20, 1e16, 1e17,
        float(MAX_SAFE_INTEGER), -float(MAX_SAFE_INTEGER),
        9.007199254740992e15,
        1.152921504606847e18,
        5e-324, -5e-324, 1e-320, 4.9e-324,
        2.2250738585072014e-308,
        1.7976931348623157e308, -1.7976931348623157e308,
        1.7976931348623155e308,
        1.797e308, -1.797e308,
        math.pi, math.e,
        6.02214076e23, 1.602176634e-19, 6.62607015e-34, 2.99792458e8,
        299792458.0,
        0.30000000000000004, 1.1, 2.2, 3.3, 4.4,
        1.5e-8, 2.5e-8, -2.5e-8,
        123456.789, 1234567890.12345,
    ]
    # (ids are positional: distinct doubles can share a shortest repr, so
    # repr-derived ids are not unique)
    for idx, f in enumerate(float_boundary):
        add("float_%03d" % idx, f)

    # -- powers of ten sweep (single-significand-digit literals) --------------
    for k in range(-320, 309, 8):
        add("pow10_exp%d" % k, 10.0 ** k)

    # -- dyadic rationals (exact, short) + repeating decimals -----------------
    for i in range(1, 25):
        add("dyadic_%d_over_64" % i, i / 64.0)
    for i in range(1, 7):
        add("repeating_%d_over_7" % i, i / 7.0)

    # -- strings: every C0 control individually -------------------------------
    for o in range(0x20):
        add(
            "ctrl_u%04x" % o,
            "x%sy" % chr(o),
        )
    # -- strings: boundaries, astral plane, scripts ---------------------------
    strings = [
        "", "a", "ascii text", '"', "\\",
        "\x7f", "\u0080", "é", "€", "中",
        "😀", "𝄞", "\U00010000", "", "�",
        "\ue000", "\ud7ff", "a\u0301", "עברית", "日本語",
        "line\u2028sep", "zwj‍", "tab\tnewline\ncarriage\r",
        "solidus/not/escaped", "back\\slash\"quote",
        "👨‍👩‍👧", "\ud7ff\ue000", "😀A", "\U0010FFFF",
        "ééé€€€", "a" * 64,
    ]
    for idx, s in enumerate(strings):
        add("string_%02d" % idx, s)

    # -- objects / arrays ------------------------------------------------------
    add("obj_empty", {}, composite=True)
    add("arr_empty", [], composite=True)
    add("obj_two_keys", {"b": 1, "a": 2}, composite=True)
    add(
        "obj_utf16_order_torture",
        {
            "\U0001F335": 1, "￿": 2, "é": 3, "A": 4, "ÿ": 5,
            "�": 6, "\U00010000": 7, "": 8, "Z": 9, "a": 10,
        },
        composite=True,
    )
    add("obj_astral_vs_uffff", {"😀": 1, "￿": 2}, composite=True)
    add("obj_ue000_vs_u10000", {"": 1, "\U00010000": 2}, composite=True)
    add(
        "obj_nested",
        {"o": {"b": 1, "a": [True, False, None, 1.5]}, "z": [[], {}]},
        composite=True,
    )
    deep = 1.5
    for _ in range(40):
        deep = [deep]
    add("arr_deep_40", deep, composite=True)
    deep_obj = {"leaf": True}
    for i in range(25):
        deep_obj = {"k%02d" % i: deep_obj}
    add("obj_deep_25", deep_obj, composite=True)
    add(
        "obj_appendix_b_numbers",
        {"numbers": [333333333.333333, 1e30, 4.5, 0.002, 1e-27]},
        composite=True,
    )
    add(
        "arr_number_boundary_mix",
        [0, -1, 1.5, -1.5, 1e21, 1e-7, 5e-324, 1e-6, 1e20, MAX_SAFE_INTEGER],
        composite=True,
    )
    add(
        "obj_all_value_kinds",
        {
            "null": None, "bool": True, "num": 1.5, "str": "x",
            "arr": [1], "obj": {"k": False},
        },
        composite=True,
    )
    add(
        "obj_escaped_keys",
        {"tab\tkey": 1, 'quote"key': 2, "back\\slash": 3, "new\nline": 4},
        composite=True,
    )
    add("obj_empty_key", {"": "empty"}, composite=True)
    add("arr_quad_nested", [[[[1]]]], composite=True)
    add(
        "arr_nested_mixed",
        [{"nested": [{"deep": [True]}]}, [None, -0.5]],
        composite=True,
    )
    add(
        "obj_unicode_values_and_keys",
        {"ékey": "évalue", "中key": "中value", "😀key": "😀value"},
        composite=True,
    )
    add(
        "arr_fifty_ints",
        list(range(50)),
        composite=True,
    )
    add(
        "obj_reverse_alphabet",
        {chr(ord("z") - i): i for i in range(26)},
        composite=True,
    )
    add("obj_numeric_string_keys", {"1": 1, "10": 2, "2": 3}, composite=True)
    add("obj_case_ordering", {"A": 1, "a": 2, "B": 3}, composite=True)
    add("obj_accent_vs_plain", {"é": 1, "e": 2}, composite=True)
    add(
        "obj_control_string_values",
        {"c0": "\x00\x1f\x7f", "shorthands": "\b\t\n\f\r", "quotes": "\"\\"},
        composite=True,
    )
    add(
        "obj_kitchen_sink",
        {
            "z": [1, {"k": "v"}, "é"],
            "_": None,
            "A": 1e30,
            "😀": {"\u00e9": [5e-324, 1e21]},
        },
        composite=True,
    )

    return cases


# ---------------------------------------------------------------------------
# Main: emit corpus + expected sidecar
# ---------------------------------------------------------------------------

def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    corpus_path = os.path.join(repo_root, CORPUS_RELPATH)
    expected_path = os.path.join(repo_root, EXPECTED_RELPATH)

    cases = build_cases()

    counts = {"canonical": 0, "refusal": 0, "float_scalar": 0}
    expected_cases = []
    for case in cases:
        value = case["value"]
        try:
            canonical = canonicalize(value)
        except NonCanonicalNumber:
            counts["refusal"] += 1
            expected_cases.append(
                {"id": case["id"], "refusal": "NonCanonicalNumber"}
            )
            continue
        counts["canonical"] += 1
        entry = {"id": case["id"], "canonical": canonical}
        if isinstance(value, float) and not isinstance(value, bool):
            bits = struct.unpack(">Q", struct.pack(">d", value))[0]
            entry["value_f64_bits"] = "0x%016x" % bits
            counts["float_scalar"] += 1
        expected_cases.append(entry)

    corpus_doc = {
        "meta": {
            "generator": "tools/jcs_differential.py",
            "schema": "cases[{id, value}]",
            "note": (
                "RFC 8785 JCS cross-runtime differential corpus (lane "
                "W5-W1-L6). Values are parsed JSON; canonicalization is "
                "compared against fixtures/crypto_trust_jcs_expected.json."
            ),
        },
        "cases": cases,
    }
    expected_doc = {
        "meta": {
            "generator": "tools/jcs_differential.py",
            "schema": (
                "cases[{id, canonical}|{id, refusal}] plus value_f64_bits "
                "for scalar float cases (hex IEEE-754 bits of the double "
                "Python parsed; used by the Rust test to separate "
                "serde_json parse-precision artifacts from real "
                "canonicalization divergences)"
            ),
        },
        "cases": expected_cases,
    }

    with open(corpus_path, "w", encoding="utf-8") as handle:
        json.dump(corpus_doc, handle, ensure_ascii=False, separators=(",", ":"))
        handle.write("\n")
    with open(expected_path, "w", encoding="utf-8") as handle:
        json.dump(expected_doc, handle, ensure_ascii=False, separators=(",", ":"))
        handle.write("\n")

    print("JCS differential corpus generated (Python side, RFC 8785)")
    print("  total cases:            %d" % len(cases))
    print("  canonical comparisons:  %d" % counts["canonical"])
    print("  refusal comparisons:    %d" % counts["refusal"])
    print("  scalar float w/ bits:   %d" % counts["float_scalar"])
    print("  wrote: %s" % os.path.relpath(corpus_path, repo_root))
    print("  wrote: %s" % os.path.relpath(expected_path, repo_root))
    return 0


if __name__ == "__main__":
    sys.exit(main())
