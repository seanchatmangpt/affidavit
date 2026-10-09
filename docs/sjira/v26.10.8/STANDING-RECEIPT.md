# STANDING-RECEIPT — affidavit — v26.10.8

Lane R127, 2026-10-09. Repo `/Users/sac/affidavit`, branch `main`. Completes
the R90 standing-receipt pattern for affidavit; folds in R126's tag check.

## Standing

**CERTIFIED** — subject-bound at HEAD `1380ac7f72b0f395c511178124aa22c2873698a2`.

The certify receipt (see `DOC-HDIT-CERTIFY-RECEIPT.md`, this directory) is
ACCEPTED with gates inside thresholds:

| gate | value | threshold | verdict |
|---|---|---|---|
| S_coverage | 0.9805524239007892 | >= 0.9000 | PASS |
| Phi_halluc | 0.00009684761028521622 | <= 0.0010 | PASS |
| Q_density | 0.9999031523897148 | >= 0.6500 | PASS |

Receipt-native fields re-read from the binary witness
`docs/doc-hdit.receipts.jsonl` (last entry):

- verdict `ACCEPTED`
- subject `668585cebc70bf620a26353d2a6d3fc07d62d7661da068a26607429f9d40bbe7`
- chain hash `4268a731860f6349e84700dec8d5d54ccdf14203eaf8a3cc5ff59e6556cf51ac`
- extractor BLAKE3 identity
  `1b5d967679c730756a18addedea9a6e829b949d6aee9aefef939896370a56038`
- thresholds 0.90 / 0.001 / 0.65 (unmodified)

## Pin lineage

- Receipt was certified at fleet pin
  `f51d81ac4f7e4119dff950327237effee4968f4aa9aba52c7d62c5362f441fa9` (gmp
  commit `b99942bdb`) — **historical, hop 1** of the rotation lineage.
- Single stable fleet-pin reference is
  `ggen-marketplace/docs/sjira/v26.10.8/PIN-ROTATION-LEDGER.md`: hop 2
  `b88297e6…` @ `2c81947e4` (R46), **hop 3 (current)**
  `3d2abae19dac9f529b8250a0f96a02b34dbf86348dad241fbdb003410dc35590` (R64,
  module-level coverage denominator). Sibling notes citing `f51d81ac` are
  historical; this receipt's pin is bound to its certified subject and does
  not claim live-pin standing.
- The receipt-native pin form is the BLAKE3 identity `1b5d9676…6038` (fleet
  law [150]).

## Tag coverage

- `v26.10.8` @ `9da04a4` — frozen, subject receipt-cited externally
  (ggen-marketplace `docs/sjira/v26.10.8/_CLOSURE_RECEIPT.md:37`); not moved.
- `v26.10.8-2` @ `1380ac7` — annotated tag, resolves to commit
  `1380ac7f72b0f395c511178124aa22c2873698a2` == current HEAD
  (`git rev-parse v26.10.8-2^{commit}` witnessed 2026-10-09). **Covers HEAD.**
- Drift check `git log --oneline 1380ac7..HEAD`: **empty** — zero commits
  landed after the certified landing commit. No `v26.10.8-3` minted; the
  mint-a-new-tag law triggers only on advancement past the covered tip.
- Disclosure: the current ledger pin (hop 3) is newer than the extractor
  identity in this receipt (hop 1). The certify is subject-bound to
  `668585ce…` at `1380ac7`, not pin-bound; a re-certify at hop 3 would be
  required only if docs advance past `1380ac7`.

## Standing match

Standing CERTIFIED is supported: (a) ACCEPTED verdict with all three gates
PASS at unmodified thresholds, witnessed in `docs/doc-hdit.receipts.jsonl`;
(b) subject digest `668585ce…` bound to the docs state certified at `c60da06`
inputs; (c) tag `v26.10.8-2` resolves to the same commit as HEAD, so the
certification covers the current tip exactly; (d) zero post-certify drift
(`1380ac7..HEAD` empty). Known residuals (Φ mass 2 protocol-string claims in
`docs/integrations/LSP_MAX_INTEGRATION_CODE_TEMPLATES.md`) are disclosed in
the certify receipt and remain under-gate, not standing-breaking.

## Falsifier

Any of the following breaks this standing receipt: a commit landing after
`1380ac7` without re-certification; `v26.10.8-2` failing to resolve to
`1380ac7`; the `docs/doc-hdit.receipts.jsonl` tail entry changing verdict or
subject; thresholds moving; the pin ledger minting a hop that revokes the
`1b5d9676…` identity as invalid.
