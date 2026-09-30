# SLH-DSA-PIN — resolving the `slh-dsa = "=0.2.0-rc.5"` pre-release pin

- Ticket: SLH-DSA-PIN (wave 1, lane 3, finish v26.9.28)
- Consumer: `/Users/sac/affidavit` (branch `feat/v26.9.28-crypto-trust-plane`)
- Pack: `/Users/sac/ggen-marketplace/packs/affidavit-trust-plane-pack/`
- Date: 2026-09-29
- Standing: **ALIVE** (decision made on crates.io API evidence + observed compile/test runs this session)
- Owned outputs: this ticket + `/tmp/w5-w1-lane3/ctp-cargo-pin-patch.md` (patch proposal, NOT applied)

## Decision

**Keep the exact pin `slh-dsa = "=0.2.0-rc.5"`. No change.**

The rc pin is the honest choice, not a compromise:

1. **No stable alternative exists.** crates.io lists 10 slh-dsa versions; the newest is
   `0.2.0-rc.5` (2026-04-28). `max_stable_version = 0.1.0`. There is no 0.2.x stable and no
   0.1.1+.
2. **The "stable" 0.1.0 is the incompatible one.** Its dependency stack is itself pre-release:
   `signature ^2.3.0-pre.4`, `hybrid-array ^0.2.0-rc.8`, `rand_core ^0.6.4`, `digest ^0.10`,
   `sha2 ^0.10.8` (crates.io deps API, 2026-09-29). Pinning it would trade a visible rc pin for a
   hidden pre-release dependency stack AND force a rewrite of the pqc module (0.1.0 has a
   different API from the 0.2 rc line the templates are written against).
3. **rc.5 sits on the stack the plane standardizes on.** Its normal deps: `signature ^3.0.0-rc.10`
   (satisfied by the now-stable `signature 3.0.0`, released 2026-05-02 — confirmed in the resolved
   lock), `hybrid-array ^0.4`, `rand_core ^0.10`, `digest ^0.11`, `sha2/sha3 ^0.11`, `const-oid
   ^0.10`, `pkcs8 ^0.11`. This is the same stack as `ml-dsa 0.1.1` (`signature ^3`,
   `hybrid-array ^0.4`) — the PQ half of the plane is coherent.
4. **The API the templates use is FIPS 205 final semantics**: `Sha2_128s`,
   `slh_keygen_internal(sk_seed, sk_prf, pk_seed)`, deterministic `try_sign`, encoded lengths
   pk = 2n = 32 B, sig = 7856 B — all witnessed by the template's tests (length + determinism +
   tamper falsifiers), all green on rc.5.

## Evidence: crates.io version tables (API queried 2026-09-29, User-Agent `affidavit-dfcm-lane3/0.1`)

### slh-dsa — `GET https://crates.io/api/v1/crates/slh-dsa/versions` (all 10 versions)

| version | created | yanked | note |
|---|---|---|---|
| 0.2.0-rc.5 | 2026-04-28 | no | **current pin; newest version of the crate** |
| 0.2.0-rc.4 | 2026-02-02 | no | |
| 0.2.0-rc.3 | 2026-01-27 | no | |
| 0.2.0-rc.2 | 2026-01-05 | no | |
| 0.2.0-rc.1 | 2025-11-07 | no | |
| 0.2.0-rc.0 | 2025-09-13 | no | |
| 0.0.3 | 2025-04-10 | no | |
| 0.1.0 | 2024-08-18 | no | max STABLE; built on signature 2.3.0-pre.4 / hybrid-array 0.2-rc.8 |
| 0.0.2 | 2024-05-31 | yes | |
| 0.0.1 | 2023-08-24 | yes | |

Crate summary: `max_version = 0.2.0-rc.5`, `max_stable_version = 0.1.0`, `newest = 0.2.0-rc.5`,
`updated = 2026-04-28`.

Dependency stacks (`/dependencies` endpoint, normal deps):

| crate@version | signature | hybrid-array | rand_core | digest/sha2 |
|---|---|---|---|---|
| slh-dsa 0.2.0-rc.5 | ^3.0.0-rc.10 | ^0.4 | ^0.10 | ^0.11 |
| slh-dsa 0.1.0 | ^2.3.0-pre.4 | ^0.2.0-rc.8 | ^0.6.4 | ^0.10 |
| ml-dsa 0.1.1 | ^3 | ^0.4 | (n/a) | shake ^0.1 |

### ml-dsa — `GET /crates/ml-dsa/versions` (21 versions; newest first: 0.1.1 2026-06-05 stable)

**Pin `0.1.1` is the newest version. No 0.2 line exists (stable or rc). No change.**

### p256 — `GET /crates/p256/versions` (61 versions)

**`0.14.0` STABLE EXISTS, released 2026-07-03** (after 16 rc + 12 pre). Consumer pins `0.13.2`
(2023-04-15). A verified bump candidate exists — see the patch proposal; it is the ES256 lane's
call, not this ticket's.

### signature — `GET /crates/signature/versions`

`3.0.0` stable released 2026-05-02 (rc line rc.4..rc.10 back to 2025-09). The consumer's `=`
absence on `signature = "2.2"` currently resolves TWO signature majors side by side
(2.2.0 for p256 0.13.2, 3.0.0 for the PQ stack) — legal because the plane composes hybrids at the
encoded-bytes level and never bridges signature traits across crates.

