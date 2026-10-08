# affidavit reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### affidavit-core/src/accumulator/mmr.rs

| `MmrError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) |  |  |  |  |

| `peaks` | function | peaks(&self) |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `MmrAccumulator` | struct |  |  |  |  |  |

| `MmrProof` | struct |  |  |  |  |  |

| `MountainPeak` | struct |  |  |  |  |  |


### affidavit-core/src/accumulator/mmr.rs

| `MmrError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) |  |  |  |  |

| `peaks` | function | peaks(&self) |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `MmrAccumulator` | struct |  |  |  |  |  |

| `MmrProof` | struct |  |  |  |  |  |

| `MountainPeak` | struct |  |  |  |  |  |


### affidavit-core/src/chain.rs

| `borrow` | function | borrow(&self) |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `ChainBuilder` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `OwnedEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Seal` | struct |  |  |  |  |  |


### affidavit-core/src/chain.rs

| `borrow` | function | borrow(&self) |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `ChainBuilder` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `OwnedEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Seal` | struct |  |  |  |  |  |


### affidavit-core/src/crypto_verify.rs

| `Algorithm` | enum |  |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `Profile` | enum |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `wire_str` | function | wire_str(self) |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) |  |  |  |  |

| `EnvelopeRef` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |


### affidavit-core/src/crypto_verify.rs

| `Algorithm` | enum |  |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `Profile` | enum |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `wire_str` | function | wire_str(self) |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) |  |  |  |  |

| `EnvelopeRef` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |


### affidavit-core/src/digest.rs

| `as_bytes` | function | as_bytes(&self) |  |  |  |  |

| `is_zero` | function | is_zero(&self) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |


### affidavit-core/src/digest.rs

| `as_bytes` | function | as_bytes(&self) |  |  |  |  |

| `is_zero` | function | is_zero(&self) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |


### affidavit-core/src/external_evidence.rs

| `EvidenceError` | enum |  |  |  |  |  |

| `SvidType` | enum |  |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence(
    request: AuthZenRequestRef<'_>,
    decision: AuthZenDecisionEvidenceRef<'_>,
    expected_policy_decision_point: &str,
    expected_principal: &str,
    expected_effect_digest: &str,
) |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity(
    evidence: WorkloadIdentityEvidenceRef<'_>,
    expected_spiffe_id: &str,
    expected_trust_domain: &str,
    allow_jwt: bool,
) |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `parse` | function | parse(raw: &'a str) |  |  |  |  |

| `AuthZenActionRef` | struct |  |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct |  |  |  |  |  |

| `AuthZenEntityRef` | struct |  |  |  |  |  |

| `AuthZenRequestRef` | struct |  |  |  |  |  |

| `SpiffeIdRef` | struct |  |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct |  |  |  |  |  |


### affidavit-core/src/external_evidence.rs

| `EvidenceError` | enum |  |  |  |  |  |

| `SvidType` | enum |  |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence(
    request: AuthZenRequestRef<'_>,
    decision: AuthZenDecisionEvidenceRef<'_>,
    expected_policy_decision_point: &str,
    expected_principal: &str,
    expected_effect_digest: &str,
) |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity(
    evidence: WorkloadIdentityEvidenceRef<'_>,
    expected_spiffe_id: &str,
    expected_trust_domain: &str,
    allow_jwt: bool,
) |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `parse` | function | parse(raw: &'a str) |  |  |  |  |

| `AuthZenActionRef` | struct |  |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct |  |  |  |  |  |

| `AuthZenEntityRef` | struct |  |  |  |  |  |

| `AuthZenRequestRef` | struct |  |  |  |  |  |

| `SpiffeIdRef` | struct |  |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct |  |  |  |  |  |


### affidavit-core/src/mining/conformance.rs

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `fitness` | function | fitness(&self) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |


### affidavit-core/src/mining/conformance.rs

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `fitness` | function | fitness(&self) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |


### affidavit-core/src/mining/footprint.rs

| `AlphaRelation` | enum |  |  |  |  |  |

| `activities` | function | activities(&self) |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) |  |  |  |  |

| `Footprint` | struct |  |  |  |  |  |


### affidavit-core/src/mining/footprint.rs

| `AlphaRelation` | enum |  |  |  |  |  |

| `activities` | function | activities(&self) |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) |  |  |  |  |

| `Footprint` | struct |  |  |  |  |  |


### affidavit-core/src/mining/mod.rs

| `activity_list` | function | activity_list(&self) |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `DirectlyFollowsGraph` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |


### affidavit-core/src/mining/mod.rs

| `activity_list` | function | activity_list(&self) |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `DirectlyFollowsGraph` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |


### affidavit-core/src/mining/stats.rs

| `distinct_activities` | function | distinct_activities(&self) |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) |  |  |  |  |

| `LogStatistics` | struct |  |  |  |  |  |


### affidavit-core/src/mining/stats.rs

| `distinct_activities` | function | distinct_activities(&self) |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) |  |  |  |  |

| `LogStatistics` | struct |  |  |  |  |  |


### affidavit-core/src/verifier.rs

| `RejectReason` | enum |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `is_accept` | function | is_accept(&self) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |


### affidavit-core/src/verifier.rs

| `RejectReason` | enum |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `is_accept` | function | is_accept(&self) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |


### affidavit-wasm/src/abi.rs

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `AbiError` | struct |  |  |  |  |  |


### affidavit-wasm/src/abi.rs

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `AbiError` | struct |  |  |  |  |  |


### affidavit-wasm/src/advanced.rs

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) |  |  |  |  |


### affidavit-wasm/src/advanced.rs

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) |  |  |  |  |


### affidavit-wasm/src/crypto.rs

| `SignatureInputError` | enum |  |  |  |  |  |

| `code` | function | code(&self) |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) |  |  |  |  |


### affidavit-wasm/src/crypto.rs

| `SignatureInputError` | enum |  |  |  |  |  |

| `code` | function | code(&self) |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) |  |  |  |  |


### affidavit-wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


### affidavit-wasm/src/ffi.rs

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


### affidavit-wasm/src/receipt.rs

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |


### affidavit-wasm/src/receipt.rs

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |


### affidavit-wasm/tests/common/mod.rs

| `call` | function | call(&mut self, request: Value) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |

| `Host` | struct |  |  |  |  |  |


### affidavit-wasm/tests/common/mod.rs

| `call` | function | call(&mut self, request: Value) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |

| `Host` | struct |  |  |  |  |  |


### affidavit-web

| `build` | script | next build |  |  |  |  |

| `dev` | script | next dev |  |  |  |  |

| `lint` | script | next lint |  |  |  |  |

| `start` | script | next start |  |  |  |  |


### praxis/crates/chatman-common/src/cli.rs

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `bold` | function | bold(text: &str) |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) |  |  |  |  |

| `dim` | function | dim(text: &str) |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) |  |  |  |  |

| `green` | function | green(text: &str) |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) |  |  |  |  |

| `red` | function | red(text: &str) |  |  |  |  |

| `yellow` | function | yellow(text: &str) |  |  |  |  |

| `GlobalArgs` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/cli.rs

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `bold` | function | bold(text: &str) |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) |  |  |  |  |

| `dim` | function | dim(text: &str) |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) |  |  |  |  |

| `green` | function | green(text: &str) |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) |  |  |  |  |

| `red` | function | red(text: &str) |  |  |  |  |

| `yellow` | function | yellow(text: &str) |  |  |  |  |

| `GlobalArgs` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/error.rs

| `Error` | enum |  |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) |  |  |  |  |


### praxis/crates/chatman-common/src/error.rs

| `Error` | enum |  |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) |  |  |  |  |


### praxis/crates/chatman-common/src/provenance.rs

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `current` | function | current(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(domain: &str) |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct |  |  |  |  |  |

| `RollingHash` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/provenance.rs

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `current` | function | current(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(domain: &str) |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct |  |  |  |  |  |

| `RollingHash` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/telemetry.rs

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) |  |  |  |  |

| `new` | function | new(service_name: &str) |  |  |  |  |

| `TracingGuard` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/telemetry.rs

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) |  |  |  |  |

| `new` | function | new(service_name: &str) |  |  |  |  |

| `TracingGuard` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/testkit.rs

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder() |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) |  |  |  |  |

| `dir` | function | dir(&self) |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) |  |  |  |  |

| `TempReceipt` | struct |  |  |  |  |  |

| `TempReceiptBuilder` | struct |  |  |  |  |  |


### praxis/crates/chatman-common/src/testkit.rs

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder() |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) |  |  |  |  |

| `dir` | function | dir(&self) |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) |  |  |  |  |

| `TempReceipt` | struct |  |  |  |  |  |

| `TempReceiptBuilder` | struct |  |  |  |  |  |


### praxis/template/src/chain.rs

| `append` | function | append(&mut self, event_bytes: &[u8]) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |


### praxis/template/src/chain.rs

| `append` | function | append(&mut self, event_bytes: &[u8]) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |


### praxis/template/src/cli.rs

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `enabled` | function | enabled(self) |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |


### praxis/template/src/cli.rs

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `enabled` | function | enabled(self) |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |


### praxis/template/src/error.rs

| `AppError` | enum |  |  |  |  |  |


### praxis/template/src/error.rs

| `AppError` | enum |  |  |  |  |  |


### praxis/template/src/types.rs

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |


### praxis/template/src/types.rs

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |


### src/1000x_auto_remediate_dx.rs

| `new` | function | new(receipt_path: impl AsRef<Path>, source_dir: impl AsRef<Path>) |  |  |  |  |

| `remediate` | function | remediate(&self) |  |  |  |  |

| `AutoRemediator` | struct |  |  |  |  |  |


### src/1000x_autonomous_governance.rs

| `audit_workspace` | function | audit_workspace(&self) |  |  |  |  |

| `handle_governance_audit` | function | handle_governance_audit() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ArchitectureProposal` | struct |  |  |  |  |  |

| `GovernanceAgent` | struct |  |  |  |  |  |

| `GovernanceReport` | struct |  |  |  |  |  |


### src/1000x_chaos_e2e.rs

| `verify_with_chaos` | function | verify_with_chaos(&self, receipt: &Receipt) |  |  |  |  |

| `ChaosVerifier` | struct |  |  |  |  |  |


### src/1000x_cli_telepathy_qol.rs

| `predict` | function | predict(&self) |  |  |  |  |

| `run_cli` | function | run_cli() |  |  |  |  |

| `shell_integration` | function | shell_integration() |  |  |  |  |

| `Prediction` | struct |  |  |  |  |  |

| `Telepathy` | struct |  |  |  |  |  |


### src/1000x_distributed_sharding.rs

| `ShardingError` | enum |  |  |  |  |  |

| `new` | function | new(dht: Arc<dyn KademliaDHT>) |  |  |  |  |

| `shard_receipt` | function | shard_receipt(receipt: Receipt, shard_size: usize) |  |  |  |  |

| `verify_distributed` | function | verify_distributed(&self, receipt_id: &Blake3Hash) |  |  |  |  |

| `DistributedVerifier` | struct |  |  |  |  |  |

| `ReceiptManifest` | struct |  |  |  |  |  |

| `ReceiptShard` | struct |  |  |  |  |  |

| `KademliaDHT` | trait |  |  |  |  |  |


### src/1000x_formal_verification_spec.rs

| `State` | enum |  |  |  |  |  |

| `get` | function | get() |  |  |  |  |

| `init` | function | init() |  |  |  |  |

| `terminate` | function | terminate() |  |  |  |  |

| `transition_to` | function | transition_to(expected_prev: State, next: State) |  |  |  |  |

| `CurrentState` | struct |  |  |  |  |  |


### src/1000x_gpu_verifier.rs

| `is_accepted` | function | is_accepted(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prepare_batch` | function | prepare_batch(
        receipts: &[crate::types::Receipt],
    ) |  |  |  |  |

| `verify_batch` | function | verify_batch(
        &self,
        events: &[GpuEvent],
        metadata: &[GpuReceiptMetadata],
    ) |  |  |  |  |

| `GpuEvent` | struct |  |  |  |  |  |

| `GpuReceiptMetadata` | struct |  |  |  |  |  |

| `GpuVerdict` | struct |  |  |  |  |  |

| `GpuVerifier` | struct |  |  |  |  |  |


### src/1000x_holographic_lsp_dx.rs

| `get_hologram_svg` | function | get_hologram_svg(events: &[OperationEvent]) |  |  |  |  |


### src/1000x_nlp_query_qol.rs

| `new` | function | new() |  |  |  |  |

| `parse` | function | parse(&self, input: &str) |  |  |  |  |

| `NlpQueryParser` | struct |  |  |  |  |  |


### src/1000x_otel_hyper_spec.rs

| `all_attribute_keys` | function | all_attribute_keys() |  |  |  |  |

| `record_crypto_blake3_maximalist` | function | record_crypto_blake3_maximalist(
        _bytes_hashed: u64,
        _duration_ns: u64,
    ) |  |  |  |  |

| `record_wasm_verify_maximalist` | function | record_wasm_verify_maximalist(
        _module_hash: &str,
        _instruction_count: u64,
        _memory_peak: u64,
    ) |  |  |  |  |


### src/1000x_receipt_to_wasm_qol.rs

| `compile` | function | compile(&self) |  |  |  |  |

| `new` | function | new(receipt: Receipt) |  |  |  |  |

| `ReceiptWasmCompiler` | struct |  |  |  |  |  |


### src/1000x_time_travel_dx.rs

| `run_repl` | function | run_repl(&mut self) |  |  |  |  |

| `TimeTravelDebugger` | struct |  |  |  |  |  |


### src/admission.rs

| `AffidavitRefusal` | enum |  |  |  |  |  |

| `admit` | function | admit(receipt: Receipt) |  |  |  |  |


### src/architecture.rs

| `ArchitectureRefusal` | enum |  |  |  |  |  |

| `ArchitectureStanding` | enum |  |  |  |  |  |

| `EvidenceSource` | enum |  |  |  |  |  |

| `admit` | function | admit(
        &mut self,
        receipt: ArchitectureQualificationReceipt,
    ) |  |  |  |  |

| `admit_supersession` | function | admit_supersession(
        &mut self,
        supersession: Supersession,
    ) |  |  |  |  |

| `binding_digest` | function | binding_digest(&self) |  |  |  |  |

| `certify` | function | certify(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        qualification_evidence_digests: Vec<String>,
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) |  |  |  |  |

| `certify_from_evidence` | function | certify_from_evidence(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        evidence: &[QualificationEvidence],
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) |  |  |  |  |

| `current_qualified` | function | current_qualified(&self, abb_digest: &str) |  |  |  |  |

| `from_json_verified` | function | from_json_verified(json: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe` | function | observe(
        source: EvidenceSource,
        producer_digest: impl Into<String>,
        bytes: &[u8],
    ) |  |  |  |  |

| `query_json` | function | query_json(&self, abb_digest: &str) |  |  |  |  |

| `standing_of` | function | standing_of(&self, receipt_digest: &str) |  |  |  |  |

| `supersede` | function | supersede(
        &self,
        new_sbb_digest: impl Into<String>,
        new_subject_digest: impl Into<String>,
        evidence: Vec<String>,
    ) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `verify` | function | verify(
        &self,
        prior: &ArchitectureQualificationReceipt,
    ) |  |  |  |  |

| `verify_bytes` | function | verify_bytes(&self, bytes: &[u8]) |  |  |  |  |

| `verify_chain` | function | verify_chain(&self, prior: &Self) |  |  |  |  |

| `verify_evidence` | function | verify_evidence(
        &self,
        observed: &[(QualificationEvidence, &[u8]) |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) |  |  |  |  |

| `verify_replay` | function | verify_replay(
        &self,
        expected_abb_digest: &str,
        expected_contract_digest: &str,
        expected_sbb_digest: &str,
        expected_subject_digest: &str,
    ) |  |  |  |  |

| `ArchitectureQualificationReceipt` | struct |  |  |  |  |  |

| `ArchitectureStandingLedger` | struct |  |  |  |  |  |

| `QualificationEvidence` | struct |  |  |  |  |  |

| `Supersession` | struct |  |  |  |  |  |


### src/authority_fence.rs

| `DoWitness` | enum |  |  |  |  |  |

| `FastPathRefusal` | enum |  |  |  |  |  |

| `FenceError` | enum |  |  |  |  |  |

| `confirm` | function | confirm(&self, fence: &AuthorityFence) |  |  |  |  |

| `gate_do` | function | gate_do(witness: &DoWitness, live_root: Option<StateRoot>) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prepare_do_permit` | function | prepare_do_permit(&mut self, id: &StateKey) |  |  |  |  |

| `prove_revoked` | function | prove_revoked(&mut self, id: &StateKey) |  |  |  |  |

| `revoke` | function | revoke(&mut self, id: &StateKey) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `routes_to_sibling_slot` | function | routes_to_sibling_slot(queried: &StateKey, neighbor: &StateKey, depth: usize) |  |  |  |  |

| `unrevoke` | function | unrevoke(&mut self, id: &StateKey) |  |  |  |  |

| `verify_fast_path` | function | verify_fast_path(
        fence: &mut AuthorityFence,
        replay: &mut ReplayFilter,
        clock: &mut HlcClock,
        claim: &ConsequenceClaim,
    ) |  |  |  |  |

| `AdmittedProof` | struct |  |  |  |  |  |

| `AuthorityFence` | struct |  |  |  |  |  |

| `ConsequenceClaim` | struct |  |  |  |  |  |


### src/bench.rs

| `bench_throughput` | function | bench_throughput(iterations: u32) |  |  |  |  |

| `bench_variance_on_receipt` | function | bench_variance_on_receipt(path: &str, iterations: u32) |  |  |  |  |

| `bench_variance_suite` | function | bench_variance_suite(_iterations: u32) |  |  |  |  |

| `run_profile_workload` | function | run_profile_workload(seconds: u64, _receipt_path: Option<&str>) |  |  |  |  |


### src/bin/affi-shell.rs

| `run` | function | run() |  |  |  |  |


### src/binary_envelope.rs

| `EnvelopeError` | enum |  |  |  |  |  |


### src/bls_aggregate.rs

| `BlsError` | enum |  |  |  |  |  |

| `aggregate_committee` | function | aggregate_committee(
    signatures: &[Signature<TinyBLS381>],
    public_keys: &[PublicKey<TinyBLS381>],
) |  |  |  |  |

| `public` | function | public(&self) |  |  |  |  |

| `sign` | function | sign(&self, context: &[u8], message: &[u8]) |  |  |  |  |

| `verify_committee` | function | verify_committee(
    context: &[u8],
    message: &[u8],
    aggregate_signature: &Signature<TinyBLS381>,
    aggregate_key: &PublicKey<TinyBLS381>,
) |  |  |  |  |

| `CommitteeKey` | struct |  |  |  |  |  |


### src/brce.rs

| `Admission` | enum |  |  |  |  |  |

| `BrceError` | enum |  |  |  |  |  |

| `Entry` | enum |  |  |  |  |  |

| `Observation` | enum |  |  |  |  |  |

| `ReconciliationVerdict` | enum |  |  |  |  |  |

| `Rule` | enum |  |  |  |  |  |

| `actuate` | function | actuate(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        actuator: &mut (impl Actuator + Observer) |  |  |  |  |

| `admit` | function | admit(
        &mut self,
        request: Request,
        route: RouteDecision,
        policy: impl Fn(&Request, &RouteDecision) |  |  |  |  |

| `admitted` | function | admitted(&self) |  |  |  |  |

| `append` | function | append(&mut self, entry: Entry) |  |  |  |  |

| `compute_digest` | function | compute_digest(&self) |  |  |  |  |

| `construct` | function | construct(
        &mut self,
        admitted: &Admitted,
        consequence_id: &str,
        idempotent: bool,
    ) |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) |  |  |  |  |

| `court` | function | court(ledger: &BrceLedger, world: &dyn Observer) |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `execute` | function | execute(
        &mut self,
        action: &ConstructedAction,
        attempt_id: &str,
        actuator: &mut dyn Actuator,
        now: u64,
    ) |  |  |  |  |

| `from_entries` | function | from_entries(entries: impl IntoIterator<Item = Entry>) |  |  |  |  |

| `from_records` | function | from_records(records: Vec<LedgerRecord>) |  |  |  |  |

| `grant_digest` | function | grant_digest(&self) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `mutant_suite` | function | mutant_suite(base: &BrceLedger, world: &dyn Observer) |  |  |  |  |

| `new` | function | new(root: impl Into<PathBuf>) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `pending_consequences` | function | pending_consequences(ledger: &BrceLedger) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        now: u64,
    ) |  |  |  |  |

| `receipt` | function | receipt(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        observer: &dyn Observer,
    ) |  |  |  |  |

| `reconcile` | function | reconcile(
        &mut self,
        observer: &dyn Observer,
    ) |  |  |  |  |

| `record_digest` | function | record_digest(seq: u64, prev: &str, entry: &Entry) |  |  |  |  |

| `records` | function | records(&self) |  |  |  |  |

| `refused_rules` | function | refused_rules(&self) |  |  |  |  |

| `replay_digest` | function | replay_digest(ledger: &BrceLedger) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `seal` | function | seal(mut self) |  |  |  |  |

| `snapshot` | function | snapshot(world: &dyn Observer) |  |  |  |  |

| `ActuationEvidence` | struct |  |  |  |  |  |

| `ActuationResult` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `AuthorityGrant` | struct |  |  |  |  |  |

| `BrceLedger` | struct |  |  |  |  |  |

| `BrcePipeline` | struct |  |  |  |  |  |

| `BrceReceipt` | struct |  |  |  |  |  |

| `ConstructedAction` | struct |  |  |  |  |  |

| `CourtRefusal` | struct |  |  |  |  |  |

| `CourtVerdict` | struct |  |  |  |  |  |

| `EffectEvidence` | struct |  |  |  |  |  |

| `FileActuator` | struct |  |  |  |  |  |

| `LedgerRecord` | struct |  |  |  |  |  |

| `MutantOutcome` | struct |  |  |  |  |  |

| `ReplayIdentity` | struct |  |  |  |  |  |

| `Request` | struct |  |  |  |  |  |

| `RouteDecision` | struct |  |  |  |  |  |

| `StaticWorld` | struct |  |  |  |  |  |

| `VerificationEvidence` | struct |  |  |  |  |  |

| `Actuator` | trait |  |  |  |  |  |

| `Observer` | trait |  |  |  |  |  |


### src/canonical_jcs.rs

| `JcsError` | enum |  |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &serde_json::Value) |  |  |  |  |

| `canonical_commitment` | function | canonical_commitment(
    value: &serde_json::Value,
) |  |  |  |  |


### src/canonical_time.rs

| `CanonicalTimeError` | enum |  |  |  |  |  |

| `canonical_string` | function | canonical_string(ts: Timestamp) |  |  |  |  |

| `canonicalize` | function | canonicalize(s: &str) |  |  |  |  |

| `checked_add_ms` | function | checked_add_ms(ts: Timestamp, millis: u64) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |


### src/catalog.rs

| `format_catalog` | function | format_catalog(fixtures: &[Fixture]) |  |  |  |  |

| `list_fixtures` | function | list_fixtures(
    db: &FixtureDatabase,
    name_filter: Option<String>,
    events_filter: Option<usize>,
) |  |  |  |  |


### src/causal_graph.rs

| `CausalError` | enum |  |  |  |  |  |

| `add_dependency` | function | add_dependency(&mut self, from: K, to: K) |  |  |  |  |

| `contains` | function | contains(&self, node: K) |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `topological_order` | function | topological_order(&self) |  |  |  |  |

| `verify_acyclic` | function | verify_acyclic(&self) |  |  |  |  |

| `CausalGraph` | struct |  |  |  |  |  |


### src/chain.rs

| `ChainError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, event: OperationEvent) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `deserialize_receipt` | function | deserialize_receipt(bytes: &[u8]) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `from_events` | function | from_events(events: Vec<OperationEvent>) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `load_working` | function | load_working() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `save_receipt` | function | save_receipt(receipt: &Receipt, path: &Path) |  |  |  |  |

| `save_working` | function | save_working(events: &[OperationEvent]) |  |  |  |  |

| `serialize_receipt` | function | serialize_receipt(receipt: &Receipt) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |


### src/cli.rs

| `assemble` | function | assemble(out: Option<&str>) |  |  |  |  |

| `emit` | function | emit(
    event_type: &str,
    objects: &[String],
    payload: &str,
) |  |  |  |  |

| `show` | function | show(receipt: &str) |  |  |  |  |

| `verify` | function | verify(receipt: &str) |  |  |  |  |


### src/crypto_trust_attestation.rs

| `AttestationError` | enum |  |  |  |  |  |

| `AttestationKind` | enum |  |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `attest_key` | function | attest_key(
    record: &KeyRecord,
    kind: AttestationKind,
    device: Option<(&str, bool, bool) |  |  |  |  |

| `verify_attestation` | function | verify_attestation(
    att: &AttestationRecord,
    signer_pk: &[u8],
) |  |  |  |  |

| `AttestationRecord` | struct |  |  |  |  |  |


### src/crypto_trust_canonical.rs

| `CanonicalError` | enum |  |  |  |  |  |

| `digest` | function | digest(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `digest_hex` | function | digest_hex(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `domain_separated` | function | domain_separated(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `jcs` | function | jcs(value: &Value) |  |  |  |  |


### src/crypto_trust_crl_file.rs

| `CrlFileError` | enum |  |  |  |  |  |

| `load_and_apply` | function | load_and_apply(
        path: Option<&Path>,
        revocations: &mut RevocationList,
        issuer_pk: &[u8],
        current_epoch: u64,
        max_staleness: u64,
        now: u64,
    ) |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `publish_verified` | function | publish_verified(
        revocations: &RevocationList,
        issuer: &Es256SigningKey,
        issuer_kid: &KeyId,
        epoch: u64,
        at: u64,
        out: Option<&Path>,
    ) |  |  |  |  |

| `read` | function | read(path: Option<&Path>) |  |  |  |  |

| `write` | function | write(
        list: &SignedRevocationList,
        path: Option<&Path>,
    ) |  |  |  |  |

| `CrlFile` | struct |  |  |  |  |  |


### src/crypto_trust_doctor.rs

| `envelope_law_finding` | function | envelope_law_finding() |  |  |  |  |

| `es256_selftest_finding` | function | es256_selftest_finding() |  |  |  |  |

| `pqc_selftest_finding` | function | pqc_selftest_finding() |  |  |  |  |

| `run_crypto_checks` | function | run_crypto_checks() |  |  |  |  |

| `store_integrity_finding` | function | store_integrity_finding(path: &Path) |  |  |  |  |


### src/crypto_trust_enclave.rs

| `EnclaveError` | enum |  |  |  |  |  |

| `delete_enclave_key` | function | delete_enclave_key(label: &str) |  |  |  |  |

| `enclave_sign` | function | enclave_sign(label: &str, msg: &[u8]) |  |  |  |  |

| `enclave_verify` | function | enclave_verify(
        public_key_sec1: &[u8],
        msg: &[u8],
        sig_der: &[u8],
    ) |  |  |  |  |

| `generate_enclave_key` | function | generate_enclave_key(label: &str) |  |  |  |  |

| `EnclaveKeyRef` | struct |  |  |  |  |  |


### src/crypto_trust_envelope.rs

| `EnvelopeError` | enum |  |  |  |  |  |

| `envelope_document` | function | envelope_document(&self) |  |  |  |  |

| `from_bytes` | function | from_bytes(b: &[u8]) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) |  |  |  |  |

| `record` | function | record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_checked` | function | signing_input_checked(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `NonceJournal` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |


### src/crypto_trust_es256.rs

| `Es256Error` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) |  |  |  |  |

| `generate` | function | generate() |  |  |  |  |

| `key_id_fingerprint` | function | key_id_fingerprint(&self) |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) |  |  |  |  |

| `verify_es256` | function | verify_es256(
    public_key_sec1: &[u8],
    msg: &[u8],
    sig_der: &[u8],
) |  |  |  |  |

| `Es256SigningKey` | struct |  |  |  |  |  |


### src/crypto_trust_journal.rs

| `JournalError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, draft: JournalEntryDraft) |  |  |  |  |

| `by_epoch` | function | by_epoch(&self, epoch: u64) |  |  |  |  |

| `by_key` | function | by_key(&self, kid: &str) |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `record_receipt` | function | record_receipt(
    journal: &mut StandingJournal,
    receipt: &crate::crypto_trust_verify::CryptoStandingReceipt,
    policy_epoch: u64,
    revocation_epoch: u64,
) |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) |  |  |  |  |

| `JournalEntry` | struct |  |  |  |  |  |

| `JournalEntryDraft` | struct |  |  |  |  |  |

| `StandingJournal` | struct |  |  |  |  |  |


### src/crypto_trust_journal_persist.rs

| `PersistError` | enum |  |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `load` | function | load(path: impl Into<PathBuf>) |  |  |  |  |

| `open_or_create` | function | open_or_create(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) |  |  |  |  |

| `record` | function | record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) |  |  |  |  |

| `refresh` | function | refresh(&mut self) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) |  |  |  |  |

| `NonceLedgerFile` | struct |  |  |  |  |  |

| `NonceRecord` | struct |  |  |  |  |  |


### src/crypto_trust_jwks.rs

| `JwksError` | enum |  |  |  |  |  |

| `export_jwk` | function | export_jwk(record: &KeyRecord) |  |  |  |  |

| `export_jwks` | function | export_jwks(records: &[KeyRecord]) |  |  |  |  |


### src/crypto_trust_kat.rs

| `KatError` | enum |  |  |  |  |  |

| `build_vector` | function | build_vector(index: usize, algorithm: &str) |  |  |  |  |

| `export_json` | function | export_json(corpus: &[KatVector]) |  |  |  |  |

| `generate_corpus` | function | generate_corpus() |  |  |  |  |

| `import_json` | function | import_json(s: &str) |  |  |  |  |

| `verify_corpus` | function | verify_corpus(vectors: &[KatVector]) |  |  |  |  |

| `verify_vector` | function | verify_vector(vector: &KatVector) |  |  |  |  |

| `KatReport` | struct |  |  |  |  |  |

| `KatVector` | struct |  |  |  |  |  |


### src/crypto_trust_keys.rs

| `AlgorithmId` | enum |  |  |  |  |  |

| `CryptoProfile` | enum |  |  |  |  |  |

| `KeyOrigin` | enum |  |  |  |  |  |

| `KeyProviderKind` | enum |  |  |  |  |  |

| `PublicKeyMaterial` | enum |  |  |  |  |  |

| `RegistryError` | enum |  |  |  |  |  |

| `algorithm` | function | algorithm(&self) |  |  |  |  |

| `all` | function | all() |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(&self) |  |  |  |  |

| `encoded_len` | function | encoded_len(&self) |  |  |  |  |

| `fingerprint_public_key` | function | fingerprint_public_key(
    algorithm: AlgorithmId,
    public_key: &PublicKeyMaterial,
) |  |  |  |  |

| `from_fingerprint` | function | from_fingerprint(fingerprint: &KeyFingerprint) |  |  |  |  |

| `key_material_exportable` | function | key_material_exportable(self) |  |  |  |  |

| `kind` | function | kind(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(self) |  |  |  |  |

| `public_key_len` | function | public_key_len(self) |  |  |  |  |

| `CustodianIdentity` | struct |  |  |  |  |  |

| `InMemoryKeyRegistry` | struct |  |  |  |  |  |

| `KeyFingerprint` | struct |  |  |  |  |  |

| `KeyId` | struct |  |  |  |  |  |

| `KeyRecord` | struct |  |  |  |  |  |

| `KeyRegistry` | trait |  |  |  |  |  |


### src/crypto_trust_lifecycle.rs

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `active_at` | function | active_at(&self, now: u64) |  |  |  |  |

| `active_epoch` | function | active_epoch(&self, kid: &str) |  |  |  |  |

| `admit_opening` | function | admit_opening(&self, kid: &str, history: &[KeyEpoch]) |  |  |  |  |

| `current_epoch` | function | current_epoch(&self) |  |  |  |  |

| `epochs` | function | epochs(&self, kid: &str) |  |  |  |  |

| `is_revoked` | function | is_revoked(&self, kid: &str) |  |  |  |  |

| `new` | function | new(policy: RotationPolicy) |  |  |  |  |

| `open_epoch` | function | open_epoch(&mut self, kid: &str, at: u64) |  |  |  |  |

| `policy` | function | policy(&self) |  |  |  |  |

| `require_active_epoch` | function | require_active_epoch(&self, kid: &str, now: u64) |  |  |  |  |

| `retire_epoch` | function | retire_epoch(&mut self, kid: &str, index: u64, at: u64) |  |  |  |  |

| `revoke` | function | revoke(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `revoked_at` | function | revoked_at(&self, kid: &str) |  |  |  |  |

| `signature_epoch_live` | function | signature_epoch_live(
        &self,
        kid: &str,
        sig_revocation_epoch: u64,
        now: u64,
    ) |  |  |  |  |

| `validate_epoch` | function | validate_epoch(
        &self,
        kid: &str,
        epoch: &KeyEpoch,
        now: u64,
    ) |  |  |  |  |

| `validate_signature_key` | function | validate_signature_key(&self, kid: &str, now: u64) |  |  |  |  |

| `KeyEpoch` | struct |  |  |  |  |  |

| `LifecycleLedger` | struct |  |  |  |  |  |

| `RevocationList` | struct |  |  |  |  |  |

| `RevocationRecord` | struct |  |  |  |  |  |

| `RotationPolicy` | struct |  |  |  |  |  |


### src/crypto_trust_log.rs

| `LogOpsError` | enum |  |  |  |  |  |

| `audit` | function | audit(&self) |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) |  |  |  |  |

| `heads` | function | heads(&self) |  |  |  |  |

| `journal` | function | journal(&self) |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) |  |  |  |  |

| `log` | function | log(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&self, leaf: u64) |  |  |  |  |

| `publish_head` | function | publish_head(
        &self,
        signer: &Es256SigningKey,
        kid: &str,
        now: u64,
    ) |  |  |  |  |

| `record` | function | record(
        &mut self,
        receipt: &CryptoStandingReceipt,
        commitment: [u8; 32],
        signer: &Es256SigningKey,
        signer_kid: &str,
        now: u64,
    ) |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) |  |  |  |  |

| `verify_head` | function | verify_head(head: &SignedTreeHead, public_key_sec1: &[u8]) |  |  |  |  |

| `AuditReport` | struct |  |  |  |  |  |

| `SignedTreeHead` | struct |  |  |  |  |  |

| `TrustLogOps` | struct |  |  |  |  |  |


### src/crypto_trust_nonce_store.rs

| `NonceStoreError` | enum |  |  |  |  |  |

| `default_journal_path` | function | default_journal_path() |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open_default` | function | open_default() |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64) |  |  |  |  |

| `record` | function | record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `DiskNonceJournal` | struct |  |  |  |  |  |


### src/crypto_trust_pqc.rs

| `PqcError` | enum |  |  |  |  |  |

| `hybrid_sign` | function | hybrid_sign(secret: &HybridSecret, msg: &[u8]) |  |  |  |  |

| `hybrid_verify` | function | hybrid_verify(
    es256_pk: &[u8],
    mldsa65_pk: &[u8],
    msg: &[u8],
    sig: &HybridSignature,
) |  |  |  |  |

| `ml_dsa65_from_seed` | function | ml_dsa65_from_seed(seed: &[u8; ML_DSA_65_SEED_LEN]) |  |  |  |  |

| `ml_dsa65_generate` | function | ml_dsa65_generate() |  |  |  |  |

| `ml_dsa65_sign` | function | ml_dsa65_sign(
    seed: &[u8; ML_DSA_65_SEED_LEN],
    msg: &[u8],
    rnd: &[u8; 32],
) |  |  |  |  |

