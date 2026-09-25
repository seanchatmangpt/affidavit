//! BRCE ledger + zero-unreceipted-actuation court (AC-08 / F-05).
//!
//! Chicago style: real JSON-lines ledgers on disk, a real filesystem actuator
//! writing real files, real reloads after a simulated crash (the pipeline value is
//! dropped mid-DO), and state assertions on court verdicts, receipts and files.

use std::collections::BTreeMap;
use std::path::Path;

use affidavit::brce::{
    court, mutant_suite, replay_digest, Actuator, Admission, AuthorityGrant, BrceError, BrceLedger,
    BrcePipeline, ConstructedAction, FileActuator, Observer, ReconciliationVerdict, Request,
    RouteDecision, Rule, PROFILE_ACTUATION, PROFILE_RECONCILIATION,
};

const SUBJECT: &str = "cc1577a8de1e4e92625386ebbecb0c6df9abd66a";

fn request(id: &str, target: &str, content: &str) -> Request {
    let mut parameters = BTreeMap::new();
    parameters.insert("content".to_string(), content.to_string());
    Request {
        request_id: id.to_string(),
        subject: SUBJECT.to_string(),
        operation: "write".to_string(),
        target: target.to_string(),
        parameters,
        requester: "test".to_string(),
    }
}

fn route() -> RouteDecision {
    RouteDecision {
        capability: "fs.write".to_string(),
        executor_class: "FileActuator".to_string(),
    }
}

fn admit_all(_: &Request, _: &RouteDecision) -> Admission {
    Admission::Admitted
}

fn grant_for(action: &ConstructedAction, expires_at: u64, uses: u32) -> AuthorityGrant {
    AuthorityGrant {
        grant_id: format!("grant-{}", action.consequence_id),
        issuer: "test-broker".to_string(),
        subject: action.subject.clone(),
        operation: action.operation.clone(),
        target: action.target.clone(),
        construct_digest: action.construct_digest(),
        expires_at,
        maximum_uses: uses,
    }
}

fn pipeline(dir: &Path) -> BrcePipeline {
    BrcePipeline::new(
        BrceLedger::open(dir.join("ledger.jsonl")).unwrap(),
        "run-test",
    )
}

/// Lawful run: one receipted write. Returns (pipeline, actuator).
fn lawful(dir: &Path) -> (BrcePipeline, FileActuator) {
    let mut p = pipeline(dir);
    let mut act = FileActuator::new(dir.join("world")).unwrap();
    let adm = p
        .admit(request("r1", "c-alpha", "alpha"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-alpha", false).unwrap();
    let grant = grant_for(&action, 100, 1);
    p.actuate(&adm, &action, &grant, "a1", &mut act, 10)
        .unwrap();
    (p, act)
}

#[test]
fn lawful_actuation_is_receipted_and_admitted() {
    let dir = tempfile::tempdir().unwrap();
    let (p, act) = lawful(dir.path());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("world/c-alpha")).unwrap(),
        "alpha"
    );
    let v = court(&p.ledger, &act);
    assert!(v.admitted(), "refusals: {:?}", v.refusals);
    assert_eq!(v.standing, "ADMITTED");
    assert_eq!(v.receipts, 1);
    assert_eq!(v.consequences_observed, 1);
    let r = p
        .ledger
        .entries()
        .into_iter()
        .find_map(|e| match e {
            affidavit::brce::Entry::Receipted { receipt } => Some(receipt),
            _ => None,
        })
        .unwrap();
    assert_eq!(r.profile, PROFILE_ACTUATION);
    assert_eq!(r.subject, SUBJECT);
    assert_eq!(r.receipt_digest, r.compute_digest());
    assert!(r.effect.executed && r.effect.changed);
    assert_eq!(r.verification.verdict, "EFFECT_VERIFIED");
}

