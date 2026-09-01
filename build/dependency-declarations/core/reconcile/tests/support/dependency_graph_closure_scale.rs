use super::dependency_graph::*;
use dependency_declarations_reconcile::*;

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
