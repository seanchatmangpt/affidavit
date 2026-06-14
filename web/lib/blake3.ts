// Dependency-free, pure-TypeScript BLAKE3 (256-bit / 32-byte output).
//
// This is a faithful port of the BLAKE3 reference specification:
//   - 16-word (64-byte) message blocks, 8-word (32-byte) chaining values
//   - the compression function with 7 rounds and the fixed G mixing function
//   - the message permutation applied between rounds
//   - 1024-byte chunks split into 16 blocks, each compressed in sequence with
//     CHUNK_START on the first block and CHUNK_END on the last
//   - a binary tree of chaining values combined with PARENT, and ROOT set when
//     producing the final output
//
// We implement only the default hash mode (no keyed hashing, no key derivation)
// and a single 32-byte output, which is all affidavit's chain rule needs
// (`blake3::hash(bytes).to_hex()` in the Rust types). Correctness is verified at
// module load against the official empty-input test vector.
//
// All arithmetic is done on unsigned 32-bit words via `>>> 0` / `| 0` and
// `Math.imul`, matching BLAKE3's little-endian 32-bit word model exactly.

// ── Constants ───────────────────────────────────────────────────────────────

// IV: first 32 bits of the fractional parts of the square roots of the first
// 8 primes (the SHA-256 initial hash values).
const IV: readonly number[] = [
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
  0x1f83d9ab, 0x5be0cd19,
];

// Domain-separation flags.
const CHUNK_START = 1 << 0;
const CHUNK_END = 1 << 1;
const PARENT = 1 << 2;
const ROOT = 1 << 3;

const BLOCK_LEN = 64; // bytes per message block
const CHUNK_LEN = 1024; // bytes per chunk
const OUT_LEN = 32; // default output length in bytes

// The message word permutation applied after each round (except the last).
const MSG_PERMUTATION: readonly number[] = [
  2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8,
];

// ── Core compression ─────────────────────────────────────────────────────────

function rotr(x: number, n: number): number {
  return ((x >>> n) | (x << (32 - n))) >>> 0;
}

// The G mixing function operates on 4 words of the 16-word state plus 2 message
// words. We mutate the `state` array in place at the given indices.
function g(
  state: Int32Array | Uint32Array,
  a: number,
  b: number,
  c: number,
  d: number,
  mx: number,
  my: number,
): void {
  state[a] = (state[a] + state[b] + mx) >>> 0;
  state[d] = rotr((state[d] ^ state[a]) >>> 0, 16);
  state[c] = (state[c] + state[d]) >>> 0;
  state[b] = rotr((state[b] ^ state[c]) >>> 0, 12);
  state[a] = (state[a] + state[b] + my) >>> 0;
  state[d] = rotr((state[d] ^ state[a]) >>> 0, 8);
  state[c] = (state[c] + state[d]) >>> 0;
  state[b] = rotr((state[b] ^ state[c]) >>> 0, 7);
}

function round(state: Uint32Array, m: Uint32Array): void {
  // Mix the columns.
  g(state, 0, 4, 8, 12, m[0], m[1]);
  g(state, 1, 5, 9, 13, m[2], m[3]);
  g(state, 2, 6, 10, 14, m[4], m[5]);
  g(state, 3, 7, 11, 15, m[6], m[7]);
  // Mix the diagonals.
  g(state, 0, 5, 10, 15, m[8], m[9]);
  g(state, 1, 6, 11, 12, m[10], m[11]);
  g(state, 2, 7, 8, 13, m[12], m[13]);
  g(state, 3, 4, 9, 14, m[14], m[15]);
}

function permute(m: Uint32Array): Uint32Array {
  const permuted = new Uint32Array(16);
  for (let i = 0; i < 16; i++) permuted[i] = m[MSG_PERMUTATION[i]];
  return permuted;
}

