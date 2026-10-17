import type { ErrorCode } from './generated/types.js';
export interface AbiRefused {
    ok: false;
    op?: string;
    error?: string;
    code?: ErrorCode | string;
    reason?: string;
    [key: string]: unknown;
}
export type AbiOk<T> = {
    ok: true;
} & T;
export type AbiResult<T> = AbiOk<T> | AbiRefused;
export * from './generated/types.js';
//# sourceMappingURL=types.d.ts.map