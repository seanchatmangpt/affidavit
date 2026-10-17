export class AffidavitError extends Error {
    code;
    constructor(code, message) {
        super(message);
        this.name = new.target.name;
        this.code = code;
        Object.setPrototypeOf(this, new.target.prototype);
    }
}
export class AffidavitInitError extends AffidavitError {
    constructor(message, options) {
        super('init', message);
        if (options?.cause !== undefined)
            this.cause = options.cause;
    }
}
export class AffidavitIntegrityError extends AffidavitError {
    expected;
    actual;
    constructor(expected, actual) {
        super('integrity', `wasm artifact digest mismatch: expected ${expected}, got ${actual}`);
        this.expected = expected;
        this.actual = actual;
    }
}
export class AffidavitAllocRefusedError extends AffidavitError {
    constructor(message) {
        super('alloc_refused', message);
    }
}
export class AffidavitRequestTooLargeError extends AffidavitError {
    observed;
    max;
    constructor(observed, max) {
        super('request_too_large', `request is ${observed} bytes; the ABI limit is ${max}`);
        this.observed = observed;
        this.max = max;
    }
}
export class AffidavitRefusalError extends AffidavitError {
    refusalCode;
    constructor(message, refusalCode) {
        super(refusalCode ?? 'refused', message);
        if (refusalCode !== undefined) {
            this.refusalCode = refusalCode;
        }
    }
}
