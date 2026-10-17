// Copyright (c) 2026 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `restitution calculate` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Computes liquidated avoided-cost distributions under Restatement of Restitution § 42
#[rustfmt::skip]
#[verb("calculate", "restitution")]
pub fn restitution_calculate(
    #[arg(help = "Cloud provider retail tariff schedule (azure|aws|gcp)")]
    tariff_schedule: Option<String>,
) -> Result<()> {
    crate::handlers::restitution_calculate(tariff_schedule)
}
