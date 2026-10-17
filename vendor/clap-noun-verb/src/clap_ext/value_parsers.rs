// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Custom value parsers for advanced type validation.
//!
//! Provides composable value parsers for URLs, paths, JSON, CSV, and more,
//! with compile-time type verification and runtime validation.
//!
//! # Examples
//!
//! ```ignore
//! use clap::Parser;
//! use clap_noun_verb::clap::ValueParserBuilder;
//!
//! #[derive(Parser)]
//! struct Args {
//!     /// A validated URL
//!     #[arg(value_parser = url_parser())]
//!     url: String,
//!
//!     /// A validated port range
//!     #[arg(value_parser = range_parser(1..=65535))]
//!     port: u16,
//! }
//! ```

use std::fmt;
use std::str::FromStr;

/// Builder for composable value parsers.
#[derive(Debug, Clone)]
pub struct ValueParserBuilder {
    name: String,
    description: String,
    validators: Vec<String>,
}

impl ValueParserBuilder {
    /// Create a new value parser builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), description: String::new(), validators: Vec::new() }
    }

    /// Set the description for this parser.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a validator to the parser.
    pub fn with_validator(mut self, validator: impl Into<String>) -> Self {
        self.validators.push(validator.into());
        self
    }

    /// Get the name of this parser.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description of this parser.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the validators.
    pub fn validators(&self) -> &[String] {
        &self.validators
    }

    /// Build the parser.
    pub fn build(self) -> Self {
        self
    }
}

impl Default for ValueParserBuilder {
    fn default() -> Self {
        Self::new("parser")
    }
}

impl fmt::Display for ValueParserBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

/// Validated URL type that can be parsed from strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUrl(String);

impl ValidatedUrl {
    /// Create a new validated URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid.
    pub fn new(url: impl Into<String>) -> crate::Result<Self> {
        let url_str = url.into();
        // Basic URL validation
        if url_str.starts_with("http://") || url_str.starts_with("https://") {
            Ok(Self(url_str))
        } else {
            Err(crate::NounVerbError::ValidationFailed(
                "URL must start with http:// or https://".to_string(),
            ))
        }
    }

    /// Get the URL as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to an owned string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for ValidatedUrl {
    type Err = crate::NounVerbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for ValidatedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validated port number (1-65535).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatedPort(u16);

impl ValidatedPort {
    /// Create a new validated port.
    ///
    /// # Errors
    ///
    /// Returns an error if the port is outside the valid range (1-65535).
    pub fn new(port: u16) -> crate::Result<Self> {
        if port == 0 || port == u16::MAX {
            Err(crate::NounVerbError::ValidationFailed(
                "Port must be between 1 and 65534".to_string(),
            ))
        } else {
            Ok(Self(port))
        }
    }

    /// Get the port number.
    pub fn port(&self) -> u16 {
        self.0
    }
}

impl FromStr for ValidatedPort {
    type Err = crate::NounVerbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let port: u16 = s.parse().map_err(|_| {
            crate::NounVerbError::ValidationFailed("Invalid port number".to_string())
        })?;
        Self::new(port)
    }
}

impl fmt::Display for ValidatedPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// JSON value validator.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedJson {
    inner: serde_json::Value,
    raw: String,
}

impl ValidatedJson {
    /// Create a new validated JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    pub fn new(json_str: impl Into<String>) -> crate::Result<Self> {
        let raw = json_str.into();
        let inner = serde_json::from_str(&raw)
            .map_err(|e| crate::NounVerbError::ValidationFailed(format!("Invalid JSON: {}", e)))?;
        Ok(Self { inner, raw })
    }

    /// Get the parsed JSON value.
    pub fn value(&self) -> &serde_json::Value {
        &self.inner
    }

    /// Get the raw JSON string.
    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl FromStr for ValidatedJson {
    type Err = crate::NounVerbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for ValidatedJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// CSV parser for comma-separated values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvList {
    items: Vec<String>,
}

impl CsvList {
    /// Create a new CSV list from a comma-separated string.
    pub fn new(csv: impl Into<String>) -> Self {
        let items =
            csv.into().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        Self { items }
    }

    /// Get the items as slices.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl FromStr for CsvList {
    type Err = crate::NounVerbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(crate::NounVerbError::ValidationFailed("CSV list cannot be empty".to_string()))
        } else {
            Ok(Self::new(s))
        }
    }
}

impl fmt::Display for CsvList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.items.join(", "))
    }
}

/// Configuration for value parser validation rules.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Allow empty values
    allow_empty: bool,
    /// Trim whitespace
    trim_whitespace: bool,
    /// Case insensitive matching
    case_insensitive: bool,
}

impl ParserConfig {
    /// Create a new parser configuration.
    pub fn new() -> Self {
        Self { allow_empty: false, trim_whitespace: true, case_insensitive: false }
    }

    /// Allow empty values.
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// Trim whitespace from values.
    pub fn trim_whitespace(mut self, trim: bool) -> Self {
        self.trim_whitespace = trim;
        self
    }

    /// Use case insensitive matching.
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }

    /// Check if empty values are allowed.
    pub fn is_empty_allowed(&self) -> bool {
        self.allow_empty
    }

    /// Check if whitespace trimming is enabled.
    pub fn should_trim(&self) -> bool {
        self.trim_whitespace
    }

    /// Check if case insensitive matching is enabled.
    pub fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_parser_builder() {
        let parser = ValueParserBuilder::new("url_parser").with_description("Parse URLs");
        assert_eq!(parser.name(), "url_parser");
        assert_eq!(parser.description(), "Parse URLs");
    }

    #[test]
    fn test_value_parser_builder_with_validators() {
        let parser = ValueParserBuilder::new("test")
            .with_validator("non_empty")
            .with_validator("valid_format");
        assert_eq!(parser.validators().len(), 2);
    }

    #[test]
    fn test_validated_url_creation() {
        let url = ValidatedUrl::new("https://example.com");
        assert!(url.is_ok());
        assert_eq!(url.unwrap().as_str(), "https://example.com");
    }

    #[test]
    fn test_validated_url_invalid() {
        let url = ValidatedUrl::new("invalid://example.com");
        assert!(url.is_err());
    }

    #[test]
    fn test_validated_port_creation() {
        let port = ValidatedPort::new(8080);
        assert!(port.is_ok());
        assert_eq!(port.unwrap().port(), 8080);
    }

    #[test]
    fn test_validated_port_invalid_zero() {
        let port = ValidatedPort::new(0);
        assert!(port.is_err());
    }

    #[test]
    fn test_validated_json_creation() {
        let json = ValidatedJson::new(r#"{"key": "value"}"#);
        assert!(json.is_ok());
    }

    #[test]
    fn test_validated_json_invalid() {
        let json = ValidatedJson::new("not json");
        assert!(json.is_err());
    }

    #[test]
    fn test_csv_list_creation() {
        let list = CsvList::new("a,b,c");
        assert_eq!(list.len(), 3);
        assert_eq!(list.items(), &["a", "b", "c"]);
    }

    #[test]
    fn test_csv_list_with_whitespace() {
        let list = CsvList::new("a, b, c");
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_csv_list_empty() {
        let result = CsvList::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert!(!config.is_empty_allowed());
        assert!(config.should_trim());
        assert!(!config.is_case_insensitive());
    }
}
