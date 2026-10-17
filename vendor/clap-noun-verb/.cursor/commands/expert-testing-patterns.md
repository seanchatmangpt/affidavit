# Expert-Level Testing Patterns - Multi-Step Workflow

## Purpose

This command guides agents through implementing expert-level testing patterns that catch 80% of production bugs. It breaks down complex testing scenarios into clear, sequential steps with examples and validation checkpoints.

## Workflow Overview

```
Step 1: Identify Test Type → Step 2: Choose Pattern → Step 3: Implement Test → Step 4: Verify Coverage → Step 5: Validate Quality
```

## Core Principle: 80/20 Rule

**Expert testing focuses on the 20% of test cases that catch 80% of bugs**:
- Error paths (not just happy path)
- Boundary conditions (not just normal values)
- Resource cleanup (not just normal execution)
- Concurrency (not just single-threaded)
- Real dependencies (not just mocks)

## Step-by-Step Pattern Implementation

### Pattern 1: Error Path Testing (Critical - 80% of bugs)

#### Step 1.1: Identify Error Scenarios

**Action**: List all possible error conditions for the function/feature.

**Questions to ask**:
- What inputs cause errors?
- What error variants exist?
- Can errors be recovered from?
- Are errors properly propagated?

**Example**: For `parse_argument(input: &str) -> Result<String, NounVerbError>`
- Empty input → `NounVerbError::missing_argument`
- Invalid format → `NounVerbError::argument_error`
- Edge cases: `"  "`, `"\n"`, etc.

#### Step 1.2: Create Test Cases

**Action**: Create test cases for each error scenario.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{NounVerbError, Result};

    #[test]
    fn test_parse_argument_all_error_paths() -> Result<()> {
        // Arrange: Test all error variants
        let test_cases = vec![
            ("", NounVerbError::missing_argument("input")),
            ("  ", NounVerbError::argument_error("Empty input")),
            ("\n", NounVerbError::argument_error("Invalid format")),
        ];
        
        // Act & Assert: Verify each error path
        for (input, expected_error) in test_cases {
            let result = parse_argument(input);
            assert!(result.is_err(), "Should fail for input: {}", input);
            match result {
                Err(e) => {
                    // Verify error type matches
                    assert!(matches!(e, NounVerbError::MissingArgument { .. }) || 
                            matches!(e, NounVerbError::ArgumentError { .. }));
                },
                Ok(_) => panic!("Expected error for input: {}", input),
            }
        }
        Ok(())
    }
}
```

#### Step 1.3: Test Error Recovery

**Action**: Verify system can recover from errors.

```rust
#[test]
fn test_error_recovery() -> Result<()> {
    // Arrange: Create parser
    let mut parser = ArgumentParser::new();
    
    // Act: Cause error
    assert!(parser.parse("invalid").is_err());
    
    // Assert: Parser should still be usable after error
    let result = parser.parse("valid");
    assert!(result.is_ok(), "Parser should recover from error");
    Ok(())
}
```

#### Step 1.4: Verify Coverage

**Checklist**:
- [ ] All error variants tested
- [ ] Error messages verified
- [ ] Error recovery tested
- [ ] Edge cases covered

---

### Pattern 2: Boundary Condition Testing

#### Step 2.1: Identify Boundaries

**Action**: List all boundary conditions.

**Common boundaries**:
- Empty collections
- Single item
- Maximum size
- Zero values
- Negative values (if applicable)
- Minimum/maximum ranges

#### Step 2.2: Create Boundary Tests

**Action**: Test each boundary condition.

```rust
#[test]
fn test_collection_boundaries() -> Result<()> {
    // Arrange: Test empty collection
    let empty: Vec<String> = vec![];
    assert_eq!(process_commands(&empty)?, 0, "Empty collection should return 0");
    
    // Arrange: Test single item
    let single = vec!["command".to_string()];
    assert_eq!(process_commands(&single)?, 1, "Single item should work");
    
    // Arrange: Test zero values
    let zeros = vec!["".to_string(); 100];
    let result = process_commands(&zeros);
    assert!(result.is_ok(), "Zero values should work");
    
    Ok(())
}
```

---

### Pattern 3: Behavior Verification Testing

#### Step 3.1: Identify Observable Behaviors

**Action**: List what the code should actually do (not just that it returns Ok/Err).

**Behaviors to verify**:
- State changes after operations
- Output values match expectations
- Execution order is correct
- Side effects occur as expected

#### Step 3.2: Create Behavior Tests

**Action**: Test observable behaviors, not just function existence.

```rust
#[test]
fn test_registry_registers_noun() -> Result<()> {
    // Arrange: Create registry
    let mut registry = CommandRegistry::new();
    
    // Act: Register noun
    let noun = noun!("services", "Manage services", []);
    registry.register_noun(Box::new(noun))?;
    
    // Assert: Verify observable behavior - noun appears in command
    let command = registry.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "services"),
            "Noun should appear in command structure");
    Ok(())
}
```

**CRITICAL**: This test verifies actual behavior (noun appears in command structure), not just that `register_noun` returns `Ok`.

---

### Pattern 4: AAA Pattern (Arrange-Act-Assert)

#### Step 4.1: Structure Tests with AAA

**Action**: Always use clear AAA structure with comments.

```rust
#[test]
fn test_verb_executes_with_args() -> Result<()> {
    // Arrange - Set up test data
    let verb = verb!("status", "Show status", |args: &VerbArgs| {
        let value = args.get_one_str("service")?;
        assert_eq!(value, "api");
        Ok(())
    }, args: [
        Arg::new("service").required(true)
    ]);

    // Act - Execute the code under test
    let matches = Command::new("test")
        .arg(Arg::new("service").required(true))
        .get_matches_from(vec!["test", "api"]);
    let verb_args = VerbArgs::new(matches);

    // Assert - Verify the results
    verb.run(&verb_args)?;
    Ok(())
}
```

---

### Pattern 5: Real Collaborators (Minimize Mocks)

#### Step 5.1: Use Real Objects When Possible

**Action**: Prefer real objects over mocks for better test confidence.

```rust
#[test]
fn test_command_execution_with_real_args() -> Result<()> {
    // Arrange: Use real Command and ArgMatches
    let command = Command::new("test")
        .arg(Arg::new("value").required(true));
    
    // Act: Get real matches
    let matches = command.get_matches_from(vec!["test", "value"]);
    let verb_args = VerbArgs::new(matches);
    
    // Assert: Verify with real data
    assert_eq!(verb_args.get_one_str("value")?, "value");
    Ok(())
}
```

**When to use mocks**: Only when real objects are too expensive, have side effects, or are external dependencies.

---

## Complete Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{NounVerbError, Result};
    use clap::{Arg, Command};

    #[test]
    fn test_verb_command_executes_successfully_with_required_args() -> Result<()> {
        // Arrange - Set up test data
        let verb = verb!("status", "Show status", |args: &VerbArgs| {
            let service = args.get_one_str("service")?;
            assert_eq!(service, "api");
            Ok(())
        }, args: [
            Arg::new("service").required(true)
        ]);

        // Act - Execute the code under test
        let matches = Command::new("test")
            .arg(Arg::new("service").required(true))
            .get_matches_from(vec!["test", "api"]);
        let verb_args = VerbArgs::new(matches);

        // Assert - Verify the results
        verb.run(&verb_args)?;
        Ok(())
    }

    #[test]
    fn test_verb_command_fails_with_missing_required_arg() {
        // Arrange
        let verb = verb!("status", "Show status", |_args: &VerbArgs| Ok(()), args: [
            Arg::new("service").required(true)
        ]);

        // Act
        let matches = Command::new("test")
            .arg(Arg::new("service").required(true))
            .get_matches_from(vec!["test"]); // Missing required arg

        // Assert - Should fail to parse
        assert!(matches.is_err());
    }
}
```

