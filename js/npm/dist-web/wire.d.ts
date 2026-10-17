export interface AfExports {
    memory: WebAssembly.Memory;
    af_alloc: (len: number) => number;
    af_call: (ptr: number, len: number) => bigint | number;
    af_free: (ptr: number, len: number) => void;
    af_abi_version?: () => number;
    affi_admit_and_seal?: (ptr: number, len: number) => bigint | number;
}
export declare function hasInitialize(exports_: Record<string, unknown>): boolean;
export declare function callInitialize(exports_: Record<string, unknown>): void;
export declare class WasmAbi {
    private readonly exports;
    constructor(exports: AfExports);
    call(request: unknown): Promise<unknown>;
    callJson(requestJson: string): Promise<string>;
    private view;
}
//# sourceMappingURL=wire.d.ts.map