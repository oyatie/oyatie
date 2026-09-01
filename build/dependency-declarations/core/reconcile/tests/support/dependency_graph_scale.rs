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
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes,
        edges,
        continue_dependency_graph_construction,
    )
    .unwrap();
    let now = LifecycleTimestampV1::from_unix_seconds(200);
    let mut final_progress = None;
    let forward = graph
        .try_analyze_candidates(&[h2.clone(), tokio.clone()], now, |progress| {
            final_progress = Some(progress);
            LifecycleControlDecisionV1::Continue
        })
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
    let progress = final_progress.unwrap();
    assert_eq!(progress.visited_nodes(), 5);
    assert_eq!(progress.visited_edges(), 4);
    assert_eq!(progress.materialized_root_nodes(), 2);
    assert_eq!(progress.materialized_nodes(), 6);
    assert_eq!(progress.materialized_edges(), 4);
    assert_eq!(forward.selection_bytes(), 48);
    assert!(forward.retained_bytes_upper_bound() > forward.selection_bytes());
    assert!(
        forward.retained_bytes_upper_bound()
            <= LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_RESULT_BYTES
    );
    let debug = format!("{forward:?}");
    assert!(debug.len() < 1_024);
    assert!(!debug.contains("DependencyGraphNodeV1"));
}

#[test]
fn candidate_roots_inside_one_cycle_share_one_condensed_traversal() {
    let h2 = qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        vec![
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
            node("shared-app", DependencyGraphNodeKindV1::BuckTarget, None),
        ],
        vec![
            edge(
                "h2-package",
                "tokio-package",
                DependencyGraphEdgeKindV1::NormalTarget,
            ),
            edge(
                "tokio-package",
                "h2-package",
                DependencyGraphEdgeKindV1::NormalTarget,
            ),
            edge(
                "shared-app",
                "h2-package",
                DependencyGraphEdgeKindV1::ConfiguredBuck,
            ),
        ],
        continue_dependency_graph_construction,
    )
    .unwrap();
    let mut final_progress = None;
    let batch = graph
        .try_analyze_candidates(
            &[h2, tokio],
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                final_progress = Some(progress);
                LifecycleControlDecisionV1::Continue
            },
        )
        .unwrap();

    assert_eq!(batch.impacts().len(), 2);
    assert!(
        batch
            .impacts()
            .iter()
            .all(|impact| impact.affected_nodes().len() == 3)
    );
    assert!(
        batch
            .impacts()
            .iter()
            .all(|impact| impact.affected_edges().len() == 3)
    );
    let progress = final_progress.unwrap();
    assert_eq!(progress.visited_nodes(), 3);
    assert_eq!(progress.visited_edges(), 3);
    assert_eq!(progress.materialized_nodes(), 6);
    assert_eq!(progress.materialized_edges(), 6);
    assert_eq!(batch.selection_bytes(), 56);
    assert!(
        batch.retained_bytes_upper_bound() <= LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_RESULT_BYTES
    );
}

#[test]
fn candidate_membership_crosses_the_first_word_without_repeating_graph_work() {
    let candidates = (0..65)
        .map(|index| qualified_candidate(&format!("dependency-{index}"), "1.0.0", "1.0.1"))
        .collect::<Vec<_>>();
    let mut nodes = Vec::with_capacity(candidates.len() + 1);
    let mut edges = Vec::with_capacity(candidates.len());
    for (index, candidate) in candidates.iter().enumerate() {
        let package = format!("dependency-{index}-package");
        nodes.push(node(
            &package,
            DependencyGraphNodeKindV1::CargoPackage,
            Some(candidate.current().identity_sha256()),
        ));
        edges.push(edge(
            "shared-consumer",
            &package,
            DependencyGraphEdgeKindV1::NormalTarget,
        ));
    }
    nodes.push(node(
        "shared-consumer",
        DependencyGraphNodeKindV1::CargoTarget,
        None,
    ));
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes,
        edges,
        continue_dependency_graph_construction,
    )
    .unwrap();
    let mut final_progress = None;
    let batch = graph
        .try_analyze_candidates(
            &candidates,
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                final_progress = Some(progress);
                LifecycleControlDecisionV1::Continue
            },
        )
        .unwrap();

    assert_eq!(batch.impacts().len(), 65);
    assert!(
        batch
            .impacts()
            .iter()
            .all(|impact| impact.affected_nodes().len() == 2)
    );
    assert!(
        batch
            .impacts()
            .iter()
            .all(|impact| impact.affected_edges().len() == 1)
    );
    let progress = final_progress.unwrap();
    assert_eq!(progress.visited_nodes(), 66);
    assert_eq!(progress.visited_edges(), 65);
    assert_eq!(progress.materialized_root_nodes(), 65);
    assert_eq!(progress.materialized_nodes(), 130);
    assert_eq!(progress.materialized_edges(), 65);
    assert_eq!(batch.selection_bytes(), 1_040);
}
