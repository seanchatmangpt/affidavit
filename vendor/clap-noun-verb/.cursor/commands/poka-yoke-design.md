# Poka-Yoke Design (Error Prevention) - Multi-Step Workflow

## Purpose

This command guides agents to design code that prevents errors at compile time through type safety and invariants. Poka-yoke means "mistake-proofing" - making errors impossible through design. Experts use the type system to prevent entire classes of errors.

## Workflow Overview

```
Step 1: Identify Error Modes → Step 2: Design Type-Level Prevention → Step 3: Add Compile-Time Checks → Step 4: Verify Prevention → Step 5: Document Invariants
```

## Step-by-Step Instructions

### Step 1: Identify Error Modes

**Action**: List all ways code can fail at runtime.

**Error mode categories**:

1. **Invalid state** - States that shouldn't exist
   - Example: Negative count, empty required field, invalid enum variant

2. **Invalid input** - Inputs that cause errors
   - Example: Empty string when non-empty required, None when value required

3. **Invalid operations** - Operations that fail in certain states
   - Example: Reading from closed handle, modifying immutable data

4. **Resource errors** - Resource-related failures
   - Example: File not found, network error

5. **Logic errors** - Errors in program logic
   - Example: Division by zero, index out of bounds, overflow

**Action**: Create error mode inventory

```markdown
## Error Modes Inventory

### Invalid State
- [ ] Registry can have duplicate noun names
- [ ] Command can be in invalid state after error

### Invalid Input
- [ ] Empty string passed to parser (should be non-empty)
- [ ] None passed where value required

### Invalid Operations
- [ ] Reading from finalized command
- [ ] Modifying command after build

### Resource Errors
- [ ] File not found errors
- [ ] Network connection errors

### Logic Errors
- [ ] Index out of bounds
- [ ] Integer overflow
```

---

### Step 2: Design Type-Level Prevention

**Action**: Use Rust's type system to make errors impossible.

#### 2.1: Use Newtypes for Validation

**Action**: Create newtypes that enforce invariants.

**Example**:
```rust
// ❌ BAD: Can have invalid state
struct CommandRegistry {
    nouns: HashMap<String, Box<dyn NounCommand>>, // Can have duplicates!
}

// ✅ GOOD: Type prevents invalid state
#[derive(Debug, Clone)]
struct NounName(String);

impl NounName {
    fn new(name: impl Into<String>) -> Result<Self, NounVerbError> {
        let name = name.into();
        if name.is_empty() {
            return Err(NounVerbError::invalid_structure("Noun name cannot be empty"));
        }
        Ok(Self(name))
    }
    
    fn as_str(&self) -> &str {
        &self.0
    }
}

struct CommandRegistry {
    nouns: HashMap<NounName, Box<dyn NounCommand>>, // Type prevents duplicates
}
```

#### 2.2: Use Enums for State Machines

**Action**: Use enums to represent valid states only.

**Example**:
```rust
// ❌ BAD: Can be in invalid state
struct CommandBuilder {
    is_built: bool,
    is_finalized: bool,
    // Can have both true - invalid state!
}

// ✅ GOOD: Enum prevents invalid states
enum BuilderState {
    Building,
    Built(Command),
    Finalized,
}

struct CommandBuilder {
    state: BuilderState, // Only valid states possible
}

impl CommandBuilder {
    fn build(self) -> Result<Command, NounVerbError> {
        match self.state {
            BuilderState::Building => {
                // Build command
                Ok(Command::new())
            },
            _ => Err(NounVerbError::invalid_structure("Builder already used")),
        }
    }
}
```

#### 2.3: Use Option/Result for Nullable Values

**Action**: Use `Option<T>` and `Result<T, E>` instead of nullable types or panics.

**Example**:
```rust
// ❌ BAD: Can panic, causes runtime error
fn get_argument(matches: &ArgMatches, name: &str) -> String {
    matches.get_one::<String>(name).unwrap().clone() // Panics if missing!
}

// ✅ GOOD: Type forces handling of None
fn get_argument(matches: &ArgMatches, name: &str) -> Result<String, NounVerbError> {
    matches.get_one::<String>(name)
        .cloned()
        .ok_or_else(|| NounVerbError::missing_argument(name))
}
```

#### 2.4: Use PhantomData for Type-Level Invariants

**Action**: Use PhantomData to encode invariants in types.

**Example**:
```rust
use std::marker::PhantomData;

// Type-level invariant: CommandBuilder<Building> vs CommandBuilder<Built>
struct Building;
struct Built;

struct CommandBuilder<State> {
    command: Command,
    _state: PhantomData<State>,
}

impl CommandBuilder<Building> {
    fn add_noun(mut self, noun: Box<dyn NounCommand>) -> Result<Self, NounVerbError> {
        // Can only add nouns when Building
        self.command.add_noun(noun)?;
        Ok(self)
    }
    
    fn build(self) -> CommandBuilder<Built> {
        // Consumes Building builder, returns Built builder
        CommandBuilder {
            command: self.command,
            _state: PhantomData,
        }
    }
}

impl CommandBuilder<Built> {
    fn execute(&self) -> Result<()> {
        // Can only execute when Built
        self.command.execute()
    }
}

// Cannot add nouns to Built builder - compiler error!
```

---

### Step 3: Add Compile-Time Checks

**Action**: Leverage Rust's compiler to catch errors.

#### 3.1: Use Type Bounds

**Action**: Add trait bounds to restrict valid types.

**Example**:
```rust
// Function only accepts types that implement NounCommand
fn register_noun<T: NounCommand + 'static>(registry: &mut CommandRegistry, noun: T) -> Result<()> {
    registry.register(Box::new(noun))
}

// Compiler error if type doesn't implement NounCommand
```

