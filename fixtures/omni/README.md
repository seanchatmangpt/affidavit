# omni-evidence/v1 fixture bundles

Real bundles produced by the two sibling producers, consumed by
`tests/omni_bundle_e2e.rs` (emit-batch -> assemble -> verify ACCEPT, then
tamper -> REJECT).

| Bundle | Producer | Regenerate |
|---|---|---|
| `cgap/` | `seanchatmangpt/cgap` `scripts/affidavit_bundle.py` (default gate set at cgap `ede726f`) | `python3 scripts/affidavit_bundle.py --out <dir>` |
| `ash_pplan/` | `seanchatmangpt/ash_pplan` `AshPPlan.AffidavitBundle` (the two receipts in `test/affidavit_bundle_test.exs`) | `AshPPlan.AffidavitBundle.entries(receipts) \|> AshPPlan.AffidavitBundle.write!(dir)` |

Format and schema: `docs/integrations/OMNIPLATFORM.md`,
`schemas/omni-evidence-batch.v1.schema.json`.