| `ml_dsa65_verify` | function | ml_dsa65_verify(public: &[u8], msg: &[u8], sig: &[u8]) |  |  |  |  |

| `slh_dsa128s_from_seed` | function | slh_dsa128s_from_seed(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) |  |  |  |  |

| `slh_dsa128s_generate` | function | slh_dsa128s_generate() |  |  |  |  |

| `slh_dsa128s_sign` | function | slh_dsa128s_sign(
    seeds: &[u8; SLH_DSA_128S_SEED_LEN],
    msg: &[u8],
) |  |  |  |  |

| `slh_dsa128s_verify` | function | slh_dsa128s_verify(public: &[u8], msg: &[u8], sig: &[u8]) |  |  |  |  |

| `HybridSecret` | struct |  |  |  |  |  |

| `HybridSignature` | struct |  |  |  |  |  |

| `MlDsa65KeyPair` | struct |  |  |  |  |  |

| `SlhDsa128sKeyPair` | struct |  |  |  |  |  |


### src/crypto_trust_provider.rs

| `ProviderRefusal` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(id: KeyId, seed: &[u8; 32]) |  |  |  |  |

| `generate` | function | generate(id: KeyId) |  |  |  |  |

| `key_record` | function | key_record(&self, custodian: CustodianIdentity, created_epoch: u64) |  |  |  |  |

| `DetachedSignature` | struct |  |  |  |  |  |

| `SoftwareEs256Provider` | struct |  |  |  |  |  |

| `SigningProvider` | trait |  |  |  |  |  |


### src/crypto_trust_quorum.rs

| `QuorumError` | enum |  |  |  |  |  |

| `allowed_algorithms` | function | allowed_algorithms(&self) |  |  |  |  |

