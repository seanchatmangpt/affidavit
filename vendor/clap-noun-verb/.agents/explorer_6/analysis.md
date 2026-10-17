# Bug Analysis and Fix Strategies Report

This report documents the analysis and proposed fix strategies for four identified bugs within the `clap-noun-verb-utils` project.

---

## 1. Configuration Overriding in `LayeredConfigAdapter`

### Problem Statement
Currently, `LayeredConfigAdapter::resolve` deserializes CLI arguments using `arg_matches_to_json` and merges them on top of the configuration file and environment variables. However, `arg_matches_to_json` captures default values defined on CLI arguments (such as `false` for `SetTrue` boolean flags). If a user specifies a setting (e.g. `verbose = true`) in their config file or environment, but does not explicitly pass the corresponding flag (e.g. `--verbose`) on the CLI, the CLI override layer merges the default value (e.g. `"verbose": false`), silently overriding the user's custom configuration.

### File Location
- File: `/Users/sac/clap-noun-verb/utils/src/adapters.rs`
- Lines: 88–92

### Root Cause
In `LayeredConfigAdapter::resolve`:
```rust
        // 4. Override with CLI ArgMatches
        let cli_val = crate::display_json::arg_matches_to_json(matches);
        if let Some(cli_obj) = cli_val.as_object() {
            merge_json_maps(merged_map, cli_obj.clone());
        }
```
The helper `arg_matches_to_json` converts all present arguments in `ArgMatches` to JSON values. Because `ArgMatches` contains default values, they are serialized and merged directly, obliterating custom values from previous config layers.

### Proposed Fix Strategy
Use `clap::parser::ValueSource` to filter out arguments whose source is `ValueSource::DefaultValue`. Only arguments explicitly supplied by the user (e.g. `ValueSource::CommandLine` or `ValueSource::Env`) should be merged into the resolved configuration map.

**Code Change Details:**
Modify the CLI override step in `resolve` as follows:
```rust
        // 4. Override with CLI ArgMatches
        let cli_val = crate::display_json::arg_matches_to_json(matches);
        if let Some(cli_obj) = cli_val.as_object() {
            let mut filtered_cli = serde_json::Map::new();
            for (key, val) in cli_obj {
                if let Some(source) = matches.value_source(key) {
                    if source != clap::parser::ValueSource::DefaultValue {
                        filtered_cli.insert(key.clone(), val.clone());
                    }
                }
            }
            merge_json_maps(merged_map, filtered_cli);
        }
```

---

## 2. Configuration Merging for Nested Keys

### Problem Statement
CLI arguments and environment variables are represented as flat maps. In contrast, custom configurations can contain nested structs (e.g., `{"database": {"host": "localhost"}}`). When merging a flat key (such as `database__host` or `database.host`) using `merge_json_maps`, the key is inserted at the root of the JSON map instead of recursively updating the nested map. As a result, nested configuration fields cannot be overridden by CLI or environment parameters.

### File Location
- File: `/Users/sac/clap-noun-verb/utils/src/adapters.rs`
- Lines: 116–134 (`merge_json_maps`)

### Root Cause
The current implementation of `merge_json_maps` only recurses if both the target and the source have the exact same key as an object:
```rust
fn merge_json_maps(target: &mut serde_json::Map<String, Value>, source: serde_json::Map<String, Value>) {
    for (k, v) in source {
        if v.is_null() {
            continue;
        }
        match target.entry(k) {
            serde_json::map::Entry::Occupied(mut entry) => {
                if let (Some(target_obj), Some(source_obj)) = (entry.get_mut().as_object_mut(), v.as_object()) {
                    merge_json_maps(target_obj, source_obj.clone());
                } else {
                    entry.insert(v);
                }
            }
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(v);
            }
        }
    }
}
```
If the source contains a flat key path representing a nested structure (e.g. `"database__host"`), it is placed directly into the root map because the key `"database__host"` is not present in the target.

### Proposed Fix Strategy
Implement a deep recursive merge algorithm that parses delimiters (`__` for environment variables, `.` for CLI/config key paths) and traverses/creates intermediate JSON objects as needed. This approach handles both flat maps with delimited keys and pre-nested structures.

