# Affidavit documentation hub

`affidavit` is **the provenance layer**. It assembles and certifies *provenance
receipts*: append-only, content-addressed BLAKE3 chains of operation-events that
record what a process did. The `affi` binary lets you emit events, finalize them
into an immutable receipt, and certify that receipt against a fixed format
standard.

The lifecycle is four verbs:

```text
affi receipt emit     append an operation-event to .affi/working.json
affi receipt assemble finalize + seal + content-address into a receipt
affi receipt verify   run the 7-stage certify pipeline -> ACCEPT / REJECT
affi receipt show     human-readable dump of the chain (no verdict)
```

The doctrine in one line: **certify, don't decide** — the verifier checks a
witness against a decidable format standard; it never judges honesty, and the
forging bypass is unconstructable (a private `_seal` field makes it a compile
error, `E0451`).

**New here? Read these three first, in order:**

1. [architecture.md](architecture.md) — concise overview with diagrams of the
   lifecycle, the 7-stage pipeline, and the module map.
2. [glossary.md](glossary.md) — precise definitions of every key term.
3. [`../README.md`](../README.md) — the project README (install, CLI, golden run).

---

## Documentation index

The repository root holds the full set of design and integration docs. They are
grouped below by purpose. (These files live at the repo root; this hub only links
to them.)

### In this `docs/` hub

| Doc | What it is |
| --- | --- |
| [architecture.md](architecture.md) | Architecture overview: lifecycle, 7-stage pipeline, and module-map diagrams (Mermaid). |
| [glossary.md](glossary.md) | Definitions: receipt, operation-event, OCEL object_ref, commitment, rolling chain hash, content address, genesis seed, profile, verdict, the seal/E0451, determinism, certify-don't-decide. |

### Start here

| Doc | What it is |
| --- | --- |
| [`../README.md`](../README.md) | Project overview: doctrine, install/build, CLI surface, the golden run, determinism, and source layout. |
| [`../STATUS.md`](../STATUS.md) | Current status report: phase completion, test counts, per-library integration status (wired vs. planned), and known residuals. |
| [`../CHANGELOG.md`](../CHANGELOG.md) | Notable changes per version. |
| [`../ACCOMPLISHMENTS_v26614.md`](../ACCOMPLISHMENTS_v26614.md) | Narrative of what was accomplished in the v26.6.17 session. |

### Architecture & doctrine

| Doc | What it is |
| --- | --- |
| [`../ARDPRD.md`](../ARDPRD.md) | The combined Product & Architecture Requirements document — functional/non-functional requirements, ADRs, and the certify-don't-decide doctrine. The authoritative design reference. |
| [`../REPRESENTATION_MAP.md`](../REPRESENTATION_MAP.md) | The affidavit ↔ Next.js UI representation map: rule that the UI renders only real capabilities, with the gap tracked to zero. |
| [`../REFACTOR_EVIDENCE_DESIGN.md`](../REFACTOR_EVIDENCE_DESIGN.md) | Design for wrapping `Receipt` in the `Evidence<Receipt, Admitted, AffidavitReceiptChain>` typestate (the Layer 2/3 admission seam). |

### DX / QOL roadmap

| Doc | What it is |
| --- | --- |
| [`../DX_QOL_INDEX.md`](../DX_QOL_INDEX.md) | Master index for the DX/QOL "1000x" feature expansion (22 features across 6 library areas). |
| [`../DX_QOL_1000X_DESIGN.md`](../DX_QOL_1000X_DESIGN.md) | The full 80/20 design for the DX/QOL feature expansion. |
| [`../DX_QOL_IMPLEMENTATION_CHECKLIST.md`](../DX_QOL_IMPLEMENTATION_CHECKLIST.md) | Implementation roadmap: per-feature checklist and dependency graph. |
| [`../DX_QOL_CODE_TEMPLATES.md`](../DX_QOL_CODE_TEMPLATES.md) | Copy-paste-ready code templates for Phase 1 DX/QOL features. |
| [`../FEATURES_DX_QOL.md`](../FEATURES_DX_QOL.md) | Feature catalogue: the 80/20 DX/QOL improvements drawn from ecosystem libraries. |
| [`../DX_QOL_EXECUTIVE_SUMMARY.txt`](../DX_QOL_EXECUTIVE_SUMMARY.txt) | One-page executive summary of the DX/QOL expansion. |
| [`../DX_REPORT.md`](../DX_REPORT.md) | Generated DX report (current working-tree snapshot). |

### Integrations

**Overview**

| Doc | What it is |
| --- | --- |
| [`../INTEGRATIONS.md`](../INTEGRATIONS.md) | How Affidavit integrates with ecosystem libraries and how to use the integrated features (the top-level integration reference). |

