//! Build a [`FeatureSpace`] from a real `Cargo.toml [features]` table.
//!
//! This is what makes confevo *project-agnostic*: instead of a hand-written
//! feature list, it reads the universe and the implication edges straight out of
//! Cargo's own manifest. In a `[features]` table, `a = ["b", "c"]` means "feature
//! `a` enables `b` and `c`" — exactly the implication edges [`FeatureSpace`] wants.
//!
//! ## What counts as an edge
//!
//! A value entry is treated as an intra-crate implication **only** when it names
//! another feature in the same table. The following Cargo value forms are *not*
//! feature→feature edges and are skipped:
//!
//! * `"dep:foo"` — activates an optional dependency, not a feature.
//! * `"foo/bar"` / `"foo?/bar"` — enables feature `bar` of dependency `foo`.
//!
//! ## Scope
//!
//! This is a focused, dependency-free parser for the `[features]` *table only* — it
//! is not a general TOML parser. It handles bare and quoted keys, multi-line
//! arrays, `#` comments, and the value forms above. The `default` feature is
//! excluded from the toggleable universe by default (it is the "do nothing"
//! baseline), with an opt-in to include it.

use std::path::Path;

use crate::space::{FeatureSpace, SpaceError};

/// Error returned when a feature space cannot be built from a manifest.
#[derive(Debug)]
pub enum ManifestError {
    /// The manifest file could not be read.
    Io(std::io::Error),
    /// The manifest has no `[features]` table (nothing to optimize).
    NoFeaturesTable,
    /// The parsed features were rejected when building the space.
    Space(SpaceError),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "reading manifest: {e}"),
            ManifestError::NoFeaturesTable => {
                write!(f, "manifest has no [features] table to optimize")
            }
            ManifestError::Space(e) => write!(f, "building feature space: {e}"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<std::io::Error> for ManifestError {
    fn from(e: std::io::Error) -> Self {
        ManifestError::Io(e)
    }
}

/// Read `path` and build a [`FeatureSpace`] from its `[features]` table.
///
/// `include_default` controls whether the `default` feature is part of the
/// toggleable universe; usually you want `false` so the search explores explicit
/// feature choices rather than the crate's default bundle.
pub fn feature_space_from_cargo_toml(
    path: impl AsRef<Path>,
    include_default: bool,
) -> Result<FeatureSpace, ManifestError> {
    let text = std::fs::read_to_string(path)?;
    feature_space_from_str(&text, include_default)
}

/// Build a [`FeatureSpace`] from the text of a `Cargo.toml`.
///
/// Separated from the file read so it is unit-testable without touching disk.
pub fn feature_space_from_str(
    toml: &str,
    include_default: bool,
) -> Result<FeatureSpace, ManifestError> {
    let entries = parse_features_table(toml).ok_or(ManifestError::NoFeaturesTable)?;
    if entries.is_empty() {
        return Err(ManifestError::NoFeaturesTable);
    }

    // The toggleable universe: every key, optionally minus `default`.
    let universe: Vec<String> = entries
        .iter()
        .map(|(k, _)| k.clone())
        .filter(|k| include_default || k != "default")
        .collect();

    let in_universe: std::collections::BTreeSet<&str> =
        universe.iter().map(String::as_str).collect();

    // Implication edges: value names that are themselves features in the universe.
    let mut implications: Vec<(String, Vec<String>)> = Vec::new();
    for (key, values) in &entries {
        if !in_universe.contains(key.as_str()) {
            continue;
        }
        let mut targets = Vec::new();
        for v in values {
            if let Some(feat) = edge_target(v) {
                if in_universe.contains(feat) {
                    targets.push(feat.to_string());
                }
            }
        }
        if !targets.is_empty() {
            implications.push((key.clone(), targets));
        }
    }

    FeatureSpace::new(universe, implications).map_err(ManifestError::Space)
}

/// Resolve a Cargo feature *value* to the intra-crate feature it enables, if any.
///
/// Returns `None` for optional-dependency activations (`dep:foo`) and cross-crate
/// feature enables (`foo/bar`, `foo?/bar`), which are not feature→feature edges.
fn edge_target(value: &str) -> Option<&str> {
    let v = value.trim();
    if v.is_empty() || v.starts_with("dep:") {
        return None;
    }
    if v.contains('/') {
        // `foo/bar` or `foo?/bar`: enables a feature of dependency `foo`, not a
        // feature of this crate.
        return None;
    }
    Some(v)
}

/// Parse the `[features]` table into `(key, [values])` pairs, preserving order.
///
/// Returns `None` if there is no `[features]` table at all.
fn parse_features_table(toml: &str) -> Option<Vec<(String, Vec<String>)>> {
    // 1. Slice out the [features] section body (comment-stripped, line by line).
    let mut in_section = false;
    let mut body = String::new();
    let mut found = false;
    for raw in toml.lines() {
        let line = strip_comment(raw);
        let trimmed = line.trim();
        if is_table_header(trimmed) {
            in_section = trimmed == "[features]";
            if in_section {
                found = true;
            }
            continue;
        }
        if in_section {
            body.push_str(line);
            body.push('\n');
        }
    }
    if !found {
        return None;
    }

    // 2. Parse `key = [ ... ]` entries from the section body, tolerating arrays
    //    that span multiple lines.
    Some(parse_entries(&body))
}

/// `true` if `trimmed` is a TOML table header like `[features]` or `[deps.x]`.
fn is_table_header(trimmed: &str) -> bool {
    trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.contains('=')
}

/// Remove a `#` comment from a line, respecting `"`-quoted strings.
fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_str = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_str = !in_str,
            b'#' if !in_str => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Parse `key = [values]` entries from comment-stripped section text.
fn parse_entries(body: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    let mut rest = body;

    while let Some(eq) = rest.find('=') {
        // Key is everything before '='.
        let key = unquote(rest[..eq].trim());
        let after = &rest[eq + 1..];

        // Value: an array `[ ... ]` (possibly multi-line) or a scalar to end-of-line.
        let after_trimmed = after.trim_start();
        if let Some(open_rel) = after_trimmed.find('[') {
            // Absolute offset of '[' within `after`.
            let lead = after.len() - after_trimmed.len();
            let open = lead + open_rel;
            if let Some(close) = find_matching_bracket(&after[open..]) {
                let inner = &after[open + 1..open + close];
                if !key.is_empty() {
                    out.push((key, extract_strings(inner)));
                }
                rest = &after[open + close + 1..];
                continue;
            }
        }

        // No array value: skip to next line and keep scanning.
        match after.find('\n') {
            Some(nl) => rest = &after[nl + 1..],
            None => break,
        }
    }

    out
}

/// Given a slice starting at `[`, return the offset of the matching `]`.
fn find_matching_bracket(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_str = !in_str,
            b'[' if !in_str => depth += 1,
            b']' if !in_str => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Extract the quoted string literals from an array body.
fn extract_strings(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            if j <= bytes.len() {
                out.push(inner[start..j].to_string());
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Strip surrounding quotes from a (possibly quoted) bare key.
fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_table_is_an_error() {
        let err = feature_space_from_str("[package]\nname = \"x\"\n", false);
        assert!(matches!(err, Err(ManifestError::NoFeaturesTable)));
    }

    #[test]
    fn parses_simple_table_excluding_default() {
        let toml = r#"
[package]
name = "demo"

[features]
default = ["alloc"]
alloc = []
"#;
        let space = feature_space_from_str(toml, false).unwrap();
        // `default` excluded → only `alloc`.
        assert_eq!(space.features(), &["alloc".to_string()]);
    }

    #[test]
    fn includes_default_when_requested() {
        let toml = "[features]\ndefault = [\"a\"]\na = []\n";
        let space = feature_space_from_str(toml, true).unwrap();
        let mut feats = space.features().to_vec();
        feats.sort();
        assert_eq!(feats, vec!["a".to_string(), "default".to_string()]);
        // default -> a is a real edge once default is in the universe.
        let edges: Vec<&String> = space.implications_of("default").collect();
        assert_eq!(edges, vec![&"a".to_string()]);
    }

    #[test]
    fn captures_feature_to_feature_edges_only() {
        let toml = r#"
[features]
default = []
a = ["b", "dep:somecrate", "tokio/rt", "c?/x"]
b = ["c"]
c = []
"#;
        let space = feature_space_from_str(toml, false).unwrap();
        // a implies b (intra-crate); dep:/slashed forms are ignored.
        let a_edges: Vec<&String> = space.implications_of("a").collect();
        assert_eq!(a_edges, vec![&"b".to_string()]);
        let b_edges: Vec<&String> = space.implications_of("b").collect();
        assert_eq!(b_edges, vec![&"c".to_string()]);
        // Transitive closure works through the parsed edges.
        let closure = space.closure(&["a".to_string()].into_iter().collect());
        let mut got: Vec<String> = closure.into_iter().collect();
        got.sort();
        assert_eq!(got, vec!["a", "b", "c"]);
    }

    #[test]
    fn handles_multiline_arrays_and_comments() {
        let toml = r#"
[features]
# the default bundle
default = ["a"]
a = [
    "b",   # inline comment
    "c",
]
b = []
c = []

[dependencies]
serde = "1"
"#;
        let space = feature_space_from_str(toml, false).unwrap();
        let mut feats = space.features().to_vec();
        feats.sort();
        assert_eq!(feats, vec!["a", "b", "c"]);
        let a_edges: Vec<&String> = space.implications_of("a").collect();
        assert_eq!(a_edges, vec![&"b".to_string(), &"c".to_string()]);
    }

    #[test]
    fn stops_at_next_table_header() {
        let toml = r#"
[features]
a = []

[dependencies]
b = { version = "1", features = ["should-not-be-parsed"] }
"#;
        let space = feature_space_from_str(toml, false).unwrap();
        assert_eq!(space.features(), &["a".to_string()]);
    }

    #[test]
    fn quoted_keys_are_supported() {
        let toml = "[features]\n\"a-b\" = [\"c\"]\nc = []\n";
        let space = feature_space_from_str(toml, false).unwrap();
        assert!(space.contains("a-b"));
        let edges: Vec<&String> = space.implications_of("a-b").collect();
        assert_eq!(edges, vec![&"c".to_string()]);
    }
}
