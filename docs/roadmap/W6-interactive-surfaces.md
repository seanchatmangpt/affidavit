# W6 — Interactive Surfaces

**Workstream:** W6 (Interactive Surfaces) · **Owner role:** REPL · TUI · LSP/IDE
**Binary:** `affi` / `affi-shell` (+ a planned `affidavit-lsp`) · **Baseline:** 26.6.17
**Horizon:** 2026 H2 → 2030
**Status:** roadmap / design proposal — no source modified

> **Doctrine guardrail — *certify, don't decide*.** Every surface in W6 is a *window*,
> never a *judge*. The REPL, the TUI, and the LSP **display** the verdict that
> `affidavit::verifier::verify` already produced and **re-run that verifier verbatim**;
> they never mint a verdict, never recolour a REJECT into an ACCEPT, never "fix" a chain
> into passing. A red stage in the TUI and an `Error` diagnostic in the editor are the
> *same* refusal the CLI prints — just navigable. This is enforced by construction: W6
> code calls `verifier::verify` / `cli::verify` and renders the result; it owns no
> adjudication logic.

> **Build caveat.** The external deps are private-registry `26.6` crates
> (`clap-noun-verb`, `lsp-max`, `clnrm-core`, `wasm4pm`, …) that do not resolve in a lone
> checkout, so **nothing here was `cargo build`/`test`-verified.** All Rust is
> *compilable-style*: correct against the in-tree types and seams, pending signature
> finalization against the sibling crates.

---

## 1. Mission & Scope

**Mission.** Make a human's interaction with a provenance chain *immediate and legible*.
Three surfaces, one job — turn 68 opaque verbs and a hash-soup JSON file into something
you can *drive*, *browse*, and *see inside your editor*:

1. **REPL** (`affi-shell`) — a stateful provenance console. Registry-driven completion
   across all 68 verbs, a `Session` that holds the working chain / current receipt so the
   prompt reads `affi[receipt.json]>` and bare `verify` Just Works, and full handler
   dispatch (today only ~11 of 68 verbs are wired).
2. **TUI dashboard** (ratatui-style, behind `ui`) — a read-only browser over a receipt or
   a `.affi/` store: event list ▸ event detail ▸ chain status ▸ per-stage verdict, with
   drill-down and an in-pane `v`erify that repaints the verdict bar.
3. **LSP / IDE** (`lsp-max`) — provenance *inside the editor*: hover a receipt `event_id`
   → summary card; diagnostics → verification failures pinned to the offending line;
   code actions → suggested fixes (surfaced from W2). Grows into editor-native provenance
   (inlay verdicts, CodeLens "verify", workspace store view) by 2030.

### In scope (W6 owns)
- `src/bin/affi-shell.rs` upgrade: `Session`, registry-driven completion, full dispatch,
  "did you mean", inline `help`/`examples`.
- A new TUI surface (e.g. `src/tui/` + `affi receipt tui` wrapper, behind `ui`/`ratatui`).
- The `src/lsp/` module (hover, diagnostics, goto-definition) and a future `affidavit-lsp`
  server binary (the `LanguageServer` impl described in the integration architecture doc).
- The *interaction model*: keybindings, panes, prompt grammar, hover/CodeLens UX.

### Out of scope (consumed, not owned)
| Concern | Owner | W6 relationship |
|---|---|---|
| Verb **registry** data (`registry.rs`: group/summary/keywords/examples) | **W4** | **Consume.** Completion, REPL `help`, TUI legend, LSP completion all read `registry::all()`. W6 does not author registry entries. |
| Tutorial / `examples` / `man` / "did you mean" data + `suggest.rs` | **W4** | **Consume.** REPL calls `suggest::did_you_mean`; LSP completion reuses the same source. |
| Output / format primitives (`Out`, `--format/--json`, stream split, color) | **W1 / W3** | **Consume.** TUI/REPL render through the contract; W6 owns no JSON hand-formatting. |
| The verifier & its stages | **W7** | **Display only.** W6 calls `verify`; W7 owns stage logic. |
| Doctor / fix logic (`DoctorCheck`/`Finding`) | **W2** | **Surface only.** TUI store-health pane and LSP code actions *render* W2 findings; W6 never authors a fix. |
| Crypto/trust (signatures, attest) | **W8** | **Display.** TUI/LSP show trust badges from W8 verdict fields. |

