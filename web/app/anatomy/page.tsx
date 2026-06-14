import Link from "next/link";
import styles from "./anatomy.module.css";

export const metadata = {
  title: "Receipt Anatomy — affidavit",
  description:
    "A field-by-field anatomy of the affidavit Receipt, OperationEvent, ObjectRef, and Verdict: every type, what it means, and which of the 7 certify stages enforces it.",
};

/* ------------------------------------------------------------------ *
 * The 7 certify stages, in pipeline order. Names match src/verifier.rs
 * exactly (stage_decode … stage_evaluate_profile, then emit_verdict).
 * Each carries the CSS-module class that gives it a stable hue so a
 * field badge, the legend entry, and a rejection caption all share one
 * color the eye can trace.
 * ------------------------------------------------------------------ */
type StageKey =
  | "decode"
  | "check_format"
  | "chain_integrity"
  | "continuity"
  | "verify_commitments"
  | "evaluate_profile"
  | "emit_verdict"
  | "none";

const STAGES: {
  key: Exclude<StageKey, "none">;
  n: number;
  name: string;
  desc: string;
  cls: string;
}[] = [
  {
    key: "decode",
    n: 1,
    name: "decode",
    desc: "Receipt is structurally present and format_version is non-empty / parseable.",
    cls: styles.stageDecode,
  },
  {
    key: "check_format",
    n: 2,
    name: "check_format",
    desc: "format_version equals the standard this verifier knows (core/v1).",
    cls: styles.stageCheckFormat,
  },
  {
    key: "chain_integrity",
    n: 3,
    name: "chain_integrity",
    desc: "Recomputed rolling BLAKE3 over the events equals the stored chain_hash.",
    cls: styles.stageChainIntegrity,
  },
  {
    key: "continuity",
    n: 4,
    name: "continuity",
    desc: "seq is contiguous from 0 (no gaps) and every event id is unique.",
    cls: styles.stageContinuity,
  },
  {
    key: "verify_commitments",
    n: 5,
    name: "verify_commitments",
    desc: "Every payload_commitment is a well-formed 64-char lowercase-hex digest.",
    cls: styles.stageVerifyCommitments,
  },
  {
    key: "evaluate_profile",
    n: 6,
    name: "evaluate_profile",
    desc: "CoreV1 profile: each event has a non-empty event_type and a commitment.",
    cls: styles.stageEvaluateProfile,
  },
  {
    key: "emit_verdict",
    n: 7,
    name: "emit_verdict",
    desc: "accepted iff every prior stage passed; reason summarizes the first failure.",
    cls: styles.stageEmitVerdict,
  },
];

const STAGE_INDEX: Record<
  Exclude<StageKey, "none">,
  (typeof STAGES)[number]
> = STAGES.reduce(
  (acc, s) => {
    acc[s.key] = s;
    return acc;
  },
  {} as Record<Exclude<StageKey, "none">, (typeof STAGES)[number]>,
);

function StageBadge({ stage }: { stage: StageKey }) {
  if (stage === "none") {
    return (
      <span
        className={`${styles.badge} ${styles.stageNone}`}
        title="No verify stage decides this — set at construction time or implied by the canonical encoding."
      >
        —
      </span>
    );
  }
  const s = STAGE_INDEX[stage];
  return (
    <span
      className={`${styles.badge} ${s.cls}`}
      title={s.desc}
    >
      {s.n}. {s.name}
    </span>
  );
}

/* ------------------------------------------------------------------ *
 * Field tables. One row per real field from src/types.rs. The JSON keys
 * and shapes are faithful: Blake3Hash serializes as a bare hex string;
 * ObjectRef.qualifier is Option<String> (string | null); the private
 * _seal field is #[serde(skip)] so it never appears in JSON.
 * ------------------------------------------------------------------ */
type Field = {
  name: string;
  type: string;
  meaning: React.ReactNode;
  stages: StageKey[];
};

