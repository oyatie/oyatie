use dependency_declarations_reconcile::{
    ExecutionToolchainProfile, ExecutionVersionDelta, PatchOnlyExecutionToolchainDecision,
    PatchOnlyExecutionToolchainRefusal, ToolchainSide, apply_patch_only_execution_toolchain_policy,
};

use crate::support::{analyze, apply_policy, custom_declaration, declaration};

#[test]
fn patch_only_policy_admits_unchanged_and_forward_patch_execution() {
    let unchanged = apply_policy(
        &declaration("1.98.0"),
        &declaration("1.98.0"),
        "1.96.0",
        "1.96.0",
    )
    .unwrap();
    let PatchOnlyExecutionToolchainDecision::Unchanged(version) = unchanged else {
        panic!("expected unchanged decision")
    };
    assert_eq!(version.to_string(), "1.98.0");

    let forward = apply_policy(
        &declaration("1.98.0"),
        &declaration("1.98.1"),
        "1.96.0",
        "1.96.0",
    )
    .unwrap();
    let PatchOnlyExecutionToolchainDecision::ForwardPatch {
        protected,
        candidate,
    } = forward
    else {
        panic!("expected forward-patch decision")
    };
    assert_eq!(
        (protected.to_string(), candidate.to_string()),
        ("1.98.0".to_owned(), "1.98.1".to_owned())
    );
}

#[test]
fn only_nondecreasing_patch_transitions_are_admitted() {
    for (protected, candidate, expected) in [
        ("1.98.1", "1.98.0", ExecutionVersionDelta::Downgrade),
        ("1.98.0", "1.99.0", ExecutionVersionDelta::ForwardMinor),
        ("1.98.0", "2.0.0", ExecutionVersionDelta::ForwardMajor),
    ] {
        assert!(matches!(
            apply_policy(
                &declaration(protected),
                &declaration(candidate),
                "1.96.0",
                "1.96.0",
            ),
            Err(PatchOnlyExecutionToolchainRefusal::VersionDeltaNotAdmitted(
                delta,
                _,
                _
            )) if delta == expected
        ));
    }
}

#[test]
fn patch_only_policy_refuses_unqualified_declaration_changes() {
    let changed_msrv = analyze(
        &declaration("1.98.0"),
        &declaration("1.98.1"),
        "1.96.0",
        "1.97.0",
    )
    .unwrap();
    assert!(matches!(
        apply_patch_only_execution_toolchain_policy(&changed_msrv),
        Err(PatchOnlyExecutionToolchainRefusal::MsrvChanged(_, _))
    ));

    let changed_profile = analyze(
        &declaration("1.98.0"),
        &custom_declaration("1.98.1", "[\"rustfmt\", \"clippy\"]", "default", ""),
        "1.96.0",
        "1.96.0",
    )
    .unwrap();
    assert!(matches!(
        apply_patch_only_execution_toolchain_policy(&changed_profile),
        Err(PatchOnlyExecutionToolchainRefusal::NonMinimalProfile(
            ToolchainSide::Candidate,
            ExecutionToolchainProfile::Default
        ))
    ));

    for components in ["[\"rustfmt\"]", "[\"rustfmt\", \"clippy\", \"rust-src\"]"] {
        let changed_components = analyze(
            &declaration("1.98.0"),
            &custom_declaration("1.98.1", components, "minimal", ""),
            "1.96.0",
            "1.96.0",
        )
        .unwrap();
        assert!(matches!(
            apply_patch_only_execution_toolchain_policy(&changed_components),
            Err(PatchOnlyExecutionToolchainRefusal::ComponentSet(
                ToolchainSide::Candidate,
                _
            ))
        ));
    }

    let changed_targets = analyze(
        &declaration("1.98.0"),
        &custom_declaration(
            "1.98.1",
            "[\"rustfmt\", \"clippy\"]",
            "minimal",
            "targets = [\"wasm32-unknown-unknown\"]\n",
        ),
        "1.96.0",
        "1.96.0",
    )
    .unwrap();
    assert!(matches!(
        apply_patch_only_execution_toolchain_policy(&changed_targets),
        Err(PatchOnlyExecutionToolchainRefusal::TargetsChanged(_, _))
    ));
}

#[test]
fn execution_below_msrv_is_refused_separately() {
    assert!(matches!(
        apply_policy(
            &declaration("1.98.0"),
            &declaration("1.98.0"),
            "1.98.1",
            "1.98.1",
        ),
        Err(PatchOnlyExecutionToolchainRefusal::ExecutionBelowMsrv(_, _))
    ));
}