**One-line boundary:** *W6 builds the glass; W4 fills the catalog behind it, W7/W2/W8 decide
what the glass shows. W6 never decides.*

---

## 2. Current State (cited) & Gap

### 2.1 REPL — `src/bin/affi-shell.rs`
Good bones, badly under-wired. It is a `rustyline` editor with filename completion, a
`MatchingBracketHighlighter`, `HistoryHinter`, multi-line via trailing `\`, and persisted
`.affi_history` (`affi-shell.rs:109-190`). But:

- **Completion is a hardcoded 16-word vec** — `receipt, emit, assemble, verify, show,
  inspect, stats, graph, replay, model, conformance, diagnose, help, exit, clear, history`
  (`affi-shell.rs:121-138`). It does not include `diff`, `query`, `timeline`, `attest`,
  `sbom-*`, `dora-metrics`, or ~50 others, and it is hand-maintained → already drifted.
- **Dispatch covers ~11 of 68 verbs**, all under a literal `"receipt"` match arm with
  bespoke arg-parsing per verb (`affi-shell.rs:225-358`). Anything else hits the catch-all
  `_ => Err(anyhow!("Unknown command: {}", args[0]))` (`affi-shell.rs:359`) — no suggestion.
- **No chain context.** The prompt is a constant `"affi> "` (`affi-shell.rs:153`); every
  `verify`/`show` requires a full path. There is no notion of a "current" receipt or of the
  events emitted this session.

Each verb is dispatched by `tokio::task::spawn_blocking(move || handlers::verify(...))`
(`affi-shell.rs:278-282`) — that `spawn_blocking`-the-handler seam is exactly right and is
reused unchanged below.

### 2.2 TUI — does not exist
No TUI surface, no `tui` verb, **no `ratatui` dependency** (Cargo.toml has only
`ui = ["colored", "indicatif"]`, `Cargo.toml:153`). The richest browse today is
`handlers::show` / `inspect` dumping text. Innovation doc 05 sketches the target three-pane
dashboard (`docs/innovation/05-dx-onboarding.md:559-580`) — W6 productionizes it.

### 2.3 LSP — real foundation, three primitives, no server loop yet
This is further along than the REPL. The `lsp` feature gates `pub mod lsp` (`lib.rs:77-78`,
`lsp = ["lsp-max", "tokio"]`, `Cargo.toml:133`), and `src/lsp/` already ships three working
resolvers over `lsp_max::lsp_types_max`:

| Primitive | Function | Location | Returns |
|---|---|---|---|
| Hover card | `hover_for_event_id(event_id, &Receipt)` | `src/lsp/hover.rs:11` | `Option<Hover>` (markdown table: type/seq/commitment/objects) |
| Diagnostics | `verdict_to_diagnostics(&Verdict)` | `src/lsp/diagnostics.rs:10` | `Vec<Diagnostic>`, one `ERROR` per failed `CheckOutcome`, `source="affidavit"` |
| Goto-def | `goto_definition_for_event_type(event_type)` | `src/lsp/goto_definition.rs:32` | `Option<Location>` → handler source file |

A reference integration test already drives a **real** reject verdict through
`verdict_to_diagnostics` and asserts a continuity `Error` surfaces
(`tests/reference_lsp_real_reject.rs:21-63`) — proof the verdict→editor path is honest, not
a hand-built literal. The integration architecture doc specs the missing piece: an
`affidavit-lsp` binary implementing `lsp-max`'s `LanguageServer` trait (`initialize`,
`did_open/change/close`, `hover`, `goto_definition`, `completion`, publish-diagnostics) over
stdio/TCP (`docs/integrations/LSP_MAX_INTEGRATION_ARCHITECTURE.md:25-108`).

**Gaps:** (a) the three resolvers are *libraries* with no server to call them — no
`did_change` → re-verify → publishDiagnostics loop, no document state, no transport;
(b) `goto_definition` line numbers are all `0` and its handler map is a hand-maintained
12-entry `BTreeMap` (`goto_definition.rs:15-26`) that should read the W4 registry;
(c) no code actions, no completion, no inlay/CodeLens; (d) diagnostics pin every failure to
`line 0, char 0..10` (`diagnostics.rs:18-27`) — not the real JSON offset of the bad event.

### 2.4 Consolidated gap
| Surface | Today | Target (2030) |
|---|---|---|
| REPL completion | 16 hardcoded words (`affi-shell.rs:121-138`) | all 68 verbs from `registry::all()`, zero drift |
| REPL dispatch | ~11 verbs, one `receipt` arm (`affi-shell.rs:225-358`) | every verb via one table; bare `verify` uses session |
| REPL context | constant `"affi> "` (`:153`) | `Session` → `affi[receipt.json]>`, working-chain aware |
| TUI | none (no `ratatui`, `Cargo.toml:153`) | navigable events/chain/verdict dashboard + store view |
| LSP server | resolvers only, no loop | full `affidavit-lsp` (`LanguageServer`) + code actions + CodeLens/inlay |
| LSP precision | failures at line 0 (`diagnostics.rs:18-27`) | precise JSON-range diagnostics on the offending event |

---

## 3. Phased Plan (2026 H2 → 2030)

Anchor: the unified rollout puts **"REPL upgrade (registry-driven completion, working-chain
`Session`, full dispatch) + optional TUI"** in **P2** (`docs/innovation/00-SYNTHESIS.md:140`).
W6's 2026 H2 *is* that P2 line — it can only begin once P0's registry (`registry.rs`) and
output contract land (synthesis P0/P1, `00-SYNTHESIS.md:124-133`), which W4/W1/W3 own.

---

### Phase 1 — 2026 H2 · "Drive the chain" (REPL to parity + LSP server skeleton)

**Objective.** Turn `affi-shell` from a 16-word stub into a registry-driven, stateful
console that dispatches all 68 verbs; stand up the `affidavit-lsp` server loop so the three
existing resolvers actually serve an editor. *Anchored in synthesis P2 — the REPL line.*

**Deliverables**
- D1.1 `Session` type (current receipt + events emitted this session) and a context-aware
  prompt (`affi> ` → `affi(2 evt)> ` → `affi[receipt.json]>`).
- D1.2 Registry-driven completion: replace the hardcoded vec (`affi-shell.rs:121-138`) with
  `registry::all()` (W4). Completion now covers 68 verbs and cannot drift.
- D1.3 One dispatch table over `registry::all()` replacing the 11-arm `receipt` match
  (`affi-shell.rs:225-358`); `spawn_blocking` each handler exactly as today.
- D1.4 Bare-verb context injection: `verify`/`show`/`inspect`/`stats` with no path target
  the `Session` receipt.
- D1.5 "did you mean" on unknown (call W4 `suggest::did_you_mean` in the catch-all that is
  today `Unknown command`, `affi-shell.rs:359`); inline `help <verb>` / `examples <verb>`
  from the registry.
- D1.6 `affidavit-lsp` binary: a `ReceiptLsp` implementing `lsp-max`'s `LanguageServer`
  (`initialize`, `did_open`, `did_change`, `did_close`, `hover`, `goto_definition`) over
  stdio, wiring the existing `hover_for_event_id` / `verdict_to_diagnostics` /
  `goto_definition_for_event_type`. `did_change` re-runs `verifier::verify` and publishes
  diagnostics.

**REPL `Session` + dispatch sketch** (compilable-style; reuses the `spawn_blocking` seam at
`affi-shell.rs:261-282`):

```rust
// src/bin/affi-shell.rs — Session context + registry-driven dispatch (sketch)
use std::path::PathBuf;

