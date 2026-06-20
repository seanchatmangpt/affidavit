# affidavit — DX / QoL / Doctor Innovation: Synthesis & Roadmap

**Status:** design / proposal · **Branch:** `claude/jolly-turing-t488iq` · **Date:** 2026-06-20

This directory is the output of a 5-agent fan-out chartered to "innovate DX / QoL /
doctor 1000x" for the `affi` CLI. Each agent grounded itself in the real codebase
(the `#[verb("verb","noun")]` → `crate::handlers::*` seam, the feature-flag matrix,
the existing verbs) and produced one design doc. This file is the entry point: it
indexes the five, consolidates the **bugs they found along the way**, reconciles the
places where their designs overlap or conflict, and sequences the work.

> **Doctrine guardrail (applies to everything below):** *certify, don't decide.* No
> proposal here lets a tool judge whether work is honest. Doctor/fix/watch/cache
> **surface, suggest, and re-run the existing verifier verbatim** — they never mint a
> verdict or launder a bad chain into ACCEPT. Each doc carries its own doctrine check.

> **Verification caveat:** the external deps are private-registry `26.6` crates
> (`clap-noun-verb`, `clnrm-core`, `wasm4pm`, `lsp-max`, …) that do not resolve in a
> lone checkout, so **none of this was `cargo build`/`test`-verified here.** All Rust
> in these docs is compilable-*style* — correct against the patterns in-tree, pending
> signature finalization when built against the sibling crates.

---

## The five proposals

| # | Doc | Theme | Headline |
|---|-----|-------|----------|
| 01 | [`01-doctor-command.md`](01-doctor-command.md) | **Doctor — environment** | `affi doctor` for install/config/feature health; `DoctorCheck` trait + `linkme` registry; `--fix`/`--json`/`--check`. |
| 02 | [`02-doctor-receipts.md`](02-doctor-receipts.md) | **Doctor — receipts** | Store-wide chain health scan + `affi fix` with only structurally-safe (Finalize/Quarantine) ops; deterministic 0–100 health score. |
| 03 | [`03-dx-cli-ergonomics.md`](03-dx-cli-ergonomics.md) | **DX — ergonomics** | `src/diag.rs` + `src/output.rs`: stable error codes, exit-code catalog, `--explain`, uniform `--format/--json`, `affi why`. |
| 04 | [`04-qol-workflow.md`](04-qol-workflow.md) | **QoL — workflow** | `affi init`/`watch`/`config`/`run` + supercharged git hooks; drives the existing `FileWatcher`; content-addressed verdict cache. |
| 05 | [`05-dx-onboarding.md`](05-dx-onboarding.md) | **DX — onboarding** | `guide` noun (tutorial/examples/search/man) over one `registry.rs` source-of-truth; groups the ~67 verbs; "did you mean". |

---

## Part A — Bugs found while grounding (fix these regardless of any new feature)

The most immediately valuable output of the fan-out is not the new features — it's
that grounding five agents in the source surfaced concrete, cited defects. These are
worth fixing independent of whether any proposal ships.

| # | Sev | Defect | Location | Found by |
|---|-----|--------|----------|----------|
| B1 | **High** | **Output stream split:** `emit` writes results to **stdout**, but `verify`/`show`/`inspect`/`stats` write to **stderr** — so `affi receipt show r.json > out.txt` captures nothing. Breaks every redirect/pipe. | `handlers.rs:127` (stdout) vs `handlers.rs:356-365` (stderr) | 03 |
| B2 | **High** | **JSON via `format!` interpolation** — output is hand-built with `format!`, so any value containing a quote/backslash produces invalid (and injectable) JSON. | `handlers.rs:157,303,324` | 03 |
| B3 | **High** | **Tampered receipts silently dropped:** `load_receipts_from_path` uses `if let Ok(r)`, swallowing the deserialize-time chain-hash check, so `verify_family`/`query` skip tampered receipts instead of reporting them. | `handlers.rs:84` | 02 |
| B4 | **High** | **`GENESIS_SEED` version drift:** seed pinned to `affidavit-v26.6.14-genesis` while the package is `26.6.17` → "verifies on my box, not my teammate's." | `chain.rs:22` | 01 |
| B5 | Med | **`monitor` is a stub** (`"tokio-based watch loop not yet implemented"`) even though a working `FileWatcher` already exists. | `handlers.rs:2632` vs `quality.rs:1158` | 04 |
| B6 | Med | **Exit codes scattered/collapsed:** ad-hoc `std::process::exit(2)`; `verify_sla` failure collapses to a generic `1` with no catalog. | `handlers.rs:352,366,562` | 03 |
| B7 | Low | **Stray duplicate module file:** `receipt-throughput.rs` *and* `receipt_throughput.rs`; the hyphenated name can't be a Rust module → dead file. | `src/verbs/` | 05 |
| B8 | Low | **Stale shell completions:** committed `completions/*` cover ~4 of 59+ verbs and omit PowerShell entirely. | `completions/` | 03 |
| B9 | Low | **Docs drift:** README claims "59 capabilities"; `verbs/mod.rs` declares ~67 modules. The true verb count is itself unclear — a discoverability problem. | `README.md`, `verbs/mod.rs` | 03, 05 |
| B10 | Info | **`linkme` declared but unused** in `src/` (`Cargo.toml:42`) — `doctor`'s check registry would be its first idiomatic use. | `Cargo.toml:42` | 01 |

**Recommendation:** land B1–B4 as a small, self-contained "correctness" PR first. B1/B2
are also *prerequisites* for the output contract in 03, so fixing them early pays twice.

