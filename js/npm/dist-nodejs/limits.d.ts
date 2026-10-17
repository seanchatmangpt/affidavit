/**
 * ABI limits, mirrored from registry/capability-registry.json limits.
 */
export declare const LIMITS: {
    readonly max_json_depth: 64;
    readonly max_request_bytes: 16777216;
    readonly stack_size_bytes: 16777216;
};
export type LimitName = keyof typeof LIMITS;
export declare const ABI_VERSION = 1;
//# sourceMappingURL=limits.d.ts.map