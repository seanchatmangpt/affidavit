# Gemba Walk - Multi-Step Workflow

## Purpose

This command guides agents to "go to the source" (Gemba) - work directly with actual code and data, not abstractions or assumptions. Gemba means the actual place where work happens. Experts always verify at the source.

## Workflow Overview

```
Step 1: Go to Gemba → Step 2: Observe Actual Behavior → Step 3: Verify Claims → Step 4: Create Todo List for Fixing Discrepancies → Step 5: Fix at Source
```

## Step-by-Step Instructions

### Step 1: Go to Gemba (The Actual Place)

**Action**: Read actual source code, not documentation or comments.

**Gemba locations**:
- **Source code** (`src/**/*.rs`) - The actual implementation
- **Test code** (`tests/**/*.rs`) - How code is actually used
- **Example code** (`examples/**/*.rs`) - Real usage patterns
- **Build output** (`cargo make check` output) - Actual compilation behavior

**Avoid**:
- ❌ Documentation that may be outdated
- ❌ Comments that may be wrong
- ❌ Assumptions about how code works
- ❌ Second-hand information

**Action**: Navigate to actual source files

```bash
# Read actual source code
read_file src/registry.rs

# Read actual test code
read_file tests/registry_test.rs

# Read actual example usage
read_file examples/basic.rs
```

**Principle**: "Go see, ask why, show respect" - Go to the source, understand why it works that way, respect the actual implementation.

---

### Step 2: Observe Actual Behavior

**Action**: Run code and observe what actually happens, not what should happen.

#### 2.1: Run Code

**Action**: Execute code to see actual behavior.

```bash
# Run tests to see actual behavior
cargo make test

# Run examples to see actual usage
cargo make test-integration

# Check compilation to see actual errors
cargo make check
```

#### 2.2: Trace Execution

**Action**: Follow code execution path.

**Methods**:
- Read code flow line by line
- Add debug output to trace execution
- Use debugger to step through code
- Examine stack traces

**Purpose**: Understand actual execution path, not assumed path

#### 2.3: Examine Outputs

**Action**: Look at actual outputs, not expected outputs.

**What to examine**:
- Test results (pass/fail, actual vs expected)
- Compiler errors (actual error messages)
- Runtime behavior (actual performance, actual errors)
- Data structures (actual values, actual types)

**Action**: Capture actual outputs

```bash
# Capture test output
cargo make test > test_output.txt 2>&1

# Capture compilation output
cargo make check > check_output.txt 2>&1

# Examine actual outputs
cat test_output.txt
cat check_output.txt
```

---

### Step 3: Verify Claims

**Action**: Compare claims (documentation, comments, test names) with actual behavior.

#### 3.1: Identify Claims

**Action**: Find all claims about code behavior.

**Claim sources**:
- Documentation comments (`///`, `//!`)
- Inline comments (`//`)
- Test names (`test_*`)
- README files
- API documentation

**Action**: Extract claims

```markdown
## Claims Found

### Documentation Claims
- `src/registry.rs:45` - "Registers a noun command"
- `src/builder.rs:123` - "Builds a valid command"

### Comment Claims
- `src/verb.rs:67` - "Executes the verb with given arguments"

### Test Name Claims
- `tests/registry_test.rs:12` - `test_registry_registers_noun`
```

#### 3.2: Verify Against Actual Behavior

**Action**: Check if claims match actual behavior.

**Verification steps**:
1. Read actual code implementation
2. Run code and observe behavior
3. Compare claim with actual behavior
4. Document discrepancies

**Action**: Verify each claim

```rust
// Claim: "Registers a noun command"
// Actual code:
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    let name = noun.name();
    if self.nouns.contains_key(name) {
        return Err(NounVerbError::duplicate_noun(name));
    }
    self.nouns.insert(name.to_string(), noun);
    Ok(())
}

// Verification: ✅ Claim matches - function does register noun
```

#### 3.3: Document Discrepancies

**Action**: List all discrepancies between claims and actual behavior.

```markdown
## Discrepancies Found

### Documentation vs Code
- [ ] `src/registry.rs:45` - Docs say "always succeeds" but function returns `Result`
- [ ] `src/builder.rs:123` - Docs say "validates command" but validation is optional

### Comments vs Behavior
- [ ] `src/verb.rs:67` - Comment says "synchronous" but code is async

### Test Names vs Behavior
- [ ] `tests/registry_test.rs:12` - Test name says "registers" but test only checks return value
```

