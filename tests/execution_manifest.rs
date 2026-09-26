use affidavit::execution_manifest::{
    requalification_reason, verify_binding, ExecutionManifest, ManifestRefusal,
};
use std::collections::BTreeMap;

fn manifest() -> ExecutionManifest {
    ExecutionManifest {
        exact_subject: "git:repo@abc".into(),
        subject_digest: "subject-digest".into(),
        repository: Some("owner/repo".into()),
        base_sha: Some("abc".into()),
        ontology_digest: Some("ontology".into()),
        policy_digest: Some("policy".into()),
        capability_manifest_digest: Some("caps".into()),
        tool_manifest_digest: Some("tools".into()),
        planner_identity: Some("planner".into()),
        planner_version: Some("1".into()),
        generator_identity: Some("ggen".into()),
        generator_version: Some("1".into()),
        runtime_identity: Some("beam".into()),
        runtime_version: Some("29".into()),
        dependency_lock_digest: Some("lock".into()),
        authority_grant_digest: Some("grant".into()),
        intent_digest: "intent".into(),
        environment_constraints: BTreeMap::from([("arch".into(), "x86_64".into())]),
    }
}

#[test]
fn stable_manifest_has_stable_digest_and_binding() {
    let m = manifest();
    assert_eq!(m.digest().unwrap(), m.clone().digest().unwrap());
    verify_binding(&m, &m.binding().unwrap()).unwrap();
}

#[test]
fn mutated_subject_is_rejected() {
    let m = manifest();
    let binding = m.binding().unwrap();
    let mut changed = m.clone();
    changed.subject_digest = "other".into();
    assert_eq!(
        verify_binding(&changed, &binding),
        Err(ManifestRefusal::ManifestDigestMismatch)
    );
    assert_eq!(
        requalification_reason(&m, &changed).unwrap(),
        Some(ManifestRefusal::SubjectIdentityChanged)
    );
}

#[test]
fn authority_policy_ontology_and_tool_drift_are_typed() {
    let base = manifest();

    let mut changed = base.clone();
    changed.authority_grant_digest = Some("other".into());
    assert_eq!(
        requalification_reason(&base, &changed).unwrap(),
        Some(ManifestRefusal::AuthorityBindingChanged)
    );

    let mut changed = base.clone();
    changed.policy_digest = Some("other".into());
    assert_eq!(
        requalification_reason(&base, &changed).unwrap(),
        Some(ManifestRefusal::PolicyBindingChanged)
    );

    let mut changed = base.clone();
    changed.ontology_digest = Some("other".into());
    assert_eq!(
        requalification_reason(&base, &changed).unwrap(),
        Some(ManifestRefusal::OntologyBindingChanged)
    );

    let mut changed = base.clone();
    changed.tool_manifest_digest = Some("other".into());
    assert_eq!(
        requalification_reason(&base, &changed).unwrap(),
        Some(ManifestRefusal::ToolSurfaceChanged)
    );
}

#[test]
fn empty_exact_subject_is_invalid() {
    let mut m = manifest();
    m.exact_subject.clear();
    assert_eq!(
        m.digest(),
        Err(ManifestRefusal::InvalidManifest("exact_subject"))
    );
}
