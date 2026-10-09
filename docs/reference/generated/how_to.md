# How to: Using affidavit

## Prerequisites


- affidavit-core/src/accumulator/mmr.rs::MmrAccumulator (struct)

- affidavit-core/src/accumulator/mmr.rs::MmrAccumulator (struct)

- affidavit-core/src/accumulator/mmr.rs::MmrError (enum)

- affidavit-core/src/accumulator/mmr.rs::MmrError (enum)

- affidavit-core/src/accumulator/mmr.rs::MmrProof (struct)

- affidavit-core/src/accumulator/mmr.rs::MmrProof (struct)

- affidavit-core/src/accumulator/mmr.rs::MountainPeak (struct)

- affidavit-core/src/accumulator/mmr.rs::MountainPeak (struct)

- affidavit-core/src/accumulator/mmr.rs::append (function)

- affidavit-core/src/accumulator/mmr.rs::append (function)

- affidavit-core/src/accumulator/mmr.rs::append_payload (function)

- affidavit-core/src/accumulator/mmr.rs::append_payload (function)

- affidavit-core/src/accumulator/mmr.rs::bag_peaks (function)

- affidavit-core/src/accumulator/mmr.rs::bag_peaks (function)

- affidavit-core/src/accumulator/mmr.rs::hash_children (function)

- affidavit-core/src/accumulator/mmr.rs::hash_children (function)

- affidavit-core/src/accumulator/mmr.rs::hash_leaf_payload (function)

- affidavit-core/src/accumulator/mmr.rs::hash_leaf_payload (function)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:bag: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:bag: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:leaf: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:leaf: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:node: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mmr:v1:node: (str_key)

- affidavit-core/src/accumulator/mmr.rs::mountain_heights (function)

- affidavit-core/src/accumulator/mmr.rs::mountain_heights (function)

- affidavit-core/src/accumulator/mmr.rs::new (function)

- affidavit-core/src/accumulator/mmr.rs::new (function)

- affidavit-core/src/accumulator/mmr.rs::num_leaves (function)

- affidavit-core/src/accumulator/mmr.rs::num_leaves (function)

- affidavit-core/src/accumulator/mmr.rs::peaks (function)

- affidavit-core/src/accumulator/mmr.rs::peaks (function)

- affidavit-core/src/accumulator/mmr.rs::prove (function)

- affidavit-core/src/accumulator/mmr.rs::prove (function)

- affidavit-core/src/accumulator/mmr.rs::root (function)

- affidavit-core/src/accumulator/mmr.rs::root (function)

- affidavit-core/src/accumulator/mmr.rs::verify (function)

- affidavit-core/src/accumulator/mmr.rs::verify (function)

- affidavit-core/src/accumulator/mod.rs::mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } (use)

- affidavit-core/src/accumulator/mod.rs::mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } (use)


## Steps


1. Use `MmrError` from `affidavit-core/src/accumulator/mmr.rs`.

2. Use `append` from `affidavit-core/src/accumulator/mmr.rs`.

3. Use `append_payload` from `affidavit-core/src/accumulator/mmr.rs`.

4. Use `bag_peaks` from `affidavit-core/src/accumulator/mmr.rs`.

5. Use `hash_children` from `affidavit-core/src/accumulator/mmr.rs`.

6. Use `hash_leaf_payload` from `affidavit-core/src/accumulator/mmr.rs`.

7. Use `mountain_heights` from `affidavit-core/src/accumulator/mmr.rs`.

8. Use `new` from `affidavit-core/src/accumulator/mmr.rs`.

9. Use `num_leaves` from `affidavit-core/src/accumulator/mmr.rs`.

10. Use `peaks` from `affidavit-core/src/accumulator/mmr.rs`.

11. Use `prove` from `affidavit-core/src/accumulator/mmr.rs`.

12. Use `root` from `affidavit-core/src/accumulator/mmr.rs`.


## Verified snippet

<!-- The snippet slot carries code copied from the extracted code surface -->
<!-- (doc:Claim rows whose doc:attribute is "snippet"), never agent prose. -->

```rust
// affidavit-core/src/accumulator/mmr.rs :: append
append(&mut self, leaf: Digest) -> u64
```

<!-- AGENT-COMMENTARY-BEGIN -->
<!-- The ONLY region an agent may write into. Bounds: <= 12 lines,    -->
<!-- <= 100 chars/line, no new code facts (any new symbol mentioned   -->
<!-- must exist in queries/ast_extract.rq output; the doc_quality     -->
<!-- court fails Phi_halluc > 0.001 otherwise). No tables, no         -->
<!-- signatures, no parameters, no error lists — AGENT-FORBIDDEN      -->
<!-- everywhere.                                                      -->
<!-- AGENT-COMMENTARY-END -->