struct Session {
    receipt: Option<PathBuf>, // "current" assembled receipt (set by `use`/`assemble`)
    emitted: usize,           // events appended to the working chain this session
}

impl Session {
    fn prompt(&self) -> String {
        match (self.emitted, &self.receipt) {
            (0, None)    => "affi> ".into(),
            (n, None)    => format!("affi({n} evt)> "),
            (_, Some(p)) => format!("affi[{}]> ", p.file_name().unwrap().to_string_lossy()),
        }
    }
    /// Doctrine: context is a *convenience*, never a verdict. We only fill in a
    /// missing path argument; we never alter what verify decides.
    fn resolve_receipt(&self, explicit: Option<&str>) -> anyhow::Result<String> {
        explicit.map(str::to_string)
            .or_else(|| self.receipt.as_ref().map(|p| p.display().to_string()))
            .ok_or_else(|| anyhow::anyhow!("no receipt: pass a path or `use <receipt>`"))
    }
}

async fn dispatch(line: &str, sess: &mut Session) -> anyhow::Result<()> {
    let args = shlex::split(line).ok_or_else(|| anyhow::anyhow!("bad quoting"))?;
    let Some(cmd) = args.first() else { return Ok(()) };

    match cmd.as_str() {
        "exit" | "quit"    => std::process::exit(0),
        "use"              => { sess.receipt = args.get(1).map(Into::into); return Ok(()); }
        "help"             => { print_help(args.get(1).map(String::as_str)); return Ok(()); }
        "examples"         => { affidavit::handlers::examples(args.get(1).cloned(), None, None)?; return Ok(()); }
        "receipt" | "guide" | "bench" => {}                  // noun → fall through to (noun,verb)
        other => {
            // bare verb: `verify` == `receipt verify <session receipt>`
            if let Some(doc) = affidavit::registry::all().iter().find(|d| d.verb == other) {
                return run_verb(doc, &args[1..], sess).await;
            }
            // unknown → did you mean? (W4 suggester; replaces affi-shell.rs:359)
            match affidavit::suggest::did_you_mean(other) {
                Some(s) => { eprintln!("unknown '{other}'. did you mean:");
                             for x in s { eprintln!("  {}  ({:?})", x.verb, x.group); } }
                None    => eprintln!("unknown '{other}' — try `help` or `guide search <topic>`"),
            }
            return Ok(());
        }
    }
    let (noun, verb) = (cmd.as_str(), args.get(1).map(String::as_str).unwrap_or(""));
    match affidavit::registry::all().iter().find(|d| d.noun == noun && d.verb == verb) {
        Some(doc) => run_verb(doc, &args[2..], sess).await,
        None      => { eprintln!("unknown {noun} subcommand '{verb}'"); Ok(()) }
    }
}

