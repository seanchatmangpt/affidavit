from __future__ import annotations
import importlib.util, json, sys, unittest
from pathlib import Path
P=Path(__file__).parents[1]/"verify_v26_9_1_ecosystem_composition.py"
spec=importlib.util.spec_from_file_location("crown",P); m=importlib.util.module_from_spec(spec); sys.modules[spec.name]=m; spec.loader.exec_module(m)
MANIFEST=Path(__file__).parents[2]/"profiles/v26.9.1-ecosystem-composition.json"
class CompositionTests(unittest.TestCase):
    def setUp(self): self.x=json.loads(MANIFEST.read_text())
    def test_exact_closure_is_admitted_without_crown_transfer(self):
        r=m.verify(self.x); self.assertEqual(r["composition_standing"],"ALIVE"); self.assertEqual(r["release_crown_standing"],"PARTIAL_ALIVE"); self.assertFalse(r["standing_transfer"])
    def test_planner_authority_collapse_refused(self):
        self.x["separations"]["planner_policy_role_agent_authority"]="COLLAPSED"
        with self.assertRaisesRegex(m.Refusal,"ROLE_COLLAPSE"): m.verify(self.x)
    def test_bcre_brce_collapse_refused(self):
        self.x["separations"]["bcre_brce"]="EQUIVALENT"
        with self.assertRaisesRegex(m.Refusal,"BCRE_BRCE_COLLAPSE"): m.verify(self.x)
    def test_live_azure_cannot_satisfy_crown(self):
        self.x["separations"]["live_azure"]="ALIVE"
        with self.assertRaisesRegex(m.Refusal,"LIVE_AZURE_AUTHORITY"): m.verify(self.x)
    def test_pin_is_not_standing(self):
        self.x["standing_policy"]["pin_is_not_alive"]=False
        with self.assertRaisesRegex(m.Refusal,"STANDING_TRANSFER"): m.verify(self.x)
    def test_receipt_replay_tamper_refused(self):
        r=m.seal(m.verify(self.x)); m.replay(r); r["edge_count"]=999
        with self.assertRaisesRegex(m.Refusal,"RECEIPT_TAMPER"): m.replay(r)
if __name__=="__main__": unittest.main()
