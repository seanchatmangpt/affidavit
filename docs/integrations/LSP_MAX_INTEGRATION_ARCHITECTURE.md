# lsp-max Integration — Technical Architecture & Dependency Mapping

This document provides deep technical details on how lsp-max integrates with affidavit, including module structure, trait implementations, and error handling.

---

## System Architecture

### Layering Model

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 5: IDE Integration (out-of-scope for this plan)           │
│          - VS Code extension (.receipt.json language mode)      │
│          - Neovim/Emacs LSP client config                       │
└────────────────────┬────────────────────────────────────────────┘
                     │ LSP protocol (JSON-RPC)
┌────────────────────▼────────────────────────────────────────────┐
│ Layer 4: affidavit-lsp Binary (main.rs)                         │
│          - CLI: --stdio / --tcp [port]                          │
│          - Transport selection (stdio, TCP)                      │
│          - Graceful shutdown                                    │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ Layer 3: lsp-max Runtime (lsp-max crate)                        │
│          - LspService<ReceiptLsp> (async dispatch)             │
│          - Server<R, W> (socket handling)                       │
│          - jsonrpc::{Request, Response, Notification}          │
│          - Client (notification callbacks)                      │
│          - ErrorResponse marshalling                            │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ Layer 2: ReceiptLsp Handler (server.rs)                         │
│          - LanguageServer trait impl                            │
│          - DocumentState management                             │
│          - Per-method handlers (document_symbol, hover, etc.)   │
│          - Diagnostics publishing                               │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ Layer 1: Receipt Analysis (index.rs + handlers/)                │
│          - ReceiptIndex builder (from Receipt JSON)            │
│          - Symbol extraction (events → symbols)                 │
│          - Proof gate validation (chain hash, commitments)      │
│          - Hover/definition/completion resolvers                │
│          - Range calculation (LSP Position ↔ JSON offset)       │
└────────────────────┬────────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────────┐
│ Underlying Crates (reused):                                     │
│          - affidavit::types (Receipt, OperationEvent, etc.)    │
│          - affidavit::chain (recompute_chain)                  │
│          - affidavit::verifier (proof gates)                   │
│          - lsp-types-max (LSP 3.18 types)                      │
│          - serde_json (JSON parsing/serialization)             │
└─────────────────────────────────────────────────────────────────┘
```

---

## Trait Implementation Map

### `LanguageServer` Trait (from lsp-max)

```rust
#[async_trait]
pub trait LanguageServer: Send + Sync {
    // Core lifecycle
    async fn initialize(&self, params: InitializeParams) 
        -> jsonrpc::Result<InitializeResult>;
    async fn shutdown(&self) -> jsonrpc::Result<()>;
    async fn exit(&self) -> jsonrpc::Result<()> { ... }

    // Document lifecycle (notifications)
    async fn did_open(&self, params: DidOpenTextDocumentParams);
    async fn did_change(&self, params: DidChangeTextDocumentParams);
    async fn did_close(&self, params: DidCloseTextDocumentParams);
    async fn did_save(&self, params: DidSaveTextDocumentParams) { ... }

    // Symbol queries (requests)
    async fn document_symbol(&self, params: DocumentSymbolParams)
        -> jsonrpc::Result<Option<DocumentSymbolResponse>>;

    // Hover / type information
    async fn hover(&self, params: HoverParams)
        -> jsonrpc::Result<Option<Hover>>;

    // Definition / reference navigation
    async fn goto_definition(&self, params: GotoDefinitionParams)
        -> jsonrpc::Result<Option<GotoDefinitionResponse>>;
    async fn references(&self, params: ReferenceParams)
        -> jsonrpc::Result<Option<Vec<Location>>> { ... }

    // Code completion
    async fn completion(&self, params: CompletionParams)
        -> jsonrpc::Result<Option<CompletionResponse>>;
    async fn completion_resolve(&self, item: CompletionItem)
        -> jsonrpc::Result<CompletionItem> { ... }

