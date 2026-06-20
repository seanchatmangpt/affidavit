# rust-boilerplate

The `seanchatmangpt` Rust **house style**, as a kit: one place that defines how
every Rust repo in this account should be scaffolded, linted, built, and
released — plus the tooling to apply it to new and existing repos.

It was derived empirically, not invented: 10 agents surveyed 17 public repos
plus `affidavit`, and the findings were consolidated into
[`survey/00-SYNTHESIS.md`](survey/00-SYNTHESIS.md).

## What's here

| Path | What it is |
|------|-----------|
| `template/` | The canonical scaffold — Cargo metadata + `[lints]`, `rustfmt.toml`, `deny.toml`, `typos.toml`, `.editorconfig`, `rust-toolchain.toml`, `.github/` (CI + release + dependabot + PR template), `justfile`, dual license, doc skeletons, `src/`. Ships a single-crate `Cargo.toml` and a `Cargo.workspace.toml` variant. |
| `crates/chatman-common/` | The shared **house crate** — unified `Error`/`Result`, tracing/otel init, CLI bootstrap, blake3 provenance helpers, testkit. Feature-gated; `cargo check`-clean. |
| `apply.sh` | Drops the project-agnostic standardization files from `template/` into an existing repo. |
| `CHECKLIST.md` | Per-repo refactor checklist for the whole fleet (public + private). |
| `BROADEN-ACCESS.md` | How to let a Claude Code session reach repos beyond `affidavit`. |
| `survey/` | The raw 10-agent reports + the consolidated synthesis. |

## Use it

### New project
```bash
cargo generate --git https://github.com/seanchatmangpt/rust-boilerplate template
# (or copy template/ by hand; pick Cargo.toml for a single crate,
#  Cargo.workspace.toml for a workspace)
```

### Existing repo
```bash
/path/to/rust-boilerplate/apply.sh . --dry-run   # preview
/path/to/rust-boilerplate/apply.sh .             # apply
# then do the manual Cargo.toml steps it prints (lints + metadata)
```

### Shared crate
```toml
# in a repo's Cargo.toml
chatman-common = { git = "https://github.com/seanchatmangpt/chatman-common", features = ["telemetry", "provenance"] }
```

## House defaults (the rulings)

| Axis | Default |
|------|---------|
| Version | CalVer `YY.M.patch` |
| Edition | 2021 (opt-in 2024 per repo) |
| MSRV | 1.82, verified in CI |
| Toolchain | pinned stable `1.82.0` (nightly only for `#![feature]` crates) |
| License | dual `MIT OR Apache-2.0`, **both files committed** |
| Errors | `thiserror` 2 |
| Task runner | `just` |
| Lints | `[workspace.lints]`; `unsafe_code = forbid`; clippy `all` + `pedantic` warn; `todo`/`unimplemented`/`dbg_macro` deny |

The evidence behind each ruling is in `survey/00-SYNTHESIS.md` (§6).

## Why this lives inside `affidavit`

It was built in a session scoped to the single writable repo `affidavit`. The
kit is self-contained and depends on nothing in affidavit; it is meant to
graduate into its own `seanchatmangpt/rust-boilerplate` repo. See
`BROADEN-ACCESS.md` for how to roll it out to the rest of the fleet.
