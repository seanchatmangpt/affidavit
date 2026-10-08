# SJ-ALIGNED-RECORD-DESIGN — affidavit — v26.10.8

**Status**: design (no code in this doc). Companion to
[CAMPAIGN-RECEIPT.md](CAMPAIGN-RECEIPT.md).
**Scope**: how an affidavit campaign record becomes an `R` (Receipt) under
`schemas/dfcm-receipt.schema.json` (`$id`
`https://chatmangpt.com/schema/receipt/v2`), under the repo's BLAKE3 chain law
as documented in [`docs/CRYPTO_TRUST_PLANE.md`](../../CRYPTO_TRUST_PLANE.md).

---

## 0. Problem

Today the campaign record is a markdown table
([CAMPAIGN-RECEIPT.md](CAMPAIGN-RECEIPT.md)): commits, courts, residues —
human-readable, but it is **O, not R**. It carries no replayable command list,
no digest-bound subject identity, no attestation envelope, and no chain law: a
markdown file can be edited in place with no structural refusal. The design
below promotes the campaign record to a schema-valid `R` with the five
required faces (identity, authority, consequence, replay, standing) plus the
chain, seal, and canonicalization laws the trust plane already enforces.

## 1. The chain law: domain-separated BLAKE3 re-verification (tamper = non-existence)

The base receipt law is inherited from `src/chain.rs` /
`src/verifier.rs` stage 3 (`chain_integrity`): the rolling BLAKE3 chain hash is
recomputed over the event bytes; any edit propagates through every later link
and mismatches the stored `chain_hash`.

The trust plane tightens this into an identity law, per
[CRYPTO_TRUST_PLANE.md](../../CRYPTO_TRUST_PLANE.md) module map row
`crypto_trust_verify`: "receipt-hash tamper refusal on deserialize". The
canonical statement, from the Consumption section:

> The base receipt's own law is untouched: `Receipt` deserialization re-verifies
> the BLAKE3 chain, so a tampered base cannot even become a `Receipt` value.

**Design requirement (a)**: a campaign record that aspires to be an R is
*carried* by a chain-sealed `affidavit::types::Receipt`. The campaign's
commits, courts, and residues serialize as chain events (`emit`); assembly via
`ChainAssembler::finalize` seals the chain. There is no deserialization path
that skips chain re-verification — **a tampered base cannot deserialize**;
tamper-evidence is not a check the consumer must remember to run, it is a
precondition of the value existing at all.

- Chain events: one event per campaign commit (payload: sha, one-line, court
  results), one event per court witness, one event for the residue declaration.
- `chain_hash` folds commitments `blake3(payload)` — order-sensitive, so the
  commit sequence itself is tamper-bound.
- Not stored as markdown: the markdown companion stays human-facing
  (projection), the R is the machine surface.

## 2. JCS canonical serialization + 32-byte subject_digest binding

Per the trust plane, all digest pre-images are **JCS (RFC 8785) canonical
JSON** — UTF-16 key ordering, minimal escaping, ECMAScript number rendering,
typed refusal of integers beyond 2^53 (`NonCanonicalNumber`) — digested with
BLAKE3 under the domain tag:

```text
domain_separated("affidavit.crypto-trust-plane.v1", [jcs(envelope_document)])
```

"Both sides of the interlock must reproduce the exact same bytes; there is no
'approximately canonical'."

**Design requirement (b)**: the R's subject identity is digest-bound, not
name-bound:

1. Serialize the campaign record document (commits/courts/residues as chain
   events) with **JCS**. No "approximately canonical" serialization is
   admissible; integers > 2^53 refuse typed.
