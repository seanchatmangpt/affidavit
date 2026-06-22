# Contributing to affidavit

Thanks for working on **affidavit** — the Provenance Layer (`affi` CLI). This
guide covers the doctrine you are contributing into, the one cultural rule that
decides whether a change is *done*, and the practical mechanics of building,
testing, and shipping.

Read this alongside:

- [`README.md`](README.md) — what the project is and the CLI surface.
- [`STATUS.md`](STATUS.md) — current implementation status and the admission
  criterion applied per session.
- [`ARDPRD.md`](docs/archive/ARDPRD.md) — the product/architecture requirements and the
  ADRs (the *why* behind the seal, the typestate, and the witnesses).

---

## Doctrine in brief

These four principles are not style preferences — they are the thing the code
exists to enforce. A change that violates them is wrong even if it compiles and
the tests are green.

1. **Certify, don't decide.** The verifier never decides whether work is
   *honest* — by Rice's theorem that question is undecidable. It *checks a
   witness* (the receipt) against a fixed format standard, and every check is
   decidable. We relocate the undecidable from the artifact to the **form of
   construction** (see ARDPRD §2).

2. **A receipt is an append-only BLAKE3 chain of operation-events.** Each link
   folds the previous chain hash with the canonical bytes of the next event, so
   any edit to any event propagates through every later link. There is no
   in-place update; you only ever append.

3. **Unverifiable work is rejected, not detected.** A tampered or malformed
   receipt simply fails a stage and yields `REJECT`. The verifier proves a
   lawful chain exists; it does not hunt for fraud.

4. **The bypass is unconstructable.** Receipt struct-literal construction fails
   at **compile time** — `E0451: field '_seal' is private`. Only the canonical
   seam ([`crate::chain::ChainAssembler::finalize`]) can construct sealed
   receipts, and the `Admitted` carrier mints **only** after the structural
   certify pipeline returns `ACCEPT`. This is enforced by construction (the
   type system), not by a runtime check that can be forgotten or removed (see
   ARDPRD ADR-2 / ADR-3, NFR-4).

---

## The admission criterion (the gate your work is judged by)

> **An integration is admitted only when removing it breaks a test that
> exercises the real capability.** A green that is true whether or not the work
> happened carries no information. No hollow stamps.

This is the repo's culture, stated in [`STATUS.md`](STATUS.md) and ARDPRD NFR-6
("witnessed surface"). Concretely, when you add or change a capability:

- **Add a witness that fails when the capability is removed.** If you can
  delete your implementation and the suite stays green, the test is decorative
  — it witnesses nothing. Fix the test, not the metric.
- **The witness must terminate outside its producer.** A compile-fail fixture
  proving the bypass is unconstructable; a behavioral test with a *negative
  control* proving the witness can actually fail.
- **A broken harness produces no number, not a fake one.** (A criterion bench
  that prints `0 measured` reported nothing; a real run prints `~2.4 µs`.)

Worked examples already in the tree (from STATUS.md):

- Remove the verdict check in `admission::admit()` →
  `forged_receipt_cannot_be_admitted` fails.
- Remove the `chicago-tdd-tools` dependency → `tests/chicago_tdd_witness.rs`
  does not compile.
- Remove the `trace_verify` wrapper → `verify_emits_an_observable_span` fails.

If you cannot point at the test that turns red when your change is reverted,
the change is not done.

---

## Building

### Rust crate (`affi`) — requires the sibling workspace

The crate is pinned to the **nightly** toolchain (see `rust-toolchain.toml`)
and the binary is `affi` (`src/bin/affi.rs`).

> **Honest build note.** `affidavit` depends on **five sibling PATH crates that
> are NOT vendored in this repository**:
>
> | Dependency | Path |
> | --- | --- |
> | `clap-noun-verb` (+ macros) | `../clap-noun-verb` |
> | `wasm4pm` | `../wasm4pm/wasm4pm` |
> | `wasm4pm-compat` | `../wasm4pm-compat` |
> | `lsp-max` | `../lsp-max` |
> | `clnrm-core` | `../clnrm/crates/clnrm-core` |
> | `chicago-tdd-tools` (dev-dep) | `../chicago-tdd-tools` |
>
> A bare, lone checkout **cannot** `cargo build` or `cargo test`. Those
> commands only succeed when the full sibling workspace is checked out
> alongside this repo. This is a real constraint — please do not file or write
> docs claiming a clean single-repo checkout builds the binary.

With the siblings in place, from the repo root:

```bash
cargo build           # builds the affi binary -> target/debug/affi
cargo test            # the lib + dispatch + e2e + ui suites
```

The one Rust task that works **without** the siblings is formatting, because it
needs no dependency resolution:

```bash
cargo fmt --all -- --check   # the CI formatting gate (needs the rustfmt component)
```

### Web app — self-contained

The Next.js app in [`web/`](web/) is fully self-contained (Node 22; see
`web/package.json`). It needs none of the sibling Rust crates:

```bash
cd web
npm ci
npm run build         # production build
npm run dev           # dev server
```

### Convenience entry points

