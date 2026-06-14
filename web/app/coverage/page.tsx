import { readDoc, coverageGreenCount } from "@/lib/affidavit";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export default async function CoveragePage() {
  const [coverage, docCoverage, status, green] = await Promise.all([
    readDoc("coverage"),
    readDoc("docCoverage"),
    readDoc("status"),
    coverageGreenCount(),
  ]);

  return (
    <>
      <h1>Coverage</h1>
      <p className="muted">
        Rendered verbatim from the authoritative docs in the repo — the UI does not
        summarize or invent; it surfaces what the project actually records.
        {green !== null && (
          <> Mined metric: <span className="green">{green}</span> 🟢 witness rows in the gap-grid.</>
        )}
      </p>

      <h2>STATUS.md</h2>
      <Doc body={status} src="STATUS.md" />

      <h2>DOC_COVERAGE_LOG.md</h2>
      <Doc body={docCoverage} src="DOC_COVERAGE_LOG.md" />

      <h2>reference/COVERAGE.md</h2>
      <Doc body={coverage} src="reference/COVERAGE.md" />
    </>
  );
}

function Doc({ body, src }: { body: string | null; src: string }) {
  if (body === null) {
    return <p className="unavailable">{src} not found in repo.</p>;
  }
  return (
    <>
      <pre className="doc">{body}</pre>
      <p className="src">source: {src} ({body.length.toLocaleString()} chars)</p>
    </>
  );
}
