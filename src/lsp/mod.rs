//! LSP module for affidavit.
//! Re-exports diagnostics, hover, and goto-definition logic.

pub mod diagnostics;
pub mod hover;
pub mod goto_definition;

pub use diagnostics::*;
pub use hover::*;
pub use goto_definition::*;