## Evidence: the current pin still builds and tests green (observed this session)

Scratch crate `/tmp/w5-w1-lane3/scratch-a/` — verbatim copies of the RENDERED modules
`src/crypto_trust_canonical.rs`, `src/crypto_trust_keys.rs`, `src/crypto_trust_es256.rs`,
`src/crypto_trust_pqc.rs` with the consumer's exact crypto-trust pins:

```toml
signature = "2.2"
p256 = "0.13.2"
ml-dsa = "0.1.1"
slh-dsa = "=0.2.0-rc.5"
rand_core = { version = "0.6", features = ["getrandom"] }
```

```
cargo test  (CARGO_TARGET_DIR=/tmp/ctp-shared-target)
exit 0
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured  (18.22s)
```

Includes the SLH-DSA teeth: `slh_dsa128s_round_trip`, `..._encoded_lengths_match_fips_205`
(7856 B), `..._is_deterministic_in_seeds_and_message`, tamper-msg/sig falsifiers, cross-key
refusal, malformed-pk/sig typed refusals.

Resolved lock (scratch-a/Cargo.lock): slh-dsa 0.2.0-rc.5 with **signature 3.0.0 (stable)**,
rand_core 0.10.1, hybrid-array 0.4.15 — plus signature 2.2.0 / rand_core 0.6.4 for the p256 0.13.2
half.

## Upgrade law (when does the pin move?)

- **Trigger**: a `slh-dsa` 0.2.x STABLE (or 0.1.1+) release on the signature-3/rand_core-0.10
  stack. Watch `https://crates.io/crates/slh-dsa` (RustCrypto; the rc cadence suggests the
  0.2.0 final tracks `signature 3.0.0` stability — signature 3.0.0 itself already landed
  2026-05-02, so the gate is slh-dsa's own release).
- **Mechanics at trigger time**: change exactly one Cargo.toml byte-sequence
  (`"=0.2.0-rc.5"` → `"0.2"` or the stable `=` pin), rerun the scratch test battery
  (this ticket's scratch-a is the harness: `cd /tmp/w5-w1-lane3/scratch-a && cargo test`);
  API-delta watchpoints if upstream drifted: `Sha2_128s` naming, `slh_keygen_internal` signature,
  `Signature::<Sha2_128s>::try_from` / `VerifyingKey::<Sha2_128s>::try_from` encodings,
  `slh_dsa::signature::{Signer, Verifier, Keypair}` re-export paths.
- **Pin law**: while on ANY pre-release, keep the `=` exact pin. A caret range
  (`"0.2.0-rc.5"` without `=` would still NOT auto-jump majors, but `=` is deliberate:
  rc lines have historically broken between rc.2→rc.3 (2026-01-05 → 2026-01-27). Exact pin +
  explicit upgrade transition = no surprise API drift inside a release.
- **Risk stated honestly**: pre-release = API drift until 0.2.0 final; the `=` pin contains it to
  one deliberate transition. The 7856-byte signature length, 2n=32 pk length, determinism, and
  tamper falsifiers are permanent tripwires: any rc that breaks FIPS 205 semantics fails the test
  battery before it can ship.

## p256 0.14.0 bump candidate (evidence for the ES256 lane — proposal only, NOT applied)

Scratch `/tmp/w5-w1-lane3/scratch-c/` (same rendered modules) with
`signature = "3"`, `p256 = { version = "0.14.0", features = ["getrandom"] }`, everything else
identical, plus the two es256 template edits in `/tmp/w5-w1-lane3/ctp-es256-template.patch`:

```
cargo test  (CARGO_TARGET_DIR=/tmp/ctp-shared-target)
exit 0
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured  (18.62s)
```

API deltas p256 0.13.2 → 0.14.0 (all four witnessed as compile errors in scratch-b, exit 101):

1. `ecdsa 0.16 → 0.17`: Signer/Verifier move to signature 3 → bump consumer `signature` to `"3"`
   (only `crypto_trust_es256.rs` imports it directly; collapses the dual-major lock to one
   signature 3.0.0).
2. `SigningKey::random(&mut rand_core::OsRng)` — gone: deprecated in ecdsa 0.17 ("use the
   `Generate` trait instead") AND rand_core 0.10 has no `OsRng`. Replacement:
   `SigningKey::try_generate()` via `p256::elliptic_curve::Generate` with p256 feature
   `getrandom` — typed refusal, preserves the house no-panic law.
3. `verifying_key().to_encoded_point(false).as_bytes()` — gone. Replacement:
   `to_sec1_bytes()` (same uncompressed SEC1 2.3.3 `0x04||X||Y`, 65 B).
4. `rand_core` consumer pin stays `0.6 + getrandom` for the pqc half (its `OsRng` seed
   generation is untouched); the RustCrypto internals carry rand_core 0.10 transitively —
   dual rand_core in the lock is inherent to the ecosystem's current rc line and harmless under
   the encoded-bytes composition law.

## History

- ts 2026-09-29 | standing ALIVE | branch feat/v26.9.28-crypto-trust-plane (coordinator-owned tree; lane wrote only this file) | gates: cargo test scratch-a exit 0 (80/80, current pins), cargo test scratch-c exit 0 (80/80, p256 candidate), crates.io API cited per claim | remaining: coordinator admits patch proposal or defers p256 bump to ES256 lane