/// Inject the session receipt for path-less verbs, then run the handler off-thread.
async fn run_verb(doc: &affidavit::registry::VerbDoc, rest: &[String], sess: &mut Session)
    -> anyhow::Result<()>
{
    let rest = rest.to_vec();
    match doc.verb {
        "verify" => {
            let r = sess.resolve_receipt(rest.first().map(String::as_str))?;
            tokio::task::spawn_blocking(move || affidavit::handlers::verify(r, None, None, None))
                .await?.map_err(|e| anyhow::anyhow!("{e}"))   // handlers.rs:341 signature
        }
        "assemble" => {
            let out = rest.first().cloned();
            let res = tokio::task::spawn_blocking(move || affidavit::handlers::assemble(None, out.clone()))
                .await?.map_err(|e| anyhow::anyhow!("{e}"));
            if res.is_ok() { sess.receipt = sess.receipt.clone(); } // tracked once assemble reports path
            res
        }
        // ...one arm per verb, generated from the registry's arg-shape metadata...
        _ => { eprintln!("dispatch for '{}' not yet wired", doc.verb); Ok(()) }
    }
}
```

**Acceptance criteria (2026 H2)**
- Completion offers every verb in `registry::all()`; no literal verb list remains in
  `affi-shell.rs` (the `:121-138` vec is deleted).
- Every registry verb is reachable from the REPL; `Unknown command` (`:359`) is replaced by
  a suggester path.
- Prompt reflects session state; bare `verify` after `assemble` verifies the right receipt
  and prints the *unchanged* CLI verdict.
- `affidavit-lsp --stdio` answers `initialize`; on `did_change` of a tampered receipt it
  publishes the same `Error` diagnostic the reference test asserts
  (`tests/reference_lsp_real_reject.rs:54-62`).

**Cross-workstream deps:** W4 (`registry.rs`, `suggest.rs`, `handlers::examples`) — *hard
blocker*; W1/W3 (`Out`/stream-split so REPL output is pipe-safe) — strong; W7 (`verify`)
— display only.

---

### Phase 2 — 2027 · "See inside the chain" (TUI dashboard v1 + LSP precision)

**Objective.** Ship the navigable three-pane TUI over a single receipt; make LSP
diagnostics pin to the *real* offending event range and drive `goto_definition` from the
registry instead of a hand-rolled map.

**Deliverables**
- D2.1 `ratatui` behind `ui` (extend `Cargo.toml:153`); `affi receipt tui <receipt>` thin
  wrapper → `crate::tui::run`.
- D2.2 Three-pane widget tree: event list (left), event detail (right), chain+verdict bar
  (bottom). Keys: `↑/↓` select, `enter` expand objects/commitment, `v` re-verify, `q` quit.
- D2.3 The verdict bar renders `verifier::verify` output verbatim — green ACCEPT / red
  REJECT, failing stage highlighted, mismatch reason inline.
- D2.4 LSP: a JSON-offset index mapping each event to its `(line,char)` span so
  `verdict_to_diagnostics` pins failures to the offending event (replacing the fixed
  `line:0, char:0..10` at `diagnostics.rs:18-27`).
- D2.5 LSP: `goto_definition` reads `registry::all()` for the handler source (retiring the
  12-entry `BTreeMap`, `goto_definition.rs:15-26`); resolve real line numbers (the `0`s).

**TUI widget tree sketch** (compilable-style; field names per `types.rs:186-299`):

```rust
// src/tui/mod.rs — ratatui dashboard over one receipt (read-only) (sketch)
use ratatui::{prelude::*, widgets::*};
use crate::types::{Receipt, Verdict};

