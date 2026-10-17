# Detailed Analysis Report: `clap-noun-verb-macros`

## 1. Executive Summary
This report analyzes the procedural macro crate `clap-noun-verb-macros` (version `26.5.19`) inside the `/Users/sac/clap-noun-verb` workspace. 

The crate compiles successfully and passes all 120 internal unit tests without warnings. However, it contains a large volume of experimental "frontier" macros (such as `#[meta_aware]`, `#[federated]`, `#[spec]`, and others) that are completely unused in the main workspace's examples, CLIs, or playground. Many of these frontier macros are incomplete prototypes or contain hygiene errors (e.g., referencing internal proc-macro crate types that cannot be exported to macro consumers).

Conversely, the core `#[verb]` and `#[arg]` macros are mature, structurally sound, and feature robust, compiler-propagated compile-time error checks (Poka-Yoke guards) that validate return types, attribute syntax, cyclomatic complexity, and domain dependency boundaries.

---

## 2. Macro Attributes Breakdown

### Active & Stable Macros
1. **`#[verb]` (Attribute Macro)**:
   - Generates the CLI command registration. Parses parameters, auto-detects Option/Vec/bool wrappers, and binds them to the `CommandRegistry`.
   - Wires the function's return type into `HandlerOutput::from_data(...)` which enforces `serde::Serialize` bounds.
2. **`#[arg]` (Attribute Macro)**:
   - Acts as a placeholder/no-op on function parameters to prevent Rust from complaining about "unknown attribute" (since Rust does not natively allow custom proc-macro attributes directly on function parameters).
   - If applied directly to an item (like a function or struct), it emits a compile-time error pointing out the misuse and showing the correct pattern.
3. **`#[noun]` (Attribute Macro)**:
   - Deprecated as of `5.6.0` (nouns are now auto-detected from the file context/name at compile-time and runtime using the `file!()` macro and reading module doc comments `//!`).
   - Modifies the function to have a `#[deprecated]` compiler warning attribute, encouraging developers to remove it.

### Frontier & Experimental Macros (Unused / Prototypes)
A significant portion of the crate contains prototypes for future feature sets (flagged as `FUTURE` in comments). None of these are used outside their own macro crate unit tests:

| Macro Attribute | Source File | Purpose | Status / Release Gaps |
|---|---|---|---|
| `#[meta_aware]` | `meta_framework.rs` | Self-introspecting capabilities (RDF schema, similarity queries). | **Hygiene Bug**: Generates code referencing types like `OptimizationHint`, `Capability`, etc., which are defined inside the proc-macro crate but not emitted or re-exported, causing compilation to fail. |
| `#[federated]` | `macros/federated_network.rs` | Participates in a federated CLI network. | Unused prototype. |
| `#[advertise_capability]` | `macros/federated_network.rs` | Advertises RDF metadata to a discovery service. | Unused prototype. |
| `#[remote_invoke]` | `macros/federated_network.rs` | Generates RPC stubs for remote command execution. | Unused prototype. |
| `#[noun_level]` | `macros/fractal_patterns.rs` | Generates `FractalNoun` traits for struct hierarchy. | Unused prototype. |
| `#[verb_level]` | `macros/fractal_patterns.rs` | Generates `FractalVerb` traits for impl blocks. | Unused prototype. |
| `#[spec]` | `macros/executable_specs.rs` | Property tests and evidence collection. | Unused prototype. |
| `#[milestone]` | `macros/executable_specs.rs` | Compile-time target milestones and criteria tracking. | Unused prototype. |
| `#[invariant]` | `macros/executable_specs.rs` | Runtime invariant pre/post-conditions. | Unused prototype (generates runtime `panic!` code if invariant fails under the `invariant_panic` feature). |
| `#[competency]` | `macros/learning_trajectories.rs` | Skill proficiency level tracking. | Unused prototype. |
| `#[assessment]` | `macros/learning_trajectories.rs` | Evaluates proficiency against thresholds. | Unused prototype. |
| `#[learning_path]` | `macros/learning_trajectories.rs` | Generates path sequences for learning. | Unused prototype. |
| `#[auto_test]` | `macros/reflexive_testing_macro.rs` | Generates tests using RDF metadata. | **Stub**: Generates dummy unit tests that assert `true`. |
| `#[semantic_composable]` | `macros/semantic_composition.rs` | Enables semantic discovery and MCP integration. | Unused prototype. |

