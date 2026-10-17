# Kaizen (Continuous Improvement) - Multi-Step Workflow

## Purpose

This command guides agents to make small, incremental improvements rather than big rewrites. Kaizen means "change for better" - continuous small improvements that compound over time. Experts make many small improvements rather than waiting for perfect solutions.

## Workflow Overview

```
Step 1: Identify Opportunity → Step 2: Plan Change → Step 3: Do (Implement) → Step 4: Check (Verify) → Step 5: Act (Standardize)
```

## Step-by-Step Instructions

### Step 1: Identify Improvement Opportunity

**Action**: Find a small, focused improvement opportunity.

**Opportunity criteria**:
- **Small**: Can be done in minutes, not hours
- **Focused**: Addresses one specific thing
- **Safe**: Low risk of breaking things
- **Value**: Adds value (clarity, performance, maintainability)

**Types of opportunities**:
1. **Code clarity** - Make code more readable
2. **Performance** - Small performance improvement
3. **Maintainability** - Easier to maintain
4. **Error prevention** - Prevent a class of errors
5. **Consistency** - Match existing patterns

**Action**: List improvement opportunities

```markdown
## Kaizen Opportunities

### Code Clarity
- [ ] Extract magic number to named constant
- [ ] Add clarifying comment
- [ ] Rename variable for clarity

### Performance
- [ ] Use reference instead of clone
- [ ] Remove unnecessary allocation
- [ ] Optimize hot path

### Maintainability
- [ ] Extract repeated pattern to function
- [ ] Simplify complex expression
- [ ] Remove dead code

### Error Prevention
- [ ] Add type safety (see [Poka-Yoke Design](./poka-yoke-design.md))
- [ ] Add validation
- [ ] Handle edge case

### Consistency
- [ ] Match naming convention
- [ ] Match code style
- [ ] Match error handling pattern
```

---

### Step 2: Plan Change

**Action**: Design minimal change that improves code.

**Improvement statement**:
- **What**: What will change?
- **Why**: Why is this improvement valuable?
- **How**: How will it be implemented?
- **Risk**: What could go wrong?

**Example improvement statement**:
```markdown
## Improvement Plan

**What**: Extract magic number `42` to named constant `DEFAULT_TIMEOUT_SECONDS`
**Why**: Makes code more readable, easier to change, self-documenting
**How**: 
1. Add constant: `const DEFAULT_TIMEOUT_SECONDS: u64 = 42;`
2. Replace `42` with `DEFAULT_TIMEOUT_SECONDS`
**Risk**: Low - simple refactoring, no logic change
```

---

### Step 3: Do (Implement)

**Action**: Implement the improvement.

**Implementation steps**:
1. Make the change
2. Ensure code compiles
3. Run tests to verify behavior unchanged

**Action**: Implement improvement

```rust
// BEFORE: Magic number
fn timeout() -> Duration {
    Duration::from_secs(42) // Magic number
}

// AFTER: Named constant
const DEFAULT_TIMEOUT_SECONDS: u64 = 42;

fn timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECONDS) // Self-documenting
}
```

**Verification**:
```bash
# Ensure code compiles
cargo make check

# Run tests to verify behavior unchanged
cargo make test
```

---

### Step 4: Check (Verify)

**Action**: Verify the improvement achieved its goal.

**Verification steps**:
1. Code compiles
2. Tests pass
3. Improvement goal achieved (clarity, performance, etc.)

**Action**: Verify improvement

```bash
# Check compilation
cargo make check

# Run tests
cargo make test

# Verify improvement (e.g., code is more readable)
# Review code - constant name makes intent clear
```

---

### Step 5: Act (Standardize)

**Action**: Standardize the improvement across codebase.

**Standardization steps**:
1. Apply same pattern to similar code
2. Document pattern for future use
3. Update style guide if needed

**Action**: Standardize improvement

```markdown
## Standardization

1. Applied same pattern to other timeout values
2. Documented pattern in code style guide
3. Updated team standards
```

---

## Complete Workflow Example

```markdown
## Opportunity
Extract magic number to named constant

## Plan
Add `DEFAULT_TIMEOUT_SECONDS` constant, replace magic number `42`

## Do
Implemented constant, replaced magic number

## Check
Code compiles, tests pass, code is more readable

## Act
Applied same pattern to other magic numbers in codebase
```

## Best Practices

1. **Small improvements** - Make many small changes, not few big ones
2. **Low risk** - Choose safe improvements
3. **High value** - Focus on improvements that add value
4. **Verify always** - Always check that improvement works
5. **Standardize** - Apply patterns consistently

## Anti-Patterns to Avoid

### ❌ Big Rewrites

```rust
// ❌ BAD: Big rewrite instead of small improvement
// Rewrote entire module to "improve" it

// ✅ GOOD: Small, focused improvement
// Extracted magic number to constant
```

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules
- **[Root Cause Analysis](./root-cause-analysis.md)** - Problem solving
- **[Eliminate Muda](./eliminate-muda.md)** - Waste elimination

