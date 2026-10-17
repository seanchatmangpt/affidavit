import { AffidavitInitError, AffidavitIntegrityError } from './errors.js';
import type { WasmAbi } from './wire.js';
export type AffidavitRuntime = 'node' | 'web';
export interface AffidavitInitOptions {
    wasmBytes?: Uint8Array;
    skipShaCheck?: boolean;
    runtime?: AffidavitRuntime;
}
export declare function initAffidavit(options?: AffidavitInitOptions): Promise<WasmAbi>;
export { AffidavitInitError, AffidavitIntegrityError };
//# sourceMappingURL=init.d.ts.map