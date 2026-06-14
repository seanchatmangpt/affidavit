# Representation Map — affidavit ↔ Next.js UI

**Rule:** the UI renders from the project's ACTUAL data and capabilities, never from
fixtures dressed as real. A component showing a number/type/receipt/benchmark the
project does not actually produce is the closed-door attack in frontend form. This
map is the deliverable and the metric; drive the gap to zero both directions:
- **rendered-but-fabricated** — UI data with no real source (FORBIDDEN; must be 0).
- **exposed-but-unrepresented** — a real capability the UI never surfaces.

The UI app lives in `web/` (Next.js App Router). All data derives from the
authoritative sources below, captured BY TOOL (run the binary / read the file),
never hand-typed into the UI.

## Exposed surface (what the project actually produces — captured by tool)

| # | Exposed capability | Authoritative source (by tool) | Real shape |
|---|---|---|---|
| E1 | Capability manifest (11 verbs) | `affi --introspect` → JSON array | `[{name, description, parameters: JSONSchema}]` |
| E2 | CLI surface + global options | `affi --help` | verbs `receipt {emit,assemble,verify,show,inspect,graph,replay,model,conformance,diagnose,stats}`; opts `--format {json,json-pretty,yaml,table,plain,tsv,quiet} --select --introspect --structured-errors --autonomic` |
| E3 | Receipt (the chain) | `affi receipt show --format json` / sealed receipt JSON | `Receipt{format_version, events[OperationEvent{id,seq,event_type,objects[ObjectRef{id,obj_type,qualifier?}],payload_commitment}], chain_hash}` |
| E4 | Verdict (certify) | `affi receipt verify --format json` | `Verdict{accepted, profile, outcomes[CheckOutcome{stage,passed,detail}], reason}` |
| E5 | Discovered model | `affi receipt model` / `graph` | process-tree string; DFG `(nodes,edges,starts,ends)` |
| E6 | Conformance metrics | `affi receipt conformance` | `(fitness, activity_coverage, simplicity)` — honest labels |
| E7 | Editor diagnostics | `affi receipt diagnose` | lsp-max `Diagnostic[]` (severity/source/message) |
| E8 | ggen provenance receipts | `.ggen/receipts/*.json` | `{operation_id, timestamp, input_hashes[], output_hashes[], signature, previous_receipt_hash}` |
| E9 | Reference coverage gap-grid | `reference/COVERAGE.md` | per-type witness table + §7 ghost/compiler-blocked findings |
| E10 | Doc↔example coverage | `DOC_COVERAGE_LOG.md` + `examples/*.rs` | gap map; 9 running examples |
| E11 | OTel semconv registry | `semconv/registry/affidavit.yaml` + `weaver registry check` | span attrs `operation`, `target` |
| E12 | Status | `STATUS.md` | capability ledger (CLOSED / OPEN-substrate) |
| E13 | Version | `affi --version` | crate version string |

## Rendered surface (what the UI currently renders)

The `web/` Next.js app (App Router, RSC, streaming Suspense, server action, Node
route handler) builds (`next build` ✓, 5 routes) and was witnessed serving **real**
data at runtime (`npm start` + curl):

| Exposed | UI route | How (real source) | Witnessed |
|---|---|---|---|
| E1 capability manifest | `/capabilities`, `/api/introspect` | RSC + route handler shell `affi --introspect` | ✓ 11 real verbs in SSR HTML + JSON |
| E2 CLI verb surface | `/capabilities` | same manifest (names + required params) | ✓ |
| E3 Receipt + E4 Verdict | `/pipeline` | **server action** runs real `affi` emit→assemble→verify in a temp dir | ✓ build; renders genuine Verdict |
| E8 ggen receipt (summary) | `/` dashboard tile | reads `.ggen/receipts/latest.json` | ✓ |
| E9 coverage gap-grid | `/coverage` | reads `reference/COVERAGE.md` verbatim + mines 🟢 count | ✓ real content in HTML |
| E10 doc↔example coverage | `/coverage` | reads `DOC_COVERAGE_LOG.md` | ✓ |
| E12 status | `/coverage` | reads `STATUS.md` | ✓ |
| E13 version | `/` dashboard tile | `affi --version` | ✓ |

## Gap (iteration 1 — UI scaffolded)

- **rendered-but-fabricated:** 0 — every value flows from the data layer
  (`web/lib/affidavit.ts`), which only shells `affi` or reads repo files; missing
  sources render an explicit "unavailable"/"binary not built", never a fixture.
- **exposed-but-unrepresented:** 5 — **E5** discovered model/DFG (`affi receipt
  model`/`graph`), **E6** conformance metrics (`affi receipt conformance`), **E7**
  editor diagnostics (`affi receipt diagnose`), **E8** full receipt browser (only the
  summary tile so far), **E11** OTel semconv registry + `weaver registry check`.

**GAP metric = 5 exposed-but-unrepresented (was 13).** Target 0 — next iterations add
the model/DFG, conformance, diagnostics, receipt-browser, and weaver-registry routes,
each wired to the real `affi` output / real files.

## Faithfulness invariants (carried from the reference genre)

- Every number/type/receipt shown is fetched from E1–E13 at request time (RSC /
  route handler / server action), never inlined as a literal.
- Capabilities the project marks OPEN-substrate (e.g. OTel SDK collector export)
  must be shown AS open, not as working — the UI does not over-claim.
- Honest findings (ghost variants, compiler-blocked types) are represented as
  findings, not hidden.
