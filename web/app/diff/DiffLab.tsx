"use client";

import { useMemo, useState } from "react";
import styles from "./diff.module.css";

/* ------------------------------------------------------------------ *
 * Locally-redeclared receipt types (no imports from web/lib).
 * Mirrors the affidavit core/v1 receipt shape used for parsing/diffing.
 * ------------------------------------------------------------------ */

interface RObject {
  id?: unknown;
  obj_type?: unknown;
  qualifier?: unknown;
}
interface REvent {
  id?: unknown;
  seq?: unknown;
  event_type?: unknown;
  objects?: unknown;
  payload_commitment?: unknown;
}
interface Receipt {
  format_version?: unknown;
  chain_hash?: unknown;
  events?: unknown;
}

const HEX64 = /^[0-9a-f]{64}$/;

/* ------------------------------------------------------------------ *
 * Example inputs (clearly labelled as EXAMPLE INPUT in the UI).
 * Right side is the SAME chain as left, but tampered in two
 * instructive ways:
 *   - event seq=1 has its event_type changed "transform" -> "tamper"
 *   - a seq GAP is introduced (last event seq 2 -> 3) breaking continuity
 * This lets a developer see exactly which structural stage each trip hits.
 * ------------------------------------------------------------------ */

const C0 = "a".repeat(64);
const C1 = "b".repeat(64);
const C2 = "c".repeat(64);

const EXAMPLE_LEFT = JSON.stringify(
  {
    format_version: "core/v1",
    chain_hash: "f".repeat(64),
    events: [
      {
        id: "evt-0",
        seq: 0,
        event_type: "create",
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "seed" }],
        payload_commitment: C0,
      },
      {
        id: "evt-1",
        seq: 1,
        event_type: "transform",
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "work" }],
        payload_commitment: C1,
      },
      {
        id: "evt-2",
        seq: 2,
        event_type: "release",
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "ship" }],
        payload_commitment: C2,
      },
    ],
  },
  null,
  2,
);

const EXAMPLE_RIGHT = JSON.stringify(
  {
    format_version: "core/v1",
    chain_hash: "0".repeat(64), // differs from baseline
    events: [
      {
        id: "evt-0",
        seq: 0,
        event_type: "create",
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "seed" }],
        payload_commitment: C0,
      },
      {
        id: "evt-1",
        seq: 1,
        event_type: "tamper", // CHANGED event_type
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "work" }],
        payload_commitment: C1,
      },
      {
        id: "evt-2",
        seq: 3, // SEQ GAP: should be 2 -> continuity break
        event_type: "release",
        objects: [{ id: "a1", obj_type: "artifact", qualifier: "ship" }],
        payload_commitment: C2,
      },
    ],
  },
  null,
  2,
);

/* ------------------------------------------------------------------ *
 * Parsing
 * ------------------------------------------------------------------ */

type ParseState =
  | { ok: true; receipt: Receipt; raw: string }
  | { ok: false; error: string; raw: string };

function parseReceipt(raw: string): ParseState {
  const trimmed = raw.trim();
  if (!trimmed) return { ok: false, error: "empty input", raw };
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch (e) {
    return { ok: false, error: `invalid JSON: ${(e as Error).message}`, raw };
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    return { ok: false, error: "top-level value must be a JSON object", raw };
  }
  return { ok: true, receipt: parsed as Receipt, raw };
}

/* ------------------------------------------------------------------ *
 * Structural verdict (real, decidable checks — no BLAKE3).
 * ------------------------------------------------------------------ */

interface Check {
  stage: string;
  label: string;
  passed: boolean;
  detail: string;
}
interface Verdict {
  checks: Check[];
  firstFailure: Check | null;
}

function asString(v: unknown): string | null {
  return typeof v === "string" ? v : null;
}

