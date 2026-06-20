# W9 — Ecosystem & Standards Interop

**Workstream:** W9 (of 10) · **Owner role:** Ecosystem & Standards Interop
**Program horizon:** 2026 H2 → 2030 · **Status:** design / roadmap
**Doctrine:** *certify, don't decide.* Ingest and export **translate formats and record events**; they never mint a verdict. An imported OCEL log or SBOM becomes events to **certify**, not an answer to "is this honest."

> **Build caveat.** Private `26.6` registry crates (`clap-noun-verb`, `clnrm-core`,
> `wasm4pm`, `lsp-max`) do not resolve in a lone checkout, so nothing here was
> `cargo build`/`test`-verified. All Rust below is compilable-*style*: correct against
> the patterns in-tree, pending signature finalization against the sibling crates.

---

## 1. Mission, scope & boundaries

### Mission
Make `affidavit` a **first-class interoperability hub** for standard provenance and
supply-chain formats. Everything outside the receipt chain should be able to **flow in**
(become certifiable events) and **flow out** (be consumed by external tools) through one
coherent, extensible, deterministic ingestion/export framework, fully observable via
OpenTelemetry and Prometheus.

### In scope (W9 owns)
1. **OCEL** (Object-Centric Event Logs) — import *and* export, round-tripping receipts
   ↔ OCEL 2.0 (JSON/XML) with measured fidelity.
2. **OpenTelemetry / metrics** — mature the dormant `otel`/`metrics` surface: real spans
   for the verify pipeline, metric instruments wired into the verifier, exporters
   (OTLP/Jaeger/Prometheus), and **trace→receipt correlation**.
3. **SBOM ingestion** — CycloneDX / SPDX (and SWID) feeding receipts, building on the
   already-strong canonical model.
4. **The `emit_from_*` source-adapter family** — CI/CD, cloud, GitHub/GitLab,
   monitoring, security — refactored into one `SourceAdapter` trait + `linkme` plugin
   registry, an extensible *ingestion framework* rather than seven copy-pasted handlers.

### Out of scope (explicit boundaries — reference only in §5)
- **Compliance reports / attestations** (SOC2/HIPAA/PCI/GDPR) from ingested evidence → **W10**.
  W9 *delivers the events*; W10 *decides conformance language*. (`sbom_compliance.rs`'s
  framework scoring is a gray zone; W9 keeps the *ingest + NTIA presence-check*, hands
  framework verdicts to W10.)
- **Cryptographic signing** of exports / signed OTLP / detached signatures → **W8**.
- **The verification engine itself** (stages, profiles, parallelization) → **W7**.
  W9 *instruments* the pipeline (spans/metrics) and *feeds* it (events); it never changes
  what a stage decides.

---

## 2. Current state (grounded) + gaps

### 2.1 What exists today — strengths

**Base OCEL model — `src/ocel.rs`.** A clean, deterministic event-construction core:
`SeqCounter` (monotonic, time-free, `ocel.rs:19-46`), `parse_object_ref` for the
`id:type[:qualifier]` CLI grammar (`ocel.rs:95-108`), `build_event` that commits to
`BLAKE3(payload)` and derives `evt-{seq}` ids (`ocel.rs:118-135`), and `validate_event`
(`ocel.rs:141-157`). This is the substrate every adapter already funnels through.

**SBOM ingestion — `src/sbom.rs` (1173 lines).** Genuinely mature and the keystone of
the supply-chain layer:
- One canonical `Sbom` model normalizing SPDX 2.3/3.0, CycloneDX 1.5/1.6, SWID
  (`sbom.rs:40-51`), 12 `ComponentType`s (`sbom.rs:77-142`), PURL/CPE/hash/license/supplier.
- `detect_format` (`sbom.rs:528-549`), `parse_spdx` (`sbom.rs:552-692`),
  `parse_cyclonedx` (`sbom.rs:695-798`), auto-dispatch `parse_sbom_json` (`sbom.rs:907-917`).
- `content_address` — canonicalize-then-BLAKE3, **format-independent & order-stable**
  (`sbom.rs:457-480`), proven by `content_address_is_format_independent_and_deterministic`
  (`sbom.rs:1114-1128`).
- `ntia_minimum_elements` — certifies the 7 NTIA fields *as presence*, explicitly
  "does not decide whether values are truthful" (`sbom.rs:428-453`) — doctrine-clean.
