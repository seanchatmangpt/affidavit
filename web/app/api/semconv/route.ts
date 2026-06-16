import { readSemconvRegistry } from "./registry";

// Node runtime: this route reads the real semconv registry YAML from the repo tree.
export const runtime = "nodejs";
export const dynamic = "force-dynamic";

/**
 * GET /api/semconv — the machine-readable twin of the /observe page.
 *
 * Reads the real OTel Weaver semantic-convention registry under
 * `semconv/registry/` and returns the parsed span groups + attribute definitions
 * (id / type / brief / requirement level, plus enum members and the span name) as
 * JSON. No fixtures: if the registry directory is absent or unparseable, responds
 * with `{ available: false, message }` and HTTP 503 — never fabricated span data.
 */
export async function GET() {
  const result = await readSemconvRegistry();

  if (!result.available) {
    return Response.json(result, {
      status: 503,
      headers: { "x-affidavit-source": result.expectedSource },
    });
  }

  return Response.json(result, {
    headers: { "x-affidavit-source": result.sources.join(", ") },
  });
}
