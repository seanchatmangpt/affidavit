import {
  AffidavitAllocRefusedError,
  AffidavitRequestTooLargeError,
} from './errors.js';
import { LIMITS } from './limits.js';

export interface AfExports {
  memory: WebAssembly.Memory;
  af_alloc: (len: number) => number;
  af_call: (ptr: number, len: number) => bigint | number;
  af_free: (ptr: number, len: number) => void;
  af_abi_version?: () => number;
  affi_admit_and_seal?: (ptr: number, len: number) => bigint | number;
}

export function hasInitialize(exports_: Record<string, unknown>): boolean {
  return typeof exports_['_initialize'] === 'function';
}

export function callInitialize(exports_: Record<string, unknown>): void {
  const init = exports_['_initialize'] as (() => void) | undefined;
  if (typeof init === 'function') init();
}

const enc = new TextEncoder();
const dec = new TextDecoder();

export class WasmAbi {
  private readonly exports: AfExports;

  constructor(exports: AfExports) {
    this.exports = exports;
  }

  async call(request: unknown): Promise<unknown> {
    const bytes = enc.encode(JSON.stringify(request));
    if (bytes.length > LIMITS.max_request_bytes) {
      throw new AffidavitRequestTooLargeError(bytes.length, LIMITS.max_request_bytes);
    }
    const ptr = this.exports.af_alloc(bytes.length);
    if (ptr === 0) {
      throw new AffidavitAllocRefusedError(
        `af_alloc refused ${bytes.length} bytes (over request/limit or outstanding cap)`,
      );
    }
    this.view().set(bytes, ptr);
    const packed = this.exports.af_call(ptr, bytes.length);
    const packedBig = typeof packed === 'bigint' ? packed : BigInt(packed);
    const outPtr = Number(packedBig >> 32n);
    const outLen = Number(packedBig & 0xffffffffn);
    try {
      const out = dec.decode(this.view().subarray(outPtr, outPtr + outLen));
      return JSON.parse(out);
    } finally {
      this.exports.af_free(outPtr, outLen);
    }
  }

  async callJson(requestJson: string): Promise<string> {
    const response = (await this.call(JSON.parse(requestJson))) as unknown;
    return JSON.stringify(response);
  }

  private view(): Uint8Array {
    return new Uint8Array(this.exports.memory.buffer);
  }
}
