# Analysis Report: shared `utils` Integration Targets

This report details a codebase scan of the `clap-noun-verb` repository to identify duplication of helper functions, traits, or stubs that can be replaced with implementations inside the newly created shared `utils` (`clap-noun-verb-utils`) package.

---

## 1. Summary of Crate Capabilities in `clap-noun-verb-utils`

The shared `clap-noun-verb-utils` library exports the following modular utilities:
1. **`completions`**: `generate_completions<S: Shell>(cmd: &mut Command, shell: S, buf: &mut dyn Write)` using `clap_complete`.
2. **`mangen`**: `generate_manpage(cmd: &Command, buf: &mut dyn Write) -> std::io::Result<()>` using `clap_mangen`.
3. **`markdown`**: `generate_markdown(cmd: &Command, buf: &mut dyn Write) -> std::io::Result<()>` which walks the `Command` tree and writes recursive markdown.
4. **`number_parsing`**: Composable validation/parsing functions:
   - `decimal_range` (restricted integer ranges)
   - `maybe_hex` and `maybe_hex_range` (accepts decimal or 0x-prefixed hex)
   - `parse_percentage` (e.g., `"50%"` -> `0.5`)
   - `parse_bytes` (e.g., `"10kb"` -> `10240`)
   - `parse_duration` (e.g., `"30s"`, `"1h 15m"` -> `std::time::Duration`)
5. **`display_json`**: `extract_command_schema(cmd: &Command) -> CommandSchema`, `arg_matches_to_json(matches: &ArgMatches) -> Value`, and the `PrintJson` trait for printing/pretty-printing serializable types.
6. **`adapters`**:
   - `parse_key_val(s: &str) -> Result<(String, String), String>`
   - `extract_key_value_pairs(matches: &ArgMatches, arg_name: &str) -> Result<HashMap<String, String>, String>`
   - `FromArgMatches` trait for loading domain models directly.
   - `LayeredConfigAdapter<T>` resolver for merging configuration files, environment variables, and CLI overrides.
7. **`help`**: Text formatting and layout helpers (`style_header`, `style_item`, `expand_line`, `display_width`, `format_box_text`, `format_table`).

---

## 2. Scan Findings: Target Replacements in `examples/`

Scanning the `examples/` directory (excluding `examples/playground/`) revealed the following replacement opportunity:

### target A: Key-Value Parsing in `examples/ggen/validators.rs`
* **File Path**: `examples/ggen/validators.rs` (Lines 98-121)
* **Observed Duplication**:
  ```rust
  pub fn validate_template_vars(vars: &[String]) -> Result<Vec<(String, String)>, UserError> {
      let mut parsed = Vec::new();

      for var in vars {
          if let Some((key, value)) = var.split_once('=') {
              if key.trim().is_empty() {
                  return Err(UserError::new(
                      ErrorCategory::Validation,
                      format!("Variable '{}' has empty key", var),
                      "Variable keys must not be empty:\n  \
                      ✓ Correct: name=value\n  \
                      ✗ Wrong: =value"
                          .to_string(),
                  ));
              }

              parsed.push((key.trim().to_string(), value.trim().to_string()));
          } else {
              return Err(super::errors::invalid_var_format(var));
          }
      }

      Ok(parsed)
  }
  ```
* **Replacement Strategy**:
  Replace the manual key-value extraction and parsing using `clap_noun_verb_utils::adapters::parse_key_val` which handles `KEY=VALUE` splitting and trimming out-of-the-box.
  ```rust
  pub fn validate_template_vars(vars: &[String]) -> Result<Vec<(String, String)>, UserError> {
      let mut parsed = Vec::new();
      for var in vars {
          let (key, val) = clap_noun_verb_utils::adapters::parse_key_val(var)
              .map_err(|_| super::errors::invalid_var_format(var))?;
          if key.is_empty() {
              return Err(UserError::new(
                  ErrorCategory::Validation,
                  format!("Variable '{}' has empty key", var),
                  "Variable keys must not be empty:\n  ✓ Correct: name=value\n  ✗ Wrong: =value".to_string(),
              ));
          }
          parsed.push((key, val));
      }
      Ok(parsed)
  }
  ```

