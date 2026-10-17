# RELEASE-TIMELINE — affidavit — v26.10.17 wave (2026-10-16 night → 2026-10-17)

| field | value |
|---|---|
| id | AFFI-261017-01-TL |
| repo | `/Users/sac/affidavit` |
| branch | `v26.10.16` (in-tree wave) |
| window | 2026-10-16 22:50 → 2026-10-17 08:12 |
| source | `git log --oneline --format="%h %ad %s" --date=format:"%H:%M"` — every timestamp below is from git, not memory |
| standing | UNKNOWN — closing gate receipts not yet witnessed (see Phase 5) |

All times local, 2026-10-16 (PM) / 2026-10-17 (AM) as marked.

## Phase 1 — Wave-1 explore (16 lanes) — opened 2026-10-16 22:50

Wave substrate and the 16-lane dependency-mining survey.

| time | anchor commit | event |
|---|---|---|
| 22:50 | `ef061db` | feat(affidavit-core): law-complete rebuild — zero in-crate crypto, PQC court, BLAKE3 chain, OCEL v2, wasm-bound seams (wave substrate) |
| 23:02 | `1f7103a` | fix(justfile): close validate-ladder gaps — core feature planes, golden, wasm courts |
| 23:02 | `fd04c66` | docs: reconcile operator-law claims with code (16-lane survey wave) |
| 23:03 | `247d4b6` | **docs(reference): capability register from 16-lane dependency-mining wave 1** (phase anchor) |

## Phase 2 — ERRC audit lanes + capability cycles 1–5 — 2026-10-16 23:04 → 2026-10-17 01:08

Audit/ERRC lanes interleaved with the capability-register generation cycles.

| time | anchor commit | event |
|---|---|---|
| 23:04 | `d7c40d7` / `2af1142` / `c774a2e` | docs reconciliation cluster (wasm, core, operator law) |
| 23:11 | `f9da11b` | **capability register cycle-2** — 25 new proposals, 4 delete-class findings |
| 23:40 | `e37a717` | **capability register cycle-3** — rows 48–78 (30 proposals) |
| 00:10 | `ea24b1c` | fix(ontology): close FM-LAW-020 orphans; ggen render green; re-pin artifact |
| 00:12 | `fc87f56` | **capability register cycles 4–5** — rows 79–102 |
| 01:08 | `abcea17` | **CLI vacuity audit + 80/20 ERRC — 101 verbs, all executed** (ERRC lane anchor) |

Cycle-5 landing surge (00:45–01:59): 30+ commits landing rows 5/20/22/23/24/34/39/40/44/45/48/61/63/74/79–82, including confirmed defects `3bb560c` (00:46 FROST nonce, row 81c) and `84a430f` (00:53 BLS rogue-key FALSIFIED, row 82), ERRC reduce/raise/create classes `d7f4443`/`a48a5a7`/`5b8589b` (01:33–01:44), and the last THIN pair `e9a89be` (01:48).

## Phase 3 — Stabilization toward the bump — 2026-10-17 01:55 → 02:07

| time | anchor commit | event |
|---|---|---|
| 01:55–01:59 | `6c14830`, `1a81553`, `63055dd`, `580c4dd` | crypto-trust API migration + e2e fixtures + tripwire suite serialization |
| 02:00–02:03 | `8624548`, `e79ef96` | e2e courts aligned to the diag exit catalog; pqc-plane scoping fix |
| 02:05–02:07 | `38a9ed4`, `2c3a6a0` | README verb count 101 → 100 (release_identity court) |

## Phase 4 — Post-bump re-pin — 2026-10-17 02:13 → 08:12

| time | anchor commit | event |
|---|---|---|
| 02:13 | `568beb9` | **chore(release): bump CalVer to 26.10.17 in unison** + changelog citing real exits (release base, 7 files) |
| 02:52 | `b1726cb` | chore(gitignore): lane build-root lease dir `affidavit-wasm/target-repro/` (two-target-dir reproducibility invariant for the re-pin) |
| 08:04 | `11f2b59` | docs: post-bump version sweep (5 stale-current sites → 26.10.17) + KD-BLS-1 registered |
| 08:08 | `b0325d4` | docs(reference): capability register disposition sweep — 50 LANDED / 49 OPEN / 2 ADJUDICATION |
| 08:12 | `1b6411d` | docs(sjira): root work order AFFI-261017-01 (6 acceptance cells; pin cell PENDING-REPIN against digest `b3cf38ba`) |

Working-tree re-pin residue (uncommitted at window end): `affidavit-wasm/registry/ARTIFACTS.sha256`,
`artifact-pin.json`, `capability-registry.json`, `op-examples.json`, `dist/affidavit_wasm.wasm`,
and regenerated projections (`ffi.rs`, `abi.rs`, `abi_meta.rs`) — the post-bump artifact refresh in flight.

## Phase 5 — Closing events: two consecutive green gate passes

Per work order `AFFI-261017-01` cells 1–2, the wave closes only with:

1. `just validate` → exit 0 (pass A)
2. `just validate` → exit 0 (pass B, immediately after A, no tree change)

These two consecutive green passes are the closing events of the timeline. **No git-committed
receipt for either pass exists yet** — both cells are PENDING on base `568beb9`+ in the work
order, and the pin cell (cell 6) is PENDING-REPIN. Until witnessed, the release standing of
this wave is UNKNOWN; a gate that executes 0 tests is a loud FAIL, not a pass.
