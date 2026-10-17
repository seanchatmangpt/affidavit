import { AffidavitRefusalError } from './errors.js';
import { initAffidavit } from './init.js';
export class Affidavit {
    abi;
    constructor(abi) {
        this.abi = abi;
    }
    async dispatch(request) {
        const wire = await this.abi;
        const envelope = (await wire.call(request));
        if (!envelope.ok) {
            throw new AffidavitRefusalError(envelope.error || envelope.reason || 'refused', envelope.code || envelope.error);
        }
        return envelope;
    }
    async tryDispatch(request) {
        const wire = await this.abi;
        return (await wire.call(request));
    }
    async capabilities(req = {}) {
        return this.dispatch({ op: 'capabilities', ...req });
    }
    async commit(payload) {
        return this.dispatch({ op: 'commit', payload });
    }
    async assemble(events) {
        return this.dispatch({ op: 'assemble', events });
    }
    async verify(receipt) {
        return this.dispatch({ op: 'verify', receipt });
    }
    async mine(receipts) {
        return this.dispatch({ op: 'mine', receipts });
    }
    async conform(model, trace) {
        return this.dispatch({ op: 'conform', model, trace });
    }
    async jcsCanonicalize(json) {
        return this.dispatch({ op: 'jcs_canonicalize', json });
    }
    async deriveSubjectDigest(domainTag, subject) {
        return this.dispatch({ op: 'derive_subject_digest', domain_tag: domainTag, subject });
    }
    async verifySignature(req) {
        return this.dispatch({ op: 'verify_signature', ...req });
    }
    async pqcVerify(req) {
        return this.dispatch({ op: 'pqc_verify', ...req });
    }
    async raw(req) {
        return this.tryDispatch(req);
    }
}
export async function createClient(options) {
    const abi = initAffidavit(options);
    await abi;
    return new Affidavit(abi);
}