2. Compute the content address: `content_address(receipt)` (the sealed base
   receipt's BLAKE3 content address, `src/chain.rs`).
3. The R's `identity.subject_digest` is

   ```text
   subject_digest = BLAKE3(domain_separated(DOMAIN_TAG, [content_address_hex]))
   ```

   exactly the law implemented in `subject_digest_of` in
   `src/crypto_trust_seal.rs` (`subject_digest == BLAKE3(content address)`),
   which satisfies the schema's `identity.subject_digest` shape:
   `{"algorithm": "blake3", "value": <64 hex>}`. Per the schema description,
   non-commit digests never ride in `subject_sha`; the git commit anchor of
   the campaign HEAD stays in `subject_sha` (40-hex, must resolve in
   `identity.repo`), the digest rides in `subject_digest`.

Because the pre-image is JCS-canonical, a producer and a verifier on opposite
sides of the trust plane re-derive identical bytes, so the 32-byte digest is a
verifiable identity, not a label. Because the digest is over the *content
address of the chain-sealed base*, and the base re-verifies its chain on
deserialize, the digest binds the whole campaign record — commits, courts,
residues, order included.

## 3. Optional SealedReceipt envelope: who attests (key / epoch / policy)

`crypto_trust_seal` provides `SealedReceipt` (`PQ-SEAL-v1`): base receipt +
`SignatureEnvelope` (12 signed fields, `CTP-ENVELOPE-v1`) + signature, "bound
to an affidavit `Receipt` via its content address", with the linkage law that
the envelope's `subject_digest` must equal `subject_digest_of(receipt)` or
`SealError::SubjectMismatch` refuses.

The 12 signed fields are exactly the who-attests answer:

| attestation face | signed field(s) |
|---|---|
| which key | `algorithm`, `key_id`, `generation`, `profile` |
| which epoch | `policy_epoch`, `revocation_epoch` |
| which policy | `profile` (assurance floor), `not_before`/`expires_at` (validity window), `audience` |
| what is attested | `subject_digest` (32 bytes, domain-separated BLAKE3) |
| replay evidence | `nonce` (16 bytes), journaled as `(kid, nonce)` in the `NonceJournal` (300 s window, `REPLAY_REJECTED` on repeat) |

**Design requirement (c)**: the optional seal answers *who attests* — under
which key, epoch, and policy — while the base chain law answers *what
happened*. Design constraints:

1. **Optional, additive**: an R is schema-valid without a seal; a seal adds a
   second, independent binding. No authority inversion: affidavit certifies,
   it never decides what the standing authorizes (`certify-don't-decide`).
   `verify_sealed` returns
   a `CryptographicVerdict`/`CryptographicStanding` (closed vocabulary,
   order 0..7), and **only `VALID` mints a `CryptoStandingReceipt`**; every
   non-VALID outcome refuses certification.
2. **Re-derive, never trust the claim**: `verify_sealed` re-derives the digest
   from the base receipt (`subject_digest_of`), never from the envelope's
   claim; a mismatch is `SubjectMismatch` (pinned in
   `tests/ag4_adversarial_crypto.rs` as the certified-receipt envelope-swap
   court).
3. **Seal after finalize**: seal only an assembled (chain-sealed) receipt —
   sealing is defined only over a finalized chain, so the seal inherits the
   tamper law rather than replacing it.

## 4. Replay binding: the face markdown lacks

Markdown campaign records state; they do not replay. The schema's `replay`
face requires `commands[]` with `cmd` / `exit` / `cwd` (+ optional
`output_sha256`), and `replay_binding` adds the causal DAG edges:
`event_ids` (OCEL event ids this receipt claims), `chain_head_hash` (the
BLAKE3 chain head this receipt binds to), and
`predecessor_work_order_ids` (prior receipts extended).

**Design requirement (d)**: the campaign R carries:

- `replay.commands`: the court commands actually executed (pin court, gate
  runs), each with real `exit` and `cwd`; per the schema, `durable_location`
  is a git-tracked path — this doc's directory (`docs/sjira/v26.10.8/`) is
  the durable location, never gitignored tmp.
- `replay_binding.event_ids`: the chain event ids minted in §1 — "every
  actuation event must be claimed by >=1 receipt or it is an unreceipted
  actuation".
- `replay_binding.chain_head_hash`: the assembled `chain_hash`.
- `replay_binding.predecessor_work_order_ids`: the previous campaign receipt's
  work order id (v26.10.7's, once minted retroactively or skipped for the
  first receipt).
- `standing.derived_from`: which replay command(s) at which `subject_sha`
  justify the standing value; `broken_term` required for
  BLOCKED/BUILD_BROKEN/REFUSED per the schema's `allOf` conditional.

These four faces — subject identity, replayable commands, chain-head binding,
predecessor edges — are precisely what a markdown record cannot carry, and the
reason the campaign record is promoted from O to R.

## 5. Implementation seam

No code in this design doc. The carrier is the existing certified-receipts
seam:

- **`src/receipts_certified.rs` — `CertifiedReceiptEnvelope`** (feature
  `certified-receipts`) is the carrier: it presents the signed
  `SignatureEnvelope` plus a carried `receipt` that already carries the
  canonical subject string
  `"affidavit-paid-delivery/v1|<subject>|<payload_hash_hex"`. For a campaign
  record, the canonical subject extends to the campaign domain — e.g.
  `"affidavit-campaign/v1|<work_order_id>|<chain_head_hash_hex>"` — with the
  chain-head hash inside the subject binding (payload hash rides inside the
  subject digest, per the module's subject binding law:
  `BLAKE3(domain_separated(DOMAIN_TAG, [canonical_subject]))`), so a tampered
  chain head breaks both the subject-binding law and the signature.
- `certify_paid_delivery_payload` / `verify_certified_paid_delivery` are the
  certify/verify pair to mirror; verification re-derives the canonical subject
  from *claimed* values and refuses on digest mismatch
  ("payload hash / subject does not match the certified binding").
- `src/crypto_trust_seal.rs` (`seal_receipt` / `verify_sealed`,
  `subject_digest_of`, `SealError::SubjectMismatch`) is the base-receipt
  binding layer.
- `src/crypto_trust_canonical.rs` is the JCS + domain-separated BLAKE3
  substrate (RFC 8785 vectors; `NonCanonicalNumber` 2^53 refusal).
- Schema conformance target:
  `schemas/dfcm-receipt.schema.json` — required top-level faces `identity`,
  `authority`, `consequence`, `replay`, `standing`, `work_order_id`,
  `origin_authority`, `provider`, `provider_execution_id`;
  `identity.subject_digest = {algorithm: blake3, value: hex}` for the
  non-commit digest face.

## 6. Falsifiers

The design is falsified if:

1. A campaign R deserializes with a modified event/commit payload (chain law
   broken — today: impossible, `chain_integrity` stage 3 + deserialize-time
   re-verification).
2. Two producers with the same campaign record produce different
   `identity.subject_digest` values (JCS law broken).
3. `SealedReceipt.verify_sealed` admits an envelope whose `subject_digest`
   differs from `subject_digest_of(base)` (linkage law broken — pinned as a
   refusal court in `tests/ag4_adversarial_crypto.rs`).
4. A campaign R passes schema validation missing any of `replay.commands`
   (minItems 1), `replay_binding.chain_head_hash`, or `standing.derived_from`
   (replay face missing — i.e. the record is still markdown-shaped).

## 7. Standing

DESIGN only — no code landed in this doc. Standing of the named surfaces
inherits from the trust plane's honest standing table
([CRYPTO_TRUST_PLANE.md](../../CRYPTO_TRUST_PLANE.md)): the seal module,
canonicalization module, and certified-receipts seam exist and carry their own
courts; the e2e court `tests/crypto_trust_e2e.rs` is recorded BUILD_BROKEN at
v26.9.28 wave 3 (mechanical `crate::` import fixes) — recorded honestly, not
repaired in this documentation lane.

## See Also

- [CAMPAIGN-RECEIPT.md](CAMPAIGN-RECEIPT.md)
- [CRYPTO_TRUST_PLANE.md](../../CRYPTO_TRUST_PLANE.md)
- `schemas/dfcm-receipt.schema.json`
- `src/receipts_certified.rs`, `src/crypto_trust_seal.rs`,
  `src/crypto_trust_canonical.rs`