#[test]
fn do_without_valid_authority_is_refused_with_zero_consequence() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = pipeline(dir.path());
    let mut act = FileActuator::new(dir.path().join("world")).unwrap();
    let adm = p
        .admit(request("r1", "c-beta", "beta"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-beta", false).unwrap();

    let expired = grant_for(&action, 5, 1);
    let e = p
        .actuate(&adm, &action, &expired, "a1", &mut act, 10)
        .unwrap_err();
    assert!(matches!(e, BrceError::Refused(ref r) if r.starts_with("GRANT_EXPIRED")));

    let mut wrong_target = grant_for(&action, 100, 1);
    wrong_target.target = "c-elsewhere".to_string();
    let e = p
        .actuate(&adm, &action, &wrong_target, "a1", &mut act, 10)
        .unwrap_err();
    assert!(matches!(e, BrceError::Refused(ref r) if r.starts_with("TARGET_MISMATCH")));

    let mut substituted = grant_for(&action, 100, 1);
    substituted.construct_digest = "00".repeat(32);
    let e = p
        .actuate(&adm, &action, &substituted, "a1", &mut act, 10)
        .unwrap_err();
    assert!(matches!(e, BrceError::Refused(ref r) if r == "CONSTRUCT_DIGEST_MISMATCH"));

    let mut other_subject = grant_for(&action, 100, 1);
    other_subject.subject = "f".repeat(40);
    let e = p
        .actuate(&adm, &action, &other_subject, "a1", &mut act, 10)
        .unwrap_err();
    assert!(matches!(e, BrceError::Refused(ref r) if r.starts_with("SUBJECT_MISMATCH")));

    assert!(act.consequences().is_empty(), "no file may exist");
    let v = court(&p.ledger, &act);
    assert!(
        v.admitted(),
        "refusals are lawful terminal states: {:?}",
        v.refusals
    );
}

#[test]
fn unadmitted_work_has_distinct_terminal_states() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = pipeline(dir.path());
    let refused = p.admit(request("r1", "t", "x"), route(), |_, _| {
        Admission::Refused("policy".into())
    });
    let blocked = p.admit(request("r2", "t", "x"), route(), |_, _| {
        Admission::Blocked("fact".into())
    });
    let unsupported = p.admit(request("r3", "t", "x"), route(), |_, _| {
        Admission::Unsupported("cap".into())
    });
    assert!(matches!(refused, Err(BrceError::Refused(ref r)) if r == "policy"));
    assert!(matches!(blocked, Err(BrceError::Blocked(ref r)) if r == "fact"));
    assert!(matches!(unsupported, Err(BrceError::Refused(ref r)) if r == "UNSUPPORTED:cap"));
}

