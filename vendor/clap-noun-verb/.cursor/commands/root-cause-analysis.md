# Root Cause Analysis (5 Whys) - Multi-Step Workflow

## Purpose

This command guides agents through root cause analysis using the 5 Whys technique. Root cause analysis finds the underlying cause of problems, not just symptoms. Experts dig deeper to find root causes rather than fixing symptoms.

## Workflow Overview

```
Step 1: Define the Problem → Step 2: Ask Why #1 → Step 3: Ask Why #2-5 → Step 4: Verify Root Cause → Step 5: Fix Root Cause
```

## Step-by-Step Instructions

### Step 1: Define the Problem

**Action**: Clearly state the observable problem (symptom).

**Problem definition format**:
- **What**: What is the observable problem?
- **Where**: Where does it occur?
- **When**: When does it occur?
- **Impact**: What is the impact?

**Example problem definition**:
```markdown
## Problem Definition

**What**: Test fails with "assertion failed: expected 1000, got 999"
**Where**: `tests/registry_test.rs` - `test_registry_builds_command`
**When**: Approximately 30% of test runs, more frequent in CI
**Impact**: Blocks CI/CD pipeline, causes false negatives
```

**Principle**: Start with the observable symptom, not assumptions about cause.

---

### Step 2: Ask Why #1

**Action**: Ask why the problem occurred (first level).

**Why #1 question**: "Why did [problem] occur?"

**Answer format**:
- Direct cause of the symptom
- Observable fact, not assumption
- Can be verified

**Example**:
```markdown
## Why #1

**Question**: Why does the test fail with "expected 1000, got 999"?
**Answer**: The registry counts 999 nouns instead of 1000
**Verification**: Checked test output - actual count is 999
```

---

### Step 3: Ask Why #2-5

**Action**: Continue asking why until reaching root cause.

**Why chain**:
- Why #2: Why did [Why #1 answer] occur?
- Why #3: Why did [Why #2 answer] occur?
- Why #4: Why did [Why #3 answer] occur?
- Why #5: Why did [Why #4 answer] occur?

**Example**:
```markdown
## 5 Whys Chain

**Why #1**: Registry counts 999 nouns instead of 1000
**Why #2**: One noun is not being registered
**Why #3**: Duplicate noun name check fails silently
**Why #4**: Error handling uses `unwrap_or` instead of returning Result
**Why #5 (Root Cause)**: Missing proper error handling - function doesn't propagate errors
```

**Principle**: Stop when you reach a root cause that can be fixed, not a fundamental limitation.

---

### Step 4: Verify Root Cause

**Action**: Verify the root cause is correct.

**Verification steps**:
1. Check if fixing root cause would prevent the problem
2. Verify root cause exists in code
3. Test hypothesis with minimal change

**Action**: Verify root cause

```rust
// Root cause: Missing proper error handling
// Verification: Check actual code
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    let name = noun.name();
    if self.nouns.contains_key(name) {
        // Root cause: Uses unwrap_or instead of returning error
        return Ok(()); // Should return Err
    }
    self.nouns.insert(name.to_string(), noun);
    Ok(())
}
```

---

### Step 5: Fix Root Cause

**Action**: Fix the root cause, not just the symptom.

**Fix strategy**:
1. Fix root cause in code
2. Add tests to prevent recurrence
3. Verify fix works

**Action**: Fix root cause

```rust
// BEFORE: Root cause - missing error handling
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    let name = noun.name();
    if self.nouns.contains_key(name) {
        return Ok(()); // Should return error
    }
    self.nouns.insert(name.to_string(), noun);
    Ok(())
}

// AFTER: Fixed root cause - proper error handling
pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) -> Result<()> {
    let name = noun.name();
    if self.nouns.contains_key(name) {
        return Err(NounVerbError::duplicate_noun(name));
    }
    self.nouns.insert(name.to_string(), noun);
    Ok(())
}
```

**Verification**:
```bash
# Run tests to verify fix
cargo make test

# Verify error handling works
cargo make check
```

---

## Complete Workflow Example

```markdown
## Problem Definition
**What**: Test fails with "expected 1000, got 999"
**Where**: `tests/registry_test.rs`
**When**: 30% of test runs

## 5 Whys Chain
**Why #1**: Registry counts 999 nouns instead of 1000
**Why #2**: One noun is not being registered
**Why #3**: Duplicate noun name check fails silently
**Why #4**: Error handling uses `unwrap_or` instead of returning Result
**Why #5 (Root Cause)**: Missing proper error handling

## Root Cause Fix
Fixed error handling to return Result instead of silently failing
Added test to verify duplicate noun detection
Verified fix with cargo make test
```

## Best Practices

1. **Start with symptom** - Define observable problem first
2. **Ask why 5 times** - Dig deep to find root cause
3. **Verify root cause** - Don't assume, verify in code
4. **Fix root cause** - Not just symptoms
5. **Prevent recurrence** - Add tests/controls

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules
- **[Kaizen Improvement](./kaizen-improvement.md)** - Continuous improvement
- **[Poka-Yoke Design](./poka-yoke-design.md)** - Error prevention

