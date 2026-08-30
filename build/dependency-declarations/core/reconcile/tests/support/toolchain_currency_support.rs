use super::lifecycle_support::{digest, profile, profile_with_qualification};
use dependency_declarations_reconcile::*;

#[derive(Clone, Copy)]
pub(super) struct MatrixFixtureV1<'a> {
    pub(super) msrv_minor: u16,
    pub(super) stable_minor: u16,
    pub(super) beta_minor: u16,
    pub(super) nightly_minor: u16,
    pub(super) msrv_commit: &'a str,
    pub(super) stable_commit: &'a str,
    pub(super) beta_commit: &'a str,
    pub(super) nightly_commit: &'a str,
    pub(super) nightly_cargo_commit: &'a str,
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

pub(super) fn matrix(fixture: MatrixFixtureV1<'_>) -> ToolchainMatrixV1 {
    matrix_with_stable_qualification(
        fixture,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest(&format!("{}-stable", fixture.stable_commit)),
        },
    )
}

pub(super) fn matrix_with_stable_qualification(
    fixture: MatrixFixtureV1<'_>,
    qualification: ToolchainQualificationV1,
) -> ToolchainMatrixV1 {
    ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            version(fixture.msrv_minor),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            fixture.msrv_commit,
            "msrv-cargo",
        ),
        profile_with_qualification(
            ToolchainRoleV1::QualifiedStableExecution,
            version(fixture.stable_minor),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            fixture.stable_commit,
            "stable-cargo",
            qualification,
        )
        .unwrap(),
        profile(
            ToolchainRoleV1::BetaShadow,
            version(fixture.beta_minor),
            LifecycleChannelV1::Beta,
            SourceMaturityV1::Provisional,
            fixture.beta_commit,
            "beta-cargo",
        ),
        profile(
            ToolchainRoleV1::NightlyShadow,
            version(fixture.nightly_minor),
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            fixture.nightly_commit,
            fixture.nightly_cargo_commit,
        ),
    )
    .unwrap()
}

pub(super) fn candidate(
    current: ToolchainMatrixV1,
    proposed: ToolchainMatrixV1,
) -> ToolchainCandidateV1 {
    ToolchainCandidateV1::try_new(current, proposed, None, digest("toolchain-discovery")).unwrap()
}

pub(super) fn snapshot(
    matrix: &ToolchainMatrixV1,
    published_at: [u64; 3],
    observed_at: u64,
) -> ToolchainChannelSnapshotV1 {
    ToolchainChannelSnapshotV1::try_new(
        ToolchainChannelHeadV1::new(
            matrix.stable().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(published_at[0]),
        ),
        ToolchainChannelHeadV1::new(
            matrix.beta().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(published_at[1]),
        ),
        ToolchainChannelHeadV1::new(
            matrix.nightly().material().clone(),
            LifecycleTimestampV1::from_unix_seconds(published_at[2]),
        ),
        LifecycleTimestampV1::from_unix_seconds(observed_at),
        ToolchainChannelSnapshotEvidenceV1::new(
            digest("rust-distribution-mirror"),
            digest("toolchain-channel-schema"),
            digest("toolchain-channel-snapshot"),
            digest("toolchain-channel-completeness"),
        ),
    )
    .unwrap()
}

pub(super) fn policy() -> ToolchainCurrencyPolicyV1 {
    ToolchainCurrencyPolicyV1::new(120, 60, 30, 1_000, 600, digest("toolchain-currency-policy"))
}

pub(super) fn decision() -> ToolchainCurrencyDecisionEvidenceV1 {
    ToolchainCurrencyDecisionEvidenceV1::new(
        digest("toolchain-currency-authority"),
        digest("toolchain-currency-decision-schema"),
        digest("toolchain-currency-decision-receipt"),
    )
}

pub(super) fn version(minor: u16) -> RustVersionV1 {
    RustVersionV1::try_new(1, minor, 0).unwrap()
}