- Dependency-free ISO-8601→Unix and `days_from_civil` keep it lean (`sbom.rs:924-958`).

**SBOM→OCEL projection — `src/sbom_ocel.rs` (795 lines).** Projects a canonical SBOM
into a deterministic, ordered OCEL stream: fixed 7-type taxonomy (`sbom_ocel.rs:72-99`),
`sbom_to_ocel_events` (import → component-catalogued → dependency-resolved,
`sbom_ocel.rs:217-309`), causal chains (`build_sbom_causal_chain`, `sbom_ocel.rs:359-412`),
license correlation (`sbom_ocel.rs:437-476`). Determinism is tested across runs
(`commitments_are_deterministic_across_runs`, `sbom_ocel.rs:660-675`). A sibling
`quality_ocel.rs` mirrors the style for quality measures.

**`emit_from_*` family — `src/verbs/emit_from_*.rs` + `src/handlers.rs:165-275`.** Seven
source adapters (github/gitlab/cicd/cloud/monitoring/security + sbom) exposed as thin
`#[verb(...)]` wrappers (e.g. `emit_from_github.rs:13-16`) delegating to handlers. The
SBOM handler (`sbom_emit`, `handlers.rs:3629-3667`) is real: load → project → append.
The `sbom_*` verb family (`sbom_ntia`, `sbom_compliance`, `sbom_scan`,
`handlers.rs:3669-3760+`) is wired to `sbom.rs` / `sbom_compliance.rs` / `sbom_vulnerability.rs`.

**Observability primitives — `src/tracing.rs`, `src/metrics.rs`.**
- `tracing.rs` is an honest, witnessed span seam: `SpanRecord` (`tracing.rs:23-29`),
  thread-local sink + optional `AFFI_TRACE_SINK` file (`tracing.rs:37-59`), and
  `trace_emit/assemble/verify/show` wrappers (`tracing.rs:74-109`). Its own docs are
  scrupulous about scope: "we do not claim Jaeger export is witnessed — only that
  operations emit observable spans" (`tracing.rs:9-17`).
- `metrics.rs` has a `MetricsCollector` (rolling-window SLIs, `metrics.rs:79-226`),
  SLO checks (`check_slo`, `metrics.rs:176-207`), and a `PrometheusExporter`
  (`metrics.rs:230-284`).
- `Cargo.toml` declares the full OTel stack behind features: `otel` (opentelemetry +
  jaeger + tracing-subscriber, `Cargo.toml:143`) and `metrics` (+ prometheus +
  opentelemetry-prometheus, `Cargo.toml:145`). `1000x_otel_hyper_spec.rs` catalogs 100+
  semantic-convention attribute constants.

### 2.2 Gaps — what's missing or dormant

