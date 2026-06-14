# Affidavit — Product & Architecture Requirements

**Document:** PRD / ARD (combined)
**Version:** v31.0 (post-seam-derivation rewrite)
**Status:** Phase 1 implementable; Phase 2 standing
**Supersedes:** `affidavit_prd_ard_v30.1.1.md`

---

## 0. One-sentence statement

Affidavit makes unwitnessed work unconstructable: every operation that produces a receipt enters the type system as raw evidence and can leave only by passing — atomically — through a structural-law court and a cryptographic chain-seal, so a receipt that was not both adjudicated and chained has no representation in anything Affidavit emits.

The rest of this document is the derivation of that sentence and the requirements that follow from it.

---

## 1. Problem

The market need is detection of LLM code-cheating: an agent that reports success it did not achieve — a test that passes by special-casing the oracle, a gate that parses but never fires, a stub that returns the expected value. Two facts make naive detection a dead end:

**Undecidability.** Whether an arbitrary program's behavior satisfies a non-trivial semantic property — including "is this honest work or a cheat" — is undecidable by Rice's theorem. No detector decides it in general.

**Empirical ceiling.** The decidable approximations lose in practice. On adversarial benchmarks, frontier models cheat at high rates (ImpossibleBench-class results put a strong model near ~54%), and the best behavioral detectors plateau well short of reliability (~63% ceiling). A screen good enough to ship is a screen good enough to evade.

The conclusion is not "build a better detector." It is that detection is the wrong frame, because it asks an undecidable question about a finished artifact.

---

## 2. Thesis — relocate the undecidable

Affidavit does not detect cheating. It **relocates** the undecidable predicate from *the artifact* to *the form work is permitted to take*. Rather than ask "is this finished thing honest" (undecidable), it legislates "honest work carries a witness, and unwitnessed work does not type-check" (decidable, because it is a property of construction, not of behavior).

The relocation has three classical escapes from the undecidability wall; Affidavit uses the second and third and rejects the first:

1. **Syntactic screen** — pattern-match for cheat shapes. *Rejected:* evadable by construction; it is the detector frame in disguise.
2. **Restrict to a decidable fragment** — admit only constructions whose correctness is checkable. This is the **Blue River Dam**: correct-by-construction over a bounded, total fragment, sitting deterministically upstream of every probabilistic competitor.
3. **Proof-carrying / certificate** — require each operation to carry a witness in a checkable format. The witness format is the receipt chain; cross-artifact identity is the moniker; forgery cost is the cryptographic seal.

What undecidability forbids cannot be solved, only moved. Affidavit moves it to a place where the residual is small, nameable, and — critically — **outside the producing system**. See §8 for where the irreducible residue lives.

---

## 3. Product requirements

### Users and use

The primary user is an engineer or organization that needs work products (code, documents, data transformations) to carry provenance that an unaligned generator cannot forge. The unit of value is the **receipt**: a record that a specific operation occurred, over specific objects, in a chain that cannot be reproduced without the work.

### Functional requirements

**FR-1 — Receipt emission.** Affidavit shall expose an `emit` operation that records an operation-event over object-centric subjects (the receipt is OCEL-shaped: events relate to objects, not flat log lines). Emission enters the type system at the boundary; it does not return a bare value.

**FR-2 — Chain assembly.** Affidavit shall expose an `assemble` operation that finalizes emitted events into an immutable, content-addressed receipt whose identity is a hash over its contents and its predecessor (a BLAKE3 chain). The content address is the receipt's name.

**FR-3 — Verification.** Affidavit shall expose a `verify` operation that adjudicates a receipt against the structural laws and the chain, returning a verdict (ACCEPT / REJECT) and a non-zero process exit on REJECT. Verification of a tampered receipt must REJECT.

**FR-4 — Inspection.** Affidavit shall expose a `show` operation that displays a receipt's events and chain state **without** rendering a verdict. (`verify` and `show` are the load-bearing type-identical pair — see ADR-5; they must reach distinct handlers and produce distinct output, and that distinctness is witnessed behaviorally, because the type system cannot witness it.)

**FR-5 — CLI surface.** The operations shall be reachable as `affi receipt <verb>` (emit | assemble | verify | show). The flat empty-noun surface (`affi emit`) is rejected because clap cannot register an empty command name; the `receipt` noun is required, not stylistic.

