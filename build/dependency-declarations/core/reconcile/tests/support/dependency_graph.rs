use super::dependency_candidate::*;
use super::lifecycle_support::digest;
use dependency_declarations_reconcile::*;

pub(super) fn qualified_h2_candidate() -> DependencyCandidateV1 {
    qualified_candidate("h2", "0.4.15", "0.4.16")
}

pub(super) fn qualified_candidate(
    package_name: &str,
    current_version: &str,
    proposed_version: &str,
) -> DependencyCandidateV1 {
    let current = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            &format!("{package_name}-{current_version}"),
        ),
        package(package_name),
        current_version,
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        false,
    );
    let proposed = release(
        dependency_source(
            LifecycleComponentV1::DependencyRegistry,
            SourceMaturityV1::Released,
            &format!("{package_name}-{proposed_version}"),
        ),
        package(package_name),
        proposed_version,
        DependencyPublicationStateV1::Available,
        qualified_dependency(),
        true,
    );
    DependencyCandidateV1::try_new(
        current,
        proposed,
        digest(&format!("{package_name}-candidate")),
    )
    .unwrap()
}

pub(super) fn complete_envelope(
    evidence: Vec<FactEvidenceClassV1>,
    certainty: FactCertaintyV1,
    coverage: FactCoverageV1,
    observed_at: u64,
    fresh_until: u64,
) -> FactEnvelopeV1 {
    complete_envelope_for_toolchain(
        evidence,
        certainty,
        coverage,
        digest("rust-toolchain-profile"),
        observed_at,
        fresh_until,
    )
}

pub(super) fn complete_envelope_for_toolchain(
    evidence: Vec<FactEvidenceClassV1>,
    certainty: FactCertaintyV1,
    coverage: FactCoverageV1,
    toolchain_sha256: DigestV1,
    observed_at: u64,
    fresh_until: u64,
) -> FactEnvelopeV1 {
    let scope = FactTemporalScopeV1::try_new(
        "oyatie/oyatie",
        digest("repository-revision"),
        digest("repository-snapshot"),
        digest("cargo-buck-configurations"),
        toolchain_sha256,
        digest("dependency-graph-producer"),
        digest("dependency-graph-schema"),
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
        digest("dependency-graph-qualification"),
        digest("dependency-graph-derivation"),
    )
}

pub(super) fn safe_envelope() -> FactEnvelopeV1 {
    complete_envelope(
        vec![FactEvidenceClassV1::Declared, FactEvidenceClassV1::Proven],
        FactCertaintyV1::Exact,
        FactCoverageV1::CompleteForScope {
            scope_sha256: digest("cargo-and-configured-buck-scope"),
            exclusions_sha256: digest("declared-exclusions"),
        },
        100,
        300,
    )
}

pub(super) fn node(
    name: &str,
    kind: DependencyGraphNodeKindV1,
    release_identity: Option<DigestV1>,
) -> DependencyGraphNodeV1 {
    DependencyGraphNodeV1::new(
        digest(name),
        kind,
        DependencyExecutionDomainV1::Target,
        release_identity,
        digest("all-target-platforms"),
        digest(&format!("{name}-evidence")),
    )
}

pub(super) fn edge(
    dependent: &str,
    dependency: &str,
    kind: DependencyGraphEdgeKindV1,
) -> DependencyGraphEdgeV1 {
    DependencyGraphEdgeV1::new(
        digest(dependent),
        digest(dependency),
        kind,
        digest("configured-profile"),
        digest(&format!("{dependent}-to-{dependency}-evidence")),
    )
    .unwrap()
}

pub(super) const fn continue_dependency_impact(
    _: DependencyImpactProgressV1,
) -> LifecycleControlDecisionV1 {
    LifecycleControlDecisionV1::Continue
}

pub(super) const fn continue_dependency_graph_construction(
    _: DependencyGraphConstructionProgressV1,
) -> LifecycleControlDecisionV1 {
    LifecycleControlDecisionV1::Continue
}

#[test]
fn whole_graph_analysis_is_order_independent_cycle_safe_and_closure_complete() {
    let candidate = qualified_h2_candidate();
    let h2_release = candidate.current().identity_sha256();
    let nodes = vec![
        node(
            "h2-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(h2_release),
        ),
        node("hyper-target", DependencyGraphNodeKindV1::CargoTarget, None),
        node("cycle-helper", DependencyGraphNodeKindV1::CargoTarget, None),
        node(
            "reqwest-target",
            DependencyGraphNodeKindV1::CargoTarget,
            None,
        ),
        node("app-buck", DependencyGraphNodeKindV1::BuckTarget, None),
        node("unrelated", DependencyGraphNodeKindV1::BuckTarget, None),
    ];
    let edges = vec![
        edge(
            "hyper-target",
            "h2-package",
            DependencyGraphEdgeKindV1::NormalTarget,
        ),
        edge(
            "cycle-helper",
            "hyper-target",
            DependencyGraphEdgeKindV1::BuildHost,
        ),
        edge(
            "hyper-target",
            "cycle-helper",
            DependencyGraphEdgeKindV1::DevTarget,
        ),
        edge(
            "reqwest-target",
            "hyper-target",
            DependencyGraphEdgeKindV1::NormalTarget,
        ),
        edge(
            "app-buck",
            "reqwest-target",
            DependencyGraphEdgeKindV1::ConfiguredBuck,
        ),
    ];
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes.clone(),
        edges.clone(),
        continue_dependency_graph_construction,
    )
    .unwrap();
    let reversed = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes.into_iter().rev().collect(),
        edges.into_iter().rev().collect(),
        continue_dependency_graph_construction,
    )
    .unwrap();
    assert_eq!(graph.identity_sha256(), reversed.identity_sha256());

    let batch = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            continue_dependency_impact,
        )
        .unwrap();
    let reversed_batch = reversed
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            continue_dependency_impact,
        )
        .unwrap();
    assert_eq!(batch.identity_sha256(), reversed_batch.identity_sha256());
    assert_eq!(batch.impacts().len(), 1);
    assert_eq!(batch.fact_envelope(), graph.envelope());

    let impact = &batch.impacts()[0];
    assert_eq!(impact.fact_envelope(), graph.envelope());
    let mut expected = vec![
        digest("h2-package"),
        digest("hyper-target"),
        digest("cycle-helper"),
        digest("reqwest-target"),
        digest("app-buck"),
    ];
    expected.sort_unstable();
    let actual: Vec<DigestV1> = impact
        .affected_nodes()
        .iter()
        .map(DependencyGraphNodeV1::unit_identity_sha256)
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(impact.root_nodes().len(), 1);
    assert_eq!(impact.affected_edges().len(), 5);
    assert!(
        impact
            .affected_edges()
            .iter()
            .any(|edge| { edge.kind() == DependencyGraphEdgeKindV1::ConfiguredBuck })
    );
    assert!(!actual.contains(&digest("unrelated")));
}
