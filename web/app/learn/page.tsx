import Link from "next/link";
import Pipeline from "./Pipeline";
import styles from "./learn.module.css";

export const metadata = {
  title: "Learn — affidavit",
  description:
    "How affidavit works in two minutes: the doctrine, the 7-stage certify pipeline, and the limits it is honest about.",
};

const PRINCIPLES = [
  {
    title: "Certify, don't decide",
    body: (
      <>
        The verifier never decides whether work is <em>honest</em> — that is
        undecidable (Rice&rsquo;s theorem). It checks a witness (the receipt)
        against a fixed format standard, and every check is decidable.
      </>
    ),
  },
  {
    title: "A receipt is an append-only chain",
    body: (
      <>
        A receipt is an append-only BLAKE3 chain of operation-events. Each link
        folds the previous chain hash with the canonical bytes of the next
        event, so any edit to any event propagates through every later link.
      </>
    ),
  },
  {
    title: "Reject, don't detect",
    body: (
      <>
        Unverifiable work is <em>rejected</em>, not detected. A tampered or
        malformed receipt simply fails a stage and yields <code>REJECT</code>.
        The verifier proves a lawful chain exists; it does not hunt for fraud.
      </>
    ),
  },
  {
    title: "The bypass is unconstructable",
    body: (
      <>
        Receipt struct-literal construction fails at <strong>compile</strong>{" "}
        time (Rust error <code>E0451</code>: private field <code>_seal</code>).
        Only the canonical seam <code>ChainAssembler::finalize</code> can
        construct sealed receipts; deserialization re-verifies the chain, so
        forged JSON is rejected too.
      </>
    ),
  },
];

const DETERMINISM = [
  {
    key: "No wall-clock",
    body: "Events are ordered by a monotonic seq, never by time.",
  },
  {
    key: "No RNG, no map-iteration order",
    body: "Canonical sorted-key JSON makes the bytes reproducible.",
  },
  {
    key: "Pure over the receipt bytes",
    body: "The same receipt always yields the same verdict.",
  },
  {
    key: "Commitments, never payloads",
    body: "The verifier reads BLAKE3 commitments and never the raw payloads.",
  },
];

const RESIDUALS = [
  "Undecidability is relocated to the construction boundary, not solved.",
  "The verifier's structural laws are trusted — the root-of-trust is open.",
  "At least one witness (the verify vs. show distinction) is irreducibly human.",
];

