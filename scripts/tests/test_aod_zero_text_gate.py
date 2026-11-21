"""Unit court for scripts/aod_zero_text_gate.py (AFFI-SPEC-26.11.19 PR-2)."""
from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "aod_zero_text_gate.py"
SPEC = importlib.util.spec_from_file_location("aod_zero_text_gate", MODULE_PATH)
assert SPEC and SPEC.loader
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


class AodZeroTextGateTests(unittest.TestCase):
    def test_deny_list_covers_spec_tokens(self) -> None:
        required = {
            "CTRT",
            "ontology",
            "parquet",
            "Arrow",
            "schema",
            "affidavit",
            "lease_nullifier",
            "row_count",
        }
        self.assertTrue(required.issubset(set(gate.DOMAIN_TOKENS)))

    def test_clean_non_ascii_magic_passes(self) -> None:
        blob = bytes([0x9F, 0x1A, 0xC4, 0x01]) + bytes([0x00, 0x01, 0xFE, 0xFF]) * 64
        self.assertEqual(gate.scan_blob(blob, "clean"), [])

    def test_ctrt_token_refused(self) -> None:
        blob = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"pad CTRT pad"
        hits = gate.scan_blob(blob, "x")
        self.assertTrue(any("CTRT" in h for h in hits))

    def test_json_row_count_refused(self) -> None:
        blob = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b'{"row_count":1}'
        hits = gate.scan_blob(blob, "x")
        self.assertTrue(any("row_count" in h for h in hits))

    def test_ascii_header_refused(self) -> None:
        blob = b"AOD1" + b"\x00" * 32
        hits = gate.scan_blob(blob, "x")
        self.assertTrue(any("ASCII header" in h for h in hits))

    def test_ontology_iri_refused(self) -> None:
        blob = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"http://affidavit.example/onto#T"
        hits = gate.scan_blob(blob, "x")
        self.assertTrue(any("IRI" in h or "affidavit" in h for h in hits))

    def test_scan_file_round_trip(self) -> None:
        clean = bytes([0x9F, 0x1A, 0xC4, 0x01]) + bytes(range(128, 256)) * 2
        dirty = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"xxxxlease_nullifieryyyy"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            good = root / "good.aod"
            bad = root / "bad.aod"
            good.write_bytes(clean)
            bad.write_bytes(dirty)
            self.assertEqual(gate.scan_file(good), [])
            self.assertTrue(gate.scan_file(bad))

    def test_cli_self_test_exit_zero(self) -> None:
        self.assertEqual(gate.main(["--self-test"]), 0)

    def test_cli_refuses_dirty_file(self) -> None:
        dirty = bytes([0x9F, 0x1A, 0xC4, 0x01]) + b"schema"
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "dirty.aod"
            path.write_bytes(dirty)
            self.assertEqual(gate.main([str(path)]), 2)


if __name__ == "__main__":
    unittest.main()