struct Dashboard {
    receipt: Receipt,
    verdict: Verdict,     // from verifier::verify — we DISPLAY, never decide
    selected: usize,      // event cursor
    expanded: bool,
}

impl Dashboard {
    fn new(path: &str) -> anyhow::Result<Self> {
        let receipt: Receipt = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        let verdict = crate::verifier::verify(&receipt);   // doctrine: reuse the verifier
        Ok(Self { receipt, verdict, selected: 0, expanded: false })
    }

    /// `v` key: re-run the *real* verifier and repaint. No local adjudication.
    fn reverify(&mut self) { self.verdict = crate::verifier::verify(&self.receipt); }

    fn draw(&self, f: &mut Frame) {
        let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(f.area());
        let cols = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(rows[0]);

        // LEFT: event list
        let items: Vec<ListItem> = self.receipt.events.iter().map(|e| {
            ListItem::new(format!("[{}] {:<10} {}", e.seq, e.event_type,
                e.objects.first().map(|o| o.id.as_str()).unwrap_or("-")))
        }).collect();
        let mut st = ListState::default(); st.select(Some(self.selected));
        f.render_stateful_widget(
            List::new(items).block(Block::bordered().title("EVENTS"))
                .highlight_symbol("▸ "),
            cols[0], &mut st);

        // RIGHT: event detail
        if let Some(ev) = self.receipt.events.get(self.selected) {
            let detail = format!(
                "seq         {}\nevent_id    {}\nevent_type  {}\ncommitment  {}…\nobjects     {}",
                ev.seq, ev.id, ev.event_type,
                &ev.payload_commitment.as_hex()[..12],   // Blake3Hash::as_hex, types.rs:65
                ev.objects.iter().map(|o| format!("{}:{}", o.id, o.obj_type))
                    .collect::<Vec<_>>().join(", "));
            f.render_widget(
                Paragraph::new(detail).block(Block::bordered().title("EVENT DETAIL")),
                cols[1]);
        }

        // BOTTOM: chain + verdict (the only place a verdict appears — read straight from W7)
        let (label, style) = if self.verdict.accepted {
            ("✓ ACCEPT", Style::new().green())
        } else {
            ("✗ REJECT", Style::new().red().bold())
        };
        let passed = self.verdict.outcomes.iter().filter(|o| o.passed).count();
        let bar = Line::from(vec![
            Span::raw(format!(" CHAIN {}…  ", &self.receipt.chain_hash_hex()[..12])),
            Span::styled(format!(" VERDICT {label} "), style),
            Span::raw(format!("  {}/{} stages   v=verify  q=quit", passed, self.verdict.outcomes.len())),
        ]);
        f.render_widget(Paragraph::new(bar).block(Block::bordered()), rows[1]);
    }
}
```

**ASCII TUI mock** (the v1 target — see also `05-dx-onboarding.md:563-580`):

```
┌ affidavit ─ receipt.json ───────────────────────────────── core/v1 · 2 events ┐
│ EVENTS                         │ EVENT DETAIL                                   │
│ ▸ [0] seed         art1▸input  │  seq          0                               │
│   [1] validate     art1▸output │  event_id     evt-0                           │
│                                │  event_type   seed                            │
│                                │  commitment   6ef47c82a1b3…                   │
│                                │  objects      art1:artifact:input             │
│                                │  payload      (not stored — commitment only)  │
├────────────────────────────────┴───────────────────────────────────────────────┤
│ CHAIN 203d3bbf91ac…  │ VERDICT ✓ ACCEPT │ 7/7 stages │  v=verify d=diff g=graph │
│ stages: decode✓ format✓ chain✓ continuity✓ commit✓ profile✓ verdict✓           │
└──────────────────────────────────────────────────────────────────────────────────┘
  ↑/↓ event   enter expand   v verify   d diff   g graph   q quit
