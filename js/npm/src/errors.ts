import type { ErrorCode } from './generated/types.js';

export class AffidavitError extends Error {
  readonly code: string;
  constructor(code: string, message: string) {
    super(message);
    this.name = new.target.name;
    this.code = code;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export class AffidavitInitError extends AffidavitError {
  constructor(message: string, options?: { cause?: unknown }) {
    super('init', message);
    if (options?.cause !== undefined) (this as any).cause = options.cause;
  }
}

export class AffidavitIntegrityError extends AffidavitError {
  readonly expected: string;
  readonly actual: string;
  constructor(expected: string, actual: string) {
    super('integrity', `wasm artifact digest mismatch: expected ${expected}, got ${actual}`);
    this.expected = expected;
    this.actual = actual;
  }
}

export class AffidavitAllocRefusedError extends AffidavitError {
  constructor(message: string) {
    super('alloc_refused', message);
  }
}

export class AffidavitRequestTooLargeError extends AffidavitError {
  readonly observed: number;
  readonly max: number;
  constructor(observed: number, max: number) {
    super(
      'request_too_large',
      `request is ${observed} bytes; the ABI limit is ${max}`,
    );
    this.observed = observed;
    this.max = max;
  }
}

export class AffidavitRefusalError extends AffidavitError {
  readonly refusalCode?: ErrorCode | string | undefined;
  constructor(message: string, refusalCode?: ErrorCode | string) {
    super(refusalCode ?? 'refused', message);
    if (refusalCode !== undefined) {
      this.refusalCode = refusalCode;
    }
  }
}
