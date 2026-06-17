# lsp-max Integration — Quick Reference Card

**Print this. Keep at desk during implementation.**

---

## One-Page Architecture

```
IDE ──LSP──> affidavit-lsp (LanguageServer impl)
                  │
                  ├─→ index.rs (ReceiptIndex builder)
                  │    └─→ validate proof gates
                  │
                  ├─→ handlers/
                  │    ├─→ document_symbol (outline)
                  │    ├─→ hover (tooltips)
                  │    ├─→ goto_definition (navigation)
                  │    └─→ completion (hints)
                  │
                  ├─→ diagnostics.rs (ProofGateDiagnostic → LSP)
                  │
                  └─→ lsp-max (protocol, transport, dispatch)
```

---

## File Structure at a Glance

```
crates/affidavit-lsp/src/
├── main.rs                 ← CLI entry: --stdio / --tcp
├── server.rs               ← LanguageServer impl (ReceiptLsp)
├── index.rs                ← ReceiptIndex builder (from Receipt)
├── diagnostics.rs          ← ProofGate → LSP Diagnostic
├── text.rs                 ← Position ↔ offset helpers
├── handlers/
│   ├── document_symbol.rs  ← events → symbols (outline)
│   ├── hover.rs            ← event details (tooltip)
│   ├── goto_definition.rs  ← object ref → event (navigation)
│   └── completion.rs       ← event types, objects (hints)
└── tests/
    └── integration.rs      ← LSP roundtrip test
```

---

## Key Data Structures

### ReceiptIndex (in-memory representation)
```rust
pub struct ReceiptIndex {
    pub receipt: Receipt,                          // parsed receipt
    pub events: Vec<ReceiptSymbol>,               // symbol table
    pub object_refs: HashMap<String, Vec<...>>,  // object → events
    pub commitments: HashSet<String>,            // all Blake3 hashes
    pub event_types: Vec<String>,                // unique event types
    pub object_types: Vec<String>,               // unique object types
    pub diagnostics: Vec<ProofGateDiagnostic>,   // validation issues
}
```

### Proof Gate Diagnostics
| Code | Severity | Meaning |
|------|----------|---------|
| AFFI-001 | ERROR | JSON parse error |
| AFFI-002 | ERROR | Format version mismatch |
| AFFI-003 | ERROR | Chain hash mismatch (TAMPER!) |
| AFFI-004 | ERROR | Seq discontiguity |
| AFFI-005 | WARNING | Malformed commitment |
| AFFI-006 | ERROR | Duplicate event id |

---

## LSP Method Quick Map

| Method | Input | Output | Handler |
|---|---|---|---|
| `initialize` | InitializeParams | InitializeResult | server.rs::initialize |
| `did_open` | DidOpenTextDocumentParams | — | server.rs::did_open |
| `did_change` | DidChangeTextDocumentParams | — | server.rs::did_change |
| `did_close` | DidCloseTextDocumentParams | — | server.rs::did_close |
| `documentSymbol` | DocumentSymbolParams | Vec<DocumentSymbol> | handlers/document_symbol.rs |
| `hover` | HoverParams | Hover | handlers/hover.rs |
| `definition` | GotoDefinitionParams | Vec<Location> | handlers/goto_definition.rs |
| `completion` | CompletionParams | Vec<CompletionItem> | handlers/completion.rs |
| `shutdown` | () | () | server.rs::shutdown |

---

## Implementation Checklist (Phase 1)

- [ ] Create `crates/affidavit-lsp/Cargo.toml`
- [ ] Implement `main.rs` (CLI: --stdio / --tcp)
- [ ] Implement `server.rs` (LanguageServer trait)
  - [ ] initialize()
  - [ ] did_open()
  - [ ] did_change()
  - [ ] did_close()
- [ ] Implement `index.rs` (ReceiptIndex builder)
  - [ ] Parse Receipt
  - [ ] Validate proof gates
  - [ ] Build symbol table
- [ ] Implement `diagnostics.rs` (convert to LSP Diagnostic)
- [ ] Implement `handlers/document_symbol.rs` (basic outline)
- [ ] Build & test: `cargo build -p affidavit-lsp && cargo test -p affidavit-lsp`

---

## Key Functions to Import

```rust
// From affidavit
use affidavit::types::{Receipt, OperationEvent, Blake3Hash, ObjectRef};
use affidavit::chain::recompute_chain;  // ← for chain validation

// From lsp-max
use lsp_max::{LspService, Server, Client, LanguageServer};
use lsp_max::jsonrpc;

// From lsp-types-max
use lsp_types_max::*;  // InitializeParams, Diagnostic, Range, Position, etc.

// From serde
use serde_json;  // JSON parse/serialize

// From tokio (async runtime)
use tokio;

// From dashmap (concurrent map)
use dashmap::DashMap;
```

---

## Error Handling Pattern

```rust
// Three-level fallback in every handler:

async fn handler(...) -> jsonrpc::Result<Option<T>> {
    // Level 1: Document not open?
    let Some(state) = self.documents.get(&uri) else {
        return Ok(None);  // ← silent
    };

    // Level 2: Parse failed (already emitted diagnostic)?
    let Some(ref index) = state.index else {
        return Ok(None);  // ← silent
    };

    // Level 3: Process normally
    Ok(Some(do_thing(index)))
}
```

---

## Position Conversion (UTF-16!)

```rust
// LSP uses UTF-16 code units (not bytes!)
// This is critical for multi-byte characters

fn position_to_offset(text: &str, pos: Position) -> Option<usize> {
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
            col += ch.len_utf16() as u32;  // ← UTF-16 code units!
        }
        offset += ch.len_utf8();
    }
    None
}
```

