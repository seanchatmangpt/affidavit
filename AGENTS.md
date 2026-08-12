# AGENTS.md — Affidavit execution doctrine

This file is operational ground truth for repository work. Read nested
`AGENTS.md` files before changing a governed subtree.

## 1. Mission

Affidavit is the Chatman ecosystem's small provenance/certification kernel. Its
prime directive is **certify, don't decide**: construct sealed evidence carriers,
validate bounded witnesses, and refuse malformed claims. Planning, policy
selection, and machine-state actuation are outside this kernel unless a specific
boundary explicitly grants them.

The root crate is buildable. Current dependency policy is deliberately bounded:

- `wasm4pm-compat = 26.8.7` is the real published structural
  admission/authority dependency;
- `wasm4pm` remains fenced by `stubs/wasm4pm`;
- `clnrm-core` remains fenced by `stubs/clnrm-core`.

A stub is a named capability boundary, not proof that upstream behavior executed.
Do not silently replace or broaden one without an observed integration proof.

## 2. Foundational invariants

1. **Exact subject** — execution standing names the exact candidate SHA/tree that
   was actually executed.
2. **Admission before crown** — UNKNOWN is not admitted; PARTIAL_ALIVE is not
   ALIVE.
3. **Zero forged receipts** — a struct or JSON object named `receipt` has no
   standing unless the canonical verifier accepts it.
4. **Certify ≠ decide** — receipt construction carries no ambient policy or
   actuation authority.
5. **Generated surfaces are projections** — `src/verbs/**` is generated from the
   ggen ontology/config. Edit its authoritative inputs, then regenerate.
6. **Replay is evidence** — exact command/toolchain/config identity matters.
7. **One failed edge is topology** — classify the failing transition; do not
   generalize one transport or court failure into whole-project failure.

## 3. Architecture map

- `src/types.rs`, `src/chain.rs`, `src/admission.rs`, `src/verifier.rs` — receipt
  trust path.
- `src/standing.rs` — ecosystem standing v2. ALIVE is structurally
  unconstructable without successful execution + independent verification +
  replay evidence.
- `src/errc.rs` — formal ERRC v1 transformation receipts. This is CONSTRUCT only;
  see `docs/ERRC.md`.
- `ontology/affi-cli.ttl`, `ggen.toml`, `.ggen/**` — authoritative CLI generation
  graph and templates.
- `src/verbs/**` — generated CLI projection; do not hand edit.
- `affidavit-core/` — separately gated minimal core.
- `web/`, `tools/confevo/` — independent evidence lanes.
- `scripts/ci_errc.py` — exact-head ERRC fast court, reconstituted from pinned
  `ggen-legacy` mechanics.

## 4. ERRC reconstruction law

The normative profile is `affidavit/errc/v1`. Its archaeological source is
fixed to:

`seanchatmangpt/ggen-legacy@60d38265b8d1d94c43f04ca6bdb8537184e510a8:scripts/ci_errc.py`

ERRC is not a score. Every claim is attached to one exact
`(target, metric, unit)` coordinate and must satisfy exactly one directional
relation:

- ELIMINATE: `baseline > 0`, `candidate = 0`
- REDUCE: `baseline > candidate > 0`
- RAISE: `candidate > baseline > 0`
- CREATE: `baseline = 0`, `candidate > 0`

A non-empty preservation fence is mandatory. Heterogeneous units are never
summed. The receipt claim ceiling excludes causality, utility, optimality, and
actuation conclusions.

## 5. Generation

The CLI graph is ontology-first:

```text
ontology/affi-cli.ttl
    -> ggen.toml inference + SPARQL validation
    -> .ggen/templates/**
    -> src/verbs/**
    -> Rust execution courts
```

When changing a generated CLI surface, modify the ontology/config/template and
run the repository's documented ggen generation/check path. Never patch a
projection solely to make CI green.

## 6. Verification ladder

Use the cheapest high-information court first, then expand after success:

```bash
python3 -m unittest discover -s scripts/tests -p 'test_ci_errc.py'
python3 scripts/ci_errc.py --base <BASE_SHA> --head <HEAD_SHA> \
  --report evidence/ci/errc-fast.json
cargo fmt --all -- --check
cargo build --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
```

The ERRC fast court has a hard ceiling: success is `PARTIAL_ALIVE`. It verifies
exact-head identity, path-to-court routing, and changed JSON/TOML parsing; it
cannot stand in for Rust execution.

The root Rust workflow, `affidavit-core`, web, and confevo courts own their
respective execution claims. GitHub workflow metadata is not execution proof;
inspect the exact-head job steps/logs.

## 7. Toolchain and dependency replay

`rust-toolchain.toml` is intentionally date-pinned. Do not float nightly without
an explicit migration receipt. `Cargo.lock` is part of replay identity. During a
dependency graduation, preserve Cargo's resolver-produced lockfile as an
artifact even when later compilation fails; after it is committed, move the
court to `--locked`.

## 8. Failure discipline

On failure:

1. preserve the exact subject/head and command;
2. classify the failed transition;
3. locate the narrowest cause;
4. repair the lawful path rather than skipping the verifier;
5. encode a permanent regression witness/refusal;
6. rerun that boundary;
7. expand only after it succeeds.

Never use `continue-on-error`, fake fixtures, vacuous assertions, hand-written
generated outputs, or local compatibility inventions to manufacture green.

## 9. Publication

Use a purpose branch based on a recorded exact base SHA. Commit intentionally,
non-force push, and keep review work in a draft PR until the exact-head courts
support the claimed standing. Never merge or release unless explicitly asked.
