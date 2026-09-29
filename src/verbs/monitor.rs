// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper rendered from O* by ggen. The pack is authoritative for the
// CLI *interface* only; the body delegates to a stable consumer-implemented
// handler. There is NO logic slot here — business logic lives behind the seam in
// `crate::handlers::*`, which is hand-written (a missing impl is a compile error).
//
// Consumed query columns (verb-signatures.rq): noun_name, verb_name, verb_about,
// return_type, handler_name, args.

//! `receipt monitor` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Monitor code quality in real-time using Western Electric control-chart rules
#[rustfmt::skip]
#[verb("monitor", "receipt")]
pub fn monitor(
    watch: Option<String>,
    metrics: Option<String>,
    rules: Option<String>,
    baseline_commits: Option<u32>,
    interval: Option<u64>,
    output: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::monitor(
        watch,
        metrics,
        rules,
        baseline_commits,
        interval,
        output,
        format,
    )
}