| # | Gap | Evidence | Impact |
|---|-----|----------|--------|
| **G1** | **No OCEL *export*.** Only `sbom_to_ocel_events` (SBOM→events) exists; there is **no receipt→OCEL** and **no OCEL-JSON→receipt** import. No round-trip. `grep` for `from_ocel`/`OcelLog`/`ocel.*export` finds nothing in `src/`. | `src/sbom_ocel.rs:217` is the only projection; no reverse. | A receipt cannot be handed to a process-mining tool (ProM/PM4Py/Celonis); external OCEL logs cannot be certified. |
| **G2** | **Observability is dormant — not wired into the verifier.** `verifier.rs` has **zero** references to `trace_verify`, `MetricsCollector`, `record_receipt_verified`, or stage latency. | `grep MetricsCollector\|record_receipt_verified\|trace_verify src/verifier.rs` → *No matches*. | Spans/metrics exist but never observe the real 7-stage pipeline. `metrics.rs`/`1000x_otel_hyper_spec.rs` are **not even declared in `lib.rs`** (orphaned). The whole stack is a parallel universe. |
| **G3** | **`emit_from_*` build JSON by `format!` interpolation** — a repo/event name containing `"` or `\` yields **invalid, injectable** JSON. | `handlers.rs:166,184,202-203,222-223,242-243,263-264`. (Matches synthesis bug **B2**, `docs/innovation/00-SYNTHESIS.md:46`.) | Untrusted webhook/CI fields corrupt the chain payload silently. |
| **G4** | **Adapters are seven copy-pasted handlers, no shared abstraction, no plugin model.** No `SourceAdapter` trait; `linkme` is declared (`Cargo.toml:42`) but unused in `src/` for handlers (matches synthesis **B10**). Adding a source = hand-copying a handler + verb + mod line. | `handlers.rs:165-275` are near-identical; `linkme::distributed_slice` appears only in `quality_object_level.rs`. | No third-party adapter extensibility; high drift risk; the "ingestion framework" is aspirational. |
| **G5** | **Adapters fabricate payloads, don't ingest real evidence.** Each builds a 3-field synthetic JSON (`{"source":...,"repo":...}`) instead of consuming the actual webhook/audit-log body and committing to *its* bytes. | `handlers.rs:166`, `:184`, `:202`. | The receipt commits to a stub, not to the real GitHub delivery / CloudTrail record — weak provenance. |
| **G6** | **Prometheus export uses wall-clock; metrics not reproducible; no OTLP push.** `PrometheusExporter::render` stamps `SystemTime::now()` (`metrics.rs:248-251`), and there is no exporter to an OTLP collector / pushgateway — only a `String`. | `metrics.rs:248`. | Violates the determinism ethos for snapshot artifacts; can't ship to a real telemetry backend. |
| **G7** | **No trace→receipt correlation.** Spans carry only `operation` + `target` (`tracing.rs:24-29`); no `receipt.content_address` / `chain_hash` / `trace_id` linkage, and no way to look up "the receipt this trace certified." | `tracing.rs:23-29`. | Can't pivot from a Jaeger trace to the receipt or vice versa. |
| **G8** | **SWID ingest is a stub; SPDX 3.0 graph + CycloneDX VEX/services under-parsed; no YAML/XML/tag-value inputs.** | `sbom.rs:913-915` ("SWID ingest not yet implemented"); `parse_spdx` handles 2.3 JSON only. | Format coverage claimed in module docs (`sbom.rs:13`) exceeds what parses. |
| **G9** | **No format-detection / capability discovery surface.** No `affi import --detect`, no registry listing "which formats can I ingest/export." | — | Users can't discover the interop matrix. |

**Framing:** the *model* layer (sbom.rs, ocel.rs, sbom_ocel.rs) is strong and
doctrine-clean; the *plumbing* layer (adapter framework, OCEL round-trip, live
observability wiring, exporters) is where W9 spends its multi-year budget.

---

## 3. Phased plan (2026 H2 → 2030)

Design rules for every phase: **(a)** ingest/export only *translate + record* — never
decide; **(b)** every transform is deterministic (time-free, canonical ordering, BLAKE3
commitments) and round-trip-tested; **(c)** new code is **additive** (new modules/verbs),
never edits to W7's verifier semantics.

---

### Phase 2026 H2 — Foundations: adapter framework + correctness

**Objective.** Replace the seven copy-pasted handlers with one extensible, *safe*
`SourceAdapter` trait + `linkme` registry; fix the JSON-injection defect (G3); ingest
real payload bytes (G5). Land the smallest honest slice of OCEL export.

**Deliverables**
- `src/ingest/mod.rs` — the `SourceAdapter` trait, `IngestEnvelope`, `IngestError`, and a
  `linkme`-backed registry (first idiomatic use of `linkme` for adapters, closing B10/G4).
- Refactor `emit_from_{github,gitlab,cicd,cloud,monitoring,security}` to register adapters;
  keep verb signatures byte-identical (no CLI break). Replace `format!` JSON with
  `serde_json::json!` (closes G3/B2).
- `--payload <file|->` on each `emit_from_*` so the **real** webhook/audit body is ingested
  and committed (closes G5). Absent payload → today's synthetic stub (back-compat).
- `src/ocel_export.rs` — `receipt_to_ocel(&Receipt) -> OcelLog` producing **OCEL 2.0 JSON**;
  `affi receipt export-ocel <RECEIPT> [--format ocel-json]`.

```rust
// src/ingest/mod.rs — the ingestion framework keystone.
use crate::types::ObjectRef;

/// One normalized unit of evidence an adapter produces from a source payload.
/// It is *data to certify*, never a verdict.
pub struct IngestEnvelope {
    pub event_type: String,          // e.g. "github.push", "cicd.circleci.failed"
    pub objects: Vec<ObjectRef>,     // typed object refs (id:type[:qualifier])
    pub payload: Vec<u8>,            // the bytes the receipt commits to (BLAKE3)
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("source payload malformed: {0}")] Malformed(String),
    #[error("unknown source: {0}")] UnknownSource(String),
}

