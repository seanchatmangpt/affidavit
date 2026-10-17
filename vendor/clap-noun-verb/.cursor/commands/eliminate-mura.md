# Eliminate Mura (Unevenness) - Multi-Step Workflow

## Purpose

This command guides agents to eliminate unevenness (Mura) in code quality, patterns, and style. Mura refers to variability or inconsistency. Experts maintain consistent quality and patterns across the codebase.

## Workflow Overview

```
Step 1: Identify Mura → Step 2: Measure Variability → Step 3: Standardize → Step 4: Apply Consistently → Step 5: Control
```

## Step-by-Step Instructions

### Step 1: Identify Mura (Unevenness)

**Action**: Find inconsistencies in code quality, patterns, and style.

**Types of Mura to identify**:

1. **Code style inconsistency** - Different formatting, naming conventions
   - Example: Some functions use `snake_case`, others use `camelCase`
   - Example: Some use tabs, others use spaces

2. **Pattern inconsistency** - Same problem solved differently
   - Example: Error handling differs across modules
   - Example: Some use `Result`, others use `Option` for similar cases

3. **Quality inconsistency** - Different quality levels
   - Example: Some modules have tests, others don't
   - Example: Some code has error handling, others don't

4. **Complexity inconsistency** - Different complexity levels for similar problems
   - Example: Simple problem solved with complex solution in one place, simple solution in another

5. **Documentation inconsistency** - Different documentation levels
   - Example: Some functions documented, others not
   - Example: Different documentation styles

**Action**: Create Mura inventory

```markdown
## Mura Inventory

### Code Style Inconsistency
- [ ] `src/parser.rs` uses different naming convention
- [ ] `src/validator.rs` uses different formatting

### Pattern Inconsistency
- [ ] `src/api.rs` uses `Result<T, ApiError>` for errors
- [ ] `src/db.rs` uses `Result<T, DbError>` for errors

### Quality Inconsistency
- [ ] `src/core.rs` has 90% test coverage
- [ ] `src/utils.rs` has 20% test coverage

### Complexity Inconsistency
- [ ] `src/simple.rs` uses complex abstraction for simple problem
- [ ] `src/complex.rs` uses simple solution for complex problem

### Documentation Inconsistency
- [ ] Public functions in `src/api.rs` are documented
- [ ] Public functions in `src/db.rs` are not documented
```

---

### Step 2: Measure Variability

**Action**: Quantify the inconsistency.

**Metrics to measure**:
- **Style consistency** - How many style violations?
- **Pattern consistency** - How many different patterns for same problem?
- **Quality consistency** - What's the quality variance?
- **Complexity consistency** - What's the complexity variance?

**Action**: Measure variability

```bash
# Measure style consistency
cargo make fmt --check
# Count violations

# Measure test coverage consistency
cargo make test
# Compare coverage across modules
```

---

### Step 3: Standardize

**Action**: Choose one standard pattern/style for each inconsistency.

**Standardization steps**:
1. Choose best pattern (or most common)
2. Document standard
3. Create migration plan

**Action**: Standardize

```markdown
## Standardization Plan

### Error Handling
**Standard**: Use `Result<T, NounVerbError>` for all errors
**Rationale**: Consistent error type across codebase
**Migration**: Update all modules to use `NounVerbError`

### Naming Convention
**Standard**: Use `snake_case` for all functions
**Rationale**: Rust convention
**Migration**: Rename all `camelCase` functions to `snake_case`
```

---

### Step 4: Apply Consistently

**Action**: Apply standard across entire codebase.

**Application steps**:
1. Update code to match standard
2. Run formatter/linter
3. Update tests
4. Verify consistency

**Action**: Apply consistently

```rust
// BEFORE: Inconsistent error handling
fn parse_api(input: &str) -> Result<String, ApiError> { }
fn parse_db(input: &str) -> Result<String, DbError> { }

// AFTER: Consistent error handling
fn parse_api(input: &str) -> Result<String, NounVerbError> { }
fn parse_db(input: &str) -> Result<String, NounVerbError> { }
```

**Verification**:
```bash
# Format code
cargo make format

# Lint code
cargo make lint

# Run tests
cargo make test
```

---

### Step 5: Control

**Action**: Prevent inconsistency from returning.

**Controls**:
- **Formatter** - `cargo make format` enforces style
- **Linter** - `cargo make lint` catches inconsistencies
- **Code reviews** - Review for consistency
- **Documentation** - Document standards

---

## Complete Workflow Example

```markdown
## Mura Identified
- Inconsistent error handling across modules
- Inconsistent naming conventions

## Variability Measured
- 3 different error types for similar operations
- 15 functions using wrong naming convention

## Standardized
- Standard: Use `NounVerbError` for all errors
- Standard: Use `snake_case` for all functions

## Applied Consistently
- Updated all modules to use `NounVerbError`
- Renamed all functions to `snake_case`

## Controls Established
- Added lint rules to catch inconsistencies
- Updated code review checklist
```

## Best Practices

1. **Identify systematically** - Use checklist of inconsistency types
2. **Measure variability** - Quantify inconsistency
3. **Standardize** - Choose one standard pattern
4. **Apply consistently** - Update entire codebase
5. **Control** - Prevent inconsistency from returning

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules
- **[Eliminate Muda](./eliminate-muda.md)** - Waste elimination
- **[Kaizen Improvement](./kaizen-improvement.md)** - Continuous improvement