```

On a tampered receipt, the verdict bar flips to red `✗ REJECT`, and the failing stage in the
`stages:` line is highlighted red with the mismatch reason — the **same** information
`golden_run.sh` prints, made navigable. The TUI computes nothing; it shows `verdict`.

**Acceptance criteria (2027)**
- `affi receipt tui receipt.json` opens the dashboard; `↑/↓/enter/v/q` behave; a clean
  receipt shows green ACCEPT, a tampered one red REJECT with the failing stage named.
- The TUI displays the verdict from `verifier::verify` only — grep confirms no comparison
  or pass/fail computation inside `src/tui/`.
- LSP diagnostics point at the offending event's JSON line (not line 0); `goto_definition`
  resolves via the registry.

**Cross-workstream deps:** W1/W3 (color/`NO_COLOR` honored in the TUI palette) — strong;
W7 (stage names/order for the `stages:` legend) — hard; W4 (registry for goto-def) — hard;
W2 (optional: surface a doctor hint in the verdict bar) — soft.

---

### Phase 3 — 2028 · "Browse the store + actionable editor" (TUI store view + LSP code actions/CodeLens)

**Objective.** Scale the TUI from one receipt to a whole `.affi/` store; make the editor
*actionable* — code actions that surface W2 fixes and CodeLens "verify" affordances.

**Deliverables**
- D3.1 TUI store mode: `affi store tui [.affi/]` — a receipt list with per-receipt verdict
  badges, drill from store → receipt → event. Health column **renders** W2 `Finding`s.
- D3.2 TUI actions: `d` diff two selected receipts (→ `handlers::diff`, `handlers.rs:769`),
  `g` graph (→ `handlers::graph`, `:863`), `t` timeline (→ `handlers::timeline`, `:1079`).
- D3.3 LSP `code_action`: for a failed stage, offer **W2-authored** remediations as
  `CodeAction`s (e.g. "re-assemble to repair chain hash", "quarantine event"). W6 renders
  the action; W2 owns whether it is safe and what it does.
- D3.4 LSP `completion`: inside a `.receipt.json`, complete `event_type` / object-type /
  qualifier from `registry::all()` and discovered schemas.
- D3.5 CodeLens above each receipt's top-level: `▶ verify (ACCEPT)` / `▶ verify (REJECT)`,
  click re-runs `verify`.

**LSP `LanguageServer` handler sketch** (compilable-style; per the architecture doc's trait
at `LSP_MAX_INTEGRATION_ARCHITECTURE.md:67-103`):

```rust
// src/bin/affidavit-lsp.rs — ReceiptLsp server (sketch)
use lsp_max::{LanguageServer, jsonrpc, Client};
use lsp_max::lsp_types_max::*;
use std::sync::Mutex;
use std::collections::HashMap;

struct ReceiptLsp { client: Client, docs: Mutex<HashMap<Url, String>> }

