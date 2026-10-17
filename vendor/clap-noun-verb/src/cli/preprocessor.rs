// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::error::Result;
use std::io::Read;

/// Get a nested value from a JSON object using dot notation
pub fn get_json_path(value: &serde_json::Value, path: &str) -> Option<String> {
    let mut current = value;
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some(obj) = current.as_object() {
            current = obj.get(part)?;
        } else {
            let arr = current.as_array()?;
            let idx = part.parse::<usize>().ok()?;
            current = arr.get(idx)?;
        }
    }
    match current {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        _ => Some(current.to_string()),
    }
}

/// Preprocesses a single step's arguments
pub fn preprocess_args(
    args: &[String],
    stdin_val: &Option<String>,
    step_results: &[serde_json::Value],
) -> Result<Vec<String>> {
    let mut processed = Vec::new();
    for arg in args {
        let mut new_arg = arg.clone();

        // 1. Resolve step references @{step.key}
        let mut search_idx = 0;
        while let Some(start_offset) = new_arg[search_idx..].find("@{") {
            let start_idx = search_idx + start_offset;
            if let Some(end_offset) = new_arg[start_idx..].find('}') {
                let end_idx = start_idx + end_offset;
                let ref_content = &new_arg[start_idx + 2..end_idx];

                let mut resolved = false;
                if let Some(dot_idx) = ref_content.find('.') {
                    let (step_str, path) = ref_content.split_at(dot_idx);
                    let path = &path[1..]; // skip '.'
                    if let Ok(step_num) = step_str.parse::<usize>() {
                        if step_num > 0 && step_num <= step_results.len() {
                            let step_data = &step_results[step_num - 1];
                            if let Some(resolved_val) = get_json_path(step_data, path) {
                                new_arg.replace_range(start_idx..=end_idx, &resolved_val);
                                search_idx = start_idx + resolved_val.len();
                                resolved = true;
                            }
                        }
                    }
                }
                if !resolved {
                    new_arg.replace_range(start_idx..=end_idx, "");
                    search_idx = start_idx;
                }
            } else {
                break;
            }
        }

        // 2. Resolve stdin bindings @- and @-::key
        if new_arg == "@-" {
            if let Some(ref stdin_str) = stdin_val {
                new_arg = stdin_str.clone();
            } else {
                new_arg = String::new();
            }
        } else if new_arg.starts_with("@-::") {
            let key = &new_arg[4..];
            if let Some(ref stdin_str) = stdin_val {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(stdin_str) {
                    if let Some(resolved_val) = get_json_path(&json_val, key) {
                        new_arg = resolved_val;
                    } else {
                        new_arg = String::new();
                    }
                } else {
                    new_arg = String::new();
                }
            } else {
                new_arg = String::new();
            }
        }

        processed.push(new_arg);
    }
    Ok(processed)
}

/// Helper to read stdin if any argument needs it
pub fn read_stdin_if_needed(steps: &[Vec<String>]) -> Option<String> {
    let mut needs_stdin = false;
    for step in steps {
        if step.iter().any(|arg| arg == "@-" || arg.starts_with("@-::")) {
            needs_stdin = true;
            break;
        }
    }

    if needs_stdin {
        let mut buffer = String::new();
        if std::io::stdin().read_to_string(&mut buffer).is_ok() {
            Some(buffer)
        } else {
            None
        }
    } else {
        None
    }
}
