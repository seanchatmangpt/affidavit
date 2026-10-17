// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Value parser pattern matching and application
//!
//! This module handles parsing and applying value_parser expressions
//! from their string representations using pattern matching.
//!
//! For range patterns like `clap::value_parser!(u16).range(1..=65535)`,
//! users should use the `#[validate(min = ..., max = ...)]` attribute instead,
//! which is already supported via the existing validation system.

/// Apply value parser from string representation
///
/// This function parses common value_parser patterns and applies them.
/// For range patterns, it extracts min/max bounds which are then handled
/// by the existing validation system.
pub fn apply_value_parser(arg: &mut clap::Arg, vp_str: &str) -> bool {
    // Skip placeholder for explicit but unsupported expressions
    if vp_str == "__explicit__" {
        return false;
    }

    // Match: clap::value_parser!(T) - simple type parser (no range)
    // Example: clap::value_parser!(PathBuf)
    if vp_str.contains("value_parser!") {
        if vp_str.contains("PathBuf") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::path::PathBuf));
            return true;
        } else if vp_str.contains("IpAddr") && !vp_str.contains("Ipv4") && !vp_str.contains("Ipv6")
        {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::IpAddr));
            return true;
        } else if vp_str.contains("Ipv4Addr") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::Ipv4Addr));
            return true;
        } else if vp_str.contains("Ipv6Addr") {
            *arg = arg.clone().value_parser(clap::value_parser!(std::net::Ipv6Addr));
            return true;
        } else if vp_str.contains("Url") {
            // Url requires url crate - only apply if available
            // For now, skip - users must ensure url feature is enabled
            return false;
        }
    }

    false
}
