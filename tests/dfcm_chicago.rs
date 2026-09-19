use affidavit::dfcm::{
    certify_dfcm, v26_9_18_profile, ClosureGap, DfcmProfile, DfcmRefusal, EvidenceKey,
    EvidenceKind, EvidenceWitness, ExactSubject, Obligation, ProofPath, SubjectObservation,
    SubjectRequirement, V26_9_18Subjects,
};
use affidavit::{Blake3Hash, Standing};

fn exact(repository: &str, candidate: &str) -> ExactSubject {
    ExactSubject {
        repository: repository.to_string(),
        candidate: candidate.to_string(),
    }
}

fn evidence(kind: EvidenceKind, court: &str, seed: &str) -> EvidenceWitness {
    EvidenceWitness {
        key: EvidenceKey {
            kind,
            court: court.to_string(),
        },
        commitment: Blake3Hash::from_bytes(seed.as_bytes()),
    }
}

fn requirement(subject: ExactSubject, kind: EvidenceKind, court: &str) -> SubjectRequirement {
    SubjectRequirement {
        subject,
        required_evidence: vec![EvidenceKey {
            kind,
            court: court.to_string(),
        }],
        required_standing: None,
        require_standing_receipt: false,
    }
}

fn observation(
    subject: ExactSubject,
    standing: Standing,
    evidence: Vec<EvidenceWitness>,
) -> SubjectObservation {
    SubjectObservation {
        subject,
        standing,
        evidence,
        standing_receipt: None,
    }
}

fn single_obligation_profile(subject: ExactSubject) -> DfcmProfile {
    DfcmProfile {
        release: "test-release".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 1,
        obligations: vec![Obligation {
            id: "EXECUTION".to_string(),
            description: "exact-head execution must be independently witnessed".to_string(),
            paths: vec![ProofPath {
                name: "primary".to_string(),
                subjects: vec![requirement(
                    subject,
                    EvidenceKind::ExactHeadCourt,
                    "repo/exact-head",
                )],
            }],
        }],
    }
}

#[test]
fn source_merge_and_publication_do_not_imply_exact_head_execution() {
    let subject = exact("example/repo", "deadbeef");
    let receipt = certify_dfcm(
        single_obligation_profile(subject.clone()),
        vec![observation(
            subject,
            Standing::PartialAlive,
            vec![
                evidence(EvidenceKind::ImplementedSource, "repo/source", "source"),
                evidence(EvidenceKind::Merge, "repo/merge", "merge"),
                evidence(EvidenceKind::Publication, "repo/publication", "publication"),
            ],
        )],
    )
    .expect("well-formed fail-closed crown");

    assert!(!receipt.closed);
    assert!(receipt.minimal_frontiers.iter().any(|frontier| {
        frontier.gaps.iter().any(|gap| {
            matches!(
                gap,
                ClosureGap::MissingEvidence { evidence, .. }
                    if evidence.kind == EvidenceKind::ExactHeadCourt
            )
        })
    }));
}

#[test]
fn one_failed_path_does_not_destroy_a_qualified_alternative() {
    let broken = exact("example/broken", "broken-head");
    let alive = exact("example/alive", "alive-head");
    let court = "example/closure";
    let profile = DfcmProfile {
        release: "alternative-paths".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 2,
        obligations: vec![Obligation {
            id: "X".to_string(),
            description: "either admitted proof path may close X".to_string(),
            paths: vec![
                ProofPath {
                    name: "broken-path".to_string(),
                    subjects: vec![requirement(
                        broken.clone(),
                        EvidenceKind::ExactHeadCourt,
                        court,
                    )],
                },
                ProofPath {
                    name: "alive-path".to_string(),
                    subjects: vec![requirement(
                        alive.clone(),
                        EvidenceKind::ExactHeadCourt,
                        court,
                    )],
                },
            ],
        }],
    };

    let receipt = certify_dfcm(
        profile,
        vec![
            observation(broken, Standing::BuildBroken, vec![]),
            observation(
                alive,
                Standing::Alive,
                vec![evidence(EvidenceKind::ExactHeadCourt, court, "qualified")],
            ),
        ],
    )
    .expect("DfCM evaluates alternatives");

    assert!(receipt.closed);
    assert!(receipt.minimal_frontiers.is_empty());
    let paths = &receipt.obligations[0].paths;
    assert!(
        !paths
            .iter()
            .find(|p| p.name == "broken-path")
            .unwrap()
            .satisfied
    );
    assert!(
        paths
            .iter()
            .find(|p| p.name == "alive-path")
            .unwrap()
            .satisfied
    );
}

#[test]
fn unresolved_or_paths_are_separate_minimal_frontiers() {
    let profile = DfcmProfile {
        release: "minimal-frontiers".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 2,
        obligations: vec![Obligation {
            id: "X".to_string(),
            description: "two lawful alternatives remain open".to_string(),
            paths: vec![
                ProofPath {
                    name: "path-a".to_string(),
                    subjects: vec![requirement(
                        exact("example/a", "a-head"),
                        EvidenceKind::ExactHeadCourt,
                        "court/a",
                    )],
                },
                ProofPath {
                    name: "path-b".to_string(),
                    subjects: vec![requirement(
                        exact("example/b", "b-head"),
                        EvidenceKind::ExactHeadCourt,
                        "court/b",
                    )],
                },
            ],
        }],
    };

    let receipt = certify_dfcm(profile, vec![]).expect("bounded alternatives");
    assert!(!receipt.closed);
    assert_eq!(receipt.minimal_frontiers.len(), 2);
    assert!(receipt
        .minimal_frontiers
        .iter()
        .all(|frontier| frontier.gaps.len() == 1));
}

