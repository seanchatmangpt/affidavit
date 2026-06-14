"use client";

import { useMemo, useState } from "react";
import {
  buildExampleReceipt,
  canonicalJson,
  genesisHash,
  parseReceipt,
  recomputeChain,
  verifyReceipt,
  type Verdict,
} from "@/lib/verify-client";
import { selfTest } from "@/lib/blake3";
import styles from "./studio.module.css";

// A clearly-labeled, EDITABLE example. This is NOT captured data: it is a real
// chain computed in-browser by the same rule the binary uses, pretty-printed so a
// user can read and tweak it. Editing any field (e.g. a commitment or a seq) and
// re-verifying shows a stage flip exactly as it would against `affi`.
function exampleText(): string {
  return JSON.stringify(buildExampleReceipt(), null, 2);
}

interface Result {
  verdict: Verdict;
  recomputed: string;
  storedChainHash: string;
  canonicalEventBytes: { id: string; canonical: string }[];
  receiptCanonical: string;
}

export default function StudioPage() {
  const [text, setText] = useState<string>(exampleText);
  const [result, setResult] = useState<Result | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Run the blake3 self-test once and surface it honestly in the UI: the studio
  // only claims parity if its hash primitive matches the official vectors.
  const blake3Status = useMemo(() => selfTest(), []);

  function onVerify() {
    const parsed = parseReceipt(text);
    if (!parsed.ok) {
      setResult(null);
      setError(parsed.error);
      return;
    }
    setError(null);
    const receipt = parsed.receipt;
    const verdict = verifyReceipt(receipt);
    const recomputed = recomputeChain(receipt.events);
    setResult({
      verdict,
      recomputed,
      storedChainHash: receipt.chain_hash,
      canonicalEventBytes: receipt.events.map((ev) => ({
        id: ev.id,
        canonical: canonicalJson({
          id: ev.id,
          seq: ev.seq,
          event_type: ev.event_type,
          objects: ev.objects,
          payload_commitment: ev.payload_commitment,
        }),
      })),
      receiptCanonical: canonicalJson(parsed.value),
    });
  }

  function onReset() {
    setText(exampleText());
    setResult(null);
    setError(null);
  }

  return (
    <>
      <h1>Verifier Studio</h1>
      <p className="muted">
        A faithful, in-browser port of affidavit&apos;s 7-stage certify pipeline.
        Paste a Receipt JSON and get the REAL verdict — every stage is recomputed
        client-side from the bytes you provide, including the rolling BLAKE3 chain
        hash. Nothing is fabricated: the result is a pure function of your input.
      </p>

      <p className={styles.exampleNote}>example input, edit me</p>
      <textarea
        className={styles.textarea}
        rows={18}
        spellCheck={false}
        value={text}
        onChange={(e) => setText(e.target.value)}
        aria-label="Receipt JSON (example input, edit me)"
      />
      <div className={styles.toolbar}>
        <button type="button" className={styles.button} onClick={onVerify}>
          verify
        </button>
        <button type="button" className={styles.secondary} onClick={onReset}>
          reset example
        </button>
        <span className={`${styles.selfTest} ${blake3Status.ok ? "green" : "warn"}`}>
          {blake3Status.ok
            ? "✓ blake3 self-test passed (official vectors)"
            : `✗ blake3 self-test FAILED: ${blake3Status.failures.join("; ")}`}
        </span>
      </div>

      {error && (
        <div className={styles.errorBox}>
          <strong>parse error:</strong> {error}
        </div>
      )}

      {result && (
        <div>
          <div className={styles.verdictRow}>
            <span
              className={`${styles.headline} ${
                result.verdict.accepted ? "green" : "warn"
              }`}
            >
              {result.verdict.accepted ? "ACCEPTED" : "REJECTED"}
            </span>
            <span className="muted">{result.verdict.reason}</span>
          </div>

          <table style={{ marginTop: "1rem" }}>
            <thead>
              <tr>
                <th>stage</th>
                <th>passed</th>
                <th>detail</th>
              </tr>
            </thead>
            <tbody>
              {result.verdict.outcomes.map((o, i) => (
                <tr key={i}>
                  <td>
                    <code>{o.stage}</code>
                  </td>
                  <td className={o.passed ? "green" : "warn"}>
                    {o.passed ? "✓" : "✗"}
                  </td>
                  <td className="muted">{o.detail}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <div className="grid" style={{ marginTop: "1.25rem" }}>
            <div className="card">
              <div className={styles.hashLabel}>recomputed chain hash</div>
              <div className={styles.hashBox}>{result.recomputed}</div>
            </div>
            <div className="card">
              <div className={styles.hashLabel}>stored chain_hash</div>
              <div className={styles.hashBox}>{result.storedChainHash}</div>
            </div>
            <div className="card">
              <div className={styles.hashLabel}>chain match</div>
              <div
                className={`metric ${
                  result.recomputed === result.storedChainHash
                    ? styles.match
                    : styles.mismatch
                }`}
                style={{ fontSize: "1.1rem" }}
              >
                {result.recomputed === result.storedChainHash
                  ? "✓ match"
                  : "✗ mismatch"}
              </div>
              <div className="src">
                genesis h0: <code>{genesisHash().slice(0, 16)}…</code>
              </div>
            </div>
          </div>

          <details className={styles.details}>
            <summary>canonical bytes that were hashed (what you can verify)</summary>
            <p className="muted" style={{ fontSize: 12, marginTop: "0.6rem" }}>
              Each event&apos;s canonical form is compact JSON with object keys
              sorted recursively — byte-identical to Rust&apos;s{" "}
              <code>serde_json</code> over a sorted value. The chain folds{" "}
              <code>blake3( ascii(prev_hex) ++ canonical_bytes(event) )</code>{" "}
              starting from <code>blake3(GENESIS_SEED)</code>.
            </p>
            {result.canonicalEventBytes.map((e) => (
              <div key={e.id} style={{ marginTop: "0.5rem" }}>
                <div className={styles.hashLabel}>
                  event <code>{e.id}</code> canonical bytes
                </div>
                <pre className="doc">{e.canonical}</pre>
              </div>
            ))}
            <div style={{ marginTop: "0.6rem" }}>
              <div className={styles.hashLabel}>full receipt, canonical form</div>
              <pre className="doc">{result.receiptCanonical}</pre>
            </div>
          </details>
        </div>
      )}

      <p className={styles.note}>
        This is a faithful in-browser port of affidavit&apos;s 7-stage pipeline; it
        computes the same verdict as the <code>affi</code> binary. The logic is
        ported from <code>src/verifier.rs</code>, <code>src/chain.rs</code>, and{" "}
        <code>src/types.rs</code> (BLAKE3 + sorted-key canonical JSON), with the
        BLAKE3 primitive validated against the official test vectors at load time.
      </p>
    </>
  );
}