| `new` | function | new(registry: &'a R) |  |  |  |  |

| `verify_quorum` | function | verify_quorum(
        &self,
        signing_input: &[u8],
        shares: &[SignatureShare],
        k: usize,
    ) |  |  |  |  |

| `with_allowed_algorithms` | function | with_allowed_algorithms(
        mut self,
        algs: impl IntoIterator<Item = AlgorithmId>,
    ) |  |  |  |  |

| `QuorumEngine` | struct |  |  |  |  |  |

| `QuorumVerdict` | struct |  |  |  |  |  |

| `SignatureShare` | struct |  |  |  |  |  |


### src/crypto_trust_revocation.rs

| `RevocationPubError` | enum |  |  |  |  |  |

| `apply_to` | function | apply_to(
    revocations: &mut RevocationList,
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
    current_epoch: u64,
    max_staleness: u64,
    now: u64,
) |  |  |  |  |

| `publish` | function | publish(
    revocations: &RevocationList,
    issuer: &Es256SigningKey,
    issuer_kid: &KeyId,
    epoch: u64,
    at: u64,
) |  |  |  |  |

| `verify_publication` | function | verify_publication(
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
) |  |  |  |  |

| `RevocationRecordMirror` | struct |  |  |  |  |  |

| `SignedRevocationList` | struct |  |  |  |  |  |


### src/crypto_trust_rotation.rs

| `RotationError` | enum |  |  |  |  |  |

| `admission_allowed` | function | admission_allowed(from: CryptoProfile, to: CryptoProfile) |  |  |  |  |

| `assert_not_downgrade` | function | assert_not_downgrade(from: CryptoProfile, to: CryptoProfile) |  |  |  |  |

| `assert_not_downgrade_under` | function | assert_not_downgrade_under(
    from: CryptoProfile,
    to: CryptoProfile,
    policy: &MigrationPolicy,
) |  |  |  |  |

| `rotate_es256_to_hybrid` | function | rotate_es256_to_hybrid(
        old: &Es256SigningKey,
        hybrid_secret: &HybridSecret,
        at: u64,
    ) |  |  |  |  |

| `verify_rotation` | function | verify_rotation(
        record: &RotationRecord,
        hybrid_pk_es256: &[u8],
        hybrid_pk_mldsa65: &[u8],
    ) |  |  |  |  |

| `MigrationPolicy` | struct |  |  |  |  |  |

| `RotationCeremony` | struct |  |  |  |  |  |

| `RotationRecord` | struct |  |  |  |  |  |


### src/crypto_trust_rotation_store.rs

| `RotationStoreError` | enum |  |  |  |  |  |

| `append` | function | append(
        &self,
        record: &RotationRecord,
        successor_public_es256: &[u8],
        successor_public_mldsa65: &[u8],
    ) |  |  |  |  |

| `latest_for_predecessor` | function | latest_for_predecessor(
        &self,
        kid: &str,
    ) |  |  |  |  |

| `latest_for_successor` | function | latest_for_successor(
        &self,
        kid: &str,
    ) |  |  |  |  |

| `load` | function | load(&self) |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `RotationStore` | struct |  |  |  |  |  |

| `RotationStoreEnvelope` | struct |  |  |  |  |  |

| `VerifiedRotation` | struct |  |  |  |  |  |


### src/crypto_trust_sa2a.rs

| `Sa2aWireError` | enum |  |  |  |  |  |

| `approval_to_envelope` | function | approval_to_envelope(a: &Sa2aApproval) |  |  |  |  |

| `envelope_sa2a_signing_input` | function | envelope_sa2a_signing_input(
    env: &SignatureEnvelope,
    principal: &str,
) |  |  |  |  |

| `envelope_to_approval` | function | envelope_to_approval(
    env: &SignatureEnvelope,
    principal: &str,
) |  |  |  |  |

| `sa2a_signing_input` | function | sa2a_signing_input(a: &Sa2aApproval) |  |  |  |  |

| `sa2a_signing_input_checked` | function | sa2a_signing_input_checked(a: &Sa2aApproval) |  |  |  |  |

| `Sa2aApproval` | struct |  |  |  |  |  |


### src/crypto_trust_seal.rs

| `SealError` | enum |  |  |  |  |  |

| `seal_receipt` | function | seal_receipt(
    receipt: &Receipt,
    envelope: SignatureEnvelope,
    signature: Vec<u8>,
) |  |  |  |  |

| `subject_digest_of` | function | subject_digest_of(receipt: &Receipt) |  |  |  |  |

| `verify_sealed` | function | verify_sealed(
    sealed: &SealedReceipt,
    engine: &VerificationEngine,
) |  |  |  |  |

| `SealedReceipt` | struct |  |  |  |  |  |


### src/crypto_trust_store.rs

| `StoreError` | enum |  |  |  |  |  |

| `checksum_for` | function | checksum_for(records: &[KeyRecord]) |  |  |  |  |

| `load` | function | load(&self) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `records` | function | records(&self) |  |  |  |  |

| `register_checked` | function | register_checked(&mut self, record: KeyRecord) |  |  |  |  |

| `FileKeyStore` | struct |  |  |  |  |  |

| `KeyStoreFile` | struct |  |  |  |  |  |


### src/crypto_trust_transparency.rs

| `TransparencyError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, commitment: [u8; 32]) |  |  |  |  |

| `consistency_proof` | function | consistency_proof(&self, first: u64) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `inclusion_proof` | function | inclusion_proof(&self, idx: u64) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `verify_consistency` | function | verify_consistency(
    proof: &ConsistencyProof,
    first_head: &[u8; 32],
    second_head: &[u8; 32],
) |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(leaf: &[u8; 32], proof: &InclusionProof, head: &[u8; 32]) |  |  |  |  |

| `ConsistencyProof` | struct |  |  |  |  |  |

| `InclusionProof` | struct |  |  |  |  |  |

| `TransparencyLog` | struct |  |  |  |  |  |


### src/crypto_trust_verify.rs

| `CryptographicStanding` | enum |  |  |  |  |  |

| `StandingReceiptError` | enum |  |  |  |  |  |

| `VerifyRefusal` | enum |  |  |  |  |  |

| `all` | function | all() |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `certify_signed` | function | certify_signed(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
        subject: &str,
        signing: &Es256SigningKey,
    ) |  |  |  |  |

| `from_graph_defaults` | function | from_graph_defaults() |  |  |  |  |

| `new` | function | new(
        registry: InMemoryKeyRegistry,
        revocations: RevocationList,
        nonces: NonceJournal,
        policy: TrustPolicy,
    ) |  |  |  |  |

| `nonce_seen_at` | function | nonce_seen_at(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `policy` | function | policy(&self) |  |  |  |  |

| `prune_nonces` | function | prune_nonces(&mut self) |  |  |  |  |

| `register_key` | function | register_key(&mut self, record: KeyRecord) |  |  |  |  |

| `registry` | function | registry(&self) |  |  |  |  |

| `revocations` | function | revocations(&self) |  |  |  |  |

| `revoke_key` | function | revoke_key(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `verify_envelope` | function | verify_envelope(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
    ) |  |  |  |  |

| `with_now` | function | with_now(mut self, now: u64) |  |  |  |  |

| `CryptoStandingReceipt` | struct |  |  |  |  |  |

| `CryptographicVerdict` | struct |  |  |  |  |  |

| `TrustPolicy` | struct |  |  |  |  |  |

| `VerificationEngine` | struct |  |  |  |  |  |


### src/crypto_trust_witness.rs

| `WitnessError` | enum |  |  |  |  |  |

| `collect` | function | collect(
        &self,
        witnesses: &[(&str, &Es256SigningKey) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `new` | function | new(head: SignedTreeHead) |  |  |  |  |

| `preimage` | function | preimage(&self) |  |  |  |  |

| `verify_cosigned` | function | verify_cosigned(
    head: &SignedTreeHead,
    cosigs: &[WitnessSignature],
    min: usize,
    pks: &[(&str, &[u8]) |  |  |  |  |

| `CosignVerdict` | struct |  |  |  |  |  |

| `WitnessCosign` | struct |  |  |  |  |  |

| `WitnessSignature` | struct |  |  |  |  |  |


### src/dfcm.rs

| `ClosureGap` | enum |  |  |  |  |  |

| `DfcmRefusal` | enum |  |  |  |  |  |

| `EvidenceKind` | enum |  |  |  |  |  |

| `certify_dfcm` | function | certify_dfcm(
    mut release_profile: DfcmProfile,
    mut observations: Vec<SubjectObservation>,
) |  |  |  |  |

| `v26_9_18_profile` | function | v26_9_18_profile(s: V26_9_18Subjects) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `DfcmProfile` | struct |  |  |  |  |  |

| `DfcmReceipt` | struct |  |  |  |  |  |

| `EvidenceKey` | struct |  |  |  |  |  |

| `EvidenceWitness` | struct |  |  |  |  |  |

| `ExactSubject` | struct |  |  |  |  |  |

| `MinimalFrontier` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `ObligationEvaluation` | struct |  |  |  |  |  |

| `PathEvaluation` | struct |  |  |  |  |  |

| `ProofPath` | struct |  |  |  |  |  |

| `SubjectObservation` | struct |  |  |  |  |  |

| `SubjectRequirement` | struct |  |  |  |  |  |

| `V26_9_18Subjects` | struct |  |  |  |  |  |


### src/diag.rs

| `ErrorCode` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `exit_code` | function | exit_code(self) |  |  |  |  |

| `from_error` | function | from_error(code: ErrorCode, err: &dyn std::error::Error) |  |  |  |  |

| `hint` | function | hint(self) |  |  |  |  |

| `message` | function | message(self) |  |  |  |  |

| `new` | function | new(code: ErrorCode, message: impl Into<String>) |  |  |  |  |

| `with_hint` | function | with_hint(mut self, hint: impl Into<String>) |  |  |  |  |

| `with_span` | function | with_span(mut self, file: impl Into<String>, line: Option<u32>) |  |  |  |  |

| `Diag` | struct |  |  |  |  |  |

| `Span` | struct |  |  |  |  |  |


### src/diff.rs

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `DiffEntry` | struct |  |  |  |  |  |

| `DiffResult` | struct |  |  |  |  |  |

| `ModifiedEntry` | struct |  |  |  |  |  |


### src/discovery.rs

| `conformance_metrics` | function | conformance_metrics(receipt: &Receipt) |  |  |  |  |

| `discover_dfg_summary` | function | discover_dfg_summary(receipt: &Receipt) |  |  |  |  |

| `discover_from_admitted` | function | discover_from_admitted(admitted: &crate::types::AdmittedReceipt) |  |  |  |  |

| `discover_process_tree` | function | discover_process_tree(receipt: &Receipt) |  |  |  |  |

| `project_to_event_log` | function | project_to_event_log(receipt: &Receipt) |  |  |  |  |

| `quality_metrics` | function | quality_metrics(receipt: &Receipt) |  |  |  |  |

| `quality_metrics_from_admitted` | function | quality_metrics_from_admitted(admitted: &crate::types::AdmittedReceipt) |  |  |  |  |


### src/doctor_check.rs

| `FindingStatus` | enum |  |  |  |  |  |

| `auto_fixable` | function | auto_fixable(mut self) |  |  |  |  |

| `fail` | function | fail(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `ok` | function | ok(id: &'static str, message: impl Into<String>) |  |  |  |  |

| `run_all` | function | run_all() |  |  |  |  |

| `warn` | function | warn(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) |  |  |  |  |

| `Finding` | struct |  |  |  |  |  |

| `DoctorCheck` | trait |  |  |  |  |  |


### src/ecosystem.rs

| `EcosystemRefusal` | enum |  |  |  |  |  |

| `EcosystemRole` | enum |  |  |  |  |  |

| `certify_ecosystem` | function | certify_ecosystem(
    admitted: &AdmittedReceipt,
    mut observation: EcosystemObservation,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `EcosystemMember` | struct |  |  |  |  |  |

| `EcosystemObservation` | struct |  |  |  |  |  |

| `EcosystemReceipt` | struct |  |  |  |  |  |

| `RoleCoverage` | struct |  |  |  |  |  |

| `RoleRequirement` | struct |  |  |  |  |  |


### src/ed25519_witness.rs

| `Ed25519WitnessError` | enum |  |  |  |  |  |

| `generate` | function | generate() |  |  |  |  |

| `public` | function | public(&self) |  |  |  |  |

| `sign` | function | sign(&self, message: &[u8]) |  |  |  |  |

| `verify_witness` | function | verify_witness(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) |  |  |  |  |

| `WitnessKeyPair` | struct |  |  |  |  |  |


### src/errc.rs

| `ErrcQuadrant` | enum |  |  |  |  |  |

| `ErrcRefusal` | enum |  |  |  |  |  |

| `certify_errc` | function | certify_errc(
    admitted: &AdmittedReceipt,
    mut observation: ErrcObservation,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `ErrcClaim` | struct |  |  |  |  |  |

| `ErrcMeasure` | struct |  |  |  |  |  |

| `ErrcObservation` | struct |  |  |  |  |  |

| `ErrcReceipt` | struct |  |  |  |  |  |

| `ErrcSource` | struct |  |  |  |  |  |

| `PreservedInvariant` | struct |  |  |  |  |  |

| `QuadrantCounts` | struct |  |  |  |  |  |


### src/errc_claim_assurance.rs

| `ErrcClaimAssuranceRefusal` | enum |  |  |  |  |  |

| `certify_errc_claim_assurance` | function | certify_errc_claim_assurance(
    parent: &ErrcReceipt,
    mut witnesses: Vec<ErrcClaimWitness>,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `verify_against` | function | verify_against(&self, parent: &ErrcReceipt) |  |  |  |  |

| `ErrcClaimAssuranceReceipt` | struct |  |  |  |  |  |

| `ErrcClaimWitness` | struct |  |  |  |  |  |


### src/error.rs

| `AffidavitError` | enum |  |  |  |  |  |

| `ChainError` | enum |  |  |  |  |  |

| `MiningError` | enum |  |  |  |  |  |

| `OcelError` | enum |  |  |  |  |  |

| `PqcError` | enum |  |  |  |  |  |

| `PredictionError` | enum |  |  |  |  |  |

| `ShardingError` | enum |  |  |  |  |  |

| `SloViolation` | enum |  |  |  |  |  |


### src/event_builder.rs

| `build` | function | build(self, counter: &mut SeqCounter) |  |  |  |  |

| `new` | function | new(event_type: impl Into<String>) |  |  |  |  |

| `object` | function | object(mut self, id: impl Into<String>, object_type: impl Into<String>) |  |  |  |  |

| `payload` | function | payload(mut self, payload: impl Into<Vec<u8>>) |  |  |  |  |

| `payload_str` | function | payload_str(mut self, payload: impl Into<String>) |  |  |  |  |

| `qualified_object` | function | qualified_object(
        mut self,
        id: impl Into<String>,
        object_type: impl Into<String>,
        qualifier: impl Into<String>,
    ) |  |  |  |  |

| `EventBuilder` | struct |  |  |  |  |  |


### src/execution_manifest.rs

| `ManifestRefusal` | enum |  |  |  |  |  |

| `binding` | function | binding(&self) |  |  |  |  |

| `digest` | function | digest(&self) |  |  |  |  |

| `requalification_reason` | function | requalification_reason(
    before: &ExecutionManifest,
    after: &ExecutionManifest,
) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `verify_binding` | function | verify_binding(
    manifest: &ExecutionManifest,
    binding: &ExecutionBinding,
) |  |  |  |  |

| `ExecutionBinding` | struct |  |  |  |  |  |

| `ExecutionManifest` | struct |  |  |  |  |  |


### src/federation.rs

| `accepted` | function | accepted(&self) |  |  |  |  |

| `cli_standing_authority` | function | cli_standing_authority(scope: &str) |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(receipt: &str, observation: &str) |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: &str) |  |  |  |  |

| `errc_assure` | function | errc_assure(parent: &str, witnesses: &str) |  |  |  |  |

| `errc_certify` | function | errc_certify(receipt: &str, observation: &str) |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: &str) |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(receipt: &str, parent: Option<&str>) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |

| `standing_certify` | function | standing_certify(receipt: &str, observation: &str, scope: &str) |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: &str) |  |  |  |  |

| `CourtOutcome` | struct |  |  |  |  |  |


### src/fixture_db.rs

| `all` | function | all(&self) |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `reindex` | function | reindex(&mut self) |  |  |  |  |

| `save` | function | save(&self) |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) |  |  |  |  |

| `Fixture` | struct |  |  |  |  |  |

| `FixtureDatabase` | struct |  |  |  |  |  |

| `FixtureQuery` | struct |  |  |  |  |  |


### src/gall.rs

| `CrownRefusal` | enum |  |  |  |  |  |

| `CrownStanding` | enum |  |  |  |  |  |

| `GateStatus` | enum |  |  |  |  |  |

| `certify_gall_crown` | function | certify_gall_crown(manifest: &CrownManifest) |  |  |  |  |

| `CrownManifest` | struct |  |  |  |  |  |

| `CrownReceipt` | struct |  |  |  |  |  |

| `GateWitness` | struct |  |  |  |  |  |

| `PredecessorWitness` | struct |  |  |  |  |  |


### src/generation.rs

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) |  |  |  |  |

| `from_json` | function | from_json(json: &str) |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, 
        pattern_name: &str, 
        events: Vec<serde_json::Value>, 
        expected_verdict: &str,
        expected_failure_stage: Option<&str>
    ) |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) |  |  |  |  |

| `main` | function | main() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `CodegenEngine` | struct |  |  |  |  |  |

| `Snippet` | struct |  |  |  |  |  |

| `SnippetRegistry` | struct |  |  |  |  |  |


### src/handlers.rs

| `CheckStatus` | enum |  |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect(
    receipts_path: String,
    sensitivity: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble` | function | assemble(out: Option<String>, format: Option<String>) |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize(
    notary_provider: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature(
    signing_method: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `attest` | function | attest(
    receipt: String,
    attestation_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `audit` | function | audit() |  |  |  |  |

| `bus_factor` | function | bus_factor(receipts_path: String, format: Option<String>) |  |  |  |  |

| `catalog` | function | catalog(filter_name: Option<String>, filter_events: Option<usize>) |  |  |  |  |

| `causality_chain` | function | causality_chain(
    start_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `conformance` | function | conformance(receipt: String) |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix(
    receipts_path: String,
    output_matrix: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `diagnose` | function | diagnose(receipt: String) |  |  |  |  |

| `diff` | function | diff(receipt_a: String, receipt_b: String, format: Option<String>) |  |  |  |  |

| `doctor` | function | doctor(receipts: Option<String>, fix: bool) |  |  |  |  |

| `dora_metrics` | function | dora_metrics(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `emit` | function | emit(r#type: String, object: String, payload: String, format: Option<String>) |  |  |  |  |

| `emit_batch` | function | emit_batch(batch_file: String, format: Option<String>) |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd(provider: String, job_status: String, format: Option<String>) |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud(
    provider: String,
    resource_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_github` | function | emit_from_github(repo: String, event_type: String, format: Option<String>) |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab(repo: String, event_type: String, format: Option<String>) |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring(
    provider: String,
    alert_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_quality` | function | emit_from_quality(working_dir: Option<String>, format: Option<String>) |  |  |  |  |

| `emit_from_security` | function | emit_from_security(
    provider: String,
    vuln_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_ocel_quality_measurement` | function | emit_ocel_quality_measurement(
    working_dir: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `emit_ocel_quality_violation` | function | emit_ocel_quality_violation(
    working_dir: Option<String>,
    baseline_commits: Option<u32>,
    format: Option<String>,
    rules: Option<String>,
) |  |  |  |  |

| `emit_violation_causal_chain` | function | emit_violation_causal_chain(
    receipt_path: String,
    metric_filter: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `envelope_export` | function | envelope_export(sealed_file: String, format: Option<String>) |  |  |  |  |

| `envelope_list` | function | envelope_list(store: Option<String>) |  |  |  |  |

| `envelope_sign` | function | envelope_sign(receipt: String, key_file: String, out: Option<String>) |  |  |  |  |

| `envelope_verify` | function | envelope_verify(
    sealed_file: String,
    store: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_assure` | function | errc_assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_certify` | function | errc_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply(file: String, store: Option<String>) |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish(
    kid: String,
    epoch: u64,
    store: Option<String>,
    out: Option<String>,
) |  |  |  |  |

| `evidence_heads` | function | evidence_heads(journal_file: Option<String>) |  |  |  |  |

| `evidence_journal` | function | evidence_journal(subject: String, out: Option<String>) |  |  |  |  |

| `explain_incident` | function | explain_incident(
    incident_desc: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius(
    change_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `fix_receipt` | function | fix_receipt(
    receipt: String,
    action: Option<String>,
    dry_run: bool,
    format: Option<String>,
) |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `graph` | function | graph(receipt: String, format: Option<String>) |  |  |  |  |

| `guide_search` | function | guide_search(keyword: String, format: Option<String>) |  |  |  |  |

| `hipaa` | function | hipaa(receipts_path: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `inspect` | function | inspect(receipt: String, format: Option<String>) |  |  |  |  |

| `install_git_hook` | function | install_git_hook(threshold: Option<String>) |  |  |  |  |

| `keys_generate` | function | keys_generate(algorithm: String, custodian: String, out: Option<String>) |  |  |  |  |

| `keys_import` | function | keys_import(
    algorithm: String,
    public_key_hex: String,
    custodian: String,
    out: Option<String>,
) |  |  |  |  |

| `keys_list` | function | keys_list(store: Option<String>) |  |  |  |  |

| `keys_revoke` | function | keys_revoke(kid: String, reason: String, store: Option<String>) |  |  |  |  |

| `keys_rotate` | function | keys_rotate(kid: String, store: Option<String>, out: Option<String>) |  |  |  |  |

| `license_compliance` | function | license_compliance(
    receipts_path: String,
    license_policy: String,
    format: Option<String>,
) |  |  |  |  |

| `model` | function | model(receipt: String) |  |  |  |  |

| `monitor` | function | monitor(
    watch: Option<String>,
    _metrics: Option<String>,
    _rules: Option<String>,
    baseline_commits: Option<u32>,
    interval: Option<u64>,
    output: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `notarize` | function | notarize(receipt: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `orphaned_code` | function | orphaned_code(
    receipts_path: String,
    days: Option<u32>,
    format: Option<String>,
) |  |  |  |  |

| `pci_dss` | function | pci_dss(receipts_path: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `policy_enforce` | function | policy_enforce(
    receipts_path: String,
    policy_file: String,
    format: Option<String>,
) |  |  |  |  |

| `portfolio_health` | function | portfolio_health(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `predict` | function | predict(
    receipts_path: String,
    prediction_type: String,
    _model: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `profile` | function | profile(receipt: Option<String>, duration: Option<u64>) |  |  |  |  |

| `query` | function | query(q: String, receipts_path: String, format: Option<String>) |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput(iterations: Option<u32>) |  |  |  |  |

| `replay` | function | replay(receipt: String) |  |  |  |  |

| `root_cause` | function | root_cause(
    effect_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_attest` | function | sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius(
    sbom_path: String,
    component: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance(
    sbom_path: String,
    framework: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_emit` | function | sbom_emit(sbom_path: String, format: Option<String>) |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia(sbom_path: String, format: Option<String>) |  |  |  |  |

| `sbom_scan` | function | sbom_scan(
    sbom_path: String,
    advisories_path: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `search` | function | search(pattern: String, receipts_path: String, format: Option<String>) |  |  |  |  |

| `security_debt` | function | security_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `send_violation_webhook` | function | send_violation_webhook(
    violation: &crate::quality::QualityViolation,
    webhook_url: &str,
) |  |  |  |  |

| `show` | function | show(receipt: String, format: Option<String>) |  |  |  |  |

| `sign` | function | sign(
    receipt: String,
    key_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `soc2_audit` | function | soc2_audit(
    receipts_path: String,
    soc2_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_certify` | function | standing_certify(
    receipt: String,
    observation: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `stats` | function | stats(receipt: String, format: Option<String>) |  |  |  |  |

| `team_velocity` | function | team_velocity(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `tech_debt` | function | tech_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `test` | function | test() |  |  |  |  |

| `timeline` | function | timeline(
    receipts_path: String,
    start_time: Option<String>,
    end_time: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `trend_analysis` | function | trend_analysis(
    receipts_path: String,
    metric: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `variance` | function | variance(receipt: Option<String>, iterations: Option<u32>) |  |  |  |  |

| `verify` | function | verify(
    receipt: String,
    format: Option<String>,
    _profile: Option<String>,
    _strict: Option<bool>,
) |  |  |  |  |

| `verify_compliance` | function | verify_compliance(receipt: String, framework: String, format: Option<String>) |  |  |  |  |

| `verify_family` | function | verify_family(receipts_dir: String, format: Option<String>) |  |  |  |  |

| `verify_sla` | function | verify_sla(receipt: String, sla_file: String, format: Option<String>) |  |  |  |  |

| `visualize` | function | visualize(format: String, receipt: String) |  |  |  |  |

| `why` | function | why(receipt: String, format: Option<String>) |  |  |  |  |

| `DoctorFinding` | struct |  |  |  |  |  |

| `RevocationSidecarEntry` | struct |  |  |  |  |  |


### src/hlc.rs

| `HlcError` | enum |  |  |  |  |  |

| `concurrent` | function | concurrent(a: HlcTimestamp, b: HlcTimestamp) |  |  |  |  |

| `happened_before` | function | happened_before(a: HlcTimestamp, b: HlcTimestamp) |  |  |  |  |

| `peek` | function | peek(&self) |  |  |  |  |

| `receive` | function | receive(&mut self, received: HlcTimestamp) |  |  |  |  |

| `send` | function | send(&mut self) |  |  |  |  |

| `with_skew_bound` | function | with_skew_bound(max_skew_ms: u64) |  |  |  |  |

| `HlcClock` | struct |  |  |  |  |  |

| `HlcTimestamp` | struct |  |  |  |  |  |


### src/lib.rs

| `run` | function | run() |  |  |  |  |


### src/lsp/diagnostics.rs

| `verdict_to_diagnostics` | function | verdict_to_diagnostics(verdict: &crate::types::Verdict) |  |  |  |  |


### src/lsp/goto_definition.rs

| `goto_definition_for_event_type` | function | goto_definition_for_event_type(event_type: &str) |  |  |  |  |


### src/lsp/hover.rs

| `hover_for_event_id` | function | hover_for_event_id(event_id: &str, receipt: &Receipt) |  |  |  |  |


### src/metrics.rs

| `SloViolation` | enum |  |  |  |  |  |

| `check_slo` | function | check_slo(&self) |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `new_noop` | function | new_noop() |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) |  |  |  |  |

| `MetricsCollector` | struct |  |  |  |  |  |

| `PrometheusExporter` | struct |  |  |  |  |  |

| `ServiceLevelIndicators` | struct |  |  |  |  |  |


### src/mining.rs

| `MiningError` | enum |  |  |  |  |  |

| `alignment_fitness_score` | function | alignment_fitness_score(
    admitted: &AdmittedReceipt,
    model: &PetriNet,
) |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `interpret_score` | function | interpret_score(fitness: f64) |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `AlignmentReport` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |


### src/model_mining.rs

| `MiningError` | enum |  |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |


### src/mutate.rs

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### src/mutation.rs

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### src/ocel.rs

| `build_event` | function | build_event(
    event_type: impl Into<String>,
    objects: Vec<ObjectRef>,
    payload: &[u8],
    counter: &mut SeqCounter,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `next_seq` | function | next_seq(&mut self) |  |  |  |  |

| `object_ref` | function | object_ref(id: impl Into<String>, obj_type: impl Into<String>) |  |  |  |  |

| `parse_object_ref` | function | parse_object_ref(spec: &str) |  |  |  |  |

| `peek` | function | peek(&self) |  |  |  |  |

| `qualified_object_ref` | function | qualified_object_ref(
    id: impl Into<String>,
    obj_type: impl Into<String>,
    qualifier: impl Into<String>,
) |  |  |  |  |

| `starting_at` | function | starting_at(value: u64) |  |  |  |  |

| `validate_event` | function | validate_event(event: &OperationEvent) |  |  |  |  |

| `SeqCounter` | struct |  |  |  |  |  |


### src/output.rs

| `Format` | enum |  |  |  |  |  |

| `diag` | function | diag(&mut self, diag: &crate::diag::Diag) |  |  |  |  |

| `from_str` | function | from_str(s: &str) |  |  |  |  |

| `info` | function | info(&mut self, msg: &str) |  |  |  |  |

| `json` | function | json(&mut self, value: &serde_json::Value) |  |  |  |  |

| `line` | function | line(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `new` | function | new(format: Format) |  |  |  |  |

| `no_color` | function | no_color(mut self) |  |  |  |  |

| `print_` | function | print_(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `quiet` | function | quiet(mut self) |  |  |  |  |

| `verbose` | function | verbose(mut self) |  |  |  |  |

| `warn` | function | warn(&mut self, msg: &str) |  |  |  |  |

| `with_format` | function | with_format(format: Format) |  |  |  |  |

| `with_sinks` | function | with_sinks(
        format: Format,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) |  |  |  |  |

| `Out` | struct |  |  |  |  |  |


### src/policy_cedar.rs

| `GateVerdict` | enum |  |  |  |  |  |

| `PolicyError` | enum |  |  |  |  |  |

| `evaluate` | function | evaluate(
        &self,
        principal: &str,
        action: &str,
        resource: &str,
    ) |  |  |  |  |

| `from_policies` | function | from_policies(policies: &str) |  |  |  |  |

| `PolicyGate` | struct |  |  |  |  |  |


### src/portable_protocol.rs

| `verify_foreign_receipt` | function | verify_foreign_receipt(
    consequence_digest: &str,
    authority: ForeignAuthority<'_>,
    observation: ForeignObservation<'_>,
    receipt: ForeignReceipt<'_>,
) |  |  |  |  |

| `ForeignAuthority` | struct |  |  |  |  |  |

| `ForeignObservation` | struct |  |  |  |  |  |

| `ForeignReceipt` | struct |  |  |  |  |  |


### src/predict_maximalist.rs

| `PredictionError` | enum |  |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model(
    model: &AdmittedReceipt,
    current_trace: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |


### src/quality.rs

| `Notification` | enum |  |  |  |  |  |

| `QualityViolation` | enum |  |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `another_good` | function | another_good() |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `feature1` | function | feature1() |  |  |  |  |

| `feature2` | function | feature2() |  |  |  |  |

| `feature3` | function | feature3() |  |  |  |  |

| `good_feature` | function | good_feature() |  |  |  |  |

| `measure_code_quality` | function | measure_code_quality(src_path: &str) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64, window_size: usize) |  |  |  |  |

| `run_watch_loop` | function | run_watch_loop(&mut self) |  |  |  |  |

| `run_watch_loop_async` | function | run_watch_loop_async(
        path: &str,
        interval_secs: u64,
    ) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `CodeQualityMetrics` | struct |  |  |  |  |  |

| `FileWatcher` | struct |  |  |  |  |  |

| `WesternElectricAnalyzer` | struct |  |  |  |  |  |


### src/quality_correlation.rs

| `amplify_severity_for_correlated_violations` | function | amplify_severity_for_correlated_violations(
    violations: &[QualityViolation],
    correlations: &[MetricCorrelation],
) |  |  |  |  |

| `analyze_correlations` | function | analyze_correlations(
    history: &[CodeQualityMetrics],
    violations: &[QualityViolation],
) |  |  |  |  |

| `compute_metric_correlations` | function | compute_metric_correlations(history: &[CodeQualityMetrics]) |  |  |  |  |

| `detect_simultaneous_violations` | function | detect_simultaneous_violations(
    violations: &[QualityViolation],
) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `infer_root_cause` | function | infer_root_cause(
    metrics: &[CodeQualityMetrics],
    violation: &QualityViolation,
) |  |  |  |  |

| `is_actionable` | function | is_actionable(&self) |  |  |  |  |

| `is_compound` | function | is_compound(&self) |  |  |  |  |

| `is_significant` | function | is_significant(&self) |  |  |  |  |

| `max_severity` | function | max_severity(&self) |  |  |  |  |

| `new` | function | new(
        metric_names: Vec<String>,
        severities: Vec<String>,
        timestamps: Vec<u64>,
        time_window_secs: u64,
    ) |  |  |  |  |

| `strength` | function | strength(&self) |  |  |  |  |

| `CorrelationAnalysis` | struct |  |  |  |  |  |

| `MetricCorrelation` | struct |  |  |  |  |  |

| `RootCauseHypothesis` | struct |  |  |  |  |  |

| `SimultaneousViolation` | struct |  |  |  |  |  |


### src/quality_extended.rs

| `RuleVariant` | enum |  |  |  |  |  |

| `add_custom_threshold` | function | add_custom_threshold(mut self, rule_name: String, threshold: f64) |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `compute` | function | compute(violations: &[QualityViolation]) |  |  |  |  |

| `compute_aggregate_severity` | function | compute_aggregate_severity(violations: &[QualityViolation]) |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `detect_all_rule_variants` | function | detect_all_rule_variants(
    metrics: &[f64],
    config: &WesternElectricConfig,
) |  |  |  |  |

| `detect_rule_storms` | function | detect_rule_storms(violations: &[QualityViolation]) |  |  |  |  |

| `finalize` | function | finalize(&mut self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `with_enabled_rules` | function | with_enabled_rules(
        mut self,
        rule1: bool,
        rule2: bool,
        rule3: bool,
        rule4: bool,
        rule5: bool,
        rule6: bool,
        rule7: bool,
    ) |  |  |  |  |

| `with_sigmas` | function | with_sigmas(mut self, primary: f64, secondary: f64, tertiary: f64) |  |  |  |  |

| `AggregatedSeverity` | struct |  |  |  |  |  |

| `EnhancedWesternElectricAnalyzer` | struct |  |  |  |  |  |

| `RuleStorm` | struct |  |  |  |  |  |

| `WesternElectricConfig` | struct |  |  |  |  |  |


### src/quality_object_level.rs

| `ObjectViolation` | enum |  |  |  |  |  |

| `aggregate_module_metrics` | function | aggregate_module_metrics(files: &[FileQualityMetrics]) |  |  |  |  |

| `complex_fn` | function | complex_fn(x: i32) |  |  |  |  |

| `compute_package_health` | function | compute_package_health(modules: &[ModuleQualityMetrics]) |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `detect_object_level_violations` | function | detect_object_level_violations(
    object_metric: &FileQualityMetrics,
    baseline: f64,
    stddev: f64,
) |  |  |  |  |

| `documented` | function | documented() |  |  |  |  |

| `good_fn` | function | good_fn() |  |  |  |  |

| `maintainability_index` | function | maintainability_index(&self) |  |  |  |  |

| `measure_file_quality` | function | measure_file_quality(path: &str) |  |  |  |  |

| `new` | function | new(path: String) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `typed_fn` | function | typed_fn(x: i32, y: String) |  |  |  |  |

| `undocumented` | function | undocumented() |  |  |  |  |

| `untyped_fn` | function | untyped_fn() |  |  |  |  |

| `update_health_score` | function | update_health_score(&mut self) |  |  |  |  |

| `FileQualityMetrics` | struct |  |  |  |  |  |

| `ModuleQualityMetrics` | struct |  |  |  |  |  |

| `PackageHealthScore` | struct |  |  |  |  |  |


### src/quality_ocel.rs

| `add_event` | function | add_event(&mut self, event: OcelQualityEvent) |  |  |  |  |

| `build_causal_chain` | function | build_causal_chain(
    violation_event: &OcelQualityEvent,
    event_log: &[OcelQualityEvent],
) |  |  |  |  |

| `correlate_violations_across_objects` | function | correlate_violations_across_objects(
    log: &OcelQualityLog,
) |  |  |  |  |

| `events_by_type` | function | events_by_type(&self, quality_event_type: &str) |  |  |  |  |

| `from_metrics` | function | from_metrics(
        object_id: impl Into<String>,
        object_type: impl Into<String>,
        metrics: &CodeQualityMetrics,
    ) |  |  |  |  |

| `measure_to_ocel_event` | function | measure_to_ocel_event(
    event_id: &str,
    seq: u64,
    metrics: &CodeQualityMetrics,
    objects: &[ObjectRef],
) |  |  |  |  |

| `measurements` | function | measurements(&self) |  |  |  |  |

| `new` | function | new(log_id: impl Into<String>, timestamp: u64) |  |  |  |  |

| `violation_to_ocel_event` | function | violation_to_ocel_event(
    event_id: &str,
    seq: u64,
    violation: &QualityViolation,
    triggered_by_event_id: &str,
    objects: &[ObjectRef],
) |  |  |  |  |

| `violations` | function | violations(&self) |  |  |  |  |

| `ObjectCorrelation` | struct |  |  |  |  |  |

| `ObjectQualityRecord` | struct |  |  |  |  |  |

| `OcelQualityEvent` | struct |  |  |  |  |  |

| `OcelQualityLog` | struct |  |  |  |  |  |

| `ViolationCausalChain` | struct |  |  |  |  |  |


### src/quantized_payoff.rs

| `DimensionError` | enum |  |  |  |  |  |

| `MatrixError` | enum |  |  |  |  |  |

| `QuantizationRefusal` | enum |  |  |  |  |  |

| `bits` | function | bits(&self) |  |  |  |  |

| `from_fractions` | function | from_fractions(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<(u64, u64) |  |  |  |  |

| `from_scores` | function | from_scores(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<f64>],
    ) |  |  |  |  |

| `lenses` | function | lenses(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `quantize` | function | quantize(score: f64) |  |  |  |  |

| `quantize_fraction` | function | quantize_fraction(num: u64, den: u64) |  |  |  |  |

| `zeroed` | function | zeroed(n_nodes: usize, lenses: usize) |  |  |  |  |

| `PayoffMatrix` | struct |  |  |  |  |  |


### src/receipts_certified.rs

| `build_canonical_subject` | function | build_canonical_subject(payload_hash_hex: &str, subject: &str) |  |  |  |  |

| `certify_paid_delivery_payload` | function | certify_paid_delivery_payload(
    payload_hash_hex: &str,
    subject: &str,
    signing: &Es256SigningKey,
) |  |  |  |  |

| `verify_certified_paid_delivery` | function | verify_certified_paid_delivery(
    certified: &CertifiedReceiptEnvelope,
    payload_hash_hex: &str,
    subject: &str,
) |  |  |  |  |

| `CertifiedReceiptEnvelope` | struct |  |  |  |  |  |


### src/registry.rs

| `VerbGroup` | enum |  |  |  |  |  |

| `by_group` | function | by_group(group: VerbGroup) |  |  |  |  |

| `description` | function | description(self) |  |  |  |  |

| `did_you_mean` | function | did_you_mean(input: &str) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `lookup` | function | lookup(verb: &str, noun: &str) |  |  |  |  |

| `new` | function | new(
        verb: &'static str,
        noun: &'static str,
        group: VerbGroup,
        summary: &'static str,
        keywords: &'static [&'static str],
    ) |  |  |  |  |

| `search` | function | search(query: &str) |  |  |  |  |

| `verb_count` | function | verb_count() |  |  |  |  |

| `with_example` | function | with_example(mut self, example: &'static str) |  |  |  |  |

| `VerbEntry` | struct |  |  |  |  |  |


### src/replay_filter.rs

| `ReplayFilterError` | enum |  |  |  |  |  |

| `capacity` | function | capacity(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(capacity: usize) |  |  |  |  |

| `probably_seen` | function | probably_seen(&self, id: &[u8; 32]) |  |  |  |  |

| `retract` | function | retract(&mut self, id: &[u8; 32]) |  |  |  |  |

| `witness` | function | witness(&mut self, id: &[u8; 32]) |  |  |  |  |

| `ReplayFilter` | struct |  |  |  |  |  |


### src/sbom.rs

| `ComponentType` | enum |  |  |  |  |  |

| `SbomError` | enum |  |  |  |  |  |

| `SbomFormat` | enum |  |  |  |  |  |

| `canonicalize` | function | canonicalize(&mut self) |  |  |  |  |

| `component` | function | component(&self, bom_ref: &str) |  |  |  |  |

| `content_address` | function | content_address(&self) |  |  |  |  |

| `detect_format` | function | detect_format(doc: &serde_json::Value) |  |  |  |  |

| `expr` | function | expr(expression: impl Into<String>) |  |  |  |  |

| `family` | function | family(&self) |  |  |  |  |

| `has_unique_identifier` | function | has_unique_identifier(&self) |  |  |  |  |

| `id` | function | id(spdx_id: impl Into<String>) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `library` | function | library(
        bom_ref: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) |  |  |  |  |

| `license_labels` | function | license_labels(&self) |  |  |  |  |

| `missing` | function | missing(&self) |  |  |  |  |

| `new` | function | new(algorithm: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `ntia_minimum_elements` | function | ntia_minimum_elements(&self) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |

| `parse_cyclonedx` | function | parse_cyclonedx(doc: &serde_json::Value) |  |  |  |  |

| `parse_sbom_json` | function | parse_sbom_json(json: &str) |  |  |  |  |

| `parse_spdx` | function | parse_spdx(doc: &serde_json::Value) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, root: &str) |  |  |  |  |

| `Component` | struct |  |  |  |  |  |

| `Dependency` | struct |  |  |  |  |  |

| `Hash` | struct |  |  |  |  |  |

| `License` | struct |  |  |  |  |  |

| `NtiaMinimumElements` | struct |  |  |  |  |  |

| `Sbom` | struct |  |  |  |  |  |

| `SbomMetadata` | struct |  |  |  |  |  |

| `Supplier` | struct |  |  |  |  |  |

| `Tool` | struct |  |  |  |  |  |


### src/sbom_artifacts.rs

| `bundle` | function | bundle(&self) |  |  |  |  |

| `cleanup` | function | cleanup(mut self) |  |  |  |  |

| `create_bundle` | function | create_bundle(
        &self,
        sbom_id: &str,
        sbom_format: &str,
        description: Option<String>,
    ) |  |  |  |  |

| `load_bundle` | function | load_bundle(&self, path: PathBuf) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `redact_sensitive` | function | redact_sensitive(&self, value: &str) |  |  |  |  |

| `save_bundle` | function | save_bundle(&self, bundle: &SbomForensicsBundle, path: PathBuf) |  |  |  |  |

| `take_bundle` | function | take_bundle(mut self) |  |  |  |  |

| `with_path` | function | with_path(mut self, path: PathBuf) |  |  |  |  |

| `with_redaction` | function | with_redaction(mut self, redact: bool) |  |  |  |  |

| `AttestationData` | struct |  |  |  |  |  |

| `BundleMetadata` | struct |  |  |  |  |  |

| `ComplianceRecord` | struct |  |  |  |  |  |

| `ComponentNode` | struct |  |  |  |  |  |

| `RiskPropagationRecord` | struct |  |  |  |  |  |

| `SbomArtifactCollector` | struct |  |  |  |  |  |

| `SbomArtifactGuard` | struct |  |  |  |  |  |

| `SbomForensicsBundle` | struct |  |  |  |  |  |

| `SupplyChainGraph` | struct |  |  |  |  |  |

| `VulnerabilityRecord` | struct |  |  |  |  |  |


### src/sbom_compliance.rs

| `ComplianceError` | enum |  |  |  |  |  |

| `Framework` | enum |  |  |  |  |  |

| `SlsaLevel` | enum |  |  |  |  |  |

| `assess_all` | function | assess_all(sbom: &Sbom) |  |  |  |  |

| `assess_slsa` | function | assess_slsa(sbom: &Sbom) |  |  |  |  |

| `check_cisa` | function | check_cisa(sbom: &Sbom) |  |  |  |  |

| `check_cscrm` | function | check_cscrm(sbom: &Sbom) |  |  |  |  |

| `check_eo_14028` | function | check_eo_14028(sbom: &Sbom) |  |  |  |  |

| `check_in_toto` | function | check_in_toto(sbom: &Sbom) |  |  |  |  |

| `check_iso_27001` | function | check_iso_27001(sbom: &Sbom) |  |  |  |  |

| `check_ntia` | function | check_ntia(sbom: &Sbom) |  |  |  |  |

| `check_slsa` | function | check_slsa(sbom: &Sbom) |  |  |  |  |

| `check_soc2` | function | check_soc2(sbom: &Sbom) |  |  |  |  |

| `display_name` | function | display_name(&self) |  |  |  |  |

| `rank` | function | rank(&self) |  |  |  |  |

| `requirement_count` | function | requirement_count(&self) |  |  |  |  |

| `score` | function | score(&self) |  |  |  |  |

| `supported_frameworks` | function | supported_frameworks() |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `vex_readiness` | function | vex_readiness(sbom: &Sbom) |  |  |  |  |

| `ComplianceResult` | struct |  |  |  |  |  |


### src/sbom_ocel.rs

| `SbomOcelError` | enum |  |  |  |  |  |

| `build_sbom_causal_chain` | function | build_sbom_causal_chain(
    events: &[SbomOcelEvent],
    sbom: &Sbom,
    root_bom_ref: &str,
) |  |  |  |  |

| `component_object_type` | function | component_object_type(component: &Component) |  |  |  |  |

| `correlate_components_by_license` | function | correlate_components_by_license(
    events: &[SbomOcelEvent],
    sbom: &Sbom,
) |  |  |  |  |

| `license_object_type` | function | license_object_type(label: &str) |  |  |  |  |

| `sbom_to_ocel_events` | function | sbom_to_ocel_events(
    sbom: &Sbom,
    counter: &mut SeqCounter,
) |  |  |  |  |

| `supplier_object_type` | function | supplier_object_type(name: &str) |  |  |  |  |

| `supported_event_types` | function | supported_event_types() |  |  |  |  |

| `vulnerability_object_type` | function | vulnerability_object_type(id: &str) |  |  |  |  |

| `ObjectCorrelation` | struct |  |  |  |  |  |

| `SbomCausalChain` | struct |  |  |  |  |  |

| `SbomOcelEvent` | struct |  |  |  |  |  |


### src/sbom_supply_chain.rs

| `SupplyChainError` | enum |  |  |  |  |  |

| `attest_provenance` | function | attest_provenance(sbom: &Sbom, receipt_ref: Option<&str>) |  |  |  |  |

| `blast_radius` | function | blast_radius(
    graph: &DependencyGraph,
    bom_ref: &str,
) |  |  |  |  |

| `build_report` | function | build_report(sbom: &Sbom, spof_threshold: usize) |  |  |  |  |

| `contains` | function | contains(&self, bom_ref: &str) |  |  |  |  |

| `depth` | function | depth(&self, root: &str) |  |  |  |  |

| `direct_dependencies` | function | direct_dependencies(&self, bom_ref: &str) |  |  |  |  |

| `direct_dependents` | function | direct_dependents(&self, bom_ref: &str) |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `from_sbom` | function | from_sbom(sbom: &Sbom) |  |  |  |  |

| `is_cyclic` | function | is_cyclic(&self) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `single_points_of_failure` | function | single_points_of_failure(
    graph: &DependencyGraph,
    _sbom: &Sbom,
    threshold: usize,
) |  |  |  |  |

| `supplier_concentration` | function | supplier_concentration(sbom: &Sbom) |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, bom_ref: &str) |  |  |  |  |

| `transitive_dependents` | function | transitive_dependents(&self, bom_ref: &str) |  |  |  |  |

| `BlastRadius` | struct |  |  |  |  |  |

| `DependencyGraph` | struct |  |  |  |  |  |

| `ProvenanceAttestation` | struct |  |  |  |  |  |

| `SupplierConcentration` | struct |  |  |  |  |  |

| `SupplyChainReport` | struct |  |  |  |  |  |


### src/sbom_vulnerability.rs

| `Severity` | enum |  |  |  |  |  |

| `VexStatus` | enum |  |  |  |  |  |

| `VulnerabilityError` | enum |  |  |  |  |  |

| `apply_vex` | function | apply_vex(matches: &[VulnerabilityMatch], vex: &[VexStatement]) |  |  |  |  |

| `build_report` | function | build_report(
    sbom: &Sbom,
    vulns: &[Vulnerability],
    vex: &[VexStatement],
) |  |  |  |  |

| `cvss_band` | function | cvss_band(score: f64) |  |  |  |  |

| `from_cvss` | function | from_cvss(score: f64) |  |  |  |  |

| `from_score` | function | from_score(base_score: f64) |  |  |  |  |

| `match_vulnerabilities` | function | match_vulnerabilities(sbom: &Sbom, vulns: &[Vulnerability]) |  |  |  |  |

| `new` | function | new(id: impl Into<String>, base_score: f64) |  |  |  |  |

| `propagate_risk` | function | propagate_risk(
    sbom: &Sbom,
    matches: &[VulnerabilityMatch],
) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `suppresses` | function | suppresses(&self) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `with_vector` | function | with_vector(base_score: f64, vector: impl Into<String>) |  |  |  |  |

| `CvssVector` | struct |  |  |  |  |  |

| `VexStatement` | struct |  |  |  |  |  |

| `Vulnerability` | struct |  |  |  |  |  |

| `VulnerabilityMatch` | struct |  |  |  |  |  |

| `VulnerabilityReport` | struct |  |  |  |  |  |


### src/secp256k1_witness.rs

| `Secp256k1WitnessError` | enum |  |  |  |  |  |

| `WitnessSigningError` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) |  |  |  |  |

| `verify_bip340` | function | verify_bip340(
    x_only_public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) |  |  |  |  |

| `verify_bip340_raw` | function | verify_bip340_raw(
    x_only_public_key: &[u8; 32],
    message: &[u8; 32],
    signature: &[u8; 64],
) |  |  |  |  |

| `verify_ecdsa` | function | verify_ecdsa(
    compressed_sec1_public_key: &[u8; 33],
    message: &[u8],
    signature: &[u8],
) |  |  |  |  |

| `WitnessSigningKey` | struct |  |  |  |  |  |


### src/seq_bitmap.rs

| `SeqBitmapError` | enum |  |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `has` | function | has(&self, seq: u32) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `record` | function | record(&mut self, seq: u32) |  |  |  |  |

| `verify_contiguous` | function | verify_contiguous(&self) |  |  |  |  |

| `SeqContiguityCertifier` | struct |  |  |  |  |  |


### src/smt.rs

| `SmtError` | enum |  |  |  |  |  |

| `common_prefix_len` | function | common_prefix_len(
    a: &BitSlice<u8, bitvec::prelude::Msb0>,
    b: &BitSlice<u8, bitvec::prelude::Msb0>,
) |  |  |  |  |

| `get` | function | get(&mut self, key: &StateKey) |  |  |  |  |

| `insert` | function | insert(&mut self, key: &StateKey, value: &StateValue) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prove_absence` | function | prove_absence(&mut self, key: &StateKey) |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&mut self, key: &StateKey) |  |  |  |  |

| `remove` | function | remove(&mut self, key: &StateKey) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `verify_absence` | function | verify_absence(
    root: &StateRoot,
    neighbor_value: &StateValue,
    witness: &AbsenceWitness,
) |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(
    root: &StateRoot,
    expected_value: &StateValue,
    proof: &InclusionProof,
) |  |  |  |  |

| `AbsenceWitness` | struct |  |  |  |  |  |

| `InclusionProof` | struct |  |  |  |  |  |

| `StateTree` | struct |  |  |  |  |  |


### src/standing.rs

| `Standing` | enum |  |  |  |  |  |

| `StandingRefusal` | enum |  |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `AuthorityBinding` | struct |  |  |  |  |  |

| `ExecutionEvidence` | struct |  |  |  |  |  |

| `ReplayEvidence` | struct |  |  |  |  |  |

| `StandingObservation` | struct |  |  |  |  |  |

| `StandingReceipt` | struct |  |  |  |  |  |

| `SubjectIdentity` | struct |  |  |  |  |  |

| `VerificationEvidence` | struct |  |  |  |  |  |


### src/threshold_quorum.rs

| `QuorumError` | enum |  |  |  |  |  |

| `aggregate_signature` | function | aggregate_signature(
    package: &SigningPackage,
    shares: &[(ParticipantId, frost::round2::SignatureShare) |  |  |  |  |

| `round2_sign` | function | round2_sign(
    secret_share: &SecretShare,
    nonces: &SigningNonces,
    package: &SigningPackage,
) |  |  |  |  |

| `signing_package` | function | signing_package(
    commitments: &[(ParticipantId, frost::round1::SigningCommitments) |  |  |  |  |

| `verify_quorum` | function | verify_quorum(
    group_public_key: &GroupPublicKey,
    message: &[u8],
    signature_bytes: &[u8; 64],
) |  |  |  |  |

| `DealtQuorum` | struct |  |  |  |  |  |

| `GroupPublicKey` | struct |  |  |  |  |  |

| `ParticipantId` | struct |  |  |  |  |  |


### src/tracing.rs

| `captured_spans` | function | captured_spans() |  |  |  |  |

| `clear_spans` | function | clear_spans() |  |  |  |  |

| `SpanRecord` | struct |  |  |  |  |  |


### src/types.rs

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `AffidavitReceiptChain` | struct |  |  |  |  |  |

| `AssembleOutput` | struct |  |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `EmitOutput` | struct |  |  |  |  |  |

| `EventSummary` | struct |  |  |  |  |  |

| `InspectionReport` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `QualityMeasurement` | struct |  |  |  |  |  |

| `QualityMetricValue` | struct |  |  |  |  |  |

| `QualityViolationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `StatsOutput` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |


### src/verbs/anomaly_detect.rs

| `anomaly_detect` | function | anomaly_detect(
    receipts_path: String,
    sensitivity: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/assemble.rs

| `assemble` | function | assemble(
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/assemble_and_notarize.rs

| `assemble_and_notarize` | function | assemble_and_notarize(
    notary_provider: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/assemble_with_signature.rs

| `assemble_with_signature` | function | assemble_with_signature(
    signing_method: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/assure.rs

| `assure` | function | assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/attest.rs

| `attest` | function | attest(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/audit.rs

| `audit` | function | audit(
) |  |  |  |  |


### src/verbs/bus_factor.rs

| `bus_factor` | function | bus_factor(
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/catalog.rs

| `catalog` | function | catalog(
    filter_name: Option<String>,
    filter_events: Option<usize>,
) |  |  |  |  |


### src/verbs/causality_chain.rs

| `causality_chain` | function | causality_chain(
    start_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/conformance.rs

| `conformance` | function | conformance(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/coverage_analysis.rs

| `coverage_analysis` | function | coverage_analysis(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/dependency_matrix.rs

| `dependency_matrix` | function | dependency_matrix(
    receipts_path: String,
    output_matrix: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/diagnose.rs

| `diagnose` | function | diagnose(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/diff.rs

| `diff` | function | diff(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/doctor.rs

| `doctor` | function | doctor(
    receipts: Option<String>,
    fix: bool,
) |  |  |  |  |


### src/verbs/dora_metrics.rs

| `dora_metrics` | function | dora_metrics(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/ecosystem_certify.rs

| `ecosystem_certify` | function | ecosystem_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/ecosystem_verify.rs

| `ecosystem_verify` | function | ecosystem_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit.rs

| `emit` | function | emit(
    #[arg(alias = "type") |  |  |  |  |


### src/verbs/emit_batch.rs

| `emit_batch` | function | emit_batch(
    batch_file: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_cicd.rs

| `emit_from_cicd` | function | emit_from_cicd(
    provider: String,
    job_status: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_cloud.rs

| `emit_from_cloud` | function | emit_from_cloud(
    provider: String,
    resource_type: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_github.rs

| `emit_from_github` | function | emit_from_github(
    repo: String,
    event_type: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_gitlab.rs

| `emit_from_gitlab` | function | emit_from_gitlab(
    repo: String,
    event_type: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_monitoring.rs

| `emit_from_monitoring` | function | emit_from_monitoring(
    provider: String,
    alert_type: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_sbom.rs

| `emit_from_sbom` | function | emit_from_sbom(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/emit_from_security.rs

| `emit_from_security` | function | emit_from_security(
    provider: String,
    vuln_type: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/envelope_export.rs

| `envelope_export` | function | envelope_export(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/envelope_list.rs

| `envelope_list` | function | envelope_list(
    store: Option<String>,
) |  |  |  |  |


### src/verbs/envelope_sign.rs

| `envelope_sign` | function | envelope_sign(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/envelope_verify.rs

| `envelope_verify` | function | envelope_verify(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/errc_assure.rs

| `errc_assure` | function | errc_assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/errc_certify.rs

| `errc_certify` | function | errc_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/errc_verify.rs

| `errc_verify` | function | errc_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/errc_verify_assurance.rs

| `errc_verify_assurance` | function | errc_verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/evidence_crl_apply.rs

| `evidence_crl_apply` | function | evidence_crl_apply(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/evidence_crl_publish.rs

| `evidence_crl_publish` | function | evidence_crl_publish(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/evidence_heads.rs

| `evidence_heads` | function | evidence_heads(
    journal_file: Option<String>,
) |  |  |  |  |


### src/verbs/evidence_journal.rs

| `evidence_journal` | function | evidence_journal(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/explain_incident.rs

| `explain_incident` | function | explain_incident(
    incident_desc: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/find_blast_radius.rs

| `find_blast_radius` | function | find_blast_radius(
    change_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/fix.rs

| `fix` | function | fix(
    receipt: String,
    action: Option<String>,
    dry_run: Option<bool>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/fix_receipt.rs

| `fix_receipt` | function | fix_receipt(
    receipt: String,
    action: Option<String>,
    dry_run: bool,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/gdpr_proof.rs

| `gdpr_proof` | function | gdpr_proof(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/graph.rs

| `graph` | function | graph(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/guide_search.rs

| `guide_search` | function | guide_search(
    keyword: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/hipaa.rs

| `hipaa` | function | hipaa(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/inspect.rs

| `inspect` | function | inspect(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/install_git_hook.rs

| `install_git_hook` | function | install_git_hook(
    threshold: Option<String>,
) |  |  |  |  |


### src/verbs/keys_generate.rs

| `keys_generate` | function | keys_generate(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/keys_import.rs

| `keys_import` | function | keys_import(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/keys_list.rs

| `keys_list` | function | keys_list(
    store: Option<String>,
) |  |  |  |  |


### src/verbs/keys_revoke.rs

| `keys_revoke` | function | keys_revoke(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/keys_rotate.rs

| `keys_rotate` | function | keys_rotate(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/license_compliance.rs

| `license_compliance` | function | license_compliance(
    receipts_path: String,
    license_policy: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/model.rs

| `model` | function | model(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/monitor.rs

| `monitor` | function | monitor(
    watch: Option<String>,
    metrics: Option<String>,
    rules: Option<String>,
    baseline_commits: Option<u32>,
    interval: Option<u64>,
    output: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/notarize.rs

| `notarize` | function | notarize(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/orphaned_code.rs

| `orphaned_code` | function | orphaned_code(
    receipts_path: String,
    days: Option<u32>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/pci_dss.rs

| `pci_dss` | function | pci_dss(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/policy_enforce.rs

| `policy_enforce` | function | policy_enforce(
    receipts_path: String,
    policy_file: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/portfolio_health.rs

| `portfolio_health` | function | portfolio_health(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/predict.rs

| `predict` | function | predict(
    receipts_path: String,
    prediction_type: String,
    model: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/profile.rs

| `profile` | function | profile(
    receipt: Option<String>,
    duration: Option<u64>,
) |  |  |  |  |


### src/verbs/query.rs

| `query` | function | query(
    q: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/receipt_throughput.rs

| `receipt_throughput` | function | receipt_throughput(
    iterations: Option<u32>,
) |  |  |  |  |


### src/verbs/replay.rs

| `replay` | function | replay(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/root_cause.rs

| `root_cause` | function | root_cause(
    effect_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_attest.rs

| `sbom_attest` | function | sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_blast_radius.rs

| `sbom_blast_radius` | function | sbom_blast_radius(
    sbom_path: String,
    component: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_compliance.rs

| `sbom_compliance` | function | sbom_compliance(
    sbom_path: String,
    framework: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_emit.rs

| `sbom_emit` | function | sbom_emit(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_ntia.rs

| `sbom_ntia` | function | sbom_ntia(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/sbom_scan.rs

| `sbom_scan` | function | sbom_scan(
    sbom_path: String,
    advisories_path: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/search.rs

| `search` | function | search(
    pattern: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/security_debt.rs

| `security_debt` | function | security_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/show.rs

| `show` | function | show(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/sign.rs

| `sign` | function | sign(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/soc2_audit.rs

| `soc2_audit` | function | soc2_audit(
    receipts_path: String,
    soc2_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/standing_certify.rs

| `standing_certify` | function | standing_certify(
    receipt: String,
    observation: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/standing_verify.rs

| `standing_verify` | function | standing_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/stats.rs

| `stats` | function | stats(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/team_velocity.rs

| `team_velocity` | function | team_velocity(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/tech_debt.rs

| `tech_debt` | function | tech_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/test.rs

| `test` | function | test(
) |  |  |  |  |


### src/verbs/timeline.rs

| `timeline` | function | timeline(
    receipts_path: String,
    start_time: Option<String>,
    end_time: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/trend_analysis.rs

| `trend_analysis` | function | trend_analysis(
    receipts_path: String,
    metric: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/variance.rs

| `variance` | function | variance(
    receipt: Option<String>,
    iterations: Option<u32>,
) |  |  |  |  |


### src/verbs/verify.rs

| `verify` | function | verify(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/verify_assurance.rs

| `verify_assurance` | function | verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/verify_compliance.rs

| `verify_compliance` | function | verify_compliance(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/verify_family.rs

| `verify_family` | function | verify_family(
    receipts_dir: String,
    format: Option<String>,
) |  |  |  |  |


### src/verbs/verify_sla.rs

| `verify_sla` | function | verify_sla(
    #[arg(index = 1) |  |  |  |  |


### src/verbs/visualize.rs

| `visualize` | function | visualize(
    format: String,
    #[arg(index = 1) |  |  |  |  |


### src/verbs/why.rs

| `why` | function | why(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |


### src/verifier.rs

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |


### src/visualize.rs

| `build_graph` | function | build_graph(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |


### src/wasm_court.rs

| `WasmCourtError` | enum |  |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `run_bounded` | function | run_bounded(
        &self,
        wasm_bytes: &[u8],
        export: &str,
        fuel_budget: u64,
    ) |  |  |  |  |

| `WasmCourt` | struct |  |  |  |  |  |


### src/zero_copy.rs

| `ArchiveError` | enum |  |  |  |  |  |

| `freeze` | function | freeze(archive: &ReceiptArchive) |  |  |  |  |

| `thaw` | function | thaw(bytes: &[u8]) |  |  |  |  |

| `unfreeze` | function | unfreeze(bytes: &[u8]) |  |  |  |  |

| `validate_archived_range` | function | validate_archived_range(
    bytes: &[u8],
    min_seq: u64,
    max_seq: u64,
) |  |  |  |  |

| `ReceiptArchive` | struct |  |  |  |  |  |

| `SeqEntry` | struct |  |  |  |  |  |


### src/zk_range.rs

| `RangeProofError` | enum |  |  |  |  |  |

| `verify_range` | function | verify_range(
    commitment: &[u8; 32],
    proof_bytes: &[u8],
    bits: usize,
    label: &'static [u8],
) |  |  |  |  |

| `RangeWitness` | struct |  |  |  |  |  |


### stubs/clnrm-core/src/lib.rs

| `generate_digest` | function | generate_digest(data: &[u8]) |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |


### stubs/clnrm-core/src/lib.rs

| `generate_digest` | function | generate_digest(data: &[u8]) |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |


### stubs/wasm4pm-compat/src/lib.rs

| `ArcDirection` | enum |  |  |  |  |  |

| `ArcDirectionConst` | enum |  |  |  |  |  |

| `BpmnEvent` | enum |  |  |  |  |  |

| `BpmnGateway` | enum |  |  |  |  |  |

| `BpmnNodeKind` | enum |  |  |  |  |  |

| `BpmnRefusal` | enum |  |  |  |  |  |

| `CausalConsistency` | enum |  |  |  |  |  |

| `CausalNetRefusal` | enum |  |  |  |  |  |

| `CompatDiagnostic` | enum |  |  |  |  |  |

| `ComplianceKind` | enum |  |  |  |  |  |

| `ConformanceRefusal` | enum |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `CorrelationSchema` | enum |  |  |  |  |  |

| `CubeDimensionKind` | enum |  |  |  |  |  |

| `DeclareRefusal` | enum |  |  |  |  |  |

| `DeclareScope` | enum |  |  |  |  |  |

| `DeclareTemplate` | enum |  |  |  |  |  |

| `DfgRefusal` | enum |  |  |  |  |  |

| `DiagnosticKind` | enum |  |  |  |  |  |

| `DiagnosticSeverity` | enum |  |  |  |  |  |

| `EventLogRefusal` | enum |  |  |  |  |  |

| `EventPredicateKind` | enum |  |  |  |  |  |

| `EvidenceMode` | enum |  |  |  |  |  |

| `FilterShape` | enum |  |  |  |  |  |

| `FormatKind` | enum |  |  |  |  |  |

| `InstanceCreationKind` | enum |  |  |  |  |  |

| `InteropRefusal` | enum |  |  |  |  |  |

| `KernelRefusal` | enum |  |  |  |  |  |

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `LossFunction` | enum |  |  |  |  |  |

| `LossPolicy` | enum |  |  |  |  |  |

| `LossRefusal` | enum |  |  |  |  |  |

| `OCELAttributeValue` | enum |  |  |  |  |  |

| `ObjectCentricity` | enum |  |  |  |  |  |

| `ObjectLifecyclePhase` | enum |  |  |  |  |  |

| `ObjectPredicateKind` | enum |  |  |  |  |  |

| `ObjectTypeCardinality` | enum |  |  |  |  |  |

| `OcDeclareRefusal` | enum |  |  |  |  |  |

| `OcelAttributeValue` | enum |  |  |  |  |  |

| `OcelRefusal` | enum |  |  |  |  |  |

| `OcpqRefusal` | enum |  |  |  |  |  |

| `OcpqScopeKind` | enum |  |  |  |  |  |

| `PerspectiveRefusal` | enum |  |  |  |  |  |

| `PetriNetRefusal` | enum |  |  |  |  |  |

| `PetriRefusal` | enum |  |  |  |  |  |

| `Pm4pyShape` | enum |  |  |  |  |  |

| `Powl8Op` | enum |  |  |  |  |  |

| `Powl8OpError` | enum |  |  |  |  |  |

| `PowlNodeKind` | enum |  |  |  |  |  |

| `PowlProjectionState` | enum |  |  |  |  |  |

| `PowlRefusal` | enum |  |  |  |  |  |

| `PredicateKind` | enum |  |  |  |  |  |

| `PredictionHorizon` | enum |  |  |  |  |  |

| `PredictionRefusal` | enum |  |  |  |  |  |

| `PredictionTarget` | enum |  |  |  |  |  |

| `ProcessPerspective` | enum |  |  |  |  |  |

| `ProcessShapeKind` | enum |  |  |  |  |  |

| `ProcessTreeNode` | enum |  |  |  |  |  |

| `ProcessTreeOperator` | enum |  |  |  |  |  |

| `ProcessTreeRefusal` | enum |  |  |  |  |  |

| `QualityDimension` | enum |  |  |  |  |  |

| `QualityMetricKind` | enum |  |  |  |  |  |

| `ReceiptRefusal` | enum |  |  |  |  |  |

| `ReceiptVerdict` | enum |  |  |  |  |  |

| `RelationLaw` | enum |  |  |  |  |  |

| `RelationPredicateKind` | enum |  |  |  |  |  |

| `ReplayHintKind` | enum |  |  |  |  |  |

| `SoundnessState` | enum |  |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum |  |  |  |  |  |

| `SummaryShape` | enum |  |  |  |  |  |

| `TemporalOrder` | enum |  |  |  |  |  |

| `TemporalRefusal` | enum |  |  |  |  |  |

| `TemporalRelation` | enum |  |  |  |  |  |

| `WitnessFamily` | enum |  |  |  |  |  |

| `WorkflowPattern` | enum |  |  |  |  |  |

| `activity` | function | activity(&self) |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) |  |  |  |  |

| `admits` | function | admits(&self, count: usize) |  |  |  |  |

| `arcs` | function | arcs(&self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `as_f64` | function | as_f64(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) |  |  |  |  |

| `attribute` | function | attribute(&self) |  |  |  |  |

| `attributes` | function | attributes(&self) |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) |  |  |  |  |

| `by` | function | by(mut self, resource: &str) |  |  |  |  |

| `case_id` | function | case_id(&self) |  |  |  |  |

| `category` | function | category(&self) |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) |  |  |  |  |

| `claim_sound` | function | claim_sound(self) |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) |  |  |  |  |

| `default` | function | default() |  |  |  |  |

| `den` | function | den(&self) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) |  |  |  |  |

| `edges` | function | edges(&self) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) |  |  |  |  |

| `event_count` | function | event_count(&self) |  |  |  |  |

| `event_id` | function | event_id(&self) |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) |  |  |  |  |

| `event_set` | function | event_set(&self) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `expression` | function | expression(&self) |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) |  |  |  |  |

| `final_marking` | function | final_marking(&self) |  |  |  |  |

| `float` | function | float(key: &str, value: f64) |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) |  |  |  |  |

| `frequency` | function | frequency(&self) |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) |  |  |  |  |

| `from_owned` | function | from_owned(s: String) |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) |  |  |  |  |

| `get` | function | get(&self) |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) |  |  |  |  |

| `inner` | function | inner(&self) |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) |  |  |  |  |

| `into_admitted` | function | into_admitted(self) |  |  |  |  |

| `into_evidence` | function | into_evidence(self) |  |  |  |  |

| `into_exportable` | function | into_exportable(self) |  |  |  |  |

| `into_inner` | function | into_inner(self) |  |  |  |  |

| `into_lost` | function | into_lost(self) |  |  |  |  |

| `into_parsed` | function | into_parsed(self) |  |  |  |  |

| `into_projected` | function | into_projected(self) |  |  |  |  |

| `into_reason` | function | into_reason(self) |  |  |  |  |

| `into_receipted` | function | into_receipted(self) |  |  |  |  |

| `is_chain` | function | is_chain(&self) |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) |  |  |  |  |

| `is_negative` | function | is_negative(&self) |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) |  |  |  |  |

| `is_silent` | function | is_silent(&self) |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `kind` | function | kind(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `lanes` | function | lanes(&self) |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `length` | function | length(&self) |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `members` | function | members(&self) |  |  |  |  |

| `min` | function | min(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `net` | function | net(&self) |  |  |  |  |

| `new` | function | new(value: T) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `node_ids` | function | node_ids(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `num` | function | num(&self) |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) |  |  |  |  |

| `object_changes` | function | object_changes(&self) |  |  |  |  |

| `object_id` | function | object_id(&self) |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) |  |  |  |  |

| `object_set` | function | object_set(&self) |  |  |  |  |

| `object_type` | function | object_type(&self) |  |  |  |  |

| `object_types` | function | object_types(&self) |  |  |  |  |

| `objects` | function | objects(&self) |  |  |  |  |

| `place_id` | function | place_id(&self) |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) |  |  |  |  |

| `places` | function | places(&self) |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) |  |  |  |  |

| `process` | function | process(&self) |  |  |  |  |

| `projection` | function | projection(&self) |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) |  |  |  |  |

| `qualifier` | function | qualifier(&self) |  |  |  |  |

| `raw` | function | raw(value: T) |  |  |  |  |

| `resource` | function | resource(&self) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `scope` | function | scope(&self) |  |  |  |  |

| `silent` | function | silent(id: &str) |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) |  |  |  |  |

| `source` | function | source(&self) |  |  |  |  |

| `source_id` | function | source_id(&self) |  |  |  |  |

| `steps` | function | steps(&self) |  |  |  |  |

| `string` | function | string(key: &str, value: &str) |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) |  |  |  |  |

| `summary` | function | summary(&self, category: &str) |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `target` | function | target(&self) |  |  |  |  |

| `target_id` | function | target_id(&self) |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) |  |  |  |  |

| `tip` | function | tip(&self) |  |  |  |  |

| `tokens` | function | tokens(&self) |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) |  |  |  |  |

| `trace_count` | function | trace_count(&self) |  |  |  |  |

| `traces` | function | traces(&self) |  |  |  |  |

| `transition_id` | function | transition_id(&self) |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) |  |  |  |  |

| `transitions` | function | transitions(&self) |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `value` | function | value(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `weight` | function | weight(&self) |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) |  |  |  |  |

| `Activity` | struct |  |  |  |  |  |

| `Admission` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Arc` | struct |  |  |  |  |  |

| `ArtifactGrounding` | struct |  |  |  |  |  |

| `Between01` | struct |  |  |  |  |  |

| `BipartiteArcConst` | struct |  |  |  |  |  |

| `BpmnEdge` | struct |  |  |  |  |  |

| `BpmnLane` | struct |  |  |  |  |  |

| `BpmnNode` | struct |  |  |  |  |  |

| `BpmnPool` | struct |  |  |  |  |  |

| `BpmnProcess` | struct |  |  |  |  |  |

| `BpmnTask` | struct |  |  |  |  |  |

| `CancellationRegion` | struct |  |  |  |  |  |

| `CausalBinding` | struct |  |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct |  |  |  |  |  |

| `CausalNet` | struct |  |  |  |  |  |

| `CausallyOrderedEvidence` | struct |  |  |  |  |  |

| `ChoiceGraph` | struct |  |  |  |  |  |

| `ConditionCell` | struct |  |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `ConformanceVerdict` | struct |  |  |  |  |  |

| `ConsistencyVerified` | struct |  |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct |  |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct |  |  |  |  |  |

| `DenseKernel` | struct |  |  |  |  |  |

| `DependencyMeasure` | struct |  |  |  |  |  |

| `Deviation` | struct |  |  |  |  |  |

| `Dfg` | struct |  |  |  |  |  |

| `DfgEdge` | struct |  |  |  |  |  |

| `DfgEdgeFull` | struct |  |  |  |  |  |

| `DfgNode` | struct |  |  |  |  |  |

| `DfgWeight` | struct |  |  |  |  |  |

| `DiagnosticReport` | struct |  |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `EventLog` | struct |  |  |  |  |  |

| `EventObjectLink` | struct |  |  |  |  |  |

| `EventStream` | struct |  |  |  |  |  |

| `EventTypeName` | struct |  |  |  |  |  |

| `Evidence` | struct |  |  |  |  |  |

| `Exportable` | struct |  |  |  |  |  |

| `F1` | struct |  |  |  |  |  |

| `Fitness` | struct |  |  |  |  |  |

| `Generalization` | struct |  |  |  |  |  |

| `InitialFinalMarkingPair` | struct |  |  |  |  |  |

| `InputBinding` | struct |  |  |  |  |  |

| `LifecycleEvent` | struct |  |  |  |  |  |

| `LossChain` | struct |  |  |  |  |  |

| `LossReport` | struct |  |  |  |  |  |

| `Marking` | struct |  |  |  |  |  |

| `Metric` | struct |  |  |  |  |  |

| `MultiPerspectiveEvidence` | struct |  |  |  |  |  |

| `MultiPerspectiveLog` | struct |  |  |  |  |  |

| `MultipleInstanceSpec` | struct |  |  |  |  |  |

| `MultipleInstanceSpecConst` | struct |  |  |  |  |  |

| `NamedLoss` | struct |  |  |  |  |  |

| `OCEL` | struct |  |  |  |  |  |

| `OCELEvent` | struct |  |  |  |  |  |

| `OCELEventAttribute` | struct |  |  |  |  |  |

| `OCELObject` | struct |  |  |  |  |  |

| `OCELRelationship` | struct |  |  |  |  |  |

| `OCELType` | struct |  |  |  |  |  |

| `OCELTypeAttribute` | struct |  |  |  |  |  |

| `Object` | struct |  |  |  |  |  |

| `ObjectCentricDfg` | struct |  |  |  |  |  |

| `ObjectCentricPetriNet` | struct |  |  |  |  |  |

| `ObjectChange` | struct |  |  |  |  |  |

| `ObjectLifecycle` | struct |  |  |  |  |  |

| `ObjectObjectLink` | struct |  |  |  |  |  |

| `ObjectScope` | struct |  |  |  |  |  |

| `ObjectScopeConst` | struct |  |  |  |  |  |

| `ObjectTypeCardinality` | struct |  |  |  |  |  |

| `ObjectTypeName` | struct |  |  |  |  |  |

| `OcDeclareConstraint` | struct |  |  |  |  |  |

| `OcelAttribute` | struct |  |  |  |  |  |

| `OcelEvent` | struct |  |  |  |  |  |

| `OcelLog` | struct |  |  |  |  |  |

| `OcpqQuery` | struct |  |  |  |  |  |

| `OcpqQueryConst` | struct |  |  |  |  |  |

| `OrderEdge` | struct |  |  |  |  |  |

| `OutputBinding` | struct |  |  |  |  |  |

| `PackedKeyTable` | struct |  |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct |  |  |  |  |  |

| `PerspectiveCombination` | struct |  |  |  |  |  |

| `PetriNet` | struct |  |  |  |  |  |

| `Place` | struct |  |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct |  |  |  |  |  |

| `Powl` | struct |  |  |  |  |  |

| `PowlChoiceNode` | struct |  |  |  |  |  |

| `PowlComposition` | struct |  |  |  |  |  |

| `PowlNode` | struct |  |  |  |  |  |

| `PowlNodeId` | struct |  |  |  |  |  |

| `Precision` | struct |  |  |  |  |  |

| `Predicate` | struct |  |  |  |  |  |

| `ProcessCube` | struct |  |  |  |  |  |

| `ProcessSlice` | struct |  |  |  |  |  |

| `ProcessTree` | struct |  |  |  |  |  |

| `ProcessTreeNodeId` | struct |  |  |  |  |  |

| `Projected` | struct |  |  |  |  |  |

| `ProjectionName` | struct |  |  |  |  |  |

| `ProjectionNameOwned` | struct |  |  |  |  |  |

| `QualityProfile` | struct |  |  |  |  |  |

| `Raw` | struct |  |  |  |  |  |

| `ReceiptChain` | struct |  |  |  |  |  |

| `ReceiptChainConst` | struct |  |  |  |  |  |

| `ReceiptEnvelope` | struct |  |  |  |  |  |

| `Receipted` | struct |  |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |

| `Refused` | struct |  |  |  |  |  |

| `ReplayHint` | struct |  |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct |  |  |  |  |  |

| `SeparableWfNet` | struct |  |  |  |  |  |

| `Simplicity` | struct |  |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct |  |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct |  |  |  |  |  |

| `TemporalConstraint` | struct |  |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `Transition` | struct |  |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct |  |  |  |  |  |

| `TypedEventPredicate` | struct |  |  |  |  |  |

| `TypedId` | struct |  |  |  |  |  |

| `TypedLoopNode` | struct |  |  |  |  |  |

| `TypedObjectPredicate` | struct |  |  |  |  |  |

| `TypedPowlLoopNode` | struct |  |  |  |  |  |

| `TypedRelationPredicate` | struct |  |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct |  |  |  |  |  |

| `WfNetConst` | struct |  |  |  |  |  |

| `Witnessed` | struct |  |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |


### stubs/wasm4pm-compat/src/lib.rs

| `ArcDirection` | enum |  |  |  |  |  |

| `ArcDirectionConst` | enum |  |  |  |  |  |

| `BpmnEvent` | enum |  |  |  |  |  |

| `BpmnGateway` | enum |  |  |  |  |  |

| `BpmnNodeKind` | enum |  |  |  |  |  |

| `BpmnRefusal` | enum |  |  |  |  |  |

| `CausalConsistency` | enum |  |  |  |  |  |

| `CausalNetRefusal` | enum |  |  |  |  |  |

| `CompatDiagnostic` | enum |  |  |  |  |  |

| `ComplianceKind` | enum |  |  |  |  |  |

| `ConformanceRefusal` | enum |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `CorrelationSchema` | enum |  |  |  |  |  |

| `CubeDimensionKind` | enum |  |  |  |  |  |

| `DeclareRefusal` | enum |  |  |  |  |  |

| `DeclareScope` | enum |  |  |  |  |  |

| `DeclareTemplate` | enum |  |  |  |  |  |

| `DfgRefusal` | enum |  |  |  |  |  |

| `DiagnosticKind` | enum |  |  |  |  |  |

| `DiagnosticSeverity` | enum |  |  |  |  |  |

| `EventLogRefusal` | enum |  |  |  |  |  |

| `EventPredicateKind` | enum |  |  |  |  |  |

| `EvidenceMode` | enum |  |  |  |  |  |

| `FilterShape` | enum |  |  |  |  |  |

| `FormatKind` | enum |  |  |  |  |  |

| `InstanceCreationKind` | enum |  |  |  |  |  |

| `InteropRefusal` | enum |  |  |  |  |  |

| `KernelRefusal` | enum |  |  |  |  |  |

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `LossFunction` | enum |  |  |  |  |  |

| `LossPolicy` | enum |  |  |  |  |  |

| `LossRefusal` | enum |  |  |  |  |  |

| `OCELAttributeValue` | enum |  |  |  |  |  |

| `ObjectCentricity` | enum |  |  |  |  |  |

| `ObjectLifecyclePhase` | enum |  |  |  |  |  |

| `ObjectPredicateKind` | enum |  |  |  |  |  |

| `ObjectTypeCardinality` | enum |  |  |  |  |  |

| `OcDeclareRefusal` | enum |  |  |  |  |  |

| `OcelAttributeValue` | enum |  |  |  |  |  |

| `OcelRefusal` | enum |  |  |  |  |  |

| `OcpqRefusal` | enum |  |  |  |  |  |

| `OcpqScopeKind` | enum |  |  |  |  |  |

| `PerspectiveRefusal` | enum |  |  |  |  |  |

| `PetriNetRefusal` | enum |  |  |  |  |  |

| `PetriRefusal` | enum |  |  |  |  |  |

| `Pm4pyShape` | enum |  |  |  |  |  |

| `Powl8Op` | enum |  |  |  |  |  |

| `Powl8OpError` | enum |  |  |  |  |  |

| `PowlNodeKind` | enum |  |  |  |  |  |

| `PowlProjectionState` | enum |  |  |  |  |  |

| `PowlRefusal` | enum |  |  |  |  |  |

| `PredicateKind` | enum |  |  |  |  |  |

| `PredictionHorizon` | enum |  |  |  |  |  |

| `PredictionRefusal` | enum |  |  |  |  |  |

| `PredictionTarget` | enum |  |  |  |  |  |

| `ProcessPerspective` | enum |  |  |  |  |  |

| `ProcessShapeKind` | enum |  |  |  |  |  |

| `ProcessTreeNode` | enum |  |  |  |  |  |

| `ProcessTreeOperator` | enum |  |  |  |  |  |

| `ProcessTreeRefusal` | enum |  |  |  |  |  |

| `QualityDimension` | enum |  |  |  |  |  |

| `QualityMetricKind` | enum |  |  |  |  |  |

| `ReceiptRefusal` | enum |  |  |  |  |  |

| `ReceiptVerdict` | enum |  |  |  |  |  |

| `RelationLaw` | enum |  |  |  |  |  |

| `RelationPredicateKind` | enum |  |  |  |  |  |

| `ReplayHintKind` | enum |  |  |  |  |  |

| `SoundnessState` | enum |  |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum |  |  |  |  |  |

| `SummaryShape` | enum |  |  |  |  |  |

| `TemporalOrder` | enum |  |  |  |  |  |

| `TemporalRefusal` | enum |  |  |  |  |  |

| `TemporalRelation` | enum |  |  |  |  |  |

| `WitnessFamily` | enum |  |  |  |  |  |

| `WorkflowPattern` | enum |  |  |  |  |  |

| `activity` | function | activity(&self) |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) |  |  |  |  |

| `admits` | function | admits(&self, count: usize) |  |  |  |  |

| `arcs` | function | arcs(&self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `as_f64` | function | as_f64(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) |  |  |  |  |

| `attribute` | function | attribute(&self) |  |  |  |  |

| `attributes` | function | attributes(&self) |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) |  |  |  |  |

| `by` | function | by(mut self, resource: &str) |  |  |  |  |

| `case_id` | function | case_id(&self) |  |  |  |  |

| `category` | function | category(&self) |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) |  |  |  |  |

| `claim_sound` | function | claim_sound(self) |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) |  |  |  |  |

| `default` | function | default() |  |  |  |  |

| `den` | function | den(&self) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) |  |  |  |  |

| `edges` | function | edges(&self) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) |  |  |  |  |

| `event_count` | function | event_count(&self) |  |  |  |  |

| `event_id` | function | event_id(&self) |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) |  |  |  |  |

| `event_set` | function | event_set(&self) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `expression` | function | expression(&self) |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) |  |  |  |  |

| `final_marking` | function | final_marking(&self) |  |  |  |  |

| `float` | function | float(key: &str, value: f64) |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) |  |  |  |  |

| `frequency` | function | frequency(&self) |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) |  |  |  |  |

| `from_owned` | function | from_owned(s: String) |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) |  |  |  |  |

| `get` | function | get(&self) |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) |  |  |  |  |

| `inner` | function | inner(&self) |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) |  |  |  |  |

| `into_admitted` | function | into_admitted(self) |  |  |  |  |

| `into_evidence` | function | into_evidence(self) |  |  |  |  |

| `into_exportable` | function | into_exportable(self) |  |  |  |  |

| `into_inner` | function | into_inner(self) |  |  |  |  |

| `into_lost` | function | into_lost(self) |  |  |  |  |

| `into_parsed` | function | into_parsed(self) |  |  |  |  |

| `into_projected` | function | into_projected(self) |  |  |  |  |

| `into_reason` | function | into_reason(self) |  |  |  |  |

| `into_receipted` | function | into_receipted(self) |  |  |  |  |

| `is_chain` | function | is_chain(&self) |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) |  |  |  |  |

| `is_negative` | function | is_negative(&self) |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) |  |  |  |  |

| `is_silent` | function | is_silent(&self) |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `kind` | function | kind(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `lanes` | function | lanes(&self) |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `length` | function | length(&self) |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `members` | function | members(&self) |  |  |  |  |

| `min` | function | min(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `net` | function | net(&self) |  |  |  |  |

| `new` | function | new(value: T) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `node_ids` | function | node_ids(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `num` | function | num(&self) |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) |  |  |  |  |

| `object_changes` | function | object_changes(&self) |  |  |  |  |

| `object_id` | function | object_id(&self) |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) |  |  |  |  |

| `object_set` | function | object_set(&self) |  |  |  |  |

| `object_type` | function | object_type(&self) |  |  |  |  |

| `object_types` | function | object_types(&self) |  |  |  |  |

| `objects` | function | objects(&self) |  |  |  |  |

| `place_id` | function | place_id(&self) |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) |  |  |  |  |

| `places` | function | places(&self) |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) |  |  |  |  |

| `process` | function | process(&self) |  |  |  |  |

| `projection` | function | projection(&self) |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) |  |  |  |  |

| `qualifier` | function | qualifier(&self) |  |  |  |  |

| `raw` | function | raw(value: T) |  |  |  |  |

| `resource` | function | resource(&self) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `scope` | function | scope(&self) |  |  |  |  |

| `silent` | function | silent(id: &str) |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) |  |  |  |  |

| `source` | function | source(&self) |  |  |  |  |

| `source_id` | function | source_id(&self) |  |  |  |  |

| `steps` | function | steps(&self) |  |  |  |  |

| `string` | function | string(key: &str, value: &str) |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) |  |  |  |  |

| `summary` | function | summary(&self, category: &str) |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `target` | function | target(&self) |  |  |  |  |

| `target_id` | function | target_id(&self) |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) |  |  |  |  |

| `tip` | function | tip(&self) |  |  |  |  |

| `tokens` | function | tokens(&self) |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) |  |  |  |  |

| `trace_count` | function | trace_count(&self) |  |  |  |  |

| `traces` | function | traces(&self) |  |  |  |  |

| `transition_id` | function | transition_id(&self) |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) |  |  |  |  |

| `transitions` | function | transitions(&self) |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `value` | function | value(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `weight` | function | weight(&self) |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) |  |  |  |  |

| `Activity` | struct |  |  |  |  |  |

| `Admission` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Arc` | struct |  |  |  |  |  |

| `ArtifactGrounding` | struct |  |  |  |  |  |

| `Between01` | struct |  |  |  |  |  |

| `BipartiteArcConst` | struct |  |  |  |  |  |

| `BpmnEdge` | struct |  |  |  |  |  |

| `BpmnLane` | struct |  |  |  |  |  |

| `BpmnNode` | struct |  |  |  |  |  |

| `BpmnPool` | struct |  |  |  |  |  |

| `BpmnProcess` | struct |  |  |  |  |  |

| `BpmnTask` | struct |  |  |  |  |  |

| `CancellationRegion` | struct |  |  |  |  |  |

| `CausalBinding` | struct |  |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct |  |  |  |  |  |

| `CausalNet` | struct |  |  |  |  |  |

| `CausallyOrderedEvidence` | struct |  |  |  |  |  |

| `ChoiceGraph` | struct |  |  |  |  |  |

| `ConditionCell` | struct |  |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `ConformanceVerdict` | struct |  |  |  |  |  |

| `ConsistencyVerified` | struct |  |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct |  |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct |  |  |  |  |  |

| `DenseKernel` | struct |  |  |  |  |  |

| `DependencyMeasure` | struct |  |  |  |  |  |

| `Deviation` | struct |  |  |  |  |  |

| `Dfg` | struct |  |  |  |  |  |

| `DfgEdge` | struct |  |  |  |  |  |

| `DfgEdgeFull` | struct |  |  |  |  |  |

| `DfgNode` | struct |  |  |  |  |  |

| `DfgWeight` | struct |  |  |  |  |  |

| `DiagnosticReport` | struct |  |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `EventLog` | struct |  |  |  |  |  |

| `EventObjectLink` | struct |  |  |  |  |  |

| `EventStream` | struct |  |  |  |  |  |

| `EventTypeName` | struct |  |  |  |  |  |

| `Evidence` | struct |  |  |  |  |  |

| `Exportable` | struct |  |  |  |  |  |

| `F1` | struct |  |  |  |  |  |

| `Fitness` | struct |  |  |  |  |  |

| `Generalization` | struct |  |  |  |  |  |

| `InitialFinalMarkingPair` | struct |  |  |  |  |  |

| `InputBinding` | struct |  |  |  |  |  |

| `LifecycleEvent` | struct |  |  |  |  |  |

| `LossChain` | struct |  |  |  |  |  |

| `LossReport` | struct |  |  |  |  |  |

| `Marking` | struct |  |  |  |  |  |

| `Metric` | struct |  |  |  |  |  |

| `MultiPerspectiveEvidence` | struct |  |  |  |  |  |

| `MultiPerspectiveLog` | struct |  |  |  |  |  |

| `MultipleInstanceSpec` | struct |  |  |  |  |  |

| `MultipleInstanceSpecConst` | struct |  |  |  |  |  |

| `NamedLoss` | struct |  |  |  |  |  |

| `OCEL` | struct |  |  |  |  |  |

| `OCELEvent` | struct |  |  |  |  |  |

| `OCELEventAttribute` | struct |  |  |  |  |  |

| `OCELObject` | struct |  |  |  |  |  |

| `OCELRelationship` | struct |  |  |  |  |  |

| `OCELType` | struct |  |  |  |  |  |

| `OCELTypeAttribute` | struct |  |  |  |  |  |

| `Object` | struct |  |  |  |  |  |

| `ObjectCentricDfg` | struct |  |  |  |  |  |

| `ObjectCentricPetriNet` | struct |  |  |  |  |  |

| `ObjectChange` | struct |  |  |  |  |  |

| `ObjectLifecycle` | struct |  |  |  |  |  |

| `ObjectObjectLink` | struct |  |  |  |  |  |

| `ObjectScope` | struct |  |  |  |  |  |

| `ObjectScopeConst` | struct |  |  |  |  |  |

| `ObjectTypeCardinality` | struct |  |  |  |  |  |

| `ObjectTypeName` | struct |  |  |  |  |  |

| `OcDeclareConstraint` | struct |  |  |  |  |  |

| `OcelAttribute` | struct |  |  |  |  |  |

| `OcelEvent` | struct |  |  |  |  |  |

| `OcelLog` | struct |  |  |  |  |  |

| `OcpqQuery` | struct |  |  |  |  |  |

| `OcpqQueryConst` | struct |  |  |  |  |  |

| `OrderEdge` | struct |  |  |  |  |  |

| `OutputBinding` | struct |  |  |  |  |  |

| `PackedKeyTable` | struct |  |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct |  |  |  |  |  |

| `PerspectiveCombination` | struct |  |  |  |  |  |

| `PetriNet` | struct |  |  |  |  |  |

| `Place` | struct |  |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct |  |  |  |  |  |

| `Powl` | struct |  |  |  |  |  |

| `PowlChoiceNode` | struct |  |  |  |  |  |

| `PowlComposition` | struct |  |  |  |  |  |

| `PowlNode` | struct |  |  |  |  |  |

| `PowlNodeId` | struct |  |  |  |  |  |

| `Precision` | struct |  |  |  |  |  |

| `Predicate` | struct |  |  |  |  |  |

| `ProcessCube` | struct |  |  |  |  |  |

| `ProcessSlice` | struct |  |  |  |  |  |

| `ProcessTree` | struct |  |  |  |  |  |

| `ProcessTreeNodeId` | struct |  |  |  |  |  |

| `Projected` | struct |  |  |  |  |  |

| `ProjectionName` | struct |  |  |  |  |  |

| `ProjectionNameOwned` | struct |  |  |  |  |  |

| `QualityProfile` | struct |  |  |  |  |  |

| `Raw` | struct |  |  |  |  |  |

| `ReceiptChain` | struct |  |  |  |  |  |

| `ReceiptChainConst` | struct |  |  |  |  |  |

| `ReceiptEnvelope` | struct |  |  |  |  |  |

| `Receipted` | struct |  |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |

| `Refused` | struct |  |  |  |  |  |

| `ReplayHint` | struct |  |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct |  |  |  |  |  |

| `SeparableWfNet` | struct |  |  |  |  |  |

| `Simplicity` | struct |  |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct |  |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct |  |  |  |  |  |

| `TemporalConstraint` | struct |  |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `Transition` | struct |  |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct |  |  |  |  |  |

| `TypedEventPredicate` | struct |  |  |  |  |  |

| `TypedId` | struct |  |  |  |  |  |

| `TypedLoopNode` | struct |  |  |  |  |  |

| `TypedObjectPredicate` | struct |  |  |  |  |  |

| `TypedPowlLoopNode` | struct |  |  |  |  |  |

| `TypedRelationPredicate` | struct |  |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct |  |  |  |  |  |

| `WfNetConst` | struct |  |  |  |  |  |

| `Witnessed` | struct |  |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |


### stubs/wasm4pm/src/lib.rs

| `AttributeValue` | enum |  |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) |  |  |  |  |

| `DFG` | struct |  |  |  |  |  |

| `DFGNode` | struct |  |  |  |  |  |

| `DirectlyFollowsRelation` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |


### stubs/wasm4pm/src/lib.rs

| `AttributeValue` | enum |  |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) |  |  |  |  |

| `DFG` | struct |  |  |  |  |  |

| `DFGNode` | struct |  |  |  |  |  |

| `DirectlyFollowsRelation` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |


### tests/catalog_tests.rs

| `filter_fixtures` | function | filter_fixtures(
        fixtures: Vec<FixtureMeta>,
        name: Option<&str>,
        events: Option<usize>,
    ) |  |  |  |  |

| `FixtureMeta` | struct |  |  |  |  |  |


### tests/property_based.rs

| `ArbitraryOperationEvent` | struct |  |  |  |  |  |

| `ArbitraryReceipt` | struct |  |  |  |  |  |


### tests/quality_monitor.rs

| `add` | function | add(a: i32, b: i32) |  |  |  |  |

| `another_feature` | function | another_feature() |  |  |  |  |

| `complex_feature` | function | complex_feature() |  |  |  |  |

| `compute` | function | compute(x: i32) |  |  |  |  |

| `dangerous_path` | function | dangerous_path() |  |  |  |  |

| `subtract` | function | subtract(a: i32, b: i32) |  |  |  |  |


### tests/visualize_tests.rs

| `from_receipt_dfg` | function | from_receipt_dfg(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(&self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/csp.rs

| `CspAc3` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/csp.rs

| `CspAc3` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/encode.rs

| `Engine` | enum |  |  |  |  |  |

| `breed_id` | function | breed_id(self) |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) |  |  |  |  |

| `from_id` | function | from_id(s: &str) |  |  |  |  |

| `generate_config` | function | generate_config(
    space: &FeatureSpace,
    query: &ConfigQuery,
    engine: Engine,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) |  |  |  |  |

| `ConfigQuery` | struct |  |  |  |  |  |

| `SemanticConfig` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/encode.rs

| `Engine` | enum |  |  |  |  |  |

| `breed_id` | function | breed_id(self) |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) |  |  |  |  |

| `from_id` | function | from_id(s: &str) |  |  |  |  |

| `generate_config` | function | generate_config(
    space: &FeatureSpace,
    query: &ConfigQuery,
    engine: Engine,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) |  |  |  |  |

| `ConfigQuery` | struct |  |  |  |  |  |

| `SemanticConfig` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/mod.rs

| `Verdict` | enum |  |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_sat` | function | is_sat(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) |  |  |  |  |

| `supported_breeds` | function | supported_breeds() |  |  |  |  |

| `tag` | function | tag(self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `BreedResult` | struct |  |  |  |  |  |

| `Contract` | struct |  |  |  |  |  |

| `Fact` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `TraceStep` | struct |  |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |


### tools/confevo-rs/src/breeds/mod.rs

| `Verdict` | enum |  |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_sat` | function | is_sat(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) |  |  |  |  |

| `supported_breeds` | function | supported_breeds() |  |  |  |  |

| `tag` | function | tag(self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `BreedResult` | struct |  |  |  |  |  |

| `Contract` | struct |  |  |  |  |  |

| `Fact` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `TraceStep` | struct |  |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |


### tools/confevo-rs/src/breeds/sat.rs

| `SatCdcl` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/sat.rs

| `SatCdcl` | struct |  |  |  |  |  |


### tools/confevo-rs/src/evolve.rs

| `run_ga` | function | run_ga(
    eval: &mut impl Evaluator,
    space: &FeatureSpace,
    cfg: &GaConfig,
) |  |  |  |  |

| `GaConfig` | struct |  |  |  |  |  |

| `GaError` | struct |  |  |  |  |  |

| `GaResult` | struct |  |  |  |  |  |

| `GenerationRecord` | struct |  |  |  |  |  |


### tools/confevo-rs/src/evolve.rs

| `run_ga` | function | run_ga(
    eval: &mut impl Evaluator,
    space: &FeatureSpace,
    cfg: &GaConfig,
) |  |  |  |  |

| `GaConfig` | struct |  |  |  |  |  |

| `GaError` | struct |  |  |  |  |  |

| `GaResult` | struct |  |  |  |  |  |

| `GenerationRecord` | struct |  |  |  |  |  |


### tools/confevo-rs/src/fitness.rs

| `cargo_available` | function | cargo_available() |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) |  |  |  |  |

| `generic` | function | generic() |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) |  |  |  |  |

| `score_from` | function | score_from(
    weights: &ScoreWeights,
    builds: bool,
    resolves: bool,
    error_count: u64,
    n_features: usize,
    elapsed_s: f64,
) |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) |  |  |  |  |

| `CargoEvaluator` | struct |  |  |  |  |  |

| `EvalResult` | struct |  |  |  |  |  |

| `ScoreWeights` | struct |  |  |  |  |  |

| `SyntheticEvaluator` | struct |  |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |


### tools/confevo-rs/src/fitness.rs

| `cargo_available` | function | cargo_available() |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) |  |  |  |  |

| `generic` | function | generic() |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) |  |  |  |  |

| `score_from` | function | score_from(
    weights: &ScoreWeights,
    builds: bool,
    resolves: bool,
    error_count: u64,
    n_features: usize,
    elapsed_s: f64,
) |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) |  |  |  |  |

| `CargoEvaluator` | struct |  |  |  |  |  |

| `EvalResult` | struct |  |  |  |  |  |

| `ScoreWeights` | struct |  |  |  |  |  |

| `SyntheticEvaluator` | struct |  |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |


### tools/confevo-rs/src/genome.rs

| `canonical` | function | canonical(&self, space: &FeatureSpace) |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `feature_set` | function | feature_set(&self) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) |  |  |  |  |

| `Genome` | struct |  |  |  |  |  |


### tools/confevo-rs/src/genome.rs

| `canonical` | function | canonical(&self, space: &FeatureSpace) |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `feature_set` | function | feature_set(&self) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) |  |  |  |  |

| `Genome` | struct |  |  |  |  |  |


### tools/confevo-rs/src/manifest.rs

| `ManifestError` | enum |  |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml(
    path: impl AsRef<Path>,
    include_default: bool,
) |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str(
    toml: &str,
    include_default: bool,
) |  |  |  |  |


### tools/confevo-rs/src/manifest.rs

| `ManifestError` | enum |  |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml(
    path: impl AsRef<Path>,
    include_default: bool,
) |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str(
    toml: &str,
    include_default: bool,
) |  |  |  |  |


### tools/confevo-rs/src/report.rs

| `Mode` | enum |  |  |  |  |  |

| `to_json` | function | to_json(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `to_markdown` | function | to_markdown(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |


### tools/confevo-rs/src/report.rs

| `Mode` | enum |  |  |  |  |  |

| `to_json` | function | to_json(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `to_markdown` | function | to_markdown(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |


### tools/confevo-rs/src/rng.rs

| `below` | function | below(&mut self, n: usize) |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) |  |  |  |  |

| `new` | function | new(seed: u64) |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) |  |  |  |  |

| `Rng` | struct |  |  |  |  |  |


### tools/confevo-rs/src/rng.rs

| `below` | function | below(&mut self, n: usize) |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) |  |  |  |  |

| `new` | function | new(seed: u64) |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) |  |  |  |  |

| `Rng` | struct |  |  |  |  |  |


### tools/confevo-rs/src/space.rs

| `SpaceError` | enum |  |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) |  |  |  |  |

| `contains` | function | contains(&self, name: &str) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `FeatureSpace` | struct |  |  |  |  |  |


### tools/confevo-rs/src/space.rs

| `SpaceError` | enum |  |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) |  |  |  |  |

| `contains` | function | contains(&self, name: &str) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `FeatureSpace` | struct |  |  |  |  |  |


### wip/1.2_diff_logic.rs

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `DiffResult` | struct |  |  |  |  |  |

| `ModifiedEvent` | struct |  |  |  |  |  |


### wip/1.2_diff_observability.rs

| `DiffError` | enum |  |  |  |  |  |

| `DiffSummary` | struct |  |  |  |  |  |

| `InstrumentedDiff` | trait |  |  |  |  |  |


### wip/1.3_visualize_logic.rs

| `build_graph` | function | build_graph(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |


### wip/1.3_visualize_observability.rs

| `new` | function | new(receipt_path: &str, format: &str) |  |  |  |  |

| `VisualizeInstrumentation` | struct |  |  |  |  |  |

| `VisualizeExt` | trait |  |  |  |  |  |


### wip/1.5_completion_logic.rs

| `generate_completions` | function | generate_completions(shell_name: &str) |  |  |  |  |

| `try_dispatch_completion` | function | try_dispatch_completion() |  |  |  |  |


### wip/1.5_completion_observability.rs

| `CompletionError` | enum |  |  |  |  |  |

| `Shell` | enum |  |  |  |  |  |

| `completion` | function | completion(shell_name: String) |  |  |  |  |


### wip/2.1_model_maximalist.rs

| `MiningError` | enum |  |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |


### wip/2.3_predict_maximalist.rs

| `PredictionError` | enum |  |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model(
    model: &AdmittedReceipt,
    current_trace: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |


### wip/2.4_2.5_lsp_maximalist.rs

| `from_receipt` | function | from_receipt(receipt: &Receipt, uri: &Url, text: &str) |  |  |  |  |

| `handle_definition` | function | handle_definition(pos: Position, index: &ReceiptIndex) |  |  |  |  |

| `handle_hover` | function | handle_hover(pos: Position, index: &ReceiptIndex) |  |  |  |  |

| `ObjectRefLocation` | struct |  |  |  |  |  |

| `ReceiptIndex` | struct |  |  |  |  |  |

| `ReceiptSymbol` | struct |  |  |  |  |  |


### wip/3.1_mutate_maximalist.rs

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### wip/3.2_3.3_generate_maximalist.rs

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) |  |  |  |  |

| `from_json` | function | from_json(json: &str) |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, 
        pattern_name: &str, 
        events: Vec<serde_json::Value>, 
        expected_verdict: &str,
        expected_failure_stage: Option<&str>
    ) |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) |  |  |  |  |

| `main` | function | main() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `CodegenEngine` | struct |  |  |  |  |  |

| `Snippet` | struct |  |  |  |  |  |

| `SnippetRegistry` | struct |  |  |  |  |  |


### wip/3.4_property_maximalist.rs

| `ArbitraryOperationEvent` | struct |  |  |  |  |  |

| `ArbitraryReceipt` | struct |  |  |  |  |  |


### wip/3.5_fixture_db_maximalist.rs

| `all` | function | all(&self) |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `reindex` | function | reindex(&mut self) |  |  |  |  |

| `save` | function | save(&self) |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) |  |  |  |  |

| `Fixture` | struct |  |  |  |  |  |

| `FixtureDatabase` | struct |  |  |  |  |  |

| `FixtureQuery` | struct |  |  |  |  |  |


### wip/4.2_4.5_metrics_maximalist.rs

| `SloViolation` | enum |  |  |  |  |  |

| `check_slo` | function | check_slo(&self) |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `new_noop` | function | new_noop() |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) |  |  |  |  |

| `MetricsCollector` | struct |  |  |  |  |  |

| `PrometheusExporter` | struct |  |  |  |  |  |

| `ServiceLevelIndicators` | struct |  |  |  |  |  |


### wip/5.5_repl_maximalist.rs

| `run` | function | run() |  |  |  |  |


### wip/B.1_throughput_observability.rs

| `compare_baseline` | function | compare_baseline(&mut self, bench_id: &str, current: f64, baseline: f64) |  |  |  |  |

| `load_baseline_value` | function | load_baseline_value(path: impl AsRef<Path>) |  |  |  |  |

| `new` | function | new(assembler: &'a mut ChainAssembler, counter: &'a mut SeqCounter) |  |  |  |  |

| `observe_criterion_bench` | function | observe_criterion_bench(&mut self, criterion_root: &str, bench_id: &str) |  |  |  |  |

| `record_throughput` | function | record_throughput(&mut self, bench_id: &str, ops_per_sec: f64) |  |  |  |  |

| `ThroughputObserver` | struct |  |  |  |  |  |


### wip/arch_upgrade_maximalist.rs

| `to_noun_verb_error` | function | to_noun_verb_error(err: AffidavitError) |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `MmrError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) |  |  |  |  |

| `peaks` | function | peaks(&self) |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `MmrAccumulator` | struct |  |  |  |  |  |

| `MmrProof` | struct |  |  |  |  |  |

| `MountainPeak` | struct |  |  |  |  |  |

| `MmrError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) |  |  |  |  |

| `peaks` | function | peaks(&self) |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `MmrAccumulator` | struct |  |  |  |  |  |

| `MmrProof` | struct |  |  |  |  |  |

| `MountainPeak` | struct |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `ChainBuilder` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `OwnedEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Seal` | struct |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(&self) |  |  |  |  |

| `ChainBuilder` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `OwnedEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Seal` | struct |  |  |  |  |  |

| `Algorithm` | enum |  |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `Profile` | enum |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `wire_str` | function | wire_str(self) |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) |  |  |  |  |

| `EnvelopeRef` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |

| `Algorithm` | enum |  |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `Profile` | enum |  |  |  |  |  |

| `borrow` | function | borrow(&self) |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `wire_str` | function | wire_str(self) |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) |  |  |  |  |

| `EnvelopeRef` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) |  |  |  |  |

| `is_zero` | function | is_zero(&self) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) |  |  |  |  |

| `is_zero` | function | is_zero(&self) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |

| `EvidenceError` | enum |  |  |  |  |  |

| `SvidType` | enum |  |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence(
    request: AuthZenRequestRef<'_>,
    decision: AuthZenDecisionEvidenceRef<'_>,
    expected_policy_decision_point: &str,
    expected_principal: &str,
    expected_effect_digest: &str,
) |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity(
    evidence: WorkloadIdentityEvidenceRef<'_>,
    expected_spiffe_id: &str,
    expected_trust_domain: &str,
    allow_jwt: bool,
) |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `parse` | function | parse(raw: &'a str) |  |  |  |  |

| `AuthZenActionRef` | struct |  |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct |  |  |  |  |  |

| `AuthZenEntityRef` | struct |  |  |  |  |  |

| `AuthZenRequestRef` | struct |  |  |  |  |  |

| `SpiffeIdRef` | struct |  |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct |  |  |  |  |  |

| `EvidenceError` | enum |  |  |  |  |  |

| `SvidType` | enum |  |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence(
    request: AuthZenRequestRef<'_>,
    decision: AuthZenDecisionEvidenceRef<'_>,
    expected_policy_decision_point: &str,
    expected_principal: &str,
    expected_effect_digest: &str,
) |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity(
    evidence: WorkloadIdentityEvidenceRef<'_>,
    expected_spiffe_id: &str,
    expected_trust_domain: &str,
    allow_jwt: bool,
) |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `parse` | function | parse(raw: &'a str) |  |  |  |  |

| `AuthZenActionRef` | struct |  |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct |  |  |  |  |  |

| `AuthZenEntityRef` | struct |  |  |  |  |  |

| `AuthZenRequestRef` | struct |  |  |  |  |  |

| `SpiffeIdRef` | struct |  |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `fitness` | function | fitness(&self) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `fitness` | function | fitness(&self) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `AlphaRelation` | enum |  |  |  |  |  |

| `activities` | function | activities(&self) |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) |  |  |  |  |

| `Footprint` | struct |  |  |  |  |  |

| `AlphaRelation` | enum |  |  |  |  |  |

| `activities` | function | activities(&self) |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) |  |  |  |  |

| `Footprint` | struct |  |  |  |  |  |

| `activity_list` | function | activity_list(&self) |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `DirectlyFollowsGraph` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `activity_list` | function | activity_list(&self) |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `DirectlyFollowsGraph` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `distinct_activities` | function | distinct_activities(&self) |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) |  |  |  |  |

| `LogStatistics` | struct |  |  |  |  |  |

| `distinct_activities` | function | distinct_activities(&self) |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) |  |  |  |  |

| `LogStatistics` | struct |  |  |  |  |  |

| `RejectReason` | enum |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `is_accept` | function | is_accept(&self) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |

| `RejectReason` | enum |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `is_accept` | function | is_accept(&self) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `AbiError` | struct |  |  |  |  |  |

| `call` | function | call(request: &[u8]) |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() |  |  |  |  |

| `AbiError` | struct |  |  |  |  |  |

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) |  |  |  |  |

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) |  |  |  |  |

| `SignatureInputError` | enum |  |  |  |  |  |

| `code` | function | code(&self) |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) |  |  |  |  |

| `SignatureInputError` | enum |  |  |  |  |  |

| `code` | function | code(&self) |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |

| `call` | function | call(&mut self, request: Value) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |

| `Host` | struct |  |  |  |  |  |

| `call` | function | call(&mut self, request: Value) |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `wasm_path` | function | wasm_path() |  |  |  |  |

| `Host` | struct |  |  |  |  |  |

| `build` | script | next build |  |  |  |  |

| `dev` | script | next dev |  |  |  |  |

| `lint` | script | next lint |  |  |  |  |

| `start` | script | next start |  |  |  |  |

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `bold` | function | bold(text: &str) |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) |  |  |  |  |

| `dim` | function | dim(text: &str) |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) |  |  |  |  |

| `green` | function | green(text: &str) |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) |  |  |  |  |

| `red` | function | red(text: &str) |  |  |  |  |

| `yellow` | function | yellow(text: &str) |  |  |  |  |

| `GlobalArgs` | struct |  |  |  |  |  |

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `bold` | function | bold(text: &str) |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) |  |  |  |  |

| `dim` | function | dim(text: &str) |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) |  |  |  |  |

| `green` | function | green(text: &str) |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) |  |  |  |  |

| `red` | function | red(text: &str) |  |  |  |  |

| `yellow` | function | yellow(text: &str) |  |  |  |  |

| `GlobalArgs` | struct |  |  |  |  |  |

| `Error` | enum |  |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) |  |  |  |  |

| `Error` | enum |  |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `current` | function | current(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(domain: &str) |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct |  |  |  |  |  |

| `RollingHash` | struct |  |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `current` | function | current(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(domain: &str) |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct |  |  |  |  |  |

| `RollingHash` | struct |  |  |  |  |  |

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) |  |  |  |  |

| `new` | function | new(service_name: &str) |  |  |  |  |

| `TracingGuard` | struct |  |  |  |  |  |

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) |  |  |  |  |

| `new` | function | new(service_name: &str) |  |  |  |  |

| `TracingGuard` | struct |  |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder() |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) |  |  |  |  |

| `dir` | function | dir(&self) |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) |  |  |  |  |

| `TempReceipt` | struct |  |  |  |  |  |

| `TempReceiptBuilder` | struct |  |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) |  |  |  |  |

| `builder` | function | builder() |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) |  |  |  |  |

| `dir` | function | dir(&self) |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) |  |  |  |  |

| `TempReceipt` | struct |  |  |  |  |  |

| `TempReceiptBuilder` | struct |  |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `enabled` | function | enabled(self) |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |

| `ColorMode` | enum |  |  |  |  |  |

| `OutputFormat` | enum |  |  |  |  |  |

| `enabled` | function | enabled(self) |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |

| `AppError` | enum |  |  |  |  |  |

| `AppError` | enum |  |  |  |  |  |

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `new` | function | new(receipt_path: impl AsRef<Path>, source_dir: impl AsRef<Path>) |  |  |  |  |

| `remediate` | function | remediate(&self) |  |  |  |  |

| `AutoRemediator` | struct |  |  |  |  |  |

| `audit_workspace` | function | audit_workspace(&self) |  |  |  |  |

| `handle_governance_audit` | function | handle_governance_audit() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `ArchitectureProposal` | struct |  |  |  |  |  |

| `GovernanceAgent` | struct |  |  |  |  |  |

| `GovernanceReport` | struct |  |  |  |  |  |

| `verify_with_chaos` | function | verify_with_chaos(&self, receipt: &Receipt) |  |  |  |  |

| `ChaosVerifier` | struct |  |  |  |  |  |

| `predict` | function | predict(&self) |  |  |  |  |

| `run_cli` | function | run_cli() |  |  |  |  |

| `shell_integration` | function | shell_integration() |  |  |  |  |

| `Prediction` | struct |  |  |  |  |  |

| `Telepathy` | struct |  |  |  |  |  |

| `ShardingError` | enum |  |  |  |  |  |

| `new` | function | new(dht: Arc<dyn KademliaDHT>) |  |  |  |  |

| `shard_receipt` | function | shard_receipt(receipt: Receipt, shard_size: usize) |  |  |  |  |

| `verify_distributed` | function | verify_distributed(&self, receipt_id: &Blake3Hash) |  |  |  |  |

| `DistributedVerifier` | struct |  |  |  |  |  |

| `ReceiptManifest` | struct |  |  |  |  |  |

| `ReceiptShard` | struct |  |  |  |  |  |

| `KademliaDHT` | trait |  |  |  |  |  |

| `State` | enum |  |  |  |  |  |

| `get` | function | get() |  |  |  |  |

| `init` | function | init() |  |  |  |  |

| `terminate` | function | terminate() |  |  |  |  |

| `transition_to` | function | transition_to(expected_prev: State, next: State) |  |  |  |  |

| `CurrentState` | struct |  |  |  |  |  |

| `is_accepted` | function | is_accepted(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prepare_batch` | function | prepare_batch(
        receipts: &[crate::types::Receipt],
    ) |  |  |  |  |

| `verify_batch` | function | verify_batch(
        &self,
        events: &[GpuEvent],
        metadata: &[GpuReceiptMetadata],
    ) |  |  |  |  |

| `GpuEvent` | struct |  |  |  |  |  |

| `GpuReceiptMetadata` | struct |  |  |  |  |  |

| `GpuVerdict` | struct |  |  |  |  |  |

| `GpuVerifier` | struct |  |  |  |  |  |

| `get_hologram_svg` | function | get_hologram_svg(events: &[OperationEvent]) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `parse` | function | parse(&self, input: &str) |  |  |  |  |

| `NlpQueryParser` | struct |  |  |  |  |  |

| `all_attribute_keys` | function | all_attribute_keys() |  |  |  |  |

| `record_crypto_blake3_maximalist` | function | record_crypto_blake3_maximalist(
        _bytes_hashed: u64,
        _duration_ns: u64,
    ) |  |  |  |  |

| `record_wasm_verify_maximalist` | function | record_wasm_verify_maximalist(
        _module_hash: &str,
        _instruction_count: u64,
        _memory_peak: u64,
    ) |  |  |  |  |

| `compile` | function | compile(&self) |  |  |  |  |

| `new` | function | new(receipt: Receipt) |  |  |  |  |

| `ReceiptWasmCompiler` | struct |  |  |  |  |  |

| `run_repl` | function | run_repl(&mut self) |  |  |  |  |

| `TimeTravelDebugger` | struct |  |  |  |  |  |

| `AffidavitRefusal` | enum |  |  |  |  |  |

| `admit` | function | admit(receipt: Receipt) |  |  |  |  |

| `ArchitectureRefusal` | enum |  |  |  |  |  |

| `ArchitectureStanding` | enum |  |  |  |  |  |

| `EvidenceSource` | enum |  |  |  |  |  |

| `admit` | function | admit(
        &mut self,
        receipt: ArchitectureQualificationReceipt,
    ) |  |  |  |  |

| `admit_supersession` | function | admit_supersession(
        &mut self,
        supersession: Supersession,
    ) |  |  |  |  |

| `binding_digest` | function | binding_digest(&self) |  |  |  |  |

| `certify` | function | certify(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        qualification_evidence_digests: Vec<String>,
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) |  |  |  |  |

| `certify_from_evidence` | function | certify_from_evidence(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        evidence: &[QualificationEvidence],
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) |  |  |  |  |

| `current_qualified` | function | current_qualified(&self, abb_digest: &str) |  |  |  |  |

| `from_json_verified` | function | from_json_verified(json: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `observe` | function | observe(
        source: EvidenceSource,
        producer_digest: impl Into<String>,
        bytes: &[u8],
    ) |  |  |  |  |

| `query_json` | function | query_json(&self, abb_digest: &str) |  |  |  |  |

| `standing_of` | function | standing_of(&self, receipt_digest: &str) |  |  |  |  |

| `supersede` | function | supersede(
        &self,
        new_sbb_digest: impl Into<String>,
        new_subject_digest: impl Into<String>,
        evidence: Vec<String>,
    ) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `verify` | function | verify(
        &self,
        prior: &ArchitectureQualificationReceipt,
    ) |  |  |  |  |

| `verify_bytes` | function | verify_bytes(&self, bytes: &[u8]) |  |  |  |  |

| `verify_chain` | function | verify_chain(&self, prior: &Self) |  |  |  |  |

| `verify_evidence` | function | verify_evidence(
        &self,
        observed: &[(QualificationEvidence, &[u8]) |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) |  |  |  |  |

| `verify_replay` | function | verify_replay(
        &self,
        expected_abb_digest: &str,
        expected_contract_digest: &str,
        expected_sbb_digest: &str,
        expected_subject_digest: &str,
    ) |  |  |  |  |

| `ArchitectureQualificationReceipt` | struct |  |  |  |  |  |

| `ArchitectureStandingLedger` | struct |  |  |  |  |  |

| `QualificationEvidence` | struct |  |  |  |  |  |

| `Supersession` | struct |  |  |  |  |  |

| `DoWitness` | enum |  |  |  |  |  |

| `FastPathRefusal` | enum |  |  |  |  |  |

| `FenceError` | enum |  |  |  |  |  |

| `confirm` | function | confirm(&self, fence: &AuthorityFence) |  |  |  |  |

| `gate_do` | function | gate_do(witness: &DoWitness, live_root: Option<StateRoot>) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prepare_do_permit` | function | prepare_do_permit(&mut self, id: &StateKey) |  |  |  |  |

| `prove_revoked` | function | prove_revoked(&mut self, id: &StateKey) |  |  |  |  |

| `revoke` | function | revoke(&mut self, id: &StateKey) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `routes_to_sibling_slot` | function | routes_to_sibling_slot(queried: &StateKey, neighbor: &StateKey, depth: usize) |  |  |  |  |

| `unrevoke` | function | unrevoke(&mut self, id: &StateKey) |  |  |  |  |

| `verify_fast_path` | function | verify_fast_path(
        fence: &mut AuthorityFence,
        replay: &mut ReplayFilter,
        clock: &mut HlcClock,
        claim: &ConsequenceClaim,
    ) |  |  |  |  |

| `AdmittedProof` | struct |  |  |  |  |  |

| `AuthorityFence` | struct |  |  |  |  |  |

| `ConsequenceClaim` | struct |  |  |  |  |  |

| `bench_throughput` | function | bench_throughput(iterations: u32) |  |  |  |  |

| `bench_variance_on_receipt` | function | bench_variance_on_receipt(path: &str, iterations: u32) |  |  |  |  |

| `bench_variance_suite` | function | bench_variance_suite(_iterations: u32) |  |  |  |  |

| `run_profile_workload` | function | run_profile_workload(seconds: u64, _receipt_path: Option<&str>) |  |  |  |  |

| `run` | function | run() |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `BlsError` | enum |  |  |  |  |  |

| `aggregate_committee` | function | aggregate_committee(
    signatures: &[Signature<TinyBLS381>],
    public_keys: &[PublicKey<TinyBLS381>],
) |  |  |  |  |

| `public` | function | public(&self) |  |  |  |  |

| `sign` | function | sign(&self, context: &[u8], message: &[u8]) |  |  |  |  |

| `verify_committee` | function | verify_committee(
    context: &[u8],
    message: &[u8],
    aggregate_signature: &Signature<TinyBLS381>,
    aggregate_key: &PublicKey<TinyBLS381>,
) |  |  |  |  |

| `CommitteeKey` | struct |  |  |  |  |  |

| `Admission` | enum |  |  |  |  |  |

| `BrceError` | enum |  |  |  |  |  |

| `Entry` | enum |  |  |  |  |  |

| `Observation` | enum |  |  |  |  |  |

| `ReconciliationVerdict` | enum |  |  |  |  |  |

| `Rule` | enum |  |  |  |  |  |

| `actuate` | function | actuate(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        actuator: &mut (impl Actuator + Observer) |  |  |  |  |

| `admit` | function | admit(
        &mut self,
        request: Request,
        route: RouteDecision,
        policy: impl Fn(&Request, &RouteDecision) |  |  |  |  |

| `admitted` | function | admitted(&self) |  |  |  |  |

| `append` | function | append(&mut self, entry: Entry) |  |  |  |  |

| `compute_digest` | function | compute_digest(&self) |  |  |  |  |

| `construct` | function | construct(
        &mut self,
        admitted: &Admitted,
        consequence_id: &str,
        idempotent: bool,
    ) |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) |  |  |  |  |

| `court` | function | court(ledger: &BrceLedger, world: &dyn Observer) |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `execute` | function | execute(
        &mut self,
        action: &ConstructedAction,
        attempt_id: &str,
        actuator: &mut dyn Actuator,
        now: u64,
    ) |  |  |  |  |

| `from_entries` | function | from_entries(entries: impl IntoIterator<Item = Entry>) |  |  |  |  |

| `from_records` | function | from_records(records: Vec<LedgerRecord>) |  |  |  |  |

| `grant_digest` | function | grant_digest(&self) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `mutant_suite` | function | mutant_suite(base: &BrceLedger, world: &dyn Observer) |  |  |  |  |

| `new` | function | new(root: impl Into<PathBuf>) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `pending_consequences` | function | pending_consequences(ledger: &BrceLedger) |  |  |  |  |

| `prepare` | function | prepare(
        &mut self,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        now: u64,
    ) |  |  |  |  |

| `receipt` | function | receipt(
        &mut self,
        admitted: &Admitted,
        action: &ConstructedAction,
        grant: &AuthorityGrant,
        attempt_id: &str,
        observer: &dyn Observer,
    ) |  |  |  |  |

| `reconcile` | function | reconcile(
        &mut self,
        observer: &dyn Observer,
    ) |  |  |  |  |

| `record_digest` | function | record_digest(seq: u64, prev: &str, entry: &Entry) |  |  |  |  |

| `records` | function | records(&self) |  |  |  |  |

| `refused_rules` | function | refused_rules(&self) |  |  |  |  |

| `replay_digest` | function | replay_digest(ledger: &BrceLedger) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `seal` | function | seal(mut self) |  |  |  |  |

| `snapshot` | function | snapshot(world: &dyn Observer) |  |  |  |  |

| `ActuationEvidence` | struct |  |  |  |  |  |

| `ActuationResult` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `AuthorityGrant` | struct |  |  |  |  |  |

| `BrceLedger` | struct |  |  |  |  |  |

| `BrcePipeline` | struct |  |  |  |  |  |

| `BrceReceipt` | struct |  |  |  |  |  |

| `ConstructedAction` | struct |  |  |  |  |  |

| `CourtRefusal` | struct |  |  |  |  |  |

| `CourtVerdict` | struct |  |  |  |  |  |

| `EffectEvidence` | struct |  |  |  |  |  |

| `FileActuator` | struct |  |  |  |  |  |

| `LedgerRecord` | struct |  |  |  |  |  |

| `MutantOutcome` | struct |  |  |  |  |  |

| `ReplayIdentity` | struct |  |  |  |  |  |

| `Request` | struct |  |  |  |  |  |

| `RouteDecision` | struct |  |  |  |  |  |

| `StaticWorld` | struct |  |  |  |  |  |

| `VerificationEvidence` | struct |  |  |  |  |  |

| `Actuator` | trait |  |  |  |  |  |

| `Observer` | trait |  |  |  |  |  |

| `JcsError` | enum |  |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &serde_json::Value) |  |  |  |  |

| `canonical_commitment` | function | canonical_commitment(
    value: &serde_json::Value,
) |  |  |  |  |

| `CanonicalTimeError` | enum |  |  |  |  |  |

| `canonical_string` | function | canonical_string(ts: Timestamp) |  |  |  |  |

| `canonicalize` | function | canonicalize(s: &str) |  |  |  |  |

| `checked_add_ms` | function | checked_add_ms(ts: Timestamp, millis: u64) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |

| `format_catalog` | function | format_catalog(fixtures: &[Fixture]) |  |  |  |  |

| `list_fixtures` | function | list_fixtures(
    db: &FixtureDatabase,
    name_filter: Option<String>,
    events_filter: Option<usize>,
) |  |  |  |  |

| `CausalError` | enum |  |  |  |  |  |

| `add_dependency` | function | add_dependency(&mut self, from: K, to: K) |  |  |  |  |

| `contains` | function | contains(&self, node: K) |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `topological_order` | function | topological_order(&self) |  |  |  |  |

| `verify_acyclic` | function | verify_acyclic(&self) |  |  |  |  |

| `CausalGraph` | struct |  |  |  |  |  |

| `ChainError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, event: OperationEvent) |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) |  |  |  |  |

| `deserialize_receipt` | function | deserialize_receipt(bytes: &[u8]) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `finalize` | function | finalize(self) |  |  |  |  |

| `from_events` | function | from_events(events: Vec<OperationEvent>) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `load_working` | function | load_working() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) |  |  |  |  |

| `save_receipt` | function | save_receipt(receipt: &Receipt, path: &Path) |  |  |  |  |

| `save_working` | function | save_working(events: &[OperationEvent]) |  |  |  |  |

| `serialize_receipt` | function | serialize_receipt(receipt: &Receipt) |  |  |  |  |

| `ChainAssembler` | struct |  |  |  |  |  |

| `assemble` | function | assemble(out: Option<&str>) |  |  |  |  |

| `emit` | function | emit(
    event_type: &str,
    objects: &[String],
    payload: &str,
) |  |  |  |  |

| `show` | function | show(receipt: &str) |  |  |  |  |

| `verify` | function | verify(receipt: &str) |  |  |  |  |

| `AttestationError` | enum |  |  |  |  |  |

| `AttestationKind` | enum |  |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `attest_key` | function | attest_key(
    record: &KeyRecord,
    kind: AttestationKind,
    device: Option<(&str, bool, bool) |  |  |  |  |

| `verify_attestation` | function | verify_attestation(
    att: &AttestationRecord,
    signer_pk: &[u8],
) |  |  |  |  |

| `AttestationRecord` | struct |  |  |  |  |  |

| `CanonicalError` | enum |  |  |  |  |  |

| `digest` | function | digest(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `digest_hex` | function | digest_hex(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `domain_separated` | function | domain_separated(domain: &str, parts: &[&[u8]]) |  |  |  |  |

| `jcs` | function | jcs(value: &Value) |  |  |  |  |

| `CrlFileError` | enum |  |  |  |  |  |

| `load_and_apply` | function | load_and_apply(
        path: Option<&Path>,
        revocations: &mut RevocationList,
        issuer_pk: &[u8],
        current_epoch: u64,
        max_staleness: u64,
        now: u64,
    ) |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `publish_verified` | function | publish_verified(
        revocations: &RevocationList,
        issuer: &Es256SigningKey,
        issuer_kid: &KeyId,
        epoch: u64,
        at: u64,
        out: Option<&Path>,
    ) |  |  |  |  |

| `read` | function | read(path: Option<&Path>) |  |  |  |  |

| `write` | function | write(
        list: &SignedRevocationList,
        path: Option<&Path>,
    ) |  |  |  |  |

| `CrlFile` | struct |  |  |  |  |  |

| `envelope_law_finding` | function | envelope_law_finding() |  |  |  |  |

| `es256_selftest_finding` | function | es256_selftest_finding() |  |  |  |  |

| `pqc_selftest_finding` | function | pqc_selftest_finding() |  |  |  |  |

| `run_crypto_checks` | function | run_crypto_checks() |  |  |  |  |

| `store_integrity_finding` | function | store_integrity_finding(path: &Path) |  |  |  |  |

| `EnclaveError` | enum |  |  |  |  |  |

| `delete_enclave_key` | function | delete_enclave_key(label: &str) |  |  |  |  |

| `enclave_sign` | function | enclave_sign(label: &str, msg: &[u8]) |  |  |  |  |

| `enclave_verify` | function | enclave_verify(
        public_key_sec1: &[u8],
        msg: &[u8],
        sig_der: &[u8],
    ) |  |  |  |  |

| `generate_enclave_key` | function | generate_enclave_key(label: &str) |  |  |  |  |

| `EnclaveKeyRef` | struct |  |  |  |  |  |

| `EnvelopeError` | enum |  |  |  |  |  |

| `envelope_document` | function | envelope_document(&self) |  |  |  |  |

| `from_bytes` | function | from_bytes(b: &[u8]) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) |  |  |  |  |

| `record` | function | record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `signing_input` | function | signing_input(&self) |  |  |  |  |

| `signing_input_checked` | function | signing_input_checked(&self) |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) |  |  |  |  |

| `NonceJournal` | struct |  |  |  |  |  |

| `SignatureEnvelope` | struct |  |  |  |  |  |

| `Es256Error` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) |  |  |  |  |

| `generate` | function | generate() |  |  |  |  |

| `key_id_fingerprint` | function | key_id_fingerprint(&self) |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) |  |  |  |  |

| `verify_es256` | function | verify_es256(
    public_key_sec1: &[u8],
    msg: &[u8],
    sig_der: &[u8],
) |  |  |  |  |

| `Es256SigningKey` | struct |  |  |  |  |  |

| `JournalError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, draft: JournalEntryDraft) |  |  |  |  |

| `by_epoch` | function | by_epoch(&self, epoch: u64) |  |  |  |  |

| `by_key` | function | by_key(&self, kid: &str) |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `record_receipt` | function | record_receipt(
    journal: &mut StandingJournal,
    receipt: &crate::crypto_trust_verify::CryptoStandingReceipt,
    policy_epoch: u64,
    revocation_epoch: u64,
) |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) |  |  |  |  |

| `JournalEntry` | struct |  |  |  |  |  |

| `JournalEntryDraft` | struct |  |  |  |  |  |

| `StandingJournal` | struct |  |  |  |  |  |

| `PersistError` | enum |  |  |  |  |  |

| `entries` | function | entries(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `load` | function | load(path: impl Into<PathBuf>) |  |  |  |  |

| `open_or_create` | function | open_or_create(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) |  |  |  |  |

| `record` | function | record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) |  |  |  |  |

| `refresh` | function | refresh(&mut self) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) |  |  |  |  |

| `NonceLedgerFile` | struct |  |  |  |  |  |

| `NonceRecord` | struct |  |  |  |  |  |

| `JwksError` | enum |  |  |  |  |  |

| `export_jwk` | function | export_jwk(record: &KeyRecord) |  |  |  |  |

| `export_jwks` | function | export_jwks(records: &[KeyRecord]) |  |  |  |  |

| `KatError` | enum |  |  |  |  |  |

| `build_vector` | function | build_vector(index: usize, algorithm: &str) |  |  |  |  |

| `export_json` | function | export_json(corpus: &[KatVector]) |  |  |  |  |

| `generate_corpus` | function | generate_corpus() |  |  |  |  |

| `import_json` | function | import_json(s: &str) |  |  |  |  |

| `verify_corpus` | function | verify_corpus(vectors: &[KatVector]) |  |  |  |  |

| `verify_vector` | function | verify_vector(vector: &KatVector) |  |  |  |  |

| `KatReport` | struct |  |  |  |  |  |

| `KatVector` | struct |  |  |  |  |  |

| `AlgorithmId` | enum |  |  |  |  |  |

| `CryptoProfile` | enum |  |  |  |  |  |

| `KeyOrigin` | enum |  |  |  |  |  |

| `KeyProviderKind` | enum |  |  |  |  |  |

| `PublicKeyMaterial` | enum |  |  |  |  |  |

| `RegistryError` | enum |  |  |  |  |  |

| `algorithm` | function | algorithm(&self) |  |  |  |  |

| `all` | function | all() |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(&self) |  |  |  |  |

| `encoded_len` | function | encoded_len(&self) |  |  |  |  |

| `fingerprint_public_key` | function | fingerprint_public_key(
    algorithm: AlgorithmId,
    public_key: &PublicKeyMaterial,
) |  |  |  |  |

| `from_fingerprint` | function | from_fingerprint(fingerprint: &KeyFingerprint) |  |  |  |  |

| `key_material_exportable` | function | key_material_exportable(self) |  |  |  |  |

| `kind` | function | kind(self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `profile` | function | profile(self) |  |  |  |  |

| `public_key_len` | function | public_key_len(self) |  |  |  |  |

| `CustodianIdentity` | struct |  |  |  |  |  |

| `InMemoryKeyRegistry` | struct |  |  |  |  |  |

| `KeyFingerprint` | struct |  |  |  |  |  |

| `KeyId` | struct |  |  |  |  |  |

| `KeyRecord` | struct |  |  |  |  |  |

| `KeyRegistry` | trait |  |  |  |  |  |

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `active_at` | function | active_at(&self, now: u64) |  |  |  |  |

| `active_epoch` | function | active_epoch(&self, kid: &str) |  |  |  |  |

| `admit_opening` | function | admit_opening(&self, kid: &str, history: &[KeyEpoch]) |  |  |  |  |

| `current_epoch` | function | current_epoch(&self) |  |  |  |  |

| `epochs` | function | epochs(&self, kid: &str) |  |  |  |  |

| `is_revoked` | function | is_revoked(&self, kid: &str) |  |  |  |  |

| `new` | function | new(policy: RotationPolicy) |  |  |  |  |

| `open_epoch` | function | open_epoch(&mut self, kid: &str, at: u64) |  |  |  |  |

| `policy` | function | policy(&self) |  |  |  |  |

| `require_active_epoch` | function | require_active_epoch(&self, kid: &str, now: u64) |  |  |  |  |

| `retire_epoch` | function | retire_epoch(&mut self, kid: &str, index: u64, at: u64) |  |  |  |  |

| `revoke` | function | revoke(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `revoked_at` | function | revoked_at(&self, kid: &str) |  |  |  |  |

| `signature_epoch_live` | function | signature_epoch_live(
        &self,
        kid: &str,
        sig_revocation_epoch: u64,
        now: u64,
    ) |  |  |  |  |

| `validate_epoch` | function | validate_epoch(
        &self,
        kid: &str,
        epoch: &KeyEpoch,
        now: u64,
    ) |  |  |  |  |

| `validate_signature_key` | function | validate_signature_key(&self, kid: &str, now: u64) |  |  |  |  |

| `KeyEpoch` | struct |  |  |  |  |  |

| `LifecycleLedger` | struct |  |  |  |  |  |

| `RevocationList` | struct |  |  |  |  |  |

| `RevocationRecord` | struct |  |  |  |  |  |

| `RotationPolicy` | struct |  |  |  |  |  |

| `LogOpsError` | enum |  |  |  |  |  |

| `audit` | function | audit(&self) |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) |  |  |  |  |

| `heads` | function | heads(&self) |  |  |  |  |

| `journal` | function | journal(&self) |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) |  |  |  |  |

| `log` | function | log(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&self, leaf: u64) |  |  |  |  |

| `publish_head` | function | publish_head(
        &self,
        signer: &Es256SigningKey,
        kid: &str,
        now: u64,
    ) |  |  |  |  |

| `record` | function | record(
        &mut self,
        receipt: &CryptoStandingReceipt,
        commitment: [u8; 32],
        signer: &Es256SigningKey,
        signer_kid: &str,
        now: u64,
    ) |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) |  |  |  |  |

| `verify_head` | function | verify_head(head: &SignedTreeHead, public_key_sec1: &[u8]) |  |  |  |  |

| `AuditReport` | struct |  |  |  |  |  |

| `SignedTreeHead` | struct |  |  |  |  |  |

| `TrustLogOps` | struct |  |  |  |  |  |

| `NonceStoreError` | enum |  |  |  |  |  |

| `default_journal_path` | function | default_journal_path() |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open_default` | function | open_default() |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64) |  |  |  |  |

| `record` | function | record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `DiskNonceJournal` | struct |  |  |  |  |  |

| `PqcError` | enum |  |  |  |  |  |

| `hybrid_sign` | function | hybrid_sign(secret: &HybridSecret, msg: &[u8]) |  |  |  |  |

| `hybrid_verify` | function | hybrid_verify(
    es256_pk: &[u8],
    mldsa65_pk: &[u8],
    msg: &[u8],
    sig: &HybridSignature,
) |  |  |  |  |

| `ml_dsa65_from_seed` | function | ml_dsa65_from_seed(seed: &[u8; ML_DSA_65_SEED_LEN]) |  |  |  |  |

| `ml_dsa65_generate` | function | ml_dsa65_generate() |  |  |  |  |

| `ml_dsa65_sign` | function | ml_dsa65_sign(
    seed: &[u8; ML_DSA_65_SEED_LEN],
    msg: &[u8],
    rnd: &[u8; 32],
) |  |  |  |  |

| `ml_dsa65_verify` | function | ml_dsa65_verify(public: &[u8], msg: &[u8], sig: &[u8]) |  |  |  |  |

| `slh_dsa128s_from_seed` | function | slh_dsa128s_from_seed(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) |  |  |  |  |

| `slh_dsa128s_generate` | function | slh_dsa128s_generate() |  |  |  |  |

| `slh_dsa128s_sign` | function | slh_dsa128s_sign(
    seeds: &[u8; SLH_DSA_128S_SEED_LEN],
    msg: &[u8],
) |  |  |  |  |

| `slh_dsa128s_verify` | function | slh_dsa128s_verify(public: &[u8], msg: &[u8], sig: &[u8]) |  |  |  |  |

| `HybridSecret` | struct |  |  |  |  |  |

| `HybridSignature` | struct |  |  |  |  |  |

| `MlDsa65KeyPair` | struct |  |  |  |  |  |

| `SlhDsa128sKeyPair` | struct |  |  |  |  |  |

| `ProviderRefusal` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(id: KeyId, seed: &[u8; 32]) |  |  |  |  |

| `generate` | function | generate(id: KeyId) |  |  |  |  |

| `key_record` | function | key_record(&self, custodian: CustodianIdentity, created_epoch: u64) |  |  |  |  |

| `DetachedSignature` | struct |  |  |  |  |  |

| `SoftwareEs256Provider` | struct |  |  |  |  |  |

| `SigningProvider` | trait |  |  |  |  |  |

| `QuorumError` | enum |  |  |  |  |  |

| `allowed_algorithms` | function | allowed_algorithms(&self) |  |  |  |  |

| `new` | function | new(registry: &'a R) |  |  |  |  |

| `verify_quorum` | function | verify_quorum(
        &self,
        signing_input: &[u8],
        shares: &[SignatureShare],
        k: usize,
    ) |  |  |  |  |

| `with_allowed_algorithms` | function | with_allowed_algorithms(
        mut self,
        algs: impl IntoIterator<Item = AlgorithmId>,
    ) |  |  |  |  |

| `QuorumEngine` | struct |  |  |  |  |  |

| `QuorumVerdict` | struct |  |  |  |  |  |

| `SignatureShare` | struct |  |  |  |  |  |

| `RevocationPubError` | enum |  |  |  |  |  |

| `apply_to` | function | apply_to(
    revocations: &mut RevocationList,
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
    current_epoch: u64,
    max_staleness: u64,
    now: u64,
) |  |  |  |  |

| `publish` | function | publish(
    revocations: &RevocationList,
    issuer: &Es256SigningKey,
    issuer_kid: &KeyId,
    epoch: u64,
    at: u64,
) |  |  |  |  |

| `verify_publication` | function | verify_publication(
    crl: &SignedRevocationList,
    issuer_pk_sec1: &[u8],
) |  |  |  |  |

| `RevocationRecordMirror` | struct |  |  |  |  |  |

| `SignedRevocationList` | struct |  |  |  |  |  |

| `RotationError` | enum |  |  |  |  |  |

| `admission_allowed` | function | admission_allowed(from: CryptoProfile, to: CryptoProfile) |  |  |  |  |

| `assert_not_downgrade` | function | assert_not_downgrade(from: CryptoProfile, to: CryptoProfile) |  |  |  |  |

| `assert_not_downgrade_under` | function | assert_not_downgrade_under(
    from: CryptoProfile,
    to: CryptoProfile,
    policy: &MigrationPolicy,
) |  |  |  |  |

| `rotate_es256_to_hybrid` | function | rotate_es256_to_hybrid(
        old: &Es256SigningKey,
        hybrid_secret: &HybridSecret,
        at: u64,
    ) |  |  |  |  |

| `verify_rotation` | function | verify_rotation(
        record: &RotationRecord,
        hybrid_pk_es256: &[u8],
        hybrid_pk_mldsa65: &[u8],
    ) |  |  |  |  |

| `MigrationPolicy` | struct |  |  |  |  |  |

| `RotationCeremony` | struct |  |  |  |  |  |

| `RotationRecord` | struct |  |  |  |  |  |

| `RotationStoreError` | enum |  |  |  |  |  |

| `append` | function | append(
        &self,
        record: &RotationRecord,
        successor_public_es256: &[u8],
        successor_public_mldsa65: &[u8],
    ) |  |  |  |  |

| `latest_for_predecessor` | function | latest_for_predecessor(
        &self,
        kid: &str,
    ) |  |  |  |  |

| `latest_for_successor` | function | latest_for_successor(
        &self,
        kid: &str,
    ) |  |  |  |  |

| `load` | function | load(&self) |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `RotationStore` | struct |  |  |  |  |  |

| `RotationStoreEnvelope` | struct |  |  |  |  |  |

| `VerifiedRotation` | struct |  |  |  |  |  |

| `Sa2aWireError` | enum |  |  |  |  |  |

| `approval_to_envelope` | function | approval_to_envelope(a: &Sa2aApproval) |  |  |  |  |

| `envelope_sa2a_signing_input` | function | envelope_sa2a_signing_input(
    env: &SignatureEnvelope,
    principal: &str,
) |  |  |  |  |

| `envelope_to_approval` | function | envelope_to_approval(
    env: &SignatureEnvelope,
    principal: &str,
) |  |  |  |  |

| `sa2a_signing_input` | function | sa2a_signing_input(a: &Sa2aApproval) |  |  |  |  |

| `sa2a_signing_input_checked` | function | sa2a_signing_input_checked(a: &Sa2aApproval) |  |  |  |  |

| `Sa2aApproval` | struct |  |  |  |  |  |

| `SealError` | enum |  |  |  |  |  |

| `seal_receipt` | function | seal_receipt(
    receipt: &Receipt,
    envelope: SignatureEnvelope,
    signature: Vec<u8>,
) |  |  |  |  |

| `subject_digest_of` | function | subject_digest_of(receipt: &Receipt) |  |  |  |  |

| `verify_sealed` | function | verify_sealed(
    sealed: &SealedReceipt,
    engine: &VerificationEngine,
) |  |  |  |  |

| `SealedReceipt` | struct |  |  |  |  |  |

| `StoreError` | enum |  |  |  |  |  |

| `checksum_for` | function | checksum_for(records: &[KeyRecord]) |  |  |  |  |

| `load` | function | load(&self) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `records` | function | records(&self) |  |  |  |  |

| `register_checked` | function | register_checked(&mut self, record: KeyRecord) |  |  |  |  |

| `FileKeyStore` | struct |  |  |  |  |  |

| `KeyStoreFile` | struct |  |  |  |  |  |

| `TransparencyError` | enum |  |  |  |  |  |

| `append` | function | append(&mut self, commitment: [u8; 32]) |  |  |  |  |

| `consistency_proof` | function | consistency_proof(&self, first: u64) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `inclusion_proof` | function | inclusion_proof(&self, idx: u64) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `verify_consistency` | function | verify_consistency(
    proof: &ConsistencyProof,
    first_head: &[u8; 32],
    second_head: &[u8; 32],
) |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(leaf: &[u8; 32], proof: &InclusionProof, head: &[u8; 32]) |  |  |  |  |

| `ConsistencyProof` | struct |  |  |  |  |  |

| `InclusionProof` | struct |  |  |  |  |  |

| `TransparencyLog` | struct |  |  |  |  |  |

| `CryptographicStanding` | enum |  |  |  |  |  |

| `StandingReceiptError` | enum |  |  |  |  |  |

| `VerifyRefusal` | enum |  |  |  |  |  |

| `all` | function | all() |  |  |  |  |

| `as_str` | function | as_str(self) |  |  |  |  |

| `certify_signed` | function | certify_signed(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
        subject: &str,
        signing: &Es256SigningKey,
    ) |  |  |  |  |

| `from_graph_defaults` | function | from_graph_defaults() |  |  |  |  |

| `new` | function | new(
        registry: InMemoryKeyRegistry,
        revocations: RevocationList,
        nonces: NonceJournal,
        policy: TrustPolicy,
    ) |  |  |  |  |

| `nonce_seen_at` | function | nonce_seen_at(&self, kid: &str, nonce: &[u8; 16]) |  |  |  |  |

| `policy` | function | policy(&self) |  |  |  |  |

| `prune_nonces` | function | prune_nonces(&mut self) |  |  |  |  |

| `register_key` | function | register_key(&mut self, record: KeyRecord) |  |  |  |  |

| `registry` | function | registry(&self) |  |  |  |  |

| `revocations` | function | revocations(&self) |  |  |  |  |

| `revoke_key` | function | revoke_key(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `verify_envelope` | function | verify_envelope(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
    ) |  |  |  |  |

| `with_now` | function | with_now(mut self, now: u64) |  |  |  |  |

| `CryptoStandingReceipt` | struct |  |  |  |  |  |

| `CryptographicVerdict` | struct |  |  |  |  |  |

| `TrustPolicy` | struct |  |  |  |  |  |

| `VerificationEngine` | struct |  |  |  |  |  |

| `WitnessError` | enum |  |  |  |  |  |

| `collect` | function | collect(
        &self,
        witnesses: &[(&str, &Es256SigningKey) |  |  |  |  |

| `head` | function | head(&self) |  |  |  |  |

| `new` | function | new(head: SignedTreeHead) |  |  |  |  |

| `preimage` | function | preimage(&self) |  |  |  |  |

| `verify_cosigned` | function | verify_cosigned(
    head: &SignedTreeHead,
    cosigs: &[WitnessSignature],
    min: usize,
    pks: &[(&str, &[u8]) |  |  |  |  |

| `CosignVerdict` | struct |  |  |  |  |  |

| `WitnessCosign` | struct |  |  |  |  |  |

| `WitnessSignature` | struct |  |  |  |  |  |

| `ClosureGap` | enum |  |  |  |  |  |

| `DfcmRefusal` | enum |  |  |  |  |  |

| `EvidenceKind` | enum |  |  |  |  |  |

| `certify_dfcm` | function | certify_dfcm(
    mut release_profile: DfcmProfile,
    mut observations: Vec<SubjectObservation>,
) |  |  |  |  |

| `v26_9_18_profile` | function | v26_9_18_profile(s: V26_9_18Subjects) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `DfcmProfile` | struct |  |  |  |  |  |

| `DfcmReceipt` | struct |  |  |  |  |  |

| `EvidenceKey` | struct |  |  |  |  |  |

| `EvidenceWitness` | struct |  |  |  |  |  |

| `ExactSubject` | struct |  |  |  |  |  |

| `MinimalFrontier` | struct |  |  |  |  |  |

| `Obligation` | struct |  |  |  |  |  |

| `ObligationEvaluation` | struct |  |  |  |  |  |

| `PathEvaluation` | struct |  |  |  |  |  |

| `ProofPath` | struct |  |  |  |  |  |

| `SubjectObservation` | struct |  |  |  |  |  |

| `SubjectRequirement` | struct |  |  |  |  |  |

| `V26_9_18Subjects` | struct |  |  |  |  |  |

| `ErrorCode` | enum |  |  |  |  |  |

| `code` | function | code(self) |  |  |  |  |

| `exit_code` | function | exit_code(self) |  |  |  |  |

| `from_error` | function | from_error(code: ErrorCode, err: &dyn std::error::Error) |  |  |  |  |

| `hint` | function | hint(self) |  |  |  |  |

| `message` | function | message(self) |  |  |  |  |

| `new` | function | new(code: ErrorCode, message: impl Into<String>) |  |  |  |  |

| `with_hint` | function | with_hint(mut self, hint: impl Into<String>) |  |  |  |  |

| `with_span` | function | with_span(mut self, file: impl Into<String>, line: Option<u32>) |  |  |  |  |

| `Diag` | struct |  |  |  |  |  |

| `Span` | struct |  |  |  |  |  |

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `DiffEntry` | struct |  |  |  |  |  |

| `DiffResult` | struct |  |  |  |  |  |

| `ModifiedEntry` | struct |  |  |  |  |  |

| `conformance_metrics` | function | conformance_metrics(receipt: &Receipt) |  |  |  |  |

| `discover_dfg_summary` | function | discover_dfg_summary(receipt: &Receipt) |  |  |  |  |

| `discover_from_admitted` | function | discover_from_admitted(admitted: &crate::types::AdmittedReceipt) |  |  |  |  |

| `discover_process_tree` | function | discover_process_tree(receipt: &Receipt) |  |  |  |  |

| `project_to_event_log` | function | project_to_event_log(receipt: &Receipt) |  |  |  |  |

| `quality_metrics` | function | quality_metrics(receipt: &Receipt) |  |  |  |  |

| `quality_metrics_from_admitted` | function | quality_metrics_from_admitted(admitted: &crate::types::AdmittedReceipt) |  |  |  |  |

| `FindingStatus` | enum |  |  |  |  |  |

| `auto_fixable` | function | auto_fixable(mut self) |  |  |  |  |

| `fail` | function | fail(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `ok` | function | ok(id: &'static str, message: impl Into<String>) |  |  |  |  |

| `run_all` | function | run_all() |  |  |  |  |

| `warn` | function | warn(
        id: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) |  |  |  |  |

| `Finding` | struct |  |  |  |  |  |

| `DoctorCheck` | trait |  |  |  |  |  |

| `EcosystemRefusal` | enum |  |  |  |  |  |

| `EcosystemRole` | enum |  |  |  |  |  |

| `certify_ecosystem` | function | certify_ecosystem(
    admitted: &AdmittedReceipt,
    mut observation: EcosystemObservation,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `EcosystemMember` | struct |  |  |  |  |  |

| `EcosystemObservation` | struct |  |  |  |  |  |

| `EcosystemReceipt` | struct |  |  |  |  |  |

| `RoleCoverage` | struct |  |  |  |  |  |

| `RoleRequirement` | struct |  |  |  |  |  |

| `Ed25519WitnessError` | enum |  |  |  |  |  |

| `generate` | function | generate() |  |  |  |  |

| `public` | function | public(&self) |  |  |  |  |

| `sign` | function | sign(&self, message: &[u8]) |  |  |  |  |

| `verify_witness` | function | verify_witness(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) |  |  |  |  |

| `WitnessKeyPair` | struct |  |  |  |  |  |

| `ErrcQuadrant` | enum |  |  |  |  |  |

| `ErrcRefusal` | enum |  |  |  |  |  |

| `certify_errc` | function | certify_errc(
    admitted: &AdmittedReceipt,
    mut observation: ErrcObservation,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `ErrcClaim` | struct |  |  |  |  |  |

| `ErrcMeasure` | struct |  |  |  |  |  |

| `ErrcObservation` | struct |  |  |  |  |  |

| `ErrcReceipt` | struct |  |  |  |  |  |

| `ErrcSource` | struct |  |  |  |  |  |

| `PreservedInvariant` | struct |  |  |  |  |  |

| `QuadrantCounts` | struct |  |  |  |  |  |

| `ErrcClaimAssuranceRefusal` | enum |  |  |  |  |  |

| `certify_errc_claim_assurance` | function | certify_errc_claim_assurance(
    parent: &ErrcReceipt,
    mut witnesses: Vec<ErrcClaimWitness>,
) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `verify_against` | function | verify_against(&self, parent: &ErrcReceipt) |  |  |  |  |

| `ErrcClaimAssuranceReceipt` | struct |  |  |  |  |  |

| `ErrcClaimWitness` | struct |  |  |  |  |  |

| `AffidavitError` | enum |  |  |  |  |  |

| `ChainError` | enum |  |  |  |  |  |

| `MiningError` | enum |  |  |  |  |  |

| `OcelError` | enum |  |  |  |  |  |

| `PqcError` | enum |  |  |  |  |  |

| `PredictionError` | enum |  |  |  |  |  |

| `ShardingError` | enum |  |  |  |  |  |

| `SloViolation` | enum |  |  |  |  |  |

| `build` | function | build(self, counter: &mut SeqCounter) |  |  |  |  |

| `new` | function | new(event_type: impl Into<String>) |  |  |  |  |

| `object` | function | object(mut self, id: impl Into<String>, object_type: impl Into<String>) |  |  |  |  |

| `payload` | function | payload(mut self, payload: impl Into<Vec<u8>>) |  |  |  |  |

| `payload_str` | function | payload_str(mut self, payload: impl Into<String>) |  |  |  |  |

| `qualified_object` | function | qualified_object(
        mut self,
        id: impl Into<String>,
        object_type: impl Into<String>,
        qualifier: impl Into<String>,
    ) |  |  |  |  |

| `EventBuilder` | struct |  |  |  |  |  |

| `ManifestRefusal` | enum |  |  |  |  |  |

| `binding` | function | binding(&self) |  |  |  |  |

| `digest` | function | digest(&self) |  |  |  |  |

| `requalification_reason` | function | requalification_reason(
    before: &ExecutionManifest,
    after: &ExecutionManifest,
) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `verify_binding` | function | verify_binding(
    manifest: &ExecutionManifest,
    binding: &ExecutionBinding,
) |  |  |  |  |

| `ExecutionBinding` | struct |  |  |  |  |  |

| `ExecutionManifest` | struct |  |  |  |  |  |

| `accepted` | function | accepted(&self) |  |  |  |  |

| `cli_standing_authority` | function | cli_standing_authority(scope: &str) |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(receipt: &str, observation: &str) |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: &str) |  |  |  |  |

| `errc_assure` | function | errc_assure(parent: &str, witnesses: &str) |  |  |  |  |

| `errc_certify` | function | errc_certify(receipt: &str, observation: &str) |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: &str) |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(receipt: &str, parent: Option<&str>) |  |  |  |  |

| `reason` | function | reason(&self) |  |  |  |  |

| `standing_certify` | function | standing_certify(receipt: &str, observation: &str, scope: &str) |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: &str) |  |  |  |  |

| `CourtOutcome` | struct |  |  |  |  |  |

| `all` | function | all(&self) |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `reindex` | function | reindex(&mut self) |  |  |  |  |

| `save` | function | save(&self) |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) |  |  |  |  |

| `Fixture` | struct |  |  |  |  |  |

| `FixtureDatabase` | struct |  |  |  |  |  |

| `FixtureQuery` | struct |  |  |  |  |  |

| `CrownRefusal` | enum |  |  |  |  |  |

| `CrownStanding` | enum |  |  |  |  |  |

| `GateStatus` | enum |  |  |  |  |  |

| `certify_gall_crown` | function | certify_gall_crown(manifest: &CrownManifest) |  |  |  |  |

| `CrownManifest` | struct |  |  |  |  |  |

| `CrownReceipt` | struct |  |  |  |  |  |

| `GateWitness` | struct |  |  |  |  |  |

| `PredecessorWitness` | struct |  |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) |  |  |  |  |

| `from_json` | function | from_json(json: &str) |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, 
        pattern_name: &str, 
        events: Vec<serde_json::Value>, 
        expected_verdict: &str,
        expected_failure_stage: Option<&str>
    ) |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) |  |  |  |  |

| `main` | function | main() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `CodegenEngine` | struct |  |  |  |  |  |

| `Snippet` | struct |  |  |  |  |  |

| `SnippetRegistry` | struct |  |  |  |  |  |

| `CheckStatus` | enum |  |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect(
    receipts_path: String,
    sensitivity: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble` | function | assemble(out: Option<String>, format: Option<String>) |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize(
    notary_provider: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature(
    signing_method: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `attest` | function | attest(
    receipt: String,
    attestation_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `audit` | function | audit() |  |  |  |  |

| `bus_factor` | function | bus_factor(receipts_path: String, format: Option<String>) |  |  |  |  |

| `catalog` | function | catalog(filter_name: Option<String>, filter_events: Option<usize>) |  |  |  |  |

| `causality_chain` | function | causality_chain(
    start_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `conformance` | function | conformance(receipt: String) |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix(
    receipts_path: String,
    output_matrix: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `diagnose` | function | diagnose(receipt: String) |  |  |  |  |

| `diff` | function | diff(receipt_a: String, receipt_b: String, format: Option<String>) |  |  |  |  |

| `doctor` | function | doctor(receipts: Option<String>, fix: bool) |  |  |  |  |

| `dora_metrics` | function | dora_metrics(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `emit` | function | emit(r#type: String, object: String, payload: String, format: Option<String>) |  |  |  |  |

| `emit_batch` | function | emit_batch(batch_file: String, format: Option<String>) |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd(provider: String, job_status: String, format: Option<String>) |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud(
    provider: String,
    resource_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_github` | function | emit_from_github(repo: String, event_type: String, format: Option<String>) |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab(repo: String, event_type: String, format: Option<String>) |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring(
    provider: String,
    alert_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_quality` | function | emit_from_quality(working_dir: Option<String>, format: Option<String>) |  |  |  |  |

| `emit_from_security` | function | emit_from_security(
    provider: String,
    vuln_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_ocel_quality_measurement` | function | emit_ocel_quality_measurement(
    working_dir: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `emit_ocel_quality_violation` | function | emit_ocel_quality_violation(
    working_dir: Option<String>,
    baseline_commits: Option<u32>,
    format: Option<String>,
    rules: Option<String>,
) |  |  |  |  |

| `emit_violation_causal_chain` | function | emit_violation_causal_chain(
    receipt_path: String,
    metric_filter: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `envelope_export` | function | envelope_export(sealed_file: String, format: Option<String>) |  |  |  |  |

| `envelope_list` | function | envelope_list(store: Option<String>) |  |  |  |  |

| `envelope_sign` | function | envelope_sign(receipt: String, key_file: String, out: Option<String>) |  |  |  |  |

| `envelope_verify` | function | envelope_verify(
    sealed_file: String,
    store: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_assure` | function | errc_assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_certify` | function | errc_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply(file: String, store: Option<String>) |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish(
    kid: String,
    epoch: u64,
    store: Option<String>,
    out: Option<String>,
) |  |  |  |  |

| `evidence_heads` | function | evidence_heads(journal_file: Option<String>) |  |  |  |  |

| `evidence_journal` | function | evidence_journal(subject: String, out: Option<String>) |  |  |  |  |

| `explain_incident` | function | explain_incident(
    incident_desc: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius(
    change_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `fix_receipt` | function | fix_receipt(
    receipt: String,
    action: Option<String>,
    dry_run: bool,
    format: Option<String>,
) |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `graph` | function | graph(receipt: String, format: Option<String>) |  |  |  |  |

| `guide_search` | function | guide_search(keyword: String, format: Option<String>) |  |  |  |  |

| `hipaa` | function | hipaa(receipts_path: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `inspect` | function | inspect(receipt: String, format: Option<String>) |  |  |  |  |

| `install_git_hook` | function | install_git_hook(threshold: Option<String>) |  |  |  |  |

| `keys_generate` | function | keys_generate(algorithm: String, custodian: String, out: Option<String>) |  |  |  |  |

| `keys_import` | function | keys_import(
    algorithm: String,
    public_key_hex: String,
    custodian: String,
    out: Option<String>,
) |  |  |  |  |

| `keys_list` | function | keys_list(store: Option<String>) |  |  |  |  |

| `keys_revoke` | function | keys_revoke(kid: String, reason: String, store: Option<String>) |  |  |  |  |

| `keys_rotate` | function | keys_rotate(kid: String, store: Option<String>, out: Option<String>) |  |  |  |  |

| `license_compliance` | function | license_compliance(
    receipts_path: String,
    license_policy: String,
    format: Option<String>,
) |  |  |  |  |

| `model` | function | model(receipt: String) |  |  |  |  |

| `monitor` | function | monitor(
    watch: Option<String>,
    _metrics: Option<String>,
    _rules: Option<String>,
    baseline_commits: Option<u32>,
    interval: Option<u64>,
    output: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `notarize` | function | notarize(receipt: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `orphaned_code` | function | orphaned_code(
    receipts_path: String,
    days: Option<u32>,
    format: Option<String>,
) |  |  |  |  |

| `pci_dss` | function | pci_dss(receipts_path: String, out: Option<String>, format: Option<String>) |  |  |  |  |

| `policy_enforce` | function | policy_enforce(
    receipts_path: String,
    policy_file: String,
    format: Option<String>,
) |  |  |  |  |

| `portfolio_health` | function | portfolio_health(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `predict` | function | predict(
    receipts_path: String,
    prediction_type: String,
    _model: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `profile` | function | profile(receipt: Option<String>, duration: Option<u64>) |  |  |  |  |

| `query` | function | query(q: String, receipts_path: String, format: Option<String>) |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput(iterations: Option<u32>) |  |  |  |  |

| `replay` | function | replay(receipt: String) |  |  |  |  |

| `root_cause` | function | root_cause(
    effect_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_attest` | function | sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius(
    sbom_path: String,
    component: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance(
    sbom_path: String,
    framework: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_emit` | function | sbom_emit(sbom_path: String, format: Option<String>) |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia(sbom_path: String, format: Option<String>) |  |  |  |  |

| `sbom_scan` | function | sbom_scan(
    sbom_path: String,
    advisories_path: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `search` | function | search(pattern: String, receipts_path: String, format: Option<String>) |  |  |  |  |

| `security_debt` | function | security_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `send_violation_webhook` | function | send_violation_webhook(
    violation: &crate::quality::QualityViolation,
    webhook_url: &str,
) |  |  |  |  |

| `show` | function | show(receipt: String, format: Option<String>) |  |  |  |  |

| `sign` | function | sign(
    receipt: String,
    key_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `soc2_audit` | function | soc2_audit(
    receipts_path: String,
    soc2_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_certify` | function | standing_certify(
    receipt: String,
    observation: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: String, format: Option<String>) |  |  |  |  |

| `stats` | function | stats(receipt: String, format: Option<String>) |  |  |  |  |

| `team_velocity` | function | team_velocity(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `tech_debt` | function | tech_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `test` | function | test() |  |  |  |  |

| `timeline` | function | timeline(
    receipts_path: String,
    start_time: Option<String>,
    end_time: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `trend_analysis` | function | trend_analysis(
    receipts_path: String,
    metric: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `variance` | function | variance(receipt: Option<String>, iterations: Option<u32>) |  |  |  |  |

| `verify` | function | verify(
    receipt: String,
    format: Option<String>,
    _profile: Option<String>,
    _strict: Option<bool>,
) |  |  |  |  |

| `verify_compliance` | function | verify_compliance(receipt: String, framework: String, format: Option<String>) |  |  |  |  |

| `verify_family` | function | verify_family(receipts_dir: String, format: Option<String>) |  |  |  |  |

| `verify_sla` | function | verify_sla(receipt: String, sla_file: String, format: Option<String>) |  |  |  |  |

| `visualize` | function | visualize(format: String, receipt: String) |  |  |  |  |

| `why` | function | why(receipt: String, format: Option<String>) |  |  |  |  |

| `DoctorFinding` | struct |  |  |  |  |  |

| `RevocationSidecarEntry` | struct |  |  |  |  |  |

| `HlcError` | enum |  |  |  |  |  |

| `concurrent` | function | concurrent(a: HlcTimestamp, b: HlcTimestamp) |  |  |  |  |

| `happened_before` | function | happened_before(a: HlcTimestamp, b: HlcTimestamp) |  |  |  |  |

| `peek` | function | peek(&self) |  |  |  |  |

| `receive` | function | receive(&mut self, received: HlcTimestamp) |  |  |  |  |

| `send` | function | send(&mut self) |  |  |  |  |

| `with_skew_bound` | function | with_skew_bound(max_skew_ms: u64) |  |  |  |  |

| `HlcClock` | struct |  |  |  |  |  |

| `HlcTimestamp` | struct |  |  |  |  |  |

| `run` | function | run() |  |  |  |  |

| `verdict_to_diagnostics` | function | verdict_to_diagnostics(verdict: &crate::types::Verdict) |  |  |  |  |

| `goto_definition_for_event_type` | function | goto_definition_for_event_type(event_type: &str) |  |  |  |  |

| `hover_for_event_id` | function | hover_for_event_id(event_id: &str, receipt: &Receipt) |  |  |  |  |

| `SloViolation` | enum |  |  |  |  |  |

| `check_slo` | function | check_slo(&self) |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `new_noop` | function | new_noop() |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) |  |  |  |  |

| `MetricsCollector` | struct |  |  |  |  |  |

| `PrometheusExporter` | struct |  |  |  |  |  |

| `ServiceLevelIndicators` | struct |  |  |  |  |  |

| `MiningError` | enum |  |  |  |  |  |

| `alignment_fitness_score` | function | alignment_fitness_score(
    admitted: &AdmittedReceipt,
    model: &PetriNet,
) |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `interpret_score` | function | interpret_score(fitness: f64) |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `AlignmentReport` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |

| `MiningError` | enum |  |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `build_event` | function | build_event(
    event_type: impl Into<String>,
    objects: Vec<ObjectRef>,
    payload: &[u8],
    counter: &mut SeqCounter,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `next_seq` | function | next_seq(&mut self) |  |  |  |  |

| `object_ref` | function | object_ref(id: impl Into<String>, obj_type: impl Into<String>) |  |  |  |  |

| `parse_object_ref` | function | parse_object_ref(spec: &str) |  |  |  |  |

| `peek` | function | peek(&self) |  |  |  |  |

| `qualified_object_ref` | function | qualified_object_ref(
    id: impl Into<String>,
    obj_type: impl Into<String>,
    qualifier: impl Into<String>,
) |  |  |  |  |

| `starting_at` | function | starting_at(value: u64) |  |  |  |  |

| `validate_event` | function | validate_event(event: &OperationEvent) |  |  |  |  |

| `SeqCounter` | struct |  |  |  |  |  |

| `Format` | enum |  |  |  |  |  |

| `diag` | function | diag(&mut self, diag: &crate::diag::Diag) |  |  |  |  |

| `from_str` | function | from_str(s: &str) |  |  |  |  |

| `info` | function | info(&mut self, msg: &str) |  |  |  |  |

| `json` | function | json(&mut self, value: &serde_json::Value) |  |  |  |  |

| `line` | function | line(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `new` | function | new(format: Format) |  |  |  |  |

| `no_color` | function | no_color(mut self) |  |  |  |  |

| `print_` | function | print_(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `quiet` | function | quiet(mut self) |  |  |  |  |

| `verbose` | function | verbose(mut self) |  |  |  |  |

| `warn` | function | warn(&mut self, msg: &str) |  |  |  |  |

| `with_format` | function | with_format(format: Format) |  |  |  |  |

| `with_sinks` | function | with_sinks(
        format: Format,
        stdout: Box<dyn Write + Send>,
        stderr: Box<dyn Write + Send>,
    ) |  |  |  |  |

| `Out` | struct |  |  |  |  |  |

| `GateVerdict` | enum |  |  |  |  |  |

| `PolicyError` | enum |  |  |  |  |  |

| `evaluate` | function | evaluate(
        &self,
        principal: &str,
        action: &str,
        resource: &str,
    ) |  |  |  |  |

| `from_policies` | function | from_policies(policies: &str) |  |  |  |  |

| `PolicyGate` | struct |  |  |  |  |  |

| `verify_foreign_receipt` | function | verify_foreign_receipt(
    consequence_digest: &str,
    authority: ForeignAuthority<'_>,
    observation: ForeignObservation<'_>,
    receipt: ForeignReceipt<'_>,
) |  |  |  |  |

| `ForeignAuthority` | struct |  |  |  |  |  |

| `ForeignObservation` | struct |  |  |  |  |  |

| `ForeignReceipt` | struct |  |  |  |  |  |

| `PredictionError` | enum |  |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model(
    model: &AdmittedReceipt,
    current_trace: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |

| `Notification` | enum |  |  |  |  |  |

| `QualityViolation` | enum |  |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `another_good` | function | another_good() |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `feature1` | function | feature1() |  |  |  |  |

| `feature2` | function | feature2() |  |  |  |  |

| `feature3` | function | feature3() |  |  |  |  |

| `good_feature` | function | good_feature() |  |  |  |  |

| `measure_code_quality` | function | measure_code_quality(src_path: &str) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64, window_size: usize) |  |  |  |  |

| `run_watch_loop` | function | run_watch_loop(&mut self) |  |  |  |  |

| `run_watch_loop_async` | function | run_watch_loop_async(
        path: &str,
        interval_secs: u64,
    ) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `CodeQualityMetrics` | struct |  |  |  |  |  |

| `FileWatcher` | struct |  |  |  |  |  |

| `WesternElectricAnalyzer` | struct |  |  |  |  |  |

| `amplify_severity_for_correlated_violations` | function | amplify_severity_for_correlated_violations(
    violations: &[QualityViolation],
    correlations: &[MetricCorrelation],
) |  |  |  |  |

| `analyze_correlations` | function | analyze_correlations(
    history: &[CodeQualityMetrics],
    violations: &[QualityViolation],
) |  |  |  |  |

| `compute_metric_correlations` | function | compute_metric_correlations(history: &[CodeQualityMetrics]) |  |  |  |  |

| `detect_simultaneous_violations` | function | detect_simultaneous_violations(
    violations: &[QualityViolation],
) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `infer_root_cause` | function | infer_root_cause(
    metrics: &[CodeQualityMetrics],
    violation: &QualityViolation,
) |  |  |  |  |

| `is_actionable` | function | is_actionable(&self) |  |  |  |  |

| `is_compound` | function | is_compound(&self) |  |  |  |  |

| `is_significant` | function | is_significant(&self) |  |  |  |  |

| `max_severity` | function | max_severity(&self) |  |  |  |  |

| `new` | function | new(
        metric_names: Vec<String>,
        severities: Vec<String>,
        timestamps: Vec<u64>,
        time_window_secs: u64,
    ) |  |  |  |  |

| `strength` | function | strength(&self) |  |  |  |  |

| `CorrelationAnalysis` | struct |  |  |  |  |  |

| `MetricCorrelation` | struct |  |  |  |  |  |

| `RootCauseHypothesis` | struct |  |  |  |  |  |

| `SimultaneousViolation` | struct |  |  |  |  |  |

| `RuleVariant` | enum |  |  |  |  |  |

| `add_custom_threshold` | function | add_custom_threshold(mut self, rule_name: String, threshold: f64) |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `compute` | function | compute(violations: &[QualityViolation]) |  |  |  |  |

| `compute_aggregate_severity` | function | compute_aggregate_severity(violations: &[QualityViolation]) |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `detect_all_rule_variants` | function | detect_all_rule_variants(
    metrics: &[f64],
    config: &WesternElectricConfig,
) |  |  |  |  |

| `detect_rule_storms` | function | detect_rule_storms(violations: &[QualityViolation]) |  |  |  |  |

| `finalize` | function | finalize(&mut self) |  |  |  |  |

| `metric` | function | metric(&self) |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `with_enabled_rules` | function | with_enabled_rules(
        mut self,
        rule1: bool,
        rule2: bool,
        rule3: bool,
        rule4: bool,
        rule5: bool,
        rule6: bool,
        rule7: bool,
    ) |  |  |  |  |

| `with_sigmas` | function | with_sigmas(mut self, primary: f64, secondary: f64, tertiary: f64) |  |  |  |  |

| `AggregatedSeverity` | struct |  |  |  |  |  |

| `EnhancedWesternElectricAnalyzer` | struct |  |  |  |  |  |

| `RuleStorm` | struct |  |  |  |  |  |

| `WesternElectricConfig` | struct |  |  |  |  |  |

| `ObjectViolation` | enum |  |  |  |  |  |

| `aggregate_module_metrics` | function | aggregate_module_metrics(files: &[FileQualityMetrics]) |  |  |  |  |

| `complex_fn` | function | complex_fn(x: i32) |  |  |  |  |

| `compute_package_health` | function | compute_package_health(modules: &[ModuleQualityMetrics]) |  |  |  |  |

| `description` | function | description(&self) |  |  |  |  |

| `detect_object_level_violations` | function | detect_object_level_violations(
    object_metric: &FileQualityMetrics,
    baseline: f64,
    stddev: f64,
) |  |  |  |  |

| `documented` | function | documented() |  |  |  |  |

| `good_fn` | function | good_fn() |  |  |  |  |

| `maintainability_index` | function | maintainability_index(&self) |  |  |  |  |

| `measure_file_quality` | function | measure_file_quality(path: &str) |  |  |  |  |

| `new` | function | new(path: String) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `typed_fn` | function | typed_fn(x: i32, y: String) |  |  |  |  |

| `undocumented` | function | undocumented() |  |  |  |  |

| `untyped_fn` | function | untyped_fn() |  |  |  |  |

| `update_health_score` | function | update_health_score(&mut self) |  |  |  |  |

| `FileQualityMetrics` | struct |  |  |  |  |  |

| `ModuleQualityMetrics` | struct |  |  |  |  |  |

| `PackageHealthScore` | struct |  |  |  |  |  |

| `add_event` | function | add_event(&mut self, event: OcelQualityEvent) |  |  |  |  |

| `build_causal_chain` | function | build_causal_chain(
    violation_event: &OcelQualityEvent,
    event_log: &[OcelQualityEvent],
) |  |  |  |  |

| `correlate_violations_across_objects` | function | correlate_violations_across_objects(
    log: &OcelQualityLog,
) |  |  |  |  |

| `events_by_type` | function | events_by_type(&self, quality_event_type: &str) |  |  |  |  |

| `from_metrics` | function | from_metrics(
        object_id: impl Into<String>,
        object_type: impl Into<String>,
        metrics: &CodeQualityMetrics,
    ) |  |  |  |  |

| `measure_to_ocel_event` | function | measure_to_ocel_event(
    event_id: &str,
    seq: u64,
    metrics: &CodeQualityMetrics,
    objects: &[ObjectRef],
) |  |  |  |  |

| `measurements` | function | measurements(&self) |  |  |  |  |

| `new` | function | new(log_id: impl Into<String>, timestamp: u64) |  |  |  |  |

| `violation_to_ocel_event` | function | violation_to_ocel_event(
    event_id: &str,
    seq: u64,
    violation: &QualityViolation,
    triggered_by_event_id: &str,
    objects: &[ObjectRef],
) |  |  |  |  |

| `violations` | function | violations(&self) |  |  |  |  |

| `ObjectCorrelation` | struct |  |  |  |  |  |

| `ObjectQualityRecord` | struct |  |  |  |  |  |

| `OcelQualityEvent` | struct |  |  |  |  |  |

| `OcelQualityLog` | struct |  |  |  |  |  |

| `ViolationCausalChain` | struct |  |  |  |  |  |

| `DimensionError` | enum |  |  |  |  |  |

| `MatrixError` | enum |  |  |  |  |  |

| `QuantizationRefusal` | enum |  |  |  |  |  |

| `bits` | function | bits(&self) |  |  |  |  |

| `from_fractions` | function | from_fractions(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<(u64, u64) |  |  |  |  |

| `from_scores` | function | from_scores(
        n_nodes: usize,
        lenses: usize,
        rows: &[Vec<f64>],
    ) |  |  |  |  |

| `lenses` | function | lenses(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `quantize` | function | quantize(score: f64) |  |  |  |  |

| `quantize_fraction` | function | quantize_fraction(num: u64, den: u64) |  |  |  |  |

| `zeroed` | function | zeroed(n_nodes: usize, lenses: usize) |  |  |  |  |

| `PayoffMatrix` | struct |  |  |  |  |  |

| `build_canonical_subject` | function | build_canonical_subject(payload_hash_hex: &str, subject: &str) |  |  |  |  |

| `certify_paid_delivery_payload` | function | certify_paid_delivery_payload(
    payload_hash_hex: &str,
    subject: &str,
    signing: &Es256SigningKey,
) |  |  |  |  |

| `verify_certified_paid_delivery` | function | verify_certified_paid_delivery(
    certified: &CertifiedReceiptEnvelope,
    payload_hash_hex: &str,
    subject: &str,
) |  |  |  |  |

| `CertifiedReceiptEnvelope` | struct |  |  |  |  |  |

| `VerbGroup` | enum |  |  |  |  |  |

| `by_group` | function | by_group(group: VerbGroup) |  |  |  |  |

| `description` | function | description(self) |  |  |  |  |

| `did_you_mean` | function | did_you_mean(input: &str) |  |  |  |  |

| `label` | function | label(self) |  |  |  |  |

| `lookup` | function | lookup(verb: &str, noun: &str) |  |  |  |  |

| `new` | function | new(
        verb: &'static str,
        noun: &'static str,
        group: VerbGroup,
        summary: &'static str,
        keywords: &'static [&'static str],
    ) |  |  |  |  |

| `search` | function | search(query: &str) |  |  |  |  |

| `verb_count` | function | verb_count() |  |  |  |  |

| `with_example` | function | with_example(mut self, example: &'static str) |  |  |  |  |

| `VerbEntry` | struct |  |  |  |  |  |

| `ReplayFilterError` | enum |  |  |  |  |  |

| `capacity` | function | capacity(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(capacity: usize) |  |  |  |  |

| `probably_seen` | function | probably_seen(&self, id: &[u8; 32]) |  |  |  |  |

| `retract` | function | retract(&mut self, id: &[u8; 32]) |  |  |  |  |

| `witness` | function | witness(&mut self, id: &[u8; 32]) |  |  |  |  |

| `ReplayFilter` | struct |  |  |  |  |  |

| `ComponentType` | enum |  |  |  |  |  |

| `SbomError` | enum |  |  |  |  |  |

| `SbomFormat` | enum |  |  |  |  |  |

| `canonicalize` | function | canonicalize(&mut self) |  |  |  |  |

| `component` | function | component(&self, bom_ref: &str) |  |  |  |  |

| `content_address` | function | content_address(&self) |  |  |  |  |

| `detect_format` | function | detect_format(doc: &serde_json::Value) |  |  |  |  |

| `expr` | function | expr(expression: impl Into<String>) |  |  |  |  |

| `family` | function | family(&self) |  |  |  |  |

| `has_unique_identifier` | function | has_unique_identifier(&self) |  |  |  |  |

| `id` | function | id(spdx_id: impl Into<String>) |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `library` | function | library(
        bom_ref: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) |  |  |  |  |

| `license_labels` | function | license_labels(&self) |  |  |  |  |

| `missing` | function | missing(&self) |  |  |  |  |

| `new` | function | new(algorithm: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `ntia_minimum_elements` | function | ntia_minimum_elements(&self) |  |  |  |  |

| `parse` | function | parse(s: &str) |  |  |  |  |

| `parse_cyclonedx` | function | parse_cyclonedx(doc: &serde_json::Value) |  |  |  |  |

| `parse_sbom_json` | function | parse_sbom_json(json: &str) |  |  |  |  |

| `parse_spdx` | function | parse_spdx(doc: &serde_json::Value) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, root: &str) |  |  |  |  |

| `Component` | struct |  |  |  |  |  |

| `Dependency` | struct |  |  |  |  |  |

| `Hash` | struct |  |  |  |  |  |

| `License` | struct |  |  |  |  |  |

| `NtiaMinimumElements` | struct |  |  |  |  |  |

| `Sbom` | struct |  |  |  |  |  |

| `SbomMetadata` | struct |  |  |  |  |  |

| `Supplier` | struct |  |  |  |  |  |

| `Tool` | struct |  |  |  |  |  |

| `bundle` | function | bundle(&self) |  |  |  |  |

| `cleanup` | function | cleanup(mut self) |  |  |  |  |

| `create_bundle` | function | create_bundle(
        &self,
        sbom_id: &str,
        sbom_format: &str,
        description: Option<String>,
    ) |  |  |  |  |

| `load_bundle` | function | load_bundle(&self, path: PathBuf) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `redact_sensitive` | function | redact_sensitive(&self, value: &str) |  |  |  |  |

| `save_bundle` | function | save_bundle(&self, bundle: &SbomForensicsBundle, path: PathBuf) |  |  |  |  |

| `take_bundle` | function | take_bundle(mut self) |  |  |  |  |

| `with_path` | function | with_path(mut self, path: PathBuf) |  |  |  |  |

| `with_redaction` | function | with_redaction(mut self, redact: bool) |  |  |  |  |

| `AttestationData` | struct |  |  |  |  |  |

| `BundleMetadata` | struct |  |  |  |  |  |

| `ComplianceRecord` | struct |  |  |  |  |  |

| `ComponentNode` | struct |  |  |  |  |  |

| `RiskPropagationRecord` | struct |  |  |  |  |  |

| `SbomArtifactCollector` | struct |  |  |  |  |  |

| `SbomArtifactGuard` | struct |  |  |  |  |  |

| `SbomForensicsBundle` | struct |  |  |  |  |  |

| `SupplyChainGraph` | struct |  |  |  |  |  |

| `VulnerabilityRecord` | struct |  |  |  |  |  |

| `ComplianceError` | enum |  |  |  |  |  |

| `Framework` | enum |  |  |  |  |  |

| `SlsaLevel` | enum |  |  |  |  |  |

| `assess_all` | function | assess_all(sbom: &Sbom) |  |  |  |  |

| `assess_slsa` | function | assess_slsa(sbom: &Sbom) |  |  |  |  |

| `check_cisa` | function | check_cisa(sbom: &Sbom) |  |  |  |  |

| `check_cscrm` | function | check_cscrm(sbom: &Sbom) |  |  |  |  |

| `check_eo_14028` | function | check_eo_14028(sbom: &Sbom) |  |  |  |  |

| `check_in_toto` | function | check_in_toto(sbom: &Sbom) |  |  |  |  |

| `check_iso_27001` | function | check_iso_27001(sbom: &Sbom) |  |  |  |  |

| `check_ntia` | function | check_ntia(sbom: &Sbom) |  |  |  |  |

| `check_slsa` | function | check_slsa(sbom: &Sbom) |  |  |  |  |

| `check_soc2` | function | check_soc2(sbom: &Sbom) |  |  |  |  |

| `display_name` | function | display_name(&self) |  |  |  |  |

| `rank` | function | rank(&self) |  |  |  |  |

| `requirement_count` | function | requirement_count(&self) |  |  |  |  |

| `score` | function | score(&self) |  |  |  |  |

| `supported_frameworks` | function | supported_frameworks() |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `vex_readiness` | function | vex_readiness(sbom: &Sbom) |  |  |  |  |

| `ComplianceResult` | struct |  |  |  |  |  |

| `SbomOcelError` | enum |  |  |  |  |  |

| `build_sbom_causal_chain` | function | build_sbom_causal_chain(
    events: &[SbomOcelEvent],
    sbom: &Sbom,
    root_bom_ref: &str,
) |  |  |  |  |

| `component_object_type` | function | component_object_type(component: &Component) |  |  |  |  |

| `correlate_components_by_license` | function | correlate_components_by_license(
    events: &[SbomOcelEvent],
    sbom: &Sbom,
) |  |  |  |  |

| `license_object_type` | function | license_object_type(label: &str) |  |  |  |  |

| `sbom_to_ocel_events` | function | sbom_to_ocel_events(
    sbom: &Sbom,
    counter: &mut SeqCounter,
) |  |  |  |  |

| `supplier_object_type` | function | supplier_object_type(name: &str) |  |  |  |  |

| `supported_event_types` | function | supported_event_types() |  |  |  |  |

| `vulnerability_object_type` | function | vulnerability_object_type(id: &str) |  |  |  |  |

| `ObjectCorrelation` | struct |  |  |  |  |  |

| `SbomCausalChain` | struct |  |  |  |  |  |

| `SbomOcelEvent` | struct |  |  |  |  |  |

| `SupplyChainError` | enum |  |  |  |  |  |

| `attest_provenance` | function | attest_provenance(sbom: &Sbom, receipt_ref: Option<&str>) |  |  |  |  |

| `blast_radius` | function | blast_radius(
    graph: &DependencyGraph,
    bom_ref: &str,
) |  |  |  |  |

| `build_report` | function | build_report(sbom: &Sbom, spof_threshold: usize) |  |  |  |  |

| `contains` | function | contains(&self, bom_ref: &str) |  |  |  |  |

| `depth` | function | depth(&self, root: &str) |  |  |  |  |

| `direct_dependencies` | function | direct_dependencies(&self, bom_ref: &str) |  |  |  |  |

| `direct_dependents` | function | direct_dependents(&self, bom_ref: &str) |  |  |  |  |

| `edge_count` | function | edge_count(&self) |  |  |  |  |

| `from_sbom` | function | from_sbom(sbom: &Sbom) |  |  |  |  |

| `is_cyclic` | function | is_cyclic(&self) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `single_points_of_failure` | function | single_points_of_failure(
    graph: &DependencyGraph,
    _sbom: &Sbom,
    threshold: usize,
) |  |  |  |  |

| `supplier_concentration` | function | supplier_concentration(sbom: &Sbom) |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, bom_ref: &str) |  |  |  |  |

| `transitive_dependents` | function | transitive_dependents(&self, bom_ref: &str) |  |  |  |  |

| `BlastRadius` | struct |  |  |  |  |  |

| `DependencyGraph` | struct |  |  |  |  |  |

| `ProvenanceAttestation` | struct |  |  |  |  |  |

| `SupplierConcentration` | struct |  |  |  |  |  |

| `SupplyChainReport` | struct |  |  |  |  |  |

| `Severity` | enum |  |  |  |  |  |

| `VexStatus` | enum |  |  |  |  |  |

| `VulnerabilityError` | enum |  |  |  |  |  |

| `apply_vex` | function | apply_vex(matches: &[VulnerabilityMatch], vex: &[VexStatement]) |  |  |  |  |

| `build_report` | function | build_report(
    sbom: &Sbom,
    vulns: &[Vulnerability],
    vex: &[VexStatement],
) |  |  |  |  |

| `cvss_band` | function | cvss_band(score: f64) |  |  |  |  |

| `from_cvss` | function | from_cvss(score: f64) |  |  |  |  |

| `from_score` | function | from_score(base_score: f64) |  |  |  |  |

| `match_vulnerabilities` | function | match_vulnerabilities(sbom: &Sbom, vulns: &[Vulnerability]) |  |  |  |  |

| `new` | function | new(id: impl Into<String>, base_score: f64) |  |  |  |  |

| `propagate_risk` | function | propagate_risk(
    sbom: &Sbom,
    matches: &[VulnerabilityMatch],
) |  |  |  |  |

| `severity` | function | severity(&self) |  |  |  |  |

| `suppresses` | function | suppresses(&self) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `with_vector` | function | with_vector(base_score: f64, vector: impl Into<String>) |  |  |  |  |

| `CvssVector` | struct |  |  |  |  |  |

| `VexStatement` | struct |  |  |  |  |  |

| `Vulnerability` | struct |  |  |  |  |  |

| `VulnerabilityMatch` | struct |  |  |  |  |  |

| `VulnerabilityReport` | struct |  |  |  |  |  |

| `Secp256k1WitnessError` | enum |  |  |  |  |  |

| `WitnessSigningError` | enum |  |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) |  |  |  |  |

| `verify_bip340` | function | verify_bip340(
    x_only_public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) |  |  |  |  |

| `verify_bip340_raw` | function | verify_bip340_raw(
    x_only_public_key: &[u8; 32],
    message: &[u8; 32],
    signature: &[u8; 64],
) |  |  |  |  |

| `verify_ecdsa` | function | verify_ecdsa(
    compressed_sec1_public_key: &[u8; 33],
    message: &[u8],
    signature: &[u8],
) |  |  |  |  |

| `WitnessSigningKey` | struct |  |  |  |  |  |

| `SeqBitmapError` | enum |  |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `has` | function | has(&self, seq: u32) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `record` | function | record(&mut self, seq: u32) |  |  |  |  |

| `verify_contiguous` | function | verify_contiguous(&self) |  |  |  |  |

| `SeqContiguityCertifier` | struct |  |  |  |  |  |

| `SmtError` | enum |  |  |  |  |  |

| `common_prefix_len` | function | common_prefix_len(
    a: &BitSlice<u8, bitvec::prelude::Msb0>,
    b: &BitSlice<u8, bitvec::prelude::Msb0>,
) |  |  |  |  |

| `get` | function | get(&mut self, key: &StateKey) |  |  |  |  |

| `insert` | function | insert(&mut self, key: &StateKey, value: &StateValue) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `prove_absence` | function | prove_absence(&mut self, key: &StateKey) |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&mut self, key: &StateKey) |  |  |  |  |

| `remove` | function | remove(&mut self, key: &StateKey) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `verify_absence` | function | verify_absence(
    root: &StateRoot,
    neighbor_value: &StateValue,
    witness: &AbsenceWitness,
) |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(
    root: &StateRoot,
    expected_value: &StateValue,
    proof: &InclusionProof,
) |  |  |  |  |

| `AbsenceWitness` | struct |  |  |  |  |  |

| `InclusionProof` | struct |  |  |  |  |  |

| `StateTree` | struct |  |  |  |  |  |

| `Standing` | enum |  |  |  |  |  |

| `StandingRefusal` | enum |  |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `AuthorityBinding` | struct |  |  |  |  |  |

| `ExecutionEvidence` | struct |  |  |  |  |  |

| `ReplayEvidence` | struct |  |  |  |  |  |

| `StandingObservation` | struct |  |  |  |  |  |

| `StandingReceipt` | struct |  |  |  |  |  |

| `SubjectIdentity` | struct |  |  |  |  |  |

| `VerificationEvidence` | struct |  |  |  |  |  |

| `QuorumError` | enum |  |  |  |  |  |

| `aggregate_signature` | function | aggregate_signature(
    package: &SigningPackage,
    shares: &[(ParticipantId, frost::round2::SignatureShare) |  |  |  |  |

| `round2_sign` | function | round2_sign(
    secret_share: &SecretShare,
    nonces: &SigningNonces,
    package: &SigningPackage,
) |  |  |  |  |

| `signing_package` | function | signing_package(
    commitments: &[(ParticipantId, frost::round1::SigningCommitments) |  |  |  |  |

| `verify_quorum` | function | verify_quorum(
    group_public_key: &GroupPublicKey,
    message: &[u8],
    signature_bytes: &[u8; 64],
) |  |  |  |  |

| `DealtQuorum` | struct |  |  |  |  |  |

| `GroupPublicKey` | struct |  |  |  |  |  |

| `ParticipantId` | struct |  |  |  |  |  |

| `captured_spans` | function | captured_spans() |  |  |  |  |

| `clear_spans` | function | clear_spans() |  |  |  |  |

| `SpanRecord` | struct |  |  |  |  |  |

| `ProfileId` | enum |  |  |  |  |  |

| `as_hex` | function | as_hex(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8]) |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) |  |  |  |  |

| `AffidavitReceiptChain` | struct |  |  |  |  |  |

| `AssembleOutput` | struct |  |  |  |  |  |

| `Blake3Hash` | struct |  |  |  |  |  |

| `CheckOutcome` | struct |  |  |  |  |  |

| `EmitOutput` | struct |  |  |  |  |  |

| `EventSummary` | struct |  |  |  |  |  |

| `InspectionReport` | struct |  |  |  |  |  |

| `ObjectRef` | struct |  |  |  |  |  |

| `OperationEvent` | struct |  |  |  |  |  |

| `QualityMeasurement` | struct |  |  |  |  |  |

| `QualityMetricValue` | struct |  |  |  |  |  |

| `QualityViolationEvent` | struct |  |  |  |  |  |

| `Receipt` | struct |  |  |  |  |  |

| `StatsOutput` | struct |  |  |  |  |  |

| `Verdict` | struct |  |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect(
    receipts_path: String,
    sensitivity: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble` | function | assemble(
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize(
    notary_provider: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature(
    signing_method: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `assure` | function | assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `attest` | function | attest(
    #[arg(index = 1) |  |  |  |  |

| `audit` | function | audit(
) |  |  |  |  |

| `bus_factor` | function | bus_factor(
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `catalog` | function | catalog(
    filter_name: Option<String>,
    filter_events: Option<usize>,
) |  |  |  |  |

| `causality_chain` | function | causality_chain(
    start_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `conformance` | function | conformance(
    #[arg(index = 1) |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix(
    receipts_path: String,
    output_matrix: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `diagnose` | function | diagnose(
    #[arg(index = 1) |  |  |  |  |

| `diff` | function | diff(
    #[arg(index = 1) |  |  |  |  |

| `doctor` | function | doctor(
    receipts: Option<String>,
    fix: bool,
) |  |  |  |  |

| `dora_metrics` | function | dora_metrics(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |

| `emit` | function | emit(
    #[arg(alias = "type") |  |  |  |  |

| `emit_batch` | function | emit_batch(
    batch_file: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd(
    provider: String,
    job_status: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud(
    provider: String,
    resource_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_github` | function | emit_from_github(
    repo: String,
    event_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab(
    repo: String,
    event_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring(
    provider: String,
    alert_type: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_sbom` | function | emit_from_sbom(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |

| `emit_from_security` | function | emit_from_security(
    provider: String,
    vuln_type: String,
    format: Option<String>,
) |  |  |  |  |

| `envelope_export` | function | envelope_export(
    #[arg(index = 1) |  |  |  |  |

| `envelope_list` | function | envelope_list(
    store: Option<String>,
) |  |  |  |  |

| `envelope_sign` | function | envelope_sign(
    #[arg(index = 1) |  |  |  |  |

| `envelope_verify` | function | envelope_verify(
    #[arg(index = 1) |  |  |  |  |

| `errc_assure` | function | errc_assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_certify` | function | errc_certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `errc_verify` | function | errc_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply(
    #[arg(index = 1) |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish(
    #[arg(index = 1) |  |  |  |  |

| `evidence_heads` | function | evidence_heads(
    journal_file: Option<String>,
) |  |  |  |  |

| `evidence_journal` | function | evidence_journal(
    #[arg(index = 1) |  |  |  |  |

| `explain_incident` | function | explain_incident(
    incident_desc: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius(
    change_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `fix` | function | fix(
    receipt: String,
    action: Option<String>,
    dry_run: Option<bool>,
    format: Option<String>,
) |  |  |  |  |

| `fix_receipt` | function | fix_receipt(
    receipt: String,
    action: Option<String>,
    dry_run: bool,
    format: Option<String>,
) |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `graph` | function | graph(
    #[arg(index = 1) |  |  |  |  |

| `guide_search` | function | guide_search(
    keyword: String,
    format: Option<String>,
) |  |  |  |  |

| `hipaa` | function | hipaa(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `inspect` | function | inspect(
    #[arg(index = 1) |  |  |  |  |

| `install_git_hook` | function | install_git_hook(
    threshold: Option<String>,
) |  |  |  |  |

| `keys_generate` | function | keys_generate(
    #[arg(index = 1) |  |  |  |  |

| `keys_import` | function | keys_import(
    #[arg(index = 1) |  |  |  |  |

| `keys_list` | function | keys_list(
    store: Option<String>,
) |  |  |  |  |

| `keys_revoke` | function | keys_revoke(
    #[arg(index = 1) |  |  |  |  |

| `keys_rotate` | function | keys_rotate(
    #[arg(index = 1) |  |  |  |  |

| `license_compliance` | function | license_compliance(
    receipts_path: String,
    license_policy: String,
    format: Option<String>,
) |  |  |  |  |

| `model` | function | model(
    #[arg(index = 1) |  |  |  |  |

| `monitor` | function | monitor(
    watch: Option<String>,
    metrics: Option<String>,
    rules: Option<String>,
    baseline_commits: Option<u32>,
    interval: Option<u64>,
    output: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `notarize` | function | notarize(
    #[arg(index = 1) |  |  |  |  |

| `orphaned_code` | function | orphaned_code(
    receipts_path: String,
    days: Option<u32>,
    format: Option<String>,
) |  |  |  |  |

| `pci_dss` | function | pci_dss(
    receipts_path: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `policy_enforce` | function | policy_enforce(
    receipts_path: String,
    policy_file: String,
    format: Option<String>,
) |  |  |  |  |

| `portfolio_health` | function | portfolio_health(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `predict` | function | predict(
    receipts_path: String,
    prediction_type: String,
    model: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `profile` | function | profile(
    receipt: Option<String>,
    duration: Option<u64>,
) |  |  |  |  |

| `query` | function | query(
    q: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput(
    iterations: Option<u32>,
) |  |  |  |  |

| `replay` | function | replay(
    #[arg(index = 1) |  |  |  |  |

| `root_cause` | function | root_cause(
    effect_event: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_attest` | function | sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius(
    sbom_path: String,
    component: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance(
    sbom_path: String,
    framework: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `sbom_emit` | function | sbom_emit(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia(
    sbom_path: String,
    format: Option<String>,
) |  |  |  |  |

| `sbom_scan` | function | sbom_scan(
    sbom_path: String,
    advisories_path: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `search` | function | search(
    pattern: String,
    receipts_path: String,
    format: Option<String>,
) |  |  |  |  |

| `security_debt` | function | security_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `show` | function | show(
    #[arg(index = 1) |  |  |  |  |

| `sign` | function | sign(
    #[arg(index = 1) |  |  |  |  |

| `soc2_audit` | function | soc2_audit(
    receipts_path: String,
    soc2_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_certify` | function | standing_certify(
    receipt: String,
    observation: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `standing_verify` | function | standing_verify(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |

| `stats` | function | stats(
    #[arg(index = 1) |  |  |  |  |

| `team_velocity` | function | team_velocity(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `tech_debt` | function | tech_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `test` | function | test(
) |  |  |  |  |

| `timeline` | function | timeline(
    receipts_path: String,
    start_time: Option<String>,
    end_time: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `trend_analysis` | function | trend_analysis(
    receipts_path: String,
    metric: String,
    time_range: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `variance` | function | variance(
    receipt: Option<String>,
    iterations: Option<u32>,
) |  |  |  |  |

| `verify` | function | verify(
    #[arg(index = 1) |  |  |  |  |

| `verify_assurance` | function | verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) |  |  |  |  |

| `verify_compliance` | function | verify_compliance(
    #[arg(index = 1) |  |  |  |  |

| `verify_family` | function | verify_family(
    receipts_dir: String,
    format: Option<String>,
) |  |  |  |  |

| `verify_sla` | function | verify_sla(
    #[arg(index = 1) |  |  |  |  |

| `visualize` | function | visualize(
    format: String,
    #[arg(index = 1) |  |  |  |  |

| `why` | function | why(
    receipt: String,
    format: Option<String>,
) |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) |  |  |  |  |

| `build_graph` | function | build_graph(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |

| `WasmCourtError` | enum |  |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `run_bounded` | function | run_bounded(
        &self,
        wasm_bytes: &[u8],
        export: &str,
        fuel_budget: u64,
    ) |  |  |  |  |

| `WasmCourt` | struct |  |  |  |  |  |

| `ArchiveError` | enum |  |  |  |  |  |

| `freeze` | function | freeze(archive: &ReceiptArchive) |  |  |  |  |

| `thaw` | function | thaw(bytes: &[u8]) |  |  |  |  |

| `unfreeze` | function | unfreeze(bytes: &[u8]) |  |  |  |  |

| `validate_archived_range` | function | validate_archived_range(
    bytes: &[u8],
    min_seq: u64,
    max_seq: u64,
) |  |  |  |  |

| `ReceiptArchive` | struct |  |  |  |  |  |

| `SeqEntry` | struct |  |  |  |  |  |

| `RangeProofError` | enum |  |  |  |  |  |

| `verify_range` | function | verify_range(
    commitment: &[u8; 32],
    proof_bytes: &[u8],
    bits: usize,
    label: &'static [u8],
) |  |  |  |  |

| `RangeWitness` | struct |  |  |  |  |  |

| `generate_digest` | function | generate_digest(data: &[u8]) |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `generate_digest` | function | generate_digest(data: &[u8]) |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `ArcDirection` | enum |  |  |  |  |  |

| `ArcDirectionConst` | enum |  |  |  |  |  |

| `BpmnEvent` | enum |  |  |  |  |  |

| `BpmnGateway` | enum |  |  |  |  |  |

| `BpmnNodeKind` | enum |  |  |  |  |  |

| `BpmnRefusal` | enum |  |  |  |  |  |

| `CausalConsistency` | enum |  |  |  |  |  |

| `CausalNetRefusal` | enum |  |  |  |  |  |

| `CompatDiagnostic` | enum |  |  |  |  |  |

| `ComplianceKind` | enum |  |  |  |  |  |

| `ConformanceRefusal` | enum |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `CorrelationSchema` | enum |  |  |  |  |  |

| `CubeDimensionKind` | enum |  |  |  |  |  |

| `DeclareRefusal` | enum |  |  |  |  |  |

| `DeclareScope` | enum |  |  |  |  |  |

| `DeclareTemplate` | enum |  |  |  |  |  |

| `DfgRefusal` | enum |  |  |  |  |  |

| `DiagnosticKind` | enum |  |  |  |  |  |

| `DiagnosticSeverity` | enum |  |  |  |  |  |

| `EventLogRefusal` | enum |  |  |  |  |  |

| `EventPredicateKind` | enum |  |  |  |  |  |

| `EvidenceMode` | enum |  |  |  |  |  |

| `FilterShape` | enum |  |  |  |  |  |

| `FormatKind` | enum |  |  |  |  |  |

| `InstanceCreationKind` | enum |  |  |  |  |  |

| `InteropRefusal` | enum |  |  |  |  |  |

| `KernelRefusal` | enum |  |  |  |  |  |

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `LossFunction` | enum |  |  |  |  |  |

| `LossPolicy` | enum |  |  |  |  |  |

| `LossRefusal` | enum |  |  |  |  |  |

| `OCELAttributeValue` | enum |  |  |  |  |  |

| `ObjectCentricity` | enum |  |  |  |  |  |

| `ObjectLifecyclePhase` | enum |  |  |  |  |  |

| `ObjectPredicateKind` | enum |  |  |  |  |  |

| `ObjectTypeCardinality` | enum |  |  |  |  |  |

| `OcDeclareRefusal` | enum |  |  |  |  |  |

| `OcelAttributeValue` | enum |  |  |  |  |  |

| `OcelRefusal` | enum |  |  |  |  |  |

| `OcpqRefusal` | enum |  |  |  |  |  |

| `OcpqScopeKind` | enum |  |  |  |  |  |

| `PerspectiveRefusal` | enum |  |  |  |  |  |

| `PetriNetRefusal` | enum |  |  |  |  |  |

| `PetriRefusal` | enum |  |  |  |  |  |

| `Pm4pyShape` | enum |  |  |  |  |  |

| `Powl8Op` | enum |  |  |  |  |  |

| `Powl8OpError` | enum |  |  |  |  |  |

| `PowlNodeKind` | enum |  |  |  |  |  |

| `PowlProjectionState` | enum |  |  |  |  |  |

| `PowlRefusal` | enum |  |  |  |  |  |

| `PredicateKind` | enum |  |  |  |  |  |

| `PredictionHorizon` | enum |  |  |  |  |  |

| `PredictionRefusal` | enum |  |  |  |  |  |

| `PredictionTarget` | enum |  |  |  |  |  |

| `ProcessPerspective` | enum |  |  |  |  |  |

| `ProcessShapeKind` | enum |  |  |  |  |  |

| `ProcessTreeNode` | enum |  |  |  |  |  |

| `ProcessTreeOperator` | enum |  |  |  |  |  |

| `ProcessTreeRefusal` | enum |  |  |  |  |  |

| `QualityDimension` | enum |  |  |  |  |  |

| `QualityMetricKind` | enum |  |  |  |  |  |

| `ReceiptRefusal` | enum |  |  |  |  |  |

| `ReceiptVerdict` | enum |  |  |  |  |  |

| `RelationLaw` | enum |  |  |  |  |  |

| `RelationPredicateKind` | enum |  |  |  |  |  |

| `ReplayHintKind` | enum |  |  |  |  |  |

| `SoundnessState` | enum |  |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum |  |  |  |  |  |

| `SummaryShape` | enum |  |  |  |  |  |

| `TemporalOrder` | enum |  |  |  |  |  |

| `TemporalRefusal` | enum |  |  |  |  |  |

| `TemporalRelation` | enum |  |  |  |  |  |

| `WitnessFamily` | enum |  |  |  |  |  |

| `WorkflowPattern` | enum |  |  |  |  |  |

| `activity` | function | activity(&self) |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) |  |  |  |  |

| `admits` | function | admits(&self, count: usize) |  |  |  |  |

| `arcs` | function | arcs(&self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `as_f64` | function | as_f64(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) |  |  |  |  |

| `attribute` | function | attribute(&self) |  |  |  |  |

| `attributes` | function | attributes(&self) |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) |  |  |  |  |

| `by` | function | by(mut self, resource: &str) |  |  |  |  |

| `case_id` | function | case_id(&self) |  |  |  |  |

| `category` | function | category(&self) |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) |  |  |  |  |

| `claim_sound` | function | claim_sound(self) |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) |  |  |  |  |

| `default` | function | default() |  |  |  |  |

| `den` | function | den(&self) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) |  |  |  |  |

| `edges` | function | edges(&self) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) |  |  |  |  |

| `event_count` | function | event_count(&self) |  |  |  |  |

| `event_id` | function | event_id(&self) |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) |  |  |  |  |

| `event_set` | function | event_set(&self) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `expression` | function | expression(&self) |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) |  |  |  |  |

| `final_marking` | function | final_marking(&self) |  |  |  |  |

| `float` | function | float(key: &str, value: f64) |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) |  |  |  |  |

| `frequency` | function | frequency(&self) |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) |  |  |  |  |

| `from_owned` | function | from_owned(s: String) |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) |  |  |  |  |

| `get` | function | get(&self) |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) |  |  |  |  |

| `inner` | function | inner(&self) |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) |  |  |  |  |

| `into_admitted` | function | into_admitted(self) |  |  |  |  |

| `into_evidence` | function | into_evidence(self) |  |  |  |  |

| `into_exportable` | function | into_exportable(self) |  |  |  |  |

| `into_inner` | function | into_inner(self) |  |  |  |  |

| `into_lost` | function | into_lost(self) |  |  |  |  |

| `into_parsed` | function | into_parsed(self) |  |  |  |  |

| `into_projected` | function | into_projected(self) |  |  |  |  |

| `into_reason` | function | into_reason(self) |  |  |  |  |

| `into_receipted` | function | into_receipted(self) |  |  |  |  |

| `is_chain` | function | is_chain(&self) |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) |  |  |  |  |

| `is_negative` | function | is_negative(&self) |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) |  |  |  |  |

| `is_silent` | function | is_silent(&self) |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `kind` | function | kind(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `lanes` | function | lanes(&self) |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `length` | function | length(&self) |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `members` | function | members(&self) |  |  |  |  |

| `min` | function | min(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `net` | function | net(&self) |  |  |  |  |

| `new` | function | new(value: T) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `node_ids` | function | node_ids(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `num` | function | num(&self) |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) |  |  |  |  |

| `object_changes` | function | object_changes(&self) |  |  |  |  |

| `object_id` | function | object_id(&self) |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) |  |  |  |  |

| `object_set` | function | object_set(&self) |  |  |  |  |

| `object_type` | function | object_type(&self) |  |  |  |  |

| `object_types` | function | object_types(&self) |  |  |  |  |

| `objects` | function | objects(&self) |  |  |  |  |

| `place_id` | function | place_id(&self) |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) |  |  |  |  |

| `places` | function | places(&self) |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) |  |  |  |  |

| `process` | function | process(&self) |  |  |  |  |

| `projection` | function | projection(&self) |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) |  |  |  |  |

| `qualifier` | function | qualifier(&self) |  |  |  |  |

| `raw` | function | raw(value: T) |  |  |  |  |

| `resource` | function | resource(&self) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `scope` | function | scope(&self) |  |  |  |  |

| `silent` | function | silent(id: &str) |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) |  |  |  |  |

| `source` | function | source(&self) |  |  |  |  |

| `source_id` | function | source_id(&self) |  |  |  |  |

| `steps` | function | steps(&self) |  |  |  |  |

| `string` | function | string(key: &str, value: &str) |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) |  |  |  |  |

| `summary` | function | summary(&self, category: &str) |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `target` | function | target(&self) |  |  |  |  |

| `target_id` | function | target_id(&self) |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) |  |  |  |  |

| `tip` | function | tip(&self) |  |  |  |  |

| `tokens` | function | tokens(&self) |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) |  |  |  |  |

| `trace_count` | function | trace_count(&self) |  |  |  |  |

| `traces` | function | traces(&self) |  |  |  |  |

| `transition_id` | function | transition_id(&self) |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) |  |  |  |  |

| `transitions` | function | transitions(&self) |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `value` | function | value(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `weight` | function | weight(&self) |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) |  |  |  |  |

| `Activity` | struct |  |  |  |  |  |

| `Admission` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Arc` | struct |  |  |  |  |  |

| `ArtifactGrounding` | struct |  |  |  |  |  |

| `Between01` | struct |  |  |  |  |  |

| `BipartiteArcConst` | struct |  |  |  |  |  |

| `BpmnEdge` | struct |  |  |  |  |  |

| `BpmnLane` | struct |  |  |  |  |  |

| `BpmnNode` | struct |  |  |  |  |  |

| `BpmnPool` | struct |  |  |  |  |  |

| `BpmnProcess` | struct |  |  |  |  |  |

| `BpmnTask` | struct |  |  |  |  |  |

| `CancellationRegion` | struct |  |  |  |  |  |

| `CausalBinding` | struct |  |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct |  |  |  |  |  |

| `CausalNet` | struct |  |  |  |  |  |

| `CausallyOrderedEvidence` | struct |  |  |  |  |  |

| `ChoiceGraph` | struct |  |  |  |  |  |

| `ConditionCell` | struct |  |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `ConformanceVerdict` | struct |  |  |  |  |  |

| `ConsistencyVerified` | struct |  |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct |  |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct |  |  |  |  |  |

| `DenseKernel` | struct |  |  |  |  |  |

| `DependencyMeasure` | struct |  |  |  |  |  |

| `Deviation` | struct |  |  |  |  |  |

| `Dfg` | struct |  |  |  |  |  |

| `DfgEdge` | struct |  |  |  |  |  |

| `DfgEdgeFull` | struct |  |  |  |  |  |

| `DfgNode` | struct |  |  |  |  |  |

| `DfgWeight` | struct |  |  |  |  |  |

| `DiagnosticReport` | struct |  |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `EventLog` | struct |  |  |  |  |  |

| `EventObjectLink` | struct |  |  |  |  |  |

| `EventStream` | struct |  |  |  |  |  |

| `EventTypeName` | struct |  |  |  |  |  |

| `Evidence` | struct |  |  |  |  |  |

| `Exportable` | struct |  |  |  |  |  |

| `F1` | struct |  |  |  |  |  |

| `Fitness` | struct |  |  |  |  |  |

| `Generalization` | struct |  |  |  |  |  |

| `InitialFinalMarkingPair` | struct |  |  |  |  |  |

| `InputBinding` | struct |  |  |  |  |  |

| `LifecycleEvent` | struct |  |  |  |  |  |

| `LossChain` | struct |  |  |  |  |  |

| `LossReport` | struct |  |  |  |  |  |

| `Marking` | struct |  |  |  |  |  |

| `Metric` | struct |  |  |  |  |  |

| `MultiPerspectiveEvidence` | struct |  |  |  |  |  |

| `MultiPerspectiveLog` | struct |  |  |  |  |  |

| `MultipleInstanceSpec` | struct |  |  |  |  |  |

| `MultipleInstanceSpecConst` | struct |  |  |  |  |  |

| `NamedLoss` | struct |  |  |  |  |  |

| `OCEL` | struct |  |  |  |  |  |

| `OCELEvent` | struct |  |  |  |  |  |

| `OCELEventAttribute` | struct |  |  |  |  |  |

| `OCELObject` | struct |  |  |  |  |  |

| `OCELRelationship` | struct |  |  |  |  |  |

| `OCELType` | struct |  |  |  |  |  |

| `OCELTypeAttribute` | struct |  |  |  |  |  |

| `Object` | struct |  |  |  |  |  |

| `ObjectCentricDfg` | struct |  |  |  |  |  |

| `ObjectCentricPetriNet` | struct |  |  |  |  |  |

| `ObjectChange` | struct |  |  |  |  |  |

| `ObjectLifecycle` | struct |  |  |  |  |  |

| `ObjectObjectLink` | struct |  |  |  |  |  |

| `ObjectScope` | struct |  |  |  |  |  |

| `ObjectScopeConst` | struct |  |  |  |  |  |

| `ObjectTypeCardinality` | struct |  |  |  |  |  |

| `ObjectTypeName` | struct |  |  |  |  |  |

| `OcDeclareConstraint` | struct |  |  |  |  |  |

| `OcelAttribute` | struct |  |  |  |  |  |

| `OcelEvent` | struct |  |  |  |  |  |

| `OcelLog` | struct |  |  |  |  |  |

| `OcpqQuery` | struct |  |  |  |  |  |

| `OcpqQueryConst` | struct |  |  |  |  |  |

| `OrderEdge` | struct |  |  |  |  |  |

| `OutputBinding` | struct |  |  |  |  |  |

| `PackedKeyTable` | struct |  |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct |  |  |  |  |  |

| `PerspectiveCombination` | struct |  |  |  |  |  |

| `PetriNet` | struct |  |  |  |  |  |

| `Place` | struct |  |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct |  |  |  |  |  |

| `Powl` | struct |  |  |  |  |  |

| `PowlChoiceNode` | struct |  |  |  |  |  |

| `PowlComposition` | struct |  |  |  |  |  |

| `PowlNode` | struct |  |  |  |  |  |

| `PowlNodeId` | struct |  |  |  |  |  |

| `Precision` | struct |  |  |  |  |  |

| `Predicate` | struct |  |  |  |  |  |

| `ProcessCube` | struct |  |  |  |  |  |

| `ProcessSlice` | struct |  |  |  |  |  |

| `ProcessTree` | struct |  |  |  |  |  |

| `ProcessTreeNodeId` | struct |  |  |  |  |  |

| `Projected` | struct |  |  |  |  |  |

| `ProjectionName` | struct |  |  |  |  |  |

| `ProjectionNameOwned` | struct |  |  |  |  |  |

| `QualityProfile` | struct |  |  |  |  |  |

| `Raw` | struct |  |  |  |  |  |

| `ReceiptChain` | struct |  |  |  |  |  |

| `ReceiptChainConst` | struct |  |  |  |  |  |

| `ReceiptEnvelope` | struct |  |  |  |  |  |

| `Receipted` | struct |  |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |

| `Refused` | struct |  |  |  |  |  |

| `ReplayHint` | struct |  |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct |  |  |  |  |  |

| `SeparableWfNet` | struct |  |  |  |  |  |

| `Simplicity` | struct |  |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct |  |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct |  |  |  |  |  |

| `TemporalConstraint` | struct |  |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `Transition` | struct |  |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct |  |  |  |  |  |

| `TypedEventPredicate` | struct |  |  |  |  |  |

| `TypedId` | struct |  |  |  |  |  |

| `TypedLoopNode` | struct |  |  |  |  |  |

| `TypedObjectPredicate` | struct |  |  |  |  |  |

| `TypedPowlLoopNode` | struct |  |  |  |  |  |

| `TypedRelationPredicate` | struct |  |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct |  |  |  |  |  |

| `WfNetConst` | struct |  |  |  |  |  |

| `Witnessed` | struct |  |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `ArcDirection` | enum |  |  |  |  |  |

| `ArcDirectionConst` | enum |  |  |  |  |  |

| `BpmnEvent` | enum |  |  |  |  |  |

| `BpmnGateway` | enum |  |  |  |  |  |

| `BpmnNodeKind` | enum |  |  |  |  |  |

| `BpmnRefusal` | enum |  |  |  |  |  |

| `CausalConsistency` | enum |  |  |  |  |  |

| `CausalNetRefusal` | enum |  |  |  |  |  |

| `CompatDiagnostic` | enum |  |  |  |  |  |

| `ComplianceKind` | enum |  |  |  |  |  |

| `ConformanceRefusal` | enum |  |  |  |  |  |

| `ConformanceVerdict` | enum |  |  |  |  |  |

| `CorrelationSchema` | enum |  |  |  |  |  |

| `CubeDimensionKind` | enum |  |  |  |  |  |

| `DeclareRefusal` | enum |  |  |  |  |  |

| `DeclareScope` | enum |  |  |  |  |  |

| `DeclareTemplate` | enum |  |  |  |  |  |

| `DfgRefusal` | enum |  |  |  |  |  |

| `DiagnosticKind` | enum |  |  |  |  |  |

| `DiagnosticSeverity` | enum |  |  |  |  |  |

| `EventLogRefusal` | enum |  |  |  |  |  |

| `EventPredicateKind` | enum |  |  |  |  |  |

| `EvidenceMode` | enum |  |  |  |  |  |

| `FilterShape` | enum |  |  |  |  |  |

| `FormatKind` | enum |  |  |  |  |  |

| `InstanceCreationKind` | enum |  |  |  |  |  |

| `InteropRefusal` | enum |  |  |  |  |  |

| `KernelRefusal` | enum |  |  |  |  |  |

| `LifecycleRefusal` | enum |  |  |  |  |  |

| `LossFunction` | enum |  |  |  |  |  |

| `LossPolicy` | enum |  |  |  |  |  |

| `LossRefusal` | enum |  |  |  |  |  |

| `OCELAttributeValue` | enum |  |  |  |  |  |

| `ObjectCentricity` | enum |  |  |  |  |  |

| `ObjectLifecyclePhase` | enum |  |  |  |  |  |

| `ObjectPredicateKind` | enum |  |  |  |  |  |

| `ObjectTypeCardinality` | enum |  |  |  |  |  |

| `OcDeclareRefusal` | enum |  |  |  |  |  |

| `OcelAttributeValue` | enum |  |  |  |  |  |

| `OcelRefusal` | enum |  |  |  |  |  |

| `OcpqRefusal` | enum |  |  |  |  |  |

| `OcpqScopeKind` | enum |  |  |  |  |  |

| `PerspectiveRefusal` | enum |  |  |  |  |  |

| `PetriNetRefusal` | enum |  |  |  |  |  |

| `PetriRefusal` | enum |  |  |  |  |  |

| `Pm4pyShape` | enum |  |  |  |  |  |

| `Powl8Op` | enum |  |  |  |  |  |

| `Powl8OpError` | enum |  |  |  |  |  |

| `PowlNodeKind` | enum |  |  |  |  |  |

| `PowlProjectionState` | enum |  |  |  |  |  |

| `PowlRefusal` | enum |  |  |  |  |  |

| `PredicateKind` | enum |  |  |  |  |  |

| `PredictionHorizon` | enum |  |  |  |  |  |

| `PredictionRefusal` | enum |  |  |  |  |  |

| `PredictionTarget` | enum |  |  |  |  |  |

| `ProcessPerspective` | enum |  |  |  |  |  |

| `ProcessShapeKind` | enum |  |  |  |  |  |

| `ProcessTreeNode` | enum |  |  |  |  |  |

| `ProcessTreeOperator` | enum |  |  |  |  |  |

| `ProcessTreeRefusal` | enum |  |  |  |  |  |

| `QualityDimension` | enum |  |  |  |  |  |

| `QualityMetricKind` | enum |  |  |  |  |  |

| `ReceiptRefusal` | enum |  |  |  |  |  |

| `ReceiptVerdict` | enum |  |  |  |  |  |

| `RelationLaw` | enum |  |  |  |  |  |

| `RelationPredicateKind` | enum |  |  |  |  |  |

| `ReplayHintKind` | enum |  |  |  |  |  |

| `SoundnessState` | enum |  |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum |  |  |  |  |  |

| `SummaryShape` | enum |  |  |  |  |  |

| `TemporalOrder` | enum |  |  |  |  |  |

| `TemporalRefusal` | enum |  |  |  |  |  |

| `TemporalRelation` | enum |  |  |  |  |  |

| `WitnessFamily` | enum |  |  |  |  |  |

| `WorkflowPattern` | enum |  |  |  |  |  |

| `activity` | function | activity(&self) |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) |  |  |  |  |

| `admits` | function | admits(&self, count: usize) |  |  |  |  |

| `arcs` | function | arcs(&self) |  |  |  |  |

| `arity` | function | arity(&self) |  |  |  |  |

| `as_f64` | function | as_f64(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) |  |  |  |  |

| `attribute` | function | attribute(&self) |  |  |  |  |

| `attributes` | function | attributes(&self) |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) |  |  |  |  |

| `by` | function | by(mut self, resource: &str) |  |  |  |  |

| `case_id` | function | case_id(&self) |  |  |  |  |

| `category` | function | category(&self) |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) |  |  |  |  |

| `claim_sound` | function | claim_sound(self) |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) |  |  |  |  |

| `count` | function | count(&self) |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) |  |  |  |  |

| `default` | function | default() |  |  |  |  |

| `den` | function | den(&self) |  |  |  |  |

| `direction` | function | direction(&self) |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) |  |  |  |  |

| `edges` | function | edges(&self) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) |  |  |  |  |

| `event_count` | function | event_count(&self) |  |  |  |  |

| `event_id` | function | event_id(&self) |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) |  |  |  |  |

| `event_set` | function | event_set(&self) |  |  |  |  |

| `events` | function | events(&self) |  |  |  |  |

| `expression` | function | expression(&self) |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) |  |  |  |  |

| `final_marking` | function | final_marking(&self) |  |  |  |  |

| `float` | function | float(key: &str, value: f64) |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) |  |  |  |  |

| `frequency` | function | frequency(&self) |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) |  |  |  |  |

| `from_owned` | function | from_owned(s: String) |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) |  |  |  |  |

| `get` | function | get(&self) |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) |  |  |  |  |

| `inner` | function | inner(&self) |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) |  |  |  |  |

| `into_admitted` | function | into_admitted(self) |  |  |  |  |

| `into_evidence` | function | into_evidence(self) |  |  |  |  |

| `into_exportable` | function | into_exportable(self) |  |  |  |  |

| `into_inner` | function | into_inner(self) |  |  |  |  |

| `into_lost` | function | into_lost(self) |  |  |  |  |

| `into_parsed` | function | into_parsed(self) |  |  |  |  |

| `into_projected` | function | into_projected(self) |  |  |  |  |

| `into_reason` | function | into_reason(self) |  |  |  |  |

| `into_receipted` | function | into_receipted(self) |  |  |  |  |

| `is_chain` | function | is_chain(&self) |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) |  |  |  |  |

| `is_negative` | function | is_negative(&self) |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) |  |  |  |  |

| `is_silent` | function | is_silent(&self) |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `kind` | function | kind(&self) |  |  |  |  |

| `label` | function | label(&self) |  |  |  |  |

| `lanes` | function | lanes(&self) |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `length` | function | length(&self) |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) |  |  |  |  |

| `members` | function | members(&self) |  |  |  |  |

| `min` | function | min(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `net` | function | net(&self) |  |  |  |  |

| `new` | function | new(value: T) |  |  |  |  |

| `node_count` | function | node_count(&self) |  |  |  |  |

| `node_ids` | function | node_ids(&self) |  |  |  |  |

| `nodes` | function | nodes(&self) |  |  |  |  |

| `num` | function | num(&self) |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) |  |  |  |  |

| `object_changes` | function | object_changes(&self) |  |  |  |  |

| `object_id` | function | object_id(&self) |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) |  |  |  |  |

| `object_set` | function | object_set(&self) |  |  |  |  |

| `object_type` | function | object_type(&self) |  |  |  |  |

| `object_types` | function | object_types(&self) |  |  |  |  |

| `objects` | function | objects(&self) |  |  |  |  |

| `place_id` | function | place_id(&self) |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) |  |  |  |  |

| `places` | function | places(&self) |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) |  |  |  |  |

| `process` | function | process(&self) |  |  |  |  |

| `projection` | function | projection(&self) |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) |  |  |  |  |

| `qualifier` | function | qualifier(&self) |  |  |  |  |

| `raw` | function | raw(value: T) |  |  |  |  |

| `resource` | function | resource(&self) |  |  |  |  |

| `root` | function | root(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `scope` | function | scope(&self) |  |  |  |  |

| `silent` | function | silent(id: &str) |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) |  |  |  |  |

| `source` | function | source(&self) |  |  |  |  |

| `source_id` | function | source_id(&self) |  |  |  |  |

| `steps` | function | steps(&self) |  |  |  |  |

| `string` | function | string(key: &str, value: &str) |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) |  |  |  |  |

| `summary` | function | summary(&self, category: &str) |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) |  |  |  |  |

| `tag` | function | tag(&self) |  |  |  |  |

| `target` | function | target(&self) |  |  |  |  |

| `target_id` | function | target_id(&self) |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) |  |  |  |  |

| `tip` | function | tip(&self) |  |  |  |  |

| `tokens` | function | tokens(&self) |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) |  |  |  |  |

| `trace_count` | function | trace_count(&self) |  |  |  |  |

| `traces` | function | traces(&self) |  |  |  |  |

| `transition_id` | function | transition_id(&self) |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) |  |  |  |  |

| `transitions` | function | transitions(&self) |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) |  |  |  |  |

| `validate` | function | validate(&self) |  |  |  |  |

| `value` | function | value(&self) |  |  |  |  |

| `verdict` | function | verdict(&self) |  |  |  |  |

| `verify` | function | verify(&self) |  |  |  |  |

| `weight` | function | weight(&self) |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) |  |  |  |  |

| `Activity` | struct |  |  |  |  |  |

| `Admission` | struct |  |  |  |  |  |

| `Admitted` | struct |  |  |  |  |  |

| `Arc` | struct |  |  |  |  |  |

| `ArtifactGrounding` | struct |  |  |  |  |  |

| `Between01` | struct |  |  |  |  |  |

| `BipartiteArcConst` | struct |  |  |  |  |  |

| `BpmnEdge` | struct |  |  |  |  |  |

| `BpmnLane` | struct |  |  |  |  |  |

| `BpmnNode` | struct |  |  |  |  |  |

| `BpmnPool` | struct |  |  |  |  |  |

| `BpmnProcess` | struct |  |  |  |  |  |

| `BpmnTask` | struct |  |  |  |  |  |

| `CancellationRegion` | struct |  |  |  |  |  |

| `CausalBinding` | struct |  |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct |  |  |  |  |  |

| `CausalNet` | struct |  |  |  |  |  |

| `CausallyOrderedEvidence` | struct |  |  |  |  |  |

| `ChoiceGraph` | struct |  |  |  |  |  |

| `ConditionCell` | struct |  |  |  |  |  |

| `ConformanceResult` | struct |  |  |  |  |  |

| `ConformanceVerdict` | struct |  |  |  |  |  |

| `ConsistencyVerified` | struct |  |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct |  |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct |  |  |  |  |  |

| `DenseKernel` | struct |  |  |  |  |  |

| `DependencyMeasure` | struct |  |  |  |  |  |

| `Deviation` | struct |  |  |  |  |  |

| `Dfg` | struct |  |  |  |  |  |

| `DfgEdge` | struct |  |  |  |  |  |

| `DfgEdgeFull` | struct |  |  |  |  |  |

| `DfgNode` | struct |  |  |  |  |  |

| `DfgWeight` | struct |  |  |  |  |  |

| `DiagnosticReport` | struct |  |  |  |  |  |

| `Digest` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `EventLog` | struct |  |  |  |  |  |

| `EventObjectLink` | struct |  |  |  |  |  |

| `EventStream` | struct |  |  |  |  |  |

| `EventTypeName` | struct |  |  |  |  |  |

| `Evidence` | struct |  |  |  |  |  |

| `Exportable` | struct |  |  |  |  |  |

| `F1` | struct |  |  |  |  |  |

| `Fitness` | struct |  |  |  |  |  |

| `Generalization` | struct |  |  |  |  |  |

| `InitialFinalMarkingPair` | struct |  |  |  |  |  |

| `InputBinding` | struct |  |  |  |  |  |

| `LifecycleEvent` | struct |  |  |  |  |  |

| `LossChain` | struct |  |  |  |  |  |

| `LossReport` | struct |  |  |  |  |  |

| `Marking` | struct |  |  |  |  |  |

| `Metric` | struct |  |  |  |  |  |

| `MultiPerspectiveEvidence` | struct |  |  |  |  |  |

| `MultiPerspectiveLog` | struct |  |  |  |  |  |

| `MultipleInstanceSpec` | struct |  |  |  |  |  |

| `MultipleInstanceSpecConst` | struct |  |  |  |  |  |

| `NamedLoss` | struct |  |  |  |  |  |

| `OCEL` | struct |  |  |  |  |  |

| `OCELEvent` | struct |  |  |  |  |  |

| `OCELEventAttribute` | struct |  |  |  |  |  |

| `OCELObject` | struct |  |  |  |  |  |

| `OCELRelationship` | struct |  |  |  |  |  |

| `OCELType` | struct |  |  |  |  |  |

| `OCELTypeAttribute` | struct |  |  |  |  |  |

| `Object` | struct |  |  |  |  |  |

| `ObjectCentricDfg` | struct |  |  |  |  |  |

| `ObjectCentricPetriNet` | struct |  |  |  |  |  |

| `ObjectChange` | struct |  |  |  |  |  |

| `ObjectLifecycle` | struct |  |  |  |  |  |

| `ObjectObjectLink` | struct |  |  |  |  |  |

| `ObjectScope` | struct |  |  |  |  |  |

| `ObjectScopeConst` | struct |  |  |  |  |  |

| `ObjectTypeCardinality` | struct |  |  |  |  |  |

| `ObjectTypeName` | struct |  |  |  |  |  |

| `OcDeclareConstraint` | struct |  |  |  |  |  |

| `OcelAttribute` | struct |  |  |  |  |  |

| `OcelEvent` | struct |  |  |  |  |  |

| `OcelLog` | struct |  |  |  |  |  |

| `OcpqQuery` | struct |  |  |  |  |  |

| `OcpqQueryConst` | struct |  |  |  |  |  |

| `OrderEdge` | struct |  |  |  |  |  |

| `OutputBinding` | struct |  |  |  |  |  |

| `PackedKeyTable` | struct |  |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct |  |  |  |  |  |

| `PerspectiveCombination` | struct |  |  |  |  |  |

| `PetriNet` | struct |  |  |  |  |  |

| `Place` | struct |  |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct |  |  |  |  |  |

| `Powl` | struct |  |  |  |  |  |

| `PowlChoiceNode` | struct |  |  |  |  |  |

| `PowlComposition` | struct |  |  |  |  |  |

| `PowlNode` | struct |  |  |  |  |  |

| `PowlNodeId` | struct |  |  |  |  |  |

| `Precision` | struct |  |  |  |  |  |

| `Predicate` | struct |  |  |  |  |  |

| `ProcessCube` | struct |  |  |  |  |  |

| `ProcessSlice` | struct |  |  |  |  |  |

| `ProcessTree` | struct |  |  |  |  |  |

| `ProcessTreeNodeId` | struct |  |  |  |  |  |

| `Projected` | struct |  |  |  |  |  |

| `ProjectionName` | struct |  |  |  |  |  |

| `ProjectionNameOwned` | struct |  |  |  |  |  |

| `QualityProfile` | struct |  |  |  |  |  |

| `Raw` | struct |  |  |  |  |  |

| `ReceiptChain` | struct |  |  |  |  |  |

| `ReceiptChainConst` | struct |  |  |  |  |  |

| `ReceiptEnvelope` | struct |  |  |  |  |  |

| `Receipted` | struct |  |  |  |  |  |

| `Refusal` | struct |  |  |  |  |  |

| `Refused` | struct |  |  |  |  |  |

| `ReplayHint` | struct |  |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct |  |  |  |  |  |

| `SeparableWfNet` | struct |  |  |  |  |  |

| `Simplicity` | struct |  |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct |  |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct |  |  |  |  |  |

| `TemporalConstraint` | struct |  |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `Transition` | struct |  |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct |  |  |  |  |  |

| `TypedEventPredicate` | struct |  |  |  |  |  |

| `TypedId` | struct |  |  |  |  |  |

| `TypedLoopNode` | struct |  |  |  |  |  |

| `TypedObjectPredicate` | struct |  |  |  |  |  |

| `TypedPowlLoopNode` | struct |  |  |  |  |  |

| `TypedRelationPredicate` | struct |  |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct |  |  |  |  |  |

| `WfNetConst` | struct |  |  |  |  |  |

| `Witnessed` | struct |  |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `AttributeValue` | enum |  |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) |  |  |  |  |

| `DFG` | struct |  |  |  |  |  |

| `DFGNode` | struct |  |  |  |  |  |

| `DirectlyFollowsRelation` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `AttributeValue` | enum |  |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) |  |  |  |  |

| `DFG` | struct |  |  |  |  |  |

| `DFGNode` | struct |  |  |  |  |  |

| `DirectlyFollowsRelation` | struct |  |  |  |  |  |

| `Event` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `filter_fixtures` | function | filter_fixtures(
        fixtures: Vec<FixtureMeta>,
        name: Option<&str>,
        events: Option<usize>,
    ) |  |  |  |  |

| `FixtureMeta` | struct |  |  |  |  |  |

| `ArbitraryOperationEvent` | struct |  |  |  |  |  |

| `ArbitraryReceipt` | struct |  |  |  |  |  |

| `add` | function | add(a: i32, b: i32) |  |  |  |  |

| `another_feature` | function | another_feature() |  |  |  |  |

| `complex_feature` | function | complex_feature() |  |  |  |  |

| `compute` | function | compute(x: i32) |  |  |  |  |

| `dangerous_path` | function | dangerous_path() |  |  |  |  |

| `subtract` | function | subtract(a: i32, b: i32) |  |  |  |  |

| `from_receipt_dfg` | function | from_receipt_dfg(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(&self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |

| `CspAc3` | struct |  |  |  |  |  |

| `CspAc3` | struct |  |  |  |  |  |

| `Engine` | enum |  |  |  |  |  |

| `breed_id` | function | breed_id(self) |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) |  |  |  |  |

| `from_id` | function | from_id(s: &str) |  |  |  |  |

| `generate_config` | function | generate_config(
    space: &FeatureSpace,
    query: &ConfigQuery,
    engine: Engine,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) |  |  |  |  |

| `ConfigQuery` | struct |  |  |  |  |  |

| `SemanticConfig` | struct |  |  |  |  |  |

| `Engine` | enum |  |  |  |  |  |

| `breed_id` | function | breed_id(self) |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) |  |  |  |  |

| `from_id` | function | from_id(s: &str) |  |  |  |  |

| `generate_config` | function | generate_config(
    space: &FeatureSpace,
    query: &ConfigQuery,
    engine: Engine,
) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) |  |  |  |  |

| `ConfigQuery` | struct |  |  |  |  |  |

| `SemanticConfig` | struct |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_sat` | function | is_sat(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) |  |  |  |  |

| `supported_breeds` | function | supported_breeds() |  |  |  |  |

| `tag` | function | tag(self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `BreedResult` | struct |  |  |  |  |  |

| `Contract` | struct |  |  |  |  |  |

| `Fact` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `TraceStep` | struct |  |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `Verdict` | enum |  |  |  |  |  |

| `from_json` | function | from_json(text: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `is_sat` | function | is_sat(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) |  |  |  |  |

| `supported_breeds` | function | supported_breeds() |  |  |  |  |

| `tag` | function | tag(self) |  |  |  |  |

| `to_json` | function | to_json(&self) |  |  |  |  |

| `BreedResult` | struct |  |  |  |  |  |

| `Contract` | struct |  |  |  |  |  |

| `Fact` | struct |  |  |  |  |  |

| `Trace` | struct |  |  |  |  |  |

| `TraceStep` | struct |  |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `SatCdcl` | struct |  |  |  |  |  |

| `SatCdcl` | struct |  |  |  |  |  |

| `run_ga` | function | run_ga(
    eval: &mut impl Evaluator,
    space: &FeatureSpace,
    cfg: &GaConfig,
) |  |  |  |  |

| `GaConfig` | struct |  |  |  |  |  |

| `GaError` | struct |  |  |  |  |  |

| `GaResult` | struct |  |  |  |  |  |

| `GenerationRecord` | struct |  |  |  |  |  |

| `run_ga` | function | run_ga(
    eval: &mut impl Evaluator,
    space: &FeatureSpace,
    cfg: &GaConfig,
) |  |  |  |  |

| `GaConfig` | struct |  |  |  |  |  |

| `GaError` | struct |  |  |  |  |  |

| `GaResult` | struct |  |  |  |  |  |

| `GenerationRecord` | struct |  |  |  |  |  |

| `cargo_available` | function | cargo_available() |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) |  |  |  |  |

| `generic` | function | generic() |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) |  |  |  |  |

| `score_from` | function | score_from(
    weights: &ScoreWeights,
    builds: bool,
    resolves: bool,
    error_count: u64,
    n_features: usize,
    elapsed_s: f64,
) |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) |  |  |  |  |

| `CargoEvaluator` | struct |  |  |  |  |  |

| `EvalResult` | struct |  |  |  |  |  |

| `ScoreWeights` | struct |  |  |  |  |  |

| `SyntheticEvaluator` | struct |  |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |

| `cargo_available` | function | cargo_available() |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) |  |  |  |  |

| `generic` | function | generic() |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) |  |  |  |  |

| `score_from` | function | score_from(
    weights: &ScoreWeights,
    builds: bool,
    resolves: bool,
    error_count: u64,
    n_features: usize,
    elapsed_s: f64,
) |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) |  |  |  |  |

| `CargoEvaluator` | struct |  |  |  |  |  |

| `EvalResult` | struct |  |  |  |  |  |

| `ScoreWeights` | struct |  |  |  |  |  |

| `SyntheticEvaluator` | struct |  |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |

| `canonical` | function | canonical(&self, space: &FeatureSpace) |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `feature_set` | function | feature_set(&self) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) |  |  |  |  |

| `Genome` | struct |  |  |  |  |  |

| `canonical` | function | canonical(&self, space: &FeatureSpace) |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) |  |  |  |  |

| `empty` | function | empty() |  |  |  |  |

| `feature_set` | function | feature_set(&self) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) |  |  |  |  |

| `Genome` | struct |  |  |  |  |  |

| `ManifestError` | enum |  |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml(
    path: impl AsRef<Path>,
    include_default: bool,
) |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str(
    toml: &str,
    include_default: bool,
) |  |  |  |  |

| `ManifestError` | enum |  |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml(
    path: impl AsRef<Path>,
    include_default: bool,
) |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str(
    toml: &str,
    include_default: bool,
) |  |  |  |  |

| `Mode` | enum |  |  |  |  |  |

| `to_json` | function | to_json(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `to_markdown` | function | to_markdown(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `Mode` | enum |  |  |  |  |  |

| `to_json` | function | to_json(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `to_markdown` | function | to_markdown(
    result: &GaResult,
    cfg: &GaConfig,
    mode: Mode,
    manifest: &str,
    universe: usize,
) |  |  |  |  |

| `below` | function | below(&mut self, n: usize) |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) |  |  |  |  |

| `new` | function | new(seed: u64) |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) |  |  |  |  |

| `Rng` | struct |  |  |  |  |  |

| `below` | function | below(&mut self, n: usize) |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) |  |  |  |  |

| `new` | function | new(seed: u64) |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) |  |  |  |  |

| `Rng` | struct |  |  |  |  |  |

| `SpaceError` | enum |  |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) |  |  |  |  |

| `contains` | function | contains(&self, name: &str) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `FeatureSpace` | struct |  |  |  |  |  |

| `SpaceError` | enum |  |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) |  |  |  |  |

| `contains` | function | contains(&self, name: &str) |  |  |  |  |

| `features` | function | features(&self) |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `FeatureSpace` | struct |  |  |  |  |  |

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `DiffResult` | struct |  |  |  |  |  |

| `ModifiedEvent` | struct |  |  |  |  |  |

| `DiffError` | enum |  |  |  |  |  |

| `DiffSummary` | struct |  |  |  |  |  |

| `InstrumentedDiff` | trait |  |  |  |  |  |

| `build_graph` | function | build_graph(receipt: &Receipt) |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) |  |  |  |  |

| `GraphEdge` | struct |  |  |  |  |  |

| `GraphNode` | struct |  |  |  |  |  |

| `ReceiptGraph` | struct |  |  |  |  |  |

| `new` | function | new(receipt_path: &str, format: &str) |  |  |  |  |

| `VisualizeInstrumentation` | struct |  |  |  |  |  |

| `VisualizeExt` | trait |  |  |  |  |  |

| `generate_completions` | function | generate_completions(shell_name: &str) |  |  |  |  |

| `try_dispatch_completion` | function | try_dispatch_completion() |  |  |  |  |

| `CompletionError` | enum |  |  |  |  |  |

| `Shell` | enum |  |  |  |  |  |

| `completion` | function | completion(shell_name: String) |  |  |  |  |

| `MiningError` | enum |  |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) |  |  |  |  |

| `PredictionError` | enum |  |  |  |  |  |

| `predict_next` | function | predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model(
    model: &AdmittedReceipt,
    current_trace: &AdmittedReceipt,
    top_k: usize,
) |  |  |  |  |

| `ActivityPrediction` | struct |  |  |  |  |  |

| `PredictionReport` | struct |  |  |  |  |  |

| `from_receipt` | function | from_receipt(receipt: &Receipt, uri: &Url, text: &str) |  |  |  |  |

| `handle_definition` | function | handle_definition(pos: Position, index: &ReceiptIndex) |  |  |  |  |

| `handle_hover` | function | handle_hover(pos: Position, index: &ReceiptIndex) |  |  |  |  |

| `ObjectRefLocation` | struct |  |  |  |  |  |

| `ReceiptIndex` | struct |  |  |  |  |  |

| `ReceiptSymbol` | struct |  |  |  |  |  |

| `MutationKind` | enum |  |  |  |  |  |

| `all_operators` | function | all_operators() |  |  |  |  |

| `AppliedMutation` | struct |  |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) |  |  |  |  |

| `from_json` | function | from_json(json: &str) |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, 
        pattern_name: &str, 
        events: Vec<serde_json::Value>, 
        expected_verdict: &str,
        expected_failure_stage: Option<&str>
    ) |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) |  |  |  |  |

| `main` | function | main() |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `CodegenEngine` | struct |  |  |  |  |  |

| `Snippet` | struct |  |  |  |  |  |

| `SnippetRegistry` | struct |  |  |  |  |  |

| `ArbitraryOperationEvent` | struct |  |  |  |  |  |

| `ArbitraryReceipt` | struct |  |  |  |  |  |

| `all` | function | all(&self) |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) |  |  |  |  |

| `reindex` | function | reindex(&mut self) |  |  |  |  |

| `save` | function | save(&self) |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) |  |  |  |  |

| `Fixture` | struct |  |  |  |  |  |

| `FixtureDatabase` | struct |  |  |  |  |  |

| `FixtureQuery` | struct |  |  |  |  |  |

| `SloViolation` | enum |  |  |  |  |  |

| `check_slo` | function | check_slo(&self) |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `new_noop` | function | new_noop() |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) |  |  |  |  |

| `MetricsCollector` | struct |  |  |  |  |  |

| `PrometheusExporter` | struct |  |  |  |  |  |

| `ServiceLevelIndicators` | struct |  |  |  |  |  |

| `run` | function | run() |  |  |  |  |

| `compare_baseline` | function | compare_baseline(&mut self, bench_id: &str, current: f64, baseline: f64) |  |  |  |  |

| `load_baseline_value` | function | load_baseline_value(path: impl AsRef<Path>) |  |  |  |  |

| `new` | function | new(assembler: &'a mut ChainAssembler, counter: &'a mut SeqCounter) |  |  |  |  |

| `observe_criterion_bench` | function | observe_criterion_bench(&mut self, criterion_root: &str, bench_id: &str) |  |  |  |  |

| `record_throughput` | function | record_throughput(&mut self, bench_id: &str, ops_per_sec: f64) |  |  |  |  |

| `ThroughputObserver` | struct |  |  |  |  |  |

| `to_noun_verb_error` | function | to_noun_verb_error(err: AffidavitError) |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