---

### Step 4: Create Todo List for Fixing Discrepancies

**Action**: Create comprehensive todo list for fixing all discrepancies.

**Todo list structure**:
- Discrepancy description
- Location (file:line)
- Claim vs actual behavior
- Proposed fix
- Priority (High/Medium/Low)
- Status

**Action**: Create todos (10+ items minimum)

```markdown
## Gemba Walk Discrepancies (10+ items)

### High Priority
- [ ] Fix `src/registry.rs:45` - Update docs to reflect Result return type
- [ ] Fix `src/builder.rs:123` - Clarify validation behavior in docs
- [ ] Fix `src/verb.rs:67` - Update comment to match actual behavior

### Medium Priority
- [ ] Fix `tests/registry_test.rs:12` - Update test name or test behavior
- [ ] Fix `examples/basic.rs:34` - Update example to match actual API

### Low Priority
- [ ] Fix `README.md:56` - Update README to match current API
- [ ] Fix `docs/API.md:123` - Update API docs
```

---

### Step 5: Fix at Source

**Action**: Fix discrepancies at their source.

#### 5.1: Fix Documentation

**Action**: Update documentation to match actual behavior.

```rust
// BEFORE: Incorrect claim
/// Registers a noun command. Always succeeds.
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    // ...
}

// AFTER: Accurate documentation
/// Registers a noun command.
///
/// Returns an error if a noun with the same name is already registered.
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    // ...
}
```

#### 5.2: Fix Comments

**Action**: Update comments to match actual behavior.

```rust
// BEFORE: Incorrect comment
// Executes the verb synchronously
pub fn run(&self, args: &VerbArgs) -> Result<()> {
    // ...
}

// AFTER: Accurate comment
// Executes the verb with the given arguments
pub fn run(&self, args: &VerbArgs) -> Result<()> {
    // ...
}
```

#### 5.3: Fix Test Names

**Action**: Update test names to match actual test behavior.

```rust
// BEFORE: Test name doesn't match behavior
#[test]
fn test_registry_registers_noun() {
    let registry = CommandRegistry::new();
    assert!(registry.is_ok()); // Only checks creation, not registration
}

// AFTER: Test name matches behavior
#[test]
fn test_registry_creation() {
    let registry = CommandRegistry::new();
    assert!(registry.is_ok());
}

// OR: Fix test to match name
#[test]
fn test_registry_registers_noun() -> Result<()> {
    let mut registry = CommandRegistry::new();
    let noun = noun!("services", "Manage services", []);
    registry.register_noun(Box::new(noun))?; // Actually registers
    Ok(())
}
```

#### 5.4: Verify Fixes

**Action**: Verify all fixes are correct.

```bash
# Run tests to verify behavior
cargo make test

# Check compilation
cargo make check

# Verify documentation builds
cargo make doc
```

---

## Complete Workflow Example

```bash
# Step 1: Go to Gemba
read_file src/registry.rs
read_file tests/registry_test.rs

# Step 2: Observe Actual Behavior
cargo make test
cargo make check

# Step 3: Verify Claims
# Found: Documentation says "always succeeds" but function returns Result

# Step 4: Create Todo List
# Created 10+ todos for fixing discrepancies

# Step 5: Fix at Source
# Updated documentation to reflect Result return type
# Verified fixes with cargo make test
```

## Best Practices

1. **Always go to source** - Read actual code, not documentation
2. **Verify claims** - Don't trust documentation or comments
3. **Observe behavior** - Run code to see what actually happens
4. **Fix at source** - Update code/docs/comments where they are
5. **Verify fixes** - Run tests and checks after fixing

## Anti-Patterns to Avoid

### ❌ Trusting Documentation

```rust
// ❌ BAD: Trusting documentation without verifying
// Docs say function always succeeds, so we don't handle errors
let result = registry.register_noun(noun); // Might fail!

// ✅ GOOD: Verifying actual behavior
// Checked source code - function returns Result, must handle errors
let result = registry.register_noun(noun)?; // Proper error handling
```

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules and standards
- **[Verify Tests](./verify-tests.md)** - Test verification workflow
- **[80/20 Fill Gaps](./80-20-fill-gaps.md)** - Capability completion

## Quick Reference

```bash
# Go to source
read_file src/file.rs

# Observe behavior
cargo make test
cargo make check

# Verify claims
# Compare docs/comments with actual code

# Fix at source
# Update code/docs/comments
```

