import { AffidavitInitError, AffidavitIntegrityError } from './errors.js';
import { WASM_SHA256 } from './generated/artifact-meta.js';
import type { WasmAbi } from './wire.js';

export type AffidavitRuntime = 'node' | 'web';

export interface AffidavitInitOptions {
  wasmBytes?: Uint8Array;
  skipShaCheck?: boolean;
  runtime?: AffidavitRuntime;
}

const isWeb =
  typeof window !== 'undefined' && typeof document !== 'undefined';

async function loadRuntime(
  runtime: AffidavitRuntime,
): Promise<typeof import('./runtime.js')> {
  return runtime === 'web' ? import('./runtime.web.js') : import('./runtime.js');
}

let cached: Promise<WasmAbi> | undefined;

export async function initAffidavit(options: AffidavitInitOptions = {}): Promise<WasmAbi> {
  if (options.wasmBytes === undefined && cached) return cached;
  const promise = (async () => {
    const rt = await loadRuntime(options.runtime ?? (isWeb ? 'web' : 'node'));
    const bytes = options.wasmBytes ?? (await rt.loadWasmBytes());
    if (!options.skipShaCheck) {
      const actual = await sha256Hex(bytes);
      if (actual !== WASM_SHA256) throw new AffidavitIntegrityError(WASM_SHA256, actual);
    }
    try {
      return await rt.instantiateFromBytes(bytes);
    } catch (e) {
      if (e instanceof AffidavitInitError) throw e;
      throw new AffidavitInitError('affidavit: init failed', { cause: e });
    }
  })();
  if (options.wasmBytes === undefined) cached = promise;
  return promise;
}

async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes as unknown as BufferSource);
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

export { AffidavitInitError, AffidavitIntegrityError };