/// A pluggable source adapter. Translates a raw source payload into one or more
/// certifiable envelopes. DOCTRINE: it decides *shape*, never *honesty*.
pub trait SourceAdapter: Sync {
    /// Stable source key, e.g. "github", "gitlab", "cloud".
    fn source(&self) -> &'static str;
    /// Event-type prefixes this adapter can emit (for discovery / `affi import --list`).
    fn event_types(&self) -> &'static [&'static str];
    /// Translate a raw payload into ordered envelopes. Deterministic & time-free.
    fn ingest(&self, raw: &[u8], hint: Option<&str>) -> Result<Vec<IngestEnvelope>, IngestError>;
}

/// linkme registry: external crates contribute adapters with one attribute.
#[linkme::distributed_slice]
pub static SOURCE_ADAPTERS: [&'static dyn SourceAdapter] = [..];

/// Resolve an adapter by source key (linear scan over a tiny slice).
pub fn adapter_for(source: &str) -> Option<&'static dyn SourceAdapter> {
    SOURCE_ADAPTERS.iter().copied().find(|a| a.source() == source)
}
```

```rust
// src/ingest/github.rs — a concrete adapter, registered via linkme.
use super::{IngestEnvelope, IngestError, SourceAdapter, SOURCE_ADAPTERS};
use crate::ocel::parse_object_ref;

pub struct GitHubAdapter;

impl SourceAdapter for GitHubAdapter {
    fn source(&self) -> &'static str { "github" }
    fn event_types(&self) -> &'static [&'static str] {
        &["github.push", "github.pull_request", "github.release", "github.workflow_run"]
    }
    fn ingest(&self, raw: &[u8], hint: Option<&str>) -> Result<Vec<IngestEnvelope>, IngestError> {
        // Parse the REAL delivery body; fall back to a hint-only synthetic shape.
        let doc: serde_json::Value = serde_json::from_slice(raw)
            .map_err(|e| IngestError::Malformed(e.to_string()))?;
        let repo = doc.pointer("/repository/full_name").and_then(|v| v.as_str())
            .ok_or_else(|| IngestError::Malformed("missing repository.full_name".into()))?;
        let kind = hint.unwrap_or("push");
        let objects = vec![parse_object_ref(&format!("{repo}:repo"))
            .map_err(|e| IngestError::Malformed(e.to_string()))?];
        // Commit to the *real* bytes (canonicalized), not a fabricated stub.
        let payload = serde_json::to_vec(&doc).unwrap_or_default();
        Ok(vec![IngestEnvelope { event_type: format!("github.{kind}"), objects, payload }])
    }
}

#[linkme::distributed_slice(SOURCE_ADAPTERS)]
static GITHUB: &dyn SourceAdapter = &GitHubAdapter;
```

**Acceptance**
- All six verbs route through `adapter_for(..).ingest(..)`; `emit_from_*` CLI surface
  unchanged (UI snapshot tests pass).
- Fuzz: a repo name `a"b\c` produces **valid** JSON committed bytes (G3 regression test).
- `receipt_to_ocel(r)` emits OCEL 2.0 JSON validating against the published schema; an
  empty receipt → empty-but-valid log.
- `cargo test ingest_ -- --test-threads=1` deterministic across runs.

**Cross-WS deps:** consumes W1 module-layout conventions; output contract (stdout/stderr,
`--json`) per W3. No W7 verifier change.

---

### Phase 2027 — OCEL round-trip + live verifier observability

**Objective.** Close the OCEL loop (import + export, fidelity-measured) and **wire the
dormant observability stack into the real verify pipeline** (G2), making `tracing.rs`/
`metrics.rs` first-class and feature-gated.

**Deliverables**
- `src/ocel_import.rs` — `ocel_to_events(&OcelLog) -> Vec<OperationEvent>`; verb
  `affi receipt import-ocel <FILE>` ingesting an **external** OCEL 2.0 JSON log as events
  to certify (the inverse of G1). External logs become a receipt, **not** a verdict.
- `src/ocel_export.rs` matured to full OCEL 2.0 (objects map, events map, O2O/E2O
  relations, attribute typing) + an `OcelRoundTrip` fidelity report.
- **`src/verify_observe.rs`** — a thin, opt-in observation layer wrapping the W7 pipeline:
  opens a `verify` span, a per-stage child span, and records stage latency + verdict into a
  `MetricsCollector`. Gated by `otel`/`metrics`; a no-op when the features are off so W7's
  pure pipeline is untouched (closes G2). Declare `metrics`/`1000x_otel_hyper_spec` in
  `lib.rs` (close the orphan).
