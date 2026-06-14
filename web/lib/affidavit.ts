// Faithful data layer for the affidavit representation UI.
//
// EVERY value the UI renders comes from here, and everything here is captured from
// a REAL source at request time — the `affi` binary or a file in the repo tree.
// There are NO fixtures. If the binary or a file is missing, these functions throw
// or return an explicit "unavailable" marker; the UI must show that honestly rather
// than substitute fabricated data. See REPRESENTATION_MAP.md (E1–E13).

import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, access } from "node:fs/promises";
import path from "node:path";

const execFileAsync = promisify(execFile);

// The repo root is the parent of web/. Overridable for tests/deploys.
export const REPO_ROOT = process.env.AFFIDAVIT_ROOT
  ? path.resolve(process.env.AFFIDAVIT_ROOT)
  : path.resolve(process.cwd(), "..");

async function firstExisting(...candidates: string[]): Promise<string | null> {
  for (const c of candidates) {
    try {
      await access(c);
      return c;
    } catch {
      /* keep looking */
    }
  }
  return null;
}

/** Resolve the real `affi` binary (release preferred, then debug). */
export async function affiBinary(): Promise<string | null> {
  return firstExisting(
    path.join(REPO_ROOT, "target", "release", "affi"),
    path.join(REPO_ROOT, "target", "debug", "affi"),
  );
}

async function runAffi(args: string[]): Promise<string> {
  const bin = await affiBinary();
  if (!bin) throw new Error("affi binary not built — run `cargo build` in the repo root");
  const { stdout } = await execFileAsync(bin, args, {
    cwd: REPO_ROOT,
    maxBuffer: 16 * 1024 * 1024,
  });
  return stdout;
}

// ── E1: capability manifest (affi --introspect) ────────────────────────────────
export interface Capability {
  name: string;
  description: string;
  parameters: {
    type: string;
    properties: Record<string, { type: string; items?: { type: string } }>;
    required?: string[];
  };
}

export async function capabilities(): Promise<Capability[]> {
  const out = await runAffi(["--introspect"]);
  return JSON.parse(out) as Capability[];
}

// ── E13: version (affi --version) ──────────────────────────────────────────────
export async function affiVersion(): Promise<string> {
  try {
    return (await runAffi(["--version"])).trim();
  } catch {
    return "unavailable";
  }
}

// ── E8: ggen provenance receipts (.ggen/receipts/*.json) ───────────────────────
export interface GgenReceipt {
  operation_id: string;
  timestamp: string;
  input_hashes: string[];
  output_hashes: string[];
  signature: string;
  previous_receipt_hash: string | null;
}

export async function latestGgenReceipt(): Promise<GgenReceipt | null> {
  const p = path.join(REPO_ROOT, ".ggen", "receipts", "latest.json");
  try {
    return JSON.parse(await readFile(p, "utf8")) as GgenReceipt;
  } catch {
    return null;
  }
}

// ── E9/E10/E12: authoritative markdown documents (read verbatim) ───────────────
export type DocId = "coverage" | "docCoverage" | "status" | "representationMap";

const DOC_PATHS: Record<DocId, string> = {
  coverage: "reference/COVERAGE.md",
  docCoverage: "DOC_COVERAGE_LOG.md",
  status: "STATUS.md",
  representationMap: "REPRESENTATION_MAP.md",
};

export async function readDoc(id: DocId): Promise<string | null> {
  try {
    return await readFile(path.join(REPO_ROOT, DOC_PATHS[id]), "utf8");
  } catch {
    return null;
  }
}

/** Count 🟢 rows in the coverage gap-grid — a real metric mined from the doc. */
export async function coverageGreenCount(): Promise<number | null> {
  const md = await readDoc("coverage");
  if (md === null) return null;
  return (md.match(/🟢/g) ?? []).length;
}

// ── E3/E4: run the REAL emit→assemble→verify pipeline (server action) ──────────
import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";

export interface CheckOutcome { stage: string; passed: boolean; detail: string }
export interface Verdict {
  accepted: boolean;
  profile?: string;
  outcomes: CheckOutcome[];
  reason: string;
}
export interface PipelineEvent { type: string; object: string; payload: string }

/**
 * Drive the real `affi` binary through emit→assemble→verify in an isolated temp
 * working dir and return the genuine `Verdict`. No fixtures: the verdict is exactly
 * what the binary computes over the bytes the caller supplied.
 */
export async function runPipeline(events: PipelineEvent[]): Promise<Verdict> {
  const bin = await affiBinary();
  if (!bin) throw new Error("affi binary not built — run `cargo build` in the repo root");
  if (events.length === 0) throw new Error("at least one event is required");

  const dir = await mkdtemp(path.join(os.tmpdir(), "affi-web-"));
  try {
    for (const ev of events) {
      await execFileAsync(
        bin,
        ["receipt", "emit", "--payload", ev.payload, "--object", ev.object, "--type", ev.type],
        { cwd: dir, maxBuffer: 8 * 1024 * 1024 },
      );
    }
    const receiptPath = path.join(dir, "receipt.json");
    await execFileAsync(bin, ["receipt", "assemble", "--out", receiptPath], { cwd: dir });
    // verify exits non-zero on REJECT, so capture both streams and parse regardless.
    const { stdout } = await execFileAsync(
      bin,
      ["--format", "json", "receipt", "verify", receiptPath],
      { cwd: dir, maxBuffer: 8 * 1024 * 1024 },
    ).catch((e: { stdout?: string }) => ({ stdout: e.stdout ?? "" }));
    return JSON.parse(stdout) as Verdict;
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}
