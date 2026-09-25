# affidavit: triage 12 PR-less unmerged remote branches

- Standing: OPEN
- Created: 2026-09-19 (v26.9.19 gh survey wave)
- Source: origin branches not merged into `main` with no open PR
- Evidence: `git branch -r --no-merged origin/main`: agent/format-result-20260812 agent/format-transport-20260812 audit/stubs-wip-2026-08-08 brand/forward-deployment-os-2026-08 claude/confident-mendel-mrolti claude/v26-9-6-release-ua9k2i claude/vigilant-hawking-l7is37 feat/breed-aware-receipt-model feat/dfcm-federated-capabilities-v26.9.1 feat/ecosystem-standing-receipts-20260812 feat/real-compliance-gates worktree-agent-a92dd597bb24e3709

## Work to complete
- Triage each branch: land (open a PR) or delete (`git push origin --delete <branch>`). Work in batches; record decisions in History.

## Acceptance
- `git branch -r --no-merged origin/main` is empty after `git fetch --prune`.

## History
- 2026-09-19 | OPEN | survey found 12 PR-less branches | full list above | triage pending
