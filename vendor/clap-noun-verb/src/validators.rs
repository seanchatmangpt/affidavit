// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Common value validators for CLI arguments
//!
//! This module provides reusable validators for common data types.
//!
//! # Example
//!
//! ```rust
//! use clap_noun_verb::validators::validate_port;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let port = 8080;
//! validate_port(port)?;
//! println!("Valid port: {}", port);
//! # Ok(())
//! # }
//! ```

use crate::Result;
use regex::Regex;
use std::path::Path;
use url::Url;

/// Validate that a port number is in valid range (1-65535)
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_port;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_port(8080)?;  // OK
/// assert!(validate_port(0).is_err()); // Error
/// # Ok(())
/// # }
/// ```
pub fn validate_port(port: u16) -> Result<()> {
    if port == 0 {
        return Err(crate::error::NounVerbError::execution_error(
            "Port must be between 1 and 65535",
        ));
    }
    Ok(())
}

/// Validate that a string is a valid URL
///
/// Accepts any URL scheme (http, https, ftp, etc.) and validates
/// the format according to RFC 3986. Both absolute and relative URLs
/// may be accepted depending on the parser.
///
/// # Errors
///
/// Returns an error if the URL cannot be parsed or is invalid
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_url;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_url("https://example.com")?;      // OK
/// validate_url("http://localhost:8080")?;    // OK
/// validate_url("ftp://files.example.org")?;  // OK
/// assert!(validate_url("not a url").is_err()); // Error
/// # Ok(())
/// # }
/// ```
pub fn validate_url(url_str: &str) -> Result<()> {
    Url::parse(url_str)
        .map_err(|e| crate::error::NounVerbError::execution_error(format!("Invalid URL: {}", e)))?;
    Ok(())
}

/// Validate that a string is a valid IPv4 address
///
/// Checks that the string contains exactly 4 octets (0-255) separated by dots.
/// Dotted decimal notation is the standard format for IPv4 addresses.
///
/// # Errors
///
/// Returns an error if the format is invalid or any octet is out of range
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_ipv4;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_ipv4("192.168.1.1")?;      // OK - private network
/// validate_ipv4("127.0.0.1")?;        // OK - loopback
/// validate_ipv4("255.255.255.255")?;  // OK - broadcast
/// assert!(validate_ipv4("999.999.999.999").is_err());  // Error - octets out of range
/// assert!(validate_ipv4("192.168.1").is_err());        // Error - only 3 octets
/// # Ok(())
/// # }
/// ```
pub fn validate_ipv4(ip: &str) -> Result<()> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return Err(crate::error::NounVerbError::execution_error("IPv4 must have 4 octets"));
    }

    for part in parts {
        match part.parse::<u8>() {
            Ok(_) => continue,
            Err(_) => {
                return Err(crate::error::NounVerbError::execution_error(
                    "Each IPv4 octet must be 0-255",
                ))
            }
        }
    }
    Ok(())
}

/// Validate that a string is a valid IPv6 address
///
/// Accepts standard IPv6 notation including compressed form (::) and full form.
/// Uses the standard library's IPv6Addr parser for validation.
///
/// # Errors
///
/// Returns an error if the address is not a valid IPv6 format
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_ipv6;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_ipv6("2001:db8::1")?;                // OK - compressed
/// validate_ipv6("::1")?;                        // OK - loopback
/// validate_ipv6("fe80::1")?;                    // OK - link-local
/// validate_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334")?; // OK - full
/// assert!(validate_ipv6("gggg::1").is_err());                    // Error - invalid characters
/// # Ok(())
/// # }
/// ```
pub fn validate_ipv6(ip: &str) -> Result<()> {
    ip.parse::<std::net::Ipv6Addr>()
        .map_err(|_| crate::error::NounVerbError::execution_error("Invalid IPv6 address"))?;
    Ok(())
}

/// Validate that a file or directory path exists
///
/// Checks if the given path exists in the filesystem. This is useful for
/// validating that a user-provided file or directory actually exists before
/// attempting to read from or write to it.
///
/// # Errors
///
/// Returns an error if the path does not exist or if access cannot be checked
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_path_exists;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_path_exists("Cargo.toml")?;      // OK - file exists
/// validate_path_exists("src")?;             // OK - directory exists
/// assert!(validate_path_exists("nonexistent").is_err());    // Error - path doesn't exist
/// assert!(validate_path_exists("missing.txt").is_err()); // Error - file doesn't exist
/// # Ok(())
/// # }
/// ```
pub fn validate_path_exists(path_str: &str) -> Result<()> {
    if !Path::new(path_str).exists() {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "Path does not exist: {}",
            path_str
        )));
    }
    Ok(())
}

/// Validate that a path can be created (parent directory exists)
///
/// Checks that the parent directory of the given path exists, which means
/// the file or directory at the path can be created. This is useful for
/// validating output file paths before attempting to write to them.
///
/// # Errors
///
/// Returns an error if the parent directory doesn't exist
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_path_creatable;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_path_creatable("Cargo.toml")?;    // OK - parent (current dir) exists
/// validate_path_creatable("src/newfile.txt")?; // OK - src exists
/// assert!(validate_path_creatable("nonexistent_dir/file.txt").is_err()); // Error - parent doesn't exist
/// # Ok(())
/// # }
/// ```
pub fn validate_path_creatable(path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(crate::error::NounVerbError::execution_error(format!(
                "Parent directory does not exist for: {}",
                path_str
            )));
        }
    }
    Ok(())
}

