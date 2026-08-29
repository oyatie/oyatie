use super::lifecycle_support::profile;
use dependency_declarations_reconcile::*;

#[test]
fn matrix_refuses_an_msrv_newer_than_stable() {
    let failure = ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            RustVersionV1::try_new(1, 99, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "msrv-rustc",
            "msrv-cargo",
        ),
        profile(
            ToolchainRoleV1::QualifiedStableExecution,
            RustVersionV1::try_new(1, 98, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "88d9e12ae",
            "797e8a9bc",
        ),
        profile(
            ToolchainRoleV1::BetaShadow,
            RustVersionV1::try_new(1, 99, 0).unwrap(),
            LifecycleChannelV1::Beta,
            SourceMaturityV1::Provisional,
            "f47d5bb13",
            "eb98b54bc",
        ),
        profile(
            ToolchainRoleV1::NightlyShadow,
            RustVersionV1::try_new(1, 100, 0).unwrap(),
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            "bff8e12ff",
            "e8cb624d5",
        ),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::UnsupportedVersionRelation
    );
}

#[test]
fn matrix_refuses_non_increasing_release_trains() {
    let try_matrix = |stable_minor, beta_minor, nightly_minor| {
        ToolchainMatrixV1::try_new(
            profile(
                ToolchainRoleV1::DeclaredMsrvCompatibility,
                RustVersionV1::try_new(1, 64, 0).unwrap(),
                LifecycleChannelV1::Stable,
                SourceMaturityV1::Released,
                "msrv-rustc",
                "msrv-cargo",
            ),
            profile(
                ToolchainRoleV1::QualifiedStableExecution,
                RustVersionV1::try_new(1, stable_minor, 0).unwrap(),
                LifecycleChannelV1::Stable,
                SourceMaturityV1::Released,
                "stable-rustc",
                "stable-cargo",
            ),
            profile(
                ToolchainRoleV1::BetaShadow,
                RustVersionV1::try_new(1, beta_minor, 0).unwrap(),
                LifecycleChannelV1::Beta,
                SourceMaturityV1::Provisional,
                "beta-rustc",
                "beta-cargo",
            ),
            profile(
                ToolchainRoleV1::NightlyShadow,
                RustVersionV1::try_new(1, nightly_minor, 0).unwrap(),
                LifecycleChannelV1::Nightly,
                SourceMaturityV1::Provisional,
                "nightly-rustc",
                "nightly-cargo",
            ),
        )
    };

    for versions in [(98, 98, 100), (98, 97, 100), (98, 99, 99), (98, 100, 99)] {
        let failure = try_matrix(versions.0, versions.1, versions.2).unwrap_err();
        assert_eq!(
            failure.class(),
            LifecycleFailureClassV1::UnsupportedVersionRelation
        );
    }
}
