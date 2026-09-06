# Federation courts — `affi standing` · `affi ecosystem` · `affi errc`

**Version:** 26.9.6
**Status:** normative for the `affidavit/standing/v2`, `affidavit/ecosystem/v1`,
`affidavit/errc/v1`, and `affidavit/errc-claim-assurance/v1` profiles.

---

## What these commands are

The evidence federation kernel (`src/standing.rs`, `src/ecosystem.rs`,
`src/errc.rs`, `src/errc_claim_assurance.rs`) is a set of **certifiers**: each
takes an already-admitted Affidavit receipt plus a bounded observation, and
seals those facts into a deterministic, content-addressed profile receipt.

v26.9.6 makes those certifiers reachable from the `affi` binary. The CLI layer
(`src/federation.rs`) is an adapter and nothing else — it makes **no**
accept/refuse decision of its own. Every verdict below belongs either to
`admission::admit` or to the profile's own law. This is the same doctrine as
`affi receipt verify`: **certify, don't decide.**

### What a federation receipt does *not* claim

Each profile carries a fixed, machine-readable claim ceiling, sealed into the
receipt. None of these certifiers claims that the evidence is *true*, that the
transformation *caused* an improvement, that the work is *optimal*, or that
anyone is authorised to *act*. Read the `claim_ceiling` field before quoting a
receipt as support for a decision.

---

## The two court shapes

| Shape | Input | Law that runs |
|-------|-------|---------------|
| `certify` | a `core/v1` receipt + an observation JSON | `admission::admit`, then the profile's `certify_*` |
| `verify` | a sealed profile receipt JSON | the profile's hand-written `Deserialize`, which ends in `receipt.verify()` |

`verify` is a real court, not a parse. Each sealed receipt type has a manual
`Deserialize` impl whose last statement is
`receipt.verify().map_err(D::Error::custom)?`, so a tampered receipt cannot be
deserialized into existence at all. **Loading is the court.**

---

## Exit-code contract

Stable codes from `src/diag.rs`:

| Code | Name | Meaning |
|------|------|---------|
| `0` | `OK` | the law held; a sealed receipt was produced or re-verified |
| `2` | `REJECT` | a **named refusal** — admission refused the source receipt, the kernel refused the observation, or a sealed receipt failed its own law |
| `3` | `USAGE_ERROR` | an *observation* was not well-formed JSON for its profile |
| `4` | `IO_ERROR` | a named path could not be read |

The `2` vs `3` split is deliberate. An observation is an **operator input**, not
a witness — it never claimed to be evidence, so refusing a malformed one is a
usage error and not a verdict about anything.

Refusals are always named. `alive_missing_replay`,
`standing_receipt_hash_mismatch`, `errc_duplicate_factor_coordinate` — never a
bare "invalid input".

---

## Output: `--out` and `--format json`

`certify` and `assure` accept `--out <PATH>`, which writes the sealed receipt
as a clean artifact so `certify` → `verify` composes in a script. A refused
certification writes no artifact.

`--format json` prints the full court report — `court`, `profile`, `accepted`,
`reason`, and the sealed `receipt` — as a **single parseable JSON document on
every path**, ACCEPT and REJECT alike, so `affi errc certify --format json | jq`
works. (The `clap-noun-verb` runtime otherwise renders each verb's return value
to stdout after the handler returns, appending a bare `null`; the federation
courts exit with their own code before that happens. Other `affi` verbs are
still affected — see ROADMAP B13.)

Human-format output splits the streams the usual way: the verdict line goes to
stderr, the sealed receipt to stdout.

---

## Worked example

Every command below is exercised by `tests/federation_cli.rs`.

### 1. Assemble the subject receipt

The federation courts certify *over* an admitted `core/v1` receipt, so start
with the ordinary lifecycle:

```bash
printf 'exact-head execution' > payload.txt

affi receipt emit --event-type build --object 'repo:git' --payload payload.txt
affi receipt assemble --out source.json
affi receipt verify source.json          # exit 0 — ACCEPT
```

### 2. Certify standing

`standing.json` records *what was observed*, not what you would like to be true.
ALIVE is the strongest claim and has the strictest evidence law: it is
unconstructable unless successful execution, independent verification, **and** a
replay recipe are all present.

```json
{
  "subject": {
    "subject": "seanchatmangpt/affidavit",
    "base": "<base commit sha>",
    "tree": "<source tree id>",
    "candidate": "<exact head sha that was executed>"
  },
  "observation_commitment": "<blake3 hex of the admitted observation set>",
  "standing": "ALIVE",
  "execution":    { "command": "cargo test --all-targets",              "exit_code": 0, "result_commitment": "<blake3 hex>" },
  "verification": { "command": "cargo clippy --all-targets -- -D warnings", "exit_code": 0, "report_commitment": "<blake3 hex>" },
  "replay":       { "command": "just validate",                          "environment_commitment": "<blake3 hex>" },
  "previous_receipt": null
}
```

```bash
affi standing certify \
  --receipt source.json \
  --observation observation.json \
  --scope 'repo:seanchatmangpt/affidavit' \
  --out standing.json

affi standing verify --receipt standing.json     # exit 0 — ACCEPT
```

`--scope` is mandatory and must be non-empty. The CLI does **not** inherit the
subject's authority: it declares its own bounded capability
(`affidavit.certify-standing`, digest-pinned, witness-tagged) which
wasm4pm-compat validates structurally before anything is sealed. An empty scope
violates `RequiresBoundedScope` and is refused with exit `2`.