---

## 3. Scan Findings: Target Replacements in `playground/`

The standalone `mcpp-cli` tool inside the `playground/` directory contains substantial duplication of autonomic v5 capabilities.

### Target B: Hand-rolled Shell completions in `playground/src/domain/completions.rs` and `playground/src/commands/meta.rs`
* **File Paths**:
  - `playground/src/domain/completions.rs` (Lines 1-280, entire file)
  - `playground/src/commands/meta.rs` (Lines 130-160)
* **Observed Duplication**:
  `completions.rs` defines its own `ShellType` enum (Zsh, Bash, Fish, PowerShell, Elvish) and includes over 200 lines of custom string formatting logic to generate Bash commands, Zsh descriptions, Fish subcommands, etc.
  ```rust
  pub fn generate_completion_script(
      cli_name: &str,
      capabilities: &[CliCapability],
      shell: ShellType,
  ) -> CompletionScript { ... }
  ```
  And `playground/src/commands/meta.rs` invokes it:
  ```rust
  #[verb("completions")]
  fn generate_completions(
      #[arg(index = 1)]
      shell: String,
  ) -> Result<CompletionScriptOutput> {
      ...
      let script = crate::domain::generate_completion_script("playground", &capabilities, shell_type);
      ...
  }
  ```
* **Replacement Strategy**:
  Delete `playground/src/domain/completions.rs` entirely.
  Update `playground/src/commands/meta.rs` to generate completions via `clap_noun_verb_utils::completions::generate_completions`. This leverages `clap_complete` to dynamically construct completion scripts for the supported shells using the actual parsed `clap::Command` surface, rather than relying on a custom capabilities array.

### Target C: Hardcoded manpage troff rendering in `playground/src/commands/meta.rs`
* **File Path**: `playground/src/commands/meta.rs` (Lines 230-312)
* **Observed Duplication**:
  The `manpage` verb command manually prints a static troff template:
  ```rust
  #[verb("manpage")]
  fn generate_manpage() -> Result<()> {
      let version = env!("CARGO_PKG_VERSION");
      println!(
          ".TH PLAYGROUND 1 \"2024\" \"v{}\" \"Playground CLI Manual\"",
          version
      );
      println!(
          r#"
  .SH NAME
  playground \- Comprehensive v5 feature showcase for clap-noun-verb
  ...
  "#);
      Ok(())
  }
  ```
* **Replacement Strategy**:
  Replace this static println layout with `clap_noun_verb_utils::mangen::generate_manpage`. It can render Troff format directly to stdout using the standard `clap_mangen` crate, ensuring that any changes to CLI subcommands or options are automatically reflected in the generated man page.

### Target D: Local Configuration State in `playground/src/domain/config.rs` and `playground/src/commands/config.rs`
* **File Paths**:
  - `playground/src/domain/config.rs` (Lines 55-109)
  - `playground/src/commands/config.rs` (Lines 20-120)
* **Observed Duplication**:
  `config.rs` implements a basic in-memory `HashMap` to simulate configuration options (`output_dir`, `default_family`, etc.).
* **Replacement Strategy**:
  Refactor the configuration commands to utilize `clap_noun_verb_utils::adapters::LayeredConfigAdapter<T>`. This lets the playground package automatically resolve configuration files (TOML or JSON), environment variables, and command line overrides in a standardized, layered manner.

### Target E: Manual JSON Output/Formatting inside Command Modules
* **File Path**: `playground/src/outputs.rs` (entire file)
* **Observed Duplication**:
  Output structures like `SparqlResultOutput`, `DoctorRunOutput`, and `TelemetryOutput` are serialized using manual or implicit JSON serialization calls inside the command verbs.
* **Replacement Strategy**:
  Derive/implement the `clap_noun_verb_utils::display_json::PrintJson` trait on the output structs in `outputs.rs`. This provides unified, reusable methods `.print_json()` and `.print_json_pretty()` to format outputs, eliminating duplicate boilerplate serialization statements in the commands.
