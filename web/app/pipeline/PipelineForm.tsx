"use client";

import { useActionState } from "react";
import { runPipelineAction, type PipelineResult } from "./actions";

const INITIAL: PipelineResult = { verdict: null, error: null, echo: null };

const HONEST = "create | a1:artifact | seed\ntransform | a1:artifact | work\nrelease | a1:artifact | ship";

export default function PipelineForm() {
  const [state, action, pending] = useActionState(runPipelineAction, INITIAL);

  return (
    <>
      <form action={action}>
        <p className="muted">
          One event per line: <code>type | object | payload</code>. The form runs the
          REAL <code>affi</code> emit→assemble→verify pipeline in a temp dir and shows
          the genuine <code>Verdict</code>.
        </p>
        <textarea name="events" rows={5} defaultValue={HONEST}
          style={{ width: "100%", background: "#1b2130", color: "inherit", border: "1px solid #232a39", borderRadius: 6, padding: "0.6rem", font: "inherit" }} />
        <div style={{ marginTop: "0.6rem" }}>
          <button type="submit" disabled={pending}
            style={{ background: "#1f6feb", color: "white", border: 0, borderRadius: 6, padding: "0.5rem 1rem", cursor: "pointer" }}>
            {pending ? "running affi…" : "run pipeline"}
          </button>
        </div>
      </form>

      {state.error && <p className="unavailable" style={{ marginTop: "1rem" }}>error: {state.error}</p>}

      {state.verdict && (
        <div style={{ marginTop: "1rem" }}>
          <h2>
            Verdict:{" "}
            <span className={state.verdict.accepted ? "green" : "warn"}>
              {state.verdict.accepted ? "ACCEPTED" : "REJECTED"}
            </span>
          </h2>
          <p className="muted">{state.verdict.reason}</p>
          <table>
            <thead><tr><th>stage</th><th>passed</th><th>detail</th></tr></thead>
            <tbody>
              {state.verdict.outcomes.map((o, i) => (
                <tr key={i}>
                  <td><code>{o.stage}</code></td>
                  <td className={o.passed ? "green" : "warn"}>{o.passed ? "✓" : "✗"}</td>
                  <td className="muted">{o.detail}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className="src">source: real affi receipt verify --format json</p>
        </div>
      )}
    </>
  );
}
