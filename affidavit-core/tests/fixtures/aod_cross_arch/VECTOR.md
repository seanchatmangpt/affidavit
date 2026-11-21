# AOD cross-arch golden vector (AFFI-SPEC-26.11.19 PR-3)

schema=affidavit-core/aod/cross-arch/v1

## Logical inputs (reseal these; never edit sealed bytes by hand)

| Field | Value |
|---|---|
| ontology_hash | `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa` |
| lease_nullifier | `bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb` |
| folded_receipt | `0000000000000000000000000000000000000000000000000000000000000000` |
| lease_epoch | `1` |
| row_count | `2` |
| page (16 B, stride 8) | `0102030405060708090a0b0c0d0e0f10` |

## Pins

| Artifact | Value |
|---|---|
| `vector_v1.aod` length | `544` |
| `vector_v1.blake3` | `2f8f3c7651121787bcb1f8365310fbcf9769aec69780dd0bd88e9bc0188fbdfb` |
| Initial seal host | `x86_64` (little-endian) |

## Standing

- **Court:** `cargo test --test aod_cross_arch` (fails closed on byte/digest drift).
- **CI:** `.github/workflows/aod-cross-arch.yml` matrix (`ubuntu-latest` × `ubuntu-24.04-arm`).
  When both legs green, compare job asserts `x86_64 digest == aarch64 digest == pin`
  (`AOD_CROSS_ARCH_ALIVE`). If ARM runners do not admit for the repo, golden-vector
  standing is the honest ceiling — do not invent dual-arch evidence.
- **Receipt (real dual-arch):** GitHub Actions run
  [`44841738900`](https://github.com/seanchatmangpt/affidavit/actions/runs/44841738900)
  on `cursor/aod-cross-arch-ffa4` — both `aod seal court (x86_64)` and
  `aod seal court (aarch64)` SUCCESS; compare job printed
  `AOD_CROSS_ARCH_ALIVE digest=2f8f3c7651121787bcb1f8365310fbcf9769aec69780dd0bd88e9bc0188fbdfb`.
- **Refresh:** reseal the logical inputs above via `seal_aod`, replace
  `vector_v1.aod` + `vector_v1.blake3`, update this table. Never silent overwrite
  without a typed `REFUSED_AOD_CROSS_ARCH_*` diagnosis.