---

## 3. Code Generation, Hygiene, and Arguments

### Attribute Arguments & Parsing
- Attribute arguments are parsed using `syn` (e.g., `Parser::parse2` on punctuation vectors) and matched against allowed key-value schemas.
- In `validation.rs:323`, `ALLOWED_ARG_KEYS` defines valid fields for `#[arg(...)]` (e.g., `short`, `default_value`, `env`, `action`, etc.). Typos are verified against this list using the **Levenshtein distance** algorithm (e.g., suggesting `short` if the user writes `shrt`).

### Hygiene Status
- **Good Hygiene**: Code expansion for `#[verb]` uses fully qualified paths (e.g., `::clap_noun_verb::error::Result`, `::std::path::Path`).
- **Workspace Dependencies Requirement**: The expansion generates `#[linkme::distributed_slice(...)]` which relies on the `linkme` crate name being directly in the caller's namespace. The caller's `Cargo.toml` must declare `linkme` as a dependency (as seen in `unibit-cli/Cargo.toml`).
- **Hygiene Failures**: The experimental `#[meta_aware]` macro generates types such as `OptimizationHint` and `Capability` but does not prefix them or generate their struct declarations in the output tokens. This causes immediate compiler failures.

---

## 4. Error Handling and Propagation

The macro system enforces compile-time safety (Poka-Yoke) by parsing syntax and returning `syn::Error::to_compile_error()` immediately upon validation failure. Key checks include:

1. **Return Type Bounds**: Enforces that functions return a value (not default `()`) that implements `serde::Serialize` (e.g. `Result<T>` or `Option<T>`).
2. **Cyclomatic Complexity (FM-1.1)**: Calculates the complexity of `#[verb]` functions. If it exceeds **5**, it throws a compiler error forcing the user to extract the business logic into a separate domain module, keeping the CLI layer thin.
3. **Domain Isolation (FM-1.2)**: Scans parameter types. If any CLI-specific type like `ArgMatches`, `Command`, `VerbContext`, `VerbArgs`, or `HandlerInput` is found in the domain function parameters, it rejects compilation to prevent domain contamination.
4. **Helpful Syntax Suggestions**: If invalid syntax is used in `#[verb]` (e.g. not passing a string literal), it suggests correct double-quoted formats.

---

## 5. Compiler Warnings, Panics, and Stubs Scan

- **Compiler Warnings**: There are no warnings when compiling the macro crate. The crate configures `#![allow(dead_code)]` at module levels to suppress warnings on unimplemented frontier features.
- **Panics**:
  - No compile-time panic points exist in the parsing/expansion phase of the macros.
  - Two generated `panic!` expressions are found in `src/macros/executable_specs.rs` (lines 474 and 496). These are generated as runtime invariant checks to panic if an invariant fails when `feature = "invariant_panic"` is active. This is correct runtime behavior.
- **Stubs & Placeholders**:
  - `src/macros/reflexive_testing_macro.rs`: `generate_basic_test_cases` generates stubs (`test_my_function_basic`, `test_my_function_property`, and `test_my_function_edge_cases`) containing `assert!(true, "Auto-generated test...")`.

---

## 6. Outstanding Release Gaps
1. **Unused Telemetry Validation**: `telemetry_validation.rs` defines a compile-time span registry and validator to detect dead spans (RPN 48 failure mode). However, the calls to these functions (`generate_span_registry`, `validate_span_usage`, `generate_verb_instrumentation`) are commented out and disabled in `lib.rs` (with a no-op placeholder: `let _telemetry_instrumentation = ();`).
2. **Unused I/O Detection**: `io_detection.rs` is compiled but completely unreferenced by the `#[verb]` macro. It remains a placeholder for `v4.0 Phase 2` automatic wiring.
3. **Unused RDF Ontology Code**: `rdf_generation.rs` is compiled but not hooked up to any active macro, remaining a placeholder for `v5.1`.
4. **Unused Frontier Modules**: 90% of files inside `src/macros/` (`economic_simulation.rs`, `learning_trajectories.rs`, `fractal_patterns.rs`, `federated_network.rs`, `semantic_composition.rs`) are compiled but completely unintegrated with the primary workspace libraries and binaries.
