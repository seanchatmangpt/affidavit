// Pure, dependency-free in-browser port of affidavit's certify pipeline.
//
// This is a faithful translation of the Rust logic in:
//   - src/chain.rs       (GENESIS_SEED, recompute_chain, fold_event)
//   - src/verifier.rs    (the 7-stage verify() pipeline)
//   - src/types.rs       (canonical_bytes: sorted-key compact JSON)
//
// Every function here is pure (no I/O, no React) so the studio page can compute
// the SAME verdict the `affi` binary produces, entirely client-side. Nothing is
// fabricated: the verdict is a deterministic function of the bytes the user
// pastes in.

import { blake3Hex } from "./blake3";

// ── Mirror of the verdict shape in web/lib/affidavit.ts ───────────────────────

export interface CheckOutcome {
  stage: string;
  passed: boolean;
  detail: string;
}

export interface Verdict {
  accepted: boolean;
  profile: "core/v1";
  outcomes: CheckOutcome[];
  reason: string;
}

// ── Receipt JSON shape (as deserialized from user input) ──────────────────────
//
// These mirror src/types.rs. We keep them permissive (the verifier is what
// decides validity); parseReceipt only enforces the minimal structural shape so
// the stages have well-typed values to inspect, and reports anything malformed
// honestly rather than guessing.

export interface ObjectRef {
  id: string;
  obj_type: string;
  qualifier: string | null;
}

export interface OperationEvent {
  id: string;
  seq: number;
  event_type: string;
  objects: ObjectRef[];
  payload_commitment: string;
}

export interface Receipt {
  format_version: string;
  events: OperationEvent[];
  chain_hash: string;
}

// ── Constants (must match src/chain.rs) ───────────────────────────────────────

/** GENESIS_SEED = ASCII bytes of this exact string (binds chains to the release). */
export const GENESIS_SEED_STRING = "affidavit-v26.6.17-genesis";

/** The format version this verifier certifies (src/chain.rs FORMAT_VERSION). */
export const STANDARD_VERSION = "core/v1";

const BLAKE3_HEX_LEN = 64;
const WELL_FORMED_HASH = /^[0-9a-f]{64}$/;

// ── Canonical serialization (must match Rust serde_json over a sorted value) ──
//
// canonical_bytes(value) in src/types.rs does:
//   1. serde_json::to_value(value)         — value -> JSON value
//   2. recursively sort every object's keys (BTreeMap, lexicographic by byte)
//   3. serde_json::to_vec(&sorted)         — compact, no whitespace
//
// We reproduce step 3 manually so we control key ordering and number/string
// formatting exactly, instead of trusting JSON.stringify's key order.
//
// serde_json's compact encoding:
//   - objects: {"k":v,...} with NO spaces
//   - arrays:  [v,...] with NO spaces
//   - strings: '"' ... '"' escaping  \"  \\  \b \f \n \r \t  and other control
//     chars < 0x20 as \u00XX. Non-ASCII (>= 0x80) is emitted as raw UTF-8 bytes,
//     NOT \u-escaped (serde_json default). We emit the JS string and let the
//     TextEncoder produce the UTF-8 bytes, matching that exactly.
//   - integers: bare decimal (u64 seq -> e.g. 0, 1, 2)
//   - bools/null: true / false / null

// BTreeMap<String, _> in Rust orders keys by the bytewise (UTF-8) comparison of
// the strings. For the ASCII field names in receipts this is identical to JS's
// default string `<` comparison by UTF-16 code unit; to be safe across any user
// keys we compare by UTF-8 bytes explicitly.
function compareUtf8(a: string, b: string): number {
  const ea = new TextEncoder().encode(a);
  const eb = new TextEncoder().encode(b);
  const n = Math.min(ea.length, eb.length);
  for (let i = 0; i < n; i++) {
    if (ea[i] !== eb[i]) return ea[i] - eb[i];
  }
  return ea.length - eb.length;
}

const HEX = "0123456789abcdef";

/** Encode a JS string as a serde_json-compatible compact JSON string literal. */
function encodeJsonString(s: string): string {
  let out = '"';
  for (const ch of s) {
    const code = ch.codePointAt(0) as number;
    switch (ch) {
      case '"':
        out += '\\"';
        break;
      case "\\":
        out += "\\\\";
        break;
      case "\b":
        out += "\\b";
        break;
      case "\f":
        out += "\\f";
        break;
      case "\n":
        out += "\\n";
        break;
      case "\r":
        out += "\\r";
        break;
      case "\t":
        out += "\\t";
        break;
      default:
        if (code < 0x20) {
          // Other control characters -> \u00XX (serde_json behavior).
          out +=
            "\\u00" + HEX[(code >> 4) & 0xf] + HEX[code & 0xf];
        } else {
          // Printable ASCII and all non-ASCII pass through verbatim; the final
          // UTF-8 encoding produces the same bytes serde_json writes.
          out += ch;
        }
    }
  }
  return out + '"';
}

