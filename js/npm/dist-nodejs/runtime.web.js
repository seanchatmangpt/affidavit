import { AffidavitInitError } from './errors.js';
import { callInitialize, hasInitialize, WasmAbi } from './wire.js';
const WASM_URL = new URL('../wasm/affidavit.wasm', import.meta.url);
export async function loadWasmBytes() {
    let response;
    try {
        response = await fetch(WASM_URL);
    }
    catch (e) {
        throw new AffidavitInitError('affidavit: failed to fetch the wasm module', { cause: e });
    }
    if (!response.ok) {
        throw new AffidavitInitError(`affidavit: failed to fetch the wasm module (HTTP ${response.status})`);
    }
    try {
        return new Uint8Array(await response.arrayBuffer());
    }
    catch (e) {
        throw new AffidavitInitError('affidavit: failed to read the wasm module bytes', { cause: e });
    }
}
const ERRNO_SUCCESS = 0;
export async function instantiateFromBytes(bytes) {
    let memory = null;
    const mem = () => {
        if (!memory)
            throw new AffidavitInitError('affidavit: import called before instantiation');
        return memory;
    };
    const wasiSnapshotPreview1 = {
        environ_sizes_get: (envCountPtr, envBufSizePtr) => {
            const view = new DataView(mem().buffer);
            view.setUint32(envCountPtr, 0, true);
            view.setUint32(envBufSizePtr, 0, true);
            return ERRNO_SUCCESS;
        },
        environ_get: () => ERRNO_SUCCESS,
        sched_yield: () => ERRNO_SUCCESS,
        random_get: (bufPtr, bufLen) => {
            globalThis.crypto.getRandomValues(new Uint8Array(mem().buffer, bufPtr, bufLen));
            return ERRNO_SUCCESS;
        },
        fd_write: (fd, iovecsPtr, iovecsLen, nwrittenPtr) => {
            void fd;
            const view = new DataView(mem().buffer);
            let written = 0;
            for (let i = 0; i < iovecsLen; i++) {
                written += view.getUint32(iovecsPtr + i * 8 + 4, true);
            }
            view.setUint32(nwrittenPtr, written, true);
            return ERRNO_SUCCESS;
        },
        clock_time_get: (clockId, precision, timePtr) => {
            void clockId;
            void precision;
            const view = new DataView(mem().buffer);
            const nanos = BigInt(Math.round(performance.now() * 1e6));
            view.setBigUint64(timePtr, nanos, true);
            return ERRNO_SUCCESS;
        },
        proc_exit: (code) => {
            throw new AffidavitInitError(`affidavit: module called proc_exit(${code})`);
        },
    };
    const imports = { wasi_snapshot_preview1: wasiSnapshotPreview1 };
    let instance;
    try {
        if (bytes instanceof Response) {
            const result = await WebAssembly.instantiateStreaming(bytes, imports);
            instance = result.instance;
        }
        else {
            const module_ = await WebAssembly.compile(bytes);
            instance = await WebAssembly.instantiate(module_, imports);
        }
    }
    catch (e) {
        throw new AffidavitInitError('affidavit: failed to instantiate the wasm module', { cause: e });
    }
    memory = instance.exports.memory;
    const exports_ = instance.exports;
    if (hasInitialize(exports_)) {
        const init = exports_['_initialize'];
        init();
    }
    else {
        callInitialize(exports_);
    }
    return new WasmAbi(exports_);
}
