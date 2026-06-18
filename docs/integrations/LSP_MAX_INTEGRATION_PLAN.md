# lsp-max Integration Plan — Affidavit v26.6.17

**Objective:** Add LSP (Language Server Protocol) support for affidavit `.receipt.json` files, enabling IDE browsing of receipt structure, event chains, and BLAKE3 hashes.

**Scope:** 80/20 reuse of lsp-max runtime; 20% receipt-specific handlers for navigation, hover, and diagnostics.

**Target:** Stable Rust; integrates alongside existing core/v1 verifier pipeline.

---

## Executive Summary

Affidavit receipts are deterministic, content-addressed JSON files encoding operation-event chains. Currently, developers read receipts through `affi show` (CLI display). The lsp-max integration exposes receipt structure to IDEs:

- **Document symbols** → list all events in the chain (quick outline)
- **Hover** → show event details (id, seq, event_type, chain_hash context)
- **Goto definition** → jump from ObjectRef to its containing event
- **Completion** → payload commitment suggestions, object type hints
- **Diagnostics** → flag tampered chains, malformed hashes (proof gates)

**Reuse from lsp-max:**
- LSP protocol marshalling (lsp-max-protocol)
- stdio/tcp transports (lsp-max-runtime)
- Document store + diagnostic sink (lsp-max primitives)
- Async tower-based handler pattern

**20% New Code:**
- Receipt JSON parser/indexer (adapt existing Receipt type)
- Event/object symbol extraction (from receipt.events)
- Hover resolver (event metadata → hover markdown)
- Goto definition (object references → event positions)
- Proof gate diagnostics (chain hash, commitment validation)

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────┐
│ IDE (VS Code, Neovim, Emacs) [LSP Client]              │
└──────────────────┬──────────────────────────────────────┘
                   │ LSP protocol (JSON-RPC over stdio)
                   ▼
┌─────────────────────────────────────────────────────────┐
│ affidavit-lsp (new crate) [LSP Server]                  │
│                                                         │
│ ┌──────────────────────────────────────────────────────┐│
│ │ ReceiptLsp (LanguageServer impl)                     ││
│ │  - initialize() → capabilities (docSymbols, hover)   ││
│ │  - did_open() → parse receipt, emit diagnostics      ││
│ │  - did_change() → reparse, re-validate               ││
│ │  - document_symbol() → list events as symbols        ││
│ │  - hover() → event details                           ││
│ │  - goto_definition() → object reference resolution   ││
│ │  - completion() → payload commitments, types         ││
│ └──────────────────────────────────────────────────────┘│
│                                                         │
│ ┌──────────────────────────────────────────────────────┐│
│ │ Receipt Index (receipt-specific)                     ││
│ │  - events: Vec<ReceiptSymbol> (symbol table)         ││
│ │  - objects: HashMap<String, Vec<EventIdx>>          ││
│ │  - payload_commitments: HashSet<Blake3Hash>         ││
│ │  - event_types: Vec<String>                         ││
│ │  - diagnostics: Vec<ProofGateDiagnostic>            ││
│ └──────────────────────────────────────────────────────┘│
│                                                         │
│ ┌──────────────────────────────────────────────────────┐│
│ │ lsp-max Runtime (reused)                            ││
│ │  - LspService, Server                               ││
│ │  - DocumentStore (from lsp-max::primitives)         ││
│ │  - DiagnosticSink (from lsp-max::primitives)        ││
│ │  - Transport: stdio (default) or tcp                ││
│ └──────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
                   │
                   ▼
         ┌─────────────────────┐
         │ Receipt .json file  │
         │ (document URI)      │
         └─────────────────────┘
```

### Receipt Index Structure

The Receipt Index is the central data structure mapping receipt JSON to LSP symbols:

```rust
#[derive(Debug, Clone)]
pub struct ReceiptIndex {
    /// Parsed receipt (if deserialization succeeded)
    pub receipt: Option<Receipt>,
    
    /// Symbol table: event_id → (line, col, seq, event_type, objects)
    pub events: Vec<ReceiptSymbol>,
    
    /// objects: object_id → Vec<(event_idx, seq, role/qualifier)>
    /// Enables quick traversal from object ref to containing events
    pub object_refs: HashMap<String, Vec<ObjectRefLocation>>,
    
    /// All payload commitments seen in the chain
    pub commitments: HashSet<Blake3Hash>,
    