---

## Testing Patterns

### Unit Test
```rust
#[test]
fn test_index_builder() {
    let receipt = /* build honest receipt */;
    let index = ReceiptIndex::from_receipt(&receipt, &uri, &text);
    assert_eq!(index.events.len(), 3);
    assert!(index.diagnostics.is_empty());
}
```

### Handler Test
```rust
#[test]
fn test_hover_shows_event_details() {
    let index = /* build index */;
    let pos = Position { line: 5, character: 10 };
    let hover = crate::handlers::hover::handle(&text, pos, &index);
    assert!(hover.is_some());
    assert!(hover.unwrap().contents.to_string().contains("evt-0"));
}
```

### Integration Test (sketch)
```rust
#[tokio::test]
async fn test_lsp_roundtrip() {
    // 1. Start server
    // 2. Send initialize request
    // 3. Send did_open with receipt.json
    // 4. Verify diagnostics published
    // 5. Send shutdown
}
```

---

## Debugging Tips

### Can't deserialize Receipt?
- Check JSON structure (format_version, events, chain_hash required)
- Look for parse error in ReceiptLsp::analyze()
- Emit diagnostic: "Receipt JSON parse error"

### Hover returns None?
- Is DocumentState.index = Some()?
- Check position bounds (line/character in range?)
- Verify symbol.range covers the position

### Diagnostics not publishing?
- Is client.publish_diagnostics() being called?
- Check DiagnosticSink in lsp-max::primitives
- Verify range is valid (not negative, in bounds)

### LSP client not calling handlers?
- Check ServerCapabilities in initialize() response
- Did you advertise `documentSymbolProvider: true`?
- Client caches capabilities on initialize; restart LSP if you change them

---

## Cargo Commands

```bash
# Build LSP server only
cargo build -p affidavit-lsp

# Build with all features
cargo build -p affidavit-lsp --all-features

# Run tests
cargo test -p affidavit-lsp

# Run server
cargo run -p affidavit-lsp -- --stdio

# Run with TCP
cargo run -p affidavit-lsp -- --tcp 3000

# Check formatting/lint
cargo fmt -p affidavit-lsp --check
cargo clippy -p affidavit-lsp

# Benchmark (Phase 3+)
cargo bench -p affidavit-lsp
```

---

## Phase Milestones

### Phase 1 ✓
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (15+ tests)
- [ ] `cargo run -- --stdio` accepts receipt.json
- [ ] Diagnostics published on did_change

### Phase 2 ✓
- [ ] document_symbol returns events
- [ ] hover shows event details
- [ ] goto_definition returns locations
- [ ] Navigate in IDE works

### Phase 3 ✓
- [ ] completion returns suggestions
- [ ] Integration tests pass
- [ ] 80% code coverage
- [ ] Published to crates.io

---

## Dependency Summary

```toml
[dependencies]
affidavit = { path = ".." }             # Receipt, OperationEvent, etc.
lsp-max = { path = "../../lsp-max" }    # LSP protocol + runtime
lsp-types-max = { path = "../../lsp-types-max" }  # LSP types
tokio = { version = "1", features = ["rt", "macros", "sync"] }
dashmap = "5"                           # Concurrent HashMap
serde_json = "1"                        # JSON parse
anyhow = "1"                            # Error handling

[dev-dependencies]
tempfile = "3"                          # Temp directories for tests
assert_cmd = "2"                        # CLI testing
predicates = "3"                        # Assertions
```

---

## Gotchas to Avoid

1. **JSON parse errors are not fatal** — emit diagnostic, continue
2. **DashMap.get() returns a Ref** — drop it before calling other methods
3. **Position is UTF-16, not UTF-8** — use `ch.len_utf16()` not `len()`
4. **Receipt is immutable after deserialization** — never modify Receipt in handlers
5. **Handlers are async and spawned concurrently** — use Arc/DashMap, not plain Mutex
6. **Diagnostics have line/col, not offset** — convert Position before reporting

---

## When Stuck

### Server won't start
```bash
cargo build -p affidavit-lsp --release 2>&1 | head -20
```

### Handler not called
- Check initialize() advertises capability
- Restart IDE (it caches ServerCapabilities)
- Add log: `eprintln!("handler called");`

### Test fails mysteriously
- Run in isolation: `cargo test --test <name> -- --nocapture`
- Check DocumentState is initialized
- Verify index is Some(), not None

### Performance issue
- Use `--release` build
- Check recompute_chain isn't being called excessively
- Profile with `cargo flamegraph`

---

## Quick Links

| Document | Purpose |
|---|---|
| LSP_MAX_INTEGRATION_SUMMARY.md | High-level overview (start here) |
| LSP_MAX_INTEGRATION_PLAN.md | Detailed roadmap + phases |
| LSP_MAX_INTEGRATION_CODE_TEMPLATES.md | Copy-paste code (10 templates) |
| LSP_MAX_INTEGRATION_ARCHITECTURE.md | Deep technical details |
| **← You are here** | This quick reference |

---

## Contact / Questions

- **LSP Spec:** https://microsoft.github.io/language-server-protocol/
- **lsp-max examples:** `~/lsp-max/examples/`
- **affidavit tests:** `~/affidavit/tests/`

---

**Printed from:** LSP_MAX_INTEGRATION_QUICK_REFERENCE.md  
**Last Updated:** 2026-06-14  
**Status:** Ready to implement Phase 1