    // ... (20+ other methods, all defaulting to Err(not_implemented()))
}
```

**ReceiptLsp Implementation Strategy:**
- Implement 6 core methods (initialize, shutdown, did_open, did_change, did_close, document_symbol, hover, goto_definition, completion)
- All others inherit default (not_implemented error)

---

## Module Structure & Dependencies

### Workspace Layout

```
affidavit/
├── src/                    ← EXISTING (core receipt logic)
│   ├── lib.rs
│   ├── types.rs            (Receipt, OperationEvent, etc.)
│   ├── chain.rs            (recompute_chain, ChainAssembler)
│   ├── verifier.rs         (7-stage pipeline)
│   ├── ocel.rs             (ObjectRef, SeqCounter)
│   └── ...
├── crates/affidavit-lsp/   ← NEW (LSP server)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         (cli, transport)
│       ├── lib.rs          (module decls)
│       ├── server.rs       (LanguageServer impl)
│       ├── index.rs        (ReceiptIndex builder)
│       ├── diagnostics.rs  (proof gate → LSP Diagnostic)
│       ├── text.rs         (Position ↔ offset helpers)
│       ├── handlers/
│       │   ├── mod.rs
│       │   ├── document_symbol.rs
│       │   ├── hover.rs
│       │   ├── goto_definition.rs
│       │   └── completion.rs
│       └── tests/
│           └── integration.rs
└── LSP_MAX_INTEGRATION_*.md (this series)
```

### Import Chains

```rust
// In crates/affidavit-lsp/src/server.rs
use affidavit::types::{Receipt, OperationEvent, Blake3Hash, ObjectRef};
use affidavit::chain;                    // recompute_chain
use lsp_max::{LspService, Server, Client, LanguageServer};
use lsp_types_max::{InitializeResult, ServerCapabilities, Diagnostic, ...};
use crate::index::ReceiptIndex;           // Local receipt analysis
use crate::handlers;                      // Local per-method handlers

impl LanguageServer for ReceiptLsp {
    // Dispatch to handlers which use index
}
```

---

## Data Flow: Did_Open → Diagnostics

### Sequence Diagram

```
IDE (client)
  │
  └─→ textDocument/didOpen
      │
      ├─→ LSP protocol (JSON-RPC)
      │
      ▼ (server receives)
   ReceiptLsp::did_open(DidOpenTextDocumentParams)
      │
      ├─→ Parse JSON text
      │
      ├─→ serde_json::from_str::<Receipt>(text)
      │   ├─→ [SUCCESS] → receipt: Receipt
      │   └─→ [FAIL] → index = None, emit JSON parse error diagnostic
      │
      ├─→ (if receipt parsed)
      │   ├─→ ReceiptIndex::from_receipt(&receipt, &uri, &text)
      │   │
      │   ├─→ Walk events, build symbols
      │   ├─→ Validate format_version
      │   ├─→ Recompute chain_hash (affidavit::chain::recompute_chain)
      │   ├─→ Check seq contiguity
      │   ├─→ Check commitment format
      │   ├─→ Collect ProofGateDiagnostic instances
      │   │
      │   └─→ return ReceiptIndex { events, object_refs, diagnostics, ... }
      │
      ├─→ Convert diagnostics
      │   diagnostics
      │     .iter()
      │     .map(|d| crate::diagnostics::to_lsp_diagnostic(d, &index))
      │
      ├─→ Call client.publish_diagnostics(uri, diags, version)
      │
      ▼ (client notified)
   IDE renders diagnostic squiggles/gutter icons