## Best Practices

1. **Test error paths first** - 80% of bugs are in error handling
2. **Verify behavior, not existence** - Test what code does, not just that functions exist
3. **Use AAA pattern** - Clear Arrange-Act-Assert structure
4. **Test boundaries** - Empty, single, max, zero, negative
5. **Use real collaborators** - Minimize mocks
6. **Fast tests** - Each test should complete in <100ms to allow full suite in <1s
7. **Descriptive names** - Test names should describe what they verify

## Anti-Patterns to Avoid

### ❌ Meaningless Tests

```rust
// ❌ BAD: Only checks function exists, doesn't verify behavior
#[test]
fn test_registry_creation() {
    let registry = CommandRegistry::new();
    assert!(registry.is_ok()); // Doesn't verify actual behavior
}
```

### ✅ Behavior Verification

```rust
// ✅ GOOD: Verifies actual behavior
#[test]
fn test_registry_creates_valid_command() -> Result<()> {
    let registry = CommandRegistry::new()
        .name("test")
        .register_noun(noun!("services", "Manage services", []))?;
    
    let command = registry.build_command();
    assert!(command.get_subcommands().any(|cmd| cmd.get_name() == "services"),
            "Noun should appear in command structure");
    Ok(())
}
```

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules and standards
- **[Verify Tests Command](./verify-tests.md)** - Test verification workflow
- **[80/20 Fill Gaps](./80-20-fill-gaps.md)** - Capability completion

## Quick Reference

```rust
// Error path testing
#[test]
fn test_error_paths() -> Result<()> {
    // Test all error variants
    Ok(())
}

// Boundary testing
#[test]
fn test_boundaries() -> Result<()> {
    // Test empty, single, max, zero
    Ok(())
}

// Behavior verification
#[test]
fn test_observable_behavior() -> Result<()> {
    // Verify state changes, outputs, execution order
    Ok(())
}
```