function evaluateStructure(receipt: Receipt): Verdict {
  const checks: Check[] = [];

  // 1. check_format -----------------------------------------------------
  const fv = asString(receipt.format_version);
  checks.push({
    stage: "check_format",
    label: "check_format",
    passed: fv === "core/v1",
    detail:
      fv === "core/v1"
        ? "format_version is \"core/v1\""
        : `format_version is ${fv === null ? "missing / non-string" : `"${fv}"`}, expected "core/v1"`,
  });

  // Normalise events into an array for the remaining checks.
  const rawEvents = receipt.events;
  const events: REvent[] = Array.isArray(rawEvents)
    ? (rawEvents as REvent[])
    : [];
  const eventsOk = Array.isArray(rawEvents);

  // 2. continuity: seq[i] === i for all i; ids unique -------------------
  let continuityPassed = eventsOk;
  let continuityDetail = "events[i].seq === i for all i; ids unique";
  if (!eventsOk) {
    continuityPassed = false;
    continuityDetail = "\"events\" is missing or not an array";
  } else {
    const seenIds = new Set<string>();
    for (let i = 0; i < events.length; i++) {
      const ev = events[i] ?? {};
      const seq = ev.seq;
      if (typeof seq !== "number" || seq !== i) {
        continuityPassed = false;
        continuityDetail = `events[${i}].seq is ${
          typeof seq === "number" ? seq : JSON.stringify(seq)
        }, expected ${i}`;
        break;
      }
      const id = ev.id;
      const idStr = typeof id === "string" ? id : JSON.stringify(id);
      if (seenIds.has(idStr)) {
        continuityPassed = false;
        continuityDetail = `duplicate event id ${idStr} at events[${i}]`;
        break;
      }
      seenIds.add(idStr);
    }
  }
  checks.push({
    stage: "continuity",
    label: "continuity",
    passed: continuityPassed,
    detail: continuityDetail,
  });

  // 3. verify_commitments: every payload_commitment is 64-hex ----------
  let commitPassed = eventsOk;
  let commitDetail = "every payload_commitment matches /^[0-9a-f]{64}$/";
  if (!eventsOk) {
    commitPassed = false;
    commitDetail = "\"events\" is missing or not an array";
  } else {
    for (let i = 0; i < events.length; i++) {
      const pc = events[i]?.payload_commitment;
      if (typeof pc !== "string" || !HEX64.test(pc)) {
        commitPassed = false;
        commitDetail = `events[${i}].payload_commitment ${
          typeof pc === "string" ? `"${pc}"` : "is missing / non-string"
        } is not 64 lowercase hex chars`;
        break;
      }
    }
  }
  checks.push({
    stage: "verify_commitments",
    label: "verify_commitments",
    passed: commitPassed,
    detail: commitDetail,
  });

  // 4. evaluate_profile: event_type non-empty (trimmed) & commitment non-empty
  let profilePassed = eventsOk;
  let profileDetail =
    "every event_type non-empty (trimmed) and payload_commitment non-empty";
  if (!eventsOk) {
    profilePassed = false;
    profileDetail = "\"events\" is missing or not an array";
  } else {
    for (let i = 0; i < events.length; i++) {
      const ev = events[i] ?? {};
      const et = ev.event_type;
      if (typeof et !== "string" || et.trim() === "") {
        profilePassed = false;
        profileDetail = `events[${i}].event_type is empty / non-string`;
        break;
      }
      const pc = ev.payload_commitment;
      if (typeof pc !== "string" || pc.trim() === "") {
        profilePassed = false;
        profileDetail = `events[${i}].payload_commitment is empty / non-string`;
        break;
      }
    }
  }
  checks.push({
    stage: "evaluate_profile",
    label: "evaluate_profile",
    passed: profilePassed,
    detail: profileDetail,
  });

  const firstFailure = checks.find((c) => !c.passed) ?? null;
  return { checks, firstFailure };
}

/* ------------------------------------------------------------------ *
 * Diffing
 * ------------------------------------------------------------------ */

type FieldStatus = "same" | "changed";
interface TopFieldDiff {
  name: string;
  left: string;
  right: string;
  status: FieldStatus;
}

function show(v: unknown): string {
  if (v === undefined) return "—";
  if (typeof v === "string") return v;
  return JSON.stringify(v);
}

function topLevelDiff(left: Receipt, right: Receipt): TopFieldDiff[] {
  const names: (keyof Receipt)[] = ["format_version", "chain_hash"];
  return names.map((name) => {
    const l = show(left[name]);
    const r = show(right[name]);
    return { name, left: l, right: r, status: l === r ? "same" : "changed" };
  });
}

type RowStatus = "same" | "added" | "removed" | "changed";
interface FieldChange {
  field: string;
  left: string;
  right: string;
}
interface EventDiffRow {
  key: string;
  leftSeq: string;
  rightSeq: string;
  status: RowStatus;
  changes: FieldChange[]; // only populated when status === "changed"
}

