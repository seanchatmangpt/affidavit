# BRCE Receipt Adapter Profile v0.1

**Status:** Draft adapter specification  
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
