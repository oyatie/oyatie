use super::lifecycle_support::{digest, profile, source};
use dependency_declarations_reconcile::*;

fn tool(name: &str, commit: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        "1.98.0",
        commit,
        "aarch64-apple-darwin",
        digest(&format!("{name}-{commit}")),
    )
    .unwrap()
}

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

#[test]
fn profile_refuses_missing_host_target_closure() {
    let tools = ToolchainToolsV1::try_new(
        tool("rustc", "88d9e12ae"),
        tool("cargo", "797e8a9bc"),
        tool("rustfmt", "rustfmt-1.98.0"),
        tool("clippy", "clippy-1.98.0"),
    )
    .unwrap();
    let failure = ToolchainProfileV1::try_new(
        ToolchainRoleV1::QualifiedStableExecution,
        RustVersionV1::try_new(1, 98, 0).unwrap(),
        source(
            LifecycleComponentV1::RustDistribution,
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "88d9e12ae",
        ),
        tools,
        ToolchainQualificationV1::Production {
            qualification_receipt_sha256: digest("stable-qualification"),
        },
        "LLVM 22.1.8",
        vec![
            ToolchainTargetV1::try_new(
                "x86_64-unknown-linux-gnu",
                digest("x86_64-std"),
                digest("x86_64-components"),
            )
            .unwrap(),
        ],
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainTargetMismatch
    );
}
