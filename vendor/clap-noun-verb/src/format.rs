// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Output formatting system
//!
//! This module provides pluggable output formatters for different output formats.
//! The default is JSON, but YAML, Table, Plain, and TSV formats are also supported.
//!
//! # Example
//!
//! ```rust
//! use clap_noun_verb::format::{OutputFormat, format_output};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Output {
//!     name: String,
//!     value: u32,
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = Output { name: "example".to_string(), value: 42 };
//!
//! // Method style
//! let _formatted = OutputFormat::Table.format(&output)?;
//!
//! // Function style
//! let formatted = format_output(&output, OutputFormat::Yaml)?;
//! println!("{}", formatted);
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

/// Type alias for output validation hooks.
pub type OutputValidationHook = Arc<
    dyn Fn(&serde_json::Value) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync,
>;

static VALIDATION_HOOKS: OnceLock<RwLock<Vec<OutputValidationHook>>> = OnceLock::new();

/// Register a global hook to validate formatted output before it is rendered or returned.
///
/// This allows enabling output schemas, checking serialization bounds (e.g., maximum length,
/// depth, disallowed keys), or enforcing security policies.
pub fn register_output_validation_hook<F>(hook: F)
where
    F: Fn(&serde_json::Value) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
{
    let hooks = VALIDATION_HOOKS.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut write_guard) = hooks.write() {
        write_guard.push(Arc::new(hook));
    }
}

/// Clear all registered output validation hooks.
pub fn clear_output_validation_hooks() {
    let hooks = VALIDATION_HOOKS.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(mut write_guard) = hooks.write() {
        write_guard.clear();
    }
}

/// Run all registered validation hooks on a serialized JSON value.
fn validate_output_value(
    value: &serde_json::Value,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let hooks = VALIDATION_HOOKS.get_or_init(|| RwLock::new(Vec::new()));
    if let Ok(read_guard) = hooks.read() {
        for hook in read_guard.iter() {
            hook(value).map_err(|e| {
                Box::new(crate::error::NounVerbError::execution_error(format!(
                    "Output validation failed: {}",
                    e
                ))) as Box<dyn std::error::Error>
            })?;
        }
    }
    Ok(())
}

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Compact JSON
    Json,
    /// Pretty-printed JSON (default)
    #[default]
    JsonPretty,
    /// YAML format (built-in, no external deps)
    Yaml,
    /// Pretty printed table format
    Table,
    /// Plain text (key: value pairs)
    Plain,
    /// Tab-separated values
    Tsv,
    /// Quiet mode (silence output)
    Quiet,
}

impl OutputFormat {
    /// Format a serializable value
    pub fn format<S: Serialize>(self, value: &S) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_value(value)?;
        validate_output_value(&json)?;
        let output = match self {
            OutputFormat::Json => serde_json::to_string(&json)?,
            OutputFormat::JsonPretty => serde_json::to_string_pretty(&json)?,
            OutputFormat::Yaml => json_to_yaml(&json, 0),
            OutputFormat::Table => json_to_table(&json),
            OutputFormat::Plain => json_to_plain(&json),
            OutputFormat::Tsv => json_to_tsv(&json),
            OutputFormat::Quiet => String::new(),
        };
        Ok(output)
    }

    /// Get all available format names
    pub fn available_formats() -> &'static [&'static str] {
        &["json", "json-pretty", "yaml", "table", "plain", "tsv", "quiet"]
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Json => "Compact JSON (machine-readable)",
            Self::JsonPretty => "Pretty-printed JSON (human-readable)",
            Self::Yaml => "YAML format",
            Self::Table => "ASCII table format",
            Self::Plain => "Plain text (key: value)",
            Self::Tsv => "Tab-separated values",
            Self::Quiet => "Silence output during automation/pipelines",
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::JsonPretty => write!(f, "json-pretty"),
            Self::Yaml => write!(f, "yaml"),
            Self::Table => write!(f, "table"),
            Self::Plain => write!(f, "plain"),
            Self::Tsv => write!(f, "tsv"),
            Self::Quiet => write!(f, "quiet"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "json-pretty" | "jsonpretty" | "pretty" => Ok(OutputFormat::JsonPretty),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "table" | "ascii" => Ok(OutputFormat::Table),
            "plain" | "text" => Ok(OutputFormat::Plain),
            "tsv" | "tab" => Ok(OutputFormat::Tsv),
            "quiet" | "silent" => Ok(OutputFormat::Quiet),
            _ => Err(format!(
                "Unknown format '{}'. Available: {:?}",
                s,
                OutputFormat::available_formats()
            )),
        }
    }
}

/// Format a serializable value using the specified output format.
///
/// Convenience function for commands that need to format output.
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::format::{format_output, OutputFormat};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = vec![("a", "b"), ("c", "d")];
/// println!("{}", format_output(&data, OutputFormat::Table)?);
/// # Ok(())
/// # }
/// ```
pub fn format_output<T: Serialize>(
    data: &T,
    format: OutputFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    format.format(data)
}

/// JSON formatter (compact)
fn format_json<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(serde_json::to_string(&json)?)
}

/// JSON formatter (pretty-printed)
fn format_json_pretty<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(serde_json::to_string_pretty(&json)?)
}

/// YAML formatter (built-in, no external deps)
fn format_yaml<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(json_to_yaml(&json, 0))
}

