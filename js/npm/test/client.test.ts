import { describe, expect, it } from 'vitest';
import { createClient } from '../src/index.js';

describe('@seanchatmangpt/affidavit client', () => {
  it('instantiates and queries capabilities', async () => {
    const client = await createClient();
    const caps = await client.capabilities();
    expect(caps.ok).toBe(true);
    expect(caps.module).toBe('affidavit-wasm');
    expect(caps.ops).toContain('commit');
    expect(caps.ops).toContain('jcs_canonicalize');
    expect(caps.ops).toContain('verify');
  });

  it('computes BLAKE3 commitment via commit op', async () => {
    const client = await createClient();
    const res = await client.commit('test payload');
    expect(res.ok).toBe(true);
    expect(res.commitment).toBe('d10d37c134940863c5c10eda4909f32b071c7d3be66cb5a0975a67fb30a0c281');
  });

  it('canonicalizes JSON via JCS', async () => {
    const client = await createClient();
    const res = await client.jcsCanonicalize('{"b":1,"a":2}');
    expect(res.ok).toBe(true);
    expect(res.canonical_json).toBe('{"a":2,"b":1}');
  });

  it('derives subject digest', async () => {
    const client = await createClient();
    const res = await client.deriveSubjectDigest('affidavit-paid-delivery/v1', 'paid-delivery:evt-001');
    expect(res.ok).toBe(true);
    expect(res.digest_hex).toBeDefined();
  });
});
