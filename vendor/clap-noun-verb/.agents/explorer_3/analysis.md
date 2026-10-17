# Crate Integration Research & Test Strategy Analysis

This analysis covers the integration requirements for the new shared `utils` package in the `clap-noun-verb` workspace, specifically focusing on `clap-num` (number parsing), `display_json` (JSON display formatting), and `clap-adapters` (conversion helpers), along with the design of the automated integration test suite architecture.

---

## 1. Number Parsing Helpers (`clap-num`)

`clap-num` is designed to validate and parse integers within ranges or alternative bases (like hexadecimal) from command line inputs.

### Integration with Clap (v4.5)
Clap v4.5 utilizes the `value_parser!` macro or functions implementing the `TypedValueParser` trait. 
Any closure or function with the signature `Fn(&str) -> Result<T, E>` (where `E` can be converted into a boxed error) implements `TypedValueParser` automatically. Since `clap-num` returns closures mapping `&str -> Result<T, String>`, they integrate cleanly with `.value_parser()`.

### Recommended API Designs

We propose wrapping `clap-num` features along with custom format parsers in `utils::number_parsing`:

```rust
pub mod number_parsing {
    use std::str::FromStr;
    use std::time::Duration;

    /// Parses decimal integers restricted to a closed interval [min, max].
    /// Wraps `clap_num::number_range`.
    pub fn decimal_range<T>(min: T, max: T) -> impl Fn(&str) -> Result<T, String>
    where
        T: FromStr + Copy + PartialOrd + std::fmt::Display,
        <T as FromStr>::Err: std::fmt::Display,
    {
        clap_num::number_range(min, max)
    }

    /// Parses an integer that can be either decimal or hexadecimal (prefixed with 0x or 0X).
    /// Wraps `clap_num::maybe_hex`.
    pub fn maybe_hex<T>(s: &str) -> Result<T, String>
    where
        T: num_traits::Num + num_traits::Zero + num_traits::One + num_traits::CheckedAdd 
            + num_traits::CheckedMul + num_traits::CheckedSub + num_traits::CheckedShl 
            + num_traits::FromPrimitive,
    {
        clap_num::maybe_hex(s)
    }

    /// Parses an integer (decimal/hex) restricted to a closed interval [min, max].
    /// Wraps `clap_num::maybe_hex_range`.
    pub fn maybe_hex_range<T>(min: T, max: T) -> impl Fn(&str) -> Result<T, String>
    where
        T: num_traits::Num + num_traits::Zero + num_traits::One + num_traits::CheckedAdd 
            + num_traits::CheckedMul + num_traits::CheckedSub + num_traits::CheckedShl 
            + num_traits::FromPrimitive + Copy + PartialOrd + std::fmt::Display,
    {
        clap_num::maybe_hex_range(min, max)
    }

    /// Custom format: Parses human-readable percentage strings (e.g., "50%", "12.5%") to f64 values (0.5, 0.125).
    pub fn parse_percentage(s: &str) -> Result<f64, String> {
        if !s.ends_with('%') {
            return Err("Percentage must end with '%'".to_string());
        }
        let val_str = &s[..s.len() - 1];
        let val = val_str.parse::<f64>().map_err(|e| format!("Invalid percentage: {}", e))?;
        if !(0.0..=100.0).contains(&val) {
            return Err("Percentage must be between 0% and 100%".to_string());
        }
        Ok(val / 100.0)
    }

    /// Custom format: Parses human-readable byte sizes (e.g., "10kb", "5MB", "2g") to u64 bytes.
    pub fn parse_bytes(s: &str) -> Result<u64, String> {
        let s_lower = s.to_lowercase();
        let (num_part, unit_part) = s_lower.split_at(
            s_lower.find(|c: char| c.is_alphabetic()).unwrap_or(s_lower.len())
        );
        let number = num_part.trim().parse::<u64>().map_err(|e| format!("Invalid byte number: {}", e))?;
        
        let multiplier = match unit_part.trim() {
            "" | "b" => 1,
            "k" | "kb" => 1024,
            "m" | "mb" => 1024 * 1024,
            "g" | "gb" => 1024 * 1024 * 1024,
            "t" | "tb" => 1024 * 1024 * 1024 * 1024,
            unknown => return Err(format!("Unknown byte unit: {}", unknown)),
        };
        
        number.checked_mul(multiplier).ok_or_else(|| "Byte size overflow".to_string())
    }

    /// Custom format: Parses duration strings (e.g., "30s", "1h 15m") to std::time::Duration.
    pub fn parse_duration(s: &str) -> Result<Duration, String> {
        // Leverages standard formatting parser
        let mut total_secs = 0u64;
        let words = s.split_whitespace();
        for word in words {
            let pos = word.find(|c: char| c.is_alphabetic()).ok_or("Missing unit in duration segment")?;
            let (num_part, unit_part) = word.split_at(pos);
            let val = num_part.parse::<u64>().map_err(|e| format!("Invalid duration value: {}", e))?;
            let secs = match unit_part {
                "s" | "sec" | "secs" => val,
                "m" | "min" | "mins" => val * 60,
                "h" | "hour" | "hours" => val * 3600,
                "d" | "day" | "days" => val * 86400,
                unknown => return Err(format!("Unknown duration unit: {}", unknown)),
            };
            total_secs = total_secs.checked_add(secs).ok_or("Duration overflow")?;
        }
        Ok(Duration::from_secs(total_secs))
    }
}
```