const EVENT_FIELDS = ["seq", "event_type", "objects", "payload_commitment"] as const;

function keyOf(ev: REvent, fallbackIdx: number): string {
  // Prefer id; fall back to seq; finally positional index.
  if (typeof ev.id === "string" && ev.id) return `id:${ev.id}`;
  if (typeof ev.seq === "number") return `seq:${ev.seq}`;
  return `idx:${fallbackIdx}`;
}

function eventField(ev: REvent, field: string): unknown {
  return (ev as Record<string, unknown>)[field];
}

function eventDiff(left: Receipt, right: Receipt): EventDiffRow[] {
  const le = Array.isArray(left.events) ? (left.events as REvent[]) : [];
  const re = Array.isArray(right.events) ? (right.events as REvent[]) : [];

  // Build ordered, de-duplicated key list preserving first appearance.
  const leftMap = new Map<string, REvent>();
  const rightMap = new Map<string, REvent>();
  const order: string[] = [];
  const seen = new Set<string>();
  le.forEach((ev, i) => {
    const k = keyOf(ev, i);
    if (!leftMap.has(k)) leftMap.set(k, ev);
    if (!seen.has(k)) { seen.add(k); order.push(k); }
  });
  re.forEach((ev, i) => {
    const k = keyOf(ev, i);
    if (!rightMap.has(k)) rightMap.set(k, ev);
    if (!seen.has(k)) { seen.add(k); order.push(k); }
  });

  return order.map((k) => {
    const l = leftMap.get(k);
    const r = rightMap.get(k);
    if (l && !r) {
      return {
        key: k,
        leftSeq: show(l.seq),
        rightSeq: "—",
        status: "removed",
        changes: [],
      };
    }
    if (!l && r) {
      return {
        key: k,
        leftSeq: "—",
        rightSeq: show(r.seq),
        status: "added",
        changes: [],
      };
    }
    // present on both — compare fields
    const changes: FieldChange[] = [];
    for (const f of EVENT_FIELDS) {
      const lv = JSON.stringify(eventField(l!, f) ?? null);
      const rv = JSON.stringify(eventField(r!, f) ?? null);
      if (lv !== rv) {
        changes.push({ field: f, left: show(eventField(l!, f)), right: show(eventField(r!, f)) });
      }
    }
    return {
      key: k,
      leftSeq: show(l!.seq),
      rightSeq: show(r!.seq),
      status: changes.length ? "changed" : "same",
      changes,
    };
  });
}

/* ------------------------------------------------------------------ *
 * Sub-components
 * ------------------------------------------------------------------ */

function VerdictPanel({
  side,
  parse,
}: {
  side: "left" | "right";
  parse: ParseState;
}) {
  const label = side === "left" ? "LEFT · baseline" : "RIGHT · candidate";
  if (!parse.ok) {
    return (
      <div className="card">
        <div className={`${styles.sideTag} ${styles[side]}`}>{label}</div>
        <p className="warn" style={{ fontSize: 13 }}>
          cannot evaluate — {parse.error}
        </p>
      </div>
    );
  }
  const verdict = evaluateStructure(parse.receipt);
  return (
    <div className="card">
      <div className={`${styles.sideTag} ${styles[side]}`}>{label}</div>
      {verdict.checks.map((c) => (
        <div key={c.stage} className={styles.checkLine}>
          <span className={`${styles.checkMark} ${c.passed ? "green" : "warn"}`}>
            {c.passed ? "✓" : "✗"}
          </span>
          <span>
            <span className={styles.checkName}>{c.label}</span>{" "}
            <span className={styles.checkDetail}>— {c.detail}</span>
          </span>
        </div>
      ))}

      {/* chain_integrity is intentionally NOT checked here. */}
      <div className={styles.checkLine}>
        <span className={`${styles.checkMark} muted`}>•</span>
        <span>
          <span className={styles.checkName}>chain_integrity</span>{" "}
          <span className={styles.checkDetail}>
            — not checked here — structural only; use Studio for the full BLAKE3
            check
          </span>
        </span>
      </div>

      {verdict.firstFailure ? (
        <div className={`${styles.verdictBanner} ${styles.bannerReject}`}>
          would REJECT at: <strong>{verdict.firstFailure.stage}</strong> —{" "}
          {verdict.firstFailure.detail}
        </div>
      ) : (
        <div className={`${styles.verdictBanner} ${styles.bannerOk}`}>
          structurally sound (chain_integrity not verified here)
        </div>
      )}
    </div>
  );
}

