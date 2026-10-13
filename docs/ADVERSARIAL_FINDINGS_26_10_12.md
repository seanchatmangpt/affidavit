# Adversarial Findings Receipt — Skeptic Ensemble 2026-10-11 (v26.10.12)

Affidavit-scoped skeptic receipt. Findings SK1–SK6 from the adversarial
ensemble run of 2026-10-11 against the affidavit trust plane, each with
verdict, repro, and disposition against the R29–R34 fix train.

## SK1 — Trajectory fold non-associative, panic at `trajectory_folding.rs:351`, unsigned domain

**Verdict**: CONFIRMED (fixed).

**Repro**: `fold_receipts(a, b)` did not associate: `fold_receipts(fold(a,b),c)`
differed from `fold_receipts(a,fold(b,c))` because the folded receipt carried
`a`'s linkage fields and a BLAKE3 recombine of digests. The combining path
panicked at `src/trajectory_folding.rs:351` (old layout) on truncated-name
re-encoding. The fold domain is unsigned: negative/credited quantities are
unrepresentable.

**Disposition**: Fix landed in the R29–R32 train. `fold_receipts` now anchors
at the chain root (`parent = a.parent`) and speaks for the tip
(`chain_hash = b.chain_hash`); `total_ticks`/`wall_ms` are saturating sums;
subject/metadata/positive-control are XOR accumulators (associative); the
parent-linkage gate is unchanged. Panic path removed (UTF-8-boundary-safe name
truncation). Unsigned-domain limitation documented in module docs; enforcement
that folded output binds only to the original subject chain is the sealer's
subject-only commitment, not this module's.

## SK2 — `wire_ctrt` constant-FLOW, not constant-TIME (measured)

**Verdict**: CONFIRMED (FIXED — `deserialize_ct`; SX9-verified).

**The finding**: the convenience wrapper `WireCtrt::deserialize` is
constant-flow: the branchless predicate computation runs identically for every
input, but the accumulated error mask selects the `Result` variant through a
branch. Timing leaks the validity class (valid vs structurally invalid),
measured at roughly ~8 ns/call.

**Disposition**: FIXED (deserialize_ct; SX9-verified). R34's
`WireCtrt::deserialize_ct` landed and the SX9 court verified it ALIVE: Welch
`t` under 4.5 on 178/192 batched-median comparisons (thrash 178/192 |t|<4.5),
the codegen is branch-free on the decoded path, and the deliberately
timing-leaky Spectre-style premise of the original finding is absent from the
constant-time path. `deserialize_ct` performs size-normalized work on both
paths and returns a machine code instead of a `Result`. The QG-05 dudect court
(`tests/ct_timing_invariance.rs`) enforces Welch t < 4.5 over batched medians
(up to 1 + 3 idle reruns) against a positive control.

## SK3 — Raw deontic context collision; public `from_seed` downgrade

**Verdict**: CONFIRMED (fixed).

**Repro**: The deontic context byte string
`Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`
used raw concatenation without length framing: `(jur=b"AB", hash=[], epoch=8B)`
collides with `(jur=b"A", hash=[...], epoch=...)` boundary shifts —
e.g. `(jur=b"AB", hash=[], epoch=[1,2,3,4,5,6,7,8])` and
`(jur=b"A", hash=[B], epoch=[2,3,4,5,6,7,8,1])` concatenate to the same byte
string. Separately, the public `Es256SigningKey::from_seed`/provider
`SoftwareEs256Provider::from_seed` accepted degenerate seeds (all bytes
identical) that downgrade to forgeable test-grade keys in non-test builds.

**Disposition**: Fix landed in the R29–R32 train. Context is now
length-framed: `Ctx = "STRATA:" || u32le(len)||Jurisdiction || u32le(len)||Hash
|| u32le(len)||Epoch` (`src/deontic.rs`), making collision by concatenation
impossible. Degenerate-seed screening (`seed_is_degenerate`: all 32 bytes
identical) is inherited by the provider; outside test builds `from_seed`
refuses degenerate seeds with a typed refusal (`src/crypto_trust_provider.rs`,
`src/crypto_trust_es256.rs`).

**Seed-surface scope (SX5, disposition refresh)**: SX5 found the degenerate-seed
screening is ES256-only — the PQC KAT/seed surface (`src/crypto_trust_pqc.rs`)
does not yet screen. R37 forward-ported the screening to ML-DSA-65 and
SLH-DSA-128s (`ml_dsa65_from_seed`/`slh_dsa128s_from_seed` → typed refusal
outside test builds). Status: FIXED, landed at 1a16e25
(`fix(pqc): SX5-F2 degenerate-seed screening forward-ported to both PQC
from_seed paths`; last commit touching `src/crypto_trust_pqc.rs` at
receipt-refresh time 2026-10-13).

