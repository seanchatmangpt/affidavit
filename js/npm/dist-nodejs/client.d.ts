import type { AbiOk, AbiResult } from './types.js';
import type { CapabilitiesRequest, CapabilitiesResponse, CommitResponse, AssembleResponse, VerifyResponse, MineResponse, ConformResponse, JcsCanonicalizeResponse, DeriveSubjectDigestResponse, VerifySignatureRequest, VerifySignatureResponse, PqcVerifyRequest, PqcVerifyResponse } from './generated/types.js';
import type { WasmAbi } from './wire.js';
import { type AffidavitInitOptions } from './init.js';
export type RawRequest = Record<string, unknown>;
export declare class Affidavit {
    private readonly abi;
    constructor(abi: Promise<WasmAbi>);
    private dispatch;
    private tryDispatch;
    capabilities(req?: Omit<CapabilitiesRequest, 'op'>): Promise<AbiOk<CapabilitiesResponse>>;
    commit(payload: string): Promise<AbiOk<CommitResponse>>;
    assemble(events: any[]): Promise<AbiOk<AssembleResponse>>;
    verify(receipt: Record<string, unknown>): Promise<AbiOk<VerifyResponse>>;
    mine(receipts: any[]): Promise<AbiOk<MineResponse>>;
    conform(model: any[], trace: any): Promise<AbiOk<ConformResponse>>;
    jcsCanonicalize(json: string): Promise<AbiOk<JcsCanonicalizeResponse>>;
    deriveSubjectDigest(domainTag: string, subject: string): Promise<AbiOk<DeriveSubjectDigestResponse>>;
    verifySignature(req: Omit<VerifySignatureRequest, 'op'>): Promise<AbiOk<VerifySignatureResponse>>;
    pqcVerify(req: Omit<PqcVerifyRequest, 'op'>): Promise<AbiOk<PqcVerifyResponse>>;
    raw<T = unknown>(req: RawRequest): Promise<AbiResult<T>>;
}
export declare function createClient(options?: AffidavitInitOptions): Promise<Affidavit>;
//# sourceMappingURL=client.d.ts.map