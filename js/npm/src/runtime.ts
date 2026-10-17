import { AffidavitInitError } from './errors.js';
import { callInitialize, hasInitialize, WasmAbi } from './wire.js';

export async function loadWasmBytes(): Promise<Uint8Array> {
  const { readFile } = await import('node:fs/promises');
  const { fileURLToPath } = await import('node:url');
  const wasmPath = fileURLToPath(new URL('../wasm/affidavit.wasm', import.meta.url));
  try {
    return await readFile(wasmPath);
  } catch (e) {
    throw new AffidavitInitError(`affidavit: failed to read ${wasmPath}`, { cause: e });
  }
}

const ERRNO_SUCCESS = 0;

export async function instantiateFromBytes(
  bytes: Uint8Array,
): Promise<WasmAbi> {
  let randomFillSync: (buf: Uint8Array) => Uint8Array;
  try {
    const nodeCrypto = await import('node:crypto');
    randomFillSync = nodeCrypto.randomFillSync;
  } catch (e) {
    throw new AffidavitInitError('affidavit: node:crypto unavailable', { cause: e });
  }

  let memory: WebAssembly.Memory | null = null;
  const mem = (): WebAssembly.Memory => {
    if (!memory) throw new AffidavitInitError('affidavit: import called before instantiation');
    return memory;
  };

  const wasiSnapshotPreview1: Record<string, unknown> = {
    environ_sizes_get: (envCountPtr: number, envBufSizePtr: number): number => {
      const view = new DataView(mem().buffer);
      view.setUint32(envCountPtr, 0, true);
      view.setUint32(envBufSizePtr, 0, true);
      return ERRNO_SUCCESS;
    },
    environ_get: (): number => ERRNO_SUCCESS,
    sched_yield: (): number => ERRNO_SUCCESS,
    random_get: (bufPtr: number, bufLen: number): number => {
      randomFillSync(new Uint8Array(mem().buffer, bufPtr, bufLen));
      return ERRNO_SUCCESS;
    },
    fd_write: (fd: number, iovecsPtr: number, iovecsLen: number, nwrittenPtr: number): number => {
      void fd;
      const view = new DataView(mem().buffer);
      let written = 0;
      for (let i = 0; i < iovecsLen; i++) {
        written += view.getUint32(iovecsPtr + i * 8 + 4, true);
      }
      view.setUint32(nwrittenPtr, written, true);
      return ERRNO_SUCCESS;
    },
    clock_time_get: (clockId: number, precision: bigint, timePtr: number): number => {
      void clockId;
      void precision;
      const view = new DataView(mem().buffer);
      const nanos = BigInt(Math.round(performance.now() * 1e6));
      view.setBigUint64(timePtr, nanos, true);
      return ERRNO_SUCCESS;
    },
    proc_exit: (code: number): never => {
      throw new AffidavitInitError(`affidavit: module called proc_exit(${code})`);
    },
  };

  const imports = { wasi_snapshot_preview1: wasiSnapshotPreview1 };

  let instance: WebAssembly.Instance;
  try {
    const module_ = await WebAssembly.compile(bytes as unknown as BufferSource);
    instance = await WebAssembly.instantiate(module_, imports as WebAssembly.Imports);
  } catch (e) {
    throw new AffidavitInitError('affidavit: failed to instantiate the wasm module', { cause: e });
  }
  memory = (instance.exports as { memory: WebAssembly.Memory }).memory;
  const exports_ = instance.exports as unknown as Record<string, unknown>;
  if (hasInitialize(exports_)) {
    const init = exports_['_initialize'] as () => void;
    init();
  } else {
    callInitialize(exports_);
  }
  return new WasmAbi(exports_ as never);
}