```

---

## Error Handling Strategy

### JSON Deserialization

```rust
// In ReceiptLsp::analyze()
match serde_json::from_str::<Receipt>(text) {
    Ok(receipt) => {
        // Index successfully built
        DocumentState {
            text,
            index: Some(ReceiptIndex::from_receipt(...)),
        }
    }
    Err(e) => {
        // JSON parse error: emit diagnostic but don't crash
        DocumentState {
            text,
            index: None,  // ← no index, skip handlers
        }
        // Emit single diagnostic: "Receipt JSON parse error: {}"
    }
}
```

### Handler Graceful Degradation

```rust
// In ReceiptLsp::document_symbol()
pub async fn document_symbol(&self, params: DocumentSymbolParams)
    -> jsonrpc::Result<Option<DocumentSymbolResponse>>
{
    // Three levels of fallback:
    // 1. Document not open → return Ok(None)
    // 2. Document open but index = None → return Ok(None) (emit diagnostic earlier)
    // 3. Document open, index = Some → process normally

    let Some(state) = self.documents.get(&params.text_document.uri) else {
        return Ok(None);  // ← Silent fallback (document not yet open)
    };

    let Some(ref index) = state.index else {
        return Ok(None);  // ← Silent fallback (parse error; diagnostic already published)
    };

    Ok(Some(DocumentSymbolResponse::Nested(
        crate::handlers::document_symbol::handle(index)
    )))
}
```

### LSP Error Responses

```rust
// lsp-max automatically converts panics → jsonrpc::Error(-32603, "Internal error")
// For explicit errors:

use lsp_max::jsonrpc;

// In handlers, return Err for protocol-level errors
async fn some_handler(...) -> jsonrpc::Result<T> {
    match do_thing() {
        Ok(v) => Ok(v),
        Err(e) => Err(jsonrpc::Error {
            code: -32000,  // Server error range
            message: format!("Handler failed: {}", e),
            data: None,
        }),
    }
}
```

---

## Proof Gate Validation Pipeline

### Stage Mapping (mirroring affidavit::verifier)

| Stage | Implemented in | Diagnostic Type | Severity |
|---|---|---|---|
| **Decode** | `index.rs::from_receipt()` | ChainHashMismatch (serde) | ERROR |
| **check_format** | `index.rs::from_receipt()` | FormatVersionMismatch | ERROR |
| **chain_integrity** | `index.rs::from_receipt()` | ChainHashMismatch | ERROR |
| **continuity** | `index.rs::from_receipt()` | SeqDiscontiguity | ERROR |
| **verify_commitments** | `index.rs::from_receipt()` | MalformedCommitment | WARNING |
| **evaluate_profile** | `index.rs::from_receipt()` (implicit in Receipt type) | (none) | N/A |
| **emit_verdict** | (not done in LSP; real verify still needed) | N/A | N/A |

**Key insight:** LSP doesn't need to emit a Verdict; it just flags violations. The real `affi verify` command is the authoritative gate.

---

## Type System: Receipt → LSP Symbols

### From affidavit Types

```rust
// affidavit::types::Receipt
pub struct Receipt {
    pub format_version: String,
    pub events: Vec<OperationEvent>,
    pub chain_hash: Blake3Hash,
    _seal: (),  // ← Makes Receipt construction unconstructable
}

// affidavit::types::OperationEvent
pub struct OperationEvent {
    pub id: String,
    pub seq: u64,
    pub event_type: String,
    pub objects: Vec<ObjectRef>,
    pub payload_commitment: Blake3Hash,
}

// affidavit::types::ObjectRef
pub struct ObjectRef {
    pub id: String,
    pub obj_type: String,
    pub qualifier: Option<String>,
}

// affidavit::types::Blake3Hash
pub struct Blake3Hash(pub String);  // hex string, 64 chars
```

### To LSP Types

```rust
// DocumentSymbol (for each event)
DocumentSymbol {
    name: "evt-0 (seq 0)",                // event_id + seq
    detail: Some("init"),                 // event_type
    kind: SymbolKind::FUNCTION,           // (no Event kind in LSP 3.18)
    range: Range { ... },                 // position in JSON
    selection_range: Range { ... },       // same as range
}

// Hover (on event)
Hover {
    contents: HoverContents::Markup(
        MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**Event:** {}\n**Seq:** {}\n**Type:** {}...",
                event.id, event.seq, event.event_type
            ),
        }
    ),
    range: Some(Range { ... }),
}

// Location (for goto definition)
Location {
    uri: Url { ... },                     // receipt.json URI
    range: Range { ... },                 // event object position
}

// CompletionItem (for type/commitment suggestions)
CompletionItem {
    label: "init",
    kind: Some(CompletionItemKind::KEYWORD),
    detail: Some("event type"),
    ..Default::default()
}

