#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Cross-runtime KAT verifier for the affidavit cryptographic trust plane.

Wave 4 lane W4-L8. Independently (outside the Rust crate) verifies the
published fixture `fixtures/crypto_trust_kat.json`, whose corpus is the JCS
export of `src/crypto_trust_kat.rs::generate_corpus()` -- the single source
of truth.

Honest scope of this verifier:

- JCS (RFC 8785) canonicalization: implemented for THESE objects -- arrays
  and string-only objects (every corpus field is a lowercase-hex string or
  an identifier). Key order is UTF-16 code units (RFC 8785 section 3.2.3);
  escaping per section 3.2.2.2. Numbers/bools/null in the document are
  refused loudly (SKIP, never faked).
- domain_separated port of src/crypto_trust_canonical.rs: the pre-image is
  DOMAIN_TAG || 0x00 || domain || 0x00 || (u64be len || part)* -- note the
  constant plane tag AND the domain parameter both prefix the parts.
- BLAKE3: vendored pure-Python implementation (hashlib has no blake3),
  hash mode, 32-byte outputs, single-chunk inputs (<= 1024 bytes; every KAT
  pre-image is ~105 bytes). Self-checked against the official test vectors
  for b"" and b"\\x00" before any use; differentially tested against the
  blake3 crate for input lengths 0..1024.
- Signing-input law: per the rendered source, each KAT vector signs
    message = blake3(domain_separated(KAT_DOMAIN_TAG,
                                       [b"ctp.kat.message", index_be]))
  where index is the vector's corpus position (graph registry order).
  (The domain_separated(DOMAIN_TAG, [jcs(document)]) construction is the
  ENVELOPE module's signing input; KAT vectors do not sign envelopes --
  the rendered KAT source is authoritative.)
- ES256 (P-256): public keys re-derived from the seed scalars with the
  system openssl (RFC 5915 SEC1 PEM -> `openssl ec -pubout`) and compared
  against the fixture; signatures (RFC 6979 deterministic ECDSA, DER) are
  verified with `openssl dgst -sha256 -verify` over the derived message
  bytes. The ES256 half of the hybrid vector is verified the same way
  after splitting at its self-delimiting DER prefix. One negative control
  (a tampered message MUST be refused) proves the pipeline is not
  vacuously green.
- ML-DSA-65 / SLH-DSA-SHA2-128s signatures (and the ML-DSA half of the
  hybrid): no independent verifier exists outside the Rust crate here, so
  they print SKIPPED(rust-only) -- never faked.