/**
 * Serialize a parsed JSON value to compact JSON text with all object keys sorted
 * recursively — byte-identical to Rust `canonical_bytes` once UTF-8 encoded.
 *
 * Only accepts values from `JSON.parse` (string | number | boolean | null |
 * array | object). Numbers are emitted via their canonical JSON form; affidavit
 * receipts only contain integer `seq`, which round-trips exactly.
 */
export function canonicalJson(value: unknown): string {
  if (value === null) return "null";
  const t = typeof value;
  if (t === "string") return encodeJsonString(value as string);
  if (t === "boolean") return (value as boolean) ? "true" : "false";
  if (t === "number") {
    const n = value as number;
    if (!Number.isFinite(n)) {
      // serde_json cannot represent these; receipts never contain them. Be loud.
      throw new Error("non-finite number is not representable in canonical JSON");
    }
    // Integers (the only numeric form in a receipt) print without a decimal
    // point; JS String() already yields the canonical form for these.
    return String(n);
  }
  if (Array.isArray(value)) {
    return "[" + value.map((v) => canonicalJson(v)).join(",") + "]";
  }
  if (t === "object") {
    const obj = value as Record<string, unknown>;
    const keys = Object.keys(obj).sort(compareUtf8);
    const parts: string[] = [];
    for (const k of keys) {
      parts.push(encodeJsonString(k) + ":" + canonicalJson(obj[k]));
    }
    return "{" + parts.join(",") + "}";
  }
  throw new Error("unsupported value type in canonical JSON: " + t);
}

/** Canonical UTF-8 bytes of a value (matches Rust `canonical_bytes`). */
export function canonicalBytes(value: unknown): Uint8Array {
  return new TextEncoder().encode(canonicalJson(value));
}

// ── Chain recomputation (must match src/chain.rs) ─────────────────────────────

/**
 * Build the canonical JSON-value form of an OperationEvent for hashing.
 *
 * Rust hashes the serde representation of the `OperationEvent` struct. We
 * reconstruct that value with exactly the struct's fields so canonical_bytes
 * produces identical bytes: { event_type, id, objects:[{id,obj_type,qualifier}],
 * payload_commitment, seq } (key order is irrelevant — canonicalJson sorts).
 */
function eventToCanonicalValue(ev: OperationEvent): Record<string, unknown> {
  return {
    id: ev.id,
    seq: ev.seq,
    event_type: ev.event_type,
    objects: ev.objects.map((o) => ({
      id: o.id,
      obj_type: o.obj_type,
      qualifier: o.qualifier, // null stays null (Option<String>::None)
    })),
    payload_commitment: ev.payload_commitment,
  };
}

/**
 * Recompute the rolling chain hash over an ordered list of events.
 *
 *   h0  = blake3(GENESIS_SEED)                                        (hex)
 *   h_n = blake3( ascii(h_{n-1} hex string) ++ canonicalBytes(ev_n) )
 *
 * Note the fold concatenates the ASCII bytes of the PREVIOUS hash's 64-char hex
 * string (Blake3Hash::as_hex().as_bytes()), not the raw digest bytes. Returns
 * the final hash as a 64-char lowercase hex string. Empty events => h0.
 */
export function recomputeChain(events: OperationEvent[]): string {
  const enc = new TextEncoder();
  let acc = blake3Hex(GENESIS_SEED_STRING); // h0 = blake3(GENESIS_SEED)
  for (const ev of events) {
    const prevHexBytes = enc.encode(acc); // ASCII bytes of the 64-char hex
    const eventBytes = canonicalBytes(eventToCanonicalValue(ev));
    const buf = new Uint8Array(prevHexBytes.length + eventBytes.length);
    buf.set(prevHexBytes, 0);
    buf.set(eventBytes, prevHexBytes.length);
    acc = blake3Hex(buf);
  }
  return acc;
}

/** The genesis hash h0 (blake3 of the genesis seed), exposed for the UI. */
export function genesisHash(): string {
  return blake3Hex(GENESIS_SEED_STRING);
}

