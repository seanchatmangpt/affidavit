import PipelineForm from "./PipelineForm";

export const runtime = "nodejs";

export default function PipelinePage() {
  return (
    <>
      <h1>Pipeline</h1>
      <p className="muted">
        The real certify pipeline. This is not a mock: a server action shells the
        actual <code>affi</code> binary (emit → assemble → verify) over the events you
        supply and renders the genuine multi-stage <code>Verdict</code>. Try an honest
        run, then break it (e.g. an event with an empty <code>type</code>) and watch a
        stage fail.
      </p>
      <PipelineForm />
    </>
  );
}
