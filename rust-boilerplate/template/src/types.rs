//! Core domain types for {{project-name}}.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A BLAKE3 digest rendered as a lowercase hex string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Blake3Hash(pub String);

impl Blake3Hash {
    /// Construct from a pre-computed lowercase hex string.
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Blake3Hash(hex.into())
    }

    /// Borrow the hex representation.
    pub fn as_hex(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Blake3Hash {
    fn from(s: String) -> Self {
        Blake3Hash(s)
    }
}

impl From<Blake3Hash> for String {
    fn from(h: Blake3Hash) -> Self {
        h.0
    }
}

/// A qualified reference to an object within an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    /// Stable identifier of the referenced object.
    pub id: String,
    /// Object type (the class of the object).
    #[serde(rename = "type")]
    pub type_: String,
}
