# Glossary

Precise definitions of the terms used across `affidavit`. Where a term maps to a
concrete type or constant in the source, that is noted. See the
[architecture overview](architecture.md) for how these fit together.

### Receipt

The unit of provenance: an append-only, content-addressed BLAKE3 chain of
operation-events plus its metadata (`format_version`, stored `chain_hash`,
profile). A receipt is the *witness* the verifier certifies. The `Receipt` type
carries a private `_seal` field so it cannot be built by struct literal — only
the canonical seam can mint one (see [the seal / E0451](#the-seal--e0451-unconstructable-bypass)).

### Operation-event

One recorded step of a process — a single link in the chain. Each event carries
an `event_type`, one or more OCEL object references, a monotonic `seq` number, a
unique id, and a payload commitment. Events are ordered by `seq`, never by
wall-clock time. The `OperationEvent` type lives in `src/types.rs`; builders are
in `src/ocel.rs`.

### OCEL object / object_ref / qualifier

Affidavit shapes events using the **OCEL** (Object-Centric Event Log) model: an
event refers to the *objects* it touched rather than a single case id.

- An **object** is an entity in the process (a file, a build, a commit…),
  identified by an id and a type.
- An **object_ref** binds an event to an object. On the CLI it is written
  `id:type` or `id:type:qualifier` (parsed by `ocel::parse_object_ref`).
- A **qualifier** is the optional third segment naming the *role* the object
  plays in that event (e.g. `input`, `output`). `o1:file:input` is the file `o1`
  in the role `input`.

### Payload commitment

The cryptographic stand-in for an event's payload. Affidavit stores
`blake3(payload)` — a digest — and **never** the raw payload itself. This keeps
receipts small and payloads private while still binding the event to exact
content: any change to the payload changes its commitment. Stage 5
(`verify_commitments`) checks only that each commitment is a well-formed BLAKE3
digest.

### Rolling chain hash

The running BLAKE3 accumulator that links events into a chain. Starting from the
genesis seed, each new link folds the previous chain hash together with the
canonical bytes of the next event:

```text
chain_hash_0 = blake3(GENESIS_SEED)
chain_hash_n = blake3(chain_hash_{n-1} || canonical_bytes(event_n))
```

The final value is stored on the receipt as `chain_hash`. Stage 3
(`chain_integrity`) recomputes it from the events and rejects any mismatch — so
editing any one event re-routes every later link and breaks the chain. Computed
in `src/chain.rs`.

### Content address

The receipt's identity: the BLAKE3 hash over the receipt's canonical bytes. When
`assemble` writes a receipt with no explicit `--out`, the content address is the
filename. Because the bytes are canonical (sorted JSON, no wall-clock, no RNG),
the same events always yield the same content address.

### Genesis seed

The fixed byte string that seeds the rolling chain hash, binding every chain to
this release of the tool. In `src/chain.rs` it is
`GENESIS_SEED = b"affidavit-v26.6.17-genesis"`, and `chain_hash_0 =
blake3(GENESIS_SEED)`. An empty receipt's chain hash equals the genesis hash.

### Profile (core/v1)

The named format standard a verifier knows how to certify. `core/v1` is the
current profile: stage 2 (`check_format`) requires `format_version == core/v1`,
and stage 6 (`evaluate_profile`) requires each event to carry an `event_type`
and a commitment. A receipt declaring a profile this verifier does not know
fails at stage 2. Represented by the `ProfileId` type.

### Verdict

The pipeline's decidable output: **ACCEPT** or **REJECT**. ACCEPT iff every
stage passed; REJECT otherwise, carrying the first failing stage and its reason.
ACCEPT maps to process exit code `0`, REJECT to non-zero. The `Verdict` type is
in `src/types.rs`; per-stage results are `CheckOutcome` values.

### The seal / E0451 unconstructable bypass

The mechanism that makes "fake a receipt" a *compile error* rather than a
runtime check. `Receipt` has a private field `_seal`, so external code cannot
build one with a struct literal — the compiler rejects it with **E0451**
("field `_seal` of struct `Receipt` is private"). The only way to obtain a
sealed receipt is the canonical seam `chain::ChainAssembler::finalize`. This is
the doctrine "the bypass is unconstructable": the undecidable predicate is
relocated to a construction boundary the type system enforces. A compile-fail
fixture (`tests/ui/compile_fail/receipt_private_seal.rs`) witnesses it.

### Determinism (no wall-clock / no RNG / canonical JSON)

The property that the same inputs always produce the same receipt and the same
verdict. It rests on three rules:

- **No wall-clock** — ordering uses a monotonic `seq`, never timestamps.
- **No RNG and no map-iteration order** — serialization is canonical/sorted
  JSON.
- **The verifier is pure over receipt bytes** — given the same receipt it always
  returns the same `Verdict`.

Determinism is what makes content addressing and chain recomputation meaningful
across runs and machines.

### Certify-don't-decide

The governing doctrine. The verifier does **not** decide whether work was honest
(that question is undecidable, per Rice's theorem). It *certifies a witness* —
the receipt — against a fixed format standard, and every check in the pipeline
is decidable. Unverifiable work is **rejected, not detected**: a tampered or
malformed receipt simply fails a stage and yields REJECT. The verifier proves a
lawful chain exists; it does not hunt for fraud.

### SBOM (Software Bill of Materials)

A structured inventory of components, dependencies, and metadata for a software
artifact. Affidavit v26.6.17 integrates SBOM generation, parsing, and validation:
- **sbom-scan**: Generate SBOM (SPDX/CycloneDX format) from receipt or codebase.
- **sbom-compliance**: Check SBOM against NTIA minimum elements standard.
- **sbom-vulnerability**: Aggregate vulnerability data and calculate risk/blast-radius.

### Western Electric SPC (Statistical Process Control)

Real-time quality monitoring via Western Electric decision rules. Affidavit applies these rules to receipt chains and artifact metrics:
- **Anomaly detection**: Flag process shifts via rule violations (1-of-1, 9-of-9, 6-of-6, etc.).
- **Trend analysis**: Detect sustained degradation or improvement over time.
- **Portfolio health**: Monitor process control across multiple repositories or artifacts.

Implemented in `src/quality.rs` and related modules.

### OCEL (Object-Centric Event Logs)

A standard model for event logs that track *objects* and their lifecycles across a process, rather than a single case id.
Affidavit uses OCEL concepts (objects, object-refs, qualifiers) to shape events.
Full integration with OCEL tools via `src/ocel.rs`.

### DORA Metrics (DevOps Research and Assessment)

Four key metrics for engineering team velocity:
- **Deployment Frequency**: How often code reaches production.
- **Lead Time for Changes**: Time from code commit to production.
- **Mean Time to Recovery (MTTR)**: Time to restore service after a failure.
- **Change Failure Rate**: Percentage of changes that cause production incidents.

Affidavit's `dora-metrics` verb derives these from receipt event chains.

### NTIA Minimum Elements

The U.S. National Telecommunications and Information Administration's baseline for SBOM completeness. Includes:
- Component name, version, supplier, unique identifier.
- Dependency relationships and known vulnerabilities.
- License and author information.

Affidavit's `sbom-compliance` verb validates against these elements.

---

See also: the [architecture overview](architecture.md) and the
[documentation hub](README.md).