- Trace→receipt correlation: spans carry `receipt.content_address` + `chain_hash`
  attributes (closes G7).

```rust
// src/ocel_export.rs — receipt -> OCEL 2.0, the export half of the round-trip.
use crate::types::{OperationEvent, Receipt};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub struct OcelLog {
    #[serde(rename = "ocel:version")] pub version: String,         // "2.0"
    #[serde(rename = "ocel:events")] pub events: BTreeMap<String, OcelEvent>,
    #[serde(rename = "ocel:objects")] pub objects: BTreeMap<String, OcelObject>,
}
#[derive(Serialize)] pub struct OcelEvent {
    #[serde(rename = "ocel:type")] pub activity: String,
    #[serde(rename = "ocel:omap")] pub object_ids: Vec<String>,    // E2O relations
    #[serde(rename = "affi:commitment")] pub commitment: String,   // provenance bridge
}
#[derive(Serialize)] pub struct OcelObject {
    #[serde(rename = "ocel:type")] pub object_type: String,
}

/// Project a sealed receipt into an OCEL 2.0 log. Deterministic: events keyed by
/// `evt-{seq}`, objects in BTreeMap (sorted) order. No wall clock.
pub fn receipt_to_ocel(receipt: &Receipt) -> OcelLog {
    let mut events = BTreeMap::new();
    let mut objects = BTreeMap::new();
    for ev in receipt.events() {
        for o in &ev.objects {
            objects.entry(o.id.clone())
                .or_insert(OcelObject { object_type: o.obj_type.clone() });
        }
        events.insert(ev.id.clone(), OcelEvent {
            activity: ev.event_type.clone(),
            object_ids: ev.objects.iter().map(|o| o.id.clone()).collect(),
            commitment: ev.payload_commitment.as_hex().to_string(),
        });
    }
    OcelLog { version: "2.0".into(), events, objects }
}

/// Round-trip fidelity: receipt -> OCEL -> events. Reports what survives.
#[derive(Serialize)] pub struct OcelRoundTrip {
    pub events_in: usize,
    pub events_out: usize,
    pub commitments_preserved: bool,   // every commitment survived the trip
    pub object_types_preserved: bool,
}
```

```rust
// src/verify_observe.rs — opt-in instrumentation of W7's pipeline (G2/G7).
#[cfg(feature = "metrics")]
pub fn verify_observed(
    receipt: &crate::types::Receipt,
    collector: &crate::metrics::MetricsCollector,
) -> crate::types::Verdict {
    use crate::tracing::trace_verify;
    use std::time::Instant;
    let addr = receipt.content_address();          // span attribute (G7)
    trace_verify(addr.as_hex(), || {
        let start = Instant::now();
        // W7 owns `certify`; we only WRAP it — we never alter what a stage decides.
        let verdict = crate::verifier::certify(receipt);
        collector.record_stage_latency("pipeline", start.elapsed());
        collector.record_receipt_verified(verdict.accepted);
        for stage in verdict.outcomes() {
            if !stage.passed { collector.record_stage_error(stage.name()); }
        }
        verdict
    })
}
```

**Acceptance**
- **Round-trip fidelity:** `receipt → receipt_to_ocel → ocel_to_events` preserves event
  count, ids, types, and **every `payload_commitment`** (`commitments_preserved == true`);
  a golden receipt exported and re-imported re-certifies to the **same verdict**.
- Importing a PM4Py-generated OCEL 2.0 sample yields a receipt that `affi verify` ACCEPTs.
- With `--features metrics`, `affi verify` increments `affidavit_receipts_verified_total`
  and populates p99 latency; a witness test asserts a `verify` span carries the
  receipt's content address.
- W7 pipeline output byte-identical with/without the observe layer (purity guard).

**Cross-WS deps:** **W7** exposes a stable `certify(&Receipt) -> Verdict` + per-stage
outcomes for wrapping (no behavior change). **W3** output contract for the round-trip
report. **W8** *may* later sign the OCEL export — W9 leaves a `affi:signature` slot but
does not sign.

---

### Phase 2028 — Exporters, format breadth, discovery

**Objective.** Ship telemetry to real backends (OTLP/Jaeger/Prometheus, deterministic
snapshots), broaden SBOM/format coverage (G8), and expose the interop matrix (G9).

