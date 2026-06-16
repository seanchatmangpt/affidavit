"use client";

import React from "react";
import type { OperationEvent, OcelGraph } from "./model";
import styles from "./visualizer.module.css";

interface Props {
  events: OperationEvent[];
  graph: OcelGraph;
  selectedSeq: number | null;
  onSelect: (seq: number | null) => void;
}

const COL_W = 220; // node width
const ROW_H = 52; // vertical pitch per node
const NODE_H = 38;
const PAD_Y = 28;
const PAD_X = 18;
const COL_GAP = 230; // horizontal gap between the two columns

export default function OcelGraphView({ events, graph, selectedSeq, onSelect }: Props) {
  const sortedEvents = [...events].sort((a, b) => a.seq - b.seq);
  const objects = graph.objects;

  const leftX = PAD_X;
  const rightX = PAD_X + COL_W + COL_GAP;
  const width = rightX + COL_W + PAD_X;
  const rows = Math.max(sortedEvents.length, objects.length);
  const height = PAD_Y * 2 + rows * ROW_H + 24;

  const eventY = (i: number) => PAD_Y + 24 + i * ROW_H + NODE_H / 2;
  const objIndex = new Map(objects.map((o, i) => [o.id, i]));
  const objY = (id: string) => {
    const i = objIndex.get(id) ?? 0;
    return PAD_Y + 24 + i * ROW_H + NODE_H / 2;
  };
  const eventSeqById = new Map(sortedEvents.map((e) => [e.id, e.seq]));

  // Which objects are touched by the selected event (for highlight)?
  const selectedObjIds = new Set(
    selectedSeq === null
      ? []
      : sortedEvents
          .filter((e) => e.seq === selectedSeq)
          .flatMap((e) => e.objects.map((o) => o.id)),
  );

  return (
    <div className={styles.svgScroll}>
      <svg
        className={styles.svg}
        width={width}
        height={height}
        viewBox={`0 0 ${width} ${height}`}
        role="group"
        aria-label="OCEL object graph: events and the objects they reference"
      >
        <defs>
          <marker
            id="ocel-arrow"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M0,0 L10,5 L0,10 z" fill="var(--muted)" />
          </marker>
          <marker
            id="ocel-arrow-on"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="6"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M0,0 L10,5 L0,10 z" fill="var(--accent)" />
          </marker>
        </defs>

        {/* column headers */}
        <text x={leftX} y={PAD_Y} fontSize="11" fontWeight={700} fill="var(--muted)">
          EVENTS (operations)
        </text>
        <text x={rightX} y={PAD_Y} fontSize="11" fontWeight={700} fill="var(--muted)">
          OBJECTS (distinct)
        </text>

        {/* edges first so nodes paint over endpoints */}
        {graph.edges.map((e, i) => {
          const seq = eventSeqById.get(e.eventId);
          const y1 = sortedEvents.findIndex((ev) => ev.id === e.eventId);
          if (y1 < 0) return null;
          const sy = eventY(y1);
          const ty = objY(e.objectId);
          const x1 = leftX + COL_W;
          const x2 = rightX;
          const on = selectedSeq !== null && seq === selectedSeq;
          const mx = (x1 + x2) / 2;
          const path = `M ${x1} ${sy} C ${mx} ${sy}, ${mx} ${ty}, ${x2 - 2} ${ty}`;
          return (
            <g key={i} opacity={selectedSeq !== null && !on ? 0.18 : 1}>
              <path
                d={path}
                fill="none"
                stroke={on ? "var(--accent)" : "var(--muted)"}
                strokeWidth={on ? 1.8 : 1}
                markerEnd={on ? "url(#ocel-arrow-on)" : "url(#ocel-arrow)"}
              />
              {e.qualifier && (
                <text
                  x={mx}
                  y={(sy + ty) / 2 - 3}
                  textAnchor="middle"
                  fontSize="10"
                  fill={on ? "var(--accent)" : "var(--muted)"}
                >
                  {e.qualifier}
                </text>
              )}
            </g>
          );
        })}

        {/* event nodes */}
        {sortedEvents.map((ev, i) => {
          const y = PAD_Y + 24 + i * ROW_H;
          const selected = ev.seq === selectedSeq;
          return (
            <g
              key={ev.id}
              onClick={() => onSelect(selected ? null : ev.seq)}
              style={{ cursor: "pointer" }}
              role="button"
              aria-pressed={selected}
            >
              <rect
                x={leftX}
                y={y}
                width={COL_W}
                height={NODE_H}
                rx={7}
                fill="var(--panel)"
                stroke={selected ? "var(--accent)" : "var(--border)"}
                strokeWidth={selected ? 2.2 : 1.2}
              />
              <text x={leftX + 9} y={y + 16} fontSize="11" fill="var(--muted)">
                seq {ev.seq}
              </text>
              <text
                x={leftX + 9}
                y={y + 30}
                fontSize="12"
                fontWeight={600}
                fill={selected ? "var(--accent)" : "var(--fg)"}
              >
                {truncate(ev.event_type, 24)}
              </text>
            </g>
          );
        })}

        {/* object nodes */}
        {objects.map((o, i) => {
          const y = PAD_Y + 24 + i * ROW_H;
          const hot = selectedObjIds.has(o.id);
          return (
            <g key={o.id}>
              <rect
                x={rightX}
                y={y}
                width={COL_W}
                height={NODE_H}
                rx={7}
                fill="var(--panel)"
                stroke={hot ? "var(--accent)" : "var(--border)"}
                strokeWidth={hot ? 2 : 1.2}
                opacity={selectedSeq !== null && !hot ? 0.45 : 1}
              />
              <text
                x={rightX + 9}
                y={y + 16}
                fontSize="12"
                fontWeight={600}
                fill={hot ? "var(--accent)" : "var(--fg)"}
              >
                {truncate(o.id, 26)}
              </text>
              <text x={rightX + 9} y={y + 30} fontSize="10.5" fill="var(--muted)">
                {o.obj_type}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}

function truncate(s: string, n: number): string {
  return s.length > n ? s.slice(0, n - 1) + "…" : s;
}
