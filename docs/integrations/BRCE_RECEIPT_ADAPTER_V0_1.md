# BRCE Receipt Adapter Profile v0.1

**Specification status:** FINAL_SPEC v0.1 (v26.9.24)  
**Implementation standing:** PARTIAL_ALIVE (v26.9.25) — steps 1-5 implemented in `src/brce.rs`; TLA+ ingestion (step 6) and CLI verbs (step 7) NOT_CLAIMED; see "v26.9.25 implementation" below  
**Normative source:** BRCE Protocol RFC v0.1 in `engineering-standards`  
**Affidavit role:** evidence carrier and verifier only

## Boundary

Affidavit does not define BRCE and MUST NOT become its authority broker, planner, or actuator.

The adapter implements the evidence side of:

[
BRCE consequence
\rightarrow
execution evidence
\rightarrow
Affidavit receipt
\rightarrow
verification
]

The governing separation remains:

[
ReceiptValid \not\Rightarrow AuthorityGranted
]

and:

[
Verification \not\Rightarrow DO
]

Affidavit's existing doctrine, **certify, don't decide**, therefore remains unchanged.

## Profiles

This adapter reserves the following profile identifiers:

```text
affidavit/brce-actuation/v1
affidavit/brce-reconciliation/v1
affidavit/brce-replay/v1
affidavit/tla-model-check/v1
affidavit/tla-proof/v1
```

Only the BRCE profiles are required for BRCE Core/Replay integration. TLA+ profiles are optional formal-evidence adapters.

## 1. BRCE actuation receipt

A BRCE actuation receipt binds one exact consequence attempt to the authority and constructed action that permitted it.

Required logical fields:

```text
profile
receipt_id
run_id

subject
request_digest
route_digest
admission_digest
construct_digest
authority_grant_digest

consequence_id
attempt_id

actuation
  executor_identity
  started_at
  completed_at
  result_class
  result_digest

effect
  observed
  changed
  effect_digest

verification
  verifier_identity
  verdict
  evidence_digest

replay
  command_or_entrypoint
  tool_identity
  tool_version
  config_digest
  dependency_digest

integrity
  canonicalization
  receipt_digest
  previous_receipt
```

## 2. Exact-subject law

Every receipt MUST bind the exact subject that was executed.

Evidence for subject (S_1) cannot be used to certify subject (S_2):

[
Receipt(S_1) \not\Rightarrow Standing(S_2)
quad	ext{when}quad
S_1 \neq S_2
]

## 3. Authority evidence ceiling

The receipt records the authority grant used by DO.

It MUST NOT manufacture authority.

A valid receipt can certify:

- which grant digest was presented;
- which construct digest it bound;
- which operation and target were executed;
- the observed result;
- verifier evidence;
- replay identity.

It cannot certify that the issuer was entitled to grant authority unless an independent authority-verification profile establishes that fact.

Therefore:

[
ValidReceipt \not\Rightarrow ValidIssuerAuthority
]

unless the profile explicitly includes and verifies that proof obligation.

## 4. Consequence identity

The adapter MUST preserve BRCE `consequence_id`.

Multiple attempts MAY share one consequence ID:

[
attempt_1, attempt_2, \dots \mapsto consequenceId
]

but an at-most-once profile MUST reject evidence showing more than one distinct effect for that consequence ID.

## 5. Execution versus change

Affidavit MUST keep these fields distinct:

```text
executed
changed
verified
```

A successful idempotent operation may have:

```text
executed = true
changed = false
verified = true
```

No adapter may collapse that into `changed = true`.

## 6. Crash-window reconciliation

For non-atomic external effects, BRCE permits:

[
PREPARED
\rightarrow DO
\rightarrow RECONCILE
\rightarrow RECEIPTED
]

The reconciliation profile binds:

```text
profile = affidavit/brce-reconciliation/v1
consequence_id
prepared_record_digest
construct_digest
authority_grant_digest
observation_source
observed_external_state
reconciliation_verdict
reconciliation_evidence_digest
```

Permitted verdicts include:

```text
EFFECT_CONFIRMED
NO_EFFECT_CONFIRMED
EXECUTION_UNKNOWN
BLOCKED_RECONCILIATION
```

`EXECUTION_UNKNOWN` MUST NOT be promoted to executed or changed.

## 7. Replay profile

`affidavit/brce-replay/v1` binds enough identity to reconstruct the evidence-producing path without reproducing the original external consequence.

It SHOULD bind:

- exact subject;
- original receipt digest;
- input/request digest;
- tool identity/version;
- configuration digest;
- dependency/toolchain identity;
- policy identity where material;
- environment identity where material;
- replay verifier verdict.

Replay MUST remain consequence-free unless a new BRCE DO evaluation occurs.

[
ReplayReceipt \not\Rightarrow ActuationAuthority
]

## 8. TLA+ evidence adapter

TLA+ is optional BRCE Formal evidence.

### 8.1 Model-check profile

`affidavit/tla-model-check/v1` SHOULD bind:

```text
subject
spec_digest
config_digest
tool_identity
tool_version
command
exit_code
properties_checked
states_generated
distinct_states
result
counterexample_digest
stdout_digest
stderr_digest
```

A PASS means only:

> the named properties held for the reachable state space explored by this exact model, configuration, and verifier invocation.

It does not establish production correctness, execution, deployment, or BRCE authority.

### 8.2 Proof profile

`affidavit/tla-proof/v1` MAY bind TLAPS or another admitted proof system's exact proof input, toolchain, obligations, and result.

## 9. Counterexamples

Formal counterexamples are first-class evidence.

A preserved counterexample SHOULD bind:

