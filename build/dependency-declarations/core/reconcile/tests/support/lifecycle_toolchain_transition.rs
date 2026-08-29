use super::lifecycle_support::{digest, profile};
use dependency_declarations_reconcile::*;

#[derive(Clone, Copy)]
struct MatrixFixtureV1<'a> {
    msrv_minor: u16,
    stable_minor: u16,
    beta_minor: u16,
    nightly_minor: u16,
    msrv_commit: &'a str,
    stable_commit: &'a str,
    beta_commit: &'a str,
    nightly_commit: &'a str,
    nightly_cargo_commit: &'a str,
}

impl Default for MatrixFixtureV1<'static> {
    fn default() -> Self {
        Self {
            msrv_minor: 98,
            stable_minor: 98,
            beta_minor: 99,
            nightly_minor: 100,
            msrv_commit: "88d9e12ae-msrv",
            stable_commit: "88d9e12ae",
            beta_commit: "f47d5bb13",
            nightly_commit: "c656540d6",
            nightly_cargo_commit: "cargo-c656540d6",
        }
    }
}

fn matrix(fixture: MatrixFixtureV1<'_>) -> ToolchainMatrixV1 {
    ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            RustVersionV1::try_new(1, fixture.msrv_minor, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            fixture.msrv_commit,
            "msrv-cargo",
        ),
        profile(
            ToolchainRoleV1::QualifiedStableExecution,
            RustVersionV1::try_new(1, fixture.stable_minor, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            fixture.stable_commit,
            "stable-cargo",
        ),
        profile(
            ToolchainRoleV1::BetaShadow,
            RustVersionV1::try_new(1, fixture.beta_minor, 0).unwrap(),
            LifecycleChannelV1::Beta,
            SourceMaturityV1::Provisional,
            fixture.beta_commit,
            "beta-cargo",
        ),
        profile(
            ToolchainRoleV1::NightlyShadow,
            RustVersionV1::try_new(1, fixture.nightly_minor, 0).unwrap(),
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            fixture.nightly_commit,
            fixture.nightly_cargo_commit,
        ),
    )
    .unwrap()
}

fn try_msrv_intent(
    current_minor: u16,
    proposed_minor: u16,
) -> Result<DeclaredMsrvChangeIntentV1, LifecycleFailureV1> {
    DeclaredMsrvChangeIntentV1::try_new(
        "build-product-owner",
        RustVersionV1::try_new(1, current_minor, 0).unwrap(),
        RustVersionV1::try_new(1, proposed_minor, 0).unwrap(),
        digest("semantic-intent"),
        digest("postconditions"),
    )
}

fn msrv_intent(current_minor: u16, proposed_minor: u16) -> DeclaredMsrvChangeIntentV1 {
    try_msrv_intent(current_minor, proposed_minor).unwrap()
}

#[test]
fn same_version_nightly_refresh_changes_only_the_shadow_role() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        nightly_commit: "bff8e12ff",
        nightly_cargo_commit: "e8cb624d5",
        ..MatrixFixtureV1::default()
    });
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("nightly-discovery"))
            .unwrap();

    assert_eq!(candidate.changed_roles(), &[ToolchainRoleV1::NightlyShadow]);
    assert_eq!(candidate.msrv_effect(), &ToolchainMsrvEffectV1::Unchanged);
    assert_eq!(
        candidate.current().msrv().identity_sha256(),
        candidate.proposed().msrv().identity_sha256()
    );
    assert_eq!(
        candidate.current().stable().identity_sha256(),
        candidate.proposed().stable().identity_sha256()
    );
    assert_ne!(
        candidate.current().nightly().identity_sha256(),
        candidate.proposed().nightly().identity_sha256()
    );
}

#[test]
fn same_floor_msrv_evidence_refresh_is_not_a_floor_change() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        msrv_commit: "88d9e12ae-refreshed-evidence",
        ..MatrixFixtureV1::default()
    });
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("msrv-evidence-discovery"))
            .unwrap();

    assert_eq!(
        candidate.changed_roles(),
        &[ToolchainRoleV1::DeclaredMsrvCompatibility]
    );
    assert_eq!(
        candidate.msrv_effect(),
        &ToolchainMsrvEffectV1::QualificationRefresh
    );
}

