# Adversarial Challenge Report — clap-noun-verb utils

## Challenge Summary

**Overall risk assessment**: MEDIUM

While the core functionality of the `utils` package is robust and integrates correctly with `clap` and `serde_json`, several edge cases and adversarial inputs trigger runtime panics (especially in debug mode) or result in inconsistent behavior (returning `Ok` for invalid inputs). 

Specifically, we identified:
1. An integer overflow panic in `parse_duration` when processing extremely large units.
2. Inconsistent validation in `parse_duration` where empty/whitespace inputs are treated as valid `0` durations instead of syntax errors.
3. Developer-configuration panic in `decimal_range` (from `clap-num`) when `min > max` is evaluated at runtime.
4. Serialization limits on deeply nested subcommands.

---

## Challenges

### [Critical] Challenge 1: Unchecked Multiplication Overflow in `parse_duration`

- **Assumption challenged**: That the duration parsed into a `u64` can be multiplied by segment multipliers (`60` for minutes, `3600` for hours, `86400` for days) without overflow checking.
- **Attack scenario**: An adversarial or malformed configuration input passes a huge numeric value with unit segments (e.g., `18446744073709551615m` which is `u64::MAX` minutes). The parser reads `u64::MAX` into `val`, then matches `"m"` and executes `val * 60`. 
- **Blast radius**:
  - **In debug mode**: The application crashes immediately with a panic (`attempt to multiply with overflow`).
  - **In release mode**: The multiplication silently wraps (overflows), resulting in highly incorrect duration values, causing downstream logic issues.
- **Mitigation**: Utilize `checked_mul` and `checked_add` inside `parse_duration` to return a clean error if an overflow occurs.
  ```rust
  let secs = match unit_part {
      "s" | "sec" | "secs" => Some(val),
      "m" | "min" | "mins" => val.checked_mul(60),
      "h" | "hour" | "hours" => val.checked_mul(3600),
      "d" | "day" | "days" => val.checked_mul(86400),
      unknown => return Err(format!("Unknown duration unit: {}", unknown)),
  }.ok_or_else(|| "Duration segment overflow".to_string())?;
  ```

### [Medium] Challenge 2: Empty/Whitespace String Ingestion in `parse_duration`

- **Assumption challenged**: That all invalid or empty formats passed to `parse_duration` are rejected with a syntax/parsing error.
- **Attack scenario**: Passing `""` or `" "` (only spaces/tabs) to `parse_duration`. Since `s.split_whitespace()` returns an empty iterator, the inner parsing loop is completely bypassed, and the function returns `Ok(Duration::from_secs(0))`.
- **Blast radius**: Low-to-medium. It allows blank inputs to resolve to `0s` without warning, whereas a single number without unit like `"30"` correctly fails with `Missing unit in duration segment`. This causes inconsistent interface parsing behavior.
- **Mitigation**: Add a guard check at the entry point of `parse_duration`:
  ```rust
  if s.trim().is_empty() {
      return Err("Duration string cannot be empty".to_string());
  }
  ```

### [Medium] Challenge 3: Panic on Invalid Bounds Configuration in `decimal_range`

- **Assumption challenged**: That boundary functions wrapped from `clap-num` (`decimal_range`, `maybe_hex_range`) handle invalid bounds configurations gracefully by returning error results or validation failures.
- **Attack scenario**: A developer defines an invalid range constraint where minimum exceeds maximum, such as `decimal_range(100, 0)`. When the command line arguments are parsed and the closure is executed on any input (e.g. `"50"`), the underlying `clap_num::number_range` panics with `minimum of 100 exceeds maximum of 0`.
- **Blast radius**: Medium. Developer configuration errors translate into runtime application panics when parsing CLI arguments.
- **Mitigation**: Validate that `min <= max` inside `decimal_range` or inside the returned closure before calling `clap_num`. If the configuration is invalid, return an `Err` explaining the range misconfiguration.

### [Low] Challenge 4: Deeply Nested Subcommand Deserialization Limits

- **Assumption challenged**: That any extracted command schema of arbitrary size and nesting depth can be successfully serialized and deserialized using default `serde_json` structures.
- **Attack scenario**: A user/developer constructs a command structure with a subcommand hierarchy nested more than 128 levels deep (e.g., recursive subcommands). Calling `extract_command_schema` and `serde_json::to_string` succeeds. However, attempting to deserialize it back with `serde_json::from_str` fails with a `recursion limit exceeded` error.
- **Blast radius**: Low. Deep subcommand hierarchies exceeding 128 levels are extremely rare in practical CLI design, but the deserialization fails when it does occur.
- **Mitigation**: Document that clients deserializing the command schema JSON must customize `serde_json::Deserializer::disable_recursion_limit()` or increase the recursion limit if they expect hierarchies exceeding 128 levels.

---

## Stress Test Results

All stress tests and adversarial edge cases were integrated into `utils/tests/adversarial.rs` and executed. The results are summarized below:

| Scenario / Input | Expected Behavior | Actual Behavior | Pass/Fail | Notes |
|---|---|---|---|---|
| Range parsing with `min > max` (e.g., `decimal_range(100, 0)("50")`) | Fail gracefully (error) | Panics: `minimum of 100 exceeds maximum of 0` | **Fail (Panic)** | Handled in test with `catch_unwind` |
| Overflow decimal/hex limits (e.g., `"18446744073709551616"`, `"0x10000000000000000"`) | Return parser error | Returns `Err` (correctly caught by `clap-num`) | **Pass** | Robust against u64 overflow |
| Byte parser with custom units (`"10kb"`, `"5mb"`, `"2g"`, case-insensitive) | Parsed multiplier applied | Correct u64 value returned | **Pass** | Correctly implemented |
| Byte parser multiplier overflow (e.g., `u64::MAX` with `"kb"`) | Return size overflow error | Returns `Err("Byte size overflow")` | **Pass** | Safely handled via `checked_mul` |
| Duration parser with empty string (`""` or `" "`) | Return invalid segment error | Returns `Ok(0s)` | **Fail (Inconsistent)** | Bypasses loop entirely |
| Duration parser segments overflow (e.g., `u64::MAX` minutes `"18446744073709551615m"`) | Return overflow error | Panics: `attempt to multiply with overflow` | **Fail (Panic)** | Handled in test with `catch_unwind` |
| Deeply nested commands (depth 50) | Serialize & Deserialize successfully | Serializes and deserializes successfully | **Pass** | Within Serde limit |
| Extremely nested commands (depth 1000) | Stack-safe extraction & serialization | Succeeds without stack overflow | **Pass** | Deserialization from JSON fails due to recursion limit (expected) |
| JSON Serialization with emoji and control characters | JSON serialized cleanly with escaping | Correctly escapes quotes and represents emojis | **Pass** | Clean serialization |
| Argument matching with default values of custom type | Fallback gracefully, parses as string | Parsed value returned as JSON String without panics | **Pass** | Safe due to AssertUnwindSafe and catch_unwind in `arg_matches_to_json` |

---

## Unchallenged Areas

- **System and Terminal I/O Hooks** — We focused exclusively on pure logic boundaries (number parsing constraints, JSON schemas, argument converters). Terminal resizing, system locale changes, or stdout pipes were not stress-tested because they are handled directly by stdout/standard library and are outside the scope of `utils`.
