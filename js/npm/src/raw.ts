import { initAffidavit, type AffidavitInitOptions } from './init.js';
import type { AbiResult } from './types.js';

export async function callRaw<T = unknown>(
  request: Record<string, unknown>,
  options?: AffidavitInitOptions,
): Promise<AbiResult<T>> {
  const wire = await initAffidavit(options);
  return (await wire.call(request)) as AbiResult<T>;
}

export async function callRawJson(
  requestJson: string,
  options?: AffidavitInitOptions,
): Promise<string> {
  const wire = await initAffidavit(options);
  return await wire.callJson(requestJson);
}
