#[path = "support/advisory.rs"]
mod advisory;
#[path = "support/advisory_refusals.rs"]
mod advisory_refusals;
#[path = "support/dependency_candidate.rs"]
mod dependency_candidate;
#[path = "support/dependency_candidate_refusals.rs"]
mod dependency_candidate_refusals;
#[path = "support/dependency_graph.rs"]
mod dependency_graph;
#[path = "support/dependency_graph_control.rs"]
mod dependency_graph_control;
#[path = "support/dependency_graph_refusals.rs"]
mod dependency_graph_refusals;
#[path = "support/dependency_graph_scale.rs"]
mod dependency_graph_scale;
#[path = "support/dependency_qualification.rs"]
mod dependency_qualification;
#[path = "support/dependency_qualification_refusals.rs"]
mod dependency_qualification_refusals;
#[path = "support/lifecycle_refusals.rs"]
mod lifecycle_refusals;
#[path = "support/lifecycle.rs"]
mod lifecycle_support;
#[path = "support/lifecycle_toolchain_refusals.rs"]
mod lifecycle_toolchain_refusals;
#[path = "support/lifecycle_toolchain_transition.rs"]
mod lifecycle_toolchain_transition;

use dependency_declarations_reconcile::*;
use lifecycle_support::*;

#[test]
fn msrv_stable_beta_and_nightly_are_independent_exact_profiles() {
    let v98 = RustVersionV1::try_new(1, 98, 0).unwrap();
    let msrv = profile(
        ToolchainRoleV1::DeclaredMsrvCompatibility,
        v98,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "797e8a9bc",
    );
    let stable = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        v98,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "797e8a9bc",
    );
    let beta = profile(
        ToolchainRoleV1::BetaShadow,
        RustVersionV1::try_new(1, 99, 0).unwrap(),
        LifecycleChannelV1::Beta,
        SourceMaturityV1::Provisional,
        "f47d5bb13",
        "eb98b54bc",
    );
    let nightly = profile(
        ToolchainRoleV1::NightlyShadow,
        RustVersionV1::try_new(1, 100, 0).unwrap(),
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "bff8e12ff",
        "e8cb624d5",
    );
    assert_ne!(msrv.identity_sha256(), stable.identity_sha256());

    let matrix = ToolchainMatrixV1::try_new(msrv, stable, beta, nightly.clone()).unwrap();
    assert_eq!(matrix.msrv().version(), matrix.stable().version());
    assert_ne!(
        matrix.stable().identity_sha256(),
        matrix.nightly().identity_sha256()
    );

    let prior_nightly = profile(
        ToolchainRoleV1::NightlyShadow,
        RustVersionV1::try_new(1, 100, 0).unwrap(),
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "c656540d6",
        "cargo-prior-nightly",
    );
    assert_eq!(prior_nightly.version(), nightly.version());
    assert_ne!(prior_nightly.identity_sha256(), nightly.identity_sha256());

    let cargo_drift = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        v98,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "cargo-updated-commit",
    );
    assert_ne!(
        cargo_drift.identity_sha256(),
        matrix.stable().identity_sha256()
    );
}

#[test]
fn toolchain_evidence_must_match_its_role() {
    let failure = profile_with_qualification(
        ToolchainRoleV1::QualifiedStableExecution,
        RustVersionV1::try_new(1, 98, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "797e8a9bc",
        ToolchainQualificationV1::Shadow {
            observation_receipt_sha256: digest("not-production-evidence"),
        },
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::ToolchainRoleMismatch
    );
}

#[test]
fn adoption_refuses_an_unknown_msrv_effect() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let failure = ReleaseDispositionV1::try_new(
        item.identity_sha256(),
        "product-owner",
        ReleaseDecisionV1::Adopt,
        ReleaseDispositionEvidenceV1::new(
            digest("rationale"),
            ReleaseAffectedUnitsV1::try_new(1, 32, digest("affected-units")).unwrap(),
            ReleaseMsrvEffectV1::Unknown {
                evidence_sha256: digest("unknown-msrv"),
            },
            digest("evidence"),
            ReevaluationTriggerV1::OnEvidenceChange,
        ),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::InvalidFact);
}

