#!/usr/bin/env python3
"""
Code generator for maximalist verb stubs and handlers.

Reads ontology/affi-cli.ttl, extracts all verbs and arguments, then generates:
- src/verbs/*.rs (thin wrappers with #[verb] macro)
- src/handlers_stubs.rs (handler function signatures)

Usage: python3 generate_verbs.py
"""

import re
import sys
from pathlib import Path
from collections import defaultdict

def parse_ttl(content: str) -> dict:
    """Parse TTL ontology and extract verbs with their arguments."""
    verbs = {}
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Look for verb declarations: affi:SomeVerb a cnv:Verb
        if ' a cnv:Verb' in line:
            # Extract the verb resource name
            verb_resource = re.match(r'affi:(\w+)\s+a cnv:Verb', line)
            if verb_resource:
                verb_resource_name = verb_resource.group(1)

                # Now collect all properties for this verb until we hit a period followed by blank line
                verb_props = {}
                j = i + 1
                prop_buffer = []

                while j < len(lines):
                    prop_line = lines[j].strip()

                    if not prop_line or prop_line.startswith('#'):
                        if '.' in lines[j - 1]:
                            break
                        j += 1
                        continue

                    prop_buffer.append(prop_line)

                    if '.' in prop_line and not prop_line.endswith(','):
                        # End of this resource
                        break

                    j += 1

                # Parse the collected properties
                props_text = ' '.join(prop_buffer)

                # Extract verb name
                verb_name_match = re.search(r'cnv:hasVerbName\s+"([^"]*)"', props_text)
                if verb_name_match:
                    verb_name_str = verb_name_match.group(1)

                    # Extract about
                    about_match = re.search(r'cnv:verbAbout\s+"([^"]*)"', props_text)
                    about = about_match.group(1) if about_match else 'No description'

                    # Extract argument references
                    args_refs = re.findall(r'affi:(\w+Arg)', props_text)

                    # Parse the argument definitions from the full ontology
                    args = parse_arguments(content, args_refs)

                    verbs[verb_name_str] = {
                        'name': verb_name_str,
                        'about': about,
                        'arguments': args,
                    }

        i += 1

    return verbs

def extract_identifier(line: str) -> str:
    """Extract identifier before 'a cnv:Verb'."""
    match = re.search(r'(affi:\w+)\s+a cnv:Verb', line)
    if match:
        return match.group(1)
    return None

def extract_quoted_string(line: str) -> str:
    """Extract quoted string from a line."""
    match = re.search(r'"([^"]*)"', line)
    if match:
        return match.group(1)
    return None

def parse_arguments(ontology: str, arg_refs: list) -> list:
    """Parse argument definitions from ontology."""
    args = []

    for arg_ref in arg_refs:
        # Find the argument resource in ontology
        arg_pattern = rf'affi:{arg_ref}\s+a cnv:Argument\s*;(.*?)(?:^affi:|^$|\n\n)'
        match = re.search(arg_pattern, ontology, re.MULTILINE | re.DOTALL)

        if match:
            arg_block = match.group(1)

            # Extract name
            name_match = re.search(r'cnv:hasArgumentName\s+"([^"]*)"', arg_block)
            arg_name = name_match.group(1) if name_match else arg_ref

            # Extract type
            type_match = re.search(r'cnv:valueType\s+"([^"]*)"', arg_block)
            arg_type = type_match.group(1) if type_match else "String"

            # Extract required
            req_match = re.search(r'cnv:required\s+"([^"]*)"', arg_block)
            required = req_match.group(1) == "true" if req_match else True

            rust_type = type_to_rust(arg_type, required)
            args.append({
                'name': to_rust_identifier(arg_name),
                'type': rust_type,
                'required': required,
            })

    return args

def to_rust_identifier(name: str) -> str:
    """Convert to Rust identifier."""
    return name.replace('-', '_')

