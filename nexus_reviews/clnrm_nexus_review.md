# Nexus Integration Review: `clnrm` (Cleanroom)

## 1. Project Context & Ecosystem Positioning

**Location:** `/Users/sac/clnrm`  
**Analysis Target:** `clnrm` (Cleanroom Core & Workspace)

This document provides an exhaustive evaluation of the `clnrm` project through the rigorous lens of the **Affidavit Nexus**. The primary objective is to define exactly how `clnrm`—a framework originally conceived for sandboxed validation, stress testing, and adversarial chaos engineering—must be architecturally evolved to become a foundational pillar of Combinatorial Maximalism, Cryptographic Provenance, and Bipartite Typestate Enforcement.

In its current state, `clnrm` provides a robust suite of capabilities: Tera-based scenario templating, multi-stage validations (temporal ordering, span counting, graph topology, hermeticity), and NIST-aligned chaos orchestration (network faults, resource exhaustion, cryptographic failures, and payload mutations). However, to fully integrate into the Ostar Generative Pipeline, `clnrm` must transition from being a mere "testing utility" into a **Cryptographically Verifiable Mutation Engine**. It must act as the adversarial crucible where target implementations are structurally permuted, hermetically executed, and mathematically proven against their ontologies.

## 2. Structural Evaluation (The Chatman Equation: A = μ(O))

The core tenet of the Nexus is the Chatman Equation: Architecture (A) is the manifestation (μ) of an underlying Ontology (O). 

Currently, `clnrm` operates via heuristic TOML configuration mappings and structural mutations that lack deep semantic linkage to a unified ontological graph. The chaos experiments and sandboxed validations run independently of the foundational state definitions.

