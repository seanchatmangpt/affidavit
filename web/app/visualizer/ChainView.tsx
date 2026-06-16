"use client";

import React from "react";
import type { ChainLink } from "./model";
import styles from "./visualizer.module.css";

interface Props {
  links: ChainLink[];
  selectedSeq: number | null; // event seq, or null
  onSelect: (seq: number | null) => void;
  // Index into `links` of the first re-routed link (>=0 when tampering). -1 = clean.
  divergeIndex: number;
  // seq of the tampered event (for the "edited here" marker), or null.
  tamperedSeq: number | null;
}

const NODE_W = 150;
const NODE_H = 92;
const GAP = 54; // horizontal gap between nodes (room for arrow + fold glyph)
const PAD_X = 16;
const PAD_Y = 40;
const SVG_H = NODE_H + PAD_Y * 2 + 26;

// Pure-display chip color logic. Re-routed links use --warn; the selected
// event uses --accent; genesis is muted.
export default function ChainView({
  links,
  selectedSeq,
  onSelect,
  divergeIndex,
  tamperedSeq,
}: Props) {
  const width = PAD_X * 2 + links.length * NODE_W + (links.length - 1) * GAP;

  return (
    <div className={styles.svgScroll}>
      <svg
        className={styles.svg}
        width={width}
        height={SVG_H}
        viewBox={`0 0 ${width} ${SVG_H}`}
        role="group"
        aria-label="Receipt hash chain"
      >
        <defs>
          <marker
            id="cv-arrow"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M0,0 L10,5 L0,10 z" fill="var(--muted)" />
          </marker>
          <marker
            id="cv-arrow-warn"
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="7"
            markerHeight="7"
            orient="auto-start-reverse"
          >
            <path d="M0,0 L10,5 L0,10 z" fill="var(--warn)" />
          </marker>
        </defs>

        {links.map((link, i) => {
          const x = PAD_X + i * (NODE_W + GAP);
          const cy = PAD_Y + NODE_H / 2;
          const isGenesis = link.event === null;
          const rerouted = divergeIndex >= 0 && i >= divergeIndex;
          const selected = !isGenesis && link.seq === selectedSeq;
          const isTamperSource = !isGenesis && link.seq === tamperedSeq;

          const stroke = rerouted
            ? "var(--warn)"
            : selected
              ? "var(--accent)"
              : "var(--border)";
          const strokeW = selected || isTamperSource ? 2.2 : 1.2;

          return (
            <g key={i}>
              {/* connector + fold glyph from previous node */}
              {i > 0 && (
                <Connector
                  x1={x - GAP}
                  x2={x}
                  y={cy}
                  warn={divergeIndex >= 0 && i >= divergeIndex}
                />
              )}

              {/* node box */}
              <g
                onClick={() => !isGenesis && onSelect(selected ? null : link.seq)}
                style={{ cursor: isGenesis ? "default" : "pointer" }}
                role={isGenesis ? undefined : "button"}
                aria-pressed={isGenesis ? undefined : selected}
              >
                <rect
                  x={x}
                  y={PAD_Y}
                  width={NODE_W}
                  height={NODE_H}
                  rx={8}
                  fill="var(--panel)"
                  stroke={stroke}
                  strokeWidth={strokeW}
                />

                {/* header line: seq + type */}
                {isGenesis ? (
                  <text
                    x={x + NODE_W / 2}
                    y={PAD_Y + 22}
                    textAnchor="middle"
                    fontSize="12"
                    fontWeight={700}
                    fill="var(--muted)"
                  >
                    GENESIS
                  </text>
                ) : (
                  <>
                    <text x={x + 10} y={PAD_Y + 20} fontSize="11" fill="var(--muted)">
                      seq {link.seq}
                    </text>
                    <text
                      x={x + 10}
                      y={PAD_Y + 39}
                      fontSize="12.5"
                      fontWeight={600}
                      fill={rerouted ? "var(--warn)" : "var(--fg)"}
                    >
                      {truncate(link.event!.event_type, 16)}
                    </text>
                    <text x={x + 10} y={PAD_Y + 56} fontSize="10.5" fill="var(--muted)">
                      {truncate(link.event!.id, 17)}
                    </text>
                  </>
                )}

                {/* illustrative-digest chip */}
                <g>
                  <rect
                    x={x + 10}
                    y={PAD_Y + NODE_H - 24}
                    width={NODE_W - 20}
                    height={16}
                    rx={4}
                    fill={rerouted ? "rgba(210,153,34,0.16)" : "#1b2130"}
                    stroke={rerouted ? "var(--warn)" : "var(--border)"}
                    strokeWidth={1}
                  />
                  <text
                    x={x + NODE_W / 2}
                    y={PAD_Y + NODE_H - 12}
                    textAnchor="middle"
                    fontSize="10.5"
                    fontFamily="ui-monospace, monospace"
                    fill={rerouted ? "var(--warn)" : "var(--green)"}
                  >
                    {isGenesis ? `h0 ${link.digest}` : `${link.digest}`}
                  </text>
                </g>
              </g>

              {/* tamper-source marker */}
              {isTamperSource && (
                <text
                  x={x + NODE_W / 2}
                  y={PAD_Y - 10}
                  textAnchor="middle"
                  fontSize="11"
                  fontWeight={700}
                  fill="var(--warn)"
                >
                  ✎ edited here
                </text>
              )}
              {/* first re-routed label (when divergence is downstream of edit) */}
              {rerouted && i === divergeIndex && !isTamperSource && (
                <text
                  x={x + NODE_W / 2}
                  y={PAD_Y - 10}
                  textAnchor="middle"
                  fontSize="11"
                  fontWeight={700}
                  fill="var(--warn)"
                >
                  re-routes from here →
                </text>
              )}

              {/* bottom caption: what folds into this link */}
              {!isGenesis && (
                <text
                  x={x + NODE_W / 2}
                  y={PAD_Y + NODE_H + 18}
                  textAnchor="middle"
                  fontSize="10"
                  fill="var(--muted)"
                >
                  fold(prev, evt{link.seq})
                </text>
              )}
              {isGenesis && (
                <text
                  x={x + NODE_W / 2}
                  y={PAD_Y + NODE_H + 18}
                  textAnchor="middle"
                  fontSize="10"
                  fill="var(--muted)"
                >
                  blake3(genesis)
                </text>
              )}
            </g>
          );
        })}
      </svg>
    </div>
  );
}

function Connector({
  x1,
  x2,
  y,
  warn,
}: {
  x1: number;
  x2: number;
  y: number;
  warn: boolean;
}) {
  const mid = (x1 + x2) / 2;
  const color = warn ? "var(--warn)" : "var(--muted)";
  const marker = warn ? "url(#cv-arrow-warn)" : "url(#cv-arrow)";
  return (
    <g>
      <line
        x1={x1}
        y1={y}
        x2={x2 - 2}
        y2={y}
        stroke={color}
        strokeWidth={warn ? 2 : 1.4}
        markerEnd={marker}
      />
      {/* fold glyph: a small ⊕ showing prev-hash + event combine */}
      <circle cx={mid} cy={y} r={9} fill="var(--bg)" stroke={color} strokeWidth={1.3} />
      <line x1={mid - 5} y1={y} x2={mid + 5} y2={y} stroke={color} strokeWidth={1.3} />
      <line x1={mid} y1={y - 5} x2={mid} y2={y + 5} stroke={color} strokeWidth={1.3} />
    </g>
  );
}

function truncate(s: string, n: number): string {
  return s.length > n ? s.slice(0, n - 1) + "…" : s;
}