| You want to… | Run |
| --- | --- |
| Set up local dev (toolchain check, web deps, sibling-gated cargo build) | `scripts/bootstrap.sh` |
| Start the web dev server | `scripts/web-dev.sh` |
| Run the checks that work here (fmt + web tsc) | `scripts/check.sh` |
| Run the end-to-end golden smoke | `scripts/golden.sh` |

The [`justfile`](justfile) exposes the same tasks as recipes (`just --list`);
its header carries the same sibling-workspace caveat. There is also a
[`.devcontainer/`](.devcontainer/devcontainer.json) that provisions Rust
nightly + Node 22 and runs `npm ci` on create.

---

## Running tests and the golden run

- **Unit / integration / UI tests** (needs the sibling workspace):

  ```bash
  cargo test
  ```

  This includes the compile-fail UI fixture that asserts `E0451` on Receipt
  struct-literal construction — the witness that the bypass is unconstructable.

- **The golden run** — the end-to-end tamper-teeth smoke. It drives the real
  `affi` binary through `emit → assemble → verify (ACCEPT, exit 0)`, then
  corrupts the receipt with `sed` and re-verifies, expecting `REJECT` with a
  non-zero exit. That ACCEPT-then-REJECT distinction is the admission witness
  for the whole pipeline (ARDPRD FR-6).

  ```bash
  scripts/golden.sh           # guarded wrapper (refuses early if siblings absent)
  # or directly:
  bash examples/golden_run.sh
  # or:
  just golden
  ```

- **Web checks** (self-contained):

  ```bash
  cd web && npx tsc --noEmit
  # or: scripts/check.sh   (also runs the Rust fmt gate)
  ```

When you add a verb, a stage, an admission rule, or a witness type, extend the
relevant test **and** make sure removing your change turns it red (the
admission criterion above).

---

## Code style

- **Rust:** formatted with **rustfmt** (`cargo fmt --all`); the CI gate is
  `cargo fmt --all -- --check`. 4-space indentation (rustfmt default; also
  pinned in [`.editorconfig`](.editorconfig)). Keep public items documented —
  each `pub` item cites the example that exercises it (see `DOC_COVERAGE_LOG.md`).
- **Web / TypeScript:** **2-space** indentation, LF line endings, UTF-8, final
  newline (enforced by `.editorconfig`). Keep it `tsc --noEmit` clean (this repo
  ships no ESLint config, so strict `tsc` + `next build` are the web gates).
- **Everything:** the repo's [`.editorconfig`](.editorconfig) is the source of
  truth for indentation, line endings, and trailing whitespace (Markdown keeps
  trailing whitespace — two trailing spaces are a hard break).

---

## Shell completions

Hand-authored completions for the `affi` CLI live in
[`completions/`](completions/). They are **authored from the documented CLI
surface** (this guide and the README), not auto-generated — the binary cannot
be built without the sibling workspace, so generation is not possible from a
lone checkout. Keep them in sync with the CLI when you change a verb or flag.

Install:

```bash
# bash — current shell, or copy into a bash-completion dir:
source completions/affi.bash
#   cp completions/affi.bash ~/.local/share/bash-completion/completions/affi

# zsh — name it _affi on an fpath directory, then re-run compinit:
mkdir -p ~/.zsh/completions
cp completions/affi.zsh ~/.zsh/completions/_affi
#   (ensure ~/.zshrc has, before compinit: fpath=(~/.zsh/completions $fpath))

# fish — auto-loaded from the completions dir:
cp completions/affi.fish ~/.config/fish/completions/affi.fish
```

Each file carries its own install header and notes that it is authored from the
documented surface.

---

## Branch & PR conventions

- **Never commit to the default branch.** Branch off it for every change.
- **Branch names:** short, kebab-case, scope-prefixed where it helps —
  e.g. `feat/receipt-introspect`, `fix/chain-continuity-gap`,
  `docs/contributing`, `dx/devcontainer`.
- **Commits:** imperative mood, present tense ("add", "fix", "document"), one
  logical change per commit. Reference an issue when there is one.
- **Before you open a PR**, run what applies to your change:
  - touched Rust → `cargo fmt --all -- --check`, and (in the sibling
    workspace) `cargo test` + `scripts/golden.sh`;
  - touched web → `scripts/check.sh` (or `cd web && npx tsc --noEmit`).
- **PR description:** state what changed, why, and — per the admission
  criterion — **which test goes red if the change is reverted**. A PR that adds
  a capability without a failing-on-removal witness is incomplete.
- Keep PRs focused; large mechanical changes (formatting, renames) go in their
  own commit/PR so review stays meaningful.

---

## Where to look

| Area | File / dir |
| --- | --- |
| CLI entry (parsing + dispatch) | `src/bin/affi.rs`, `src/cli.rs` |
| Receipt assembly (rolling BLAKE3 chain, seal) | `src/chain.rs` |
| 7-stage certify pipeline | `src/verifier.rs` |
| Sealed admission (Layer 2) | `src/admission.rs` |
| Shared types (Receipt, OperationEvent, Verdict, …) | `src/types.rs` |
| OCEL event/object model | `src/ocel.rs` |
| End-to-end smoke | `examples/golden_run.sh` |
| Compile-fail witnesses | `tests/ui/` |

Welcome aboard — and remember: no hollow stamps.
