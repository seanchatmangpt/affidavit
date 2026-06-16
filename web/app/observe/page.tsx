import { readSemconvRegistry, type SemconvGroup } from "../api/semconv/registry";

export const runtime = "nodejs";
export const dynamic = "force-dynamic"; // always reflect the registry on disk

/**
 * /observe — surfaces the REAL OTel span shape that affidavit pins in its Weaver
 * semantic-convention registry (semconv/registry/affidavit.yaml). The page reads
 * and parses that YAML directly (server component) so it works even when the
 * /api/semconv route is never called; the API is an additional machine-readable
 * surface. No fixtures: if the registry is absent, an honest unavailable state.
 */
export default async function ObservePage() {
  const reg = await readSemconvRegistry();

  return (
    <>
      <h1>Observability</h1>
      <p className="muted">
        affidavit emits an observable OpenTelemetry span on <code>verify</code> (and the
        other receipt-pipeline operations). The span&apos;s <em>shape</em> — its name and
        attributes — is governed by an OTel <strong>Weaver</strong> semantic-convention
        registry that lives in the repo and is validated by a real{" "}
        <code>weaver registry check</code>.
      </p>

      {!reg.available ? (
        <p className="unavailable">
          semconv registry unavailable: {reg.message}
          <br />
          <span className="src">expected source: {reg.expectedSource}</span>
        </p>
      ) : (
        <>
          <p className="muted">
            Registry{reg.name ? <> <code>{reg.name}</code></> : null}
            {reg.semconvVersion ? <> · semconv <code>{reg.semconvVersion}</code></> : null}
            {" · "}
            {reg.groups.length} span group{reg.groups.length === 1 ? "" : "s"} pinned.
          </p>

          {reg.groups.map((g) => (
            <GroupTable key={g.id} group={g} />
          ))}

          <p className="src">source: {reg.sources.join(" · ")}</p>
        </>
      )}

      <ScopePanel />
    </>
  );
}

function GroupTable({ group }: { group: SemconvGroup }) {
  return (
    <section>
      <h2>
        span: <code>{group.id}</code>
      </h2>
      <p className="muted">
        {group.brief ?? "—"}
        {group.type ? <> · type <code>{group.type}</code></> : null}
        {group.spanKind ? <> · kind <code>{group.spanKind}</code></> : null}
        {group.stability ? <> · stability <code>{group.stability}</code></> : null}
      </p>

      {group.attributes.length === 0 ? (
        <p className="unavailable">no attributes parsed for this group.</p>
      ) : (
        <table>
          <thead>
            <tr>
              <th>attribute id</th>
              <th>type</th>
              <th>requirement</th>
              <th>brief</th>
            </tr>
          </thead>
          <tbody>
            {group.attributes.map((a) => (
              <tr key={a.id}>
                <td>
                  <code>{a.id}</code>
                </td>
                <td>
                  <code>{a.type}</code>
                  {a.members && a.members.length > 0 && (
                    <div className="muted" style={{ marginTop: "0.25rem" }}>
                      {a.members.map((m) => (
                        <span key={m.id}>
                          <code>{m.value}</code>{" "}
                        </span>
                      ))}
                    </div>
                  )}
                </td>
                <td className={a.requirementLevel === "required" ? "green" : "muted"}>
                  {a.requirementLevel ?? "—"}
                </td>
                <td>{a.brief ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function ScopePanel() {
  return (
    <>
      <h2>scope — what is witnessed, and what is not</h2>
      <div className="grid">
        <div className="card">
          <div className="label">semconv registry shape</div>
          <div className="metric green">CLOSED</div>
          <p className="muted">
            The emitted span shape (<code>operation</code>, <code>target</code>) is pinned
            in <code>semconv/registry</code> and validated against a real OTel Weaver
            registry via <code>weaver registry check</code> (exits 0). A deliberately-broken{" "}
            <code>semconv/registry_broken</code> serves as the negative control (exits ≠ 0),
            and a coherence check asserts the registry attribute ids equal the{" "}
            <code>SpanRecord</code> fields in <code>src/tracing.rs</code>.
          </p>
          <p className="src">witness: tests/otel_weaver_registry.rs</p>
        </div>
        <div className="card">
          <div className="label">live collector export</div>
          <div className="metric warn">OPEN-substrate</div>
          <p className="muted">
            Full OpenTelemetry <strong>SDK export to a running collector</strong>{" "}
            (Jaeger/OTLP) is <span className="warn">not yet witnessed</span> — no test
            currently captures an exported span from a live collector. See the honest scope
            note in <code>src/tracing.rs</code> and STATUS.md.
          </p>
          <p className="src">status: OPEN-substrate (not fabricated)</p>
        </div>
      </div>
      <p className="muted">
        This split is reflected verbatim from STATUS.md: the registry surface is closed and
        machine-validated; the live-export surface is honestly marked open rather than
        presented as done.
      </p>
    </>
  );
}
