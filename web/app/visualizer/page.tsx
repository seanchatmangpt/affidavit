"use client";

import React, { useMemo, useState } from "react";
import styles from "./visualizer.module.css";
import ChainView from "./ChainView";
import OcelGraphView from "./OcelGraph";
import {
  EXAMPLE_RECEIPT_JSON,
  GENESIS_LABEL,
  buildChain,
  buildOcel,
  firstDivergence,
  parseReceipt,
  type OperationEvent,
} from "./model";

// Apply a tamper to a single event's event_type (append a marker). Returns a
// new events array; the original is untouched. This is a *structural* demo of
// "flip a field" — any byte change has the same propagation effect.
function tamperEvents(
  events: OperationEvent[],
  seq: number | null,
  newType: string,
): OperationEvent[] {
  if (seq === null) return events;
  return events.map((e) =>
    e.seq === seq ? { ...e, event_type: newType } : e,
  );
}

export default function VisualizerPage() {
  const [text, setText] = useState(EXAMPLE_RECEIPT_JSON);
  const [selectedSeq, setSelectedSeq] = useState<number | null>(null);
  const [tamperSeq, setTamperSeq] = useState<number | null>(null);
  const [tamperType, setTamperType] = useState<string>("TamperedEvent");

  const parsed = useMemo(() => parseReceipt(text), [text]);
  const events = parsed.receipt?.events ?? [];

  // Baseline chain (as authored) vs tampered chain (one field flipped).
  const baseChain = useMemo(() => buildChain(events), [events]);
  const tamperedEvents = useMemo(
    () => tamperEvents(events, tamperSeq, tamperType),
    [events, tamperSeq, tamperType],
  );
  const tamperedChain = useMemo(
    () => buildChain(tamperedEvents),
    [tamperedEvents],
  );

  const tampering = tamperSeq !== null;
  const shownChain = tampering ? tamperedChain : baseChain;
  const divergeIndex = tampering
    ? firstDivergence(baseChain, tamperedChain)
    : -1;

  const ocel = useMemo(
    () => buildOcel(tampering ? tamperedEvents : events),
    [events, tamperedEvents, tampering],
  );

  // Detail of the currently selected event (from the *shown* set).
  const shownEvents = tampering ? tamperedEvents : events;
  const selectedEvent =
    selectedSeq === null
      ? null
      : shownEvents.find((e) => e.seq === selectedSeq) ?? null;

  // How many links would re-route, for the banner copy.
  const downstreamCount =
    divergeIndex >= 0 ? shownChain.length - divergeIndex : 0;

  const eventOptions = [...events].sort((a, b) => a.seq - b.seq);

  return (
    <div className={styles.wrap}>
      <h1>Chain Visualizer</h1>
      <p className={styles.intro}>
        An affidavit <strong>receipt</strong> is an append-only chain of{" "}
        <code>OperationEvent</code>s. Two structures make it tamper-evident and
        object-aware: a <em>rolling hash</em> that folds each event into the
        previous link, and an <em>OCEL object graph</em> recording which objects
        every operation touched. This page renders both over an editable{" "}
        <span className={styles.illu}>example receipt</span> so you can poke at
        them. Click any event to highlight it across both views; use the tamper
        control to watch a single edit re-route the chain.
      </p>

      {/* ---- prose: the two properties --------------------------------- */}
      <div className="grid">
        <div className="card">
          <div className="label">rolling-hash propagation</div>
          <p className="src" style={{ marginTop: "0.5rem", lineHeight: 1.6 }}>
            <code>h0 = blake3(&quot;{GENESIS_LABEL}&quot;)</code>; then{" "}
            <code>h_n = blake3(hex(h_(n-1)) ++ canonicalBytes(event_n))</code>.
            Each link folds in the <em>previous</em> hash, so changing any byte
            of any event re-routes that link <strong>and every link after it</strong>.
            The raw payload is never stored — only its{" "}
            <code>payload_commitment</code> (64-hex BLAKE3). That cascade is why
            &quot;the bypass is unconstructable&quot;.
          </p>
        </div>
        <div className="card">
          <div className="label">OCEL object model</div>
          <p className="src" style={{ marginTop: "0.5rem", lineHeight: 1.6 }}>
            Object-Centric Event Log: each event references one or more{" "}
            <code>objects</code> — <code>{`{id, obj_type, qualifier}`}</code> —
            rather than a single case id. The graph below is bipartite: events on
            the left, the distinct objects they touched on the right, edges
            labeled by qualifier (e.g. <code>uses</code>, <code>produced</code>).
            One object (e.g. an artifact) can thread through several operations.
          </p>
        </div>
      </div>

      {/* ---- controls -------------------------------------------------- */}
      <div className={styles.controls}>
        <div className={styles.controlGroup}>
          <span className="label">selected event</span>
          <select
            className={styles.select}
            value={selectedSeq ?? ""}
            onChange={(e) =>
              setSelectedSeq(e.target.value === "" ? null : Number(e.target.value))
            }
            aria-label="Select an event to highlight"
          >
            <option value="">(none)</option>
            {eventOptions.map((e) => (
              <option key={e.id} value={e.seq}>
                seq {e.seq} · {e.event_type}
              </option>
            ))}
          </select>
        </div>

        <div className={styles.controlGroup}>
          <span className="label">tamper event</span>
          <select
            className={styles.select}
            value={tamperSeq ?? ""}
            onChange={(e) =>
              setTamperSeq(e.target.value === "" ? null : Number(e.target.value))
            }
            aria-label="Pick an event to tamper with"
          >
            <option value="">(off)</option>
            {eventOptions.map((e) => (
              <option key={e.id} value={e.seq}>
                seq {e.seq} · {e.event_type}
              </option>
            ))}
          </select>
          <span className="label">→ event_type</span>
          <input
            className={styles.select}
            value={tamperType}
            onChange={(e) => setTamperType(e.target.value)}
            disabled={!tampering}
            aria-label="Replacement event_type for the tampered event"
            style={{ minWidth: "11rem" }}
          />
        </div>

        <button
          className={`${styles.btn} ${tampering ? styles.btnWarn : ""}`}
          onClick={() => setTamperSeq(null)}
          disabled={!tampering}
        >
          reset tamper
        </button>
      </div>

      {/* ---- tamper banner --------------------------------------------- */}
      {tampering && divergeIndex >= 0 ? (
        <div className={styles.banner}>
          <strong className="warn">Chain would re-route.</strong> Editing{" "}
          <code>event_type</code> of <code>seq {tamperSeq}</code> changes its
          canonical bytes, so its link&apos;s hash changes — and because every
          later link folds in the one before it,{" "}
          <strong>{downstreamCount}</strong>{" "}
          {downstreamCount === 1 ? "link" : "links"} (this one + all downstream)
          no longer match the sealed <code>chain_hash</code>. There is no edit
          that repairs only one link without rewriting the whole tail.
        </div>
      ) : tampering && divergeIndex < 0 ? (
        <div className={`${styles.banner} ${styles.bannerOk}`}>
          Tamper selected but the replacement value matches the original — no
          divergence. Change the <code>event_type</code> to see the cascade.
        </div>
      ) : (
        <div className={`${styles.banner} ${styles.bannerOk}`}>
          Chain is consistent (as authored). Pick a{" "}
          <strong>tamper event</strong> above to watch one edit re-route every
          downstream link.
        </div>
      )}

      {/* ---- CHAIN VIEW ------------------------------------------------- */}
      <section>
        <h2>1 · Hash chain</h2>
        <p className={styles.note}>
          GENESIS → each event. The ⊕ glyph is the fold of{" "}
          <code>prev-hash + event</code>. Chips show an{" "}
          <span className={styles.illu}>illustrative digest</span> (a tiny
          non-crypto hash, <strong>not</strong> real BLAKE3) so you can see links
          change as you edit. Re-routed links are shown in{" "}
          <span className="warn">warn color</span>.
        </p>
        {parsed.error ? (
          <div className={styles.banner}>
            <span className="warn">Cannot render chain:</span> {parsed.error}
          </div>
        ) : (
          <ChainView
            links={shownChain}
            selectedSeq={selectedSeq}
            onSelect={setSelectedSeq}
            divergeIndex={divergeIndex}
            tamperedSeq={tamperSeq}
          />
        )}
      </section>

      {/* ---- OCEL GRAPH ------------------------------------------------- */}
      <section>
        <h2>2 · OCEL object graph</h2>
        <p className={styles.note}>
          Bipartite: events ↔ the distinct objects they reference, edges labeled
          by qualifier. Selecting an event highlights its edges and the objects
          it touched.
        </p>
        {parsed.error ? (
          <div className={styles.banner}>
            <span className="warn">Cannot render graph:</span> {parsed.error}
          </div>
        ) : (
          <OcelGraphView
            events={shownEvents}
            graph={ocel}
            selectedSeq={selectedSeq}
            onSelect={setSelectedSeq}
          />
        )}
      </section>

      {/* ---- legend ---------------------------------------------------- */}
      <div className={styles.legend}>
        <span className={styles.legendItem}>
          <span
            className={styles.swatch}
            style={{ borderColor: "var(--accent)", borderWidth: 2 }}
          />
          selected event
        </span>
        <span className={styles.legendItem}>
          <span
            className={styles.swatch}
            style={{ borderColor: "var(--warn)", background: "rgba(210,153,34,0.16)" }}
          />
          re-routed / chain broken
        </span>
        <span className={styles.legendItem}>
          <span className={styles.swatch} style={{ borderColor: "var(--border)" }} />
          consistent link / node
        </span>
        <span className={styles.legendItem}>
          <span className="green">abcd1234</span> illustrative digest (not BLAKE3)
        </span>
        <span className={styles.legendItem}>⊕ fold(prev-hash, event)</span>
      </div>

      {/* ---- selected event detail ------------------------------------- */}
      <section>
        <h2>Selected event</h2>
        {selectedEvent ? (
          <div className={styles.detailGrid}>
            <div className={styles.kv}>
              <span className="label">seq</span>
              <span>{selectedEvent.seq}</span>
              <span className="label">id</span>
              <span className={styles.mono}>{selectedEvent.id}</span>
              <span className="label">event_type</span>
              <span className={tampering && selectedEvent.seq === tamperSeq ? "warn" : ""}>
                {selectedEvent.event_type}
                {tampering && selectedEvent.seq === tamperSeq ? " (tampered)" : ""}
              </span>
              <span className="label">payload_commitment</span>
              <span className={styles.mono}>
                {selectedEvent.payload_commitment || "—"}
                <span className={styles.note}> (example input — not captured)</span>
              </span>
            </div>
            <div>
              <div className="label" style={{ marginBottom: "0.4rem" }}>
                objects ({selectedEvent.objects.length})
              </div>
              <table>
                <thead>
                  <tr>
                    <th>id</th>
                    <th>obj_type</th>
                    <th>qualifier</th>
                  </tr>
                </thead>
                <tbody>
                  {selectedEvent.objects.map((o, i) => (
                    <tr key={i}>
                      <td className={styles.mono}>{o.id}</td>
                      <td>{o.obj_type}</td>
                      <td>{o.qualifier ?? <span className="muted">—</span>}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : (
          <p className={styles.note}>
            No event selected. Click a node in either view, or use the{" "}
            <strong>selected event</strong> dropdown.
          </p>
        )}
      </section>

      {/* ---- editable example ------------------------------------------ */}
      <section>
        <h2>Example receipt (editable)</h2>
        <p className={styles.note}>
          This is a <span className={styles.illu}>clearly-labeled example</span>,
          not a captured receipt. Edit the JSON — both views and the digest chain
          recompute live. Shape: <code>{`{ events: [{ id, seq, event_type, objects: [{ id, obj_type, qualifier? }], payload_commitment }] }`}</code>
          .
        </p>
        <textarea
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            // keep selection/tamper valid-ish; clearing on edit is simplest
            setSelectedSeq(null);
            setTamperSeq(null);
          }}
          spellCheck={false}
          className={styles.editor}
          aria-label="Editable example receipt JSON"
        />
        {parsed.error ? (
          <p className="warn" style={{ marginTop: "0.4rem" }}>
            {parsed.error}
          </p>
        ) : (
          <p className="green" style={{ marginTop: "0.4rem" }}>
            parsed OK · {events.length} event{events.length === 1 ? "" : "s"} ·{" "}
            {ocel.objects.length} distinct object
            {ocel.objects.length === 1 ? "" : "s"}
          </p>
        )}
        <button
          className={styles.btn}
          style={{ marginTop: "0.6rem" }}
          onClick={() => {
            setText(EXAMPLE_RECEIPT_JSON);
            setSelectedSeq(null);
            setTamperSeq(null);
          }}
        >
          restore example
        </button>
      </section>
    </div>
  );
}
