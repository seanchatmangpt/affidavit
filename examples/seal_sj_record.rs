//! SEAL-RUNBOOK §3 harness — backlog [81] Phase-4 attestation closure.
//!
//! stdin: JSONL of draft wire objects (adapter output, one per ledger row).
//! For each line: SjCampaign::new -> finalize -> to_json, then an independent
//! re-verify via SjRecord::from_json(&json).verify().
//! Writes `<outdir>/<ORDER>.sj-record.json` and `<outdir>/CHAIN-HEAD.txt`
//! (one `ORDER chain-head` per line, campaign order preserved).
//!
//! Run:
//!   cargo run --features crypto-trust --example seal_sj_record -- <outdir> \
//!     < drafts.jsonl

use affidavit::sj_record::{
    Authority, AuthorityCeiling, BrokenTerm, CommitRecord, ReplayCommand, SjCampaign,
    SjCampaignDraft, SjRecord, StandingValue,
};
use serde::Deserialize;
use std::io::{self, BufRead};

/// Draft-shaped wire object (adapter contract). `SjCampaignDraft` itself is
/// not `Deserialize`; this is the mirror the adapter targets, mapped 1:1.
#[derive(Deserialize)]
struct DraftWire {
    work_order_id: String,
    origin_ceiling: Option<String>,
    origin_grant: String,
    origin_actor: String, // reserved for actor resolution; grant carries the IRI
    provider_execution_id: String,
    subject: String,
    repo: String,
    subject_sha: String,
    base_sha: String,
    commits: Vec<CommitWire>,
    residue_declaration: String,
    files_changed: Vec<String>,
    remote_effects: Vec<String>,
    replay: Vec<ReplayWire>,
    durable_location: Option<String>,
    standing: String,
    broken_term: Option<String>,
    derived_from: String,
    predecessors: Vec<String>,
}

#[derive(Deserialize)]
struct CommitWire {
    sha: String,
    summary: String,
    #[serde(default)]
    court_results: Vec<String>,
}

#[derive(Deserialize)]
struct ReplayWire {
    cmd: String,
    exit: i32,
    cwd: String,
    summary: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("seal_sj_record: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let outdir = args
        .next()
        .ok_or("usage: seal_sj_record <outdir> < drafts.jsonl")?;
    std::fs::create_dir_all(&outdir)?;

    let stdin = io::stdin();
    let mut chain_heads = String::new();
    let mut count = 0usize;
    for (index, line) in stdin.lock().lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let wire: DraftWire = serde_json::from_str(&line).map_err(|e| format!("line {}: {e}", index + 1))?;
        let order = wire.work_order_id.clone();
        let draft = draft_from_wire(&wire)
            .map_err(|e| format!("line {} ({}): {e}", index + 1, order))?;
        let record = SjCampaign::new(draft)
            .map_err(|e| format!("{}: new: {e:?}", order))?
            .finalize()
            .map_err(|e| format!("{}: finalize: {e:?}", order))?;
        let head = record.base.chain_hash.as_hex().to_string();
        let json = record.to_json()?;

        // Independent re-verify path: from_json re-runs the full verify() law.
        let roundtripped =
            SjRecord::from_json(&json).map_err(|e| format!("{}: from_json: {e:?}", order))?;
        roundtripped
            .verify()
            .map_err(|e| format!("{}: verify: {e:?}", order))?;

        let path = format!("{}/{}.sj-record.json", outdir, order);
        std::fs::write(&path, &json)?;
        chain_heads.push_str(&format!("{} {}\n", order, head));
        count += 1;
        println!("{} {} OK", order, head);
    }

    std::fs::write(format!("{outdir}/CHAIN-HEAD.txt"), &chain_heads)?;
    eprintln!("sealed {count} records");
    Ok(())
}

