# Capability Crosswalk — sjira wave v26.10.17

Projection of `docs/reference/CAPABILITY_REGISTER.md` (102 rows) into the sjira
work-order structure. Disposition sweep: commit b0325d4 — **50 LANDED /
1 MERGED / 49 OPEN / 2 ADJUDICATION**, plus registered divergence **KD-BLS-1**.
Standing of every row remains UNKNOWN until its falsifier court runs on the
exact subject; LANDED here means the register's disposition column carries a
landing commit, not that the falsifier is re-proven today.

## 1. Domain crosswalk: capability domain -> landed rows -> falsifier court

| Capability domain | Landed rows (register #) | Landing commits | Falsifier court (one-line) |
|---|---|---|---|
| Signature parity / Ed25519 | 1, 31, 88, 89 | 32a5f13, 0afe89e, 26e7c0e | malleated sig refused identically host+wasm in differential court; Debug containing any seed byte = FAIL |
| PQC: ML-DSA (signing) | 25, 26, 67, 68 | c7bad41, 5cd7eb9 | cached vs per-call sigs byte-identical; from_expanded(to_expanded(esk)) byte-identical; screening control fires |
| PQC: SLH-DSA (signing) | 2, 12, 27, 28, 52 | 74801fc, a4b65c8, 9b36a3d, 5cd7eb9 | swapped opt_rand without sig flip = FAIL; SigLen drift or ±1-byte accept flips court |
| PQC: ML-KEM | 2, 29, 30 | 74801fc, eba7b49, 3ea786d (fixup e79ef96) | fabricated seed accepted or Some-on-expanded = FAIL; single-implementation KAT agreement = vacuous FAIL |
| PQC: X-Wing hybrid | 86, 87 | a16f4ba | dk.encapsulation_key() != original pk bytes = FAIL; collapsed divergence between dual modes = FAIL |
| Hashing / blake3 seams | 7, 23, 24, 48, 49, 79 | a05c9c1, 6153dc4, 88e9ac4, c7786a4, f3ed0c4, dbac2da | no-reset mutation flips differential court; writer-vs-render digests byte-identical on 1MB corpus; resume-from-count != fresh = FAIL |
| Canonicalization / JCS | 3, 37, 38, 60, 61, 97 | 5b190fe, 3ade36e, f5cd9f9 | float-format or duplicate-key corpus row flips parity court to divergence; untagged blake3(canonical) acceptance = FAIL |
| ZK range proofs (bulletproofs) | 13, 33, 34, 63, 91 | 82dbbe2, 3c285d5, 23c3774 | prove-under-A verify-under-B refused; same seed+label+value -> cmp-identical proof bytes; open(v±1) accepted = FAIL |
| SMT (monotree) | 35, 99 | 5bcdf74, ddec917 | stale shadow-index bug flips absence court; shuffled batch vs sequential root divergence = FAIL |
| OCEL / process mining (wasm4pm-compat) | 39, 40 | 51ee682 | severed E2O relationship flips ACCEPT->refusal; zero-move alignment -> typed refusal, not fitness 0.0 |
| Registry / CLI framework (clap-noun-verb, linkme) | 5, 20, 22, 46, 47, 58 | 359a705 (partial), 89fc68a, 8153451, 617817f, 513ab76 | duplicate `#[verb]` stub compiles but court flips InvalidStructure; removing one registration drops count and fails gate; wasm32 probe verdict REFUTED-PREMISE |
| Diagnostics / doctor | 46, 47 (shared with row above) | 617817f | rebuild x2 diff of `affi doctor --format json`; duplicate id fails gate; host >=7 checks, wasm32 loud UNSUPPORTED_REGISTRATION |
| Observability (opentelemetry) | 64, 74 | 805d030, 898b09f | otel plane builds (`--features otel --no-run` exit 0); over-band span with no TimingViolation cell = FAIL |
| Federation / standing | 44 | 3215f69 | out-of-vocab standing -> typed fail |
| Core kernels (bcinr, external evidence) | 77, 78 | 6534c13, f3871f5 | one-byte frame-t mutation flips chain_hash_t..n; flipping valid_until==now boundary flips a corpus cell |
| Lane hygiene (doc/tooling fixes) | n/a (6 items) | 9ab506c, d7c40d7, 2af1142, c774a2e, 1f7103a (DONE), fd04c66 (DONE) | each fixed against its named drift (completions hyphen, WASM.md op/error counts, docstring/AGENTS drift, justfile gaps, changelog) |

Totals: 50 LANDED rows across 15 domains + 1 MERGED (row 43 -> row 31).

## 2. OPEN queue — top 12 (ranked by register cost field, S first; defect/delete-class first)

| rank | reg # | Proposal | Seam | Cost | Falsifier (one-line) |
|---|---|---|---|---|---|
| 1 | 4 | `smt_inclusion_verify` wasm op (twin of absence op) | `affidavit-wasm/src/advanced.rs` + ontology re-project | S/M | tampered proof step -> `admitted:false`; wired-swap flips court |
| 2 | 98 | Parse-precision class repair (serde_json float_roundtrip; delete-class) | `Cargo.toml` both + jcs_parity.rs | S | corpus parse_precision count == 0 with feature; revert flips nonzero |
| 3 | 21 | Runtime registry court (clap `validate()` + `command_structure()` in doctor) | `src/verbs/doctor.rs` | S | duplicate `#[verb]` stub compiles, count parity passes, court flips InvalidStructure |
| 4 | 59 | Zero-size DoctorCheck law (size_of==0 compile court) | `src/doctor_check.rs` | S | 1-field check type fails to compile/register |
| 5 | 50 | validators seam for CLI admission gates (delete hand-rolled parsers) | `src/verbs/*.rs` | S | old check vs validators:: diverge on one fixture -> court flips |
| 6 | 51 | policies wire-in to doctor (TargetPressure/ToolchainMismatch as real checks) | `src/doctor_check.rs` | S | >max_gb size flips doctor verdict; removal drops count by 1 |
| 7 | 19 | Alias parity from keywords[0] (visible_alias per verb) | `src/cli.rs` | S | `affi receipt certify` resolves to verify, exit 0 |
| 8 | 32 | JWKS OKP import point-validation | `src/crypto_trust_jwks.rs` | S | 32x0xff x refused typed; real pk passes |
| 9 | 8 | ml-dsa context-string sealing (`sign_deterministic(M, ctx)`) | core `sealed.rs` + wasm `pqc.rs` | M | ctx-signed bytes fail empty-ctx verify and vice versa |
| 10 | 36 | Generic StateTree<D: Database> fault-injectable | `src/smt.rs:87-91` | S/M | injected DB fault -> typed refusal, fence fail-closed |
| 11 | 53 | SLH multipart domain-separation court | `pqc.rs tests` | S | reordered/missing chunk must fail verify |
| 12 | 56 | Ed25519 PKCS#8 key import/export round-trip | `src/ed25519_witness.rs` | S | mutated DER byte -> typed refusal; round-trip restores pk |

Full OPEN remainder (37 rows): 6, 9, 10, 11, 14, 15, 16, 17, 18, 41, 42, 54,
55, 57, 62, 65, 66, 69, 70, 71, 72, 73, 75, 76, 80, 81, 82, 83, 84, 85, 90,
93, 94, 95, 100, 101, 102 — see the register for seams, costs, falsifiers.

Delete-class OPEN items (ERRC: outrank generation): dead `verify_bip340`
(src/secp256k1_witness.rs:69, zero call sites) + its companion defect vectors;
`metrics`/opentelemetry-prometheus optional deps with zero consumers (wire as
rows 41/42 or delete the dep plane).

## 3. Adjudication pointer

Two rows and three standing law questions are operator-decision items, not
auto-landable — tracked in the register's "Adjudication items" section
(docs/reference/CAPABILITY_REGISTER.md):

- Row 92 — ECDSA RFC-6979 nonce-determinism + low-s normalization (form policy).
- Row 96 — CUSTOM_HANDLERS plugin slice: documented but nonexistent in src/
  (delete-or-land).
- Root-crate PQC feature flag vs "no PQC feature flags" law; envelope
  `Algorithm::{Es256, HybridEs256MlDsa65}` wire labels; classical EC code
  predating the PQC-only law (destructive removal -> operator call).

Registered divergence **KD-BLS-1** (BLS rogue-key forgery verifies) lives in
the register's KD table: close condition is PoP infrastructure or de-scope.

## 4. Mining-cycle provenance

Five dependency-mining cycles produced the register this crosswalk projects:

| cycle | date | lanes | rows added | signal |
|---|---|---|---|---|
| 1 | 2026-10-16 | 16 | 1-20 (+lane hygiene) | defect + capability harvest |
| 2 | 2026-10-16 | ~16-20 | 21-47 (+delete-class) | dup of 31 caught (MERGED); delete findings |
| 3 | 2026-10-16 | 16 | 48-78 | defect class: JCS dup-key, otel BUILD_BROKEN |
| 4 | 2026-10-17 | ~16-20 | 79-100 | defect class: SMT key-binding gap, dead fix.rs |
| 5 | 2026-10-17 | ~16-20 | 101-102 | EXHAUSTED signals begin |

**Saturation evidence**: cycles 1-4 yielded 20-31 new rows each; cycle 5
yielded 2 rows and lanes reported EXHAUSTED (no new dependency seams above
the dedupe floor). Per the annealing doctrine, saturation is reached when a
full wave returns zero defects — cycle 5's EXHAUSTED signals are the first
such evidence for the dependency-mining surface; the OPEN queue (49) plus the
adjudication items remain the live frontier, mined by falsifier courts, not
further enumeration.
