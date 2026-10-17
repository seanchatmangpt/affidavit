# Eliminate Muda (Waste) - Multi-Step Workflow

## Purpose

This command guides agents through identifying and eliminating waste (Muda) in code. Muda refers to any activity that consumes resources without adding value. Experts recognize waste early and eliminate it systematically.

## Workflow Overview

```
Step 1: Identify Muda → Step 2: Measure Waste Impact → Step 3: Eliminate Waste → Step 4: Verify Value Stream → Step 5: Control
```

## Step-by-Step Instructions

### Step 1: Identify Muda (Waste)

**Action**: Scan code for the seven types of waste.

**Types of Muda to identify**:

1. **Over-processing** - Code that does more than necessary
   - Unnecessary abstractions
   - Premature optimization
   - Features not used

2. **Waiting** - Code blocked by dependencies or delays
   - Synchronous operations that could be async
   - Blocking I/O in hot paths
   - Unnecessary serialization

3. **Transportation** - Moving data unnecessarily
   - Unnecessary copies/clones
   - Passing data through multiple layers
   - Redundant data transformations

4. **Inventory** - Code that accumulates without value
   - Dead code
   - Unused dependencies
   - Commented-out code

5. **Motion** - Unnecessary code movement
   - Repeated patterns that could be abstracted
   - Copy-paste code
   - Duplicate logic

6. **Defects** - Code that causes rework
   - Error-prone patterns
   - Missing error handling
   - Incomplete implementations

7. **Over-production** - Code written before needed
   - YAGNI violations
   - Premature abstractions
   - Unused features

**Action**: Create waste inventory list

```markdown
## Muda Inventory

### Over-processing
- [ ] `src/parser.rs:45` - Unnecessary abstraction layer

### Waiting
- [ ] `src/api.rs:123` - Blocking I/O in request handler

### Transportation
- [ ] `src/transform.rs:67` - Unnecessary clone() calls

### Inventory
- [ ] `src/old.rs` - Dead code file

### Motion
- [ ] `src/utils.rs:12` - Duplicate validation logic

### Defects
- [ ] `src/error.rs:89` - Missing error handling

### Over-production
- [ ] `src/future.rs` - Unused feature module
```

---

### Step 2: Measure Waste Impact

**Action**: Quantify the impact of each waste item.

**Metrics to measure**:
- **Lines of code** - How much code is waste?
- **Complexity** - Cyclomatic complexity of waste code
- **Dependencies** - How many dependencies are unused?
- **Performance impact** - Does waste affect performance?
- **Maintenance cost** - How much time spent maintaining waste?

**Action**: Prioritize waste elimination

**Priority order**:
1. **High impact, low effort** - Quick wins
2. **High impact, high effort** - Plan carefully
3. **Low impact, low effort** - Do when convenient
4. **Low impact, high effort** - Avoid

---

### Step 3: Eliminate Waste

**Action**: Remove waste systematically.

**Elimination strategies**:
- **Dead code** - Delete unused code
- **Unused dependencies** - Remove from `Cargo.toml`
- **Duplication** - Extract to shared function
- **Unnecessary clones** - Use references
- **Over-processing** - Simplify code

**Action**: Eliminate waste

```rust
// BEFORE: Unnecessary clone
fn process(data: String) -> String {
    data.clone().to_uppercase() // Unnecessary clone
}

// AFTER: Use reference
fn process(data: &str) -> String {
    data.to_uppercase() // No clone needed
}
```

**Verification**:
```bash
# Check compilation
cargo make check

# Run tests
cargo make test

# Verify waste eliminated
# Code is simpler, faster, easier to maintain
```

---

### Step 4: Verify Value Stream

**Action**: Verify that eliminating waste improved value stream.

**Verification steps**:
1. Code compiles
2. Tests pass
3. Performance improved (if applicable)
4. Code is simpler/maintainable

**Action**: Verify value stream

```bash
# Run full validation
cargo make verify

# Check that waste is gone
# Code is cleaner, faster, easier to maintain
```

---

### Step 5: Control

**Action**: Prevent waste from returning.

**Controls**:
- **Code reviews** - Catch waste early
- **Linting** - Automated waste detection
- **Documentation** - Document waste patterns to avoid
- **Standards** - Include waste elimination in standards

---

## Complete Workflow Example

```markdown
## Muda Identified
- Dead code in `src/old.rs`
- Unnecessary clones in `src/transform.rs`

## Impact Measured
- Dead code: 200 lines, no dependencies
- Clones: 5 instances, performance impact

## Waste Eliminated
- Deleted `src/old.rs`
- Replaced clones with references

## Value Stream Verified
- Code compiles, tests pass
- Performance improved
- Code is simpler

## Controls Established
- Added lint rule to detect unnecessary clones
- Updated code review checklist
```

## Best Practices

1. **Identify systematically** - Use checklist of waste types
2. **Measure impact** - Quantify waste before eliminating
3. **Prioritize** - Focus on high-impact, low-effort waste
4. **Verify** - Always verify waste elimination improved code
5. **Control** - Prevent waste from returning

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules
- **[Eliminate Mura](./eliminate-mura.md)** - Standardization
- **[Kaizen Improvement](./kaizen-improvement.md)** - Continuous improvement