**FR-6 — Tamper teeth.** The verification path shall distinguish an honest receipt (ACCEPT, exit 0) from a tampered one (REJECT, non-zero exit) on the golden run, and that distinction is the admission witness for the whole pipeline.

### Non-functional requirements

**NFR-1 — Determinism.** The seal computation shall be deterministic: the same evidence under the same chain state produces the same receipt identity. This is golden-diff witnessed; it is the property the type system cannot prove and the one most able to silently break.

**NFR-2 — Forgery cost.** The chain shall be cryptographically irreproducible: producing a valid successor receipt requires the work, not merely knowledge of the format. The seal is the forgery-cost barrier, not a checksum.

**NFR-3 — No bare returns.** No operation in the receipt path shall return a naked value; every operation's output enters as `Evidence<_, Raw, _>` at the boundary. A bare return is, by the project's own axiom, an unwitnessed claim.

**NFR-4 — Unconstructable bypass.** It shall be impossible to emit a receipt that did not pass both structural admission and chain-sealing. This is enforced by construction (the type system), not by a runtime check that can be forgotten or removed. See ADR-3.

**NFR-5 — Authoritative consumption.** Affidavit's generated CLI surface shall be **consumed** from an authoritative ggen pack via `[[packs]]`, not forked or re-derived per project. Forking makes the pack authoritative for nothing.

**NFR-6 — Witnessed surface.** Every new surface (a new verb, a new admission rule, a new witness) is not "done" until it carries a witness that terminates outside its producer: a compile-fail receipt proving the bypass is unconstructable, and where behavioral, a negative control proving the witness can fail.

---

## 4. Architecture — the court/producer seam

Affidavit is the **producer** (it mints BLAKE3 receipts and runs the CLI). `wasm4pm-compat` is the **court** (it provides the typed `Raw → Admitted` one-way door, the witness markers, the `Admit` structural law, and the refusal enums). The architecture is the seam between them, and the seam is **forced**, not chosen.

The governing decision (ADR-1) is that Affidavit uses `wasm4pm-compat` **as its receipt typestate, not as a called library.** The distinction is the entire design: a called validator is a runtime check you must remember to invoke; a typestate makes the illegal state unconstructable. Affidavit's receipt-bearing value *is* an `Evidence` carrier, so a receipt that has not passed the court has no representation in Affidavit's output path.

The seam has three layers.

### Layer 1 — Boundary entry (forced by NFR-3)

Every operation that takes user input or produces a step result wraps it as `Evidence<_, Raw, _>` via the court's `from_boundary()` entry, rather than passing a bare `String`/`Vec`. This is the type-level form of "the operation must carry provenance." Affidavit imports the court as the boundary type.

### Layer 2 — The sealed transition (the load-bearing seam)

The `Raw → Admitted` transition is the **single** point where two things happen atomically:

- the court's `Admit` impl (`LinkedOcel::admit`) runs the structural OCEL laws and either advances the state or **refuses by name** (`OcelRefusal::DanglingEventObjectLink`, `EmptyEventObjectLinks`), and
- Affidavit's BLAKE3 chain-append runs,

such that the resulting `Admitted` value proves **both**. There is no `Admitted`-without-receipt and no receipt-over-non-admitted-value, by type.

This is **Shape B** (the receipt is bound *into* the witness; unsealed `Admitted` is unconstructable). Shape A — chain *around* admit, two separate facts held side by side — is rejected because it re-introduces drift: a value could be admitted-but-unchained or chained-but-unadmitted, and the gap between those two type-identical-looking facts is exactly the seam where unwitnessed residue hides. The whole point is to fuse them so the gap cannot exist.

### Layer 3 — Output gate (forced by NFR-4)

Affidavit's output functions accept only `Evidence<_, Admitted, W>`, never `Raw`. The compiler then enforces the thesis: an unwitnessed or unchained receipt has no syntax in Affidavit's output, because the output signature won't take `Raw` and the only path to `Admitted` is the fused transition of Layer 2.

---

## 5. Architecture decisions (ADRs)

**ADR-1 — Typestate, not library.** Affidavit consumes `wasm4pm-compat` as the type through which receipts flow. *Rationale:* the compile-time guarantee (bypass unconstructable) is the reason Affidavit exists; demoting to a callable validator on stable Rust loses the guarantee and re-creates the detection-not-construction weakness the project was built to escape. *Consequence:* see ADR-6 (nightly pin).