// ── Parsing ───────────────────────────────────────────────────────────────────

export type ParseResult =
  | { ok: true; receipt: Receipt; value: unknown }
  | { ok: false; error: string };

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

/**
 * Parse receipt text into a typed Receipt, reporting malformed input honestly.
 *
 * This enforces only the minimal structural shape the pipeline needs (the right
 * fields with the right primitive kinds). The verifier — not the parser — decides
 * whether the receipt is *accepted*; a structurally valid but tampered receipt
 * parses fine and is then REJECTED by a stage, exactly like the Rust CLI path
 * (verify reads the JSON and runs the stages). We also surface the raw parsed
 * value so the UI can show the canonical bytes of what was hashed.
 */
export function parseReceipt(text: string): ParseResult {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch (e) {
    return { ok: false, error: "invalid JSON: " + (e as Error).message };
  }

  if (!isObject(value)) {
    return { ok: false, error: "receipt must be a JSON object" };
  }

  const fv = value["format_version"];
  if (typeof fv !== "string") {
    return { ok: false, error: 'missing or non-string "format_version"' };
  }

  const ch = value["chain_hash"];
  if (typeof ch !== "string") {
    return { ok: false, error: 'missing or non-string "chain_hash"' };
  }

  const rawEvents = value["events"];
  if (!Array.isArray(rawEvents)) {
    return { ok: false, error: '"events" must be an array' };
  }

  const events: OperationEvent[] = [];
  for (let i = 0; i < rawEvents.length; i++) {
    const re = rawEvents[i];
    if (!isObject(re)) {
      return { ok: false, error: `events[${i}] must be an object` };
    }
    const id = re["id"];
    const seq = re["seq"];
    const et = re["event_type"];
    const pc = re["payload_commitment"];
    const objs = re["objects"];
    if (typeof id !== "string") {
      return { ok: false, error: `events[${i}].id must be a string` };
    }
    if (typeof seq !== "number" || !Number.isInteger(seq) || seq < 0) {
      return {
        ok: false,
        error: `events[${i}].seq must be a non-negative integer`,
      };
    }
    if (typeof et !== "string") {
      return { ok: false, error: `events[${i}].event_type must be a string` };
    }
    if (typeof pc !== "string") {
      return {
        ok: false,
        error: `events[${i}].payload_commitment must be a string`,
      };
    }
    if (!Array.isArray(objs)) {
      return { ok: false, error: `events[${i}].objects must be an array` };
    }
    const objects: ObjectRef[] = [];
    for (let j = 0; j < objs.length; j++) {
      const o = objs[j];
      if (!isObject(o)) {
        return { ok: false, error: `events[${i}].objects[${j}] must be an object` };
      }
      const oid = o["id"];
      const ot = o["obj_type"];
      const q = o["qualifier"];
      if (typeof oid !== "string") {
        return { ok: false, error: `events[${i}].objects[${j}].id must be a string` };
      }
      if (typeof ot !== "string") {
        return {
          ok: false,
          error: `events[${i}].objects[${j}].obj_type must be a string`,
        };
      }
      if (q !== null && typeof q !== "string") {
        return {
          ok: false,
          error: `events[${i}].objects[${j}].qualifier must be a string or null`,
        };
      }
      objects.push({
        id: oid,
        obj_type: ot,
        qualifier: q === null ? null : (q as string),
      });
    }
    events.push({
      id,
      seq,
      event_type: et,
      objects,
      payload_commitment: pc,
    });
  }

  const receipt: Receipt = { format_version: fv, events, chain_hash: ch };
  return { ok: true, receipt, value };
}

// ── The 7-stage pipeline (faithful port of src/verifier.rs::verify) ───────────

function isWellFormedHash(hex: string): boolean {
  return hex.length === BLAKE3_HEX_LEN && WELL_FORMED_HASH.test(hex);
}

function stageDecode(r: Receipt): CheckOutcome {
  const passed = r.format_version.trim().length > 0;
  return {
    stage: "decode",
    passed,
    detail: passed
      ? `${r.events.length} event(s), format_version present`
      : "format_version is empty or unparseable",
  };
}

function stageCheckFormat(r: Receipt): CheckOutcome {
  const passed = r.format_version === STANDARD_VERSION;
  return {
    stage: "check_format",
    passed,
    detail: passed
      ? `format_version == ${STANDARD_VERSION}`
      : `expected format_version ${STANDARD_VERSION}, found ${r.format_version}`,
  };
}

