/**
 * ABI limits, mirrored from registry/capability-registry.json limits.
 */
export const LIMITS = {
  max_json_depth: 64,
  max_request_bytes: 16777216,
  stack_size_bytes: 16777216,
} as const;

export type LimitName = keyof typeof LIMITS;

export const ABI_VERSION = 1;