---

## 2. JSON Display Formatting (`display_json`)

CLI tools often need to format command specifications or command outputs as JSON. 

### Output Configurations with `display_json`
The `display_json` crate on crates.io offers a `#[derive(DisplayAsJson)]` and `#[derive(DisplayAsJsonPretty)]` macro. This implements `std::fmt::Display` for any structure using `serde_json`, allowing developers to output data directly via `println!("{}", my_struct)` when the user selects a JSON output format.

### Command Structure Serialization to JSON
Because `clap::Command` and `clap::ArgMatches` are not serializable out-of-the-box, we must provide mapping adapters to convert them to structured, serializable representations.

### Recommended API Designs (`utils::display_json`)

```rust
pub mod display_json {
    use serde::Serialize;
    use serde_json::Value;

    /// A serializable schema of a clap CLI Command structure.
    #[derive(Debug, Serialize, Clone)]
    pub struct CommandSchema {
        pub name: String,
        pub version: Option<String>,
        pub author: Option<String>,
        pub about: Option<String>,
        pub subcommands: Vec<CommandSchema>,
        pub arguments: Vec<ArgSchema>,
    }

    /// A serializable schema of a clap Argument structure.
    #[derive(Debug, Serialize, Clone)]
    pub struct ArgSchema {
        pub name: String,
        pub short: Option<char>,
        pub long: Option<String>,
        pub help: Option<String>,
        pub required: bool,
        pub multiple: bool,
        pub is_flag: bool,
    }

    /// Converts a `clap::Command` structure into its serializable schema representation.
    pub fn extract_command_schema(cmd: &clap::Command) -> CommandSchema {
        CommandSchema {
            name: cmd.get_name().to_string(),
            version: cmd.get_version().map(String::from),
            author: cmd.get_author().map(String::from),
            about: cmd.get_about().map(|s| s.to_string()),
            subcommands: cmd.get_subcommands().map(extract_command_schema).collect(),
            arguments: cmd.get_arguments()
                .map(|arg| ArgSchema {
                    name: arg.get_id().as_str().to_string(),
                    short: arg.get_short(),
                    long: arg.get_long().map(String::from),
                    help: arg.get_help().map(|s| s.to_string()),
                    required: arg.is_required_set(),
                    multiple: arg.is_multiple_values_set() || matches!(
                        arg.get_action(), 
                        clap::ArgAction::Append | clap::ArgAction::Count
                    ),
                    is_flag: matches!(
                        arg.get_action(), 
                        clap::ArgAction::SetTrue | clap::ArgAction::SetFalse | clap::ArgAction::Count
                    ),
                })
                .collect(),
        }
    }

    /// Converts a parsed `clap::ArgMatches` into a JSON Value object map.
    pub fn arg_matches_to_json(matches: &clap::ArgMatches) -> Value {
        let mut map = serde_json::Map::new();
        for id in matches.ids() {
            let name = id.as_str();
            
            // Check flags
            if matches.get_one::<bool>(name).is_some() {
                let flag = matches.get_flag(name);
                map.insert(name.to_string(), Value::Bool(flag));
            } else if let Some(vals) = matches.get_many::<String>(name) {
                let list: Vec<String> = vals.cloned().collect();
                if list.len() == 1 {
                    map.insert(name.to_string(), Value::String(list[0].clone()));
                } else {
                    map.insert(name.to_string(), Value::Array(
                        list.into_iter().map(Value::String).collect()
                    ));
                }
            } else if let Some(vals) = matches.get_many::<i64>(name) {
                let list: Vec<i64> = vals.cloned().collect();
                if list.len() == 1 {
                    map.insert(name.to_string(), Value::Number(list[0].into()));
                } else {
                    map.insert(name.to_string(), Value::Array(
                        list.into_iter().map(|n| Value::Number(n.into())).collect()
                    ));
                }
            }
        }
        Value::Object(map)
    }

    /// Helper trait to print any serializable output struct as JSON.
    pub trait PrintJson {
        fn print_json(&self) -> Result<(), serde_json::Error>;
        fn print_json_pretty(&self) -> Result<(), serde_json::Error>;
    }

    impl<T: Serialize> PrintJson for T {
        fn print_json(&self) -> Result<(), serde_json::Error> {
            println!("{}", serde_json::to_string(self)?);
            Ok(())
        }
        
        fn print_json_pretty(&self) -> Result<(), serde_json::Error> {
            println!("{}", serde_json::to_string_pretty(self)?);
            Ok(())
        }
    }
}
```