**Nexus Upgrade Path:**
The internal logic of `clnrm` must be inextricably linked to the `affi-cli.ttl` and `1000x_w3c_provo_spec.ttl` ontologies. Its execution flow must be constrained by zero-cost Rust typestates, physically preventing invalid state transitions before compilation. When `clnrm` generates a `TestPermutation` or orchestrates a `ChaosScenario`, it must not just blindly permute variables; it must traverse the semantic graph, deliberately generating mutations that challenge specific ontological boundaries (e.g., forcing a violation of a `must_precede` temporal law to verify the target system's rejection mechanics). 

## 3. AST Mutation Operators and Verifiable Ostar Ontological Events

A critical evolutionary step for `clnrm` is the formalization of its mutation strategies—particularly its Abstract Syntax Tree (AST) mutation operators and structural chaos permutations—into first-class citizens of the Ostar Generative Pipeline. 

`clnrm` generates adversarial variants via precise modifications (e.g., `CorruptHash`, `ReorderEvents`, `DropEvent`, `InjectFakeEvent`). In a true Nexus architecture, these are not merely test-time aberrations; they are **Semantic Perturbations** applied to the target's AST or intermediate representation. 

**Emitting Ostar Ontological Events:**
When `clnrm`'s mutation engine alters an AST node (for instance, dropping a required span generation call, corrupting a sequence identifier, or injecting a synthetic network fault branch), this act of mutation itself must be recorded. 
1. **Mutation as an Event:** Every AST mutation applied by `clnrm` must trigger the `affidavit::emit!` macro, yielding an Ostar ontological event representing the exact diff/transformation applied.
2. **Receipt Linkage:** This event generates a BLAKE3 cryptographic receipt. The receipt definitively proves *what* was mutated, *why* it was mutated (referencing the NIST threat model or chaos configuration), and the *exact hash* of the resulting mutant AST.
3. **Closing the Loop:** The Ostar governor can then consume these verifiable mutation events. When the target system executes and predictably fails (or successfully defends against the mutation), its resulting trace is cryptographically linked back to `clnrm`'s initial mutation receipt. This creates an unforgeable, bipartite proof of the system's resilience: we prove both that the system defended itself, and that `clnrm` genuinely applied the attack.

## 4. Sandboxed Execution Mapping to Affidavit's Conformance Checking

`clnrm` utilizes strict sandboxing mechanisms (such as gVisor or specific Docker profiles) to ensure hermeticity—validating that execution does not leak network requests, illegal file system writes, or unauthorized syscalls (`clnrm_core::validation::HermeticityValidator`).

**The Integration with wasm4pm:**
This sandboxed execution is the perfect substrate for Affidavit's `wasm4pm` (Process Mining) conformance checking. 
*   **The Sandbox as an Enclave:** Within the `clnrm` sandbox, the target implementation executes. Because the sandbox traps and records all telemetry (OTEL spans) and system interactions, the resulting execution trace is a high-fidelity, noise-free ledger.
*   **Process Mining Alignment:** These traces are fed directly into the `wasm4pm` engine. `wasm4pm` uses the Heuristic Inductive Miner and Alpha-miner algorithms to map the empirical execution trace back to the theoretical Petri net derived from the project's ontology.
*   **Conformance Proof:** `clnrm`'s `GraphValidator` and `OrderValidator` are mathematically superseded by `wasm4pm`'s conformance checking. Instead of manually defining `must_precede` arrays in TOML, the sandbox execution trace is structurally aligned against the generative model. Any deviation—such as an unexpected branch caused by a `clnrm` AST mutation—is mathematically quantified as a conformance misalignment. The sandbox guarantees that the trace is pure; `wasm4pm` guarantees that the trace conforms to the law.

## 5. Cryptographic Provenance & The Seal of Custody

How can `clnrm` generate verifiable, append-only truth?
Currently, `clnrm` produces standard application reports (JSON, JUnit) and unstructured logs. While useful for CI/CD, these are forgeable and structurally agnostic.

**Nexus Upgrade Path:**
All state-mutating actions, chaos generation phases, and sandbox execution teardowns within `clnrm` must yield a BLAKE3 cryptographic receipt via the Affidavit core library. 
If `clnrm` generates 1,000 adversarial permutations, it must emit a Merkle root representing the exact corpus generated. The testing framework then consumes this corpus, proving that the exact mutants generated by `clnrm` were the ones evaluated by the verifier. This establishes an unbroken, cryptographically sealed chain of custody from the chaos configuration down to the final verification verdict.

## 6. Specific Architectural Upgrades

To realize this vision, the following surgical upgrades must be applied to `clnrm` and its integration bindings:

1.  **Ostar-Aware Mutation Engine:** Upgrade the `clnrm_core::chaos::ChaosOrchestrator`. It must not only produce `TestPermutation` or mutant configurations but directly interface with rust-analyzer/syn to perform verifiable AST mutations on Rust source code. Every mutation must emit a typed `MutationEvent` containing the BLAKE3 hash of the original and mutated AST node.
2.  **Affidavit Receipt Emission:** Inject `affidavit::emit!` calls into every phase transition of the `clnrm` lifecycle. 
    *   Template Generation -> Emit Receipt.
    *   Sandbox Initialization -> Emit Receipt.
    *   Hermeticity Violation Caught -> Emit Receipt.
3.  **wasm4pm Trace Translation:** Deprecate or wrap the manual `OtelValidator` logic. Instead, `clnrm` must expose an adapter that streams its intercepted sandbox telemetry (spans, syscalls) directly into `wasm4pm`'s `XES` (eXtensible Event Stream) format. This allows Affidavit to perform deep inductive mining on the sandbox results rather than shallow regex/count validations.
4.  **Zero-Cost Typestate Refactoring:** Redesign `CleanroomConfig` parsing. Instead of returning a dynamic tree of optional configurations that are checked at runtime, `clnrm` must parse TOML into distinct, non-overlapping typestates (e.g., `Config<Hermetic>`, `Config<Networked>`). The Rust compiler itself will then guarantee that chaos experiments requiring network access cannot be accidentally scheduled on a `Config<Hermetic>` sandbox.

## 7. Verdict

**Status:** Requires Architectural Alignment and Deep Integration.  
**Action:** Deploy the Ostar Generative Pipeline to synthesize the boilerplate bindings between `clnrm` and the Affidavit core library. Elevate `clnrm` from a passive validation tool into an active, cryptographically verifiable AST mutation and hermetic execution engine. Treat `clnrm` as a critical adversarial sub-graph of the Universal Provenance Ontology.