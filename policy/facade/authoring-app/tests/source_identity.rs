#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use support::*;

#[test]
fn source_identity_is_reexported_without_changing_qualification_identity() {
    let candidate = project();
    let reordered = serde_json::to_value(&candidate).unwrap();
    let decoded: policy_authoring_app::PolicyProject = serde_json::from_value(reordered).unwrap();
    assert_eq!(
        decoded.source.content_version().unwrap(),
        candidate.source.content_version().unwrap()
    );
    let first = candidate.prepare(ids()).unwrap();
    let mut renamed = project();
    renamed.cases[0].name = "renamed case".into();
    let second = renamed.prepare(ids()).unwrap();
    assert_eq!(first.bundle().version, second.bundle().version);
    assert_ne!(
        first.report().qualification_digest,
        second.report().qualification_digest
    );
}

#[test]
fn stable_fixture_source_and_qualification_identity_are_preserved() {
    let fixture: policy_authoring_app::PolicyProject =
        serde_json::from_slice(include_bytes!("fixtures/read-access.json")).unwrap();
    assert_eq!(
        fixture.source.content_version().unwrap().as_str(),
        "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2"
    );
    let prepared = fixture.prepare(ids()).unwrap();
    assert_eq!(
        prepared.report().qualification_digest,
        "sha256:c17c6d383103439f45a8d6133d3ca6901f729363bb6ee5da593e31d3aa2aef92"
    );
    assert_eq!(prepared.report().passed_cases, 2);
}
