import { capabilities, type Capability } from "@/lib/affidavit";

export const runtime = "nodejs";
export const dynamic = "force-dynamic"; // always reflect the live binary

export default async function CapabilitiesPage() {
  let caps: Capability[] | null = null;
  let error: string | null = null;
  try {
    caps = await capabilities();
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  }

  return (
    <>
      <h1>Capabilities</h1>
      <p className="muted">
        The real tool-calling manifest, captured live from{" "}
        <code>affi --introspect</code>. Each row is a verb the binary actually exposes.
      </p>
      {error && (
        <p className="unavailable">
          capability manifest unavailable: {error} — build the binary with{" "}
          <code>cargo build</code>.
        </p>
      )}
      {caps && (
        <table>
          <thead>
            <tr>
              <th>name</th>
              <th>description</th>
              <th>required params</th>
            </tr>
          </thead>
          <tbody>
            {caps.map((c) => (
              <tr key={c.name}>
                <td><code>{c.name}</code></td>
                <td>{c.description}</td>
                <td className="muted">
                  {(c.parameters.required ?? []).join(", ") || "—"}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {caps && <p className="src">{caps.length} verbs · source: affi --introspect</p>}
    </>
  );
}
