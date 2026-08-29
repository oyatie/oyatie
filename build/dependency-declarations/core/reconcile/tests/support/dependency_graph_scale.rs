use super::dependency_graph::*;
use dependency_declarations_reconcile::*;

#[test]
fn candidate_batch_is_canonical_with_shared_fanout() {
    let h2 = qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let nodes = vec![
        node(
            "h2-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(h2.current().identity_sha256()),
        ),
        node(
            "tokio-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(tokio.current().identity_sha256()),
        ),
        node("http-client", DependencyGraphNodeKindV1::CargoTarget, None),
        node("runtime", DependencyGraphNodeKindV1::CargoTarget, None),
        node("shared-app", DependencyGraphNodeKindV1::BuckTarget, None),
    ];
    let edges = vec![
        edge(
            "http-client",
            "h2-package",
            DependencyGraphEdgeKindV1::NormalTarget,
        ),
        edge(
            "runtime",
            "tokio-package",
            DependencyGraphEdgeKindV1::NormalTarget,
        ),
        edge(
            "shared-app",
            "http-client",
            DependencyGraphEdgeKindV1::ConfiguredBuck,
        ),
        edge(
            "shared-app",
            "runtime",
            DependencyGraphEdgeKindV1::ConfiguredBuck,
        ),
    ];
    let graph = DependencyGraphV1::try_new(safe_envelope(), nodes, edges).unwrap();
    let now = LifecycleTimestampV1::from_unix_seconds(200);
    let forward = graph
        .try_analyze_candidates(
            &[h2.clone(), tokio.clone()],
            now,
            continue_dependency_impact,
        )
        .unwrap();
    let reverse = graph
        .try_analyze_candidates(&[tokio, h2], now, continue_dependency_impact)
        .unwrap();

    assert_eq!(forward.identity_sha256(), reverse.identity_sha256());
    assert_eq!(forward.impacts().len(), 2);
    assert!(
        forward
            .impacts()
            .iter()
            .all(|impact| impact.affected_nodes().len() == 3)
    );
    assert!(
        forward
            .impacts()
            .iter()
            .all(|impact| impact.affected_edges().len() == 2)
    );
}

#[test]
fn long_fanout_chain_is_iterative_and_closure_complete() {
    let candidate = qualified_h2_candidate();
    let mut nodes = vec![node(
        "h2-package",
        DependencyGraphNodeKindV1::CargoPackage,
        Some(candidate.current().identity_sha256()),
    )];
    let mut edges = Vec::new();
    let mut dependency = "h2-package".to_owned();
    for index in 0..2_048 {
        let dependent = format!("consumer-{index}");
        nodes.push(node(
            &dependent,
            DependencyGraphNodeKindV1::CargoTarget,
            None,
        ));
        edges.push(edge(
            &dependent,
            &dependency,
            DependencyGraphEdgeKindV1::NormalTarget,
        ));
        dependency = dependent;
    }
    let graph = DependencyGraphV1::try_new(safe_envelope(), nodes, edges).unwrap();
    let batch = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            continue_dependency_impact,
        )
        .unwrap();
    assert_eq!(batch.impacts()[0].affected_nodes().len(), 2_049);
    assert_eq!(batch.impacts()[0].affected_edges().len(), 2_048);
}

#[test]
fn conservative_complete_facts_may_widen_the_safe_closure() {
    let candidate = qualified_h2_candidate();
    let envelope = complete_envelope(
        vec![FactEvidenceClassV1::Observed],
        FactCertaintyV1::Conservative,
        FactCoverageV1::CompleteForScope {
            scope_sha256: DigestV1::of(b"observed-scope"),
            exclusions_sha256: DigestV1::of(b"observed-exclusions"),
        },
        100,
        300,
    );
    let graph = DependencyGraphV1::try_new(
        envelope,
        vec![node(
            "h2-package",
            DependencyGraphNodeKindV1::CargoPackage,
            Some(candidate.current().identity_sha256()),
        )],
        Vec::new(),
    )
    .unwrap();
    assert!(
        graph
            .try_analyze_candidates(
                std::slice::from_ref(&candidate),
                LifecycleTimestampV1::from_unix_seconds(200),
                continue_dependency_impact,
            )
            .is_ok()
    );
}

#[test]
fn every_declared_dependency_role_participates_in_impact() {
    let candidate = qualified_h2_candidate();
    let mut nodes = vec![node(
        "h2-package",
        DependencyGraphNodeKindV1::CargoPackage,
        Some(candidate.current().identity_sha256()),
    )];
    let roles = [
        DependencyGraphEdgeKindV1::NormalTarget,
        DependencyGraphEdgeKindV1::BuildHost,
        DependencyGraphEdgeKindV1::DevTarget,
        DependencyGraphEdgeKindV1::ProcMacroHost,
        DependencyGraphEdgeKindV1::NativeHost,
        DependencyGraphEdgeKindV1::FeatureActivation,
        DependencyGraphEdgeKindV1::ConfiguredBuck,
    ];
    let mut edges = Vec::new();
    for (index, role) in roles.into_iter().enumerate() {
        let dependent = format!("role-{index}");
        nodes.push(node(
            &dependent,
            DependencyGraphNodeKindV1::CargoTarget,
            None,
        ));
        edges.push(edge(&dependent, "h2-package", role));
    }
    let graph = DependencyGraphV1::try_new(safe_envelope(), nodes, edges).unwrap();
    let batch = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            continue_dependency_impact,
        )
        .unwrap();
    assert_eq!(batch.impacts()[0].affected_nodes().len(), 8);
    assert_eq!(batch.impacts()[0].affected_edges().len(), 7);
}