/// Validate email address format using a regex pattern
///
/// Uses a simplified RFC 5322 regex to validate email addresses. This provides
/// basic validation but is not a complete implementation of the RFC.
/// For strict RFC 5322 validation, consider additional verification.
///
/// # Errors
///
/// Returns an error if the email format is invalid
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_email;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_email("user@example.com")?;         // OK
/// validate_email("john.doe+tag@domain.co.uk")?; // OK
/// validate_email("admin@localhost")?;          // OK
/// assert!(validate_email("invalid").is_err());                  // Error
/// assert!(validate_email("user@").is_err());                    // Error
/// assert!(validate_email("@domain.com").is_err());              // Error
/// # Ok(())
/// # }
/// ```
pub fn validate_email(email: &str) -> Result<()> {
    // Simple email validation regex (RFC 5322 simplified)
    let email_regex = Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$")
        .map_err(|_| crate::error::NounVerbError::execution_error("Email regex error"))?;

    if !email_regex.is_match(email) {
        return Err(crate::error::NounVerbError::execution_error("Invalid email format"));
    }
    Ok(())
}

/// Validate that a value is not empty or whitespace-only
///
/// Checks that the string contains at least one non-whitespace character.
/// Whitespace-only strings (spaces, tabs, newlines) are considered empty.
///
/// # Errors
///
/// Returns an error if the value is empty or contains only whitespace
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_not_empty;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_not_empty("hello")?;        // OK
/// validate_not_empty("   hello   ")?;  // OK - contains non-whitespace
/// assert!(validate_not_empty("").is_err());             // Error - empty string
/// assert!(validate_not_empty("   ").is_err());          // Error - whitespace only
/// # Ok(())
/// # }
/// ```
pub fn validate_not_empty(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(crate::error::NounVerbError::execution_error("Value cannot be empty"));
    }
    Ok(())
}

/// Validate that a string's length is within a specified range
///
/// Checks that the string length (in bytes, not characters) falls between
/// the specified minimum and maximum bounds (inclusive).
///
/// # Arguments
///
/// * `value` - The string to validate
/// * `min` - Minimum length (inclusive)
/// * `max` - Maximum length (inclusive)
///
/// # Errors
///
/// Returns an error if the length is less than min or greater than max
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_length;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// validate_length("hello", 1, 10)?;        // OK - 5 bytes, in range [1, 10]
/// assert!(validate_length("hi", 3, 10).is_err());           // Error - 2 bytes, less than min (3)
/// assert!(validate_length("verylongstring", 1, 5).is_err()); // Error - 14 bytes, more than max (5)
/// # Ok(())
/// # }
/// ```
pub fn validate_length(value: &str, min: usize, max: usize) -> Result<()> {
    let len = value.len();
    if len < min {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "Value must be at least {} characters",
            min
        )));
    }
    if len > max {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "Value must be at most {} characters",
            max
        )));
    }
    Ok(())
}

/// Validate that a value matches a regex pattern
///
/// Checks that the entire value matches the provided regex pattern.
/// The pattern uses standard Rust regex syntax.
///
/// # Arguments
///
/// * `value` - The string to validate
/// * `pattern` - A regex pattern string (must be a valid regex)
///
/// # Errors
///
/// Returns an error if the value doesn't match the pattern or if the pattern is invalid
///
/// # Example
///
/// ```rust
/// # use clap_noun_verb::validators::validate_regex;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Match alphanumeric strings only
/// validate_regex("hello123", r"^[a-zA-Z0-9]+$")?;  // OK
/// assert!(validate_regex("hello!", r"^[a-zA-Z0-9]+$").is_err());    // Error - contains !
///
/// // Match email pattern
/// validate_regex("user@example.com", r"^[\w\.-]+@[\w\.-]+\.\w+$")?; // OK
/// # Ok(())
/// # }
/// ```
pub fn validate_regex(value: &str, pattern: &str) -> Result<()> {
    let regex = Regex::new(pattern).map_err(|e| {
        crate::error::NounVerbError::execution_error(format!("Invalid regex: {}", e))
    })?;

    if !regex.is_match(value) {
        return Err(crate::error::NounVerbError::execution_error(format!(
            "Value does not match pattern: {}",
            pattern
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(1).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("127.0.0.1").is_ok());
        assert!(validate_ipv4("999.999.999.999").is_err());
        assert!(validate_ipv4("192.168.1").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello").is_ok());
        assert!(validate_not_empty("").is_err());
        assert!(validate_not_empty("   ").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", 1, 10).is_ok());
        assert!(validate_length("hello", 5, 10).is_ok());
        assert!(validate_length("hello", 6, 10).is_err());
        assert!(validate_length("hello", 1, 4).is_err());
    }
}