**Deliverables**
- `src/observe/export.rs` — OTLP span exporter (Jaeger via `opentelemetry-jaeger`) and a
  Prometheus path via `opentelemetry-prometheus`; a `affi observe export --backend
  {otlp,prometheus,jaeger}` verb. Deterministic snapshot mode: `--at <unix>` injects a
  fixed timestamp so artifacts are reproducible (closes G6).
- `src/observe/pushgateway.rs` — optional push of the `PrometheusExporter` text to a
  pushgateway (reuses the `webhook`/`reqwest` stack, `Cargo.toml:84`).
- SBOM breadth: real **SWID** ingest (closes the `sbom.rs:913` stub), **SPDX 3.0** element
  graph, CycloneDX **VEX** + `services`/`compositions`; **tag-value SPDX** + **XML**
  CycloneDX readers feeding the same canonical `Sbom`.
- `affi import list` / `affi export list` — enumerate the registry: every `SourceAdapter`,
  every SBOM format, OCEL in/out — the discoverable interop matrix (closes G9). Sources its
  truth from the W4 registry so it never drifts.

```rust
// src/observe/export.rs — span layout + deterministic export (G6).
/// The canonical span tree W9 emits for one verify. Names are stable contract.
/// affi.verify                       (root; attrs: receipt.content_address, chain_hash)
/// └─ affi.verify.stage.decode
/// └─ affi.verify.stage.check_format
/// └─ affi.verify.stage.chain_integrity   (attr: chain.recomputed_hash)
/// └─ affi.verify.stage.continuity
/// └─ affi.verify.stage.verify_commitments (attr: commitments.checked)
/// └─ affi.verify.stage.evaluate_profile   (attr: profile.id)
/// └─ affi.verify.emit_verdict             (attr: verdict.accepted)
#[cfg(feature = "otel")]
pub fn export_spans(spans: &[crate::tracing::SpanRecord], backend: Backend, at: Option<u64>)
    -> anyhow::Result<usize>
{
    let clock = at.unwrap_or_else(deterministic_or_now);   // --at => reproducible
    match backend {
        Backend::Otlp | Backend::Jaeger => push_otlp(spans, clock),
        Backend::Prometheus => Ok(0), // metrics path handled by exporter, not spans
    }
}
pub enum Backend { Otlp, Jaeger, Prometheus }
```

**Acceptance**
- A `verify` run exports the 9-node span tree to a local Jaeger; an integration test reads
  it back and asserts the root span's `receipt.content_address` equals the receipt's.
- `affi observe export --backend prometheus --at 1700000000` is **byte-identical** across
  two runs (G6 determinism).
- A SWID tag and an XML CycloneDX both ingest to a canonical `Sbom` with the **same
  `content_address`** as their JSON equivalents (cross-format identity).
- `affi import list` enumerates ≥ 6 adapters + 3 SBOM families + OCEL bidirectional.

**Cross-WS deps:** **W4** registry as the source of truth for `import/export list`. **W2**
doctor may add an "OTel exporter reachable?" check that calls W9's `export --dry-run`.
**W8** signs exported OCEL/SBOM if requested (slot only).

---

### Phase 2029 — Streaming, conformance-grade interop, bidirectional sync

**Objective.** Scale ingestion to streams/large logs, and make the interop *certifiable
to a published standard* (OCEL 2.0 conformance, NTIA/CISA min-elements gating) — handing
verdict *language* to W10 but proving *presence* here.

**Deliverables**
- `src/ingest/stream.rs` — incremental ingestion: a long-running `affi import watch
  --source github --webhook :8443` that turns a live webhook stream into appended events
  (drives W6/W5 watch/automation infra), back-pressured and ordered by `SeqCounter`.
- Streaming SBOM/OCEL via `serde_json::StreamDeserializer` for logs that don't fit memory
  (the README "streaming receipts" roadmap item, realized for *ingest*).
