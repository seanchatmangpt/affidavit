# DOC-HDIT-CERTIFY-RECEIPT — affidavit — v26.10.8

Lane R1+R7 (relaunch), 2026-10-09. Repo `/Users/sac/affidavit`, branch
`fix/clippy-crypto-trust`.

## Subject

- Base at lane start: `6b2dd4c21b93a66970b706cbb0d43dd93a2e658b` (clean docs/
  at lane start; no partial prior-lane docs edits on disk).
- Certified docs state: `c60da06` (`fix(examples): gate seal_sj_record behind
  crypto-trust; regen generated reference at pinned extractor` — consumed this
  lane's scaffold regen of
  `docs/reference/generated/{reference,how_to,explanation}.md` together with
  its own example-gating fix).
- Certified inputs: `/tmp/hdit-r1/aff4.inputs.json`, extracted at the working
  tree, surface-identical to `c60da06`: the only dirty file
  `tests/sj_record.rs` carries a pure rustfmt reflow with zero symbol changes
  (disclosed, not lane-owned, left untouched).

## Pipeline (commands + exits)

```sh
shasum -a 256 gen_doc_surface.py                            # pin check
python3 <pinned-extractor> code /Users/sac/affidavit > aff4.code.json   # exit 0
python3 <pinned-extractor> doc  /Users/sac/affidavit \
  --code-json aff4.code.json > aff4.doc.json                # exit 0
# canonical 5-key inputs merge (rollout.sh merge shape, mirrored inline)
python3 /tmp/hdit-r1/mirror.py aff4.inputs.json
# -> phi=2/20651=0.000097  S_coverage=0.980552  Q=0.999903
doc-hdit audit aff4.inputs.json courts/doc_quality.court    # exit 0, PASS x3
doc-hdit certify aff4.inputs.json courts/doc_quality.court \
  --docs docs --chain docs/doc-hdit.receipts.jsonl \
  --extractor pinned_extractor.py                           # exit 0
```

All extraction/audit/certify commands exit 0; audit gates PASS x3 at
thresholds 0.90 / 0.001 / 0.65 (unmodified).

## Metrics (binary witness)

| gate | value | threshold | verdict |
|---|---|---|---|
| S_coverage | 0.9805524239007892 | >= 0.9000 | PASS |
| Phi_halluc | 0.00009684761028521622 | <= 0.0010 | PASS |
| Q_density | 0.9999031523897148 | >= 0.6500 | PASS |

## Receipt (from `docs/doc-hdit.receipts.jsonl`)

- verdict ACCEPTED
- subject `668585cebc70bf620a26353d2a6d3fc07d62d7661da068a26607429f9d40bbe7`
- chain hash `4268a731860f6349e84700dec8d5d54ccdf14203eaf8a3cc5ff59e6556cf51ac`
  (chain root — parent hash empty)
- extractor BLAKE3 identity
  `1b5d967679c730756a18addedea9a6e829b949d6aee9aefef939896370a56038`
- timestamp 1791570878

## Pin record

- Certified at fleet pin
  `f51d81ac4f7e4119dff950327237effee4968f4aa9aba52c7d62c5362f441fa9`
  (`ggen-marketplace/scripts/gen_doc_surface.py` @ `b99942bdb`; bytes
  recovered via `git show b99942bdb:scripts/gen_doc_surface.py`, sha256
  verified before extraction).
- The lane brief's pin `4c862576ab…` was superseded the same date by R34/R43
  (`docs/sjira/v26.10.8/PIN-ROTATION-LEDGER.md` in ggen-marketplace): 6/6
  old-pin hits dispositioned historical, zero live-standing receipts on the
  old pin. Certifying at the stale pin would have minted a live-standing
  old-pin receipt the ledger explicitly closed. No thresholds moved; the
  rotated extractor is a superset of the old extractor's claim shapes.
- The receipt-native pin form is the BLAKE3 identity
  `1b5d9676…6038` (fleet law [150]).

## Per-span disposition (M3's Φ_halluc 0.0027 finding)

M3 recorded the finding against the v3 surface (308 modules / 2684 items) and
the pre-P3 VSA-coverage audit. Replayed at the pinned extractor against HEAD:

- The three named spans (`docs/archive/DOD_BENCHMARKING.md`,
  `docs/roadmap/W2-doctor-self-healing.md`, `docs/WASM.md`) no longer produce
  phantoms: extractor fixes 2a7355419 (spec-tier typing) + 42b031cfa
  (over-extraction filter) classify them prose-artifact or spec-tier,
  excluded from the Φ denominator by court law — not by editing them.
- Residual Φ mass is 2 claims (0.0001 <= 0.001, disclosed):
  `initialize`/`shutdown` `has_param` claims in
  `docs/integrations/LSP_MAX_INTEGRATION_CODE_TEMPLATES.md`. These are LSP
  protocol method names (protocol strings, not repo symbols). This lane's
  de-tick fix for them was reverted by the integrator in `c60da06`
  ("not generator-owned"); the residual stands disclosed under the gate.
- Coverage remediation this lane: `docs/reference/generated/` re-scaffolded
  from the pinned-extractor surface (S 0.8162 -> 0.9806; the v3 scaffold
  predated the const/str_key surface extraction).

## Tag disposition

`v26.10.8` exists at `9da04a4` and its subject is receipt-cited externally
(ggen-marketplace `docs/sjira/v26.10.8/_CLOSURE_RECEIPT.md:37` "re-gen frozen
at tag"). Per lane instruction: minted annotated **`v26.10.8-2`** at the
certified landing commit instead of moving `v26.10.8`.