**lsp-max** (IDE / Language Server Protocol support for receipts)

| Doc | What it is |
| --- | --- |
| [`../LSP_MAX_INTEGRATION_INDEX.md`](../LSP_MAX_INTEGRATION_INDEX.md) | Index to the lsp-max integration document set. |
| [`../LSP_MAX_INTEGRATION_PLAN.md`](../LSP_MAX_INTEGRATION_PLAN.md) | The lsp-max integration plan (scope, phases, 80/20 reuse). |
| [`../LSP_MAX_INTEGRATION_ARCHITECTURE.md`](../LSP_MAX_INTEGRATION_ARCHITECTURE.md) | Technical architecture: module structure, traits, error handling. |
| [`../LSP_MAX_INTEGRATION_CODE_TEMPLATES.md`](../LSP_MAX_INTEGRATION_CODE_TEMPLATES.md) | Code skeletons and patterns for implementation. |
| [`../LSP_MAX_INTEGRATION_QUICK_REFERENCE.md`](../LSP_MAX_INTEGRATION_QUICK_REFERENCE.md) | One-page quick-reference card. |
| [`../LSP_MAX_INTEGRATION_SUMMARY.md`](../LSP_MAX_INTEGRATION_SUMMARY.md) | Executive summary of the lsp-max integration. |

**wasm4pm** (process mining: discovery / conformance)

| Doc | What it is |
| --- | --- |
| [`../WASM4PM_INDEX.md`](../WASM4PM_INDEX.md) | Index to the wasm4pm integration planning set. |
| [`../WASM4PM_INTEGRATION_PLAN.md`](../WASM4PM_INTEGRATION_PLAN.md) | The wasm4pm integration plan. |
| [`../WASM4PM_INTEGRATION_SUMMARY.md`](../WASM4PM_INTEGRATION_SUMMARY.md) | Executive summary of the wasm4pm integration. |
| [`../WASM4PM_80_20_BREAKDOWN.md`](../WASM4PM_80_20_BREAKDOWN.md) | The 80/20 split: what wasm4pm provides off-the-shelf vs. the glue Affidavit must write. |
| [`../WASM4PM_QUICK_REFERENCE.md`](../WASM4PM_QUICK_REFERENCE.md) | One-page quick-reference card. |
| [`../WASM4PM_WITNESS_TEST_TEMPLATES.md`](../WASM4PM_WITNESS_TEST_TEMPLATES.md) | Chicago-TDD witness-test templates for proving the integration. |

**clnrm** (determinism harness / templates / mutations via clnrm-core)

| Doc | What it is |
| --- | --- |
| [`../CLNRM_INTEGRATION_README.md`](../CLNRM_INTEGRATION_README.md) | Quick-reference entry point for the clnrm integration. |
| [`../CLNRM_INTEGRATION_PLAN_26.6.17.md`](../CLNRM_INTEGRATION_PLAN_26.6.17.md) | The clnrm-core integration plan (library-only, 80/20). |
| [`../CLNRM_INTEGRATION_EXAMPLES.md`](../CLNRM_INTEGRATION_EXAMPLES.md) | Copy-paste code snippets for integrating clnrm-core. |
| [`../CLNRM_PUBLIC_API_SURFACE.md`](../CLNRM_PUBLIC_API_SURFACE.md) | The clnrm-core public API surface relevant to Affidavit (templates, validators, mutations). |

### Release & publishing

| Doc | What it is |
| --- | --- |
| [`../RELEASE.md`](../RELEASE.md) | Release notes for v26.6.17. |
| [`../PUBLISHING.md`](../PUBLISHING.md) | How to publish Affidavit to crates.io (checklist and metadata). |

### Coverage & provenance logs

| Doc | What it is |
| --- | --- |
| [`../DOC_COVERAGE_LOG.md`](../DOC_COVERAGE_LOG.md) | Doc ↔ example coverage log: per-example exit-code witnesses and the gap map. |
| [`../reference/COVERAGE.md`](../reference/COVERAGE.md) | The reference-implementation coverage gap-grid (which types are exercised by a worked construction). |

---

## Other useful entry points

- [`../examples/`](../examples/) — runnable examples, including
  [`../examples/golden_run.sh`](../examples/golden_run.sh) (full lifecycle:
  ACCEPT, then a tamper that flips to REJECT).
- [`../src/`](../src/) — the implementation (`types.rs`, `ocel.rs`, `chain.rs`,
  `verifier.rs`, `cli.rs` + `verbs/`, entry at `bin/affi.rs`).
- [`../tests/`](../tests/) — the witness suite (dispatch, adversarial, e2e,
  compile-fail, integration witnesses).
- [`../web/`](../web/) — the Next.js UI.