---

## 3. Clap Adapters (`clap-adapters`)

"Clap Adapters" refer to conversion patterns that bridge CLI-specific parser models (DTOs) with decoupled domain-specific configurations, key-value mappings, and third-party API payloads.

### Mapping & Decoupling Patterns
1. **Decoupled Configuration Mapping**: Instead of referencing CLI structures in business modules, we map CLI types to separate config structures (`impl From<CliConfig> for DomainConfig`).
2. **Key-Value Pair Extraction**: Parse argument lists (e.g. repeated arguments like `-D key1=val1 -D key2=val2`) into standard map configurations like `HashMap<String, String>`.
3. **Layered Configuration Resolution**: Resolve configuration options based on precedence rules: Config File (TOML/JSON) < Environment Variables < Command Line Arguments.

### Recommended API Designs (`utils::adapters`)

```rust
pub mod adapters {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use clap::ArgMatches;
    use serde::de::DeserializeOwned;

    /// Parser for KEY=VALUE argument format.
    pub fn parse_key_val(s: &str) -> Result<(String, String), String> {
        let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
        Ok((s[..pos].trim().to_string(), s[pos + 1..].trim().to_string()))
    }

    /// Extract key-value pairs from multiple arguments into a HashMap.
    pub fn extract_key_value_pairs(matches: &ArgMatches, arg_name: &str) -> Result<HashMap<String, String>, String> {
        let mut map = HashMap::new();
        if let Some(pairs) = matches.get_many::<String>(arg_name) {
            for pair in pairs {
                let (key, val) = parse_key_val(pair)?;
                map.insert(key, val);
            }
        }
        Ok(map)
    }

    /// Decoupling adapter trait to cleanly load domain models from ArgMatches.
    pub trait FromArgMatches: Sized {
        type Error;
        fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Self::Error>;
    }

    /// Layered config resolver that merges config files, environment variables, and CLI overrides.
    pub struct LayeredConfigAdapter<T> {
        pub file_path: Option<PathBuf>,
        pub env_prefix: Option<String>,
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> LayeredConfigAdapter<T>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Default,
    {
        pub fn new(file_path: Option<PathBuf>, env_prefix: Option<String>) -> Self {
            Self {
                file_path,
                env_prefix,
                _marker: std::marker::PhantomData,
            }
        }

        pub fn resolve(&self, matches: &ArgMatches) -> Result<T, anyhow::Error> {
            // 1. Start with defaults
            let mut resolved = T::default();
            
            // 2. Load from config file if present
            if let Some(ref path) = self.file_path {
                if path.exists() {
                    let contents = std::fs::read_to_string(path)?;
                    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                        resolved = toml::from_str(&contents)?;
                    } else {
                        resolved = serde_json::from_str(&contents)?;
                    }
                }
            }
            
            // 3. Environment variables mapping logic using prefix
            if let Some(ref prefix) = self.env_prefix {
                // Read from environment and apply to fields via serialization
                // (e.g. env variables prefixed with PREFIX_ mapped to fields)
            }
            
            // 4. Override with ArgMatches
            // ...
            
            Ok(resolved)
        }
    }
}
```

