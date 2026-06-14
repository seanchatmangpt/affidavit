import { capabilities } from "@/lib/affidavit";

// Node runtime: this route shells the real `affi` binary.
export const runtime = "nodejs";
export const dynamic = "force-dynamic";

/**
 * GET /api/introspect — the real capability manifest, proxied verbatim from
 * `affi --introspect`. The machine-readable twin of the /capabilities page.
 */
export async function GET() {
  try {
    const caps = await capabilities();
    return Response.json(caps, {
      headers: { "x-affidavit-source": "affi --introspect" },
    });
  } catch (e) {
    return Response.json(
      { error: e instanceof Error ? e.message : String(e), hint: "run `cargo build` in the repo root" },
      { status: 503 },
    );
  }
}
