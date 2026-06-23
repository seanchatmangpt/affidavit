# Nexus Integration Review: `capability-map` (Deep Typestate Analysis)

## 1. Project Context & Current Assessment

**Location:** `/Users/sac/capability-map`
**Analysis Target:** `cpmp` (Computer Project Mapping Protocol)

The `capability-map` project represents a significant leap in programmatic epistemology. In its current form, it operates as a sophisticated read-only semantic scanner that traverses a filesystem, applies keyword heuristics and LLM-assisted inference (via Groq), and projects a discovered inventory of codebase capabilities into an RDF graph using W3C-standard vocabularies (PROV-O, DCAT, SPDX, SKOS). It enforces strict non-deletion, demands cryptographic file hashing via BLAKE3, and meticulously chains the results into an append-only cryptographic receipt. Furthermore, it bridges static code analysis with dynamic process conformance by exporting `ocel_events` to Object-Centric Event Logs (OCEL 2.0) for validation against Partially Ordered Workflow Language (POWL) models via `pm4py`.

However, from the perspective of the **Affidavit Nexus** and the **Ostar Generative Pipeline**, `capability-map` currently operates under a critical paradigm limitation: it is fundamentally *retroactive* and *inductive*. It asks "What capabilities exist in this code?" after the code has been written. 

Under the laws of Combinatorial Maximalism and the Chatman Equation ($A = \mu(O)$—Artifacts are a strict morphological projection of the Ontology), capabilities must not be empirically guessed; they must be structurally deduced. This review details how `capability-map` must be inverted: transitioning from a heuristic scanner to a deductive typestate verifier, mapping capabilities directly to the Ostar Ontology's state-transition matrix via zero-cost Rust generics.

## 2. The Fallacy of Runtime Epistemology and Wikis

Presently, software capabilities are typically documented in human-readable wikis, architecture markdown files, or READMEs. Over time, entropy guarantees that the code will diverge from its documentation. `capability-map` attempts to solve this by generating the capability catalog directly from the code (e.g., matching regex patterns for `"BLAKE3"` or `"SPARQL"` or using an LLM to identify `"PROCESS_MINING"`). 

While this is a vast improvement over manual documentation, it still allows invalid or partial capabilities to be compiled and deployed. The system only learns it is broken or missing a capability during a later PM4Py conformance replay or an ad-hoc RDF validation. 

In the Affidavit Nexus, capabilities must not be "detected." They must be *proven* at compile-time. If a system claims to possess the `CRYPTOGRAPHIC_RECEIPT` capability, the Rust compiler itself must refuse to build the binary if the cryptographic emission law is not satisfied in the state transition. 

## 3. Translating Capability Maps to the Ostar State-Transition Matrix

To resolve this, the capability taxonomy defined in `src/rdf.rs` (e.g., `Admission`, `Conformance`, `Provenance`, `Genesis`) must be elevated from passive RDF strings (`urn:cpmp:capability:Admission`) to an active semantic law enforced by the Ostar Ontology.

The Ostar Ontology defines a target system as a precise mathematical state-transition matrix. A system moves from $State_{n}$ to $State_{n+1}$ only upon the execution of a valid $Event$. A "capability" is simply the authorization and structural mechanism that allows a specific state transition to occur. 

Instead of projecting this matrix into an external graph post-compilation, the Ostar Generative Pipeline (`ggen`) consumes the RDF triples and manufactures a Rust-based typestate pattern.

## 4. Compiling Capabilities via Rust Generics

In the proposed architecture, a capability is manifested as a zero-sized marker type in Rust. The state-transition matrix is encoded via generic constraints and type implementations.

