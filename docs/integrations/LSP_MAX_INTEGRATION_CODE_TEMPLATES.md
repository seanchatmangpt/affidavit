# lsp-max Integration — Code Templates & Implementation Guide

This document provides concrete code skeletons and patterns to accelerate Phase 1–3 implementation.

---

## Template 1: Server Entrypoint

**File:** `crates/affidavit-lsp/src/main.rs`

```rust
//! affidavit-lsp — Language Server for receipt.json files
//!
//! Serves LSP over stdio (default) or TCP.

use anyhow::Result;
use clap::Parser;
use lsp_max::{LspService, Server};
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod server;
use server::ReceiptLsp;

#[derive(Parser)]
#[command(name = "affidavit-lsp")]
#[command(about = "LSP server for affidavit receipt.json files")]
struct Args {
    /// Listen on TCP instead of stdio
    #[arg(long)]
    tcp: Option<u16>,

    /// Pipe mode (default: stdio)
    #[arg(long, default_value = "true")]
    stdio: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let (service, socket) = LspService::new(ReceiptLsp::new);

    match args.tcp {
        Some(port) => {
            // TCP mode
            let addr: SocketAddr = ([127, 0, 0, 1], port).into();
            let listener = TcpListener::bind(&addr).await?;
            eprintln!("affidavit-lsp listening on {}", addr);
            Server::new(listener, socket).serve(service).await?;
        }
        None => {
            // Stdio mode (default)
            let (read, write) = tokio::io::split(
                tokio::io::BufReader::new(std::io::stdin()).compat(),
            );
            Server::new(read, write, socket).serve(service).await?;
        }
    }

    Ok(())
}
```

---

## Template 2: LanguageServer Implementation

**File:** `crates/affidavit-lsp/src/server.rs`

```rust
use dashmap::DashMap;
use lsp_max::{
    async_trait, jsonrpc, Client, LanguageServer,
};
use lsp_types_max::*;
use std::sync::Arc;

use crate::index::ReceiptIndex;

pub struct ReceiptLsp {
    client: Client,
    /// URI → (text, index)
    documents: Arc<DashMap<Url, DocumentState>>,
}

#[derive(Clone, Debug)]
struct DocumentState {
    text: String,
    version: i32,
    index: Option<ReceiptIndex>,
}

impl ReceiptLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
        }
    }

    /// Parse receipt.json and build index
    fn analyze(&self, uri: &Url, text: &str) -> DocumentState {
        // Try to deserialize as affidavit Receipt
        match serde_json::from_str::<affidavit::types::Receipt>(text) {
            Ok(receipt) => {
                let index = ReceiptIndex::from_receipt(&receipt, uri, text);
                DocumentState {
                    text: text.to_string(),
                    version: 0,
                    index: Some(index),
                }
            }
            Err(e) => {
                // Not a valid receipt JSON; treat as invalid
                DocumentState {
                    text: text.to_string(),
                    version: 0,
                    index: None,
                }
            }
        }
    }

    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[async_trait]
impl LanguageServer for ReceiptLsp {
    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(
                    TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL),
                ),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverServerCapabilities::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["\"".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "affidavit-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        let state = self.analyze(&uri, &text);

        // Extract diagnostics if index built successfully
        let diags = if let Some(ref index) = state.index {
            index
                .diagnostics
                .iter()
                .map(|d| crate::diagnostics::to_lsp_diagnostic(d, &index))
                .collect()
        } else {
            vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Receipt JSON parse error".to_string(),
                ..Default::default()
            }]
        };

        self.publish_diagnostics(uri.clone(), diags).await;
        self.documents.insert(uri, state);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Collect all changes into new text
        let new_text = params
            .content_changes
            .into_iter()
            .fold(String::new(), |mut acc, change| {
                if let Some(text) = change.text {
                    acc = text;
                }
                acc
            });

        let state = self.analyze(&uri, &new_text);
        let diags = if let Some(ref index) = state.index {
            index
                .diagnostics
                .iter()
                .map(|d| crate::diagnostics::to_lsp_diagnostic(d, &index))
                .collect()
        } else {
            vec![]
        };

        self.publish_diagnostics(uri.clone(), diags).await;
        self.documents.insert(uri, state);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let Some(state) = self.documents.get(&uri) else {
            return Ok(None);
        };

        let Some(ref index) = state.index else {
            return Ok(None);
        };

        let symbols = crate::handlers::document_symbol::handle(index);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn hover(
        &self,
        params: HoverParams,
    ) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document.text_document.uri;
        let pos = params.position;

        let Some(state) = self.documents.get(&uri) else {
            return Ok(None);
        };

        let Some(ref index) = state.index else {
            return Ok(None);
        };

        Ok(crate::handlers::hover::handle(&state.text, pos, index))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document.text_document.uri;
        let pos = params.position;

        let Some(state) = self.documents.get(&uri) else {
            return Ok(None);
        };

        let Some(ref index) = state.index else {
            return Ok(None);
        };

        let locs = crate::handlers::goto_definition::handle(&state.text, pos, index);
        Ok(locs.map(|v| GotoDefinitionResponse::Array(v)))
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document.text_document.uri;
        let pos = params.position;

        let Some(state) = self.documents.get(&uri) else {
            return Ok(None);
        };

        let Some(ref index) = state.index else {
            return Ok(None);
        };

        let items = crate::handlers::completion::handle(&state.text, pos, index);
        Ok(Some(CompletionResponse::Array(items)))
    }
}
```

