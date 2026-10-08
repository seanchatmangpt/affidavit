#!/usr/bin/env python3
"""golden_receipt.py — mint and check the affidavit-wasm golden fixtures.

Two subcommands, both wired into the justfile:

  mint           Re-mint affidavit-wasm/tests/fixtures/golden_receipt.json and
                 tampered_receipt.json for the current workspace version.
  check-genesis  Verification-only: confirm the web lane's hand-typed genesis
                 seed literals match the current version. Never edits web/.

The version is derived from the workspace Cargo.toml. Both subcommands refuse
with REFUSED_VERSION_MISMATCH when CHANGELOG.md has no `## [<version>]`
section -- the same expectation as tests/release_identity.rs
(`the_changelog_documents_the_current_version`): a version bump without a
changelog entry is a release nobody can audit.

The mint sequence is exactly the one documented in docs/WASM.md and pinned
byte-for-byte by tests/wasm_abi.rs::assemble_reproduces_affi_byte_for_byte:
three `affi receipt emit` calls over payload files holding "compile step\\n",
"unit tests\\n", and "ship it\\n", then `affi receipt assemble`. The tampered
fixture is `sed 's/"test"/"tampered"/'` of the golden one.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIXTURE_DIR = os.path.join(REPO, "affidavit-wasm", "tests", "fixtures")
GOLDEN = os.path.join(FIXTURE_DIR, "golden_receipt.json")
TAMPERED = os.path.join(FIXTURE_DIR, "tampered_receipt.json")
WEB_FILES = (
    "web/lib/verify-client.ts",
    "web/app/visualizer/model.ts",
)

# The exact mint inputs pinned by tests/wasm_abi.rs
# (::assemble_reproduces_affi_byte_for_byte). Do not change these without
# re-pinning that test (and the tampered-reason hash it asserts).
MINT_EVENTS = [
    ("build", "repo:git:main", b"compile step\n"),
    ("test", "suite:test-suite:unit", b"unit tests\n"),
    ("deploy", "svc:service", b"ship it\n"),
]


def workspace_version() -> str:
    path = os.path.join(REPO, "Cargo.toml")
    with open(path) as f:
        m = re.search(r'^version = "([^"]+)"', f.read(), re.M)
    if not m:
        sys.exit("REFUSED_VERSION_MISMATCH: no workspace `version` in Cargo.toml")
    return m.group(1)


def changelog_has(version: str) -> bool:
    path = os.path.join(REPO, "CHANGELOG.md")
    with open(path) as f:
        return "## [" + version + "]" in f.read()


def admit_version() -> str:
    version = workspace_version()
    if not changelog_has(version):
        sys.exit(
            "REFUSED_VERSION_MISMATCH: CHANGELOG.md has no `## [{0}]` section. "
            "Bumping the version without documenting it leaves operators unable "
            "to tell why their receipts stopped verifying.".format(version)
        )
    return version


def build_affi() -> str:
    subprocess.run(
        ["cargo", "build", "--quiet", "--bin", "affi"],
        cwd=REPO,
        check=True,
    )
    return os.path.join(REPO, "target", "debug", "affi")


def mint() -> None:
    version = admit_version()
    affi = build_affi()
    work = tempfile.mkdtemp(prefix="golden-mint-")
    try:
        for event_type, objects, payload in MINT_EVENTS:
            payload_path = os.path.join(work, "payload.bin")
            with open(payload_path, "wb") as f:
                f.write(payload)
            subprocess.run(
                [
                    affi, "receipt", "emit",
                    "--type", event_type,
                    "--object", objects,
                    "--payload", payload_path,
                ],
                cwd=work,
                check=True,
            )
        receipt_path = os.path.join(work, "receipt.json")
        subprocess.run(
            [affi, "receipt", "assemble", "--out", receipt_path],
            cwd=work,
            check=True,
        )

        with open(receipt_path) as f:
            golden_text = f.read()
        with open(GOLDEN, "w") as f:
            f.write(golden_text)

        # Tamper exactly as documented: swap the "test" event_type for
        # "tampered". `"test-suite"` carries different quoting, so the sed
        # bites only the event_type.
        tampered_text = golden_text.replace('"test"', '"tampered"', 1)
        if tampered_text == golden_text:
            sys.exit("REFUSED_TAMPER_FAILED: no `\"test\"` event found in minted receipt")
        with open(TAMPERED, "w") as f:
            f.write(tampered_text)

        print("minted golden + tampered fixtures for {0}".format(version))
        print("  {0}".format(GOLDEN))
        print("  {0}".format(TAMPERED))
        print(
            "reminder: on a version bump, also update the expected hashes in "
            "tests/wasm_abi.rs::rejects_a_tampered_receipt_with_the_exact_reason_affi_gives"
        )
    finally:
        import shutil as _shutil

        _shutil.rmtree(work, ignore_errors=True)


def check_genesis() -> None:
    version = admit_version()
    expected = "affidavit-v{0}-genesis".format(version)
    failures = []
    for relative in WEB_FILES:
        path = os.path.join(REPO, relative)
        if not os.path.exists(path):
            continue  # web lane is optional; same skip as the release_identity court
        with open(path) as f:
            source = f.read()
        stale = [
            source[i : source.find('"', i) if source.find('"', i) != -1 else len(source)]
            for i in (m.start() for m in re.finditer("affidavit-v", source))
        ]
        stale = [s for s in stale if s != expected]
        if stale:
            failures.append(
                "{0} carries genesis seed(s) {1} but the binary uses `{2}`. "
                "A drifted seed makes the browser verifier reject every real receipt.".format(
                    relative, stale, expected
                )
            )
        if expected not in source:
            failures.append(
                "{0} does not mention the current genesis seed `{1}`".format(relative, expected)
            )
    if failures:
        for f in failures:
            print("GENESIS DRIFT: " + f, file=sys.stderr)
        sys.exit(1)
    print("check-genesis: both web files carry `{0}`".format(expected))


def main() -> None:
    commands = {"mint": mint, "check-genesis": check_genesis}
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd not in commands:
        sys.exit("usage: golden_receipt.py {{mint|check-genesis}}")
    commands[cmd]()


if __name__ == "__main__":
    main()
