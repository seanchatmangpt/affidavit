# @seanchatmangpt/affidavit

Zero-trust cryptographic witness verification, process mining, and post-quantum attestation over WebAssembly.

Designed for isomorphic execution across **Next.js**, browsers, Web Workers, Cloudflare Workers, and Node.js with zero CLI or native dependencies.

---

## Features

- **100% Headless & Framework Ready**: Clean ECMAScript Modules (`dist-web`, `dist-bundler`) and Node.js (`dist-nodejs`) distributions with full tree-shaking support.
- **Cryptographic Attestations**:
  - RFC 8785 JSON Canonicalization Scheme (JCS).
  - BLAKE3 payload commitments and chain integrity verification.
  - Sparse Merkle Tree (SMT) inclusion and absence proofs.
  - Zero-Knowledge Bulletproofs range proof verification.
  - Post-Quantum Cryptography: FIPS 204 ($\mathsf{ML\text{-}DSA}$) & FIPS 205 ($\mathsf{SLH\text{-}DSA}$).
  - AuthZEN & SPIFFE evidence certification.
- **Process Mining & Conformance**:
  - Direct Directly-Follows Graphs (DFG) extraction and footprint analysis.
  - Token replay and trace fitness conformance checking.
- **Formally Verified**:
  - Authored with accompanying mathematical dissertation ([`THESIS.md`](./THESIS.md)).

---

## Installation

```bash
npm install @seanchatmangpt/affidavit
```

---

## Quickstart

### Node.js / Server

```typescript
import { createClient } from '@seanchatmangpt/affidavit';

const client = await createClient();

// Query engine capabilities
const caps = await client.capabilities();
console.log(caps.ops); // ['commit', 'assemble', 'verify', 'mine', ...]

// Compute BLAKE3 commitment
const res = await client.commit('hello state transition');
console.log(res.commitment);

// Canonicalize JSON (RFC 8785)
const jcs = await client.jcsCanonicalize('{"b":1,"a":2}');
console.log(jcs.canonical_json); // '{"a":2,"b":1}'
```

### Next.js / Web Bundler

```typescript
import { createClient } from '@seanchatmangpt/affidavit';

// Automatically uses dist-web / dist-bundler with zero Node.js polyfill requirements
const client = await createClient({ runtime: 'web' });
const result = await client.verify(receipt);
```

---

## Academic Foundation

Read the full PhD dissertation analyzing the sheaf-theoretic algebraic topology, Lagrangian policy dynamics, and lattice cryptography of Affidavit in [`THESIS.md`](./THESIS.md).

---

## License

MIT OR Apache-2.0