#[lsp_max::async_trait]
impl LanguageServer for ReceiptLsp {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(true.into()),
                definition_provider: Some(true.into()),
                completion_provider: Some(Default::default()),
                code_action_provider: Some(true.into()),
                text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_change(&self, p: DidChangeTextDocumentParams) {
        let uri = p.text_document.uri.clone();
        let text = p.content_changes.into_iter().next().map(|c| c.text).unwrap_or_default();
        self.docs.lock().unwrap().insert(uri.clone(), text.clone());
        // Re-verify on every keystroke-batch and publish — DISPLAY the real verdict.
        let diags = match serde_json::from_str::<affidavit::types::Receipt>(&text) {
            Ok(r)  => affidavit::lsp::verdict_to_diagnostics(&affidavit::verifier::verify(&r)),
            Err(e) => vec![parse_error_diagnostic(&e)],
        };
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    async fn hover(&self, p: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = &p.text_document_position_params.text_document.uri;
        let receipt = self.parse(uri)?;                        // helper: text → Receipt
        let id = self.event_id_at(uri, p.text_document_position_params.position)?;
        Ok(affidavit::lsp::hover_for_event_id(&id, &receipt))  // existing, hover.rs:11
    }

    async fn goto_definition(&self, p: GotoDefinitionParams)
        -> jsonrpc::Result<Option<GotoDefinitionResponse>>
    {
        let pos = p.text_document_position_params;
        let ev_type = self.event_type_at(&pos.text_document.uri, pos.position)?;
        Ok(affidavit::lsp::goto_definition_for_event_type(&ev_type)   // goto_definition.rs:32
            .map(GotoDefinitionResponse::Scalar))
    }

    async fn code_action(&self, p: CodeActionParams) -> jsonrpc::Result<Option<CodeActionResponse>> {
        // For each affidavit diagnostic, ask W2 for safe remediations and render them.
        let mut actions = Vec::new();
        for diag in p.context.diagnostics.iter()
            .filter(|d| d.source.as_deref() == Some(affidavit::lsp::DIAGNOSTIC_SOURCE))
        {
            for fix in affidavit::doctor::suggest_for_stage(&diag.message) {   // W2 owns this
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: fix.title, kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diag.clone()]), ..Default::default()
                }));
            }
        }
        Ok(Some(actions))
    }
}
```

**Acceptance criteria (2028)**
- Store TUI lists every receipt under `.affi/` with a correct ACCEPT/REJECT badge and a W2
  health column; drill-down to event level works.
- Hovering a tampered receipt in an editor shows the failing stage; a quick-fix appears that
  is *exactly* a W2 action (W6 contributes no fix of its own).
- CodeLens `▶ verify` re-runs and updates its label from the real verdict.

**Cross-workstream deps:** W2 (`doctor::suggest_for_stage` / `Finding` for code actions &
health column) — hard; W4 (completion schema, registry) — hard; W8 (trust badges in the
store list) — soft; W7 (verify) — display.

---

### Phase 4 — 2029 · "Live & multi-receipt" (watch-driven TUI + inlay verdicts + LSP graph navigation)

**Objective.** Make the surfaces *live* and *relational*: TUI auto-refreshes on file change;
editor shows inlay verdicts and lets you navigate the chain/causality graph.

**Deliverables**
- D4.1 TUI live mode: subscribe to W5's `FileWatcher`; when a watched receipt/store changes,
  re-`verify` and repaint (debounced). A "● live" indicator in the title bar.
- D4.2 TUI graph pane: in-TUI DFG/causality view (drive `handlers::graph` / causality verbs),
  navigable with arrow keys; jump from a node to its event detail.
- D4.3 LSP inlay hints: render `// ✓ ACCEPT` / `// ✗ REJECT: <stage>` at the end of each
  receipt's top object — inline, non-modal verdict.
- D4.4 LSP references / "find usages": from an `object.id`, list every event (and receipt in
  the workspace) that references it.
- D4.5 REPL ⇄ TUI handoff: REPL `tui` command opens the dashboard on the `Session` receipt
  and returns to the prompt on `q`.

**Acceptance criteria (2029)**
- Editing a receipt on disk repaints the open TUI with the new verdict within the debounce
  window, no manual `v`.
- Inlay hints show per-receipt verdict in-editor and update on `did_change`.
- `tui` from the REPL opens the current `Session` receipt and round-trips back.

**Cross-workstream deps:** W5 (`FileWatcher` events) — hard; W7 (graph/causality data) — hard;
W4 (object/reference index) — strong; W1/W3 (debounce/refresh cadence vs. output contract) —
soft.

---

### Phase 5 — 2030 · "Editor-native provenance" (unified, embeddable surface)

**Objective.** Provenance is *ambient* in the editor and the terminal: one shared
interaction model across REPL, TUI, and LSP; a workspace-wide provenance tree; trust
visualization; and an embeddable widget other tools can host.

**Deliverables**
- D5.1 LSP workspace view: a tree of all receipts in the workspace with rolled-up verdict
  status, filterable by stage failure / framework / trust — `affi` becomes the provenance
  side-panel of the IDE.
- D5.2 Trust visualization (W8): signer/attestation badges in TUI store list, LSP hover, and
  inlay hints — *who vouched*, rendered, never re-decided.
- D5.3 Shared `render-core`: REPL, TUI, and LSP draw verdicts/events through one rendering
  module so a stage name/symbol is identical across all three surfaces (single source of
  visual truth, mirroring the single registry source of textual truth).
- D5.4 Embeddable TUI widget: expose the dashboard as a library widget (e.g.
  `affidavit::tui::ReceiptWidget`) that downstream tools / W9 partners can host.
