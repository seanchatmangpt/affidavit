from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "ci_errc.py"
SPEC = importlib.util.spec_from_file_location("ci_errc", MODULE_PATH)
assert SPEC and SPEC.loader
ci_errc = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ci_errc)


class CiErrcTests(unittest.TestCase):
    def test_root_source_routes_to_root_rust(self) -> None:
        routed = ci_errc.classify_paths(["src/errc.rs"])
        self.assertEqual(routed["root_rust"], ["src/errc.rs"])
        self.assertEqual(routed["generation"], [])

    def test_generated_verb_routes_to_generation_not_root(self) -> None:
        routed = ci_errc.classify_paths(["src/verbs/receipt_emit.rs"])
        self.assertEqual(routed["root_rust"], [])
        self.assertEqual(routed["generation"], ["src/verbs/receipt_emit.rs"])

    def test_ggen_config_routes_to_generation(self) -> None:
        routed = ci_errc.classify_paths(["ggen.toml"])
        self.assertEqual(routed["generation"], ["ggen.toml"])

    def test_docs_and_scripts_route_to_governance(self) -> None:
        routed = ci_errc.classify_paths(["docs/ERRC.md", "scripts/ci_errc.py"])
        self.assertEqual(routed["governance"], ["docs/ERRC.md", "scripts/ci_errc.py"])

    def test_rust_workflow_routes_to_root_rust(self) -> None:
        routed = ci_errc.classify_paths([".github/workflows/rust.yml"])
        self.assertEqual(routed["root_rust"], [".github/workflows/rust.yml"])

    def test_unowned_path_is_explicit_fast_only(self) -> None:
        routed = ci_errc.classify_paths(["LICENSE"])
        self.assertEqual(routed["fast_only"], ["LICENSE"])

    def test_invalid_json_is_a_typed_failure(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "bad.json").write_text("{", encoding="utf-8")
            checks = ci_errc.validate_structured_files(root, ["bad.json"])
        self.assertEqual(checks[0]["failure"], "STRUCTURED_FILE_INVALID")
        self.assertFalse(checks[0]["passed"])
        self.assertEqual(ci_errc.classify_fast_standing(checks), "BUILD_BROKEN")

    def test_valid_toml_and_json_are_admitted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "ok.toml").write_text('answer = 42\n', encoding="utf-8")
            (root / "ok.json").write_text(json.dumps({"answer": 42}), encoding="utf-8")
            checks = ci_errc.validate_structured_files(root, ["ok.toml", "ok.json"])
        self.assertEqual(len(checks), 2)
        self.assertTrue(all(check["passed"] for check in checks))
        self.assertEqual(ci_errc.classify_fast_standing([]), "PARTIAL_ALIVE")

    def test_exact_head_mismatch_is_typed_and_blocked(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = ci_errc.exact_head_check(root, "deadbeef")
        self.assertFalse(result["passed"])
        self.assertEqual(result["failure"], "EXACT_HEAD_MISMATCH")
        self.assertEqual(ci_errc.classify_fast_standing([result]), "BLOCKED")

    def test_discovery_failure_is_blocked_not_build_broken(self) -> None:
        failure = {"failure": "CHANGED_FILE_DISCOVERY_FAILED", "passed": False}
        self.assertEqual(ci_errc.classify_fast_standing([failure]), "BLOCKED")

    def test_fast_court_claim_ceiling_cannot_crown_alive(self) -> None:
        self.assertEqual(
            ci_errc.CLAIM_CEILING,
            "EXACT_HEAD_ROUTING_AND_STRUCTURED_ADMISSION_ONLY",
        )
        self.assertNotIn("ALIVE", ci_errc.CLAIM_CEILING.split("_AND_"))


if __name__ == "__main__":
    unittest.main()
