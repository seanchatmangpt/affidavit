import { Suspense } from "react";
import {
  affiVersion,
  capabilities,
  coverageGreenCount,
  latestGgenReceipt,
} from "@/lib/affidavit";

export const runtime = "nodejs";

// Static shell renders instantly (PPR); each real-data tile streams in.
export default function Dashboard() {
  return (
    <>
      <h1>affidavit — faithful representation</h1>
      <p className="muted">
        Every value below is captured at request time from the real <code>affi</code>{" "}
        binary or the repo tree. Nothing is fabricated — see{" "}
        <code>REPRESENTATION_MAP.md</code>.
      </p>
      <div className="grid">
        <Suspense fallback={<Tile label="affi version" value="…" src="affi --version" />}>
          <VersionTile />
        </Suspense>
        <Suspense fallback={<Tile label="capabilities" value="…" src="affi --introspect" />}>
          <CapabilitiesTile />
        </Suspense>
        <Suspense fallback={<Tile label="coverage 🟢 rows" value="…" src="reference/COVERAGE.md" />}>
          <CoverageTile />
        </Suspense>
        <Suspense fallback={<Tile label="latest ggen receipt" value="…" src=".ggen/receipts/latest.json" />}>
          <ReceiptTile />
        </Suspense>
      </div>
    </>
  );
}

function Tile({ label, value, src, cls }: { label: string; value: string; src: string; cls?: string }) {
  return (
    <div className="card">
      <div className="label">{label}</div>
      <div className={`metric ${cls ?? ""}`}>{value}</div>
      <div className="src">source: {src}</div>
    </div>
  );
}

async function VersionTile() {
  const v = await affiVersion();
  return <Tile label="affi version" value={v} src="affi --version" cls={v === "unavailable" ? "unavailable" : ""} />;
}

async function CapabilitiesTile() {
  try {
    const caps = await capabilities();
    return <Tile label="capabilities" value={String(caps.length)} src="affi --introspect" />;
  } catch {
    return <Tile label="capabilities" value="binary not built" src="affi --introspect" cls="unavailable" />;
  }
}

async function CoverageTile() {
  const n = await coverageGreenCount();
  return (
    <Tile
      label="coverage 🟢 rows"
      value={n === null ? "unavailable" : String(n)}
      src="reference/COVERAGE.md"
      cls={n === null ? "unavailable" : "green"}
    />
  );
}

async function ReceiptTile() {
  const r = await latestGgenReceipt();
  return (
    <Tile
      label="latest ggen receipt"
      value={r ? r.output_hashes.length + " outputs" : "none"}
      src=".ggen/receipts/latest.json"
      cls={r ? "" : "unavailable"}
    />
  );
}
