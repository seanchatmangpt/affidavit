// `affi` — hand-written STATIC binary entry point.
//
// Zero ontology input, so NOT generated (generating it would fake a μ output).
// It pulls in the ggen-rendered verb wrappers (`mod verbs`, which self-register
// via the `#[verb]` macro's `linkme` distributed slice) and the hand-written
// handlers behind the delegation seam (`mod handlers`), then hands control to the
// pack's dispatcher. Declaring both modules HERE (in the binary crate) is what
// forces the `linkme` registrations into the final executable.

// `#[path]` points at the crate-root modules (`src/handlers.rs`, `src/verbs/`);
// ggen renders the wrappers to `src/verbs/`, and the seam impl lives in
// `src/handlers.rs`, so the binary adopts both without copying.
#[path = "../handlers.rs"]
mod handlers;
#[path = "../verbs/mod.rs"]
mod verbs;

fn main() -> clap_noun_verb::Result<()> {
    // NOTE: the `run_with_default_format(Quiet)` null-suppression hook was backed
    // out upstream (clap-noun-verb e9d061c) as an undirected API expansion. Until a
    // *directed* suppression mechanism exists, affidavit uses the framework as-is:
    // verbs print their own human output via cli.rs and the framework appends its
    // default-format serialization of the unit return (a trailing "null"). The
    // null is a known OPEN residual, not a claimed guarantee.
    clap_noun_verb::run()
}