**Code Change Details:**
Replace `merge_json_maps` and add a helper `insert_nested_value`:
```rust
fn insert_nested_value(target: &mut serde_json::Map<String, Value>, key_path: &[&str], value: Value) {
    if key_path.is_empty() {
        return;
    }
    let current_key = key_path[0].to_string();
    if key_path.len() == 1 {
        // Base case: insert or merge leaf value
        match target.entry(current_key) {
            serde_json::map::Entry::Occupied(mut entry) => {
                if let (Some(target_obj), Some(source_obj)) = (entry.get_mut().as_object_mut(), value.as_object()) {
                    merge_json_maps(target_obj, source_obj.clone());
                } else {
                    entry.insert(value);
                }
            }
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    } else {
        // Recursive case: navigate down or create intermediate objects
        let entry = target.entry(current_key).or_insert_with(|| Value::Object(serde_json::Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(serde_json::Map::new());
        }
        if let Some(sub_map) = entry.as_object_mut() {
            insert_nested_value(sub_map, &key_path[1..], value);
        }
    }
}

fn merge_json_maps(target: &mut serde_json::Map<String, Value>, source: serde_json::Map<String, Value>) {
    for (k, v) in source {
        if v.is_null() {
            continue;
        }
        let parts: Vec<&str> = if k.contains("__") {
            k.split("__").collect()
        } else {
            k.split('.').collect()
        };
        insert_nested_value(target, &parts, v);
    }
}
```

---

## 3. Help Layout Corruption under UTF-8/Tabs

### Problem Statement
`format_box_text` and `format_table` calculate column and border widths using `.len()`, which measures byte length. This is incorrect for multi-byte UTF-8 characters (e.g. CJK, emojis) and tab characters (`\t`). Additionally, cells containing newline characters (`\n`) are not split properly, corrupting row alignment and borders.

### File Location
- File: `/Users/sac/clap-noun-verb/utils/src/help.rs`
- Lines: 12–79 (`format_box_text` and `format_table`)

### Root Cause
1. `.len()` counts bytes (e.g., `"你好"` has 6 bytes but occupies 4 terminal cells; `"🦀"` has 4 bytes but occupies 2 terminal cells). Using `.len()` to calculate padding leads to misalignment.
2. Tab characters `\t` are not expanded to spaces before measuring, causing the terminal to render them with variable widths while the layout borders remain misaligned.
3. Mismatched row sizes and newlines in cell values are printed as-is, which splits raw row output into multiple disorganized lines, destroying vertical alignment.

### Proposed Fix Strategy
1. Implement a helper to expand tabs (`\t`) to spaces based on tab-stop offsets.
2. Implement a helper to calculate the actual display width of characters (specifically returning 2 for CJK and emojis, 0 for control/combining characters, and 1 for others).
3. Compute formatting padding manually using the difference between target display width and string display width, instead of relying on standard `format!` padding (which counts Unicode scalar points).
4. Dynamically split cell newlines (`\n`) and print them row-by-row on multiple terminal output lines.