export default function LearnPage() {
  return (
    <>
      <h1>Learn affidavit in two minutes</h1>
      <div className={styles.intro}>
        <p className={styles.lead}>
          affidavit doesn&rsquo;t judge whether your work is honest. It{" "}
          <strong className="green">certifies</strong> a receipt: it proves that
          a lawful, tamper-evident chain of operations exists, using checks that
          a machine can always answer. What can&rsquo;t be answered is{" "}
          <em>rejected</em>, not guessed at. Here&rsquo;s the whole idea.
        </p>
      </div>

      {/* ---------- DOCTRINE ---------- */}
      <section className={styles.section}>
        <div className={styles.sectionHead}>
          <h2>The doctrine</h2>
          <span className={styles.kicker}>four principles</span>
        </div>
        <p className={styles.sectionSub}>
          Four ideas separate &ldquo;is this honest?&rdquo; (undecidable) from
          &ldquo;is this a lawful receipt?&rdquo; (decidable).
        </p>
        <div className="grid">
          {PRINCIPLES.map((p, i) => (
            <div className={`card ${styles.principle}`} key={p.title}>
              <span className={styles.principleNum}>{i + 1}</span>
              <h3 className={styles.principleTitle}>{p.title}</h3>
              <p>{p.body}</p>
            </div>
          ))}
        </div>

        {/* How the append-only chain folds — the core mental model. */}
        <div className={`card ${styles.chain}`}>
          <div className={styles.chainCaption}>
            How a link is formed —{" "}
            <code>chainᵢ = BLAKE3(chainᵢ₋₁ ∥ canonical_bytes(eventᵢ))</code>
          </div>
          <div className={styles.chainRow}>
            <span className={styles.chainSeed} title="genesis: the empty/zero chain hash">
              chain₀
            </span>
            {[0, 1, 2].map((i) => (
              <span className={styles.chainLink} key={i}>
                <span className={styles.chainArrow} aria-hidden="true">
                  +
                </span>
                <span className={styles.chainEvent}>
                  event<sub>{i}</sub>
                  <em>seq {i}</em>
                </span>
                <span className={styles.chainArrow} aria-hidden="true">
                  →
                </span>
                <span className={styles.chainHash}>chain{["₁", "₂", "₃"][i]}</span>
              </span>
            ))}
          </div>
          <p className={styles.chainNote}>
            Edit <em>any</em> event&rsquo;s bytes and every later{" "}
            <code>chain</code> value changes — so{" "}
            <code>chain_integrity</code> (stage 3) catches it by recomputing the
            roll and comparing to the stored <code>chain_hash</code>.
          </p>
        </div>
      </section>

      {/* ---------- PIPELINE ---------- */}
      <section className={styles.section}>
        <div className={styles.sectionHead}>
          <h2>The certify pipeline</h2>
          <span className={styles.kicker}>seven decidable stages</span>
        </div>
        <p className={styles.sectionSub}>
          A receipt flows through seven stages in order. Each one asks a question
          a machine can always answer; the verdict is <code>ACCEPT</code> only if
          every stage passes. Click a stage to see its check and exactly what
          makes it fail.
        </p>
        <Pipeline />

        <div className={styles.cta}>
          <Link className={styles.ctaBtn} href="/studio">
            Try a receipt in the Studio →
          </Link>
          <Link className={`${styles.ctaBtn} ${styles.ctaBtnGhost}`} href="/visualizer">
            Watch the chain in the Visualizer
          </Link>
          <Link className={`${styles.ctaBtn} ${styles.ctaBtnGhost}`} href="/pipeline">
            Run the live pipeline
          </Link>
        </div>
      </section>

      {/* ---------- DETERMINISM ---------- */}
      <section className={styles.section}>
        <div className={styles.sectionHead}>
          <h2>Why it&rsquo;s deterministic</h2>
          <span className={styles.kicker}>same receipt ⇒ same verdict</span>
        </div>
        <p className={styles.sectionSub}>
          Determinism is what makes &ldquo;decidable&rdquo; meaningful: nothing
          in the verdict depends on when, where, or how many times you run it.
        </p>
        <div className="card">
          <ul className={styles.detList}>
            {DETERMINISM.map((d) => (
              <li key={d.key}>
                <span className={styles.detKey}>{d.key}.</span> {d.body}
              </li>
            ))}
          </ul>
        </div>
      </section>

      {/* ---------- RESIDUALS (honest limits) ---------- */}
      <section className={styles.section}>
        <div className={styles.sectionHead}>
          <h2 className="warn">Honest residuals</h2>
          <span className={styles.kicker}>what this does not solve</span>
        </div>
        <p className={styles.sectionSub}>
          affidavit is deliberate about its limits. Certification narrows the
          trusted surface — it does not eliminate it.
        </p>
        <div className={`card ${styles.residuals}`}>
          {RESIDUALS.map((r) => (
            <div className={styles.residualItem} key={r}>
              <span className={styles.residualMark} aria-hidden="true">
                ▲
              </span>
              <span>{r}</span>
            </div>
          ))}
        </div>
      </section>

      <p className="src" style={{ marginTop: "2rem" }}>
        This page explains the project&rsquo;s real model. To watch the genuine
        verifier run over events you supply, open the{" "}
        <Link href="/studio">Studio</Link> or the{" "}
        <Link href="/visualizer">Visualizer</Link>; the live{" "}
        <Link href="/pipeline">Pipeline</Link> runs the same stages.
      </p>
    </>
  );
}