Drop `replay` from the observation and the ALIVE claim is refused by name:

```
standing/certify: REJECT [affidavit/standing/v2] — standing refused: alive_missing_replay
```

### 3. Federate an ecosystem

An ecosystem receipt composes exact, already-sealed member standing receipts
against a declared per-role ALIVE quorum. Members are **embedded**, so parsing
the observation re-verifies each one — a federation cannot be assembled from
unverifiable members.

```json
{
  "subject": { "subject": "chatman/ecosystem", "base": "...", "tree": "...", "candidate": "..." },
  "observation_commitment": "<blake3 hex>",
  "requirements": [{ "role": "RUNTIME_EXECUTION", "minimum_alive": 1 }],
  "members": [{ "role": "RUNTIME_EXECUTION", "standing_receipt": { "...": "contents of standing.json" } }],
  "previous_receipt": null
}
```

```bash
affi ecosystem certify --receipt source.json --observation federation.json --out ecosystem.json
affi ecosystem verify --receipt ecosystem.json
```

Roles are: `SEMANTIC_AUTHORITY`, `MANUFACTURE`, `PLANNING_CONTROL`,
`WORLD_EXECUTION`, `FORMAL_PROOF`, `RUNTIME_EXECUTION`, `PROCESS_EVIDENCE`,
`CONFIGURATION`, `SUPPLY_CHAIN`, `VERIFICATION`, `REPLAY`.

Certification **succeeds** when a quorum is unmet — the receipt simply does not
derive ALIVE. That is the point: the receipt records the topology honestly
rather than refusing to describe a federation that is not yet crowned. Read
`standing` and `coverage[].satisfied` in the sealed output.

### 4. Certify an ERRC transformation

ERRC is not a score. Every claim attaches to exactly one
`(target, metric, unit)` coordinate and must satisfy exactly one directional
relation:

| Quadrant | Law |
|----------|-----|
| `ELIMINATE` | `baseline > 0`, `candidate = 0` |
| `REDUCE` | `baseline > candidate > 0` |
| `RAISE` | `candidate > baseline > 0` |
| `CREATE` | `baseline = 0`, `candidate > 0` |

A non-empty preservation fence is mandatory, heterogeneous units are never
summed, and two claims may not share one factor coordinate.

```json
{
  "subject": { "subject": "seanchatmangpt/affidavit", "base": "...", "tree": "...", "candidate": "..." },
  "observation_commitment": "<blake3 hex>",
  "claims": [{
    "id": "eliminate-unreachable-kernel",
    "target": "affi CLI surface over the federation kernel",
    "quadrant": "ELIMINATE",
    "measure": { "metric": "kernel_certifiers_without_cli_surface", "unit": "certifiers", "baseline": 4, "candidate": 0 },
    "evidence_commitment": "<blake3 hex>"
  }],
  "preserved_invariants": [{
    "id": "certify-not-decide",
    "statement": "The CLI reaches the kernel's laws; it never adds one.",
    "evidence_commitment": "<blake3 hex>"
  }],
  "replay": { "command": "cargo test --test federation_cli", "environment_commitment": "<blake3 hex>" },
  "previous_receipt": null
}
```

```bash
affi errc certify --receipt source.json --observation errc-observation.json --out errc.json
affi errc verify --receipt errc.json
```

The sealed receipt carries the fixed archaeological lineage
(`seanchatmangpt/ggen-legacy@60d38265…:scripts/ci_errc.py`) and descriptive
`quadrant_counts` — cardinalities only, never a cross-unit total.

### 5. Assure every claim

The assurance ledger is a **bijection**: exactly one evidence witness per claim
in the parent, no more and no fewer. A witness naming an unknown claim is
refused, and so is a ledger that misses one.

```json
[{
  "claim_id": "eliminate-unreachable-kernel",
  "verifier": "cargo test --test federation_cli",
  "evidence_locator": "tests/federation_cli.rs",
  "observed_result": "8 federation verbs reachable from the affi binary",
  "evidence_commitment": "<blake3 hex>",
  "exclusions": ["Does not establish operational utility."]
}]
```

```bash
affi errc assure --parent errc.json --witnesses witnesses.json --out assurance.json
affi errc verify-assurance --receipt assurance.json --parent errc.json
```

`--parent` on `verify-assurance` is optional. Without it, the court proves
canonical structure and content identity. With it, it additionally proves
parent-hash identity and exact claim-set correspondence.

`exclusions` is not decoration. Each witness states in its own words what its
evidence does **not** establish, and that text is sealed into the receipt
alongside the claim.

---

## Version boundary

The chain genesis seed is
`concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`, so a `core/v1`
receipt only verifies under the exact binary version that assembled it. A
receipt assembled by 26.6.22 fails stage 3 (`chain_integrity`) under 26.9.6, and
therefore cannot be admitted as a federation subject. This is intended
release-boundary behaviour: re-emit and re-assemble against the current binary.

`affi --version` reports the affidavit version — check it first when a receipt
stops verifying.

---

## See also

- [`docs/ERRC.md`](ERRC.md) — the normative ERRC v1 profile
- [`docs/ERRC_CLAIM_ASSURANCE.md`](ERRC_CLAIM_ASSURANCE.md) — the claim-assurance refinement
- [`AGENTS.md`](../AGENTS.md) §4 — ERRC reconstruction law
- `src/federation.rs` — the adapter and its exit-code contract
- `tests/federation_cli.rs` — every command on this page, executed