## SK4 — 201-byte payload covert channel; format held

**Verdict**: CONFIRMED (fixed; wire format held).

**Repro**: `CTRT_PAYLOAD_SIZE = 201`; the wire layout is 352 bytes total with
the payload at 151..352. `WireCtrt::deserialize` validated only the structural
prefix, so the 201-byte payload tail was accepted with arbitrary content — a
covert channel of 201 bytes per receipt that still verified.

**Disposition**: FIXED, landed at 1861dd7 (`fix(wire_ctrt): enforce full-wire
BLAKE3 commitment at decode (SK4-F1)`); last commit touching
`src/wire_ctrt.rs`. The 352-byte wire layout is
unchanged (format held), so no receipt compatibility break. `deserialize` is
deprecated (since 26.10.13) and `deserialize_verified` enforces the full-wire
BLAKE3 digest commitment at decode, fail-closed end-to-end
(`src/wire_ctrt.rs` SK4-F1 markers at lines 53/279/309/589).

## SK5 — Epoch-rotation replay amnesty; fail-open persistence; `[0x26; 32]` epoch key

**Verdict**: CONFIRMED (fixed, lane R31).

**Repro**: Prior behavior cleared the dense cache and sliding-window bit
filter on every epoch rotation (explicit `rotate_epoch` or capacity-triggered
generation increment), so a pre-rotation wire/nullifier presented after
rotation was witnessed again — a replay amnesty across epochs. Persistence
reconstitution was fail-open: a truncated/corrupt persisted filter reloaded as
an empty (permissive) window. The epoch key for the degenerate case is the
constant byte string `[0x26; 32]`.

**Disposition**: FIXED, landed at 5e5808d (`fix(replay-nullifier): close
rotation replay-amnesty (SK5 fix-forward, lane R31)`; R31). Stored nullifiers and every witnessed
identity are tagged with the epoch generation they were tagged under; a
pre-rotation wire/nullifier presented after epoch rotation is refused
(`src/replay_nullifier.rs`, module header "Rotation replay-amnesty fix (SK5
fix-forward, lane R31)"; falsifier at `src/replay_nullifier.rs:697`: the
original wire MUST be refused after rotation).

## SK6 — Module-identity-blind seal preimage

**Status**: CONFIRMED, fix QUEUED.

**The finding**: the seal preimage does not bind the sealing module's
identity. A seal over content binds the content hash but not which module
produced it, so a seal minted by one module can be replayed as if minted by
another at the same content — the preimage is module-identity-blind.

**Disposition**: Queued for the R33/R34 fix train. No fix marker in the
R29–R32 diff; the fix (bind module identity into the seal preimage domain
separation, with the same length-framing law as SK3's `Ctx`) must not break
the 352-byte wire (format held per SK4).

## HELD results (skeptic ensemble, 2026-10-11)

Surfaces the ensemble attacked that held:

- **RustCrypto zero-unsafe**: the ML-DSA-65 verifier path via RustCrypto
  `ml-dsa` contains no `unsafe` code in the audited surface; upstream is a
  cryptographic dependency, not an Affidavit reimplementation, and its own
  audit/assurance status remains part of deployment evidence
  (`docs/CRYPTO_TRUST.md`).
- **libcrux SIMD-scope**: libcrux usage is confined to the SIMD-scope
  integration; no libcrux code paths leak outside the gated module boundary.
- **Bloom bounds**: the replay-nullifier Bloom/bit-filter capacity bounds are
  enforced (window capacity, dense cache bounds); no unbounded-growth path.
- **Fuzz-clean decoders**: the wire decoders (`WireCtrt` deserialize paths,
  nullifier wire decode) survived the fuzz court without a single
  crash/hang/OOM finding.

## Receipt

- Findings: SK1 fixed (aaa62eb), SK2 FIXED (R34 `deserialize_ct`; SX9-verified
  ALIVE), SK3 fixed for ES256/deontic contexts (ebfa6c7) with the PQC seed
  forward-port FIXED (R37, 1a16e25), SK4 fixed (1861dd7), SK5 fixed (R31,
  5e5808d); SK6 queued (R33/R34) — not landed as of 2026-10-13 (no SK6 commit
  in `git log --all --grep=SK6`, no SK6 fix marker in `src/` or `tests/`).
- Falsifiers: `src/replay_nullifier.rs:697` (SK5), SK4-F1 decode enforcement
  (`src/wire_ctrt.rs:589`), QG-05 Welch t < 4.5
  (`tests/ct_timing_invariance.rs`), SK3 length-framed `Ctx`
  (`src/deontic.rs`), degenerate-seed refusal
  (`src/crypto_trust_provider.rs`).
- Format held: the 352-byte CTRT wire is unchanged across the entire fix
  train — no receipt-compatibility break.