// Diagnostic (for proof gate violations)
Diagnostic {
    range: Range { ... },
    severity: Some(DiagnosticSeverity::ERROR),
    code: Some(NumberOrString::String("AFFI-003")),
    source: Some("affidavit-lsp"),
    message: "Chain hash mismatch: stored X, recomputed Y".to_string(),
    ..Default::default()
}
```

---

## Concurrency Model

### Thread Safety

```
                     ┌─────────────────────────────┐
                     │  LspService<ReceiptLsp>    │
                     │  (Arc<ReceiptLsp>)          │
                     └──────────┬──────────────────┘
                                │
                   ┌────────────┴────────────┬─────────────────┐
                   ▼                         ▼                 ▼
            did_open (spawn)        did_change (spawn)    hover (spawn)
                   │                         │                 │
                   └────────┬────────────────┴─────────────────┘
                            │
                            ▼
                   Arc<DashMap<Url, DocumentState>>
                   (lock-free, concurrent reads/writes)
                            │
                   ┌────────┴────────┐
                   ▼                 ▼
              get (read)         insert (write)
              to extract        to update after
              index for          analyze/reparse
              handler calls
```

**Key choices:**
- `DashMap` instead of `RwLock<HashMap>`: lock-free reads during hover/completion
- `Arc<ReceiptLsp>`: each spawned handler task gets a clone
- Handlers are `async fn`, so blocking I/O not a concern
- No global state; each document is independent

---

## LSP Capability Negotiation

### Initialize Request/Response Flow

**Client → Server:**
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "method": "initialize",
  "params": {
    "clientInfo": { "name": "vscode", ... },
    "rootUri": "file://~/affidavit",
    "capabilities": { ... }
  }
}
```

**Server → Client (ReceiptLsp::initialize):**
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "capabilities": {
      "textDocumentSync": 1,  // FULL (reparse on every change)
      "documentSymbolProvider": true,
      "hoverProvider": true,
      "definitionProvider": true,
      "completionProvider": {
        "resolveProvider": false,
        "triggerCharacters": ["\""]
      }
    },
    "serverInfo": { "name": "affidavit-lsp", "version": "26.6.17" }
  }
}
```

**Client caches these capabilities** and only calls handlers if the capability is advertised. If you advertise `documentSymbolProvider: true`, the client will call `textDocument/documentSymbol` when user requests outline.

---

## Range/Position Calculation

### LSP Coordinate System

- **Line:** 0-based, incremented at `\n`
- **Character:** 0-based UTF-16 code units (important for multi-byte chars)
- **Range:** `{ start: Position, end: Position }`

### JSON Event Position Example

```json
{
  "format_version": "core/v1",
  "events": [
    {                            ← Line 2, character 4 (event object start)
      "id": "evt-0",             ← Line 3, character 6 ("evt-0" start)
      "seq": 0,
      "event_type": "init",
      "objects": [...],
      "payload_commitment": "abcd..."
    },
    ...
  ],
  "chain_hash": "..."
}
```

**ReceiptSymbol.range:** Should point to the line/col of `"id": "evt-0"`, or the event object as a whole.

### Text Offset ↔ Position Conversion

```rust
fn text_offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut curr = 0usize;

    for ch in text.chars() {
        if curr == offset {
            return Position { line, character: col };
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;  // UTF-16 code units, not bytes
        }
        curr += ch.len_utf8();
    }
    Position { line, character: col }
}

