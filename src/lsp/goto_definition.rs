//! Feature 5: LSP Goto-Definition — Event Type → Handler Source
//!
//! Maps receipt event_type values to the source file and line number 
//! of their corresponding Rust handler functions.

use lsp_max::lsp_types_max::{Location, Position, Range, Url};
use std::collections::BTreeMap;
use std::sync::OnceLock;

static HANDLER_MAP: OnceLock<BTreeMap<&'static str, (&'static str, u32)>> = OnceLock::new();

fn handler_map() -> &'static BTreeMap<&'static str, (&'static str, u32)> {
    HANDLER_MAP.get_or_init(|| {
        let mut m = BTreeMap::new();
        m.insert("emit",      ("src/verbs/emit.rs",      0));
        m.insert("assemble",  ("src/verbs/assemble.rs",  0));
        m.insert("verify",    ("src/verbs/verify.rs",    0));
        m.insert("show",      ("src/verbs/show.rs",      0));
        m.insert("inspect",   ("src/verbs/inspect.rs",   0));
        m.insert("model",     ("src/verbs/model.rs",     0));
        m.insert("diagnose",  ("src/verbs/diagnose.rs",  0));
        m.insert("conform",   ("src/verbs/conform.rs",   0));
        m.insert("replay",    ("src/verbs/replay.rs",    0));
        m.insert("graph",     ("src/verbs/graph.rs",     0));
        m.insert("stats",     ("src/verbs/stats.rs",     0));
        m.insert("predict",   ("src/verbs/predict.rs",   0));
        m
    })
}

/// Return the LSP goto-definition location for a receipt event_type.
pub fn goto_definition_for_event_type(event_type: &str) -> Option<Location> {
    let (path, line) = handler_map().get(event_type)?;
    
    // In a real environment, we would resolve the absolute path.
    // For this implementation, we use a file:// URI relative to a presumed root.
    let uri_str = format!("file:///Users/sac/affidavit/{}", path);
    let uri = uri_str.parse().ok()?;

    Some(Location {
        uri,
        range: Range {
            start: Position { line: *line, character: 0 },
            end: Position {
                line: *line,
                character: 0,
            },
        },
    })
}