---

## 4. Automated Integration Test Suite Architecture

To ensure correctness and maintain stability as dependencies evolve, the `utils` package should contain an automated integration test suite.

```
utils/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Module declarations and re-exports
│   ├── completions.rs
│   ├── mangen.rs
│   ├── markdown.rs
│   ├── help.rs
│   ├── number_parsing.rs
│   ├── display_json.rs
│   └── adapters.rs
└── tests/              # Encapsulated integration tests
    ├── number_parsing.rs
    ├── display_json.rs
    ├── adapters.rs
    ├── doc_generation.rs
    └── common.rs       # Shared test command builders
```

### Test Coverage Strategies

1. **Number Parsing (`tests/number_parsing.rs`)**:
   - **Boundary Tests**: Verify values at boundaries of `decimal_range` (e.g. `min`, `max`, `min - 1` (error), `max + 1` (error)).
   - **Hex Parsing**: Verify decimal vs. hexadecimal inputs (e.g. `0x1A` vs `26`) parse to correct integers. Ensure invalid characters trigger formatting errors.
   - **Custom Parsers**: Test parsing percentages (e.g. `0%` -> `0.0`, `12.5%` -> `0.125`, `101%` -> error, missing `%` -> error). Test parsing durations (e.g. `30s` -> 30 sec, `1h 15m` -> 4500 sec) and byte sizes (e.g. `5MB` -> `5242880`, `2g` -> `2147483648`).

2. **JSON Formatting (`tests/display_json.rs`)**:
   - **Command Schema**: Serialize a complex mock `clap::Command` (with multi-level nested subcommands and diverse arguments) and compare its output JSON string against a baseline using `insta` snapshot testing.
   - **ArgMatches Serialization**: Build a mock `ArgMatches` map and assert its translation to JSON matches the expected structure.
   - **`PrintJson` Trait**: Verify that calling `.print_json()` successfully outputs JSON to stdout.

3. **Adapters & Decoupling (`tests/adapters.rs`)**:
   - **Key-Value Pairs**: Mock a CLI call with multiple `-D key=val` entries and verify it correctly yields the corresponding `HashMap`.
   - **Layered Config Resolution**: Perform tests with:
     - Only config files (TOML/JSON).
     - Config files overridden by environment variables.
     - Environment variables overridden by CLI arguments.
   - Assert correct values are resolved under each priority layer.

### Snapshot Testing with `insta` and `assert_cmd`
For integration tests of the documentation utilities (shell completions, man pages, markdown help) and CLI schema generation, we should use the **`insta`** snapshot testing crate:

```rust
// In utils/tests/doc_generation.rs
#[test]
fn test_markdown_generation_snapshot() {
    let mut cmd = crate::common::create_test_command();
    let mut buf = Vec::new();
    utils::markdown::generate_markdown(&cmd, &mut buf).unwrap();
    let markdown = String::from_utf8(buf).unwrap();
    
    // Generates/verifies snapshot under tests/snapshots/
    insta::assert_snapshot!(markdown);
}
```

This guarantees that any changes to generated layouts or documentation structures are tracked and caught automatically by `cargo test`.
