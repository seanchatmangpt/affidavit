/// Code generator for maximalist verb stubs and handlers.
///
/// Reads ontology/affi-cli.ttl, extracts all verbs and arguments, then generates:
/// - src/verbs/*.rs (thin wrappers with #[verb] macro)
/// - src/handlers.rs stubs (handler function signatures)
///
/// Usage: cargo run --example codegen_maximalist_verbs

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let ontology_path = "ontology/affi-cli.ttl";
    let ontology = fs::read_to_string(ontology_path)?;

    // Parse verbs and their arguments from the TTL.
    let verbs = parse_verbs(&ontology)?;

    // Generate verb wrappers in src/verbs/*.rs
    generate_verb_wrappers(&verbs)?;

    // Generate handlers stub
    generate_handlers_stub(&verbs)?;

    // Update src/verbs/mod.rs with all verb modules
    generate_verbs_mod(&verbs)?;

    println!("✅ Generated {} verb wrappers and handlers", verbs.len());
    Ok(())
}

#[derive(Debug, Clone)]
struct Verb {
    name: String,
    about: String,
    arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
struct Argument {
    name: String,
    rust_type: String,
    required: bool,
}

fn parse_verbs(ontology: &str) -> anyhow::Result<BTreeMap<String, Verb>> {
    let mut verbs = BTreeMap::new();

    // Simple line-by-line parser for TTL. In production, use a proper RDF parser.
    // This is a pragmatic parser that handles our specific ontology structure.

    let lines: Vec<&str> = ontology.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.contains("a cnv:Verb") {
            // Found a verb declaration. Extract verb name.
            if let Some(verb_name) = extract_verb_name(&lines, i) {
                let about = extract_field(&lines, i, "cnv:verbAbout");
                let mut args = Vec::new();

                // Extract arguments from cnv:hasArguments
                if let Some(args_section) = extract_arguments(&lines, i, ontology) {
                    args = parse_arguments(&lines, &args_section, ontology);
                }

                verbs.insert(
                    verb_name.clone(),
                    Verb {
                        name: verb_name,
                        about: about.unwrap_or_default(),
                        arguments: args,
                    },
                );
            }
        }

        i += 1;
    }

    Ok(verbs)
}

fn extract_verb_name(lines: &[&str], start_idx: usize) -> Option<String> {
    for i in (start_idx.saturating_sub(10))..start_idx {
        if let Some(name) = extract_field_at_line(lines[i], "cnv:hasVerbName") {
            return Some(
                name
                    .trim_matches(|c: char| c.is_whitespace() || c == '"' || c == ';')
                    .to_string(),
            );
        }
    }
    None
}

fn extract_field(lines: &[&str], start_idx: usize, field: &str) -> Option<String> {
    for i in start_idx..std::cmp::min(start_idx + 20, lines.len()) {
        if let Some(val) = extract_field_at_line(lines[i], field) {
            return Some(val);
        }
    }
    None
}

fn extract_field_at_line(line: &str, field: &str) -> Option<String> {
    if let Some(pos) = line.find(field) {
        let rest = &line[pos + field.len()..];
        if let Some(start) = rest.find('"') {
            if let Some(end) = rest[start + 1..].find('"') {
                return Some(rest[start + 1..start + 1 + end].to_string());
            }
        }
    }
    None
}

fn extract_arguments(lines: &[&str], start_idx: usize, _ontology: &str) -> Option<Vec<String>> {
    for i in start_idx..std::cmp::min(start_idx + 10, lines.len()) {
        if lines[i].contains("cnv:hasArguments") {
            let mut args = Vec::new();
            let line = lines[i];

            // Simple parsing: extract all affi:*Arg references
            for word in line.split_whitespace() {
                if word.starts_with("affi:") && word.ends_with("Arg") || word.ends_with(",") {
                    let arg_name = word
                        .trim_end_matches(',')
                        .trim_end_matches('.')
                        .trim_start_matches("affi:");
                    if !arg_name.is_empty() && arg_name.ends_with("Arg") {
                        args.push(arg_name.to_string());
                    }
                }
            }

            if !args.is_empty() {
                return Some(args);
            }
        }
    }
    None
}

