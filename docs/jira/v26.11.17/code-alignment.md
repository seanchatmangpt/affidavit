# code-alignment — affidavit v26.11.17

Standing: PASS

## Identity
- Path: `/Users/sac/affidavit`
- Branch at generation: main
- HEAD: ff3dc1a052720e74986f8abcdb6aa6d45ff6bffd
- Tracked files: 1237
- Tracked docs markdown: 128

## Build manifests (tracked at the commit above)
- `Cargo.toml`
- `justfile`
- `ggen.toml`
- `CLAUDE.md`
- `AGENTS.md`
- `README.md`

## Top-level layout (tracked file counts)
- `.`: 21
- `.cargo`: 1
- `.chatmangpt`: 1
- `.claude`: 3
- `.devcontainer`: 1
- `.ggen`: 13
- `.github`: 16
- `.qme`: 1
- `affidavit-core`: 27
- `affidavit-wasm`: 36
- `benches`: 13
- `completions`: 5
- `dist`: 2
- `docs`: 130
- `ecosystem`: 1
- `examples`: 28
- `fixtures`: 15
- `js`: 187
- `mutations`: 1
- `ontology`: 4
- `praxis`: 44
- `profiles`: 1
- `reference`: 2
- `release`: 1
- `schemas`: 3
- `scripts`: 20
- `semconv`: 4
- `src`: 235
- `stubs`: 7
- `tests`: 228
- `thesis`: 37
- `tools`: 42
- `vendor`: 72
- `web`: 35

## Prior doc waves
- `docs/jira/v26.11.16`

## Falsifiers
- Any tracked path above absent at HEAD, or a count differing from `git ls-files`.
- HEAD sha not equal to or an ancestor of the live HEAD.

## Acceptance
```bash
python3 ~/zcode-cli/scripts/docs_v261117.py validate /Users/sac/affidavit
```
Expected: exit 0 and `PASS`.

## See Also
- `~/zcode-cli/scripts/docs_v261117.py`