Exit 0 iff every non-skipped check passes and at least one check ran.
"""

import base64
import json
import os
import struct
import subprocess
import sys
import tempfile

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURE_PATH = os.path.join(REPO_ROOT, "fixtures", "crypto_trust_kat.json")

KAT_SCHEMA_VERSION = "CTP-KAT-v1"
KAT_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1"
EXPECTED_HEADER_KEYS = ["corpus", "generated_from", "schema_version"]
SEED_ROLE = {
    "ES256": [("es256", 32)],
    "ML-DSA-65": [("mldsa65", 32)],
    "ES256+ML-DSA-65": [("hybrid-es256", 32), ("hybrid-mldsa65", 32)],
}
P256_SPKI_HEADER = bytes.fromhex(
    "3059301306072a8648ce3d020106082a8648ce3d030107034200"
)
PRIME256V1_OID = bytes.fromhex("06082a8648ce3d030107")

# Official BLAKE3 test vectors (hash mode, 32-byte output).
BLAKE3_SELF_CHECK = [
    (b"", "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"),
    (b"\x00", "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"),
]

# ── Pure-Python BLAKE3 (hash mode, 32-byte output, single chunk) ────────────

MASK32 = 0xFFFFFFFF
IV = (0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
      0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19)
MSG_PERMUTATION = (2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8)
CHUNK_START, CHUNK_END, ROOT = 1, 2, 8
CHUNK_LEN = 1024


def _rotr(x, n):
    return ((x >> n) | (x << (32 - n))) & MASK32


def _g(s, a, b, c, d, mx, my):
    s[a] = (s[a] + s[b] + mx) & MASK32
    s[d] = _rotr(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & MASK32
    s[b] = _rotr(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b] + my) & MASK32
    s[d] = _rotr(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & MASK32
    s[b] = _rotr(s[b] ^ s[c], 7)


def _round(s, m):
    _g(s, 0, 4, 8, 12, m[0], m[1])
    _g(s, 1, 5, 9, 13, m[2], m[3])
    _g(s, 2, 6, 10, 14, m[4], m[5])
    _g(s, 3, 7, 11, 15, m[6], m[7])
    _g(s, 0, 5, 10, 15, m[8], m[9])
    _g(s, 1, 6, 11, 12, m[10], m[11])
    _g(s, 2, 7, 8, 13, m[12], m[13])
    _g(s, 3, 4, 9, 14, m[14], m[15])


def _compress(cv, block_words, counter, block_len, flags):
    state = [cv[0], cv[1], cv[2], cv[3], cv[4], cv[5], cv[6], cv[7],
             IV[0], IV[1], IV[2], IV[3],
             counter & MASK32, (counter >> 32) & MASK32, block_len, flags]
    m = list(block_words)
    for _ in range(7):
        _round(state, m)
        m = [m[MSG_PERMUTATION[j]] for j in range(16)]
    for i in range(8):
        state[i] ^= state[i + 8]
        state[i + 8] ^= cv[i]
    return state


def blake3_32(data):
    """BLAKE3 hash, 32 bytes. Subset: single-chunk inputs only (<= 1024)."""
    if len(data) > CHUNK_LEN:
        raise ValueError("blake3 subset: single-chunk inputs only (got %d bytes)" % len(data))
    cv = list(IV)
    if len(data) == 0:
        cv = _compress(cv, [0] * 16, 0, 0, CHUNK_START | CHUNK_END | ROOT)[:8]
    for i in range(0, len(data), 64):
        block = data[i:i + 64]
        words = struct.unpack("<16I", block + b"\x00" * (64 - len(block)))
        flags = CHUNK_START if i == 0 else 0
        if i + 64 >= len(data):
            flags |= CHUNK_END | ROOT
        cv = _compress(cv, words, 0, len(block), flags)[:8]
    return b"".join(struct.pack("<I", w) for w in cv)


# ── Ports of the rendered canonicalization law ──────────────────────────────

def domain_separated(domain, parts):
    """Byte-exact port of crypto_trust_canonical::domain_separated."""
    buf = KAT_DOMAIN_TAG.encode() + b"\x00" + domain.encode() + b"\x00"
    for part in parts:
        buf += struct.pack(">Q", len(part)) + part
    return buf


def kat_digest(parts):
    """digest(KAT_DOMAIN_TAG, parts) = blake3(domain_separated(TAG, parts))."""
    return blake3_32(domain_separated(KAT_DOMAIN_TAG, parts))


_ESCAPES = {'"': '\\"', "\\": "\\\\", "\b": "\\b", "\t": "\\t",
            "\n": "\\n", "\f": "\\f", "\r": "\\r"}


def _jcs_string(s):
    out = ['"']
    for ch in s:
        if ch in _ESCAPES:
            out.append(_ESCAPES[ch])
        elif ord(ch) < 0x20:
            out.append("\\u%04x" % ord(ch))
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def jcs_subset(value):
    """JCS for the fixture's document class: arrays and string-only objects.

    Any number/bool/null is refused loudly (RuntimeError): this verifier
    never guesses outside its documented, tested subset.
    """
    if isinstance(value, str):
        return _jcs_string(value)
    if isinstance(value, list):
        return "[" + ",".join(jcs_subset(v) for v in value) + "]"
    if isinstance(value, dict):
        # RFC 8785 section 3.2.3: order by UTF-16 code units. UTF-16-BE byte
        # comparison IS code-unit comparison.
        keys = sorted(value.keys(), key=lambda k: k.encode("utf-16-be"))
        return "{" + ",".join(_jcs_string(k) + ":" + jcs_subset(value[k])
                              for k in keys) + "}"
    raise RuntimeError(
        "jcs subset: unsupported value %r (document class is string-only)" % (value,))


def hex_bytes(s):
    return bytes.fromhex(s)


# ── openssl bridges (real subprocesses) ─────────────────────────────────────

def pem_wrap(tag, der):
    b64 = base64.b64encode(der).decode()
    lines = [b64[i:i + 64] for i in range(0, len(b64), 64)]
    return "-----BEGIN %s-----\n%s\n-----END %s-----\n" % (tag, "\n".join(lines), tag)


def ec_private_key_pem(scalar32):
    """RFC 5915 ECPrivateKey (P-256) from a 32-byte scalar."""
    inner = (b"\x02\x01\x01"                 # version INTEGER 1
             + b"\x04\x20" + scalar32        # privateKey OCTET STRING
             + b"\xa0\x0a" + PRIME256V1_OID)  # [0] parameters: prime256v1
    der = b"\x30" + bytes([len(inner)]) + inner
    return pem_wrap("EC PRIVATE KEY", der)


class Openssl:
    def __init__(self):
        self.bin = None
        self.version = None
        try:
            out = subprocess.run(["openssl", "version"],
                                 capture_output=True, text=True, check=True)
            self.bin = "openssl"
            self.version = out.stdout.strip()
        except (OSError, subprocess.CalledProcessError):
            pass
        self.tmp = None

    @property
    def available(self):
        return self.bin is not None

    def start(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="ctp-kat-")
        return self.tmp

    def stop(self):
        if self.tmp is not None:
            self.tmp.cleanup()
            self.tmp = None

    def _run(self, args):
        return subprocess.run([self.bin] + args, capture_output=True, text=True)

    def derive_public_sec1(self, scalar32):
        """RFC 5915 SEC1 PEM -> `openssl ec -pubout` -> SEC1 point bytes."""
        key_path = os.path.join(self.tmp.name, "key.pem")
        with open(key_path, "w") as f:
            f.write(ec_private_key_pem(scalar32))
        out = self._run(["ec", "-in", key_path, "-pubout", "-out", key_path + ".pub"])
        if out.returncode != 0:
            raise RuntimeError("openssl ec -pubout failed: %s" % out.stderr.strip())
        with open(key_path + ".pub") as f:
            b64 = "".join(l for l in f.read().splitlines() if "-----" not in l)
        spki = base64.b64decode(b64)
        if not spki.startswith(P256_SPKI_HEADER) or len(spki) != 26 + 65:
            raise RuntimeError("unexpected SPKI DER around derived public key")
        return spki[26:]

    def verify_es256(self, public_sec1, message, sig_der):
        """`openssl dgst -sha256 -verify` over the exact message bytes."""
        pub_path = os.path.join(self.tmp.name, "pub.pem")
        sig_path = os.path.join(self.tmp.name, "sig.der")
        msg_path = os.path.join(self.tmp.name, "msg.bin")
        with open(pub_path, "w") as f:
            f.write(pem_wrap("PUBLIC KEY",
                             P256_SPKI_HEADER + public_sec1))
        with open(sig_path, "wb") as f:
            f.write(sig_der)
        with open(msg_path, "wb") as f:
            f.write(message)
        out = self._run(["dgst", "-sha256", "-verify", pub_path,
                         "-signature", sig_path, msg_path])
        return out.returncode == 0 and "Verified OK" in out.stdout


# ── Checks ──────────────────────────────────────────────────────────────────

class Report:
    def __init__(self):
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.failures = []

    def ok(self, label):
        self.passed += 1
        print("PASS  %s" % label)

    def skip(self, label, reason):
        self.skipped += 1
        print("SKIP  %s (%s)" % (label, reason))

    def fail(self, label, detail):
        self.failed += 1
        self.failures.append((label, detail))
        print("FAIL  %s: %s" % (label, detail))

    def check(self, condition, label, detail):
        if condition:
            self.ok(label)
        else:
            self.fail(label, detail)

    def exit_code(self):
        if self.failed != 0:
            return 1
        if self.passed == 0:
            print("no check ran; refusing to exit green")
            return 1
        return 0


def expected_seed(index, algorithm):
    """Derives the seed bytes a vector must carry, per the rendered chain."""
    idx = struct.pack(">Q", index)
    if algorithm == "SLH-DSA-SHA2-128s":
        return b"".join(
            kat_digest([b"ctp.kat.seed.slh", idx, struct.pack(">Q", counter)])[:16]
            for counter in range(3))
    pieces = []
    for role, length in SEED_ROLE[algorithm]:
        d = kat_digest([b"ctp.kat.seed", role.encode(), idx])
        pieces.append(d[:length])
    return b"".join(pieces)


def hybrid_es256_der_len(signature):
    """Port of kat::hybrid_es256_der_len (short-form lengths only)."""
    if len(signature) < 2 or signature[0] != 0x30 or signature[1] >= 0x80:
        return None
    total = 2 + signature[1]
    if len(signature) < total:
        return None
    return total


def verify_vector(report, openssl, vector, index):
    vid = vector.get("vector_id", "<missing>")
    algorithm = vector.get("algorithm", "<missing>")
    label = "%s [%s]" % (vid, algorithm)

    # Vector identity: registry order and id format.
    report.check(
        vid == "kat-%03d-%s" % (index, algorithm),
        label + " vector_id format",
        "expected kat-%03d-%s" % (index, algorithm))

    # 1. Full derivation chain in Python (BLAKE3 + domain separation).
    if algorithm not in SEED_ROLE and algorithm != "SLH-DSA-SHA2-128s":
        report.skip(label + " derivation", "unknown algorithm %r" % algorithm)
        return
    try:
        want_seed = expected_seed(index, algorithm)
        want_msg = kat_digest([b"ctp.kat.message", struct.pack(">Q", index)])
    except Exception as exc:  # subset refusal -- loud, never fake
        report.skip(label + " derivation", str(exc))
        return
    try:
        seed = hex_bytes(vector["seed_hex"])
        message = hex_bytes(vector["message_hex"])
    except (KeyError, ValueError) as exc:
        report.fail(label + " derivation", "undecodable hex: %s" % exc)
        return
    report.check(seed == want_seed, label + " seed derivation (python blake3)",
                 "seed_hex does not match the domain-separated seed chain")
    report.check(message == want_msg, label + " message derivation (python blake3)",
                 "message_hex does not match blake3(domain_separated(TAG, "
                 "[ctp.kat.message, index_be]))")

    # 2. Signature verification where an independent runtime exists.
    if algorithm == "ES256":
        if not openssl.available:
            report.skip(label + " es256 sig", "openssl unavailable")
            return
        try:
            derived = openssl.derive_public_sec1(seed)
        except RuntimeError as exc:
            report.fail(label + " es256 pk derive", str(exc))
            return
        report.check(derived == hex_bytes(vector["public_key_hex"]),
                     label + " es256 pk derive (openssl)",
                     "openssl-derived public key differs from public_key_hex")
        verdict = openssl.verify_es256(derived, message, hex_bytes(vector["signature_hex"]))
        report.check(verdict, label + " es256 sig (openssl dgst -sha256 -verify)",
                     "DER signature did not verify over the derived message bytes")
    elif algorithm == "ES256+ML-DSA-65":
        signature = hex_bytes(vector["signature_hex"])
        public = hex_bytes(vector["public_key_hex"])
        es_len = hybrid_es256_der_len(signature)
        report.check(es_len is not None and len(public) == 65 + 1952,
                     label + " hybrid split",
                     "self-delimiting DER prefix or public-key split malformed")
        if es_len is None or len(public) != 65 + 1952:
            return
        if not openssl.available:
            report.skip(label + " hybrid es256 half", "openssl unavailable")
        else:
            try:
                derived = openssl.derive_public_sec1(seed[:32])
            except RuntimeError as exc:
                report.fail(label + " hybrid es256 pk derive", str(exc))
                return
            report.check(derived == public[:65],
                         label + " hybrid es256 pk derive (openssl)",
                         "openssl-derived ES256 half differs from public_key_hex prefix")
            verdict = openssl.verify_es256(derived, message, signature[:es_len])
            report.check(verdict,
                         label + " hybrid es256 half sig (openssl)",
                         "ES256 half did not verify over the derived message bytes")
        report.skip(label + " hybrid mldsa half sig", "rust-only (no PQ verifier here)")
    else:
        report.skip(label + " signature", "rust-only (%s has no independent "
                    "verifier on this host)" % algorithm)


def main():
    report = Report()

    # Verifier self-check first: a wrong digest engine must never PASS.
    for data, want in BLAKE3_SELF_CHECK:
        got = blake3_32(data).hex()
        report.check(got == want,
                     "blake3 self-check len=%d" % len(data),
                     "got %s want %s" % (got, want))
    if report.failed:
        print("the vendored BLAKE3 is broken; aborting before any KAT check")
        return 1

    # Fixture presence, parse, header, JCS fixed point.
    if not os.path.exists(FIXTURE_PATH):
        report.fail("fixture exists", FIXTURE_PATH)
        return report.exit_code()
    report.ok("fixture exists (%s)" % os.path.relpath(FIXTURE_PATH, REPO_ROOT))
    with open(FIXTURE_PATH, "r") as f:
        raw = f.read()
    try:
        document = json.loads(raw)
    except ValueError as exc:
        report.fail("fixture parses", str(exc))
        return report.exit_code()
    if not isinstance(document, dict):
        report.fail("fixture shape", "expected a JSON object header")
        return report.exit_code()
    report.check(list(document.keys()) == EXPECTED_HEADER_KEYS,
                 "fixture header keys exact+ordered",
                 "keys are %r, expected %r" % (list(document.keys()), EXPECTED_HEADER_KEYS))
    report.check(document.get("schema_version") == KAT_SCHEMA_VERSION,
                 "fixture schema_version",
                 "got %r want %r" % (document.get("schema_version"), KAT_SCHEMA_VERSION))
    note = document.get("generated_from", "")
    report.check(isinstance(note, str) and "generate_corpus" in note and "cargo test" in note,
                 "fixture generated_from note names source + regen command",
                 "note %r" % note[:80])
    corpus = document.get("corpus")
    if not isinstance(corpus, list) or not corpus:
        report.fail("fixture corpus", "corpus must be a non-empty array")
        return report.exit_code()
    try:
        canonical = jcs_subset(document)
        report.check(canonical == raw,
                     "fixture is JCS canonical fixed point (python port)",
                     "python-canonicalized bytes differ from the committed bytes")
    except RuntimeError as exc:
        report.skip("fixture JCS fixed point", str(exc))

    # openssl bridge (optional at runtime; present on macOS hosts).
    openssl = Openssl()
    if openssl.available:
        print("openssl: %s" % openssl.version)
        openssl.start()
    else:
        print("openssl: UNAVAILABLE -- signature checks will print SKIPPED loudly")

    # Per-vector verification.
    for index, vector in enumerate(corpus):
        if not isinstance(vector, dict):
            report.fail("corpus[%d] shape" % index, "vector must be an object")
            continue
        verify_vector(report, openssl, vector, index)

    # Negative control: the verify pipeline must REFUSE a tampered message.
    if openssl.available:
        es_vectors = [v for v in corpus if isinstance(v, dict) and v.get("algorithm") == "ES256"]
        if es_vectors:
            v = es_vectors[0]
            message = hex_bytes(v["message_hex"])
            tampered = bytes([message[0] ^ 0x01]) + message[1:]
            derived = None
            try:
                derived = openssl.derive_public_sec1(hex_bytes(v["seed_hex"]))
            except RuntimeError:
                pass
            if derived is not None:
                refused = not openssl.verify_es256(
                    derived, tampered, hex_bytes(v["signature_hex"]))
                report.check(refused,
                             "negative control: tampered message refused",
                             "openssl verified a tampered message; pipeline is vacuous")
        else:
            report.skip("negative control", "no ES256 vector in corpus")

    if openssl.available:
        openssl.stop()

    print("")
    print("checks: %d passed, %d failed, %d skipped" %
          (report.passed, report.failed, report.skipped))
    if report.failed:
        for label, detail in report.failures:
            print("  FAILED %s: %s" % (label, detail))
    code = report.exit_code()
    print("verdict: %s" % ("ALL NON-SKIPPED CHECKS PASS" if code == 0 else "REFUSED"))
    return code


if __name__ == "__main__":
    sys.exit(main())