---

## Template 3: Receipt Index Builder

**File:** `crates/affidavit-lsp/src/index.rs`

```rust
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent, Receipt};
use lsp_types_max::{Position, Range, Url};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ReceiptIndex {
    pub receipt: Receipt,
    pub uri: Url,
    pub text: String,

    /// event_id → ReceiptSymbol (for document symbols)
    pub events: Vec<ReceiptSymbol>,

    /// object_id → Vec<(event_idx, seq, qualifier)>
    pub object_refs: HashMap<String, Vec<ObjectRefLocation>>,

    /// All unique event types
    pub event_types: Vec<String>,

    /// All unique object types
    pub object_types: Vec<String>,

    /// All commitments
    pub commitments: HashSet<String>,

    /// Proof gate violations
    pub diagnostics: Vec<ProofGateDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct ReceiptSymbol {
    pub event_id: String,
    pub seq: u64,
    pub event_type: String,
    pub range: Range,
    pub objects: Vec<ObjectRef>,
    pub payload_commitment: Blake3Hash,
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

impl ReceiptIndex {
    pub fn from_receipt(receipt: &Receipt, uri: &Url, text: &str) -> Self {
        let mut events = Vec::new();
        let mut object_refs: HashMap<String, Vec<ObjectRefLocation>> = HashMap::new();
        let mut event_types = Vec::new();
        let mut object_types = Vec::new();
        let mut commitments = HashSet::new();
        let mut diagnostics = Vec::new();

        // Check format version
        if receipt.format_version != "core/v1" {
            diagnostics.push(ProofGateDiagnostic::FormatVersionMismatch {
                expected: "core/v1".to_string(),
                found: receipt.format_version.clone(),
            });
        }

        // Check chain integrity
        match affidavit::chain::recompute_chain(&receipt.events) {
            Ok(recomputed) => {
                if recomputed != receipt.chain_hash {
                    diagnostics.push(ProofGateDiagnostic::ChainHashMismatch {
                        stored: receipt.chain_hash.to_string(),
                        recomputed: recomputed.to_string(),
                    });
                }
            }
            Err(_) => {
                diagnostics.push(ProofGateDiagnostic::ChainHashMismatch {
                    stored: receipt.chain_hash.to_string(),
                    recomputed: "[error recomputing]".to_string(),
                });
            }
        }

        // Check seq contiguity and event uniqueness
        let mut seen_ids = HashSet::new();
        for (idx, event) in receipt.events.iter().enumerate() {
            if event.seq != idx as u64 {
                diagnostics.push(ProofGateDiagnostic::SeqDiscontiguity {
                    expected: idx as u64,
                    found: event.seq,
                });
            }

            if !seen_ids.insert(&event.id) {
                diagnostics.push(ProofGateDiagnostic::DuplicateEventId {
                    id: event.id.clone(),
                });
            }

            // Check commitment format
            if !is_valid_blake3_hex(&event.payload_commitment.as_hex()) {
                diagnostics.push(ProofGateDiagnostic::MalformedCommitment {
                    commitment: event.payload_commitment.to_string(),
                });
            }

            commitments.insert(event.payload_commitment.as_hex().to_string());
        }

        // Build events symbols and object refs
        for (idx, event) in receipt.events.iter().enumerate() {
            // Estimate range (line 0, characters 0-100 for now; refine via JSON walk)
            let range = estimate_event_range(text, &event.id);

            events.push(ReceiptSymbol {
                event_id: event.id.clone(),
                seq: event.seq,
                event_type: event.event_type.clone(),
                range,
                objects: event.objects.clone(),
                payload_commitment: event.payload_commitment.clone(),
            });

            if !event_types.contains(&event.event_type) {
                event_types.push(event.event_type.clone());
            }

            // Build object refs index
            for obj in &event.objects {
                object_refs
                    .entry(obj.id.clone())
                    .or_insert_with(Vec::new)
                    .push(ObjectRefLocation {
                        event_idx: idx,
                        seq: event.seq,
                        qualifier: obj.qualifier.clone(),
                    });

                if !object_types.contains(&obj.obj_type) {
                    object_types.push(obj.obj_type.clone());
                }
            }
        }

        Self {
            receipt: receipt.clone(),
            uri: uri.clone(),
            text: text.to_string(),
            events,
            object_refs,
            event_types,
            object_types,
            commitments,
            diagnostics,
        }
    }
}

fn is_valid_blake3_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn estimate_event_range(text: &str, event_id: &str) -> Range {
    // Search for "id": "event_id" in JSON and return approximate range
    // For now, return a dummy range; in real code, parse JSON structure
    let start_pos = text.find(event_id).unwrap_or(0);
    let line_no = text[..start_pos].matches('\n').count() as u32;
    let line_start = text.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let col = (start_pos - line_start) as u32;

    Range {
        start: Position {
            line: line_no,
            character: col,
        },
        end: Position {
            line: line_no,
            character: col + event_id.len() as u32,
        },
    }
}
```