**Code Change Details:**
Replace `/Users/sac/clap-noun-verb/utils/src/help.rs` with the following implementation:
```rust
/// Helper to print a bold, styled section header.
pub fn style_header(title: &str) -> String {
    format!("\x1b[1m\x1b[34m{}\x1b[0m", title)
}

/// Helper to print styled command/argument descriptions.
pub fn style_item(name: &str, description: &str) -> String {
    format!("  \x1b[32m{: <18}\x1b[0m {}", name, description)
}

fn char_width(c: char) -> usize {
    let val = c as u32;
    if val < 32 || (0x7F..=0x9F).contains(&val) {
        return 0;
    }
    // CJK Unified Ideographs, Hangul Syllables, and Emoji ranges
    if (0x1100..=0x115F).contains(&val)
        || val == 0x2329 || val == 0x232A
        || ((0x2E80..=0xA4CF).contains(&val) && val != 0x303F)
        || (0xAC00..=0xD7A3).contains(&val)
        || (0xF900..=0xFAFF).contains(&val)
        || (0xFE10..=0xFE19).contains(&val)
        || (0xFE30..=0xFE6F).contains(&val)
        || (0xFF01..=0xFF60).contains(&val)
        || (0xFFE0..=0xFFE6).contains(&val)
        || (0x1F300..=0x1F9FF).contains(&val)
        || (0x20000..=0x2FFFD).contains(&val)
        || (0x30000..=0x3FFFD).contains(&val)
    {
        2
    } else {
        1
    }
}

fn expand_tabs(s: &str, tab_width: usize) -> String {
    let mut expanded = String::new();
    let mut width = 0;
    for c in s.chars() {
        if c == '\t' {
            let spaces = tab_width - (width % tab_width);
            expanded.push_str(&" ".repeat(spaces));
            width += spaces;
        } else {
            expanded.push(c);
            width += char_width(c);
        }
    }
    expanded
}

fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Helper to format a block of text within an ASCII box.
pub fn format_box_text(text: &str) -> String {
    let lines: Vec<String> = text.lines().map(|l| expand_tabs(l, 4)).collect();
    let max_width = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
    
    let mut boxed = String::new();
    boxed.push('┌');
    boxed.push_str(&"─".repeat(max_width + 2));
    boxed.push_str("┐\n");
    
    for line in lines {
        let width = display_width(&line);
        let padding = " ".repeat(max_width - width);
        boxed.push_str(&format!("│ {}{} │\n", line, padding));
    }
    
    boxed.push('└');
    boxed.push_str(&"─".repeat(max_width + 2));
    boxed.push_str("┘\n");
    boxed
}

/// Helper to format a table of values (e.g. commands or parameters).
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }
    
    let col_count = headers.len();
    let mut col_widths = vec![0; col_count];
    
    for (i, header) in headers.iter().enumerate() {
        let expanded = expand_tabs(header, 4);
        col_widths[i] = display_width(&expanded);
    }
    
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            if i < col_count {
                for line in val.split('\n') {
                    let expanded = expand_tabs(line, 4);
                    let width = display_width(&expanded);
                    if width > col_widths[i] {
                        col_widths[i] = width;
                    }
                }
            }
        }
    }
    
    let mut output = String::new();
    
    // Header row
    for (i, header) in headers.iter().enumerate() {
        let expanded = expand_tabs(header, 4);
        let width = display_width(&expanded);
        let padding = if col_widths[i] > width {
            " ".repeat(col_widths[i] - width)
        } else {
            String::new()
        };
        output.push_str(&format!("{}{}", expanded, padding));
        output.push(' ');
    }
    output.push('\n');
    
    // Separator row
    for width in &col_widths {
        output.push_str(&"-".repeat(*width));
        output.push(' ');
    }
    output.push('\n');
    
    // Rows
    for row in rows {
        let split_cells: Vec<Vec<String>> = row.iter()
            .map(|cell| cell.split('\n').map(|s| expand_tabs(s, 4)).collect())
            .collect();
        let max_lines = split_cells.iter().map(|c| c.len()).max().unwrap_or(1);
        
        for line_idx in 0..max_lines {
            for i in 0..col_count {
                let line_val = split_cells.get(i)
                    .and_then(|c| c.get(line_idx))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let width = display_width(line_val);
                let padding = if col_widths[i] > width {
                    " ".repeat(col_widths[i] - width)
                } else {
                    String::new()
                };
                output.push_str(&format!("{}{}", line_val, padding));
                output.push(' ');
            }
            output.push('\n');
        }
    }
    
    output
}
```

---

## 4. Markdown Subcommand Anchors

### Problem Statement
When generating Markdown documentation, subcommands with spaces produce anchor links containing raw spaces (e.g. `(#sub command)`), which is invalid Markdown anchor syntax. They must be formatted to replace spaces with hyphens (e.g., `(#sub-command)`).

### File Location
- File: `/Users/sac/clap-noun-verb/utils/src/markdown.rs`
- Lines: 95–97

### Root Cause
In `markdown.rs`, the generation of subcommand anchor links is implemented as:
```rust
        for sub in &subcommands {
            writeln!(buf, "- [`{}`](#{})", sub.get_name(), sub.get_name().to_lowercase())?;
        }
```
Only `.to_lowercase()` is called, which leaves space characters untouched.

### Proposed Fix Strategy
Implement a simple, robust GFM-compliant anchor slugifier that maps uppercase letters to lowercase, replaces spaces/whitespace/hyphens with hyphens, and filters out other non-alphanumeric punctuation.

**Code Change Details:**
Modify the link generation loop as follows:
```rust
        for sub in &subcommands {
            let anchor: String = sub.get_name()
                .to_lowercase()
                .chars()
                .filter_map(|c| {
                    if c.is_alphanumeric() || c == '_' {
                        Some(c)
                    } else if c.is_whitespace() || c == '-' {
                        Some('-')
                    } else {
                        None
                    }
                })
                .collect();
            writeln!(buf, "- [`{}`](#{})", sub.get_name(), anchor)?;
        }
```
This maps `"sub command"` to `"sub-command"` and removes characters like parentheses or periods that are invalid in Markdown anchors.