#[test]
fn execution_train_rotation_leaves_the_declared_floor_unchanged() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        stable_minor: 99,
        beta_minor: 100,
        nightly_minor: 101,
        stable_commit: "stable-1.99",
        beta_commit: "beta-1.100",
        nightly_commit: "nightly-1.101",
        nightly_cargo_commit: "cargo-nightly-1.101",
        ..MatrixFixtureV1::default()
    });
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("execution-train-discovery"))
            .unwrap();

    assert_eq!(
        candidate.changed_roles(),
        &[
            ToolchainRoleV1::QualifiedStableExecution,
            ToolchainRoleV1::BetaShadow,
            ToolchainRoleV1::NightlyShadow,
        ]
    );
    assert_eq!(candidate.msrv_effect(), &ToolchainMsrvEffectV1::Unchanged);
}

#[test]
fn numeric_msrv_change_requires_product_owned_intent() {
    let current = matrix(MatrixFixtureV1 {
        msrv_minor: 97,
        msrv_commit: "msrv-1.97",
        ..MatrixFixtureV1::default()
    });
    let proposed = matrix(MatrixFixtureV1::default());
    let failure = ToolchainCandidateV1::try_new(
        current.clone(),
        proposed.clone(),
        None,
        digest("msrv-discovery"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::MissingToolchainIntent
    );

    let intent = msrv_intent(97, 98);
    let candidate = ToolchainCandidateV1::try_new(
        current,
        proposed,
        Some(intent.clone()),
        digest("msrv-discovery"),
    )
    .unwrap();
    assert_eq!(
        candidate.changed_roles(),
        &[ToolchainRoleV1::DeclaredMsrvCompatibility]
    );
    assert_eq!(
        candidate.msrv_effect(),
        &ToolchainMsrvEffectV1::FloorChange { intent }
    );
}

#[test]
fn no_op_and_spurious_msrv_intent_refuse() {
    let baseline = matrix(MatrixFixtureV1::default());
    let failure =
        ToolchainCandidateV1::try_new(baseline.clone(), baseline.clone(), None, digest("no-op"))
            .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainCandidate
    );

    let intent = msrv_intent(97, 98);
    let failure = ToolchainCandidateV1::try_new(
        baseline.clone(),
        matrix(MatrixFixtureV1 {
            nightly_commit: "bff8e12ff",
            ..MatrixFixtureV1::default()
        }),
        Some(intent),
        digest("spurious-intent"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidToolchainCandidate
    );
}

#[test]
fn execution_train_regression_refuses() {
    let current = matrix(MatrixFixtureV1::default());
    let proposed = matrix(MatrixFixtureV1 {
        msrv_minor: 97,
        stable_minor: 97,
        msrv_commit: "msrv-1.97",
        stable_commit: "stable-1.97",
        ..MatrixFixtureV1::default()
    });
    let failure = ToolchainCandidateV1::try_new(
        current,
        proposed,
        Some(msrv_intent(98, 97)),
        digest("regression-discovery"),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnsupportedVersionRelation
    );
}

#[test]
fn msrv_intent_cannot_be_replayed_across_floor_changes() {
    let invalid = try_msrv_intent(98, 98).unwrap_err();
    assert_eq!(
        invalid.class(),
        LifecycleFailureClassV1::ToolchainIntentMismatch
    );
    let current = matrix(MatrixFixtureV1 {
        msrv_minor: 96,
        msrv_commit: "msrv-1.96",
        ..MatrixFixtureV1::default()
    });
    let proposed = matrix(MatrixFixtureV1::default());
    let intent = msrv_intent(97, 98);
    let failure = ToolchainCandidateV1::try_new(
        current,
        proposed,
        Some(intent),
        digest("replayed-msrv-intent"),
    )
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainIntentMismatch
    );
}
