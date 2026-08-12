# ERRC as an Evidence-Bound Transformation Calculus

Status: normative for `affidavit/errc/v1`.

## 1. Purpose

Affidavit does **not** treat ERRC (Eliminate, Reduce, Raise, Create) as a
brainstorming matrix or an optimization oracle. It treats ERRC as a finite,
decidable classification of declared before/after measurements whose evidence,
subject identity, preservation fences, replay recipe, and lineage are sealed in
a receipt.

The operational source is intentionally pinned to:

- repository: `seanchatmangpt/ggen-legacy`
- commit: `60d38265b8d1d94c43f04ca6bdb8537184e510a8`
- artifact: `scripts/ci_errc.py`

That implementation supplied the archaeological mechanics: exact-head identity,
deterministic path-to-evidence routing, typed fast failures, machine-readable
receipts, and replay. Affidavit reconstitutes those mechanics under its stronger
`certify, don't decide` boundary.

## 2. Preserve before transforming

Let `P = {p_1, ..., p_n}` be explicit Chesterton-fence invariants. An ERRC
receipt is inadmissible when `P` is empty. This prevents an apparent
"improvement" from silently deleting the property that made the original
system lawful.

Typical preserved invariants are:

- zero unreceipted actuation;
- exact-subject identity;
- generated surfaces remain projections rather than authority;
- evidence admission is separate from policy selection;
- deterministic replay remains possible.

ERRC certification is therefore constrained transformation, not unconstrained
substitution.

## 3. Factor space

A factor coordinate is

`f = (target, metric, unit)`.

For each factor, the admitted observation supplies exact non-negative integer
magnitudes `(b_f, c_f)`, where `b_f` is baseline and `c_f` is candidate. Rates,
latencies, probabilities, and costs must first be normalized to an explicit
integer unit such as nanoseconds, bytes, basis points, microdollars, or count.
No floating-point comparison occurs inside the certifier.

Within one receipt, a factor coordinate is unique. This forbids contradictory
claims such as declaring the same `(target, metric, unit)` both REDUCE and
ELIMINATE.

## 4. Four disjoint relations

For a declared factor `f`, the lawful classes are:

- **ELIMINATE**: `b_f > 0 ∧ c_f = 0`
- **REDUCE**: `b_f > c_f > 0`
- **RAISE**: `c_f > b_f > 0`
- **CREATE**: `b_f = 0 ∧ c_f > 0`

These relations are pairwise disjoint. The states `b_f = c_f` and
`b_f = c_f = 0` are deliberately outside ERRC because no transformation has
occurred. A caller cannot rename CREATE as RAISE or ELIMINATE as REDUCE to obtain
a preferred narrative; the boundary refuses it.

ERRC contains no global scalar objective. Affidavit records cardinalities per
quadrant but **never sums heterogeneous magnitudes**. Ten milliseconds and two
authority paths do not become twelve "improvement points."

## 5. Observation, construction, receipt

Let:

- `S` = exact subject identity `(repository, base, tree, candidate)`;
- `O*` = BLAKE3 commitment to the admitted observation set;
- `A_r` = hash of the already-admitted Affidavit source receipt;
- `C` = canonical set of ERRC claims;
- `P` = canonical preservation fence;
- `Q` = descriptive quadrant counts;
- `Y` = replay command + environment commitment;
- `R_prev` = optional predecessor receipt.

The certifier manufactures:

`R_errc = BLAKE3(profile || source || ceiling || S || O* || A_r || C || P || Q || Y || R_prev)`.

Serialization is canonicalized by claim/invariant identifier before hashing.
Deserialization re-runs fixed-field identity, factor uniqueness, directional
laws, preservation requirements, replay structure, counts, and hash verification.
A JSON object named "receipt" is not a receipt unless those checks pass.

## 6. Claim ceiling

`affidavit/errc/v1` has the fixed ceiling:

`DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM`

Therefore an ERRC receipt can certify:

- what subject was compared;
- what measurements were declared;
- whether each before/after relation matches its ERRC class;
- what invariants were explicitly preserved;
- what evidence commitments and replay recipe were bound.

It cannot certify that the transformation **caused** an outcome, maximizes
utility, dominates another architecture, is globally optimal, or should be
actuated. Those are different proof obligations.

## 7. SELECT / CONSTRUCT / DO

`certify_errc` is a **CONSTRUCT** boundary only. It has no ambient execution
authority and performs no side effects. Selection remains outside the receipt,
and DO remains behind the system's actuation authority/BRCE boundary.

This matters because a planner, model, ERRC matrix, ontology, generated source,
or proof artifact must never acquire machine-state authority merely by being
well formed.

## 8. Composition and lineage

Receipts form a DAG through `previous_receipt`. Version 1 records lineage but
does not infer transitive optimization claims.

A future composition operator may only compose adjacent ERRC receipts if it can
prove, at minimum:

1. predecessor/successor receipt identity matches;
2. subject transition identity aligns;
3. factor coordinates and units are compatible;
4. required preservation invariants survive the morphism;
5. observation and evidence identities remain replayable.

Until such an operator exists, adjacency is **not** equivalence, causality,
transitivity, or proof of dominance.

## 9. CI ERRC court

`scripts/ci_errc.py` reconstitutes the ggen-legacy fast-court pattern with one
intentional strengthening: a fast routing court can never crown the repository
`ALIVE`.

It can produce only:

- `PARTIAL_ALIVE` — exact head observed, changed files classified, changed
  JSON/TOML admitted;
- `BUILD_BROKEN` — an exact-head, discovery, or structured-admission check failed.

Heavy evidence lanes own actual execution:

- `root_rust`
- `generation`
- `affidavit_core`
- `web`
- `confevo`
- `governance`

A path can route to multiple lanes. One failed edge is topology, not proof that
the entire graph is invalid.

## 10. Falsifiers

The following must fail certification:

- ELIMINATE whose candidate is non-zero;
- REDUCE whose candidate reaches zero or does not strictly decrease;
- RAISE whose baseline was absent or whose candidate does not strictly increase;
- CREATE whose baseline was already present;
- duplicate claim ids;
- duplicate factor coordinates;
- empty preservation fence;
- malformed commitments;
- changed profile/source/claim ceiling;
- non-canonical serialized order;
- tampered quadrant counts or receipt hash.

The following must fail the fast CI court:

- checkout HEAD differs from the admitted candidate SHA;
- base/head changed-file discovery fails;
- a changed JSON or TOML file cannot be parsed.

## 11. Replay

Fast court:

```bash
python3 -m unittest discover -s scripts/tests -p 'test_ci_errc.py'
python3 scripts/ci_errc.py \
  --base <exact-base-sha> \
  --head <exact-head-sha> \
  --report evidence/ci/errc-fast.json
```

Formal Rust court:

```bash
cargo test errc --lib
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

Only observed execution against the exact admitted subject can raise execution
standing. Inspection, routing, workflow existence, or a named receipt cannot.
