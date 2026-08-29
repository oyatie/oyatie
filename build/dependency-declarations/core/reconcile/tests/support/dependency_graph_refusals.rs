use super::dependency_graph::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

fn graph_for_candidate(
    candidate: &DependencyCandidateV1,
    envelope: FactEnvelopeV1,
) -> DependencyGraphV1 {
    DependencyGraphV1::try_new(
        envelope,
        vec![node(
            "h2-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(candidate.current().identity_sha256()),
        )],
        Vec::new(),
    )
    .unwrap()
}

#[test]
fn analysis_refuses_partial_inferred_speculative_or_stale_graphs() {
    let candidate = qualified_h2_candidate();
    let cases = [
        (
            complete_envelope(
                vec![FactEvidenceClassV1::Declared],
                FactCertaintyV1::Exact,
                FactCoverageV1::Partial {
                    scope_sha256: digest("partial-scope"),
                    evidence_sha256: digest("partial-evidence"),
                },
                100,
                300,
            ),
            LifecycleTimestampV1::from_unix_seconds(200),
            LifecycleFailureClassV1::IncompleteFactCoverage,
        ),
        (
            complete_envelope(
                vec![FactEvidenceClassV1::Inferred],
                FactCertaintyV1::Exact,
                FactCoverageV1::CompleteForScope {
                    scope_sha256: digest("scope"),
                    exclusions_sha256: digest("exclusions"),
                },
                100,
                300,
            ),
            LifecycleTimestampV1::from_unix_seconds(200),
            LifecycleFailureClassV1::UnsupportedFactEvidence,
        ),
        (
            complete_envelope(
                vec![FactEvidenceClassV1::Proven],
                FactCertaintyV1::Speculative,
                FactCoverageV1::CompleteForScope {
                    scope_sha256: digest("scope"),
                    exclusions_sha256: digest("exclusions"),
                },
                100,
                300,
            ),
            LifecycleTimestampV1::from_unix_seconds(200),
            LifecycleFailureClassV1::UnsupportedFactEvidence,
        ),
        (
            complete_envelope(
                vec![FactEvidenceClassV1::Observed],
                FactCertaintyV1::Exact,
                FactCoverageV1::Excluded {
                    scope_sha256: digest("excluded-scope"),
                    exclusion_sha256: digest("exclusion-reason"),
                },
                100,
                300,
            ),
            LifecycleTimestampV1::from_unix_seconds(200),
            LifecycleFailureClassV1::IncompleteFactCoverage,
        ),
        (
            complete_envelope(
                vec![FactEvidenceClassV1::Proven],
                FactCertaintyV1::Conservative,
                FactCoverageV1::Unknown {
                    scope_sha256: digest("unknown-scope"),
                    reason_sha256: digest("unknown-reason"),
                },
                100,
                300,
            ),
            LifecycleTimestampV1::from_unix_seconds(200),
            LifecycleFailureClassV1::IncompleteFactCoverage,
        ),
        (
            safe_envelope(),
            LifecycleTimestampV1::from_unix_seconds(301),
            LifecycleFailureClassV1::StaleFact,
        ),
        (
            safe_envelope(),
            LifecycleTimestampV1::from_unix_seconds(99),
            LifecycleFailureClassV1::StaleFact,
        ),
    ];

    for (envelope, now, expected) in cases {
        let graph = graph_for_candidate(&candidate, envelope);
        let failure = graph
            .try_analyze_candidates(std::slice::from_ref(&candidate), now)
            .unwrap_err();
        assert_eq!(failure.class(), expected);
    }
}

#[test]
fn graph_refuses_duplicate_units_edges_unknown_endpoints_and_self_edges() {
    let duplicate = node("duplicate", DependencyGraphNodeKindV1::CargoTarget, None);
    let failure = DependencyGraphV1::try_new(
        safe_envelope(),
        vec![duplicate.clone(), duplicate],
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );

    let nodes = vec![
        node("dependent", DependencyGraphNodeKindV1::CargoTarget, None),
        node("dependency", DependencyGraphNodeKindV1::CargoPackage, None),
    ];
    let duplicate_edge = edge(
        "dependent",
        "dependency",
        DependencyGraphEdgeKindV1::NormalTarget,
    );
    let conflicting_evidence = DependencyGraphEdgeV1::new(
        digest("dependent"),
        digest("dependency"),
        DependencyGraphEdgeKindV1::NormalTarget,
        digest("configured-profile"),
        digest("conflicting-evidence"),
    )
    .unwrap();
    let failure = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes.clone(),
        vec![duplicate_edge, conflicting_evidence],
    )
    .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );

    let unknown = edge(
        "dependent",
        "missing",
        DependencyGraphEdgeKindV1::NormalTarget,
    );
    let failure = DependencyGraphV1::try_new(safe_envelope(), nodes, vec![unknown]).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );

    let self_edge = DependencyGraphEdgeV1::new(
        digest("self"),
        digest("self"),
        DependencyGraphEdgeKindV1::NormalTarget,
        digest("configuration"),
        digest("evidence"),
    )
    .unwrap_err();
    assert_eq!(
        self_edge.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );

    let failure = DependencyGraphV1::try_new(safe_envelope(), Vec::new(), Vec::new()).unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );
    let misplaced_release = node(
        "target-with-release",
        DependencyGraphNodeKindV1::CargoTarget,
        Some(digest("release")),
    );
    let failure = DependencyGraphV1::try_new(safe_envelope(), vec![misplaced_release], Vec::new())
        .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::InvalidDependencyGraph
    );
}

#[test]
fn analysis_refuses_missing_or_duplicate_candidate_roots() {
    let candidate = qualified_h2_candidate();
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        vec![node(
            "unrelated",
            DependencyGraphNodeKindV1::CargoPackage,
            None,
        )],
        Vec::new(),
    )
    .unwrap();
    let failure = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
        )
        .unwrap_err();
    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::MissingDependencyRoot
    );

    let graph = graph_for_candidate(&candidate, safe_envelope());
    let failure = graph
        .try_analyze_candidates(&[], LifecycleTimestampV1::from_unix_seconds(200))
        .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    let failure = graph
        .try_analyze_candidates(
            &[candidate.clone(), candidate],
            LifecycleTimestampV1::from_unix_seconds(200),
        )
        .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::DuplicateIdentity);
}

#[test]
fn temporal_envelope_refuses_inverted_observation_window() {
    let scope = FactTemporalScopeV1::try_new(
        "oyatie/oyatie",
        digest("revision"),
        digest("snapshot"),
        digest("configuration"),
        digest("toolchain"),
        digest("producer"),
        digest("schema"),
    )
    .unwrap();
    let failure = FactTemporalIdentityV1::try_new(
        scope,
        LifecycleTimestampV1::from_unix_seconds(300),
        LifecycleTimestampV1::from_unix_seconds(200),
    )
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::InvalidFact);
}

#[test]
fn evidence_class_set_refuses_empty_or_duplicate_axes() {
    let failure = FactEvidenceClassesV1::try_new(Vec::new()).unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    let failure = FactEvidenceClassesV1::try_new(vec![
        FactEvidenceClassV1::Declared,
        FactEvidenceClassV1::Declared,
    ])
    .unwrap_err();
    assert_eq!(failure.class(), LifecycleFailureClassV1::DuplicateIdentity);
}
