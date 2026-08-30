use super::advisory::{
    active_record, candidate, continue_advisory_normalization, identifier, record_source,
};
use super::lifecycle_support::{digest, profile};
use dependency_declarations_reconcile::*;

pub(super) fn safety_envelope(
    profile: &ToolchainProfileV1,
    evidence: Vec<FactEvidenceClassV1>,
    certainty: FactCertaintyV1,
    coverage: FactCoverageV1,
    observed_at: u64,
    fresh_until: u64,
) -> FactEnvelopeV1 {
    let scope = FactTemporalScopeV1::try_new(
        "oyatie",
        digest("repository-revision"),
        digest("repository-snapshot"),
        digest("toolchain-safety-configuration"),
        profile.material_identity_sha256(),
        digest("toolchain-defect-provider"),
        digest("toolchain-defect-schema"),
    )
    .unwrap();
    let temporal = FactTemporalIdentityV1::try_new(
        scope,
        LifecycleTimestampV1::from_unix_seconds(observed_at),
        LifecycleTimestampV1::from_unix_seconds(fresh_until),
    )
    .unwrap();
    FactEnvelopeV1::new(
        FactEvidenceClassesV1::try_new(evidence).unwrap(),
        certainty,
        coverage,
        temporal,
        digest("toolchain-defect-qualification"),
        digest("toolchain-defect-derivation"),
    )
}

pub(super) fn complete_safety_envelope(profile: &ToolchainProfileV1) -> FactEnvelopeV1 {
    safety_envelope(
        profile,
        vec![FactEvidenceClassV1::Proven, FactEvidenceClassV1::Observed],
        FactCertaintyV1::Exact,
        FactCoverageV1::CompleteForScope {
            scope_sha256: digest("supported-toolchain-defect-scope"),
            exclusions_sha256: digest("declared-toolchain-defect-exclusions"),
        },
        100,
        1_000,
    )
}

pub(super) fn blocking_defect(
    profile: &ToolchainProfileV1,
    identifier: &str,
) -> ToolchainBlockingDefectV1 {
    let advisory = normalized_upstream_advisory(identifier);
    ToolchainBlockingDefectV1::try_new(
        profile,
        &advisory,
        digest(&format!("{identifier}-applicability")),
        ToolchainDefectDecisionEvidenceV1::new(
            digest("toolchain-safety-authority"),
            digest("toolchain-safety-decision-schema"),
            digest(&format!("{identifier}-decision")),
        ),
    )
    .unwrap()
}

pub(super) fn normalized_upstream_advisory(identifier_value: &str) -> NormalizedAdvisoryFactV1 {
    let advisory = identifier(AdvisoryNamespaceV1::Upstream, identifier_value);
    let record = active_record(
        record_source(
            LifecycleComponentV1::UpstreamAdvisory,
            AdvisoryAuthorityV1::Upstream(
                AdvisoryAuthorityNameV1::try_new("rust-lang/rust").unwrap(),
            ),
            identifier_value,
            candidate(),
        ),
        advisory,
        Vec::new(),
        AdvisoryAffectedSetV1::reference_only(digest(&format!("{identifier_value}-reference"))),
        200,
    );
    AdvisoryLedgerV1::try_normalize(vec![record], continue_advisory_normalization)
        .unwrap()
        .facts()[0]
        .clone()
}

pub(super) fn withdrawn_upstream_advisory(identifier_value: &str) -> NormalizedAdvisoryFactV1 {
    let advisory = identifier(AdvisoryNamespaceV1::Upstream, identifier_value);
    let record = AdvisoryRecordV1::try_new(
        record_source(
            LifecycleComponentV1::UpstreamAdvisory,
            AdvisoryAuthorityV1::Upstream(
                AdvisoryAuthorityNameV1::try_new("rust-lang/rust").unwrap(),
            ),
            identifier_value,
            candidate(),
        ),
        advisory,
        Vec::new(),
        AdvisoryLifecycleV1::try_withdrawn(
            LifecycleTimestampV1::from_unix_seconds(100),
            LifecycleTimestampV1::from_unix_seconds(300),
            LifecycleTimestampV1::from_unix_seconds(250),
        )
        .unwrap(),
        AdvisoryAffectedSetV1::reference_only(digest(&format!("{identifier_value}-reference"))),
        digest(&format!("{identifier_value}-withdrawn-record")),
    )
    .unwrap();
    AdvisoryLedgerV1::try_normalize(vec![record], continue_advisory_normalization)
        .unwrap()
        .facts()[0]
        .clone()
}

pub(super) fn safety_posture(
    profile: &ToolchainProfileV1,
    blockers: Vec<ToolchainBlockingDefectV1>,
    evaluated_at: u64,
) -> ToolchainSafetyPostureV1 {
    ToolchainSafetyPostureV1::try_evaluate(
        profile,
        blockers,
        complete_safety_envelope(profile),
        LifecycleTimestampV1::from_unix_seconds(evaluated_at),
    )
    .unwrap()
}

pub(super) fn recovery_matrices() -> (ToolchainMatrixV1, ToolchainMatrixV1) {
    let msrv = profile(
        ToolchainRoleV1::DeclaredMsrvCompatibility,
        RustVersionV1::try_new(1, 96, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "msrv-1.96.0",
        "cargo-msrv-1.96.0",
    );
    let current_stable = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        RustVersionV1::try_new(1, 98, 0).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "88d9e12ae",
        "797e8a9bc",
    );
    let retained_stable = profile(
        ToolchainRoleV1::QualifiedStableExecution,
        RustVersionV1::try_new(1, 97, 1).unwrap(),
        LifecycleChannelV1::Stable,
        SourceMaturityV1::Released,
        "1.97.1-rustc",
        "1.97.1-cargo",
    );
    let beta = profile(
        ToolchainRoleV1::BetaShadow,
        RustVersionV1::try_new(1, 99, 0).unwrap(),
        LifecycleChannelV1::Beta,
        SourceMaturityV1::Provisional,
        "beta-1.99",
        "cargo-beta-1.99",
    );
    let nightly = profile(
        ToolchainRoleV1::NightlyShadow,
        RustVersionV1::try_new(1, 100, 0).unwrap(),
        LifecycleChannelV1::Nightly,
        SourceMaturityV1::Provisional,
        "nightly-1.100",
        "cargo-nightly-1.100",
    );
    let current =
        ToolchainMatrixV1::try_new(msrv.clone(), current_stable, beta.clone(), nightly.clone())
            .unwrap();
    let proposed = ToolchainMatrixV1::try_new(msrv, retained_stable, beta, nightly).unwrap();
    (current, proposed)
}
