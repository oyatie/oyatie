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
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes,
        edges,
        continue_dependency_graph_construction,
    )
    .unwrap();
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
        continue_dependency_graph_construction,
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
    let graph = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes,
        edges,
        continue_dependency_graph_construction,
    )
    .unwrap();
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

#[test]
fn condensed_batch_matches_reference_closure_for_every_three_node_graph() {
    let h2 = qualified_h2_candidate();
    let tokio = qualified_candidate("tokio", "1.47.0", "1.48.0");
    let names = ["h2-package", "tokio-package", "consumer"];
    let nodes = vec![
        node(
            names[0],
            DependencyGraphNodeKindV1::CargoPackage,
            Some(h2.current().identity_sha256()),
        ),
        node(
            names[1],
            DependencyGraphNodeKindV1::CargoPackage,
            Some(tokio.current().identity_sha256()),
        ),
        node(names[2], DependencyGraphNodeKindV1::CargoTarget, None),
    ];
    let possible_edges = (0..names.len())
        .flat_map(|dependent| {
            (0..names.len())
                .filter(move |dependency| *dependency != dependent)
                .map(move |dependency| (dependent, dependency))
        })
        .collect::<Vec<_>>();

    for edge_mask in 0_u64..(1_u64 << possible_edges.len()) {
        let selected_pairs = possible_edges
            .iter()
            .enumerate()
            .filter(|(index, _)| edge_mask & (1_u64 << index) != 0)
            .map(|(_, pair)| *pair)
            .collect::<Vec<_>>();
        let edges = selected_pairs
            .iter()
            .map(|(dependent, dependency)| {
                edge(
                    names[*dependent],
                    names[*dependency],
                    DependencyGraphEdgeKindV1::NormalTarget,
                )
            })
            .collect::<Vec<_>>();
        let graph = DependencyGraphV1::try_new(
            safe_envelope(),
            nodes.clone(),
            edges.clone(),
            continue_dependency_graph_construction,
        )
        .unwrap();
        let batch = graph
            .try_analyze_candidates(
                &[h2.clone(), tokio.clone()],
                LifecycleTimestampV1::from_unix_seconds(200),
                continue_dependency_impact,
            )
            .unwrap();

        for (candidate, root) in [(&h2, 0_usize), (&tokio, 1_usize)] {
            let impact = batch
                .impacts()
                .iter()
                .find(|impact| {
                    impact.current_release_identity_sha256()
                        == candidate.current().identity_sha256()
                })
                .unwrap();
            let affected = reference_dependency_closure(names.len(), &selected_pairs, root);
            let mut expected_nodes = nodes
                .iter()
                .enumerate()
                .filter(|(index, _)| affected[*index])
                .map(|(_, node)| node.unit_identity_sha256())
                .collect::<Vec<_>>();
            let mut actual_nodes = impact
                .affected_nodes()
                .iter()
                .map(DependencyGraphNodeV1::unit_identity_sha256)
                .collect::<Vec<_>>();
            let mut expected_edges = edges
                .iter()
                .zip(&selected_pairs)
                .filter(|(_, (_, dependency))| affected[*dependency])
                .map(|(edge, _)| edge.identity_sha256())
                .collect::<Vec<_>>();
            let mut actual_edges = impact
                .affected_edges()
                .iter()
                .map(DependencyGraphEdgeV1::identity_sha256)
                .collect::<Vec<_>>();
            expected_nodes.sort_unstable();
            actual_nodes.sort_unstable();
            expected_edges.sort_unstable();
            actual_edges.sort_unstable();
            assert_eq!(actual_nodes, expected_nodes, "edge mask {edge_mask:#08b}");
            assert_eq!(actual_edges, expected_edges, "edge mask {edge_mask:#08b}");
        }
    }
}

fn reference_dependency_closure(
    node_count: usize,
    edges: &[(usize, usize)],
    root: usize,
) -> Vec<bool> {
    let mut affected = vec![false; node_count];
    affected[root] = true;
    loop {
        let mut changed = false;
        for (dependent, dependency) in edges {
            if affected[*dependency] && !affected[*dependent] {
                affected[*dependent] = true;
                changed = true;
            }
        }
        if !changed {
            return affected;
        }
    }
}