- `src/ocel_conformance.rs` — certify an OCEL log (in or out) against the **OCEL 2.0
  schema** as a *presence/structure* check (mirrors `Sbom::ntia_minimum_elements`'s
  certify-don't-decide stance). Emits a structured report; **does not** assert the log is
  honest.
- Bidirectional connectors: Celonis/PM4Py/ProM export profiles; CycloneDX/SPDX **export**
  from a receipt (receipt → SBOM) so an affidavit chain can re-emit a signed-by-W8 SBOM.

```rust
// src/ocel_conformance.rs — certify OCEL structure, never honesty (doctrine).
#[derive(serde::Serialize)]
pub struct OcelConformance {
    pub version_ok: bool,            // ocel:version == "2.0"
    pub events_well_typed: bool,     // every event has an activity + omap
    pub objects_resolved: bool,      // every omap id exists in ocel:objects
    pub commitments_present: bool,   // affi:commitment on every event (our bridge)
}
impl OcelConformance {
    pub fn is_conformant(&self) -> bool {
        self.version_ok && self.events_well_typed
            && self.objects_resolved && self.commitments_present
    }
    /// Names of failing checks (empty iff conformant) — for W10 to phrase a finding.
    pub fn missing(&self) -> Vec<&'static str> { /* ... mirrors NtiaMinimumElements ... */ vec![] }
}
```

**Acceptance**
- `affi import watch` ingests a replayed stream of 10k GitHub deliveries into a single
  chain with contiguous `seq` (no gaps), verified by `affi verify`.
- A 1 GB OCEL log imports under bounded memory (streaming), proven by a peak-RSS bench.
- `OcelConformance::is_conformant()` agrees with an external OCEL 2.0 validator on a
  conformance corpus (true/false parity), while emitting **no** honesty verdict.
- receipt → CycloneDX export re-ingests (`parse_sbom_json`) to a byte-identical canonical
  `Sbom` (export/ingest are inverses for the supported subset).

**Cross-WS deps:** **W5/W6** watch + interactive infra for streaming ingest. **W7** for
verifying streamed chains. **W8** signs receipt→SBOM/OCEL exports. **W10** consumes
`OcelConformance`/NTIA reports to phrase compliance statements (W9 supplies booleans).

---

### Phase 2030 — Universal interop hub, hardening, self-certifying telemetry

**Objective.** Lock the interop matrix as a **stable, versioned public contract**; make
the framework self-describing and self-certifying; reach a maintainable steady state.

**Deliverables**
- `src/ingest/spec.rs` — a versioned, machine-readable **interop manifest** (`affi import
  spec --json`): every adapter, every format, every direction, each with a fidelity class
  (lossless / lossy-documented) and the standard version it targets. Becomes the contract
  third parties build adapters against.
- Adapter **certification harness**: a golden corpus per source/format; CI gate fails any
  adapter whose round-trip fidelity regresses. Third-party `linkme` adapters run the same
  harness.
- **Self-certifying telemetry:** an `affi observe` run can itself emit a receipt of *what
  it exported* (a span/metric snapshot becomes events to certify) — telemetry that is
  itself provenance, closing the loop with the project's doctrine.
- OCEL/SBOM **version-tracking**: when OCEL 3.0 or SPDX 4 / CycloneDX 2.0 land, the
  manifest + canonical models absorb them additively without breaking older importers
  (the model layer's format-independence already supports this — `sbom.rs:457-480`).

```rust
// src/ingest/spec.rs — the 2030 public interop contract.
#[derive(serde::Serialize)]
pub struct InteropManifest {
    pub schema_version: String,                 // affi interop manifest version
    pub sources: Vec<SourceSpec>,               // every SourceAdapter
    pub sbom_formats: Vec<FormatSpec>,          // spdx-2.3, cyclonedx-1.6, swid, ...
    pub ocel: DirectionSpec,                    // { import: true, export: true, version: "2.0" }
}
#[derive(serde::Serialize)]
pub struct SourceSpec {
    pub source: &'static str,
    pub event_types: &'static [&'static str],
    pub fidelity: Fidelity,                      // Lossless | Lossy(&'static str reason)
}
pub fn manifest() -> InteropManifest { /* fold over SOURCE_ADAPTERS + format tables */ todo!() }
```

**Acceptance**
- `affi import spec --json` validates against its own published JSON schema and lists the
  complete matrix; the doc in this file is generated *from* it (no drift).
- Every shipped adapter passes the certification harness at its declared fidelity class;
  a deliberately-broken fixture fails CI.
- A telemetry export receipt `affi verify`s ACCEPT — telemetry-as-provenance works.
- Importing a (mocked) OCEL-3.0 / CycloneDX-2.0 sample is handled additively: older
  importers unaffected, new fields surfaced, **no** semantic change to the verifier.

---

## 4. Definition of done @ 2030

W9 is "done" when `affidavit` is a **universal, certifiable provenance translator**:

1. **OCEL round-trips losslessly.** receipt ↔ OCEL 2.0 (JSON/XML) with
   `commitments_preserved` and a re-import that re-certifies to the *same verdict*. External
   process-mining logs (PM4Py/ProM/Celonis) import as certifiable events; receipts export to
   those tools. *(closes G1)*
2. **Every standard event source flows in through one framework.** `emit_from_*` is a
   `linkme`-pluggable `SourceAdapter` registry; CI/CD, cloud, GitHub/GitLab, monitoring,
   security ingest **real** payload bytes via safe `serde_json` (no `format!` injection);
   third parties add adapters without forking. *(closes G3, G4, G5)*
3. **The verify pipeline is fully observable.** W7's stages emit a stable 9-node span tree
   and feed live metrics/SLIs; spans correlate to receipts by content address; exporters
   ship to OTLP/Jaeger/Prometheus with deterministic snapshot mode. The once-orphaned
   `metrics.rs`/`1000x_otel_hyper_spec.rs` are wired and tested. *(closes G2, G6, G7)*
4. **SBOM coverage matches its claims.** SPDX 2.3/3.0 (JSON + tag-value), CycloneDX
   1.5/1.6 (JSON + XML + VEX), SWID all ingest to one canonical `Sbom` with format-stable
   content addresses; receipts export back to CycloneDX/SPDX. *(closes G8)*
5. **The interop matrix is a discoverable, versioned, certified contract.** `affi import
   spec --json` + a CI fidelity-certification harness; the matrix is self-describing and
   absorbs future format versions additively. *(closes G9)*
6. **Doctrine held throughout.** No ingest/export path ever mints a verdict. OCEL/SBOM/NTIA
   conformance checks certify **presence/structure** and hand verdict *language* to W10;
   signing is deferred to W8; the verifier's decisions remain W7's. Every transform is
   deterministic, time-free, and round-trip-tested.

---

## 5. Cross-workstream dependencies

**W9 depends on:**
- **W1 (Foundations):** module-layout conventions for the new `ingest/`, `observe/`,
  `ocel_*` modules; lib.rs wiring (incl. de-orphaning `metrics`/`1000x_otel_hyper_spec`).
- **W3 (CLI Ergonomics):** the stdout/stderr + `--json`/`--format` output contract for all
  new verbs (round-trip reports, `import/export list`, `observe export`). Fixing
  synthesis bug **B2** (JSON via `format!`) is shared ground — W9's adapter refactor
  eliminates G3 at the source.
- **W4 (Onboarding/Registry):** the verb registry as the single source of truth backing
  `affi import list` / `export list` and the 2030 manifest (no drift).
- **W5/W6 (Workflow/Interactive):** watch + daemon infrastructure that streaming ingest
  (`affi import watch`) drives in 2029.
- **W7 (Verification Engine):** a stable `certify(&Receipt) -> Verdict` with enumerable
  per-stage outcomes to *wrap* with spans/metrics — **without** changing what a stage
  decides. Streamed/round-tripped chains are verified by W7 unchanged.

**W9 provides to:**
- **W10 (Compliance & Governance):** ingested SBOM/OCEL events + structured
  presence/conformance reports (`NtiaMinimumElements`, `OcelConformance`) as the **evidence
  substrate** for SOC2/HIPAA/PCI/GDPR attestations. W9 supplies booleans and events; W10
  phrases the compliance verdict. *(boundary: W9 never writes an attestation.)*
- **W8 (Crypto & Trust):** export artifacts (OCEL logs, receipt→SBOM) with a reserved
  `affi:signature` slot for W8 to sign; W9 produces the canonical bytes, W8 signs them.
  *(boundary: W9 never signs.)*
- **W2 (Doctor):** an optional "is the configured OTel/Prometheus exporter reachable?"
  check that calls W9's `observe export --dry-run`.

**Shared-but-owned-elsewhere (no W9 edits):** synthesis bugs **B1** (stream split) /
**B6** (exit codes) are W3's; W9 inherits the fixed contract. The genesis-seed drift
**B4** is W1's. W9's only overlap-fix is **B2/G3**, resolved as a side effect of the
adapter refactor.

---

*Grounded against the tree at this commit. Citations are `file:line` at read time and are
traceable. No existing source file was modified by this roadmap; only this doc was added.*