// The compression function. Returns the full 16-word state (the first 8 words
// are the truncated chaining value; all 16 are used for extended/root output).
function compress(
  chainingValue: Uint32Array, // 8 words
  blockWords: Uint32Array, // 16 words
  counter: bigint, // 64-bit chunk counter
  blockLen: number,
  flags: number,
): Uint32Array {
  const counterLow = Number(counter & 0xffffffffn) >>> 0;
  const counterHigh = Number((counter >> 32n) & 0xffffffffn) >>> 0;

  const state = new Uint32Array(16);
  state[0] = chainingValue[0];
  state[1] = chainingValue[1];
  state[2] = chainingValue[2];
  state[3] = chainingValue[3];
  state[4] = chainingValue[4];
  state[5] = chainingValue[5];
  state[6] = chainingValue[6];
  state[7] = chainingValue[7];
  state[8] = IV[0];
  state[9] = IV[1];
  state[10] = IV[2];
  state[11] = IV[3];
  state[12] = counterLow;
  state[13] = counterHigh;
  state[14] = blockLen >>> 0;
  state[15] = flags >>> 0;

  let m = blockWords;
  round(state, m); // round 1
  m = permute(m);
  round(state, m); // round 2
  m = permute(m);
  round(state, m); // round 3
  m = permute(m);
  round(state, m); // round 4
  m = permute(m);
  round(state, m); // round 5
  m = permute(m);
  round(state, m); // round 6
  m = permute(m);
  round(state, m); // round 7

  // Feed-forward: first 8 words XOR the last 8; last 8 words XOR the input CV.
  for (let i = 0; i < 8; i++) {
    state[i] = (state[i] ^ state[i + 8]) >>> 0;
    state[i + 8] = (state[i + 8] ^ chainingValue[i]) >>> 0;
  }
  return state;
}

// Read a 64-byte block into 16 little-endian 32-bit words. `block` must be
// exactly 64 bytes (callers zero-pad the final partial block).
function wordsFromBlock(block: Uint8Array): Uint32Array {
  const words = new Uint32Array(16);
  for (let i = 0; i < 16; i++) {
    const o = i * 4;
    words[i] =
      ((block[o] |
        (block[o + 1] << 8) |
        (block[o + 2] << 16) |
        (block[o + 3] << 24)) >>>
        0);
  }
  return words;
}

// ── Chunk state ───────────────────────────────────────────────────────────────
//
// A chunk hashes up to 1024 bytes (16 blocks). We process complete 64-byte
// blocks eagerly, keeping a rolling chaining value, and defer the final block so
// CHUNK_END (and possibly ROOT) can be applied to it.

interface Output {
  inputChainingValue: Uint32Array; // 8 words
  blockWords: Uint32Array; // 16 words
  counter: bigint;
  blockLen: number;
  flags: number;
}

function outputChainingValue(o: Output): Uint32Array {
  const full = compress(
    o.inputChainingValue,
    o.blockWords,
    o.counter,
    o.blockLen,
    o.flags,
  );
  return full.slice(0, 8);
}

// Produce `outLen` bytes of root output (XOF), with the ROOT flag set and the
// output-block counter starting at 0.
function outputRootBytes(o: Output, outLen: number): Uint8Array {
  const out = new Uint8Array(outLen);
  let outputBlockCounter = 0n;
  let pos = 0;
  while (pos < outLen) {
    const words = compress(
      o.inputChainingValue,
      o.blockWords,
      outputBlockCounter,
      o.blockLen,
      (o.flags | ROOT) >>> 0,
    );
    // The full 16-word state is emitted as little-endian bytes.
    for (let i = 0; i < 16 && pos < outLen; i++) {
      const w = words[i];
      for (let b = 0; b < 4 && pos < outLen; b++) {
        out[pos++] = (w >>> (8 * b)) & 0xff;
      }
    }
    outputBlockCounter += 1n;
  }
  return out;
}

class ChunkState {
  chainingValue: Uint32Array; // 8 words
  chunkCounter: bigint;
  block: Uint8Array; // 64-byte buffer for the in-progress block
  blockLen: number; // bytes currently buffered in `block`
  blocksCompressed: number;
  flags: number;

  constructor(keyWords: Uint32Array, chunkCounter: bigint, flags: number) {
    this.chainingValue = keyWords.slice(0, 8);
    this.chunkCounter = chunkCounter;
    this.block = new Uint8Array(BLOCK_LEN);
    this.blockLen = 0;
    this.blocksCompressed = 0;
    this.flags = flags >>> 0;
  }

  len(): number {
    return BLOCK_LEN * this.blocksCompressed + this.blockLen;
  }

  startFlag(): number {
    return this.blocksCompressed === 0 ? CHUNK_START : 0;
  }

  update(input: Uint8Array): void {
    let offset = 0;
    while (offset < input.length) {
      // If the block buffer is full, compress it (this is NOT the last block of
      // the chunk, so no CHUNK_END here) and roll the chaining value forward.
      if (this.blockLen === BLOCK_LEN) {
        const blockWords = wordsFromBlock(this.block);
        const out = compress(
          this.chainingValue,
          blockWords,
          this.chunkCounter,
          BLOCK_LEN,
          (this.flags | this.startFlag()) >>> 0,
        );
        this.chainingValue = out.slice(0, 8);
        this.blocksCompressed += 1;
        this.block = new Uint8Array(BLOCK_LEN);
        this.blockLen = 0;
      }
      const want = BLOCK_LEN - this.blockLen;
      const take = Math.min(want, input.length - offset);
      this.block.set(input.subarray(offset, offset + take), this.blockLen);
      this.blockLen += take;
      offset += take;
    }
  }

