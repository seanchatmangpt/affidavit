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

Last audit (v3 surface, 308 modules, 2684 items):

| gate        | value  | threshold |
| ----------- | ------ | --------- |
| S_coverage  | 0.9966 | >= 0.9000 |
| Phi_halluc  | 0.0027 | <= 0.0010 |
| Q_density   | 0.9973 | >= 0.6500 |

Phi_halluc exceeds threshold from 3 pre-existing prose spans in
`docs/archive/DOD_BENCHMARKING.md`, `docs/roadmap/W2-doctor-self-healing.md`
and `docs/WASM.md` — outside this generated tree.