---

## Template 4: Diagnostics Handler

**File:** `crates/affidavit-lsp/src/diagnostics.rs`

```rust
use crate::index::{ProofGateDiagnostic, ReceiptIndex};
use lsp_types_max::{Diagnostic, DiagnosticSeverity, Position, Range};

pub fn to_lsp_diagnostic(diag: &ProofGateDiagnostic, index: &ReceiptIndex) -> Diagnostic {
    let (code, message, severity) = match diag {
        ProofGateDiagnostic::ChainHashMismatch { stored, recomputed } => (
            "AFFI-003",
            format!(
                "Chain hash mismatch: stored {}, recomputed {}",
                stored, recomputed
            ),
            DiagnosticSeverity::ERROR,
        ),
        ProofGateDiagnostic::MalformedCommitment { commitment } => (
            "AFFI-005",
            format!("Malformed commitment (not BLAKE3 hex): {}", commitment),
            DiagnosticSeverity::WARNING,
        ),
        ProofGateDiagnostic::DuplicateEventId { id } => (
            "AFFI-006",
            format!("Duplicate event id: {}", id),
            DiagnosticSeverity::ERROR,
        ),
        ProofGateDiagnostic::SeqDiscontiguity { expected, found } => (
            "AFFI-004",
            format!("Event seq discontiguous: expected {}, found {}", expected, found),
            DiagnosticSeverity::ERROR,
        ),
        ProofGateDiagnostic::FormatVersionMismatch { expected, found } => (
            "AFFI-002",
            format!("Format version mismatch: expected {}, found {}", expected, found),
            DiagnosticSeverity::ERROR,
        ),
    };

    // Find the event range in the document
    // (simplified; real code would parse JSON to find exact range)
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 1,
        },
    };

    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(lsp_types_max::NumberOrString::String(code.to_string())),
        source: Some("affidavit-lsp".to_string()),
        message,
        ..Default::default()
    }
}
```

---

## Template 5: Document Symbols Handler

**File:** `crates/affidavit-lsp/src/handlers/document_symbol.rs`