#[test]
fn single_use_grant_and_non_idempotent_retry_are_at_most_once() {
    let dir = tempfile::tempdir().unwrap();
    let (mut p, mut act) = lawful(dir.path());
    let adm = p
        .admit(request("r1", "c-alpha", "alpha"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-alpha", false).unwrap();
    let grant = grant_for(&action, 100, 1);
    let e = p
        .actuate(&adm, &action, &grant, "a2", &mut act, 20)
        .unwrap_err();
    assert!(matches!(e, BrceError::Refused(ref r) if r.starts_with("AT_MOST_ONCE")));
    assert!(court(&p.ledger, &act).admitted());
}

#[test]
fn idempotent_retry_executes_without_change() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = pipeline(dir.path());
    let mut act = FileActuator::new(dir.path().join("world")).unwrap();
    let adm = p
        .admit(request("r1", "c-idem", "same"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-idem", true).unwrap();
    let grant = grant_for(&action, 100, 1);
    let first = p
        .actuate(&adm, &action, &grant, "a1", &mut act, 10)
        .unwrap();
    let second = p
        .actuate(&adm, &action, &grant, "a2", &mut act, 20)
        .unwrap();
    assert!(first.effect.changed);
    assert!(second.effect.executed && !second.effect.changed);
    let v = court(&p.ledger, &act);
    assert!(v.admitted(), "{:?}", v.refusals);
}

#[test]
fn crash_after_do_before_receipt_reconciles_to_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join("ledger.jsonl");
    {
        let mut p = pipeline(dir.path());
        let mut act = FileActuator::new(dir.path().join("world")).unwrap();
        let adm = p
            .admit(request("r1", "c-crash", "gamma"), route(), admit_all)
            .unwrap();
        let action = p.construct(&adm, "c-crash", false).unwrap();
        let grant = grant_for(&action, 100, 1);
        p.prepare(&action, &grant, "a1", 10).unwrap();
        p.execute(&action, "a1", &mut act, 11).unwrap();
        // crash: pipeline dropped before receipt()
    }
    let act = FileActuator::new(dir.path().join("world")).unwrap();
    let mut p = BrcePipeline::new(BrceLedger::open(&ledger_path).unwrap(), "run-recover");
    let before = court(&p.ledger, &act);
    let refused = before.refused_rules();
    assert!(refused.contains(&Rule::ZeroUnreceiptedActuation));
    assert!(refused.contains(&Rule::CrashWindowReconciled));

    // A non-idempotent retry is blocked until reconciliation.
    let adm = p
        .admit(request("r1", "c-crash", "gamma"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-crash", false).unwrap();
    let grant = grant_for(&action, 100, 1);
    let mut act2 = FileActuator::new(dir.path().join("world")).unwrap();
    let e = p
        .actuate(&adm, &action, &grant, "a2", &mut act2, 30)
        .unwrap_err();
    assert!(matches!(e, BrceError::Blocked(ref r) if r.starts_with("RECONCILE_BEFORE_RETRY")));

    let verdicts = p.reconcile(&act).unwrap();
    assert_eq!(
        verdicts,
        vec![(
            "c-crash".to_string(),
            ReconciliationVerdict::EffectConfirmed
        )]
    );
    let after = court(&p.ledger, &act);
    assert!(after.admitted(), "{:?}", after.refusals);
    let reloaded = BrceLedger::open(&ledger_path).unwrap();
    assert!(reloaded.entries().iter().any(|e| matches!(e,
        affidavit::brce::Entry::Receipted { receipt } if receipt.profile == PROFILE_RECONCILIATION)));
}

#[test]
fn crash_between_actuator_and_done_record_reconciles_from_construct() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut p = pipeline(dir.path());
        let mut act = FileActuator::new(dir.path().join("world")).unwrap();
        let adm = p
            .admit(request("r1", "c-gap", "delta"), route(), admit_all)
            .unwrap();
        let action = p.construct(&adm, "c-gap", false).unwrap();
        let grant = grant_for(&action, 100, 1);
        p.prepare(&action, &grant, "a1", 10).unwrap();
        act.execute(&action).unwrap(); // effect lands, Done never written
    }
    let act = FileActuator::new(dir.path().join("world")).unwrap();
    let mut p = BrcePipeline::new(
        BrceLedger::open(dir.path().join("ledger.jsonl")).unwrap(),
        "run-recover",
    );
    assert!(!court(&p.ledger, &act).admitted());
    assert_eq!(
        p.reconcile(&act).unwrap(),
        vec![("c-gap".to_string(), ReconciliationVerdict::EffectConfirmed)]
    );
    assert!(court(&p.ledger, &act).admitted());
}

#[test]
fn crash_before_consequence_reconciles_to_no_effect() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = pipeline(dir.path());
    let act = FileActuator::new(dir.path().join("world")).unwrap();
    let adm = p
        .admit(request("r1", "c-none", "eps"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-none", false).unwrap();
    let grant = grant_for(&action, 100, 1);
    p.prepare(&action, &grant, "a1", 10).unwrap();
    assert!(!court(&p.ledger, &act).admitted());
    assert_eq!(
        p.reconcile(&act).unwrap(),
        vec![(
            "c-none".to_string(),
            ReconciliationVerdict::NoEffectConfirmed
        )]
    );
    let v = court(&p.ledger, &act);
    assert!(v.admitted(), "{:?}", v.refusals);
    assert_eq!(v.receipts, 0);
    assert!(act.consequences().is_empty());
}

#[test]
fn unobservable_crash_window_stays_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut p = pipeline(dir.path());
    let mut act = FileActuator::new(dir.path().join("world")).unwrap();
    let adm = p
        .admit(request("r1", "c-unk", "zeta"), route(), admit_all)
        .unwrap();
    let action = p.construct(&adm, "c-unk", false).unwrap();
    let grant = grant_for(&action, 100, 1);
    p.prepare(&action, &grant, "a1", 10).unwrap();
    p.execute(&action, "a1", &mut act, 11).unwrap();
    // The effect is replaced by a directory the observer cannot read as a file.
    std::fs::remove_file(dir.path().join("world/c-unk")).unwrap();
    std::fs::create_dir(dir.path().join("world/c-unk")).unwrap();
    assert_eq!(
        p.reconcile(&act).unwrap(),
        vec![("c-unk".to_string(), ReconciliationVerdict::ExecutionUnknown)]
    );
    let v = court(&p.ledger, &act);
    assert_eq!(v.standing, "REFUSED");
    assert!(v.refused_rules().contains(&Rule::CrashWindowReconciled));
    assert!(v.refused_rules().contains(&Rule::ZeroUnreceiptedActuation));
}

#[test]
fn replay_is_consequence_free_and_digest_equal() {
    let d1 = tempfile::tempdir().unwrap();
    let d2 = tempfile::tempdir().unwrap();
    let (p1, a1) = lawful(d1.path());
    let (p2, a2) = lawful(d2.path());
    let v1 = court(&p1.ledger, &a1);
    let v2 = court(&p2.ledger, &a2);
    assert_eq!(v1.replay_digest, v2.replay_digest);
    assert_eq!(v1.ledger_head, v2.ledger_head);

    let files_before = a1.consequences();
    let reloaded = BrceLedger::open(d1.path().join("ledger.jsonl")).unwrap();
    assert_eq!(replay_digest(&reloaded), v1.replay_digest);
    assert_eq!(court(&reloaded, &a1), v1);
    assert_eq!(
        a1.consequences(),
        files_before,
        "replay created no consequence"
    );

    let mut entries = reloaded.entries();
    entries.pop();
    assert_ne!(
        replay_digest(&BrceLedger::from_entries(entries)),
        v1.replay_digest
    );
}

#[test]
fn every_rule_has_a_killed_refusing_mutant() {
    let dir = tempfile::tempdir().unwrap();
    let (p, act) = lawful(dir.path());
    assert!(court(&p.ledger, &act).admitted());
    let outcomes = mutant_suite(&p.ledger, &act);
    let rules: Vec<Rule> = outcomes.iter().map(|o| o.rule).collect();
    assert_eq!(rules, Rule::ALL.to_vec());
    for o in &outcomes {
        assert_eq!(o.standing, "REFUSED", "{o:?}");
        assert!(o.killed, "mutant survived: {o:?}");
    }
}

#[test]
fn persisted_ledger_tamper_is_refused_after_reload() {
    let dir = tempfile::tempdir().unwrap();
    let (_p, act) = lawful(dir.path());
    let path = dir.path().join("ledger.jsonl");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replacen("\"alpha\"", "\"omega\"", 1)).unwrap();
    let v = court(&BrceLedger::open(&path).unwrap(), &act);
    assert!(v.refused_rules().contains(&Rule::ChainIntegrity));
}
