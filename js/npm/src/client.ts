import { AffidavitRefusalError } from './errors.js';
import type { AbiOk, AbiResult } from './types.js';
import type {
  CapabilitiesRequest,
  CapabilitiesResponse,
  CommitRequest,
  CommitResponse,
  AssembleRequest,
  AssembleResponse,
  VerifyRequest,
  VerifyResponse,
  MineRequest,
  MineResponse,
  ConformRequest,
  ConformResponse,
  VerifySignatureInputRequest,
  VerifySignatureInputResponse,
  CertifyAuthzenEvidenceRequest,
  CertifyAuthzenEvidenceResponse,
  CertifySpiffeEvidenceRequest,
  CertifySpiffeEvidenceResponse,
  JcsCanonicalizeRequest,
  JcsCanonicalizeResponse,
  SmtAbsenceVerifyRequest,
  SmtAbsenceVerifyResponse,
  RangeProofVerifyRequest,
  RangeProofVerifyResponse,
  DeriveSubjectDigestRequest,
  DeriveSubjectDigestResponse,
  SmtInclusionVerifyRequest,
  SmtInclusionVerifyResponse,
  VerifySignatureRequest,
  VerifySignatureResponse,
  PqcAssembleRequest,
  PqcAssembleResponse,
  PqcVerifyRequest,
  PqcVerifyResponse,
} from './generated/types.js';
import type { WasmAbi } from './wire.js';
import { initAffidavit, type AffidavitInitOptions } from './init.js';

export type RawRequest = Record<string, unknown>;

export class Affidavit {
  private readonly abi: Promise<WasmAbi>;

  constructor(abi: Promise<WasmAbi>) {
    this.abi = abi;
  }

  private async dispatch<T>(request: RawRequest): Promise<AbiOk<T>> {
    const wire = await this.abi;
    const envelope = (await wire.call(request)) as any;
    if (!envelope.ok) {
      throw new AffidavitRefusalError(
        envelope.error || envelope.reason || 'refused',
        envelope.code || envelope.error
      );
    }
    return envelope as AbiOk<T>;
  }

  private async tryDispatch<T>(request: RawRequest): Promise<AbiResult<T>> {
    const wire = await this.abi;
    return (await wire.call(request)) as AbiResult<T>;
  }

  async capabilities(req: Omit<CapabilitiesRequest, 'op'> = {}): Promise<AbiOk<CapabilitiesResponse>> {
    return this.dispatch<CapabilitiesResponse>({ op: 'capabilities', ...req });
  }

  async commit(payload: string): Promise<AbiOk<CommitResponse>> {
    return this.dispatch<CommitResponse>({ op: 'commit', payload });
  }

  async assemble(events: any[]): Promise<AbiOk<AssembleResponse>> {
    return this.dispatch<AssembleResponse>({ op: 'assemble', events });
  }

  async verify(receipt: Record<string, unknown>): Promise<AbiOk<VerifyResponse>> {
    return this.dispatch<VerifyResponse>({ op: 'verify', receipt });
  }

  async mine(receipts: any[]): Promise<AbiOk<MineResponse>> {
    return this.dispatch<MineResponse>({ op: 'mine', receipts });
  }

  async conform(model: any[], trace: any): Promise<AbiOk<ConformResponse>> {
    return this.dispatch<ConformResponse>({ op: 'conform', model, trace });
  }

  async jcsCanonicalize(json: string): Promise<AbiOk<JcsCanonicalizeResponse>> {
    return this.dispatch<JcsCanonicalizeResponse>({ op: 'jcs_canonicalize', json });
  }

  async deriveSubjectDigest(domainTag: string, subject: string): Promise<AbiOk<DeriveSubjectDigestResponse>> {
    return this.dispatch<DeriveSubjectDigestResponse>({ op: 'derive_subject_digest', domain_tag: domainTag, subject });
  }

  async verifySignature(req: Omit<VerifySignatureRequest, 'op'>): Promise<AbiOk<VerifySignatureResponse>> {
    return this.dispatch<VerifySignatureResponse>({ op: 'verify_signature', ...req });
  }

  async pqcVerify(req: Omit<PqcVerifyRequest, 'op'>): Promise<AbiOk<PqcVerifyResponse>> {
    return this.dispatch<PqcVerifyResponse>({ op: 'pqc_verify', ...req });
  }

  async raw<T = unknown>(req: RawRequest): Promise<AbiResult<T>> {
    return this.tryDispatch<T>(req);
  }
}

export async function createClient(options?: AffidavitInitOptions): Promise<Affidavit> {
  const abi = initAffidavit(options);
  await abi;
  return new Affidavit(abi);
}