    /// All event types seen
    pub event_types: Vec<String>,
    
    /// All object types seen
    pub object_types: Vec<String>,
    
    /// Proof gate diagnostics (tamper, malformed, etc.)
    pub diagnostics: Vec<ProofGateDiagnostic>,
    
    /// Human-readable chain summary
    pub chain_summary: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptSymbol {
    pub event_id: String,
    pub seq: u64,
    pub event_type: String,
    pub range: Range,             // LSP Range of the event in JSON
    pub objects: Vec<ObjectRef>,
    pub payload_commitment: String,
}

#[derive(Debug, Clone)]
pub struct ObjectRefLocation {
    pub event_idx: usize,
    pub seq: u64,
    pub qualifier: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ProofGateDiagnostic {
    ChainHashMismatch { stored: String, recomputed: String },
    MalformedCommitment { commitment: String },
    DuplicateEventId { id: String },
    SeqDiscontiguity { expected: u64, found: u64 },
    FormatVersionMismatch { expected: String, found: String },
}
```

### LSP Capabilities Exported

| Capability | Supported | Notes |
|---|---|---|
| **textDocumentSync** | Full (KIND) | Reparse on every change |
| **documentSymbolProvider** | ✅ | List events (SymbolKind::Event mapped to Function) |
| **hoverProvider** | ✅ | Event details: seq, event_type, commitment, objects |
| **definitionProvider** | ✅ | ObjectRef → containing event definition |
| **completionProvider** | ✅ | Payload commitments, event types, object types |
| **diagnosticProvider** | ✅ | Chain integrity, hash validation, proof gates |
| **semanticTokensProvider** | ⏳ | Future: color coding (event_id, commitment, object_id) |

---

## Implementation Phases

### Phase 1: Core Server & Parsing (30% effort)

**Goal:** Basic LSP server serving receipt.json files with diagnostic validation.

**Tasks:**

1. **Create `crates/affidavit-lsp` crate**
   - `Cargo.toml`: depend on `lsp-max`, `affidavit` lib
   - `src/main.rs`: server entrypoint (stdio transport)
   - `src/server.rs`: LanguageServer impl

2. **Receipt Index Builder**
   - `src/index.rs`: build ReceiptIndex from Receipt
   - Walk events, extract symbols, compute ranges
   - Detect proof gate violations (chain hash, commitments)

3. **LSP Capabilities**
   - `initialize()` → report capabilities
   - `did_open()` → parse receipt, validate, emit diagnostics
   - `did_change()` → reparse, re-validate
   - `did_close()` → cleanup

4. **Diagnostics Pipeline**
   - `src/diagnostics.rs`: ProofGateDiagnostic → LSP Diagnostic
   - Chain hash validation (recompute, compare)
   - Commitment format check (BLAKE3 hex)
   - Seq contiguity, event id uniqueness
   - Format version check

**Output:** Server compiles, starts on stdio, accepts receipt.json URIs, emits diagnostics.

---

### Phase 2: Navigation (40% effort)

**Goal:** IDE navigation (symbols, hover, goto definition).

**Tasks:**

1. **Document Symbols Handler**
   - `src/handlers/document_symbol.rs`
   - Map events → SymbolInformation
   - SymbolKind::Function for events (no SymbolKind::Event in LSP 3.18)
   - Range points to the `{ "id": "evt-N", ...}` object in JSON

2. **Hover Handler**
   - `src/handlers/hover.rs`
   - Hover at event_id → show event details
   - Hover at object_id → show all events referencing this object
   - Hover at commitment → hex preview, object count
   - Markdown response: event_type, seq, objects, commitment

3. **Goto Definition Handler**
   - `src/handlers/goto_definition.rs`
   - Click on ObjectRef in event → jump to event definition
   - Resolve from DocumentState URI + position to symbol name
   - Return Location pointing to event object in JSON

4. **Text Helpers**
   - `src/text.rs`: Position ↔ offset conversion
   - JSON structure awareness (events array start, event object bounds)

**Output:** IDE can outline receipts, hover over events/objects, jump between references.

---

### Phase 3: Completion & Diagnostics (20% effort)

**Goal:** Reduce manual typing; real-time proof gate feedback.

**Tasks:**

1. **Completion Handler**
   - `src/handlers/completion.rs`
   - At cursor in event_type field → suggest event_types from index
   - At cursor in objects array → suggest seen objects
   - At cursor in commitment field → suggest existing commitments

2. **Diagnostics Integration**
   - Real-time chain validation on did_change
   - Proof gate violations → error diagnostics with ranges
   - Suggestion: "chain hash recomputed to X; verify receipt"
   - DiagnosticSink::publish() at did_change completion

3. **Stdlib Additions**
   - Transport selection (stdio/tcp)
   - Logging (optional, feature-gated)

**Output:** Full IDE UX; developers see tamper/forgery instantly while editing.

---

## File Structure

```
affidavit/
├── Cargo.toml (add feature: lsp)
├── crates/affidavit-lsp/              ← NEW
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                (server entrypoint + argparse)
│   │   ├── server.rs              (LanguageServer impl)
│   │   ├── index.rs               (ReceiptIndex builder)
│   │   ├── diagnostics.rs         (proof gate → LSP Diagnostic)
│   │   ├── text.rs                (position/offset helpers)
│   │   ├── handlers/
│   │   │   ├── document_symbol.rs (events → symbols)
│   │   │   ├── hover.rs           (event details)
│   │   │   ├── goto_definition.rs (object ref → event)
│   │   │   └── completion.rs      (payload/type hints)
│   │   └── lib.rs
│   └── tests/
│       └── integration.rs         (LSP roundtrip tests)
└── INTEGRATIONS.md (update Phase 3 status)
```

---

## Integration Points with lsp-max

### Reused Primitives

| Component | Source | Use |
|---|---|---|
| `LanguageServer` trait | `lsp-max` | Impl in ReceiptLsp |
| `LspService`, `Server` | `lsp-max` | Dispatch, transport |
| `DocumentStore` | `lsp-max::primitives` | Cache receipt JSON text |
| `DiagnosticSink` | `lsp-max::primitives` | Publish diagnostics |
| `Client` | `lsp-max` | Notification callbacks |
| `Url`, `Range`, `Position` | `lsp-types-max` | LSP types |

### Receipt-Specific Handlers

| Handler | Type | Input | Output |
|---|---|---|---|
| `index_receipt()` | fn | Receipt + uri | ReceiptIndex |
| `event_to_symbol()` | fn | OperationEvent + position | SymbolInformation |
| `hover_event()` | fn | event_id + index | Hover (markdown) |
| `complete_event_types()` | fn | index | Vec<CompletionItem> |
| `validate_chain()` | fn | events | Vec<ProofGateDiagnostic> |

---

## Testing Strategy

### Unit Tests (in `crates/affidavit-lsp/tests/`)

1. **Indexing**
   - Parse receipt.json → ReceiptIndex
   - Verify event symbols extracted
   - Verify object refs mapped

2. **Diagnostics**
   - Tampered receipt → ChainHashMismatch diagnostic
   - Malformed commitment → MalformedCommitment diagnostic
   - Discontiguous seq → SeqDiscontiguity diagnostic

3. **Handlers**
   - document_symbol(uri, index) → Vec<SymbolInformation>
   - hover(uri, position, index) → Option<Hover>
   - goto_definition(uri, position, index) → Option<Vec<Location>>

### Integration Tests

1. **LSP Roundtrip**
   - Start server on stdio
   - Send initialize
   - Open receipt.json
   - Request documentSymbol
   - Request hover at event position
   - Verify responses
   - Shutdown

2. **Tamper Detection in IDE**
   - Open receipt.json
   - Manually corrupt event_type
   - Receive ChainHashMismatch diagnostic
   - Auto-suggest verification

---

## Dependency Changes

### Cargo.toml Updates

```toml
# In root Cargo.toml:
[features]
lsp = ["affidavit-lsp"]

# In new crates/affidavit-lsp/Cargo.toml:
[dependencies]
affidavit = { path = "..", features = ["evidence"] }
lsp-max = { path = "../../lsp-max" }
lsp-types-max = { path = "../../lsp-types-max" }
wasm4pm-compat = { path = "../../wasm4pm-compat" }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros", "sync"] }
anyhow = "1"
```

### Feature Flags

- **Default build:** No LSP (affidavit binary + lib unchanged)
- **With LSP:** `cargo build --features lsp` → binary + lsp server
- **LSP only:** `cargo build -p affidavit-lsp --bin affidavit-lsp`

---

## CLI Integration

### Option 1: Unified Binary
```bash
affi lsp --stdio       # Start LSP server on stdio
affi lsp --tcp 3000    # Start LSP on TCP port 3000
```

Requires adding `lsp` subcommand to `ontology/affi-cli.ttl`.

### Option 2: Separate Binary
```bash
cargo install affidavit --features lsp
affidavit-lsp --stdio     # Separate binary
```

**Recommendation:** Option 2 (separate binary) to keep core `affi` focused on receipt operations.

---

## Proof Gates (Diagnostics)

The LSP server runs the same verifier that `affi verify` runs, but **non-blocking**:

| Proof Gate | Stage | Diagnostic Code | Severity |
|---|---|---|---|
| **Decode** | JSON parse error | `AFFI-001` | ERROR |
| **Format version** | `core/v1` mismatch | `AFFI-002` | ERROR |
| **Chain hash** | Tamper detected | `AFFI-003` | ERROR |
| **Seq contiguity** | Gap or duplicate | `AFFI-004` | ERROR |
| **Commitment format** | Invalid BLAKE3 hex | `AFFI-005` | WARNING |
| **Event uniqueness** | Duplicate event id | `AFFI-006` | ERROR |

**Flow:** `did_change()` → reparse → run diagnostics → publish

---

## 80/20 Reuse Summary

### lsp-max (80% code reuse)
- Protocol marshalling
- Transport (stdio, TCP, WebSocket)
- Document store + state management
- Async tower-based dispatch
- Hover/symbol/definition protocol plumbing

### Receipt-Specific (20% new code)
- `index_receipt()` — walk events, map to symbols
- Hover resolver — render event markdown
- Goto definition — resolve object refs
- Completion — suggest payloads/types
- Diagnostics — run proof gates on change

**Total new lines:** ~1,500–2,000 (estimate)
**Build time impact:** Minimal (lsp-max deps already in workspace)
**Stable Rust:** ✅ (lsp-max 26.6.9 is stable)

---

## Roadmap

### v26.6.17 (Current)
- [x] Affidavit core (7-stage verifier, sealed receipt)
- [ ] **Begin Phase 1** (indexing + diagnostics)

### v26.6.15 (Next)
- [ ] Complete Phase 1
- [ ] **Begin Phase 2** (navigation)

### v26.6.16 (Following)
- [ ] Complete Phase 2
- [ ] **Begin Phase 3** (completion + final QA)

### v26.6.17 (Release)
- [ ] Phase 3 complete
- [ ] Publish `affidavit-lsp` to crates.io
- [ ] Update INTEGRATIONS.md (Phase 3 standing → Phase 3 complete)

---

## Success Criteria

1. ✅ LSP server builds on stable Rust
2. ✅ Serves receipt.json files (open/change/close)
3. ✅ Document symbols list all events
4. ✅ Hover shows event details + chain context
5. ✅ Goto definition jumps between object refs and events
6. ✅ Completion suggests payloads/types
7. ✅ Diagnostics flag proof gate violations in real-time
8. ✅ IDE integration tested (VS Code, at minimum)
9. ✅ Tests: 15+ unit, 3+ integration
10. ✅ INTEGRATIONS.md updated (Phase 3 status)

---

## Future Enhancements (Phase 4+)

- **Semantic tokens** → color coding (eventId, commitment, objectId)
- **Signature help** → show receipt.json schema inline
- **Code lens** → "Verify this receipt" → `affi verify`
- **Inline values** → show chain_hash preview at line start
- **Call hierarchy** → object → events (reverse index)
- **Multi-document support** → link multiple receipt.json files
- **Workspace symbols** → search all receipts in folder

---

## Deployment

### Installation

```bash
# From workspace
cargo install --path crates/affidavit-lsp

# Or from crates.io (after publishing)
cargo install affidavit-lsp
```

### VS Code Extension (Future)

Once server is stable, create minimal client extension:
- Register LSP server for `.receipt.json` files
- Syntax highlighting (receipt JSON)
- Commands: verify, show, emit

### Configuration

```json
{
  "[receipt]": {
    "editor.defaultFormatter": "affidavit-lsp",
    "editor.formatOnSave": false
  },
  "lsp-max.receipt.verifyOnChange": true,
  "lsp-max.receipt.diagnosticSeverity": "error"
}
```

---

**Document Status:** Planning phase  
**Last Updated:** 2026-06-14  
**Affidavit Version:** 26.6.17  
**lsp-max Version:** 26.6.9  