/// Table formatter - converts JSON to ASCII table
fn format_table<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(json_to_table(&json))
}

/// TSV formatter - converts JSON to tab-separated values
fn format_tsv<S: Serialize>(value: &S) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_value(value)?;
    Ok(json_to_tsv(&json))
}

/// Helper: Build formatted table/TSV rows from JSON object array
///
/// This generic helper reduces duplication between table and TSV formatting.
/// The formatter closure controls how each cell value is converted to a string.
fn format_object_array<F>(arr: &[serde_json::Value], formatter: F) -> String
where
    F: Fn(&serde_json::Value, &str) -> String, // (value, key) -> formatted_cell
{
    if arr.is_empty() {
        return String::new();
    }

    let first = &arr[0];
    if let serde_json::Value::Object(obj) = first {
        let mut output = String::new();
        let keys: Vec<&String> = obj.keys().collect();

        // Header row - avoid Vec allocation, build directly
        for (i, k) in keys.iter().enumerate() {
            if i > 0 {
                output.push('\t');
            }
            output.push_str(k.as_str());
        }
        output.push('\n');

        // Data rows - identical structure for both table and TSV
        for item in arr {
            if let serde_json::Value::Object(item_obj) = item {
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        output.push('\t');
                    }
                    let cell_value = formatter(
                        item_obj.get(k.as_str()).unwrap_or(&serde_json::Value::Null),
                        k.as_str(),
                    );
                    output.push_str(&cell_value);
                }
                output.push('\n');
            }
        }
        output
    } else {
        // Fallback for non-object arrays
        let mut result = String::new();
        for (i, item) in arr.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&item.to_string());
        }
        result
    }
}

/// Helper: Convert JSON value to ASCII table
fn json_to_table(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return "(empty)".to_string();
            }

            // Table formatter: show "-" for missing values
            format_object_array(arr, |val, _key| match val {
                serde_json::Value::Null => "-".to_string(),
                other => other.to_string(),
            })
        }
        serde_json::Value::Object(obj) => {
            let mut output = String::new();
            for (k, v) in obj {
                output.push_str(&format!("{}\t{}\n", k, v));
            }
            output
        }
        other => other.to_string(),
    }
}

/// Helper: Convert JSON value to YAML-like format (built-in, no serde_yaml dep)
fn json_to_yaml(value: &serde_json::Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                arr.iter()
                    .map(|v| format!("{}- {}", prefix, json_to_yaml(v, indent + 1)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| {
                if v.is_object() || v.is_array() {
                    format!("{}{}:\n{}", prefix, k, json_to_yaml(v, indent + 1))
                } else {
                    format!("{}{}: {}", prefix, k, json_to_yaml(v, indent))
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// Plain text formatter - key: value pairs
fn json_to_plain(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter().map(json_to_plain).collect::<Vec<_>>().join("\n")
        }
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{}: {}", k, json_to_plain(v)))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => value.to_string(),
    }
}

/// Helper: Convert JSON to TSV
fn json_to_tsv(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return String::new();
            }

            // TSV formatter: escape special characters
            format_object_array(arr, |val, _key| escape_tsv(&val.to_string()))
        }
        serde_json::Value::Object(obj) => {
            let mut output = String::new();
            for (k, v) in obj {
                output.push_str(&format!("{}\t{}\n", k, escape_tsv(&v.to_string())));
            }
            output
        }
        other => other.to_string(),
    }
}

/// Helper: Escape TSV special characters
fn escape_tsv(s: &str) -> String {
    if s.contains('\t') || s.contains('\n') || s.contains('\\') {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Yaml.to_string(), "yaml");
        assert_eq!(OutputFormat::Quiet.to_string(), "quiet");
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml);
        assert_eq!("quiet".parse::<OutputFormat>().unwrap(), OutputFormat::Quiet);
        assert_eq!("silent".parse::<OutputFormat>().unwrap(), OutputFormat::Quiet);
        assert!("invalid".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_available_formats() {
        let formats = OutputFormat::available_formats();
        assert!(formats.contains(&"json"));
        assert!(formats.contains(&"json-pretty"));
        assert!(formats.contains(&"yaml"));
        assert!(formats.contains(&"table"));
        assert!(formats.contains(&"plain"));
        assert!(formats.contains(&"tsv"));
        assert!(formats.contains(&"quiet"));
        assert_eq!(formats.len(), 7);
    }

    #[test]
    fn test_quiet_format() {
        let data = vec![("a", "b")];
        let formatted = OutputFormat::Quiet.format(&data).unwrap();
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_output_validation_hooks() {
        clear_output_validation_hooks();

        // Register a hook checking serialization bounds (e.g. key must not be 'blocked')
        register_output_validation_hook(|val| {
            if let Some(obj) = val.as_object() {
                if obj.contains_key("blocked") {
                    return Err("Found blocked key".into());
                }
            }
            Ok(())
        });

        // Test valid data
        let valid_data = serde_json::json!({ "allowed": "value" });
        assert!(OutputFormat::Json.format(&valid_data).is_ok());

        // Test invalid data
        let invalid_data = serde_json::json!({ "blocked": "value" });
        let result = OutputFormat::Json.format(&invalid_data);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Output validation failed: Found blocked key"));

        // Clear hooks and test again
        clear_output_validation_hooks();
        assert!(OutputFormat::Json.format(&invalid_data).is_ok());
    }
}
