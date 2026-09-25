//! BRCE zero-unreceipted-actuation court run (AC-08 / F-05).
//!
//! Usage: `cargo run --example brce_court -- <work_dir> <subject_sha>`
//!
//! Executes a real BRCE scenario twice (two independent directories, JSON-lines
//! ledgers, real files as external consequences): a lawful receipted write, an
//! idempotent retry, an unauthorized (expired-grant) DO, a crash after DO before the
//! receipt followed by reconciliation, and a crash before any consequence. Each
//! ledger is reloaded from disk and judged by the court; replay digests of the two
//! runs are compared, the consequence count is checked unchanged by replay, and the
//! mutant suite runs one refusing mutant per rule. Prints one JSON document; exits 0
//! iff the court admits both runs, replay digests are equal, replay created no
//! consequence, and every mutant is killed.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use affidavit::brce::{
    court, mutant_suite, replay_digest, Admission, AuthorityGrant, BrceError, BrceLedger,
    BrcePipeline, ConstructedAction, CourtVerdict, FileActuator, Observer, Request, RouteDecision,
};

fn request(subject: &str, id: &str, target: &str, content: &str) -> Request {
    let mut parameters = BTreeMap::new();
    parameters.insert("content".to_string(), content.to_string());
    Request {
        request_id: id.to_string(),
        subject: subject.to_string(),
        operation: "write".to_string(),
        target: target.to_string(),
        parameters,
        requester: "brce_court".to_string(),
    }
}

fn grant(a: &ConstructedAction, expires_at: u64) -> AuthorityGrant {
    AuthorityGrant {
        grant_id: format!("grant-{}", a.consequence_id),
        issuer: "brce_court-broker".to_string(),
        subject: a.subject.clone(),
        operation: a.operation.clone(),
        target: a.target.clone(),
        construct_digest: a.construct_digest(),
        expires_at,
        maximum_uses: 1,
    }
}

fn scenario(dir: &Path, subject: &str) -> Result<Vec<String>, BrceError> {
    let route = RouteDecision {
        capability: "fs.write".to_string(),
        executor_class: "FileActuator".to_string(),
    };
    let admit = |_: &Request, _: &RouteDecision| Admission::Admitted;
    let ledger_path = dir.join("ledger.jsonl");
    let mut log = Vec::new();
    {
        let mut p = BrcePipeline::new(BrceLedger::open(&ledger_path)?, "brce-court-v26.9.25");
        let mut act = FileActuator::new(dir.join("world"))?;

        let adm = p.admit(
            request(subject, "r1", "c-lawful", "lawful"),
            route.clone(),
            admit,
        )?;
        let a = p.construct(&adm, "c-lawful", false)?;
        p.actuate(&adm, &a, &grant(&a, 100), "a1", &mut act, 10)?;
        log.push("c-lawful: receipted".to_string());

        let adm = p.admit(
            request(subject, "r2", "c-idem", "idem"),
            route.clone(),
            admit,
        )?;
        let a = p.construct(&adm, "c-idem", true)?;
        let g = grant(&a, 100);
        p.actuate(&adm, &a, &g, "a1", &mut act, 20)?;
        let r = p.actuate(&adm, &a, &g, "a2", &mut act, 22)?;
        log.push(format!(
            "c-idem: retry executed={} changed={}",
            r.effect.executed, r.effect.changed
        ));

        let adm = p.admit(
            request(subject, "r3", "c-expired", "x"),
            route.clone(),
            admit,
        )?;
        let a = p.construct(&adm, "c-expired", false)?;
        match p.actuate(&adm, &a, &grant(&a, 5), "a1", &mut act, 30) {
            Err(e) => log.push(format!("c-expired: {e}")),
            Ok(_) => log.push("c-expired: UNEXPECTED ACTUATION".to_string()),
        }

        let adm = p.admit(
            request(subject, "r4", "c-crash", "crash"),
            route.clone(),
            admit,
        )?;
        let a = p.construct(&adm, "c-crash", false)?;
        p.prepare(&a, &grant(&a, 100), "a1", 40)?;
        p.execute(&a, "a1", &mut act, 41)?;

        let adm = p.admit(request(subject, "r5", "c-precrash", "never"), route, admit)?;
        let a = p.construct(&adm, "c-precrash", false)?;
        p.prepare(&a, &grant(&a, 100), "a1", 50)?;
        log.push("crash: pipeline dropped after c-crash DO and c-precrash PREPARED".to_string());
    }
    let act = FileActuator::new(dir.join("world"))?;
    let mut p = BrcePipeline::new(BrceLedger::open(&ledger_path)?, "brce-court-v26.9.25");
    let pre = court(&p.ledger, &act);
    log.push(format!(
        "pre-reconcile court: {} {:?}",
        pre.standing,
        pre.refused_rules()
    ));
    for (cid, v) in p.reconcile(&act)? {
        log.push(format!("reconcile {cid}: {v:?}"));
    }
    Ok(log)
}

fn judge(dir: &Path) -> Result<(CourtVerdict, usize), BrceError> {
    let act = FileActuator::new(dir.join("world"))?;
    let ledger = BrceLedger::open(dir.join("ledger.jsonl"))?;
    Ok((court(&ledger, &act), act.consequences().len()))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (Some(work), Some(subject)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: brce_court <work_dir> <subject_sha>");
        return ExitCode::from(64);
    };
    let run = || -> Result<serde_json::Value, BrceError> {
        let work = Path::new(work);
        let (da, db) = (work.join("run-a"), work.join("run-b"));
        for d in [&da, &db] {
            if d.exists() {
                std::fs::remove_dir_all(d)?;
            }
            std::fs::create_dir_all(d)?;
        }
        let log_a = scenario(&da, subject)?;
        let log_b = scenario(&db, subject)?;
        let (va, na) = judge(&da)?;
        let (vb, _) = judge(&db)?;
        // Replay from disk a second time: consequence-free and digest-stable.
        let (va2, na2) = judge(&da)?;
        let ledger_a = BrceLedger::open(da.join("ledger.jsonl"))?;
        let act_a = FileActuator::new(da.join("world"))?;
        let mutants = mutant_suite(&ledger_a, &act_a);
        let ledger_bytes = std::fs::read(da.join("ledger.jsonl"))?;
        let ok = va.admitted()
            && vb.admitted()
            && va.replay_digest == vb.replay_digest
            && va.replay_digest == va2.replay_digest
            && va.replay_digest == replay_digest(&ledger_a)
            && na == na2
            && !mutants.is_empty()
            && mutants.iter().all(|m| m.killed);
        Ok(serde_json::json!({
            "ok": ok,
            "subject_sha": subject,
            "scenario_log": log_a,
            "scenario_log_equal_across_runs": log_a == log_b,
            "court": va,
            "court_run_b": vb,
            "replay": {
                "digest_run_a": va.replay_digest,
                "digest_run_b": vb.replay_digest,
                "digest_run_a_rejudged": va2.replay_digest,
                "digest_equal": va.replay_digest == vb.replay_digest && va.replay_digest == va2.replay_digest,
                "consequences_before_replay": na,
                "consequences_after_replay": na2,
                "consequence_free": na == na2,
            },
            "ledger_blake3": blake3::hash(&ledger_bytes).to_hex().to_string(),
            "mutants": mutants,
        }))
    };
    match run() {
        Ok(v) => {
            let ok = v["ok"].as_bool().unwrap_or(false);
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("brce_court: {e}");
            ExitCode::from(2)
        }
    }
}
