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

> The root crate resolves from crates.io plus in-repo `stubs/` and `vendor/`,
> so a clean checkout can `cargo build`/`test`/`clippy`. Only the standalone
> `courts/sj-parity/` court needs a local clnrm checkout.

- [ ] `cargo fmt --all -- --check` passes (this needs no sibling crates and IS gated in CI)
- [ ] `cargo build --all-targets` and `cargo test --all-targets` pass
- [ ] If applicable, `bash examples/golden_run.sh` was run

## Notes

<!-- Anything reviewers should know: caveats, follow-ups, what's intentionally out of scope. -->
