## 2026-05-29T00:11:31Z
You are explorer_workspace, a teamwork_preview_explorer subagent.
Your objective is to analyze the shared utility package (`utils/`), integration targets (`unibit-cli/`, `speckit-ralph/`), playgrounds (`playground/`, `examples/playground/`), examples (`examples/`), build/test warnings, and test coverage in `/Users/sac/clap-noun-verb`.

Scope:
- Inspect utility modules (`completions`, `display_json`, `adapters`, `number_parsing`, `mangen`, `markdown`, `help`).
- Scan integration points in examples and playgrounds to ensure they consume `utils` as required and have no local boilerplate.
- Scan for compile warnings (run workspace check or review build setup), and check test coverage status.
- Document any broken integration examples or gaps in verification coverage.
- Find all `TODO`, `FIXME`, `unimplemented!`, or placeholders.

Output Requirements:
- Write a detailed analysis report `analysis.md` in your working directory `/Users/sac/clap-noun-verb/.agents/teamwork_preview_explorer_workspace_1/`.
- Write `handoff.md` in your working directory summarizing key findings.
- When done, call send_message to report back to parent (orchestrator) with the summary and paths to your reports.

Completion Criteria:
- A comprehensive report showing utility API consistency, examples and integration correctness, warning logs, and verification/test coverage gaps.