  // Produce the Output for the final (possibly partial) block of this chunk,
  // tagged with CHUNK_END. The caller decides whether to set ROOT.
  output(): Output {
    const blockWords = wordsFromBlock(this.block);
    return {
      inputChainingValue: this.chainingValue.slice(0, 8),
      blockWords,
      counter: this.chunkCounter,
      blockLen: this.blockLen,
      flags: (this.flags | this.startFlag() | CHUNK_END) >>> 0,
    };
  }
}

function parentOutput(
  leftChildCv: Uint32Array,
  rightChildCv: Uint32Array,
  flags: number,
): Output {
  const blockWords = new Uint32Array(16);
  blockWords.set(leftChildCv.subarray(0, 8), 0);
  blockWords.set(rightChildCv.subarray(0, 8), 8);
  return {
    inputChainingValue: Uint32Array.from(IV),
    blockWords,
    counter: 0n,
    blockLen: BLOCK_LEN,
    flags: (PARENT | flags) >>> 0,
  };
}

// ── Hasher ────────────────────────────────────────────────────────────────────
//
// Incremental hasher implementing the standard BLAKE3 tree. A stack of subtree
// chaining values is merged whenever the number of completed chunks gains a new
// low bit (the canonical reference algorithm).

class Blake3Hasher {
  private chunkState: ChunkState;
  private readonly keyWords: Uint32Array;
  private readonly flags: number;
  private readonly cvStack: Uint32Array[] = [];

  constructor() {
    this.keyWords = Uint32Array.from(IV);
    this.flags = 0;
    this.chunkState = new ChunkState(this.keyWords, 0n, this.flags);
  }

  private addChunkChainingValue(newCv: Uint32Array, totalChunks: bigint): void {
    // Merge subtrees: while the lowest set bit of totalChunks is 0 we have a
    // matching left subtree on the stack to combine with.
    let cv = newCv;
    let chunks = totalChunks;
    while ((chunks & 1n) === 0n) {
      const left = this.cvStack.pop() as Uint32Array;
      const out = parentOutput(left, cv, this.flags);
      cv = outputChainingValue(out);
      chunks >>= 1n;
    }
    this.cvStack.push(cv);
  }

  update(input: Uint8Array): void {
    let offset = 0;
    while (offset < input.length) {
      // When the current chunk is full, finalize it into a chaining value and
      // start a fresh chunk with the next counter.
      if (this.chunkState.len() === CHUNK_LEN) {
        const cv = outputChainingValue(this.chunkState.output());
        const totalChunks = this.chunkState.chunkCounter + 1n;
        this.addChunkChainingValue(cv, totalChunks);
        this.chunkState = new ChunkState(this.keyWords, totalChunks, this.flags);
      }
      const want = CHUNK_LEN - this.chunkState.len();
      const take = Math.min(want, input.length - offset);
      this.chunkState.update(input.subarray(offset, offset + take));
      offset += take;
    }
  }

  // Compute the root Output by combining the current chunk with the CV stack
  // from the top down (rightmost subtree merges first).
  private rootOutput(): Output {
    let output = this.chunkState.output();
    let parentNodesRemaining = this.cvStack.length;
    while (parentNodesRemaining > 0) {
      parentNodesRemaining -= 1;
      const left = this.cvStack[parentNodesRemaining];
      output = parentOutput(left, outputChainingValue(output), this.flags);
    }
    return output;
  }

  finalize(outLen: number = OUT_LEN): Uint8Array {
    return outputRootBytes(this.rootOutput(), outLen);
  }
}

// ── Public API ────────────────────────────────────────────────────────────────

/** Normalize a string|Uint8Array input to bytes (UTF-8 for strings). */
function toBytes(input: Uint8Array | string): Uint8Array {
  if (typeof input === "string") return new TextEncoder().encode(input);
  return input;
}

/** Compute the 32-byte BLAKE3 digest of the input. */
export function blake3Bytes(input: Uint8Array | string): Uint8Array {
  const hasher = new Blake3Hasher();
  hasher.update(toBytes(input));
  return hasher.finalize(OUT_LEN);
}

