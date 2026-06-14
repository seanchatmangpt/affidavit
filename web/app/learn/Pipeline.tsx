"use client";

import { useState } from "react";
import styles from "./learn.module.css";

type Stage = {
  fn: string;
  name: string;
  check: string;
  /** what makes this stage emit REJECT */
  failExample: string;
};

// The real 7-stage certify pipeline, in order.
// decode → check_format → chain_integrity → continuity →
// verify_commitments → evaluate_profile → emit_verdict
const STAGES: Stage[] = [
  {
    fn: "decode",
    name: "decode",
    check:
      "The receipt is structurally present and its version field parses.",
    failExample:
      "format_version is empty or unparseable → the verifier has nothing well-formed to read.",
  },
  {
    fn: "check_format",
    name: "check_format",
    check:
      'format_version equals the standard this verifier knows: "core/v1".',
    failExample:
      'format_version is any other value (e.g. "core/v2") → unknown standard, REJECT.',
  },
  {
    fn: "chain_integrity",
    name: "chain_integrity",
    check:
      "Recompute the rolling BLAKE3 chain hash from the event bytes and compare it to the stored chain_hash.",
    failExample:
      "Any event byte changed → the link re-routes and the recomputed hash no longer matches chain_hash.",
  },
  {
    fn: "continuity",
    name: "continuity",
    check:
      "seq is contiguous from 0 with no gaps, and every event id is unique.",
    failExample:
      "A seq gap (0,1,3…) or a duplicate event id breaks contiguity/uniqueness.",
  },
  {
    fn: "verify_commitments",
    name: "verify_commitments",
    check:
      "Every payload commitment is a well-formed BLAKE3 digest: 64 lowercase hex characters. Commitments only — raw payloads are never stored or seen.",
    failExample:
      "A commitment is malformed (wrong length, uppercase, non-hex) → not a valid digest.",
  },
  {
    fn: "evaluate_profile",
    name: "evaluate_profile",
    check:
      "Profile core/v1: each event carries a non-empty event_type and a commitment.",
    failExample:
      "An event is missing its event_type or its commitment.",
  },
  {
    fn: "emit_verdict",
    name: "emit_verdict",
    check:
      "ACCEPT iff every prior stage passed; otherwise REJECT with the first failing stage's reason.",
    failExample:
      "Any earlier stage failed → the verdict is REJECT, attributed to the first failing stage.",
  },
];

export default function Pipeline() {
  // Start with the first stage open so the page reads as a guided tour.
  const [active, setActive] = useState(0);
  const stage = STAGES[active];
  const isVerdict = active === STAGES.length - 1;

  return (
    <div>
      <div className={styles.flow} role="tablist" aria-label="Certify pipeline stages">
        {STAGES.map((s, i) => {
          const last = i === STAGES.length - 1;
          const isActive = i === active;
          return (
            <div className={styles.node} key={s.fn}>
              <button
                type="button"
                role="tab"
                aria-selected={isActive}
                aria-controls="stage-detail"
                id={`stage-tab-${i}`}
                className={[
                  styles.stageBtn,
                  isActive ? (last ? styles.stageBtnFail : styles.stageBtnActive) : "",
                ]
                  .filter(Boolean)
                  .join(" ")}
                onClick={() => setActive(i)}
              >
                <span className={styles.stageNo}>{i + 1}</span>
                <span className={styles.stageName}>{s.name}</span>
              </button>
              {!last && (
                <span
                  className={[
                    styles.arrow,
                    i === STAGES.length - 2 ? styles.verdictArrow : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                  aria-hidden="true"
                />
              )}
            </div>
          );
        })}
      </div>

      <div
        className={styles.detail}
        id="stage-detail"
        role="tabpanel"
        aria-labelledby={`stage-tab-${active}`}
      >
        <div className={styles.detailHead}>
          <span className={styles.stageNo}>{active + 1}</span>
          <h3 className={styles.detailTitle}>{stage.name}</h3>
          <code className={styles.detailFn}>{stage.fn}()</code>
          {isVerdict && <span className="green">final stage</span>}
        </div>

        <div className={styles.detailRow}>
          <span className={styles.rowLabel}>Decidable check</span>
          <span>{stage.check}</span>
        </div>
        <div className={styles.detailRow}>
          <span className={styles.rowLabel}>
            <span className={styles.failBadge}>reject</span>
          </span>
          <span className="warn">{stage.failExample}</span>
        </div>

        <p className={styles.hint}>
          Tip: click any stage above to inspect what it proves and what makes it
          fail.
        </p>
      </div>
    </div>
  );
}