```rust
use crate::index::ReceiptIndex;
use lsp_types_max::{DocumentSymbol, SymbolKind, SymbolInformation};

pub fn handle(index: &ReceiptIndex) -> Vec<DocumentSymbol> {
    index
        .events
        .iter()
        .map(|symbol| DocumentSymbol {
            name: format!("{} (seq {})", symbol.event_id, symbol.seq),
            detail: Some(symbol.event_type.clone()),
            kind: SymbolKind::FUNCTION, // No SymbolKind::EVENT in LSP 3.18
            range: symbol.range,
            selection_range: symbol.range,
            deprecated: None,
            children: None,
            tags: None,
        })
        .collect()
}
```

---

## Template 6: Hover Handler

**File:** `crates/affidavit-lsp/src/handlers/hover.rs`

```rust
use crate::index::ReceiptIndex;
use lsp_types_max::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

pub fn handle(text: &str, pos: Position, index: &ReceiptIndex) -> Option<Hover> {
    // Find which event the position is in
    for symbol in &index.events {
        if symbol.range.start.line <= pos.line && pos.line <= symbol.range.end.line {
            let commitment_prefix = &symbol.payload_commitment.as_hex()[..8];
            let objects_str = symbol
                .objects
                .iter()
                .map(|o| {
                    format!(
                        "  - {} ({}{})",
                        o.id,
                        o.obj_type,
                        o.qualifier
                            .as_ref()
                            .map(|q| format!(", {})", q))
                            .unwrap_or_else(|| ")".to_string())
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let md = format!(
                "**Event:** {}\n\n\
                 **Seq:** {}\n\n\
                 **Type:** {}\n\n\
                 **Objects:**\n{}\n\n\
                 **Commitment:** `{}`...",
                symbol.event_id, symbol.seq, symbol.event_type, objects_str, commitment_prefix
            );

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: md,
                }),
                range: None,
            });
        }
    }

    None
}
```

---

## Template 7: Goto Definition Handler

**File:** `crates/affidavit-lsp/src/handlers/goto_definition.rs`

```rust
use crate::index::ReceiptIndex;
use lsp_types_max::{Location, Position, Url};

pub fn handle(text: &str, pos: Position, index: &ReceiptIndex) -> Option<Vec<Location>> {
    // Extract the word/identifier at position
    let word = word_at_position(text, pos)?;

    // Check if it's an object id in our object_refs
    let locations = index
        .object_refs
        .get(&word)?
        .iter()
        .filter_map(|loc| {
            index.events.get(loc.event_idx).map(|sym| Location {
                uri: index.uri.clone(),
                range: sym.range,
            })
        })
        .collect::<Vec<_>>();

    if locations.is_empty() {
        None
    } else {
        Some(locations)
    }
}

fn word_at_position(text: &str, pos: Position) -> Option<String> {
    // Convert LSP position to text offset and extract word
    let line_start = text.lines().take(pos.line as usize).fold(0, |acc, line| {
        acc + line.len() + 1 // +1 for newline
    });

    let offset = line_start + pos.character as usize;
    if offset >= text.len() {
        return None;
    }

    let bytes = text.as_bytes();

    // Scan backwards for opening quote
    let mut start = offset;
    while start > 0 && bytes[start - 1] != b'"' {
        start -= 1;
    }

    // Scan forwards for closing quote
    let mut end = offset;
    while end < text.len() && bytes[end] != b'"' {
        end += 1;
    }

    Some(text[start..end].to_string())
}
```

---

## Template 8: Completion Handler

**File:** `crates/affidavit-lsp/src/handlers/completion.rs`

```rust
use crate::index::ReceiptIndex;
use lsp_types_max::{CompletionItem, CompletionItemKind};

pub fn handle(text: &str, pos: lsp_types_max::Position, index: &ReceiptIndex) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Suggest event types
    for et in &index.event_types {
        items.push(CompletionItem {
            label: et.clone(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("event type".to_string()),
            ..Default::default()
        });
    }

    // Suggest object types
    for ot in &index.object_types {
        items.push(CompletionItem {
            label: ot.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("object type".to_string()),
            ..Default::default()
        });
    }

    // Suggest commitment hashes (first 8 chars)
    for commit in &index.commitments {
        items.push(CompletionItem {
            label: format!("{}...", &commit[..8.min(commit.len())]),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some("commitment".to_string()),
            insert_text: Some(commit.clone()),
            ..Default::default()
        });
    }

    items
}
```

---

## Template 9: Cargo.toml

**File:** `crates/affidavit-lsp/Cargo.toml`

