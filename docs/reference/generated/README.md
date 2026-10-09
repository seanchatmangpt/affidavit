# Generated API reference (doc-hdit)

GENERATED — do not edit by hand. The tables in `reference.md` are rendered
from the Rust code surface; the only agent-writable region in any file
here is the fenced `AGENT-COMMENTARY` slot.

## Regenerate

```sh
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code /Users/sac/affidavit \
  > /tmp/hdit/affidavit.code.v3.json
/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit scaffold \
  --code /tmp/hdit/affidavit.code.v3.json \
  --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates \
  --out docs/reference/generated
```

## Audit

```sh
# Merge grounded claims (one per surface item) into the surface JSON, then:
doc-hdit audit /tmp/hdit/affidavit.inputs.after.json
```

Last certify (v4 surface, 356 modules, 3643 public-surface items; 2026-10-09):

| gate        | value  | threshold |
| ----------- | ------ | --------- |
| S_coverage  | 0.9806 | >= 0.9000 |
| Phi_halluc  | 0.0001 | <= 0.0010 |
| Q_density   | 0.9999 | >= 0.6500 |

Verdict ACCEPTED — chained receipt in `docs/doc-hdit.receipts.jsonl`
(subject `668585cebc70bf62…`, chain head `4268a731860f6349…`); details in
`docs/sjira/v26.10.8/DOC-HDIT-CERTIFY-RECEIPT.md`.

Extractor: fleet pin `f51d81ac4f7e4119dff950327237effee4968f4aa9aba52c7d62c5362f441fa9`
(`gen_doc_surface.py` @ ggen-marketplace `b99942bdb`; BLAKE3 receipt identity
`1b5d967679c730756a18addedea9a6e829b949d6aee9aefef939896370a56038`) — the
original campaign pin `4c862576ab…` was superseded 2026-10-09 by R34/R43
(`PIN-ROTATION-LEDGER.md`, no live-standing receipts on the old pin).
Residual Phi mass: 2 `has_param` claims (`initialize`/`shutdown` LSP
protocol method names in `docs/integrations/LSP_MAX_INTEGRATION_CODE_TEMPLATES.md`
— protocol strings, not repo symbols; 0.0001 <= 0.001, disclosed).
