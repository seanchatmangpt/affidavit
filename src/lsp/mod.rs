//! LSP module for affidavit.
//! Re-exports diagnostics, hover, and goto-definition logic.

pub mod diagnostics;
pub mod goto_definition;
pub mod hover;

pub use diagnostics::*;
pub use goto_definition::*;
pub use hover::*;