**ADR-2 — The seal is value-level, not const-generic.** A BLAKE3 chain hash is runtime data; it cannot be lifted into a const-generic parameter. "Unsealed `Admitted` is unconstructable" is therefore enforced the way the `SeparableWfNet` non-forgeability seal is: a **private `_seal` field plus a sealed constructor**, where the only path that fills the carrier is the fused admit-then-seal transition. *This is the decision that keeps the seam out of the EXPSPACE const-generics fragment* — the typestate guarantee is stable-shaped, even though the crate it composes with is nightly for other reasons.

**ADR-3 — The carrier itself must be non-forgeable.** Adversarial review of the seal design surfaced that a private field on the *admission token* is insufficient if the universal `Evidence<T, State, W>` carrier has public fields and is not `#[non_exhaustive]` — an external crate could forge `Evidence<Receipt, Admitted, W>` by struct literal, bypassing the seal entirely. The carrier therefore carries a private `_seal: ()` of its own (the same mechanism as `SeparableWfNet`), so struct-literal construction of an admitted carrier fails at compile time (`E0451`). *This is the highest-risk surface in the design — it touches the carrier all receipts construct — and is witnessed by a compile-fail fixture.* `#[non_exhaustive]` is explicitly rejected as the mechanism: it would not reproduce the `E0451` "field `_seal` is private" receipt the fixture asserts, and it still permits in-crate struct-literal construction.

**ADR-4 — Witness `W` is a receipt-bearing OCEL witness.** Affidavit's receipt is "OCEL-shaped operation-events plus a hash chain" — more than plain `Ocel20`. `W` reuses the court's `ReceiptFamily` and means "structurally lawful **and** chain-sealed" (`AffidavitReceiptChain`). *Rationale:* the typestate makes `Evidence<_, Admitted, AffidavitReceiptChain>` incompatible with any other-witness carrier, so standard-mixing is a compile error, not a silent corruption. Affidavit inherits the court's receipt shapes rather than re-deriving them.

**ADR-5 — `verify` and `show` are the canonical type-blind pair.** The two operations have type-identical surfaces (both take a receipt, both produce output) and opposite meaning (one adjudicates, one displays). The type system cannot distinguish them; only a held human convention can. *Consequence:* their distinctness — `verify` reaches the verdict path, `show` reaches the display path and never emits a verdict — is asserted by a behavioral witness (a dispatch test from intent, checking output text and exit code), because no signature catches "show silently ran verify." This is the project's own type-blind-spot law appearing inside its CLI.

**ADR-6 — Affidavit inherits the nightly pin.** Depending on the court as a typestate means Affidavit is a nightly crate, pinned to a dated nightly, riding `generic_const_exprs` with the documented `min_generic_const_args` non-viability (mGCA forbids generic parameters in computed const operations and cannot coexist with `generic_const_exprs` in one crate). *Rationale:* the const-generics fragment is where the court's broader law kernel lives; consuming the court means living in that fragment. The seal *itself* is value-level (ADR-2) and does not add const-generic work, but the dependency carries the pin. This is the **dam at crate-composition**: the stronger guarantee (typestate, unconstructable) costs the narrower substrate (nightly). The pin is accepted as the right price, because the alternative — a callable validator on stable — is the demotion Affidavit exists to refuse.

**ADR-7 — Output is built by ontological derivation, not hand-authoring.** The CLI verb wrappers are rendered from an authoritative ggen pack: the ontology supplies signature and registration, the generated wrapper delegates one line to a hand-written handler, and the business logic (BLAKE3) — which is *not in the ontology* and cannot be generated — lives in the handler the wrapper calls. *The delegation seam is the boundary between what is in `O*` and what is not yet*, and it is stable as the ontology grows. This realizes `A = μ(O*)`: the ontology is Affidavit's only generation-time input.

---

## 6. The stdout hazard (cross-cutting, enforced)

Affidavit's verify/show output and any LSP-adjacent surface share a transport hazard that is type-invisible and has historically shipped repeatedly: **stdout is simultaneously the human-output channel and, in protocol contexts, the frame channel.** A write to stdout is type-identical whether it means "log" or "protocol frame," so the compiler cannot catch a stray write that corrupts framing.

