//! Holographic LSP Code Lens — Real-time process visualization in the editor.
//!
//! # Spec: Holographic DX
//!
//! **Goal:** Provide immediate visual feedback on the process state directly in
//! the IDE, eliminating the need to context-switch to a dashboard or CLI.
//!
//! ## 1. `emit!` Macro (Ergonomic API)
//! - **Syntax:** `emit!(event_type, [object_refs], payload)`
//! - **Behavior:** Automatically assigns logical sequence numbers and builds
//!   canonical OCEL events.
//! - **Implementation:** See `src/ocel.rs`.
//!
//! ## 2. LSP Code Lenses (Visual Triggers)
//! - **Detection:** Scans source code for `emit!` macro calls.
//! - **Title:** "🔍 Preview Process"
//! - **Command:** `affidavit.showHologram` (triggers a rich webview or sidebar with the full SVG).
//!
//! ## 3. LSP Inlay Hints (Inline Context)
//! - **Label:** Shows " (Step N/Total)" after each `emit!` call.
//! - **Tooltip:** Displays a mini-SVG process diagram on hover using Markdown.
//!
//! ## 4. Holographic Engine (SVG Generator)
//! - **Logic:** Turns a sequence of `OperationEvent`s into a Petri-net-style graph.
//! - **Styling:** Modern dark-mode aesthetics (nord/dracula inspired).
//!
//! # Implementation Evidence
//!
//! - **Macro:** `emit!` added to `src/ocel.rs`.
//! - **LSP Extensions:** `generate_holographic_lenses` and `generate_holographic_hints`
//!   added to `src/lsp.rs`.
//! - **Visuals:** Internal `generate_holographic_svg` logic integrated into the LSP server.

use crate::ocel::{build_event, SeqCounter};
use crate::types::{OperationEvent, ObjectRef};
use lsp_max::lsp_types::{CodeLens, InlayHint, Command, Range, Position, InlayHintLabel, InlayHintKind, MarkupContent, MarkupKind};
use serde_json::json;

/// Generates the holographic preview for the provided events.
pub fn get_hologram_svg(events: &[OperationEvent]) -> String {
    // This calls into the same logic used by the LSP server for consistency.
    crate::lsp::generate_holographic_svg(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::SeqCounter;

    #[test]
    fn test_hologram_svg_output() {
        let mut counter = SeqCounter::new();
        let events = vec![
            build_event("order-created", vec![], b"", &mut counter).unwrap(),
            build_event("payment-sent", vec![], b"", &mut counter).unwrap(),
        ];
        
        let svg = get_hologram_svg(&events);
        assert!(svg.contains("order-created"));
        assert!(svg.contains("payment-sent"));
        assert!(svg.contains("<svg"));
    }
}