```rust
// The capabilities, defined not in wikis, but as zero-cost type constraints.
pub trait Capability {}
pub struct CryptographicReceipt;
impl Capability for CryptographicReceipt {}

pub struct AdmissionGate;
impl Capability for AdmissionGate {}

// The State Machine encoded directly from the Ostar Ontology
pub struct SystemState<S, C: Capability> {
    state_data: S,
    _capability: std::marker::PhantomData<C>,
}

pub struct UnverifiedArtifact;
pub struct AdmittedArtifact;

// The Transition Matrix: An unverified artifact can ONLY become an AdmittedArtifact
// if the system proves it has the AdmissionGate capability.
impl SystemState<UnverifiedArtifact, AdmissionGate> {
    pub fn admit(self) -> SystemState<AdmittedArtifact, CryptographicReceipt> {
        // ... cryptographic validation logic ...
        SystemState {
            state_data: AdmittedArtifact,
            _capability: std::marker::PhantomData,
        }
    }
}
```

This effectively shifts capability mapping from the filesystem regex to the `rustc` compiler boundary. If a developer attempts to transition an `UnverifiedArtifact` to an `AdmittedArtifact` without the `AdmissionGate` capability in scope, or attempts to bypass the `admit` function, the code simply will not compile. 

Capabilities become structurally irrefutable. You do not need to query a SPARQL endpoint or read a wiki to know if the application supports "Admission Gate Validation"—you merely need to know that the binary successfully compiled. The capabilities are proven by the type signature.

## 5. Bipartite Typestate Enforcement and Process Mining

The integration of `capability-map` with `wasm4pm` for process mining is currently based on `ocel_events` scraped or logged at runtime. This, too, can be vastly strengthened. 

Because capabilities and state transitions are now encoded in Rust generics, the emission of OCEL events and BLAKE3 receipts can be bound directly to the typestate transitions. The `affidavit::emit!` macro is injected into the very functions that consume and produce the type states (e.g., the `admit` method above). 

This guarantees **Bipartite Typestate Enforcement**:
1. **Compile-time:** Rust generics guarantee that illegal capability transitions cannot be compiled.
2. **Runtime:** The verified transition paths intrinsically emit cryptographic receipts and OCEL events that structurally map 1:1 to the Ostar state-transition matrix. 

When `pm4py` consumes the resulting `cpmp-catalog.nq` or OCEL JSON, it will find a 1.0 fitness score natively, because the code was physically incapable of deviating from the topology defined by the ontology. The heuristic inductive miner is replaced by a deterministic deductive proof.

## 6. The Upgrade Action Plan for `capability-map`

To fully align `capability-map` with the Affidavit Nexus, the following architectural mutations are required:

1. **Invert the Direction of Authority:**
   `capability-map` currently reads code to generate RDF. It must be updated to read the Ostar Ontology (e.g., `affi-cli.ttl`) and verify that the target Rust code implements the required generic state markers for the claimed capabilities.
2. **Deprecate Regex Heuristics for AST Typestate Analysis:**
   Instead of using `regex` to find `"BLAKE3"` or `"Admission"`, `capability-map` should use `syn` or `rust-analyzer` to parse the Abstract Syntax Tree (AST). It will prove the capability by finding the explicit structural trait bounds and typestate implementations that map to the Ostar matrix.
3. **Formalize the Groq Fallback:**
   The LLM inference mechanism (`groq_augment`) should be repurposed. Rather than passively guessing capabilities, it should act as an architectural linter, analyzing code that *attempts* to implement a capability but fails to utilize the strict generic state bounds, and providing automated pull requests to wrap the logic in the correct Ostar-compliant typestates.
4. **Unforgeable Documentation:**
   With capabilities proven at compile time, documentation (wikis) can be auto-generated directly from the `rustdoc` trait bounds. The documentation becomes a 100% accurate mathematical reflection of the system's capabilities, eliminating drift forever.

## 7. Conclusion

By shifting `capability-map` from an inductive filesystem surveyor to a deductive typestate verifier, we close the loop on software epistemology. Capabilities cease to be aspirational documentation or heuristic guesses. They become rigorous, mathematically proven state transitions constrained by Rust generics at compile time. The Ostar Ontology dictates the law, the generics enforce the physics, and `capability-map` provides the definitive, cryptographic proof that the code adheres to reality.