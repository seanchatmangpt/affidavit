// Chain Visualizer — client-side model & helpers.
//
// DOCTRINE NOTE: This module computes an *illustrative digest* over the
// example receipt purely so the chain-hash chips change visibly when a field
// is edited/tampered. It is a tiny non-cryptographic hash (FNV-1a-ish) and is
// labeled "illustrative digest" everywhere it is shown. It is NOT BLAKE3 and
// must never be presented as a real captured payload_commitment or chain_hash.
//
// The real affidavit chain uses a rolling BLAKE3:
//   h0   = blake3("affidavit-v26.6.14-genesis")
//   h_n  = blake3( hex(h_{n-1}) ++ canonicalBytes(event_n) )
// We mirror that *structure* (fold prev-hash into the next link) so the
// tamper-propagation property is faithfully demonstrated, without claiming to
// compute the real hash.

export const GENESIS_LABEL = "affidavit-v26.6.14-genesis";

export interface ObjectRef {
  id: string;
  obj_type: string;
  qualifier?: string;
}

export interface OperationEvent {
  id: string;
  seq: number;
  event_type: string;
  objects: ObjectRef[];
  payload_commitment: string; // 64-hex in real receipts; example values are clearly labeled
}

export interface Receipt {
  receipt_id?: string;
  events: OperationEvent[];
}

// ---- illustrative (non-crypto) digest -------------------------------------

