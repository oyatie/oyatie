use super::lifecycle_support::{digest, profile, source};
use dependency_declarations_reconcile::*;

const HOST: &str = "aarch64-apple-darwin";

fn tool(name: &str, version: &str, commit: &str) -> ToolIdentityV1 {
    ToolIdentityV1::try_new(
        name,
        version,
        commit,
        HOST,
        digest(&format!("{name}-{commit}")),
    )
    .unwrap()
}

fn nightly_profile(
    minor: u16,
    rustc_commit: &str,
    cargo_commit: &str,
    llvm_version: &str,
    target_seed: &str,
    qualification_seed: &str,
) -> ToolchainProfileV1 {
    let version = RustVersionV1::try_new(1, minor, 0).unwrap();
    ToolchainProfileV1::try_new(
        ToolchainRoleV1::NightlyShadow,
        version,
        source(
            LifecycleComponentV1::RustDistribution,
            LifecycleChannelV1::Nightly,
            SourceMaturityV1::Provisional,
            rustc_commit,
        ),
        ToolchainToolsV1::try_new(
            tool("rustc", &format!("1.{minor}.0-nightly"), rustc_commit),
            tool("cargo", &format!("1.{minor}.0"), cargo_commit),
            tool("rustfmt", &format!("1.{minor}.0"), rustc_commit),
            tool("clippy", &format!("0.1.{minor}"), rustc_commit),
        )
        .unwrap(),
        ToolchainQualificationV1::Shadow {
            observation_receipt_sha256: digest(qualification_seed),
        },
        llvm_version,
        vec![
            ToolchainTargetV1::try_new(
                HOST,
                digest(&format!("{target_seed}-std")),
                digest(&format!("{target_seed}-components")),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn matrix(nightly: ToolchainProfileV1) -> ToolchainMatrixV1 {
    ToolchainMatrixV1::try_new(
        profile(
            ToolchainRoleV1::DeclaredMsrvCompatibility,
            RustVersionV1::try_new(1, 98, 0).unwrap(),
            LifecycleChannelV1::Stable,
            SourceMaturityV1::Released,
            "88d9e12ae-msrv",
            "797e8a9bc-msrv",
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
        nightly,
    )
    .unwrap()
}

fn changes(candidate: &ToolchainCandidateV1) -> Vec<(ToolchainRoleV1, ToolchainChangeAxisV1)> {
    candidate
        .delta()
        .changes()
        .iter()
        .map(|change| (change.role(), change.axis()))
        .collect()
}

#[test]
fn same_version_nightly_refresh_reports_exact_mechanical_axes() {
    let current = matrix(nightly_profile(
        100,
        "c656540d6",
        "cargo-c656540d6",
        "LLVM 23.1.0",
        "c656540d6",
        "c656540d6-shadow",
    ));
    let proposed = matrix(nightly_profile(
        100,
        "bff8e12ff",
        "e8cb624d5",
        "LLVM 23.1.0",
        "bff8e12ff",
        "bff8e12ff-shadow",
    ));
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("nightly-refresh")).unwrap();

    assert_eq!(
        changes(&candidate),
        [
            ToolchainChangeAxisV1::DistributionSource,
            ToolchainChangeAxisV1::Rustc,
            ToolchainChangeAxisV1::Cargo,
            ToolchainChangeAxisV1::Rustfmt,
            ToolchainChangeAxisV1::Clippy,
            ToolchainChangeAxisV1::TargetClosure,
            ToolchainChangeAxisV1::Qualification,
        ]
        .map(|axis| (ToolchainRoleV1::NightlyShadow, axis))
    );
    assert!(
        candidate
            .delta()
            .changed(ToolchainRoleV1::NightlyShadow, ToolchainChangeAxisV1::Cargo)
    );
    assert!(!candidate.delta().changed(
        ToolchainRoleV1::NightlyShadow,
        ToolchainChangeAxisV1::RustVersion
    ));
}

#[test]
fn all_profile_axes_are_reported_once_in_canonical_order() {
    let current = matrix(nightly_profile(
        100,
        "c656540d6",
        "cargo-c656540d6",
        "LLVM 22.1.8",
        "old-target",
        "old-qualification",
    ));
    let proposed = matrix(nightly_profile(
        101,
        "nightly-1.101",
        "cargo-nightly-1.101",
        "LLVM 23.1.0",
        "new-target",
        "new-qualification",
    ));
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("all-axes")).unwrap();

    assert_eq!(
        changes(&candidate),
        [
            ToolchainChangeAxisV1::RustVersion,
            ToolchainChangeAxisV1::DistributionSource,
            ToolchainChangeAxisV1::Rustc,
            ToolchainChangeAxisV1::Cargo,
            ToolchainChangeAxisV1::Rustfmt,
            ToolchainChangeAxisV1::Clippy,
            ToolchainChangeAxisV1::Llvm,
            ToolchainChangeAxisV1::TargetClosure,
            ToolchainChangeAxisV1::Qualification,
        ]
        .map(|axis| (ToolchainRoleV1::NightlyShadow, axis))
    );
    assert_eq!(candidate.changed_roles(), &[ToolchainRoleV1::NightlyShadow]);
}

#[test]
fn candidate_delta_is_derived_from_profile_axis_identities() {
    let current = matrix(nightly_profile(
        100,
        "c656540d6",
        "cargo-c656540d6",
        "LLVM 22.1.8",
        "old-target",
        "old-qualification",
    ));
    let proposed = matrix(nightly_profile(
        101,
        "nightly-1.101",
        "cargo-nightly-1.101",
        "LLVM 23.1.0",
        "new-target",
        "new-qualification",
    ));
    let candidate =
        ToolchainCandidateV1::try_new(current, proposed, None, digest("profile-axis-projection"))
            .unwrap();

    for axis in ToolchainChangeAxisV1::ALL {
        let current_axis = candidate.current().nightly().axes().identity_sha256(axis);
        let proposed_axis = candidate.proposed().nightly().axes().identity_sha256(axis);
        assert_eq!(
            candidate
                .delta()
                .changed(ToolchainRoleV1::NightlyShadow, axis),
            current_axis != proposed_axis,
            "candidate delta diverged from the profile-owned {axis:?} identity"
        );
    }
}
