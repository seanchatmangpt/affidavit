# affidavit migration plan v26.11.16

Standing: UNVERIFIED

## Identity
- Path: /Users/sac/affidavit
- Remote: https://github.com/seanchatmangpt/affidavit.git
- Branch: main@e5313a0 (tracks origin/main, ahead 0 / behind 0)
- Last commit: 2026-11-09 ("docs: re-verify and fix stale docs against live code")
- Commits: 682; size 332 MB
- Languages/build: Rust (Cargo.toml workspace: affidavit-core, affidavit-wasm), justfile, ggen.toml
- Nested repos: none

## Observed state
- Dirty: 0; untracked: 0 (clean working tree)
- Ahead/behind origin/main: 0/0
- Stash count: 272 (census) -- extremely large; contents unknown
- Local branches: 12; 6 unpushed per census. Branches with no upstream seen:
  feat/v26.9.17-release-affidavit, fix/freeze-genesis-seed, fix/rustfmt-main-red;
  two others are behind their upstream (fix/affidavit-v26.9.17-boundary behind 8,
  feat/aloopt-26925-execution-receipt-profile behind 3)
- Build dirs: dist/ (708K); no target/ present
- Submodules: none; no .env files at top level

## What it is
Deterministic cryptographic provenance engine: assembles, seals, and certifies append-only
BLAKE3 chains of operation-events, emitting a 352-byte CTRT token with post-quantum (ML-DSA-65)
signatures and statutory anchors. Rust core plus a WASM crate.

## Classification
ACTIVE (census agrees; committed within the last week).

## Migration risk
- 272 stashes would be lost on a plain re-clone (stashes never push).
- 6 local branches without upstream (unpushed commits).
- No uncommitted work, no local-only data files observed.
- Rate: MEDIUM

## Recommended action
COMMIT-PUSH

1. DECISION: triage the 272 stashes. List: `git stash list | head -50`. If any matter,
   `git stash branch stash-<n> stash@{n}` then push; otherwise accept loss.
2. Push unpushed branches: `git push -u origin feat/v26.9.17-release-affidavit
   fix/freeze-genesis-seed fix/rustfmt-main-red` (and any others shown by
   `git branch -vv | grep -v '\['`).
3. Safety net before wiping: `git bundle create /Volumes/ARCHIVE/affidavit.bundle --all`
   (bundles include all refs but not stashes; stashes must be branched first).
4. Cleanup not needed (no target/); dist/ is 708K and tracked-or-trivial.
5. On the new machine clone fresh; do not rsync this directory.

## First commands on the new machine
- `git clone https://github.com/seanchatmangpt/affidavit.git && cd affidavit`
- `cargo check --workspace` then `just --list` (stated, not run)

## See Also
- No previous docs/jira dir exists in this repo.
- /Users/sac/docs/jira/v26.11.16/README.md (master index)
