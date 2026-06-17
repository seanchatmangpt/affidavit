//! Feature 4: LSP Hover — Event ID Hover Cards
//!
//! When a user hovers over a receipt event_id string, the hover response
//! shows the event's metadata in rich markdown.

use crate::types::Receipt;
use lsp_max::lsp_types_max::{Hover, HoverContents, MarkupContent, MarkupKind};

/// Produce a hover card for the event with the given ID.
/// Returns None if no event with that ID exists in the receipt.
pub fn hover_for_event_id(event_id: &str, receipt: &Receipt) -> Option<Hover> {
    let event = receipt.events.iter().find(|e| e.id == event_id)?;

    let short_hash = &event.payload_commitment.as_hex()[..12];
    let mut objects_md = String::new();
    if event.objects.is_empty() {
        objects_md.push_str("*none*");
    } else {
        for obj in &event.objects {
            let qual = obj
                .qualifier
                .as_ref()
                .map(|q| format!(":{}", q))
                .unwrap_or_default();
            objects_md.push_str(&format!("- `{}:{}{}`\n", obj.id, obj.obj_type, qual));
        }
    }

    let markdown = format!(
        "## {}\n\n\
         | Field | Value |\n\
         |-------|-------|\n\
         | event_type | `{}` |\n\
         | seq | `{}` |\n\
         | commitment | `{}...` |\n\n\
         **Objects:**\n\
         {}",
        event.id, event.event_type, event.seq, short_hash, objects_md
    );

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    })
}
