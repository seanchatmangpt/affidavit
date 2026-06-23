# Nexus Integration Review: `unrdf`

**Project:** `unrdf`
**Location:** `/Users/sac/unrdf`
**Date:** 2025-06-15
**Evaluator:** Affidavit Nexus Audit

## 1. Executive Summary: The Substrate of Combinatorial Maximalism

The `unrdf` project is not merely an external dependency; it is the semantic nervous system required to fully realize the Chatman Equation ($A = \mu(O)$) within the Affidavit ecosystem. As a research-grade RDF knowledge graph platform equipped with Open Ontologies, PoWL v2 Process Mining (`wasm4pm`), and strict SHACL validation, `unrdf` provides the exact mechanisms needed to ingest, parse, and enforce Affidavit's semantic laws.

By formally acting as the logical bedrock for Affidavit’s typestate machines, `unrdf` shifts execution logic from human-authored procedural code to a pure mathematical derivation of W3C PROV-O semantic triples. This review details exactly how `unrdf` achieves this, transforming `affi-cli.ttl` from a static descriptive file into an active, breathing runtime supervisor that dictates the CLI's bipartite typestates and enforces cryptographic provenance. We evaluate `unrdf`'s internal packages—such as `@unrdf/core`, `@unrdf/hooks`, `@unrdf/daemon`, and `@unrdf/wasm4pm`—and define the upgrade path to weave them irrevocably into the fabric of the Affidavit CLI.

## 2. Ontological Ingestion: Parsing `affi-cli.ttl`

At the core of Affidavit's command-line interface lies `affi-cli.ttl`, an ontology written in Turtle format that maps the CLI surface using the `clap-noun-verb` vocabulary.

### The Role of `@unrdf/core` and SHACL
When the Affidavit build process or runtime initializes, it utilizes `@unrdf/core` to ingest and parse `affi-cli.ttl`. Using `unrdf`’s `createKnowledgeSubstrateCore`, the platform loads the RDF triples defining nouns (e.g., `affi:ReceiptNoun`, `affi:GovernanceNoun`) and verbs (e.g., `affi:EmitVerb`, `affi:AssembleVerb`).

1. **Ingestion & Graph Construction:** `unrdf` builds a high-performance in-memory RDF store. It translates semantic statements like `affi:EmitVerb cnv:belongsToNoun affi:ReceiptNoun` into an interconnected, queryable directed graph. This goes far beyond JSON parsing; it establishes formal ontological relations.
2. **SHACL Validation:** `unrdf` continuously applies SHACL (Shapes Constraint Language) policies to ensure that every declared Verb possesses its mandatory Arguments, Return Types, and Aliases. If a developer attempts to add a verb to `affi-cli.ttl` without a valid `cnv:argumentAbout`, `unrdf`'s validation engine throws a semantic fault, preventing invalid states before they even reach code generation.
3. **SPARQL Introspection:** The Ostar Generative Pipeline (`ggen`) queries this `unrdf` graph via SPARQL to discover the CLI shape. For example:
   ```sparql
   SELECT ?verb ?verbName ?argName ?valueType WHERE {
     ?verb a cnv:Verb ;
           cnv:hasVerbName ?verbName ;
           cnv:hasArguments ?arg .
     ?arg cnv:hasArgumentName ?argName ;
          cnv:valueType ?valueType .
   }
   ```
   This pure semantic query yields the exact blueprint needed to scaffold the CLI struct representations. There is no guessing or hardcoding; the command-line surface is algorithmically proven by the ontology.

## 3. Logical Bedrock for Bipartite Typestate Machines

In Affidavit, a state machine is not written; it is derived. Bipartite Typestate Enforcement requires that illegal state transitions be unrepresentable at compile time.

### Semantic Constraints as Typestate Generators
`unrdf` acts as the knowledge engine that feeds the typestate generator. Because `unrdf` understands the ontological relationships between actions, it dictates the structural bounds of the Rust code:

- **State Discovery:** The ontology defines the preconditions and postconditions of verbs like `emit` and `assemble`. `unrdf` computes the valid execution paths using transitive closure and graph traversal.
- **Zero-Cost Generation:** `ggen` consumes the path constraints calculated by `unrdf` and manufactures Rust types (e.g., `WorkingReceiptState`, `SealedReceiptState`). An `assemble` verb is mathematically mapped to consume a `WorkingReceiptState` and yield a `SealedReceiptState`.
- **Absolute Coherence:** Because the Rust typestates are generated directly from the `unrdf` query results, the compiled binary is a perfect, flawless mirror of the semantic model. The Chatman Equation is satisfied: the architecture ($A$) is purely a function of the ontology ($O$), processed through the mapping function ($\mu$) provided by `unrdf`. The logic is no longer human-written; it is a rigid crystalline structure enforced by the Rust compiler but authored by the semantic graph.

