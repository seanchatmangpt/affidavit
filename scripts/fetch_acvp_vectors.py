#!/usr/bin/env python3
"""Fetch and extract real NIST ACVP cross-implementation vectors for the
affidavit cryptographic trust plane (W4-L4, v26.9.28).

Upstream corpus: usnistgov/ACVP-Server (NIST's ACVP server sample corpora,
gen-val/json-files). Downloads are pinned to an exact upstream commit so the
extraction is replayable byte-for-byte; each fixture carries a provenance
block naming the source URL, commit, file sha256s, and extraction date.

Extracted subsets (deterministic, capped, first-N by tcId within each rule):

- ML-DSA-65 keyGen (ML-DSA-keyGen-FIPS204): up to 3 (seed -> pk) pairs.
  This is a real cross-implementation KAT: FIPS 204 KeyGen_internal(seed)
  must reproduce NIST's public key exactly.
- ML-DSA-65 sign -> ML-DSA-sigVer-FIPS204 (tg "pure", external interface):
  up to 3 (pk, message, context, signature, testPassed) entries. HONESTY
  NOTE: the ACVP sigGen corpus carries only the 4032-byte EXPANDED private
  key, from which the 32-byte seed (the plane's only key input, FIPS 204
  xi) cannot be derived, so exact-match sigGen vectors are NOT usable by a
  seed-only signer. The usable sign-direction proof is the verification
  direction: NIST's corpus signatures must verify under the plane's
  verifier (implementation-independent). Cases with a non-empty context
  use the FIPS 204 PureMTS framing M' = 0x00 || len(ctx) || ctx || M,
  applied by the caller (the plane exposes only the raw message input).
  Negative corpus vectors (testPassed=false) are kept as real adversarial
  cases from NIST, not self-made corruptions.
- SLH-DSA-SHA2-128s keyGen (SLH-DSA-keyGen-FIPS205): up to 2
  (skSeed || skPrf || pkSeed -> pk) triples; the 48-byte concatenation is
  exactly the plane's seed-triplet input.
- SLH-DSA-SHA2-128s sign (SLH-DSA-sigGen-FIPS205, tg with empty context,
  deterministic, external, pure): up to 1 (seeds48, message, signature)
  exact-match vector; the first 48 bytes of the ACVP 64-byte sk are
  SK.seed || SK.prf || PK.seed, i.e. the plane's key input. Context-bearing
  cases are skipped because the plane exposes no context parameter.

Usage:
  python3 scripts/fetch_acvp_vectors.py            # fetch + extract fixtures
  python3 scripts/fetch_acvp_vectors.py --check    # verify fixtures reproduce
  python3 scripts/fetch_acvp_vectors.py --commit <sha>   # override pin

No network: the script exits 2 with a typed refusal; committed fixtures
stand alone (the Rust test suite never runs this script).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

# Pinned upstream commit (replay identity; bump deliberately, never float).
PINNED_COMMIT = "975de31eb83d87039ec88934fdc47d8c312b892d"
REPO = "usnistgov/ACVP-Server"
RAW_BASE = f"https://raw.githubusercontent.com/{REPO}"
CACHE_DEFAULT = Path("/tmp/affidavit-acvp-cache")

CORPORA = {
    "ML-DSA-keyGen": {
        "prompt": "gen-val/json-files/ML-DSA-keyGen-FIPS204/prompt.json",
        "expected": "gen-val/json-files/ML-DSA-keyGen-FIPS204/expectedResults.json",
    },
    "ML-DSA-sigVer": {
        "prompt": "gen-val/json-files/ML-DSA-sigVer-FIPS204/prompt.json",
        "expected": "gen-val/json-files/ML-DSA-sigVer-FIPS204/expectedResults.json",
    },
    "SLH-DSA-keyGen": {
        "prompt": "gen-val/json-files/SLH-DSA-keyGen-FIPS205/prompt.json",
        "expected": "gen-val/json-files/SLH-DSA-keyGen-FIPS205/expectedResults.json",
    },
    "SLH-DSA-sigGen": {
        "prompt": "gen-val/json-files/SLH-DSA-sigGen-FIPS205/prompt.json",
        "expected": "gen-val/json-files/SLH-DSA-sigGen-FIPS205/expectedResults.json",
    },
}

MAX_ML_KEYGEN = 3
MAX_ML_SIGVER = 3
MAX_SLH_KEYGEN = 2
MAX_SLH_SIGN = 1

ML_DSA_65 = "ML-DSA-65"
SLH_DSA_128S = "SLH-DSA-SHA2-128s"
ML_DSA_65_PK_HEX = 2 * 1952
ML_DSA_65_SIG_HEX = 2 * 3309
SLH_128S_PK_HEX = 2 * 32


def die(msg: str, code: int = 2) -> None:
    print(f"REFUSED_FETCH_ACVP: {msg}", file=sys.stderr)
    sys.exit(code)


def fetch(rel: str, commit: str, cache: Path) -> tuple[bytes, str]:
    """Download (or reuse cached) upstream file; return (body, sha256-hex)."""
    cache_dir = cache / commit
    cache_dir.mkdir(parents=True, exist_ok=True)
    dest = cache_dir / rel.replace("/", "_")
    if dest.is_file():
        body = dest.read_bytes()
    else:
        url = f"{RAW_BASE}/{commit}/{rel}"
        try:
            with urllib.request.urlopen(url, timeout=60) as resp:
                body = resp.read()
        except Exception as exc:  # noqa: BLE001 - typed refusal below
            die(f"network fetch failed for {url}: {exc}")
        dest.write_bytes(body)
    return body, hashlib.sha256(body).hexdigest()


def load_corpus(name: str, commit: str, cache: Path) -> tuple[dict, dict, dict]:
    files = CORPORA[name]
    prompt_body, prompt_sha = fetch(files["prompt"], commit, cache)
    expected_body, expected_sha = fetch(files["expected"], commit, cache)
    prov = {
        "prompt_path": files["prompt"],
        "prompt_sha256": prompt_sha,
        "expected_path": files["expected"],
        "expected_sha256": expected_sha,
    }
    return (
        json.loads(prompt_body.decode("utf-8")),
        json.loads(expected_body.decode("utf-8")),
        prov,
    )


def join_groups(prompt: dict, expected: dict) -> dict[int, dict]:
    """Join prompt test cases with expected results by (tgId, tcId)."""
    out: dict[int, dict] = {}
    exp_by_tg = {g["tgId"]: {t["tcId"]: t for t in g["tests"]} for g in expected["testGroups"]}
    for g in prompt["testGroups"]:
        exp_tests = exp_by_tg.get(g["tgId"], {})
        for tc in g["tests"]:
            exp = exp_tests.get(tc["tcId"])
            if exp is not None:
                merged = dict(tc)
                merged.update(exp)
                out[(g["tgId"], tc["tcId"])] = {**g_for(g), "tc": merged}
    return out


def g_for(g: dict) -> dict:
    return {k: v for k, v in g.items() if k != "tests"}


def extract_ml_keygen(prompt: dict, expected: dict) -> list[dict]:
    vectors = []
    for g in prompt["testGroups"]:
        if g["parameterSet"] != ML_DSA_65:
            continue
        exp = next(
            {t["tcId"]: t for t in eg["tests"]}
            for eg in expected["testGroups"]
            if eg["tgId"] == g["tgId"]
        )
        for tc in sorted(g["tests"], key=lambda t: t["tcId"]):
            seed = tc["seed"].lower()
            pk = exp[tc["tcId"]]["pk"].lower()
            if len(seed) != 64 or len(pk) != ML_DSA_65_PK_HEX:
                die(f"unexpected keyGen field lengths at tcId {tc['tcId']}")
            vectors.append({"tcId": tc["tcId"], "seed": seed, "pk": pk})
            if len(vectors) == MAX_ML_KEYGEN:
                return vectors
    die("no ML-DSA-65 keyGen group found upstream")


def extract_ml_sigver(prompt: dict, expected: dict) -> list[dict]:
    """Select deterministic sigVer entries for ML-DSA-65 pure/external.

    Selection rule (first-N by tcId inside each rule, capped):
      1. passing entries (testPassed=true) — context-framing coverage,
      2. failing entries with empty context — raw adversarial negatives,
      3. failing entries with a context — framing negatives.
    """
    selected: list[tuple[int, dict]] = []
    for g in prompt["testGroups"]:
        if g["parameterSet"] != ML_DSA_65 or g.get("preHash") != "pure":
            continue
        exp = next(
            {t["tcId"]: t for t in eg["tests"]}
            for eg in expected["testGroups"]
            if eg["tgId"] == g["tgId"]
        )
        for tc in sorted(g["tests"], key=lambda t: t["tcId"]):
            e = exp[tc["tcId"]]
            entry = {
                "tcId": tc["tcId"],
                "pk": tc["pk"].lower(),
                "message": tc["message"].lower(),
                "context": tc["context"].lower(),
                "signature": tc["signature"].lower(),
                "testPassed": bool(e["testPassed"]),
            }
            if len(entry["pk"]) != ML_DSA_65_PK_HEX or len(entry["signature"]) != ML_DSA_65_SIG_HEX:
                die(f"unexpected sigVer field lengths at tcId {tc['tcId']}")
            selected.append((tc["tcId"], entry))
    if not selected:
        die("no ML-DSA-65 pure sigVer group found upstream")
    positives = [v for v in selected if v[1]["testPassed"]]
    negatives_empty = [v for v in selected if not v[1]["testPassed"] and v[1]["context"] == ""]
    negatives_ctx = [v for v in selected if not v[1]["testPassed"] and v[1]["context"] != ""]
    chosen = positives[:2] + negatives_empty[:1] + negatives_ctx[:1]
    chosen = chosen[:MAX_ML_SIGVER]
    return [v for _, v in sorted(chosen, key=lambda x: x[0])]


def extract_slh_keygen(prompt: dict, expected: dict) -> list[dict]:
    vectors = []
    for g in prompt["testGroups"]:
        if g["parameterSet"] != SLH_DSA_128S:
            continue
        exp = next(
            {t["tcId"]: t for t in eg["tests"]}
            for eg in expected["testGroups"]
            if eg["tgId"] == g["tgId"]
        )
        for tc in sorted(g["tests"], key=lambda t: t["tcId"]):
            seeds = tc["skSeed"].lower() + tc["skPrf"].lower() + tc["pkSeed"].lower()
            pk = exp[tc["tcId"]]["pk"].lower()
            if len(seeds) != 96 or len(pk) != SLH_128S_PK_HEX:
                die(f"unexpected SLH keyGen field lengths at tcId {tc['tcId']}")
            vectors.append({"tcId": tc["tcId"], "seeds": seeds, "pk": pk})
            if len(vectors) == MAX_SLH_KEYGEN:
                return vectors
    die("no SLH-DSA-SHA2-128s keyGen group found upstream")


def extract_slh_sign(prompt: dict, expected: dict) -> list[dict]:
    """First empty-context deterministic pure external SHA2-128s sign case.

    sk is 4n = 64 bytes: SK.seed || SK.prf || PK.seed || PK.root; the plane
    consumes the first 48 bytes (skSeed||skPrf||pkSeed) and derives the rest.
    """
    for g in prompt["testGroups"]:
        if g["parameterSet"] != SLH_DSA_128S or g.get("preHash") != "pure":
            continue
        exp = next(
            {t["tcId"]: t for t in eg["tests"]}
            for eg in expected["testGroups"]
            if eg["tgId"] == g["tgId"]
        )
        for tc in sorted(g["tests"], key=lambda t: t["tcId"]):
            if tc["context"].lower() != "":
                continue
            sk = tc["sk"].lower()
            if len(sk) != 2 * 64:
                die(f"unexpected SLH sk length at tcId {tc['tcId']}")
            return [
                {
                    "tcId": tc["tcId"],
                    "seeds": sk[:96],
                    "message": tc["message"].lower(),
                    "signature": exp[tc["tcId"]]["signature"].lower(),
                    "deterministic": bool(g["deterministic"]),
                }
            ]
    return []


def provenance_block(commit: str, corpus_prov: dict, notes: str) -> dict:
    return {
        "source_repo": f"https://github.com/{REPO}",
        "source_commit": commit,
        "files": corpus_prov,
        "extracted_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "extractor": "scripts/fetch_acvp_vectors.py",
        "notes": notes,
    }


def write_fixture(path: Path, provenance: dict, vectors_key: str, vectors: list) -> None:
    doc = {"provenance": provenance, vectors_key: vectors}
    path.write_text(json.dumps(doc, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"wrote {path} ({len(vectors)} vectors)")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--commit", default=PINNED_COMMIT, help="upstream commit to pin")
    ap.add_argument("--cache-dir", type=Path, default=CACHE_DEFAULT)
    ap.add_argument("--check", action="store_true", help="verify fixtures reproduce byte-for-byte")
    ap.add_argument("--fixtures-dir", type=Path, default=None)
    args = ap.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    fixtures_dir = args.fixtures_dir or (repo_root / "fixtures" / "crypto_trust_acvp")

    corpora = {name: load_corpus(name, args.commit, args.cache_dir) for name in CORPORA}

    ml_kg_prov = corpora["ML-DSA-keyGen"][2]
    ml_sv_prov = corpora["ML-DSA-sigVer"][2]
    slh_kg_prov = corpora["SLH-DSA-keyGen"][2]
    slh_sg_prov = corpora["SLH-DSA-sigGen"][2]

    built = {
        "ml_dsa_65_keygen.json": (
            "ml_dsa_65_keygen",
            provenance_block(
                args.commit,
                ml_kg_prov,
                "ACVP ML-DSA keyGen AFT, parameterSet ML-DSA-65: first "
                f"{MAX_ML_KEYGEN} test cases by tcId (seed -> pk exact-match KAT). "
                "Hex lowercased from the upstream uppercase corpus.",
            ),
            extract_ml_keygen(corpora["ML-DSA-keyGen"][0], corpora["ML-DSA-keyGen"][1]),
        ),
        "ml_dsa_65_sign.json": (
            "ml_dsa_65_sigver",
            provenance_block(
                args.commit,
                ml_sv_prov,
                "HONEST LIMITATION: the ACVP ML-DSA sigGen corpus (ML-DSA-sigGen-FIPS204) "
                "carries only the 4032-byte expanded private key; the 32-byte seed the "
                "affidavit plane consumes (FIPS 204 xi) is not derivable from it, so "
                "exact-match sigGen vectors are not usable by a seed-only signer. This "
                "fixture therefore uses the sigVer corpus (ML-DSA-sigVer-FIPS204, "
                "parameterSet ML-DSA-65, preHash pure): NIST corpus signatures must "
                "verify (or refuse, for testPassed=false) under the plane's verifier. "
                "Non-empty context cases require the FIPS 204 PureMTS message framing "
                "M' = 0x00 || len(ctx) || ctx || M applied by the caller. Selection: "
                "first 2 passing + first empty-context negative + first context negative "
                f"by tcId, capped at {MAX_ML_SIGVER}.",
            ),
            extract_ml_sigver(corpora["ML-DSA-sigVer"][0], corpora["ML-DSA-sigVer"][1]),
        ),
        "slh_dsa_128s_keygen.json": (
            "slh_dsa_128s",
            provenance_block(
                args.commit,
                {**slh_kg_prov, "sigGen_prompt_path": slh_sg_prov["prompt_path"],
                 "sigGen_prompt_sha256": slh_sg_prov["prompt_sha256"],
                 "sigGen_expected_path": slh_sg_prov["expected_path"],
                 "sigGen_expected_sha256": slh_sg_prov["expected_sha256"]},
                "SLH-DSA-SHA2-128s: first "
                f"{MAX_SLH_KEYGEN} keyGen test cases by tcId (skSeed||skPrf||pkSeed -> pk, "
                "the 48-byte plane seed triplet), plus up to "
                f"{MAX_SLH_SIGN} sigGen sign entry (SLH-DSA-sigGen-FIPS205, deterministic, "
                "external, pure, empty context; seeds = first 48 bytes of the 64-byte "
                "ACVP sk = SK.seed||SK.prf||PK.seed). Context-bearing cases are skipped: "
                "the plane exposes no context parameter.",
            ),
            None,
        ),
    }

    slh_kg = extract_slh_keygen(corpora["SLH-DSA-keyGen"][0], corpora["SLH-DSA-keyGen"][1])
    slh_sign = extract_slh_sign(corpora["SLH-DSA-sigGen"][0], corpora["SLH-DSA-sigGen"][1])
    built["slh_dsa_128s_keygen.json"] = (
        built["slh_dsa_128s_keygen.json"][0],
        built["slh_dsa_128s_keygen.json"][1],
        {"keygen": slh_kg, "sign": slh_sign},
    )

    outputs = {}
    for fname, (key, prov, vectors) in built.items():
        if fname == "slh_dsa_128s_keygen.json":
            doc = {"provenance": prov, "keygen": vectors["keygen"], "sign": vectors["sign"]}
            text = json.dumps(doc, indent=2, sort_keys=False) + "\n"
            if not args.check:
                print(
                    f"wrote {fixtures_dir / fname} "
                    f"({len(vectors['keygen'])} keygen + {len(vectors['sign'])} sign vectors)"
                )
        else:
            doc = {"provenance": prov, key: vectors}
            text = json.dumps(doc, indent=2, sort_keys=False) + "\n"
            if not args.check:
                print(f"wrote {fixtures_dir / fname} ({len(vectors)} vectors)")
        outputs[fname] = text

    if args.check:
        # The vector content must reproduce deterministically; the extraction
        # timestamp is metadata about the original extraction and is excluded
        # from the comparison (commit + file digests in provenance ARE checked).
        for fname, text in outputs.items():
            current = (fixtures_dir / fname).read_text(encoding="utf-8")
            fresh = json.loads(text)
            persisted = json.loads(current)
            fresh["provenance"].pop("extracted_at_utc", None)
            persisted["provenance"].pop("extracted_at_utc", None)
            if fresh != persisted:
                die(f"--check: {fname} does not reproduce (vectors or provenance drifted)")
        print("check OK: all fixtures reproduce (vectors + provenance digests, modulo timestamp)")
        return

    fixtures_dir.mkdir(parents=True, exist_ok=True)
    for fname, text in outputs.items():
        (fixtures_dir / fname).write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
