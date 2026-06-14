"use server";

import { runPipeline, type Verdict } from "@/lib/affidavit";

export interface PipelineResult {
  verdict: Verdict | null;
  error: string | null;
  echo: string | null;
}

/**
 * Server action: parse the submitted events and run the REAL affi pipeline.
 * Returns the genuine Verdict (or the binary's error). No fabrication.
 */
export async function runPipelineAction(
  _prev: PipelineResult,
  formData: FormData,
): Promise<PipelineResult> {
  const raw = String(formData.get("events") ?? "").trim();
  // One event per line: "type | object | payload" (object/payload optional).
  const events = raw
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean)
    .map((line) => {
      const [type = "", object = "", payload = ""] = line.split("|").map((s) => s.trim());
      return {
        type,
        object: object || "obj-1:artifact",
        payload: payload || type,
      };
    });

  if (events.length === 0) {
    return { verdict: null, error: "enter at least one event (one per line)", echo: raw };
  }
  try {
    const verdict = await runPipeline(events);
    return { verdict, error: null, echo: raw };
  } catch (e) {
    return { verdict: null, error: e instanceof Error ? e.message : String(e), echo: raw };
  }
}
