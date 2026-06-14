import type { Metadata } from "next";
import DiffLab from "./DiffLab";

export const metadata: Metadata = {
  title: "Receipt Diff & Tamper Lab — affidavit",
  description:
    "Paste two affidavit receipt JSONs and get a field-level structural diff plus a per-side readout of which certify stage would reject each one.",
};

export default function DiffPage() {
  return (
    <>
      <h1>Receipt Diff &amp; Tamper Lab</h1>
      <p className="muted">
        Paste two <code>core/v1</code> receipts — <span className="green">left</span> is your
        baseline, <span className="warn">right</span> is the candidate — and press{" "}
        <code>diff</code>. Everything is computed in your browser from the JSON you provide:
        a side-by-side comparison of top-level fields, an event-by-event aligned diff, and a
        per-side <em>structural verdict</em> showing which certify stage (
        <code>check_format</code>, <code>continuity</code>, <code>verify_commitments</code>,{" "}
        <code>evaluate_profile</code>) the input would be rejected at — so you can see exactly
        which tamper trips which stage. These are real, decidable structural checks only;
        BLAKE3 chain integrity is <strong>not</strong> verified here — use Studio for the full
        cryptographic check. The two boxes start with clearly-labelled example inputs that
        differ in an instructive way.
      </p>
      <DiffLab />
    </>
  );
}