fn parse_arguments(
    lines: &[&str],
    arg_names: &[String],
    ontology: &str,
) -> Vec<Argument> {
    let mut arguments = Vec::new();

    for arg_name in arg_names {
        // Find the argument definition in the ontology
        if let Some(idx) = lines.iter().position(|l| l.contains(&format!("{} a cnv:Argument", arg_name))) {
            let arg_line_start = idx;
            let rust_name = to_rust_identifier(
                extract_field(lines, arg_line_start, "cnv:hasArgumentName")
                    .unwrap_or_default()
                    .as_str(),
            );
            let value_type = extract_field(lines, arg_line_start, "cnv:valueType")
                .unwrap_or_else(|| "String".to_string());
            let required = extract_field(lines, arg_line_start, "cnv:required")
                .map(|v| v.contains("true"))
                .unwrap_or(true);

            let rust_type = type_to_rust(&value_type, required);

            arguments.push(Argument {
                name: rust_name,
                rust_type,
                required,
            });
        }
    }

    arguments
}

fn to_rust_identifier(name: &str) -> String {
    name.replace('-', "_")
}

fn type_to_rust(typ: &str, required: bool) -> String {
    let base = match typ.trim() {
        "String" => "String".to_string(),
        "bool" | "Boolean" => "bool".to_string(),
        "u32" | "u64" => typ.to_string(),
        "usize" => "usize".to_string(),
        "Vec<String>" => "Vec<String>".to_string(),
        other => other.to_string(),
    };

    if required {
        base
    } else {
        format!("Option<{}>", base)
    }
}

fn generate_verb_wrappers(verbs: &BTreeMap<String, Verb>) -> anyhow::Result<()> {
    for (verb_name, verb) in verbs {
        let mut arg_list = String::new();
        let mut handler_args = String::new();

        for arg in &verb.arguments {
            let param = format!("{}: {}", arg.name, arg.rust_type);
            arg_list.push_str(&param);
            arg_list.push_str(", ");
            handler_args.push_str(&format!("{}, ", arg.name));
        }

        // Remove trailing ", "
        arg_list = arg_list.trim_end_matches(", ").to_string();
        handler_args = handler_args.trim_end_matches(", ").to_string();

        let code = format!(
            r#"// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated by codegen_maximalist_verbs. The pack is
// authoritative for the CLI *interface* only; the body delegates to a stable
// consumer-implemented handler.

//! `receipt {}` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// {}
#[verb("{}", "receipt")]
pub fn {}({}) -> Result<()> {{
    crate::handlers::{}({})
}}
"#,
            verb_name, verb.about, verb_name, verb_name, arg_list, verb_name, handler_args
        );

        let path = format!("src/verbs/{}.rs", verb_name.replace('-', "_"));
        fs::write(&path, code)?;
        println!("  Generated {}", path);
    }

    Ok(())
}

fn generate_handlers_stub(verbs: &BTreeMap<String, Verb>) -> anyhow::Result<()> {
    let mut handler_code = r#"// Handlers for all verbs (auto-generated stubs).
// Implement these to add business logic for each verb.

use crate::types::Receipt;
use anyhow::Result;

"#
        .to_string();

    for (verb_name, verb) in verbs {
        let mut args = String::new();
        for arg in &verb.arguments {
            args.push_str(&format!("{}: {}, ", arg.name, arg.rust_type));
        }
        args = args.trim_end_matches(", ").to_string();

        handler_code.push_str(&format!(
            "/// {}\npub fn {}({}) -> Result<()> {{\n    todo!(\"Implement {} handler\")\n}}\n\n",
            verb.about, verb_name.replace('-', "_"), args, verb_name
        ));
    }

    // Append to src/handlers.rs (or create if missing)
    let path = "src/handlers.rs";
    if Path::new(path).exists() {
        println!(
            "  ℹ️  handlers.rs already exists. Stubs generated to STDERR for reference:"
        );
        eprintln!("{}", handler_code);
    } else {
        fs::write(path, handler_code)?;
        println!("  Generated {}", path);
    }

    Ok(())
}

fn generate_verbs_mod(verbs: &BTreeMap<String, Verb>) -> anyhow::Result<()> {
    let mut mod_code = r#"// Module declarations for all verbs (auto-generated).
// Each verb is a thin wrapper that delegates to crate::handlers::*.

"#
        .to_string();

    for verb_name in verbs.keys() {
        mod_code.push_str(&format!("pub mod {};\n", verb_name.replace('-', "_")));
    }

    fs::write("src/verbs/mod.rs", mod_code)?;
    println!("  Generated src/verbs/mod.rs");

    Ok(())
}
