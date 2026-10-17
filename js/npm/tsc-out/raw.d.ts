import { type AffidavitInitOptions } from './init.js';
import type { AbiResult } from './types.js';
export declare function callRaw<T = unknown>(request: Record<string, unknown>, options?: AffidavitInitOptions): Promise<AbiResult<T>>;
export declare function callRawJson(requestJson: string, options?: AffidavitInitOptions): Promise<string>;
//# sourceMappingURL=raw.d.ts.map