#[test]
fn exact_subject_substitution_fails_closed() {
    let expected = exact("example/repo", "expected-head");
    let moved = exact("example/repo", "different-head");
    let receipt = certify_dfcm(
        single_obligation_profile(expected),
        vec![observation(
            moved.clone(),
            Standing::Alive,
            vec![evidence(
                EvidenceKind::ExactHeadCourt,
                "repo/exact-head",
                "other-head",
            )],
        )],
    )
    .expect("subject movement is topology");

    assert!(!receipt.closed);
    assert!(receipt.minimal_frontiers[0].gaps.iter().any(|gap| {
        matches!(
            gap,
            ClosureGap::ExactSubjectMoved {
                repository,
                expected_candidate,
                observed_candidates,
            } if repository == "example/repo"
                && expected_candidate == "expected-head"
                && observed_candidates == &vec![moved.candidate.clone()]
        )
    }));
}

#[test]
fn empty_crown_is_refused_not_vacuously_closed() {
    let profile = DfcmProfile {
        release: "empty".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 1,
        obligations: vec![],
    };
    assert!(matches!(
        certify_dfcm(profile, vec![]),
        Err(DfcmRefusal::NoObligations)
    ));
}

#[test]
fn frontier_budget_refuses_instead_of_truncating_alternatives() {
    let profile = DfcmProfile {
        release: "bounded-combinatorics".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 1,
        obligations: vec![Obligation {
            id: "X".to_string(),
            description: "two lawful alternatives exceed the admitted budget".to_string(),
            paths: vec![
                ProofPath {
                    name: "a".to_string(),
                    subjects: vec![requirement(
                        exact("example/a", "a-head"),
                        EvidenceKind::ExactHeadCourt,
                        "court/a",
                    )],
                },
                ProofPath {
                    name: "b".to_string(),
                    subjects: vec![requirement(
                        exact("example/b", "b-head"),
                        EvidenceKind::ExactHeadCourt,
                        "court/b",
                    )],
                },
            ],
        }],
    };
    assert!(matches!(
        certify_dfcm(profile, vec![]),
        Err(DfcmRefusal::FrontierBudgetExceeded {
            budget: 1,
            required: 2
        })
    ));
}

#[test]
fn canonical_input_order_has_one_receipt_identity() {
    let a = exact("example/a", "a-head");
    let b = exact("example/b", "b-head");
    let court = "repo/exact-head";
    let mut profile = DfcmProfile {
        release: "canonical".to_string(),
        certifier: exact("seanchatmangpt/affidavit", "affidavit-head"),
        frontier_budget: 1,
        obligations: vec![
            Obligation {
                id: "B".to_string(),
                description: "B".to_string(),
                paths: vec![ProofPath {
                    name: "b".to_string(),
                    subjects: vec![requirement(b.clone(), EvidenceKind::ExactHeadCourt, court)],
                }],
            },
            Obligation {
                id: "A".to_string(),
                description: "A".to_string(),
                paths: vec![ProofPath {
                    name: "a".to_string(),
                    subjects: vec![requirement(a.clone(), EvidenceKind::ExactHeadCourt, court)],
                }],
            },
        ],
    };
    let mut observations = vec![
        observation(
            b,
            Standing::Alive,
            vec![evidence(EvidenceKind::ExactHeadCourt, court, "b")],
        ),
        observation(
            a,
            Standing::Alive,
            vec![evidence(EvidenceKind::ExactHeadCourt, court, "a")],
        ),
    ];

    let first = certify_dfcm(profile.clone(), observations.clone()).expect("first");
    profile.obligations.reverse();
    observations.reverse();
    let second = certify_dfcm(profile, observations).expect("second");
    assert_eq!(first.receipt_hash, second.receipt_hash);
}

#[test]
fn serialized_tamper_is_rejected_by_reverification() {
    let subject = exact("example/repo", "head");
    let receipt = certify_dfcm(
        single_obligation_profile(subject.clone()),
        vec![observation(
            subject,
            Standing::Alive,
            vec![evidence(
                EvidenceKind::ExactHeadCourt,
                "repo/exact-head",
                "qualified",
            )],
        )],
    )
    .expect("closed receipt");
    assert!(receipt.closed);

    let mut json = serde_json::to_value(&receipt).expect("json");
    json["closed"] = serde_json::Value::Bool(false);
    assert!(serde_json::from_value::<affidavit::dfcm::DfcmReceipt>(json).is_err());
}

#[test]
fn v26_9_18_requires_runtime_standing_but_not_merge_or_publication() {
    let subjects = V26_9_18Subjects {
        affidavit: exact("seanchatmangpt/affidavit", "affidavit-head"),
        ggen: exact("seanchatmangpt/ggen", "ggen-head"),
        bcinr: exact("seanchatmangpt/bcinr", "bcinr-head"),
        ash_r2rml: exact("seanchatmangpt/ash_r2rml", "r2rml-head"),
        ggen_igniter: exact("seanchatmangpt/ggen_igniter", "igniter-head"),
        wasm4pm: exact("seanchatmangpt/wasm4pm", "wasm-head"),
        unrdf: exact("seanchatmangpt/unrdf", "unrdf-head"),
    };
    let profile = v26_9_18_profile(subjects);
    let runtime = profile
        .obligations
        .iter()
        .find(|o| o.id == "BOUNDED_RUNTIME")
        .expect("runtime obligation");
    let required = &runtime.paths[0].subjects[0].required_evidence;
    assert!(required
        .iter()
        .any(|key| key.kind == EvidenceKind::RuntimeStanding));
    assert!(!required.iter().any(|key| key.kind == EvidenceKind::Merge));
    assert!(!required
        .iter()
        .any(|key| key.kind == EvidenceKind::Publication));
}