fn draft_from_wire(w: &DraftWire) -> Result<SjCampaignDraft, String> {
    let origin_ceiling = match w.origin_ceiling.as_deref() {
        None | Some("OBSERVE") => AuthorityCeiling::Observe,
        Some("SELECT") => AuthorityCeiling::Select,
        Some("CONSTRUCT") => AuthorityCeiling::Construct,
        Some("DO") => AuthorityCeiling::Do,
        Some(other) => return Err(format!("bad origin_ceiling: {other}")),
    };
    let refused = w.standing.starts_with("REFUSED");
    let authority_ceiling = if refused {
        AuthorityCeiling::Observe
    } else {
        origin_ceiling
    };
    Ok(SjCampaignDraft {
        work_order_id: w.work_order_id.clone(),
        origin_ceiling: Some(origin_ceiling),
        origin_grant: w.origin_grant.clone(),
        origin_actor: w.origin_actor.clone(),
        provider_name: "sjira.v26108.admitter".to_string(),
        provider_execution_id: w.provider_execution_id.clone(),
        subject: w.subject.clone(),
        repo: w.repo.clone(),
        subject_sha: w.subject_sha.clone(),
        base_sha: w.base_sha.clone(),
        commits: w
            .commits
            .iter()
            .map(|c| CommitRecord {
                sha: c.sha.clone(),
                summary: c.summary.clone(),
                court_results: c.court_results.clone(),
            })
            .collect(),
        residue_declaration: w.residue_declaration.clone(),
        files_changed: w.files_changed.clone(),
        remote_effects: w.remote_effects.clone(),
        replay_commands: w
            .replay
            .iter()
            .map(|r| ReplayCommand {
                cmd: r.cmd.clone(),
                exit: r.exit,
                cwd: r.cwd.clone(),
                summary: r.summary.clone(),
                output_sha256: None,
            })
            .collect(),
        durable_location: w.durable_location.clone(),
        standing: standing_from_wire(&w.standing, w.broken_term.as_deref())?,
        broken_term: match w.broken_term.as_deref() {
            None => None,
            Some(s) => Some(broken_term_from_wire(s)?),
        },
        derived_from: w.derived_from.clone(),
        predecessor_work_order_ids: w.predecessors.clone(),
        authority: Authority {
            ceiling: authority_ceiling,
            grant: w.origin_grant.clone(),
            actor: w.origin_actor.clone(),
        },
    })
}

fn standing_from_wire(s: &str, broken: Option<&str>) -> Result<StandingValue, String> {
    let value = match s {
        "UNKNOWN" => StandingValue::Unknown,
        "PARTIAL_ALIVE" => StandingValue::PartialAlive,
        "ALIVE" => StandingValue::Alive,
        "BUILD_BROKEN" => StandingValue::BuildBroken,
        s if s.starts_with("BLOCKED") => {
            StandingValue::Blocked(s.strip_prefix("BLOCKED:").map(str::to_string))
        }
        s if s.starts_with("REFUSED(") => {
            let inner = s
                .strip_prefix("REFUSED(")
                .and_then(|r| r.strip_suffix(')'))
                .ok_or_else(|| format!("bad standing: {s}"))?;
            StandingValue::Refused(inner.to_string())
        }
        other => return Err(format!("bad standing: {other}")),
    };
    if value.requires_broken_term() && broken.is_none() {
        return Err(format!("standing {s} requires broken_term"));
    }
    Ok(value)
}

fn broken_term_from_wire(s: &str) -> Result<BrokenTerm, String> {
    match s {
        "MuOnO" => Ok(BrokenTerm::MuOnO),
        "AdmissionVacuous" => Ok(BrokenTerm::AdmissionVacuous),
        "MuUnlawful" => Ok(BrokenTerm::MuUnlawful),
        "RMissingIdentity" => Ok(BrokenTerm::RMissingIdentity),
        "RMissingAuthority" => Ok(BrokenTerm::RMissingAuthority),
        "RMissingConsequence" => Ok(BrokenTerm::RMissingConsequence),
        "RMissingReplay" => Ok(BrokenTerm::RMissingReplay),
        "RMissingStanding" => Ok(BrokenTerm::RMissingStanding),
        "RNotFedBack" => Ok(BrokenTerm::RNotFedBack),
        other => Err(format!("bad broken_term: {other}")),
    }
}
