<!--
  affidavit PR template.

  Repo ethos: "a green that is true whether or not the work happened carries
  no information." Be honest about what you actually verified versus what you
  did not — an unchecked box is more useful than a falsely checked one.
-->

## What & why

<!-- One or two sentences: what does this change do, and why? -->

## What I verified (vs. did not)

<!--
  Describe what you actually ran and observed. Distinguish real evidence from
  assumptions. If you could not run something, say so plainly.
-->

- 

## Checklist

### Web (`web/`) — the reliable green signal

- [ ] `cd web && npm ci` succeeds
- [ ] `cd web && npx tsc --noEmit` passes (type-check)
- [ ] `cd web && npm run build` passes
- [ ] Web CI (`.github/workflows/web.yml`) is green on this PR

### Rust (only if you touched Rust)

> Note: the `affidavit` crate depends on FIVE sibling PATH crates that live
> OUTSIDE this repo (`../clap-noun-verb`, `../wasm4pm*`, `../lsp-max`,
> `../clnrm/...`, `../chicago-tdd-tools`). A clean checkout of this repo alone
> CANNOT `cargo build`/`test`/`clippy`. The full sibling workspace is required
> to actually build and run the Rust code or `examples/golden_run.sh`.

- [ ] `cargo fmt --all -- --check` passes (this needs no sibling crates and IS gated in CI)
- [ ] I built/tested against the full sibling workspace locally, OR I state below that I did not
- [ ] If applicable, `bash examples/golden_run.sh` was run against the full workspace

## Notes

<!-- Anything reviewers should know: caveats, follow-ups, what's intentionally out of scope. -->