const RECEIPT_FIELDS: Field[] = [
  {
    name: "format_version",
    type: "string",
    meaning: (
      <>
        Format-standard tag, e.g. <code>&quot;core/v1&quot;</code>.{" "}
        <span className="muted">
          decode checks it is present; check_format checks it equals the
          verifier&rsquo;s standard.
        </span>
      </>
    ),
    stages: ["decode", "check_format"],
  },
  {
    name: "events",
    type: "OperationEvent[]",
    meaning: (
      <>
        The append-only, ordered list of operation-events. Field order is the
        canonical order used for hashing.
      </>
    ),
    stages: ["chain_integrity", "continuity"],
  },
  {
    name: "chain_hash",
    type: "string (64-hex)",
    meaning: (
      <>
        Rolling BLAKE3 over the events in order. Recomputed and compared on
        verify; deserialization also re-verifies it.
      </>
    ),
    stages: ["chain_integrity"],
  },
  {
    name: "_seal",
    type: "() · private, not serialized",
    meaning: (
      <>
        Private zero-sized field present only in Rust. It makes external
        struct-literal construction a <strong>compile error</strong> (
        <code>E0451</code>), so sealed receipts can only be built through{" "}
        <code>ChainAssembler::finalize</code>. Marked{" "}
        <code>#[serde(skip)]</code> &mdash; it never appears in JSON.
      </>
    ),
    stages: ["none"],
  },
];

const EVENT_FIELDS: Field[] = [
  {
    name: "id",
    type: "string",
    meaning: <>Identifier of this event, unique within the receipt.</>,
    stages: ["continuity"],
  },
  {
    name: "seq",
    type: "u64",
    meaning: (
      <>
        Monotonic logical sequence number (0, 1, 2, …) &mdash; deterministic
        order, never wall-clock time. Contiguity from 0 is enforced.
      </>
    ),
    stages: ["continuity"],
  },
  {
    name: "event_type",
    type: "string",
    meaning: (
      <>
        The kind of operation recorded. Must be non-empty under the CoreV1
        profile. (Also folded into the chain, so editing it breaks the hash.)
      </>
    ),
    stages: ["evaluate_profile", "chain_integrity"],
  },
  {
    name: "objects",
    type: "ObjectRef[]",
    meaning: (
      <>
        Qualified OCEL object references this event relates to. Part of the
        canonical bytes, so it is covered by the chain.
      </>
    ),
    stages: ["chain_integrity"],
  },
  {
    name: "payload_commitment",
    type: "string (64-hex)",
    meaning: (
      <>
        BLAKE3 commitment to the event&rsquo;s payload bytes. The raw payload is{" "}
        <strong>never stored</strong>. Well-formedness (64 lowercase-hex chars)
        is checked; presence is required by the profile.
      </>
    ),
    stages: ["verify_commitments", "evaluate_profile"],
  },
];

const OBJECTREF_FIELDS: Field[] = [
  {
    name: "id",
    type: "string",
    meaning: <>Stable identifier of the referenced object.</>,
    stages: ["chain_integrity"],
  },
  {
    name: "obj_type",
    type: "string",
    meaning: (
      <>
        OCEL object type (the class of the object).{" "}
        <span className="warn">Note the JSON key is exactly</span>{" "}
        <code>&quot;obj_type&quot;</code>.
      </>
    ),
    stages: ["chain_integrity"],
  },
  {
    name: "qualifier",
    type: "string | null",
    meaning: (
      <>
        Optional role of the object in the event (Rust{" "}
        <code>Option&lt;String&gt;</code>). Serializes as a string, or{" "}
        <code>null</code> when absent.
      </>
    ),
    stages: ["chain_integrity"],
  },
];

const VERDICT_FIELDS: Field[] = [
  {
    name: "accepted",
    type: "boolean",
    meaning: (
      <>
        <code>true</code> (ACCEPT) only when every required stage passed,{" "}
        otherwise <code>false</code> (REJECT).
      </>
    ),
    stages: ["emit_verdict"],
  },
  {
    name: "profile",
    type: '"core/v1"',
    meaning: (
      <>
        The conformance profile the receipt was evaluated under (Rust enum{" "}
        <code>ProfileId::CoreV1</code>, serialized as its string id).
      </>
    ),
    stages: ["evaluate_profile"],
  },
  {
    name: "outcomes",
    type: "{ stage, passed, detail }[]",
    meaning: (
      <>
        One <code>CheckOutcome</code> per stage, in pipeline order: the{" "}
        <code>stage</code> name, whether it <code>passed</code>, and a
        human-readable <code>detail</code>.
      </>
    ),
    stages: ["emit_verdict"],
  },
  {
    name: "reason",
    type: "string",
    meaning: (
      <>
        Summary of the verdict: the first failing stage&rsquo;s{" "}
        <code>stage: detail</code>, or <code>&quot;all stages passed&quot;</code>
        .
      </>
    ),
    stages: ["emit_verdict"],
  },
];