```text
property
initial_state_digest
transition_trace_digest
terminal_state_digest
spec_digest
config_digest
tool_identity
replay_command
```

A repaired implementation SHOULD retain the counterexample as a permanent regression witness.

## 10. Verification rules

A BRCE receipt verifier MUST reject at least:

- malformed profile/version;
- changed subject identity;
- construct digest mismatch;
- authority-grant digest mismatch;
- receipt digest mismatch;
- duplicate consequence where the profile requires at-most-once effect;
- `EXECUTION_UNKNOWN` narrated as confirmed execution;
- replay identity missing fields required by the profile;
- tampered counterexample or formal-tool output commitments.

## 11. Standing ceiling

This adapter MAY establish receipt-level evidence such as:

```text
RECEIPT_VALID
EFFECT_OBSERVED
EFFECT_VERIFIED
REPLAY_VERIFIED
MODEL_CHECK_PASS
MODEL_CHECK_COUNTEREXAMPLE
```

It MUST NOT independently establish:

```text
AUTHORITY_GRANTED
PRODUCTION_CORRECT
DEPLOYED
PUBLISHED
ALIVE
```

unless a separate admitted standing policy consumes the verified receipt together with the additional required evidence.

## 12. Implementation order

1. Define canonical Rust structures for the three BRCE receipt profiles.
2. Add deterministic serialization and digest verification.
3. Add falsifier tests for subject/construct/authority substitution.
4. Add crash-window reconciliation fixtures.
5. Add consequence-free replay tests.
6. Add optional TLA+ model-check receipt ingestion.
7. Only then expose CLI verbs.

Generated or convenience surfaces MUST NOT precede the core verifier.

## 13. Falsifiers

This adapter is incorrect if any supported path permits:

[
ValidReceipt
land
TamperedSubject
]

or:

[
EXECUTION\_UNKNOWN
\Rightarrow
Executed
]

or:

[
Replay
\Rightarrow
UnexpectedNewConsequence
]

or if receipt verification itself grants DO authority.

## Status ceiling for this document

This document specifies an adapter contract only.

No BRCE runtime implementation, TLA+ execution, or new Affidavit receipt verifier is claimed by this documentation PR.


## v26.9.24 specification closure

The receipt-adapter contract is complete as a specification. Runtime verifier, BRCE actuation integration, and TLA+ execution remain outside this document's evidence ceiling until separately implemented and observed.


## v26.9.25 implementation

Standing is scoped to what `tests/brce_ledger.rs` and `examples/brce_court.rs` execute.

| Step | Scope | Standing | Evidence |
|---|---|---|---|
| 1 | Rust structures for `brce-actuation/v1`, `brce-reconciliation/v1`, `brce-replay/v1` | ALIVE | `src/brce.rs` `BrceReceipt`, `Entry`, `PROFILE_*` |
| 2 | Deterministic serialization + digest verification | ALIVE | `digest`, `BrceReceipt::compute_digest`, hash-chained `BrceLedger` |
| 3 | Subject / construct / authority substitution falsifiers | ALIVE | `do_without_valid_authority_is_refused_with_zero_consequence` |
| 4 | Crash-window reconciliation fixtures | ALIVE | three `crash_*` tests + `unobservable_crash_window_stays_refused` |
| 5 | Consequence-free replay | ALIVE | `replay_is_consequence_free_and_digest_equal` |
| 6 | TLA+ model-check receipt ingestion | NOT_CLAIMED | not implemented |
| 7 | CLI verbs | NOT_CLAIMED | not implemented (court runs via `cargo run --example brce_court`) |

Components:

- `BrceLedger`: append-only JSON-lines ledger; each record is BLAKE3-chained
  (`H(seq, prev, entry)`) and fsync'd on append. The ledger accepts any entry; it is not an
  authority.
- `BrcePipeline`: parse -> route -> admit/refuse -> construct -> DO (`prepare` writes the
  RFC-0001 §14 `PREPARED` intent before the actuator runs, `execute` records `Done`) -> receipt.
  DO is refused unless the grant binds subject, operation, target and construct digest, is
  unexpired and has uses left. A non-idempotent retry of a consequence with an open crash window
  is BLOCKED (`RECONCILE_BEFORE_RETRY`).
- `BrcePipeline::reconcile`: every `PREPARED` consequence without a receipt resolves to
  `EFFECT_CONFIRMED` (a `brce-reconciliation/v1` receipt is persisted), `NO_EFFECT_CONFIRMED`,
  `EXECUTION_UNKNOWN` or `BLOCKED_RECONCILIATION`. The last two are never promoted to executed.
- `court(ledger, world)`: refuses on eight rules: `CHAIN_INTEGRITY`,
  `ZERO_UNRECEIPTED_ACTUATION` (every `Done` and every consequence observed in the world has a
  receipt), `DO_REQUIRES_AUTHORITY`, `CONSTRUCT_REQUIRES_ADMISSION`, `RECEIPT_DIGEST_VALID`,
  `AT_MOST_ONCE_CONSEQUENCE`, `CRASH_WINDOW_RECONCILED`, `UNKNOWN_NOT_PROMOTED`. It reports a
  consequence-free `replay_digest` recomputed from ledger content.
- `mutant_suite`: one unlawful mutation per rule; each must be refused by its rule.

Reproduce:

```bash
cargo test --test brce_ledger
cargo run --example brce_court -- <work_dir> <subject_sha>   # exit 0 iff admitted + replay equal + all mutants killed
```

The evidence ceiling in §11 is unchanged: a court `ADMITTED` verdict is receipt-level evidence
and grants no authority (`Verification ⇏ DO`).
