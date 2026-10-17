import { initAffidavit } from './init.js';
export async function callRaw(request, options) {
    const wire = await initAffidavit(options);
    return (await wire.call(request));
}
export async function callRawJson(requestJson, options) {
    const wire = await initAffidavit(options);
    return await wire.callJson(requestJson);
}