function FieldTable({
  type,
  fields,
  jsonNote,
}: {
  type: string;
  fields: Field[];
  jsonNote?: React.ReactNode;
}) {
  return (
    <div className="card" style={{ marginBottom: "1.25rem" }}>
      <h3 className={styles.typeName}>
        <code>{type}</code>
      </h3>
      {jsonNote ? <p className={styles.note}>{jsonNote}</p> : null}
      <table>
        <thead>
          <tr>
            <th style={{ width: "20%" }}>field</th>
            <th style={{ width: "22%" }}>type</th>
            <th>meaning</th>
            <th style={{ width: "22%" }}>enforced by stage</th>
          </tr>
        </thead>
        <tbody>
          {fields.map((f) => (
            <tr key={f.name}>
              <td className={styles.fieldCell}>
                <code>{f.name}</code>
              </td>
              <td className={styles.typeCell}>{f.type}</td>
              <td>{f.meaning}</td>
              <td>
                {f.stages.map((s, i) => (
                  <StageBadge key={`${f.name}-${i}`} stage={s} />
                ))}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/* ------------------------------------------------------------------ *
 * Rejection gallery. Honest TEACHING inputs: each snippet is a
 * deliberately MALFORMED receipt fragment, captioned with the real
 * stage that rejects it and why. These are NOT captured tool output.
 * Stage captions/details mirror src/verifier.rs error messages.
 * ------------------------------------------------------------------ */
type Reject = {
  title: string;
  stage: Exclude<StageKey, "none">;
  why: React.ReactNode;
  /* lines: tuple [text, isBad?] so the offending token can be highlighted. */
  lines: [string, boolean?][];
};

const REJECTS: Reject[] = [
  {
    title: "Unknown format version",
    stage: "check_format",
    why: (
      <>
        decode passes (it is present), but check_format requires{" "}
        <code>format_version == &quot;core/v1&quot;</code>. detail:{" "}
        <em>expected format_version core/v1, found core/v2</em>.
      </>
    ),
    lines: [
      ["{", false],
      ['  "format_version": "core/v2",', true],
      ['  "events": [ /* … */ ],', false],
      ['  "chain_hash": "…"', false],
      ["}", false],
    ],
  },
  {
    title: "Empty format version",
    stage: "decode",
    why: (
      <>
        The very first stage fails: <code>format_version</code> is empty after
        trimming. detail: <em>format_version is empty or unparseable</em>.
      </>
    ),
    lines: [
      ["{", false],
      ['  "format_version": "",', true],
      ['  "events": [],', false],
      ['  "chain_hash": ""', false],
      ["}", false],
    ],
  },
  {
    title: "Gap in seq",
    stage: "continuity",
    why: (
      <>
        seq must be contiguous from 0 by array position. Here position 1 holds{" "}
        <code>seq: 2</code>. detail:{" "}
        <em>seq gap at position 1: expected 1, found 2</em>.
      </>
    ),
    lines: [
      ['"events": [', false],
      ['  { "id": "e0", "seq": 0, "event_type": "init",  … },', false],
      ['  { "id": "e1", "seq": 2, "event_type": "write", … }', true],
      ["]", false],
    ],
  },
  {
    title: "Duplicate event id",
    stage: "continuity",
    why: (
      <>
        Each event <code>id</code> must be unique within the receipt. Two events
        share <code>&quot;e0&quot;</code>. detail:{" "}
        <em>duplicate event id: e0</em>.
      </>
    ),
    lines: [
      ['"events": [', false],
      ['  { "id": "e0", "seq": 0, "event_type": "init",  … },', false],
      ['  { "id": "e0", "seq": 1, "event_type": "write", … }', true],
      ["]", false],
    ],
  },
  {
    title: "Short commitment (63 hex chars)",
    stage: "verify_commitments",
    why: (
      <>
        A BLAKE3 digest is exactly 64 lowercase-hex chars; this one is 63.
        detail:{" "}
        <em>
          event e0 has a malformed commitment (expected 64 lowercase hex chars)
        </em>
        .
      </>
    ),
    lines: [
      ['{ "id": "e0", "seq": 0, "event_type": "write",', false],
      [
        '  "payload_commitment": "af00…3 (only 63 chars)" }',
        true,
      ],
    ],
  },
  {
    title: "Empty event_type",
    stage: "evaluate_profile",
    why: (
      <>
        The CoreV1 profile requires a non-empty <code>event_type</code> on every
        event. detail: <em>event e0 has an empty event_type</em>.
      </>
    ),
    lines: [
      ['{ "id": "e0", "seq": 0,', false],
      ['  "event_type": "",', true],
      ['  "objects": [], "payload_commitment": "…64 hex…" }', false],
    ],
  },
  {
    title: "Edited field, chain not rebuilt",
    stage: "chain_integrity",
    why: (
      <>
        An attacker flips <code>event_type</code> in the JSON but leaves the old{" "}
        <code>chain_hash</code>. The recomputed hash no longer matches. detail:{" "}
        <em>chain hash mismatch: stored …, recomputed …</em>. (Deserialization
        rejects this too, before verify even runs.)
      </>
    ),
    lines: [
      ['"events": [ { "id": "e0", "seq": 0,', false],
      ['    "event_type": "forged",  /* was "write" */', true],
      ['    "payload_commitment": "…64 hex…" } ],', false],
      ['"chain_hash": "…stale hash for the old events…"', true],
    ],
  },
];

function RejectCard({ r }: { r: Reject }) {
  const s = STAGE_INDEX[r.stage];
  return (
    <div className={`card ${styles.reject}`}>
      <div className={styles.rejectHead}>
        <span className={styles.rejectTitle}>{r.title}</span>
        <StageBadge stage={r.stage} />
      </div>
      <pre className="doc">
        {r.lines.map(([text, bad], i) => (
          <span key={i}>
            {bad ? <span className={styles.bad}>{text}</span> : text}
            {"\n"}
          </span>
        ))}
      </pre>
      <div className={styles.rejectWhy}>
        <span className="warn">rejected by {s.name}</span> &mdash; {r.why}
      </div>
    </div>
  );
}

const TEMPLATE = `{
  "format_version": "core/v1",
  "events": [
    {
      "id": "<unique-within-receipt>",
      "seq": 0,
      "event_type": "<non-empty kind, e.g. build>",
      "objects": [
        { "id": "<obj-id>", "obj_type": "<ocel-type>", "qualifier": null }
      ],
      "payload_commitment": "<64 lowercase hex chars = BLAKE3 of payload>"
    }
  ],
  "chain_hash": "<64 lowercase hex chars = rolling BLAKE3 over events>"
}`;

export default function AnatomyPage() {
  return (
    <>
      <h1>Receipt Anatomy</h1>
      <div className={styles.intro}>
        <p>
          This is the reference for affidavit&rsquo;s data model. A{" "}
          <strong className="green">Receipt</strong> is an append-only,
          content-addressed chain of operation-events; the verifier certifies it
          by running each field through a fixed sequence of decidable checks.
          Below: every field of every type, what it means, and{" "}
          <strong>which of the 7 certify stages enforces it</strong> &mdash; all
          accurate to the Rust types in <code>src/types.rs</code> and the stage
          logic in <code>src/verifier.rs</code>.
        </p>
        <p className="muted">
          The colored badges map each field to the stage that decides it. The
          same colors reappear in the{" "}
          <Link href="#rejections">rejection gallery</Link> below, so you can
          trace a field to the exact reason a malformed receipt fails.
        </p>
        <p className={styles.links}>
          <Link href="/learn">→ The doctrine &amp; the pipeline (Learn)</Link>
          <Link href="/pipeline">→ Run the certify pipeline</Link>
          <Link href="/studio">→ Build &amp; verify your own (Studio)</Link>
        </p>
      </div>

      {/* ---------------- STAGE LEGEND ---------------- */}
      <section>
        <h2>The 7 certify stages</h2>
        <p className="muted">
          A receipt flows through these in order. Stage 7 only emits ACCEPT if
          stages 1&ndash;6 all passed; otherwise the verdict is REJECT and{" "}
          <code>reason</code> names the first failure.
        </p>
        <ul className={styles.legend}>
          {STAGES.map((s) => (
            <li className={styles.legendItem} key={s.key}>
              <span className={styles.legendNum}>{s.n}</span>
              <span className={styles.legendBody}>
                <span className={`${styles.badge} ${s.cls}`}>{s.name}</span>
                <span className={styles.legendDesc}>{s.desc}</span>
              </span>
            </li>
          ))}
        </ul>
      </section>

      {/* ---------------- FIELD TABLES ---------------- */}
      <section>
        <h2>The types, field by field</h2>
        <FieldTable
          type="Receipt"
          fields={RECEIPT_FIELDS}
          jsonNote={
            <>
              The top-level witness. Built only via{" "}
              <code>ChainAssembler::finalize</code>; the private{" "}
              <code>_seal</code> field is not serialized.
            </>
          }
        />
        <FieldTable
          type="OperationEvent"
          fields={EVENT_FIELDS}
          jsonNote={
            <>
              One append-only link in the chain. All five public fields are part
              of the canonical bytes, so editing any of them changes the chain
              hash.
            </>
          }
        />
        <FieldTable
          type="ObjectRef"
          fields={OBJECTREF_FIELDS}
          jsonNote={
            <>
              A qualified reference from an event to an OCEL object. JSON keys
              are <code>id</code>, <code>obj_type</code>, <code>qualifier</code>.
            </>
          }
        />
        <FieldTable
          type="Verdict"
          fields={VERDICT_FIELDS}
          jsonNote={
            <>
              The verifier&rsquo;s output (not part of the receipt). It records
              what happened in each stage and the final ACCEPT / REJECT.
            </>
          }
        />
      </section>

      {/* ---------------- CANONICAL FORM ---------------- */}
      <section>
        <h2>Canonical form</h2>
        <div className="card">
          <p>
            Receipts hash and serialize as <strong>compact JSON</strong> with
            object keys sorted recursively. The same logical receipt therefore
            always produces identical bytes &mdash; which is what makes content
            addressing reproducible.
          </p>
          <ul className={styles.note} style={{ paddingLeft: "1.1rem" }}>
            <li>
              <span className="green">No wall-clock:</span> ordering comes from{" "}
              <code>seq</code>, never from time.
            </li>
            <li>
              <span className="green">No RNG, no map-iteration order:</span>{" "}
              sorted keys make the byte form deterministic.
            </li>
            <li>
              <span className="green">Commitments, never payloads:</span> only
              the BLAKE3 digest is stored, so the canonical bytes are small and
              stable.
            </li>
          </ul>
          <p className={styles.note}>
            This canonical encoding isn&rsquo;t a stage you can fail &mdash; it
            is the shared rule every hashing step relies on, which is why the{" "}
            <code>—</code> badge marks fields no single stage decides.
          </p>
        </div>
      </section>

      {/* ---------------- TEMPLATE ---------------- */}
      <section>
        <h2>Minimal valid shape</h2>
        <p className="muted">
          <span className={styles.tag}>template &mdash; fill in your own</span>{" "}
          This is a skeleton with placeholders, <strong>not</strong> a captured
          receipt: the <code>&lt;…&gt;</code> slots are not real digests, so as
          written it will <em>not</em> verify. Replace every placeholder, then
          confirm it in <Link href="/studio">Studio</Link> or run it through the{" "}
          <Link href="/pipeline">Pipeline</Link>.
        </p>
        <pre className="doc">{TEMPLATE}</pre>
        <p className={styles.note}>
          To get a genuinely valid receipt, don&rsquo;t hand-write the hashes:
          let the tool compute the per-event <code>payload_commitment</code> and
          the rolling <code>chain_hash</code> for you. Hand-written hashes will
          fail <StageBadge stage="chain_integrity" /> /{" "}
          <StageBadge stage="verify_commitments" />.
        </p>
      </section>

      {/* ---------------- REJECTION GALLERY ---------------- */}
      <section id="rejections">
        <h2>Rejection gallery</h2>
        <p className="muted">
          Each snippet below is a deliberately <span className="warn">malformed</span>{" "}
          receipt fragment &mdash; a teaching input, <strong>not</strong> tool
          output. The highlighted token is the defect; the caption names the
          real stage that rejects it and the detail message it produces.
        </p>
        <div className={styles.gallery}>
          {REJECTS.map((r) => (
            <RejectCard key={r.title} r={r} />
          ))}
        </div>
        <p className="src" style={{ marginTop: "1.5rem" }}>
          Want to see a genuine verdict over events you supply? Build one in{" "}
          <Link href="/studio">Studio</Link> and watch it move through the{" "}
          <Link href="/pipeline">Pipeline</Link>. The reasoning behind each
          stage lives in <Link href="/learn">Learn</Link>.
        </p>
      </section>
    </>
  );
}
