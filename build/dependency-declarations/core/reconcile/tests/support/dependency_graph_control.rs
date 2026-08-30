use super::dependency_graph::*;
use dependency_declarations_reconcile::*;

fn chain_graph(candidate: &DependencyCandidateV1, consumer_count: usize) -> DependencyGraphV1 {
    let mut nodes = vec![node(
        "dependency-package",
        DependencyGraphNodeKindV1::CargoPackage,
        Some(candidate.current().identity_sha256()),
    )];
    let mut edges = Vec::with_capacity(consumer_count);
    let mut dependency = "dependency-package".to_owned();
    for index in 0..consumer_count {
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
    DependencyGraphV1::try_new(
        safe_envelope(),
        nodes,
        edges,
        continue_dependency_graph_construction,
    )
    .unwrap()
}

#[test]
fn graph_construction_cancellation_refuses_before_graph_work() {
    let nodes = vec![node(
        "only-node",
        DependencyGraphNodeKindV1::CargoTarget,
        None,
    )];
    let mut checkpoints = Vec::new();
    let failure = DependencyGraphV1::try_new(safe_envelope(), nodes, Vec::new(), |progress| {
        checkpoints.push(progress);
        LifecycleControlDecisionV1::Cancel
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyGraphConstructionCancelled
    );
    assert_eq!(
        checkpoints,
        vec![DependencyGraphConstructionProgressV1::default()]
    );
}

#[test]
fn graph_construction_deadline_refuses_at_a_bounded_node_checkpoint() {
    let nodes = (0..4_096)
        .map(|index| {
            node(
                &format!("node-{index:06}"),
                DependencyGraphNodeKindV1::CargoTarget,
                None,
            )
        })
        .collect();
    let mut checkpoints = Vec::new();
    let failure = DependencyGraphV1::try_new(safe_envelope(), nodes, Vec::new(), |progress| {
        checkpoints.push(progress);
        if progress.completed_nodes() >= 1_024 {
            LifecycleControlDecisionV1::DeadlineExceeded
        } else {
            LifecycleControlDecisionV1::Continue
        }
    })
    .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyGraphConstructionDeadlineExceeded
    );
    assert_eq!(checkpoints.last().unwrap().completed_nodes(), 1_024);
    assert_eq!(checkpoints.last().unwrap().completed_edges(), 0);
}

#[test]
fn continuing_graph_construction_preserves_identity_and_final_progress() {
    let nodes = vec![
        node("dependency", DependencyGraphNodeKindV1::CargoPackage, None),
        node("middle", DependencyGraphNodeKindV1::CargoTarget, None),
        node("consumer", DependencyGraphNodeKindV1::BuckTarget, None),
    ];
    let edges = vec![
        edge(
            "middle",
            "dependency",
            DependencyGraphEdgeKindV1::NormalTarget,
        ),
        edge(
            "consumer",
            "middle",
            DependencyGraphEdgeKindV1::ConfiguredBuck,
        ),
    ];
    let mut final_progress = None;
    let controlled =
        DependencyGraphV1::try_new(safe_envelope(), nodes.clone(), edges.clone(), |progress| {
            final_progress = Some(progress);
            LifecycleControlDecisionV1::Continue
        })
        .unwrap();
    let reversed = DependencyGraphV1::try_new(
        safe_envelope(),
        nodes.into_iter().rev().collect(),
        edges.into_iter().rev().collect(),
        continue_dependency_graph_construction,
    )
    .unwrap();

    assert_eq!(controlled.identity_sha256(), reversed.identity_sha256());
    let progress = final_progress.unwrap();
    assert_eq!(progress.completed_nodes(), 3);
    assert_eq!(progress.completed_edges(), 2);
}

#[test]
fn cancellation_refuses_before_dependency_work() {
    let candidate = qualified_h2_candidate();
    let graph = chain_graph(&candidate, 8);
    let mut checkpoints = Vec::new();
    let failure = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                checkpoints.push(progress);
                LifecycleControlDecisionV1::Cancel
            },
        )
        .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyImpactCancelled
    );
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].completed_candidates(), 0);
    assert_eq!(checkpoints[0].visited_nodes(), 0);
    assert_eq!(checkpoints[0].visited_edges(), 0);
}

#[test]
fn deadline_refuses_at_a_bounded_mid_closure_checkpoint() {
    let candidate = qualified_h2_candidate();
    let graph = chain_graph(&candidate, 4_096);
    let mut checkpoint_count = 0;
    let failure = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                checkpoint_count += 1;
                if progress.visited_edges() >= 1_000 {
                    LifecycleControlDecisionV1::DeadlineExceeded
                } else {
                    LifecycleControlDecisionV1::Continue
                }
            },
        )
        .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyImpactDeadlineExceeded
    );
    assert!(checkpoint_count > 1);
    assert!(checkpoint_count < 16);
}

#[test]
fn deadline_refuses_during_bounded_impact_materialization() {
    let candidate = qualified_h2_candidate();
    let graph = chain_graph(&candidate, 2_048);
    let mut final_progress = None;
    let failure = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                final_progress = Some(progress);
                if progress.materialized_nodes() >= 1_024 {
                    LifecycleControlDecisionV1::DeadlineExceeded
                } else {
                    LifecycleControlDecisionV1::Continue
                }
            },
        )
        .unwrap_err();

    assert_eq!(
        failure.class(),
        LifecycleFailureClassV1::DependencyImpactDeadlineExceeded
    );
    let progress = final_progress.unwrap();
    assert_eq!(progress.visited_nodes(), 2_049);
    assert_eq!(progress.visited_edges(), 2_048);
    assert_eq!(progress.materialized_root_nodes(), 1);
    assert_eq!(progress.materialized_nodes(), 1_024);
    assert_eq!(progress.materialized_edges(), 0);
}

#[test]
fn continuing_control_preserves_the_canonical_result() {
    let candidate = qualified_h2_candidate();
    let graph = chain_graph(&candidate, 2_048);
    let expected = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            continue_dependency_impact,
        )
        .unwrap();
    let mut final_progress = None;
    let actual = graph
        .try_analyze_candidates(
            std::slice::from_ref(&candidate),
            LifecycleTimestampV1::from_unix_seconds(200),
            |progress| {
                final_progress = Some(progress);
                LifecycleControlDecisionV1::Continue
            },
        )
        .unwrap();

    assert_eq!(actual, expected);
    let progress = final_progress.unwrap();
    assert_eq!(progress.completed_candidates(), 1);
    assert_eq!(progress.visited_nodes(), 2_049);
    assert_eq!(progress.visited_edges(), 2_048);
    assert_eq!(progress.materialized_root_nodes(), 1);
    assert_eq!(progress.materialized_nodes(), 2_049);
    assert_eq!(progress.materialized_edges(), 2_048);
}