/** Lowercase-hex helper for an arbitrary byte buffer. */
export function toHex(bytes: Uint8Array): string {
  let s = "";
  for (let i = 0; i < bytes.length; i++) {
    s += bytes[i].toString(16).padStart(2, "0");
  }
  return s;
}

/**
 * Compute the BLAKE3 digest of the input and return it as a 64-char lowercase
 * hex string — the exact form affidavit stores in `Blake3Hash` (matches
 * `blake3::hash(bytes).to_hex().to_string()` in the Rust types).
 */
export function blake3Hex(input: Uint8Array | string): string {
  return toHex(blake3Bytes(input));
}

// ── Self-test ─────────────────────────────────────────────────────────────────

/** The official BLAKE3 digest of the empty input. */
export const EMPTY_VECTOR =
  "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

/**
 * Run BLAKE3 against published test vectors. Returns `{ ok, failures }`.
 *
 * Vectors are the official ones from the BLAKE3 reference repository
 * (`test_vectors.json`), where input of length N is the repeating byte sequence
 * 0,1,2,...,250,0,1,... and the expected value is the first 64 hex chars of the
 * extended output. We check the empty input plus several lengths that exercise
 * single-block, multi-block, full-chunk, and multi-chunk (tree) paths.
 */
export function selfTest(): { ok: boolean; failures: string[] } {
  const failures: string[] = [];

  // Official reference vectors: input length -> first 64 hex of the hash.
  const vectors: ReadonlyArray<readonly [number, string]> = [
    [0, "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"],
    [1, "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"],
    [2, "7b7015bb92cf0b318037702a6cdd81dee41224f734684c2c122cd6359cb1ee63"],
    [3, "e1be4d7a8ab5560aa4199eea339849ba8e293d55ca0a81006726d184519e647f"],
    [4, "f30f5ab28fe047904037f77b6da4fea1e27241c5d132638d8bedce9d40494f32"],
    [7, "3f8770f387faad08faa9d8414e9f449ac68e6ff0417f673f602a646a891419fe"],
    [8, "2351207d04fc16ade43ccab08600939c7c1fa70a5c0aaca76063d04c3228eaeb"],
    [63, "e9bc37a594daad83be9470df7f7b3798297c3d834ce80ba85d6e207627b7db7b"],
    [64, "4eed7141ea4a5cd4b788606bd23f46e212af9cacebacdc7d1f4c6dc7f2511b98"],
    [65, "de1e5fa0be70df6d2be8fffd0e99ceaa8eb6e8c93a63f2d8d1c30ecb6b263dee"],
    [1023, "10108970eeda3eb932baac1428c7a2163b0e924c9a9e25b35bba72b28f70bd11"],
    [1024, "42214739f095a406f3fc83deb889744ac00df831c10daa55189b5d121c855af7"],
    [1025, "d00278ae47eb27b34faecf67b4fe263f82d5412916c1ffd97c8cb7fb814b8444"],
    [2048, "e776b6028c7cd22a4d0ba182a8bf62205d2ef576467e838ed6f2529b85fba24a"],
    [3072, "b98cb0ff3623be03326b373de6b9095218513e64f1ee2edd2525c7ad1e5cffd2"],
    [4096, "015094013f57a5277b59d8475c0501042c0b642e531b0a1c8f58d2163229e969"],
    [5120, "9cadc15fed8b5d854562b26a9536d9707cadeda9b143978f319ab34230535833"],
  ];

  for (const [len, expected] of vectors) {
    const input = referenceInput(len);
    // We only emit 32 bytes (64 hex); the reference's first 64 hex chars are the
    // default 32-byte hash, so this is a direct comparison.
    const got = blake3Hex(input);
    if (got !== expected) {
      failures.push(`len=${len}: expected ${expected}, got ${got}`);
    }
  }

  return { ok: failures.length === 0, failures };
}

/** BLAKE3 reference test input of length `len`: bytes 0..250 repeating. */
function referenceInput(len: number): Uint8Array {
  const buf = new Uint8Array(len);
  for (let i = 0; i < len; i++) buf[i] = i % 251;
  return buf;
}

// Assert the empty-input vector at module load so any regression is loud during
// development. We do not throw in production builds (the affidavit doctrine
// prefers an honest surfaced error to a hard crash), but a mismatch must never
// ship — the studio also runs `selfTest()` and renders the result.
if (process.env.NODE_ENV !== "production") {
  const result = selfTest();
  if (!result.ok) {
    // eslint-disable-next-line no-console
    console.error("[blake3] SELF-TEST FAILED:\n" + result.failures.join("\n"));
    throw new Error(
      "blake3 self-test failed at module load: " + result.failures.join("; "),
    );
  }
}
