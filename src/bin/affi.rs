// `affi` — hand-written STATIC binary entry point.
//
// Zero ontology input, so NOT generated (generating it would fake a μ output).
// It calls the library's run function which has all the verbs registered.

fn main() -> clap_noun_verb::Result<()> {
    affidavit::run()
}