---

## Part B — Cross-cutting keystones (shared infra several proposals need)

Three pieces of infrastructure recur across the docs. Building them once, first,
unblocks the rest and prevents three half-versions of the same thing.

1. **The output/diagnostics contract — `src/output.rs` + `src/diag.rs` (03).**
   A single `Out` handle (`--format human|json|yaml`, `--json`, `--quiet/--verbose`,
   `NO_COLOR/--color`, strict data→stdout / chatter→stderr) and a stable error/exit-code
   system. *Depended on by:* doctor's `--json` (01, 02), watch output (04), tutorial
   narration (05). Subsumes bug fixes B1, B2, B6.

2. **The verb registry — `src/registry.rs` (05).**
   One source of truth per verb (group, summary, keywords, examples). *Depended on by:*
   help/search/completion/man (05), `affi --explain` and completion generation (03),
   and `doctor`'s "which features does this verb need" check (01). Eliminates the
   drift behind B8/B9 and the REPL's hand-maintained 11-of-67 dispatch list.

3. **The `DoctorCheck` framework (01) shared with the receipt scanner (02).**
   A `Finding { check, status, finding, remediation, auto_fixable }` type (mirroring the
   existing `CheckOutcome` in `types.rs:270`) and a `linkme` registry of checks, reused
   by both the environment doctor and the receipt-store doctor.

---

## Part C — Reconciled cross-doc decisions

The agents worked independently, so a few designs must be unified before implementation:

- **`doctor` noun conflict.** 01 filed `#[verb("doctor","env")]` (about your machine);
  02 filed `#[verb("doctor","receipt")]` (about your data). **Decision:** ship a single
  `affi doctor` that runs environment checks by default and adds receipt-store health
  when a path/`--receipts` is supplied (or when run inside a `.affi/` store). Both modes
  share the one `DoctorCheck`/`Finding` framework from Part B.3. This keeps one obvious
  command instead of two nouns users must discover.

- **`affi fix` ownership.** Defined in 02 (Finalize/Quarantine on receipts). 01's
  `--fix` covers environment (completions, stale `working.json` archive). **Decision:**
  `affi doctor --fix` applies only its own safe environment fixes; receipt repairs live
  in the separate `affi fix` verb (02) so destructive-looking data ops are never a flag
  on a diagnostic command.

- **Output/JSON.** 03 owns the contract; 01/02/04/05 **consume** it rather than each
  rolling their own `--json`. No verb should hand-format JSON after B2 is fixed.

- **`watch` vs `monitor`.** 04's `affi watch` should drive the real `FileWatcher`
  (`quality.rs:1158`); the stub `monitor` (B5) is either retired or made an alias.

- **Verb count.** Establish the true number (resolve B7/B9) before 05's grouping and
  03's completion generation, since both enumerate the verb set.

---

## Part D — Unified rollout

Dependency-ordered. Sizes: **S** ≤ ½ day · **M** ~1–2 days · **L** ~3–5 days (estimates,
unverified against a real build).

### P0 — Foundations & correctness
| Work | Source | Size |
|------|--------|------|
| Correctness PR: fix B1 (streams), B2 (JSON), B3 (silent tamper drop), B4 (genesis seed) | A | M |
| `src/output.rs` + `src/diag.rs` contract (error codes, exit-code catalog, `Out`) | 03 | M |
| `src/registry.rs` source-of-truth; resolve verb-count (B7/B9) | 05 | M |
| `DoctorCheck`/`Finding` framework + `affi doctor` env MVP (genesis, working-dir, features, config, completions) | 01 | M |

### P1 — High-leverage features
| Work | Source | Size |
|------|--------|------|
| `affi doctor` receipt mode + `affi fix` (Finalize/Quarantine, `--dry-run/--apply`) | 02 | L |
| `affi --explain <CODE>` + `affi why <RECEIPT>` | 03 | M |
| `affi init` + layered `affi config` (flag>env>project>user>default, `config explain`) | 04 | M |
| Generated completions (bash/zsh/fish/pwsh) from the registry | 03/05 | S |
| `affi guide tutorial` + `affi guide examples` + verb grouping + "did you mean" | 05 | L |

### P2 — Depth & polish
| Work | Source | Size |
|------|--------|------|
| `affi watch` daemon (drive `FileWatcher`, debounce); retire `monitor` stub | 04 | M |
| Supercharged git-hook suite (pre-commit verify, commit-msg stamp, CI `doctor`) | 04 | M |
| REPL upgrade (registry-driven completion, working-chain `Session`, full dispatch) + optional TUI | 05 | L |
| Content-addressed verdict cache; advanced `fix` ops | 04/02 | M |
| Generated man pages from the registry | 05 | S |

---

## Caveats & provenance

- Nothing here was compiled or tested (private `26.6` deps; see top). Treat Rust as
  near-implementation-ready sketches, not merged code.
- No existing source file was modified by this initiative — only `docs/innovation/*`.
- Line/symbol citations come from the agents reading the tree at this commit; they are
  traceable via each doc's "current state" section.

## File index

- `00-SYNTHESIS.md` — this file (roadmap, bug ledger, reconciliation)
- `01-doctor-command.md` — environment/install/config doctor (470 lines)
- `02-doctor-receipts.md` — receipt-store health & safe repair (665 lines)
- `03-dx-cli-ergonomics.md` — errors, exit codes, output contract, `why` (686 lines)
- `04-qol-workflow.md` — init/watch/config/hooks automation (818 lines)
- `05-dx-onboarding.md` — tutorial/examples/discoverability/REPL (629 lines)