- D5.5 Replay/scrubber: step the chain event-by-event in the TUI (drive `handlers::replay`,
  `handlers.rs:897`), watching the rolling chain hash and verdict evolve.

**Acceptance criteria (2030)** — see Definition of Done.

**Cross-workstream deps:** W8 (trust model) — hard; W9 (embeddable widget / standards
surface) — strong; W7 (replay/verify) — display; W4 (registry) — foundational across all.

---

## 4. Definition of Done @ 2030

W6 is "done" when the three surfaces are at parity, drift-free, and provably non-deciding:

1. **REPL.** `affi-shell` completes and dispatches **100% of the verb registry** with zero
   hardcoded verb lists; a `Session` makes the working chain ambient (`affi[receipt.json]>`,
   bare `verify`); unknown input always yields a suggestion. The `:121-138` vec and the
   `:225-358` bespoke `receipt` arm are gone.
2. **TUI.** A single binary path browses one receipt *and* a whole `.affi/` store, live
   (watch-driven), with event/chain/verdict/graph panes, diff, replay-scrubber, and trust
   badges — every verdict and badge read straight from `verifier::verify` / W8.
3. **LSP.** A shipping `affidavit-lsp` implements hover, precise per-event diagnostics,
   registry-driven goto-definition, completion, **W2-sourced** code actions, CodeLens, inlay
   verdicts, references, and a workspace provenance tree — provenance is editor-native.
4. **One model, one truth.** REPL/TUI/LSP share a `render-core`; all three read the **W4**
   registry for text and a shared module for visuals. No surface duplicates verb metadata or
   verdict-rendering logic.
5. **Doctrine, provable.** A grep over `src/tui/`, the REPL dispatch, and `src/lsp/` finds
   **no** adjudication — only calls into `verifier::verify` / `cli::verify` and rendering of
   their results. The reference-test pattern (`tests/reference_lsp_real_reject.rs`) is
   extended to cover TUI and REPL: a real reject must surface, identically, on every surface.

---

## 5. Cross-Workstream Dependencies (summary)

| Dep | Direction | Phase(s) | Criticality | What W6 needs / gives |
|---|---|---|---|---|
| **W4** Onboarding/Registry | W6 **consumes** | 1–5 | **Hard blocker** | `registry::all()` (completion/dispatch/legend/completion-LSP), `suggest::did_you_mean`, `handlers::examples`, schema for LSP completion. W6 authors no registry data. |
| **W7** Verification Engine | W6 **displays** | 1–5 | Hard | `verifier::verify` / `cli::verify`, stage names+order for TUI legend & per-stage diagnostics. W6 never re-implements a stage. |
| **W2** Doctor | W6 **surfaces** | 3–5 | Hard (P3+) | `Finding`/`DoctorCheck` for LSP code actions and the TUI store health column. W2 owns safety; W6 renders. |
| **W1 / W3** Foundations / CLI Ergonomics | W6 **consumes** | 1–2 | Strong | `Out`/stream-split (pipe-safe REPL/TUI output), `--format/--json`, `NO_COLOR`/color contract for the TUI palette. No JSON hand-formatting in W6. |
| **W8** Crypto & Trust | W6 **displays** | 3, 5 | Soft→Hard@2030 | Signer/attestation verdict fields → trust badges in TUI/LSP/inlay. Rendered, never re-decided. |
| **W5** Workflow Automation | W6 **consumes** | 4 | Hard (P4) | `FileWatcher` change events → live TUI refresh + LSP re-verify. |
| **W9** Ecosystem & Standards | W6 **gives** | 5 | Strong | Embeddable `ReceiptWidget` + workspace view as the standards-facing provenance surface. |
| **W10** Compliance & Governance | W6 **displays** | 3–5 | Soft | Framework/control status (from compliance verbs) shown as filters/badges in store TUI & LSP tree. |

**Anchor restated:** W6's 2026 H2 work is the synthesis **P2** line —
"REPL upgrade (registry-driven completion, working-chain `Session`, full dispatch) + optional
TUI" (`docs/innovation/00-SYNTHESIS.md:140`) — and it cannot start before P0's `registry.rs`
and output contract (`00-SYNTHESIS.md:124-133`) land. From there W6 grows the same three
surfaces — REPL, TUI, LSP — outward each year toward editor-native, drift-free, doctrine-true
provenance by 2030.