#[test]
fn released_ledger_is_order_independent_and_disposition_complete() {
    let rust = source(
        LifecycleComponentV1::Rust,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "rust-1.98.0",
    );
    let cargo = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "cargo-1.98.0",
    );
    let rust_item = release_item(
        &rust,
        "rust#format-into",
        ReleaseItemKindV1::StandardLibrary,
    );
    let cargo_item = release_item(
        &cargo,
        "cargo#credential-token-crlf",
        ReleaseItemKindV1::Cargo,
    );
    let batches = vec![
        qualified_batch(rust, std::slice::from_ref(&rust_item)),
        qualified_batch(cargo, std::slice::from_ref(&cargo_item)),
    ];
    let items = vec![rust_item.clone(), cargo_item.clone()];
    let dispositions = vec![
        disposition(&rust_item, ReleaseDecisionV1::Benchmark),
        disposition(&cargo_item, ReleaseDecisionV1::Benchmark),
    ];
    let ledger =
        ReleaseLedgerV1::try_new(batches.clone(), items.clone(), dispositions.clone()).unwrap();
    ledger.require_released_complete().unwrap();

    let reversed = ReleaseLedgerV1::try_new(
        batches.into_iter().rev().collect(),
        items.into_iter().rev().collect(),
        dispositions.into_iter().rev().collect(),
    )
    .unwrap();
    assert_eq!(ledger.identity_sha256(), reversed.identity_sha256());
}

#[test]
fn incomplete_or_provisional_release_evidence_refuses_completeness() {
    let nightly = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "e8cb624d5",
    );
    let item = release_item(&nightly, "cargo#build-dir-layout", ReleaseItemKindV1::Cargo);
    let additional_item = release_item(&nightly, "cargo#sbom", ReleaseItemKindV1::Cargo);
    let declared = qualified_batch(nightly.clone(), std::slice::from_ref(&item));
    let ledger = ReleaseLedgerV1::try_new(
        vec![declared.clone()],
        vec![item.clone()],
        vec![disposition(&item, ReleaseDecisionV1::Benchmark)],
    )
    .unwrap();
    assert_eq!(
        ledger.completeness(),
        ReleaseLedgerCompletenessV1::Provisional
    );
    assert_eq!(
        ledger.require_released_complete().unwrap_err().class(),
        LifecycleFailureClassV1::ProvisionalSource
    );

    let missing =
        ReleaseLedgerV1::try_new(vec![declared], vec![item.clone()], Vec::new()).unwrap_err();
    assert_eq!(missing.class(), LifecycleFailureClassV1::MissingDisposition);

    let complete_batch = qualified_batch(nightly, &[item.clone(), additional_item]);
    let wrong_coverage = ReleaseLedgerV1::try_new(
        vec![complete_batch],
        vec![item.clone()],
        vec![disposition(&item, ReleaseDecisionV1::Benchmark)],
    )
    .unwrap_err();
    assert_eq!(
        wrong_coverage.class(),
        LifecycleFailureClassV1::SourceCoverageMismatch
    );
}

#[test]
fn candidate_extractor_cannot_authorize_a_released_ledger() {
    let cargo = source(
        LifecycleComponentV1::Cargo,
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "cargo-1.98.0",
    );
    let item = release_item(
        &cargo,
        "cargo#credential-token-crlf",
        ReleaseItemKindV1::Cargo,
    );
    let candidate = extraction(
        &cargo,
        ReleaseExtractionQualificationV1::Candidate {
            observation_sha256: digest("candidate-observation"),
        },
    );
    let batch = ReleaseSourceBatchV1::try_from_items(
        cargo,
        candidate,
        std::slice::from_ref(&item),
        digest("candidate-extraction"),
    )
    .unwrap();
    let ledger = ReleaseLedgerV1::try_new(
        vec![batch],
        vec![item.clone()],
        vec![disposition(&item, ReleaseDecisionV1::Benchmark)],
    )
    .unwrap();
    assert_eq!(
        ledger.completeness(),
        ReleaseLedgerCompletenessV1::UnqualifiedExtraction
    );
    assert_eq!(
        ledger.require_released_complete().unwrap_err().class(),
        LifecycleFailureClassV1::UnqualifiedExtraction
    );
}