/* ------------------------------------------------------------------ *
 * Main component
 * ------------------------------------------------------------------ */

export default function DiffLab() {
  const [leftText, setLeftText] = useState(EXAMPLE_LEFT);
  const [rightText, setRightText] = useState(EXAMPLE_RIGHT);

  // Snapshot taken on "diff" so the view does not churn while typing.
  const [snapshot, setSnapshot] = useState<{ left: string; right: string }>({
    left: EXAMPLE_LEFT,
    right: EXAMPLE_RIGHT,
  });

  const leftParse = useMemo(() => parseReceipt(snapshot.left), [snapshot.left]);
  const rightParse = useMemo(
    () => parseReceipt(snapshot.right),
    [snapshot.right],
  );

  const bothOk = leftParse.ok && rightParse.ok;

  const topDiff = useMemo(
    () =>
      bothOk
        ? topLevelDiff(
            (leftParse as { receipt: Receipt }).receipt,
            (rightParse as { receipt: Receipt }).receipt,
          )
        : [],
    [bothOk, leftParse, rightParse],
  );
  const evtDiff = useMemo(
    () =>
      bothOk
        ? eventDiff(
            (leftParse as { receipt: Receipt }).receipt,
            (rightParse as { receipt: Receipt }).receipt,
          )
        : [],
    [bothOk, leftParse, rightParse],
  );

  function runDiff() {
    setSnapshot({ left: leftText, right: rightText });
  }
  function loadExamples() {
    setLeftText(EXAMPLE_LEFT);
    setRightText(EXAMPLE_RIGHT);
    setSnapshot({ left: EXAMPLE_LEFT, right: EXAMPLE_RIGHT });
  }

  // Live parse status for the editors (so a broken edit shows immediately).
  const leftLive = useMemo(() => parseReceipt(leftText), [leftText]);
  const rightLive = useMemo(() => parseReceipt(rightText), [rightText]);

  const changedTop = topDiff.filter((d) => d.status === "changed").length;
  const changedEvents = evtDiff.filter((d) => d.status !== "same").length;

  return (
    <>
      <div className={styles.editorGrid}>
        <div className={styles.editorCol}>
          <span className={`${styles.sideTag} ${styles.left}`}>
            LEFT · baseline (example input)
          </span>
          <textarea
            className={`${styles.textarea} ${leftLive.ok ? "" : styles.bad}`}
            value={leftText}
            spellCheck={false}
            onChange={(e) => setLeftText(e.target.value)}
            aria-label="left receipt JSON (baseline)"
          />
          {!leftLive.ok && (
            <div className={styles.parseErr}>left: {leftLive.error}</div>
          )}
        </div>
        <div className={styles.editorCol}>
          <span className={`${styles.sideTag} ${styles.right}`}>
            RIGHT · candidate (example input)
          </span>
          <textarea
            className={`${styles.textarea} ${rightLive.ok ? "" : styles.bad}`}
            value={rightText}
            spellCheck={false}
            onChange={(e) => setRightText(e.target.value)}
            aria-label="right receipt JSON (candidate)"
          />
          {!rightLive.ok && (
            <div className={styles.parseErr}>right: {rightLive.error}</div>
          )}
        </div>
      </div>

      <div className={styles.controls}>
        <button className={styles.btn} onClick={runDiff} type="button">
          diff
        </button>
        <button className={styles.btnGhost} onClick={loadExamples} type="button">
          reset to example inputs
        </button>
        <span className="muted" style={{ fontSize: 12 }}>
          example inputs differ: chain_hash, one event_type (transform→tamper),
          and a seq gap.
        </span>
      </div>

      {!bothOk && (
        <p className="warn" style={{ marginTop: "1rem" }}>
          Fix the parse error(s) above, then press <code>diff</code>. (
          {!leftParse.ok ? "left invalid" : ""}
          {!leftParse.ok && !rightParse.ok ? "; " : ""}
          {!rightParse.ok ? "right invalid" : ""})
        </p>
      )}

      {/* ---------------- STRUCTURAL VERDICTS ---------------- */}
      <section className={styles.section}>
        <h2>Structural verdict — which stage would REJECT</h2>
        <p className="muted" style={{ fontSize: 13, marginTop: 0 }}>
          Real, decidable structural checks computed from the JSON you provided.
          These mirror the order the certify pipeline applies them, so the FIRST
          failing stage is the one a tamper would trip. BLAKE3 chain integrity is
          deliberately not computed here.
        </p>
        <div className={styles.verdictGrid}>
          <VerdictPanel side="left" parse={leftParse} />
          <VerdictPanel side="right" parse={rightParse} />
        </div>
      </section>

      {/* ---------------- DIFF VIEW ---------------- */}
      {bothOk && (
        <>
          <section className={styles.section}>
            <h2>
              Top-level fields{" "}
              <span className="muted" style={{ fontSize: 12 }}>
                ({changedTop} changed)
              </span>
            </h2>
            <div className="card" style={{ padding: 0 }}>
              <table>
                <thead>
                  <tr>
                    <th style={{ width: "18%" }}>field</th>
                    <th style={{ width: "41%" }}>left (baseline)</th>
                    <th style={{ width: "41%" }}>right (candidate)</th>
                  </tr>
                </thead>
                <tbody>
                  {topDiff.map((d) => (
                    <tr
                      key={d.name}
                      className={`${styles.fieldRow} ${
                        d.status === "changed" ? styles.diffHl : ""
                      }`}
                    >
                      <td className={styles.fieldName}>{d.name}</td>
                      <td className={d.status === "changed" ? "warn" : ""}>
                        {d.left}
                      </td>
                      <td className={d.status === "changed" ? "warn" : ""}>
                        {d.right}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className={styles.section}>
            <h2>
              Events{" "}
              <span className="muted" style={{ fontSize: 12 }}>
                ({changedEvents} added/removed/changed of {evtDiff.length})
              </span>
            </h2>
            <div className={styles.legend}>
              <span>
                <span
                  className={styles.legendDot}
                  style={{ background: "var(--green)" }}
                />
                added
              </span>
              <span>
                <span
                  className={styles.legendDot}
                  style={{ background: "var(--warn)" }}
                />
                removed
              </span>
              <span>
                <span
                  className={styles.legendDot}
                  style={{ background: "var(--accent)" }}
                />
                changed
              </span>
            </div>
            <div className="card" style={{ padding: 0, marginTop: "0.5rem" }}>
              <table className={styles.evtTable}>
                <thead>
                  <tr>
                    <th style={{ width: "16%" }}>key</th>
                    <th style={{ width: "8%" }}>L.seq</th>
                    <th style={{ width: "8%" }}>R.seq</th>
                    <th style={{ width: "14%" }}>status</th>
                    <th>field changes</th>
                  </tr>
                </thead>
                <tbody>
                  {evtDiff.map((row) => (
                    <tr
                      key={row.key}
                      className={row.status !== "same" ? styles.diffHl : ""}
                    >
                      <td className={styles.seqCell}>{row.key}</td>
                      <td className={styles.seqCell}>{row.leftSeq}</td>
                      <td className={styles.seqCell}>{row.rightSeq}</td>
                      <td>
                        <span
                          className={`${styles.statusPill} ${
                            row.status === "added"
                              ? styles.pillAdded
                              : row.status === "removed"
                                ? styles.pillRemoved
                                : row.status === "changed"
                                  ? styles.pillChanged
                                  : styles.pillSame
                          }`}
                        >
                          {row.status}
                        </span>
                      </td>
                      <td>
                        {row.status === "same" && (
                          <span className="muted">identical</span>
                        )}
                        {row.status === "added" && (
                          <span className={styles.added}>
                            present only on right (candidate)
                          </span>
                        )}
                        {row.status === "removed" && (
                          <span className={styles.removed}>
                            present only on left (baseline)
                          </span>
                        )}
                        {row.status === "changed" &&
                          row.changes.map((c) => (
                            <span key={c.field} className={styles.fieldChange}>
                              <span className="muted">{c.field}: </span>
                              <span className="warn">{c.left}</span>
                              <span className={styles.arrow}>→</span>
                              <span className={styles.changed}>{c.right}</span>
                            </span>
                          ))}
                      </td>
                    </tr>
                  ))}
                  {evtDiff.length === 0 && (
                    <tr>
                      <td colSpan={5} className="muted">
                        no events on either side
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </section>
        </>
      )}
    </>
  );
}