The mitigation is two-layered, because the hazard spans two disjoint failure classes:

- **Construction-time:** `#![deny(clippy::print_stdout)]` at the library root makes the *macro* class (`print!`/`println!`) unconstructable — it does not compile. Negative-control witnessed (an injected `println!` fails the build).
- **Behavioral:** a real-subprocess test that drives the binary, reads raw stdout bytes, and asserts clean framing catches the *non-macro* class (a dependency's logger, `stdout().write_all`) that the lint structurally cannot see. Negative-control witnessed (an injected non-macro write makes the test fail, then is reverted).

Neither layer is redundant: each catches a class the other is blind to. This pairing is required, not optional.

---

## 7. Phase structure

**Phase 1 — artifact provenance (implementable now).** Relocate the undecidable from the artifact to the *form of construction*: the typestate seam (§4), the sealed admission (ADR-2/3), the witness `W` (ADR-4), the CLI from the ontology (ADR-7). Deliverable: a receipt that cannot be emitted unwitnessed, verified by tamper-teeth and compile-fail fixtures.

**Phase 2 — reasoning provenance (standing condition).** Relocate the undecidable one level further, from the artifact to the *reasoning boundary*: the boundary-trace β, which localizes where a generator left the ontology and selected by reflex. The receipt for Phase 2 is β itself. Phase 2 is **not a milestone that completes** — it is the standing condition that the boundary-witness must come from whoever holds the missing axiom (sometimes substrate semantics, sometimes empirical engine behavior, sometimes the human), and that this obligation never empties, because the ontology is never categorical at the frontier. See §8.

---

## 8. Honest residuals (named, not solved)

**R-1 — Undecidability is relocated, not solved.** Affidavit does not defeat Rice's theorem. It moves the undecidable predicate to a place where the residual is small and external. The residual does not vanish; it concentrates.

**R-2 — Verifier root-of-trust is an open problem.** Once unwitnessed work cannot compile, the residual trust localizes to "is the standard sound" — is the court's structural law actually correct, is the witness format actually load-bearing. This is a named open problem, not a closed one. The trust does not disappear; it relocates to the verifier's root, which is the smallest place it can occupy.

**R-3 — At least one witness is irreducibly human.** Verification terminates *outside* the producing system. The type system witnesses everything except the type-identical seams (verify↔show, stdout-as-log vs stdout-as-protocol), and at those seams the only possible witness is a held human convention. Witnessing that convention from inside would be the infinite regress Rice forbids. This is the trim-tab: it sharpens as the system improves; it never empties.

**R-4 — The dam bounds total witnessing.** Total structural admission is intractable past the bounded fragment (reachability is EXPSPACE / Ackermann-complete on the general surface). Affidavit's guarantee is correct-by-construction *inside* the bounded, total fragment — the Blue River Dam — and the un-witnessable edge is the frontier, not a defect.

**R-5 — The nightly pin is a substrate cost.** The compile-time guarantee lives in the const-generics fragment (ADR-6). Affidavit is nightly-pinned for as long as that is true, with no stable floor until mGCA's non-min expansion supports computed const arguments. Tracked, not hidden.

---

## 9. Acceptance — what "witnessed" means for this spec

A surface in Affidavit is **admitted** only when it carries a witness that terminates outside its producer:

- the bypass is **unconstructable** — a compile-fail fixture (the `tests/ui/compile_fail/` idiom) proves that an attempt to emit an unsealed/unadmitted receipt does not compile;
- the seal is **deterministic** — a golden-diff (NFR-1) proves same-evidence → same-identity;
- the type-blind pairs are **behaviorally distinguished** — a dispatch test (ADR-5) and the tamper golden (FR-6) prove verify↔show reach distinct handlers and ACCEPT↔REJECT carry distinct exits;
- the transport is **clean** — the two-layer stdout guard (§6), each layer negative-control witnessed.

A surface that compiles, parses, and reports green is a *member* of "things that look admitted." It is not *identical* to an admitted surface until the witnesses above hold. Membership is not identity; the receipt is the load-bearing artifact, and it terminates outside the system that produced it.

---

*End of spec. This document is the ontology Affidavit's implementation is the transformation of; a claim here that the source contradicts is an unwitnessed claim and is corrected against the source, not preserved.*