// FNV-1a 32-bit, rendered as 8 hex chars. Deterministic, dependency-free.
function fnv1a(input: string): string {
  let h = 0x811c9dc5;
  for (let i = 0; i < input.length; i++) {
    h ^= input.charCodeAt(i);
    // h *= 16777619 with 32-bit overflow
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h.toString(16).padStart(8, "0");
}

// Canonical-ish serialization of one event (stable key order). This stands in
// for affidavit's canonicalBytes(event) — same idea (deterministic encoding),
// not the same wire format.
export function canonicalEvent(e: OperationEvent): string {
  const objs = e.objects.map((o) => ({
    id: o.id,
    obj_type: o.obj_type,
    qualifier: o.qualifier ?? null,
  }));
  return JSON.stringify({
    seq: e.seq,
    event_type: e.event_type,
    id: e.id,
    objects: objs,
    payload_commitment: e.payload_commitment,
  });
}

export interface ChainLink {
  // null event => genesis link
  event: OperationEvent | null;
  seq: number; // -1 for genesis
  prevDigest: string; // illustrative digest of previous link
  digest: string; // illustrative digest of this link
}

// Build the rolling illustrative-digest chain. Each link folds the previous
// link's digest with the current event's canonical bytes — so editing any
// event re-routes that link and every link after it.
export function buildChain(events: OperationEvent[]): ChainLink[] {
  const links: ChainLink[] = [];
  const h0 = fnv1a(GENESIS_LABEL);
  links.push({ event: null, seq: -1, prevDigest: "", digest: h0 });

  let prev = h0;
  const sorted = [...events].sort((a, b) => a.seq - b.seq);
  for (const e of sorted) {
    const digest = fnv1a(prev + canonicalEvent(e));
    links.push({ event: e, seq: e.seq, prevDigest: prev, digest });
    prev = digest;
  }
  return links;
}

// Given a baseline chain and a tampered chain, return the index of the first
// link whose digest diverged. Links at/after this index would "re-route".
export function firstDivergence(base: ChainLink[], tampered: ChainLink[]): number {
  const n = Math.min(base.length, tampered.length);
  for (let i = 0; i < n; i++) {
    if (base[i].digest !== tampered[i].digest) return i;
  }
  return base.length !== tampered.length ? n : -1;
}

// ---- OCEL object graph -----------------------------------------------------

export interface ObjectNode {
  id: string;
  obj_type: string;
}

export interface OcelEdge {
  eventId: string;
  objectId: string;
  qualifier?: string;
}

export interface OcelGraph {
  objects: ObjectNode[];
  edges: OcelEdge[];
}

export function buildOcel(events: OperationEvent[]): OcelGraph {
  const objMap = new Map<string, ObjectNode>();
  const edges: OcelEdge[] = [];
  for (const e of events) {
    for (const o of e.objects) {
      if (!objMap.has(o.id)) objMap.set(o.id, { id: o.id, obj_type: o.obj_type });
      edges.push({ eventId: e.id, objectId: o.id, qualifier: o.qualifier });
    }
  }
  return { objects: [...objMap.values()], edges };
}

// ---- parsing ---------------------------------------------------------------

export interface ParseResult {
  receipt: Receipt | null;
  error: string | null;
}

// Lenient parse + shape validation of the user-editable example. We only read
// the fields we visualize; unknown fields are ignored.
export function parseReceipt(text: string): ParseResult {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch (err) {
    return { receipt: null, error: "JSON parse error: " + (err as Error).message };
  }
  if (typeof raw !== "object" || raw === null) {
    return { receipt: null, error: "Top-level value must be an object." };
  }
  const obj = raw as Record<string, unknown>;
  const eventsRaw = obj.events;
  if (!Array.isArray(eventsRaw)) {
    return { receipt: null, error: "Missing `events` array." };
  }
  const events: OperationEvent[] = [];
  for (let i = 0; i < eventsRaw.length; i++) {
    const ev = eventsRaw[i];
    if (typeof ev !== "object" || ev === null) {
      return { receipt: null, error: `events[${i}] is not an object.` };
    }
    const e = ev as Record<string, unknown>;
    if (typeof e.event_type !== "string") {
      return { receipt: null, error: `events[${i}].event_type must be a string.` };
    }
    const id = typeof e.id === "string" ? e.id : `evt-${i}`;
    const seq = typeof e.seq === "number" ? e.seq : i;
    const pc =
      typeof e.payload_commitment === "string"
        ? e.payload_commitment
        : "";
    const objects: ObjectRef[] = [];
    if (Array.isArray(e.objects)) {
      for (let j = 0; j < e.objects.length; j++) {
        const oRaw = e.objects[j];
        if (typeof oRaw !== "object" || oRaw === null) {
          return { receipt: null, error: `events[${i}].objects[${j}] is not an object.` };
        }
        const o = oRaw as Record<string, unknown>;
        if (typeof o.id !== "string") {
          return { receipt: null, error: `events[${i}].objects[${j}].id must be a string.` };
        }
        objects.push({
          id: o.id,
          obj_type: typeof o.obj_type === "string" ? o.obj_type : "object",
          qualifier: typeof o.qualifier === "string" ? o.qualifier : undefined,
        });
      }
    }
    events.push({ id, seq, event_type: e.event_type, objects, payload_commitment: pc });
  }
  if (events.length === 0) {
    return { receipt: null, error: "`events` array is empty — add at least one event." };
  }
  return {
    receipt: {
      receipt_id: typeof obj.receipt_id === "string" ? obj.receipt_id : undefined,
      events,
    },
    error: null,
  };
}

// Default clearly-labeled EXAMPLE receipt. The payload_commitment values are
// obviously synthetic (repeated nibble) so nobody mistakes them for real
// captured BLAKE3 commitments.
export const EXAMPLE_RECEIPT_JSON = `{
  "receipt_id": "example-receipt-0001",
  "events": [
    {
      "id": "evt-a1",
      "seq": 0,
      "event_type": "TemplateRegistered",
      "objects": [
        { "id": "tmpl:invoice", "obj_type": "template", "qualifier": "registered" }
      ],
      "payload_commitment": "1111111111111111111111111111111111111111111111111111111111111111"
    },
    {
      "id": "evt-b2",
      "seq": 1,
      "event_type": "ContextBound",
      "objects": [
        { "id": "tmpl:invoice", "obj_type": "template", "qualifier": "uses" },
        { "id": "ctx:run-42", "obj_type": "context", "qualifier": "input" }
      ],
      "payload_commitment": "2222222222222222222222222222222222222222222222222222222222222222"
    },
    {
      "id": "evt-c3",
      "seq": 2,
      "event_type": "ArtifactEmitted",
      "objects": [
        { "id": "ctx:run-42", "obj_type": "context", "qualifier": "input" },
        { "id": "art:invoice.pdf", "obj_type": "artifact", "qualifier": "produced" }
      ],
      "payload_commitment": "3333333333333333333333333333333333333333333333333333333333333333"
    },
    {
      "id": "evt-d4",
      "seq": 3,
      "event_type": "ReceiptSealed",
      "objects": [
        { "id": "art:invoice.pdf", "obj_type": "artifact", "qualifier": "seals" }
      ],
      "payload_commitment": "4444444444444444444444444444444444444444444444444444444444444444"
    }
  ]
}`;
