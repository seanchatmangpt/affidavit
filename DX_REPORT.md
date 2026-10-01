# DX / Repo Health Report

_Generated at 2026-06-14T22:53:11.611Z by `tools/dx-report.mjs` (machine-generated — do not edit by hand)._

**Repo:** Provenance Layer — receipt assembly and certification (verify a witness against a format standard; never decide honesty).

## Rust

| Metric | Value |
| --- | --- |
| `.rs` files (tree-wide) | 168 |
| Total Rust LOC | 10101 |
| `#[test]` occurrences | 346 |
| `#[tokio::test]` occurrences | 0 |
| CLI verbs (`src/verbs/*.rs`) | 12 |
| Examples (`examples/*.rs`) | 9 |
| Integration test files (`tests/*.rs`) | 124 |
| Compile-fail UI fixtures (`tests/ui/compile_fail/*.rs`) | 9 |

## Web

| Metric | Value |
| --- | --- |
| Next.js routes (`web/app/**/page.tsx`) | 10 |
| API routes (`web/app/api/**/route.ts`) | 2 |
| Web TS LOC (`web/**/*.{ts,tsx}`, excl. node_modules/.next) | 5272 |

## Docs

| Metric | Value |
| --- | --- |
| Top-level `*.md` files | 34 |
| Total top-level markdown lines | 13635 |
| Coverage 🟢 count (`reference/COVERAGE.md`) | 162 |

## Tooling

| Metric | Value |
| --- | --- |
| GitHub workflows (`.github/workflows/*.yml`) | 2 |
| `justfile` present | yes |
| `.devcontainer` present | yes |
| Files under `scripts/` | 4 |
| Files under `completions/` | 3 |

## semconv

| Metric | Value |
| --- | --- |
| YAML files under `semconv/` | 4 |

---

### How this was measured

All values above are mined from the live repository tree at run time by `tools/dx-report.mjs`, a dependency-free Node ESM script (built-ins only). The walker starts at the repo root, skips `.git`, `node_modules`, `target`, `.next`, `dist`, and `build`, and never follows directory symlinks. No metric is hardcoded or read from a fixture; anything that could not be measured is shown as `n/a`. Counts of `#[test]` / `#[tokio::test]` are textual proxies for unit/async tests, not a compiler-verified test count.