def type_to_rust(typ: str, required: bool) -> str:
    """Map ontology type to Rust type."""
    typ = typ.strip()
    base = {
        'String': 'String',
        'bool': 'bool',
        'Boolean': 'bool',
        'u32': 'u32',
        'u64': 'u64',
        'usize': 'usize',
        'Vec<String>': 'Vec<String>',
    }.get(typ, typ)

    if required:
        return base
    else:
        return f'Option<{base}>'

def generate_verb_wrapper(verb_name: str, verb: dict) -> str:
    """Generate a single verb wrapper."""
    args_list = []
    handler_args = []

    for arg in verb['arguments']:
        args_list.append(f"{arg['name']}: {arg['type']}")
        handler_args.append(arg['name'])

    args_str = ', '.join(args_list)
    handler_args_str = ', '.join(handler_args)

    code = f'''// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt {verb_name}` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// {verb['about']}
#[verb("{verb_name}", "receipt")]
pub fn {to_rust_identifier(verb_name)}({args_str}) -> Result<()> {{
    crate::handlers::{to_rust_identifier(verb_name)}({handler_args_str})
}}
'''
    return code

def generate_handler_stubs(verbs: dict) -> str:
    """Generate handler function stubs."""
    code = '''// Handler stubs for all verbs (auto-generated).
// Implement these to add business logic for each verb.

use anyhow::Result;

'''

    for verb_name, verb in sorted(verbs.items()):
        args_list = []
        for arg in verb['arguments']:
            args_list.append(f"{arg['name']}: {arg['type']}")

        args_str = ', '.join(args_list) if args_list else ''

        code += f'''/// {verb['about']}
pub fn {to_rust_identifier(verb_name)}({args_str}) -> Result<()> {{
    todo!("Implement {verb_name} handler")
}}

'''

    return code

def generate_verbs_mod(verbs: dict) -> str:
    """Generate src/verbs/mod.rs."""
    code = '''// Module declarations for all verbs (auto-generated).
// Each verb is a thin wrapper that delegates to crate::handlers::*.

'''

    for verb_name in sorted(verbs.keys()):
        mod_name = to_rust_identifier(verb_name)
        code += f'pub mod {mod_name};\n'

    return code

def main():
    # Read ontology
    ontology_path = Path('ontology/affi-cli.ttl')
    if not ontology_path.exists():
        print(f"❌ Error: {ontology_path} not found", file=sys.stderr)
        sys.exit(1)

    ontology_content = ontology_path.read_text()

    # Parse verbs
    print("📖 Parsing ontology...")
    verbs = parse_ttl(ontology_content)
    print(f"✅ Found {len(verbs)} verbs")

    # Generate verb wrappers
    print("\n🔨 Generating verb wrappers...")
    verbs_dir = Path('src/verbs')
    verbs_dir.mkdir(exist_ok=True)

    for verb_name, verb in sorted(verbs.items()):
        wrapper = generate_verb_wrapper(verb_name, verb)
        file_name = to_rust_identifier(verb_name) + '.rs'
        file_path = verbs_dir / file_name
        file_path.write_text(wrapper)
        print(f"  ✓ Generated {file_path}")

    # Generate handler stubs
    print("\n🔨 Generating handler stubs...")
    handlers_stub = generate_handler_stubs(verbs)
    handlers_path = Path('src/handlers_stubs.rs')
    handlers_path.write_text(handlers_stub)
    print(f"  ✓ Generated {handlers_path} (reference; merge into src/handlers.rs manually)")

    # Generate verbs mod
    print("\n🔨 Generating src/verbs/mod.rs...")
    mod_content = generate_verbs_mod(verbs)
    mod_path = verbs_dir / 'mod.rs'
    mod_path.write_text(mod_content)
    print(f"  ✓ Generated {mod_path}")

    print(f"\n✨ Success! Generated {len(verbs)} verbs.")
    print("💡 Next: Review generated code and merge handlers into src/handlers.rs")

if __name__ == '__main__':
    main()