## 4. Pure Derivation of Execution Logic via W3C PROV-O

Affidavit relies heavily on the Universal Provenance Ontology (UPO), which is a formal extension of the W3C PROV-O standard (found in `1000x_w3c_provo_spec.ttl`). The execution logic of Affidavit—such as emitting receipts and assembling cryptographic witnesses—must be provably correct.

### Translating Action into Provenance
When `affi receipt emit` executes, the underlying business logic does not blindly log strings. Instead, it emits operation-events. `unrdf` proves that this logic is a pure derivation of PROV-O triples:

1. **PROV-O Alignment:** Every event emitted by the CLI is mapped by `unrdf` to a `prov:Activity`. Every receipt is a `prov:Bundle` and a `prov:Entity`. The CLI user acting as the caller is mapped to a `prov:Agent` (via UPO extensions).
2. **Knowledge Hooks (`@unrdf/hooks`):** During runtime, `unrdf` employs its Knowledge Hooks. As a new event capsule is created by Affidavit, `unrdf` intercepts the proposed state transition. The hook acts as an autonomic governance boundary.
3. **Cryptographic Sealing and The Admission Engine:** The event is validated against the PROV-O schema inside `unrdf`'s Admission Engine. If the semantic shape holds (e.g., `prov:wasAssociatedWith` links correctly to a known Agent), `unrdf` authorizes the BLAKE3 hash finalization. The logic isn't "if valid, then hash"; it is "the semantic proof of validity *is* the hashable payload."
4. **Unforgeable Traces:** This integration proves that the state machine doesn't just execute operations; it continually asserts PROV-O triples into the local graph, creating a mathematically unforgeable chain of custody.

## 5. Process Mining & Conformance (`@unrdf/wasm4pm`)

The integration of `unrdf` brings its highly potent `@unrdf/wasm4pm` module directly into the Affidavit ecosystem, solving the complex problem of behavioral validation.

### Alignment-Based Conformance and Anomaly Detection
Once the CLI executes and outputs its cryptographic receipts (the "event logs"), `unrdf` ingests these receipts.

1. **PoWL v2 Discovery:** Using the `wasm4pm` engine (the Partially Ordered Workflow Language), `unrdf` discovers the empirical process model directly from the PROV-O triples emitted by the executing Rust binaries.
2. **Ontology vs. Reality:** `unrdf` takes the normative model (the expected paths defined in `affi-cli.ttl` and `1000x_w3c_provo_spec.ttl`) and overlays the discovered empirical model.
3. **Mathematical Verdicts:** It calculates the alignment fitness score. If a user somehow circumvented the typestate and triggered an `assemble` without an `emit` (impossible under Bipartite Typestates, but testable under chaos engineering), `unrdf`'s conformance checking mathematically isolates the deviation. This closes the loop: the ontology generates the CLI, the CLI executes and emits triples, and `unrdf` verifies the triples against the origin ontology.

## 6. Generative Upgrade Path & Recommendations

To fully assimilate `unrdf` into the Affidavit Nexus, the following architectural upgrades are mandated:

1. **Deprecate Manual CLI Handlers:** All existing, manually written `affi` command dispatchers must be stripped. The `unrdf` Knowledge Engine must become the exclusive source of truth for the Ostar Generative Pipeline (`ggen`).
2. **Embed the Open Ontologies Sidecar (`@unrdf/daemon`):** Incorporate `unrdf`'s high-performance daemon and its Open Ontologies Rust binary as a sidecar to Affidavit. This allows real-time PROV-O validation of receipts before they are ever written to disk, achieving true L4/L5 autonomic resilience.
3. **Direct AST to Graph Binding:** Extend `unrdf` to map Rust AST structures (macros and traits) directly to the ontology graph. When a macro like `affidavit::emit!` is invoked, `unrdf` should statically verify the macro's shape against the `cnv:Verb` definition.
4. **Groq LLM Integration for Autonomous Repair:** Leverage the daemon's Groq integration to not only detect conformance violations but autonomously suggest graph repairs or code rectifications when the process model deviates from the ontology.

### Final Verdict
**Status:** Nexus Bedrock.
**Action:** `unrdf` is the foundational semantic engine required to unlock the ultimate vision of Affidavit. Its capacity to parse `affi-cli.ttl`, enforce W3C PROV-O semantics, and drive zero-cost typestate generation positions it as the crown jewel of the Ostar ecosystem. Execute full integration via the generative pipeline immediately.