```toml
[package]
name = "affidavit-lsp"
version = "26.6.17"
edition = "2021"
description = "LSP server for affidavit receipt.json files"
license = "MIT OR Apache-2.0"
authors = ["Sean Chatman <xpointsh@gmail.com>"]
repository = "https://github.com/anthropics/affidavit"

[[bin]]
name = "affidavit-lsp"
path = "src/main.rs"

[dependencies]
affidavit = { path = "..", features = ["evidence"] }
lsp-max = { path = "../../lsp-max" }
lsp-types-max = { path = "../../lsp-types-max" }
wasm4pm-compat = { path = "../../wasm4pm-compat" }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros", "sync", "io-util"] }
tokio-util = { version = "0.7", features = ["codec"] }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
dashmap = "5"
tracing = "0.1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

---

## Template 10: Integration Test

**File:** `crates/affidavit-lsp/tests/integration.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn lsp_server_starts() {
    let mut cmd = Command::cargo_bin("affidavit-lsp")
        .expect("affidavit-lsp builds");
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn lsp_open_receipt_publishes_diagnostics() {
    // This would be a full LSP roundtrip test:
    // 1. Start server on stdio
    // 2. Send initialize request
    // 3. Send did_open with receipt.json
    // 4. Verify diagnostics published
    // 5. Shutdown
    //
    // Requires lsp-client harness (async, JSON-RPC)
}
```

---

## Key Reuse Points from lsp-max

### 1. Server Scaffolding

```rust
use lsp_max::{LspService, Server};

// Automatically handles:
// - Initialize request
// - Shutdown/exit
// - Method dispatch
// - Error handling
```

### 2. Type Wrappers

```rust
use lsp_types_max::*;

// Automatic compatibility with LSP 3.18
// SymbolInformation, Diagnostic, Hover, Location, etc.
```

### 3. Document Management

```rust
use lsp_max::primitives::{DocumentStore, DiagnosticSink};

// In Phase 2: can wrap existing DocumentStore
// (currently using DashMap directly for simplicity)
```

### 4. Async Dispatch

```rust
#[async_trait]
impl LanguageServer for ReceiptLsp {
    async fn method(...) -> jsonrpc::Result<...> { ... }
}

// Tower-based concurrency, automatic req/resp routing
```

---

## Compilation Checklist

```bash
# Phase 1 (minimally compilable)
cargo build -p affidavit-lsp

# With lsp feature (if root Cargo.toml adds it)
cargo build --features lsp

# Tests
cargo test -p affidavit-lsp

# Run server
cargo run -p affidavit-lsp -- --stdio
```

---

## LSP Protocol Reference (Quick)

| Method | Params | Returns | Handler |
|---|---|---|---|
| `initialize` | InitializeParams | InitializeResult | ReceiptLsp::initialize |
| `textDocument/didOpen` | DidOpenTextDocumentParams | (notification) | ReceiptLsp::did_open |
| `textDocument/didChange` | DidChangeTextDocumentParams | (notification) | ReceiptLsp::did_change |
| `textDocument/didClose` | DidCloseTextDocumentParams | (notification) | ReceiptLsp::did_close |
| `textDocument/documentSymbol` | DocumentSymbolParams | Vec<DocumentSymbol> | ReceiptLsp::document_symbol |
| `textDocument/hover` | HoverParams | Option<Hover> | ReceiptLsp::hover |
| `textDocument/definition` | GotoDefinitionParams | Vec<Location> | ReceiptLsp::goto_definition |
| `textDocument/completion` | CompletionParams | Vec<CompletionItem> | ReceiptLsp::completion |
| `shutdown` | () | () | ReceiptLsp::shutdown |

---

## Common Pitfalls

1. **Position encoding:** LSP uses UTF-16 code units; Rust strings are UTF-8. Handle multi-byte chars carefully.
2. **Line/column bounds:** Verify line < total_lines before indexing.
3. **Async deadlock:** Use `Arc<DashMap>` or `tokio::sync::RwLock`, not plain `Mutex`.
4. **JSON parsing:** Always handle invalid JSON gracefully (return empty index, not error).
5. **URI normalization:** Use `uri.as_str()` consistently for HashMap keys.

---

**Templates Version:** 1.0  
**Last Updated:** 2026-06-14  
**Status:** Ready for Phase 1 Implementation
