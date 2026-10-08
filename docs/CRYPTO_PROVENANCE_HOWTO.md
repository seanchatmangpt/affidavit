# CRYPTO PROVENANCE HOWTO

How to certify cryptographic provenance with `affi` on the surfaces that are
**committed** as of `23bd25a` (branch `feat/advanced-witness-capability-set`).
Every claim below cites `file:line` in committed source; anything in-flight
(uncommitted working-tree changes) is explicitly listed in
[Not documented](#not-documented-in-flight).

## The certify-don't-decide boundary

Affidavit produces cryptographic standing, never authorization. The boundary
is documented in [CRYPTO_TRUST.md](CRYPTO_TRUST.md):
`Affidavit proof -> SA2A authority decision -> BRCE/actuator consequence -> Affidavit receipt/evidence`.

The executable version of that boundary is the certify pipeline: it certifies
that a receipt is internally consistent and chain-intact — it never decides
whether the underlying work was authorized. Authorization is a downstream
consumer decision (see the consumer-contract section of
[CRYPTO_TRUST.md](CRYPTO_TRUST.md)).

## The 7-stage certify pipeline

`affidavit::verifier::verify` (`src/verifier.rs:59`) is pure: the same receipt
always yields the same `Verdict`, `accepted` only when every prior stage
passed (doc comment at `src/verifier.rs:35-42`). Verified stage count: **7**.

| # | Stage | Committed source |
|---|---|---|
| 1 | `decode` — receipt structurally present, version parseable | `src/verifier.rs:93` |
| 2 | `check_format` — field shape, timestamps, digests well-formed | `src/verifier.rs:108` |
| 3 | `chain_integrity` — re-derives the rolling BLAKE3 chain hash from event bytes and compares against the stored hash | `src/verifier.rs:126` |
| 4 | `continuity` — event sequence numbers are contiguous, no gaps or duplicates | `src/verifier.rs:153` |
| 5 | `verify_commitments` — each commitment re-derived from event bytes | `src/verifier.rs:186` |
| 6 | `evaluate_profile` — certification against the fixed format standard / profile rules | `src/verifier.rs:208` |
| 7 | `emit_verdict` — accepted iff all prior stages passed | `src/verifier.rs:76` |

Stages 1-6 are invoked in pipeline order at `src/verifier.rs:63-75`.
A runnable walkthrough exists: `cargo run --example verify_stages`
(`examples/verify_stages.rs`, referenced from `src/verifier.rs:42`).

## The rolling BLAKE3 chain

The chain rule (committed doc comment, `src/chain.rs:1-11`):

```text
chain_hash_0 = blake3(GENESIS)
chain_hash_n = blake3(chain_hash_{n-1}.as_bytes() || canonical_bytes(event_n))
```

- Genesis is bound to the crate version:
  `GENESIS_SEED_STR = "affidavit-v{CARGO_PKG_VERSION}-genesis"`
  (`src/chain.rs:25-26`).
- Chain operations: `genesis_hash` (`src/chain.rs:52`), the fold rule
  `blake3(prev.as_hex().as_bytes() || canonical_bytes(event))`
  (`src/chain.rs:57`), pure re-derivation via `recompute_chain`
  (`src/chain.rs:72`), and the append-only `ChainAssembler`
  (`src/chain.rs:80`).
- Stage 3 of the certify pipeline uses `recompute_chain` to re-derive the
  chain hash from event bytes alone (see `src/chain.rs:68-69`).

## Witness verification (advanced-witness modules)

Both modules are feature-gated in the committed module tree
(`src/lib.rs:245-246` and `src/lib.rs:255-256`):

- **Ed25519** (`feature = "ed25519"`): `KeyPair::generate`
  (`src/ed25519_witness.rs:41`), `public` (`:60`), `sign` (`:66`), and
  `verify_witness(public_key, message, signature)`
  (`src/ed25519_witness.rs:81`) — parses the verifying key, then verifies the
  64-byte signature, with typed refusals (e.g. `MalformedPublicKey`).
- **secp256k1** (`feature = "secp256k1"`): `verify_bip340_raw`
  (`src/secp256k1_witness.rs:46`), `verify_bip340` (`:68`), and
  `verify_ecdsa` (`:92`) — BIP-340 and ECDSA verification over the same key
  material.

Chicago-style tests for both round-trip and refusal paths live in
`tests/advanced_crypto_chicago.rs` (see the HANDWRITTEN.md advanced-witness
row for the full 17-module capability list).

## CLI surface (witness / standing / evidence)

`src/registry.rs` declares the verb table (92 verbs / 9 nouns per
`HANDWRITTEN.md`). Rows cited here:

- `verify receipt` — "Run the 7-stage certify pipeline against a receipt
  (exit 0=ACCEPT, 2=REJECT)" (`src/registry.rs:151-157`), e.g.
  `affi verify receipt --receipt receipt.json`.
- `standing certify` (`src/registry.rs:650-657`):
  `affi standing certify --receipt r.json --observation o.json --scope repo:acme/app --out standing.json`
- `standing verify` (`src/registry.rs:660-666`):
  `affi standing verify --receipt standing.json`
- `ecosystem verify` (`src/registry.rs:678-684`)
- `errc certify` / `errc verify` (`src/registry.rs:688-700`)
- Trust-plane evidence cluster (`src/registry.rs:728-830`):
  `keys generate/list/import/revoke/rotate`, `envelope sign/verify/list/export`,
  `evidence journal` (`affi evidence journal release-v26.9.28 --out entry.json`),
  `evidence crl-publish`, `evidence crl-apply`, and
  `evidence heads` (`affi evidence heads --journal-file .affi/standing-journal.jsonl`).

## Certified-receipt seam

`src/receipts_certified.rs` (committed at `b4e7181`):
`certify_paid_delivery_payload` (`src/receipts_certified.rs:76`),
`build_canonical_subject` (`src/receipts_certified.rs:172`), and
`verify_certified_paid_delivery` (`src/receipts_certified.rs:182`).

## Not documented (in-flight)

The following files are dirty in the working tree at `23bd25a` and are
**NOT documented** by this how-to. Their in-flight edits are excluded under
the preservation fence; document them in a later pass once landed:

- `src/crypto_trust_crl_file.rs`
- `src/crypto_trust_envelope.rs`
- `src/crypto_trust_es256.rs`
- `src/crypto_trust_journal.rs`
- `src/crypto_trust_journal_persist.rs`
- `src/crypto_trust_kat.rs`
- `src/crypto_trust_log.rs`
- `src/crypto_trust_nonce_store.rs`
- `src/crypto_trust_quorum.rs`
- `src/crypto_trust_rotation_store.rs`
- `src/crypto_trust_store.rs`
- `src/crypto_trust_verify.rs`
- `src/crypto_trust_witness.rs`
- `tests/crypto_trust_e2e.rs`
- `benches/crypto_trust_bench.rs`

(Note: `src/crypto_trust_verify.rs` shows 332 deleted lines in the working
tree — a large in-flight rework; only its last-committed state at `8d07430`
exists, and it is intentionally not cited above.)
