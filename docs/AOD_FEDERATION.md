# AOD → standing federation adapter

**Spec:** AFFI-SPEC-26.11.19  
**Adapter:** `affidavit/aod-federation-adapter/v1`  
**Standing profile:** `affidavit/standing/v2`  
**Standing ceiling:** `PARTIAL_ALIVE` (hard)

---

## What this certifies

`affi standing certify-aod` proves **format presence** of a Live-Gate-admissible
sealed `.aod` under the production [`CompiledOntology`] fixture, and records the
ontology-hash witness. It then seals an ordinary standing receipt at
`PARTIAL_ALIVE`.

It does **not**:

* decide honesty of row contents
* grant `ALIVE` (typed stop: `REFUSED_AOD_FEDERATION_ALIVE_CEILING`)
* durable-burn the monoatomic lease into `.affi/aod-leases` (that remains
  `affi aod verify --live`)
* invent ecosystem ALIVE quorum from format presence alone

Doctrine: **certify, don't decide.**

---

## Verifier path (real, not invented)

| Step | Law |
|------|-----|
| 1 | Admit the subject `core/v1` receipt (`admission::admit`) |
| 2 | `map_live` over `.aod` bytes with fixture ontology + `InMemoryLeaseRegistry` |
| 3 | Ontology-hash witness must match the fixture pin |
| 4 | Build `StandingObservation { standing: PARTIAL_ALIVE, execution, verification, replay: None }` |
| 5 | `certify_standing` seals `affidavit/standing/v2` |

Live Gate is the real `affidavit_core::aod::map_live` path. The in-memory lease
registry proves the burn law without side-effecting the durable AFAL store.

---

## CLI

```bash
# Seal a fixture-bound .aod (or use any Live-Gate-admissible artifact)
affi aod seal --ontology "$FIXTURE_ONTOLOGY_HEX" --lease "$LEASE" \
  --rows 2 --in page.bin --out artifact.aod

# Subject identity for the standing receipt
cat > subject.json <<'JSON'
{
  "subject": "seanchatmangpt/affidavit",
  "base": "<base sha>",
  "tree": "<tree id>",
  "candidate": "<head sha>"
}
JSON

affi standing certify-aod \
  --receipt source.json \
  --aod artifact.aod \
  --subject subject.json \
  --scope 'repo:seanchatmangpt/affidavit' \
  --out standing-aod.json \
  --format json

affi standing verify --receipt standing-aod.json
```

Court report (stdout with `--format json`) carries adapter annotations
(`adapter`, `standing_ceiling`, `aod_federation.*`). The **sealed**
`--out` artifact remains profile-pure standing JSON.

---

## Typed refusals

| Token | When |
|-------|------|
| `REFUSED_AOD_FEDERATION_IO` | `.aod` path unreadable |
| `REFUSED_AOD_FEDERATION_NOT_AOD` | missing AOD magic / not an artifact |
| `REFUSED_AOD_FEDERATION_LIVE_GATE` | `map_live` refused (ontology, stride, digest, …) |
| `REFUSED_AOD_FEDERATION_ONTOLOGY_MISMATCH` | hash witness disagree |
| `REFUSED_AOD_FEDERATION_ALIVE_CEILING` | adapter asked to mint ALIVE |
| `REFUSED_AOD_FEDERATION_EMPTY_FIELD` | empty subject identity field |

---

## Ecosystem composition

A PARTIAL_ALIVE AOD standing receipt **may** be embedded as an ecosystem member.
It does **not** satisfy an ALIVE quorum requirement — that is intentional. Format
presence of Arrow-on-Disk is not crown ALIVE for the federation graph.

Sibling lane (micro-verifier / NIF / crypto fence): [`AOD_ECOSYSTEM.md`](AOD_ECOSYSTEM.md)
when present; this adapter is the **in-repo federation/standing** seam.

---

## Receipt commands

```bash
cargo test --test federation_aod
cargo test --lib federation_aod
affi standing certify-aod --help
```

## Falsifiers

* Court grants `ALIVE` from `.aod` format alone → ceiling broken.
* Court writes `.affi/aod-leases` → durable side-effect leaked into federation.
* Court accepts bytes that `map_live` rejects → invented green.
* Sealed `--out` JSON includes non-standing `aod_federation` fields → profile purity broken.

## See also

* [`FEDERATION.md`](FEDERATION.md) — standing / ecosystem / errc courts
* [`src/federation_aod.rs`](../src/federation_aod.rs) — adapter kernel
* [`reference/verbs/aod/verify.md`](reference/verbs/aod/verify.md) — durable Live Gate