fn position_to_text_offset(text: &str, pos: Position) -> Option<usize> {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut offset = 0usize;

    for ch in text.chars() {
        if line == pos.line && col == pos.character {
            return Some(offset);
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        offset += ch.len_utf8();
    }
    None
}
```

---

## Integration Points with Affidavit Core

### Reused Functions

```rust
// affidavit::chain (public API)
pub fn recompute_chain(events: &[OperationEvent]) -> Result<Blake3Hash> {
    // Called by ReceiptIndex::from_receipt() to validate chain
}

// affidavit::types (public API)
impl Receipt {
    pub fn sealed(...) -> Self { ... }  // Not called (private constructor)
}

impl<'de> Deserialize<'de> for Receipt {
    // Called by ReceiptIndex builder via serde_json::from_str()
    // Automatically re-verifies chain hash during deserialization
}

// affidavit::ocel (public API)
pub struct ObjectRef {
    pub id: String,
    pub obj_type: String,
    pub qualifier: Option<String>,
}
```

### No Modifications to affidavit Core

- LSP doesn't add features to `affi` CLI
- Doesn't change Receipt sealing logic
- Doesn't expose private `_seal` field
- Uses Receipt as read-only witness

---

## Testing Architecture

### Unit Test Pyramid

```
                 ▲
                 │         Integration Tests (3)
                 │      ┌────────────────────────┐
                 │      │ LSP roundtrip (stdio)   │
                 │      │ - initialize            │
                 │      │ - did_open + diagnostic │
                 │      │ - document_symbol       │
                 │      └────────────────────────┘
                 │
                 │         Handler Tests (12)
                 │      ┌────────────────────────┐
                 │      │ Index builder → symbol │
                 │      │ Hover resolver         │
                 │      │ Goto definition        │
                 │      │ Completion gen         │
                 │      │ Proof gate validation  │
                 │      └────────────────────────┘
                 │
                 │         Unit Tests (20)
                 │      ┌────────────────────────┐
                 │      │ Position conversion    │
                 │      │ Diagnostic mapping     │
                 │      │ Symbol extraction      │
                 │      │ Commitment validation  │
                 │      └────────────────────────┘
                 │
              Tests/Implementation Coverage
```

---

## Configuration & Environment

### No Runtime Configuration

The LSP server has minimal configuration:
- `--stdio` (default): LSP over stdin/stdout
- `--tcp <port>`: LSP over TCP

No `.lsp.toml` or environment variables. The verifier rules are compiled-in (core/v1 profile).

### IDE Configuration (client-side)

```jsonc
// VS Code settings.json
{
  "[json]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "lsp-max.affidavit.enabled": true,
  "lsp-max.affidavit.serverPath": "/usr/local/bin/affidavit-lsp",
  "lsp-max.affidavit.serverArgs": ["--stdio"]
}
```

---

## Performance Considerations

### Complexity Analysis

| Operation | Complexity | Notes |
|---|---|---|
| Parse receipt JSON | O(n) | One-time on did_open, cached |
| Recompute chain hash | O(n) | Canonical JSON + BLAKE3 hashing |
| Build symbol table | O(n) | Walk events once |
| Hover lookup | O(1) | Symbol at line, binary search |
| Goto definition | O(m) | m = count of events referencing object |
| Completion | O(k) | k = unique event types/objects (typically <100) |

### Memory Usage

- **Per-document:** Receipt JSON text + ReceiptIndex (2 copies of events data)
- **10 receipts × 1MB each:** ~20MB (text + index)
- **DashMap overhead:** ~100 bytes per entry

**Optimization opportunity (Phase 4):** De-duplicate event data by sharing Arc<OperationEvent>.

---

## Deployment Topology

### Development (testing)

```
┌─────────────┐
│  VS Code    │ (LSP client plugin, launches server)
└──────┬──────┘
       │ stdio
       ▼
┌─────────────────────────┐
│ cargo run --bin         │
│ affidavit-lsp --stdio   │
└─────────────────────────┘
```

### Production (published)

```
┌─────────────────────────────────────────┐
│  VS Code with affidavit-lsp extension   │
│  (marketplace)                          │
└──────────────────┬──────────────────────┘
                   │
                   ▼
      ┌────────────────────────┐
      │ affidavit-lsp binary   │
      │ (from crates.io)       │
      │ ~/.cargo/bin/           │
      └────────────────────────┘
```

### Remote Server (future)

```
┌──────────────────────────┐
│ VS Code LSP Client       │
└───────────┬──────────────┘
            │ WebSocket or TCP
            ▼
┌────────────────────────────────────┐
│ affidavit-lsp --tcp 3000           │
│ (running on CI/CD or remote host)  │
└────────────────────────────────────┘
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-06-14  
**Scope:** Phase 1–3 technical foundation
