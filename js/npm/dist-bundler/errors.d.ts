import type { ErrorCode } from './generated/types.js';
export declare class AffidavitError extends Error {
    readonly code: string;
    constructor(code: string, message: string);
}
export declare class AffidavitInitError extends AffidavitError {
    constructor(message: string, options?: {
        cause?: unknown;
    });
}
export declare class AffidavitIntegrityError extends AffidavitError {
    readonly expected: string;
    readonly actual: string;
    constructor(expected: string, actual: string);
}
export declare class AffidavitAllocRefusedError extends AffidavitError {
    constructor(message: string);
}
export declare class AffidavitRequestTooLargeError extends AffidavitError {
    readonly observed: number;
    readonly max: number;
    constructor(observed: number, max: number);
}
export declare class AffidavitRefusalError extends AffidavitError {
    readonly refusalCode?: ErrorCode | string | undefined;
    constructor(message: string, refusalCode?: ErrorCode | string);
}
//# sourceMappingURL=errors.d.ts.map