function stageChainIntegrity(r: Receipt): CheckOutcome {
  const computed = recomputeChain(r.events);
  const passed = computed === r.chain_hash;
  return {
    stage: "chain_integrity",
    passed,
    detail: passed
      ? "recomputed chain hash matches stored chain_hash"
      : `chain hash mismatch: stored ${r.chain_hash}, recomputed ${computed}`,
  };
}

function stageContinuity(r: Receipt): CheckOutcome {
  const seenIds = new Set<string>();
  for (let index = 0; index < r.events.length; index++) {
    const ev = r.events[index];
    const expectedSeq = index;
    if (ev.seq !== expectedSeq) {
      return {
        stage: "continuity",
        passed: false,
        detail: `seq gap at position ${index}: expected ${expectedSeq}, found ${ev.seq}`,
      };
    }
    if (seenIds.has(ev.id)) {
      return {
        stage: "continuity",
        passed: false,
        detail: `duplicate event id: ${ev.id}`,
      };
    }
    seenIds.add(ev.id);
  }
  return {
    stage: "continuity",
    passed: true,
    detail: `${r.events.length} event(s) with contiguous seq and unique ids`,
  };
}

function stageVerifyCommitments(r: Receipt): CheckOutcome {
  for (const ev of r.events) {
    if (!isWellFormedHash(ev.payload_commitment)) {
      return {
        stage: "verify_commitments",
        passed: false,
        detail: `event ${ev.id} has a malformed commitment (expected ${BLAKE3_HEX_LEN} lowercase hex chars)`,
      };
    }
  }
  return {
    stage: "verify_commitments",
    passed: true,
    detail: "all commitments are well-formed BLAKE3 digests",
  };
}

function stageEvaluateProfile(r: Receipt): CheckOutcome {
  for (const ev of r.events) {
    if (ev.event_type.trim().length === 0) {
      return {
        stage: "evaluate_profile",
        passed: false,
        detail: `event ${ev.id} has an empty event_type`,
      };
    }
    if (ev.payload_commitment.length === 0) {
      return {
        stage: "evaluate_profile",
        passed: false,
        detail: `event ${ev.id} is missing a commitment`,
      };
    }
  }
  return {
    stage: "evaluate_profile",
    passed: true,
    detail: "profile core/v1 satisfied",
  };
}

/**
 * Run the seven-stage decidable certify pipeline and emit the Verdict.
 *
 * accepted iff every stage passed; reason summarizes the FIRST failing stage as
 * "<stage>: <detail>", else "all stages passed". This is byte-for-byte the same
 * decision the `affi` binary's `receipt verify` produces over the same receipt.
 */
export function verifyReceipt(receipt: Receipt): Verdict {
  const outcomes: CheckOutcome[] = [
    stageDecode(receipt),
    stageCheckFormat(receipt),
    stageChainIntegrity(receipt),
    stageContinuity(receipt),
    stageVerifyCommitments(receipt),
    stageEvaluateProfile(receipt),
  ];

  const firstFailure = outcomes.find((o) => !o.passed);
  const accepted = firstFailure === undefined;
  const reason = firstFailure
    ? `${firstFailure.stage}: ${firstFailure.detail}`
    : "all stages passed";

  return { accepted, profile: "core/v1", outcomes, reason };
}

/**
 * Build a valid example receipt (a small genuine chain) so the studio can offer
 * an editable, clearly-labeled starting point that actually verifies. This is
 * NOT captured data — it is computed here with the real chain rule so a user can
 * edit it and watch stages pass or fail. The commitments are real BLAKE3 digests
 * of the labeled example payloads.
 */
export function buildExampleReceipt(): Receipt {
  const events: OperationEvent[] = [
    {
      id: "evt-0",
      seq: 0,
      event_type: "create",
      objects: [{ id: "a1", obj_type: "artifact", qualifier: "seed" }],
      payload_commitment: blake3Hex("example-payload-create"),
    },
    {
      id: "evt-1",
      seq: 1,
      event_type: "transform",
      objects: [{ id: "a1", obj_type: "artifact", qualifier: "work" }],
      payload_commitment: blake3Hex("example-payload-transform"),
    },
    {
      id: "evt-2",
      seq: 2,
      event_type: "release",
      objects: [{ id: "a1", obj_type: "artifact", qualifier: "ship" }],
      payload_commitment: blake3Hex("example-payload-release"),
    },
  ];
  return {
    format_version: STANDARD_VERSION,
    events,
    chain_hash: recomputeChain(events),
  };
}
