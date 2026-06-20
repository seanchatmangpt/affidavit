# Contributing

Thanks for contributing. This repo follows the seanchatmangpt Rust house style.

## Getting started

```bash
cargo install cargo-deny typos-cli just
just            # list tasks
just ci         # run everything CI runs
```

The toolchain is pinned to stable `1.82.0` via `rust-toolchain.toml`.

## Development workflow

1. Branch from `main`.
2. Make your change, with tests.
3. Run `just ci` until green (fmt, clippy `-D warnings`, test, deny, typos).
4. Add a `CHANGELOG.md` entry under `## [Unreleased]`.
5. Open a PR with the template; check only what you actually verified.

## Code style

- `cargo fmt` is law (`rustfmt.toml`, stable options only).
- No new `unwrap`/`expect`/`panic` in library code paths.
- Public items get rustdoc (`missing_docs` is on).
- Errors via `thiserror`; binaries may add context with `anyhow`.

## Versioning & releases

CalVer `YY.M.patch`. A release is cut by pushing a `vYY.M.patch` tag, which
triggers `.github/workflows/release.yml`.

## License

By contributing you agree your work is licensed under MIT OR Apache-2.0.