#### 3.2: Use Const Generics for Sizes

**Action**: Use const generics to prevent size errors.

**Example**:
```rust
// Type-level size validation
struct ValidatedCommand<const MIN_VERBS: usize> {
    verbs: Vec<Box<dyn VerbCommand>>,
}

impl<const MIN_VERBS: usize> ValidatedCommand<MIN_VERBS> {
    fn new(verbs: Vec<Box<dyn VerbCommand>>) -> Result<Self, NounVerbError> {
        if verbs.len() < MIN_VERBS {
            return Err(NounVerbError::invalid_structure(
                format!("Must have at least {} verbs", MIN_VERBS)
            ));
        }
        Ok(Self { verbs })
    }
}

// Compile-time guarantee: ValidatedCommand<3> must have at least 3 verbs
```

#### 3.3: Use Marker Traits

**Action**: Use marker traits to prevent invalid operations.

**Example**:
```rust
// Marker trait for validated commands
trait Validated {}

struct Command {
    // ...
}

impl Validated for Command {
    // Only validated commands can be executed
}

fn execute<T: Validated>(command: &T) -> Result<()> {
    // Can only execute validated commands
    Ok(())
}

// Compiler error if trying to execute non-validated command
```

---

### Step 4: Verify Prevention

**Action**: Verify that errors are prevented at compile time.

#### 4.1: Compile-Fail Tests

**Action**: Create tests that verify compile errors for invalid usage.

```rust
#[cfg(test)]
mod compile_fail_tests {
    // These tests verify that invalid code doesn't compile
    // Use compiletest or similar tools
    
    #[test]
    #[should_panic(expected = "cannot add nouns to built command")]
    fn test_cannot_add_nouns_after_build() {
        // This should fail to compile
        let builder = CommandBuilder::new();
        let built = builder.build().unwrap();
        built.add_noun(/* ... */); // Should be compile error
    }
}
```

#### 4.2: Type Safety Verification

**Action**: Verify type safety through compilation.

```bash
# Run type checking
cargo make check

# Should compile without errors if types are correct
```

---

### Step 5: Document Invariants

**Action**: Document type-level invariants for future maintainers.

**Example**:
```rust
/// A command builder that enforces invariants at compile time.
///
/// # Type-Level Invariants
///
/// - `CommandBuilder<Building>`: Can add nouns and verbs
/// - `CommandBuilder<Built>`: Can only execute, cannot modify
///
/// # Example
///
/// ```rust,no_run
/// let builder = CommandBuilder::<Building>::new();
/// let built = builder.add_noun(noun)?.build();
/// built.execute()?;
/// ```
pub struct CommandBuilder<State> {
    // ...
}
```

---

## Complete Example

```rust
use std::marker::PhantomData;

// Type-level state machine
struct Building;
struct Built;

pub struct CommandBuilder<State = Building> {
    command: Command,
    _state: PhantomData<State>,
}

impl CommandBuilder<Building> {
    pub fn new() -> Self {
        Self {
            command: Command::new(),
            _state: PhantomData,
        }
    }
    
    pub fn add_noun(mut self, noun: Box<dyn NounCommand>) -> Result<Self, NounVerbError> {
        self.command.add_noun(noun)?;
        Ok(self)
    }
    
    pub fn build(self) -> CommandBuilder<Built> {
        CommandBuilder {
            command: self.command,
            _state: PhantomData,
        }
    }
}

impl CommandBuilder<Built> {
    pub fn execute(&self) -> Result<()> {
        self.command.execute()
    }
}

// Usage - type system prevents invalid operations
let builder = CommandBuilder::new();
let built = builder.add_noun(noun)?.build();
built.execute()?;

// This would be a compile error:
// built.add_noun(another_noun)?; // Error: CommandBuilder<Built> doesn't have add_noun
```

## Best Practices

1. **Use newtypes** - Prevent invalid values through types
2. **Use enums for state** - Prevent invalid state combinations
3. **Use Option/Result** - Force error handling
4. **Use PhantomData** - Encode invariants in types
5. **Use const generics** - Compile-time size validation
6. **Document invariants** - Help future maintainers

## Anti-Patterns to Avoid

### ❌ Runtime Validation When Compile-Time Possible

```rust
// ❌ BAD: Runtime validation
fn add_noun(&mut self, name: String) -> Result<()> {
    if name.is_empty() {
        return Err(NounVerbError::invalid_structure("Name cannot be empty"));
    }
    // ...
}

// ✅ GOOD: Compile-time validation
struct NounName(String);

impl NounName {
    fn new(name: impl Into<String>) -> Result<Self, NounVerbError> {
        let name = name.into();
        if name.is_empty() {
            return Err(NounVerbError::invalid_structure("Name cannot be empty"));
        }
        Ok(Self(name))
    }
}

fn add_noun(&mut self, name: NounName) -> Result<()> {
    // Name is guaranteed non-empty by type
    // ...
}
```

## Documentation References

- **[Core Team Best Practices](../../.cursorrules)** - Project-specific rules and standards
- **[80/20 Fill Gaps](./80-20-fill-gaps.md)** - Capability completion
- **[Expert Testing Patterns](./expert-testing-patterns.md)** - Testing patterns

## Quick Reference

```rust
// Newtype pattern
struct ValidatedType(T);

// Enum state machine
enum State { A, B, C }

// PhantomData for type-level invariants
struct Type<State> {
    _state: PhantomData<State>,
}

// Const generics
struct Array<const N: usize> {
    data: [T; N],
}
```

