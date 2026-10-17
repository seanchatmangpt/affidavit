// Generated types for @seanchatmangpt/affidavit

export const ERROR_CODES = [
  "bad_json",
  "missing_field",
  "unknown_op",
  "bad_field",
  "too_large",
  "too_deep",
  "missing_buffer",
  "internal",
  "malformed",
  "pqc_refused"
] as const;
export type ErrorCode = typeof ERROR_CODES[number];

export interface AffidavitLimits {
  max_json_depth: number;
  max_request_bytes: number;
  stack_size_bytes: number;
}

export interface CapabilitiesRequest {
  op: "capabilities";
}

export interface CapabilitiesResponse {
  ok: boolean;
  op: "capabilities";
  module?: any;
  version?: any;
  abi_version?: any;
  format_version?: any;
  genesis_seed?: any;
  hash?: any;
  ops?: any;
  limits?: any;
}

export interface CommitRequest {
  op: "commit";
  payload?: any;
  payload_hex?: any;
}

export interface CommitResponse {
  ok: boolean;
  op: "commit";
  commitment?: any;
  bytes?: any;
}

export interface AssembleRequest {
  op: "assemble";
  events?: any;
}

export interface AssembleResponse {
  ok: boolean;
  op: "assemble";
  receipt?: any;
  content_address?: any;
  chain_hash?: any;
}

export interface VerifyRequest {
  op: "verify";
  receipt?: any;
}

export interface VerifyResponse {
  ok: boolean;
  op: "verify";
  accepted?: any;
  profile?: any;
  outcomes?: any;
  reason?: any;
  content_address?: any;
}

export interface MineRequest {
  op: "mine";
  receipts?: any;
}

export interface MineResponse {
  ok: boolean;
  op: "mine";
  trace_count?: any;
  event_count?: any;
  activities?: any;
  edges?: any;
  start?: any;
  end?: any;
  activity_frequency?: any;
  variants?: any;
  distinct_variants?: any;
  footprint?: any;
  footprint_omitted?: any;
  all_accepted?: any;
  rejected_receipts?: any;
}

export interface ConformRequest {
  op: "conform";
  model?: any;
  trace?: any;
}

export interface ConformResponse {
  ok: boolean;
  op: "conform";
  verdict?: any;
  fitness?: any;
  legal_moves?: any;
  total_moves?: any;
  start_ok?: any;
  end_ok?: any;
  first_violation?: any;
  model_traces?: any;
  trace_accepted?: any;
  model_all_accepted?: any;
}

export interface VerifySignatureInputRequest {
  op: "verify_signature_input";
  envelope_json?: any;
  expected_signing_input_hex?: any;
}

export interface VerifySignatureInputResponse {
  ok: boolean;
  op: "verify_signature_input";
  verified?: any;
  signing_input_hex?: any;
}

export interface CertifyAuthzenEvidenceRequest {
  op: "certify_authzen_evidence";
  request?: any;
  decision?: any;
  policy_decision_point?: any;
  expected_policy_decision_point?: any;
  expected_principal?: any;
  expected_effect_digest?: any;
}

export interface CertifyAuthzenEvidenceResponse {
  ok: boolean;
  op: "certify_authzen_evidence";
  certified?: any;
  standard?: any;
  policy_allows?: any;
  policy_decision_point?: any;
  principal?: any;
  effect_digest?: any;
  action?: any;
  authority?: any;
  consequence?: any;
  evidence_digest?: any;
  evidence?: any;
}

export interface CertifySpiffeEvidenceRequest {
  op: "certify_spiffe_evidence";
  spiffe_id?: any;
  expected_spiffe_id?: any;
  expected_trust_domain?: any;
  bundle_digest?: any;
  verified?: any;
  svid_type?: any;
  allow_jwt?: any;
}

export interface CertifySpiffeEvidenceResponse {
  ok: boolean;
  op: "certify_spiffe_evidence";
  certified?: any;
  standard?: any;
  spiffe_id?: any;
  trust_domain?: any;
  path?: any;
  svid_type?: any;
  authority?: any;
  consequence?: any;
  evidence_digest?: any;
  evidence?: any;
}

export interface JcsCanonicalizeRequest {
  op: "jcs_canonicalize";
  json?: any;
}

export interface JcsCanonicalizeResponse {
  ok: boolean;
  op: "jcs_canonicalize";
  canonical_json?: any;
  commitment?: any;
}

export interface SmtAbsenceVerifyRequest {
  op: "smt_absence_verify";
  root_hex?: any;
  queried_key_hex?: any;
  neighbor_key_hex?: any;
  neighbor_value_hex?: any;
  claimed_prefix_bits?: any;
  proof_steps?: any;
}

export interface SmtAbsenceVerifyResponse {
  ok: boolean;
  op: "smt_absence_verify";
  admitted?: any;
}

export interface RangeProofVerifyRequest {
  op: "range_proof_verify";
  commitment_hex?: any;
  proof_hex?: any;
  bits?: any;
  label?: any;
}

export interface RangeProofVerifyResponse {
  ok: boolean;
  op: "range_proof_verify";
  admitted?: any;
}

export interface DeriveSubjectDigestRequest {
  op: "derive_subject_digest";
  domain_tag?: any;
  subject?: any;
  subject_hex?: any;
}

export interface DeriveSubjectDigestResponse {
  ok: boolean;
  op: "derive_subject_digest";
  digest_hex?: any;
  pre_image_hex?: any;
}

export interface SmtInclusionVerifyRequest {
  op: "smt_inclusion_verify";
  root_hex?: any;
  leaf_key_hex?: any;
  leaf_value_hex?: any;
  proof_steps?: any;
}

export interface SmtInclusionVerifyResponse {
  ok: boolean;
  op: "smt_inclusion_verify";
  admitted?: any;
}

export interface VerifySignatureRequest {
  op: "verify_signature";
  alg?: any;
  public_key_hex?: any;
  signature_hex?: any;
  signing_input?: any;
  signing_input_hex?: any;
}

export interface VerifySignatureResponse {
  ok: boolean;
  op: "verify_signature";
  valid?: any;
  refusal?: any;
}

export interface PqcAssembleRequest {
  op: "pqc_assemble";
  events?: any;
  ml_dsa_sig_hex?: any;
  ml_dsa_pk_hex?: any;
  pqc_profile?: any;
}

export interface PqcAssembleResponse {
  ok: boolean;
  op: "pqc_assemble";
  receipt?: any;
  content_address?: any;
  chain_hash?: any;
  pqc?: any;
}

export interface PqcVerifyRequest {
  op: "pqc_verify";
  msg?: any;
  msg_hex?: any;
  sig_hex?: any;
  pk_hex?: any;
  profile?: any;
}

export interface PqcVerifyResponse {
  ok: boolean;
  op: "pqc_verify";
  profile?: any;
  algorithm?: any;
  accepted?: any;
  refusal?: any;
}

