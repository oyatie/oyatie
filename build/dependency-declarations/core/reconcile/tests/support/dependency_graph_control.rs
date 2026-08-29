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
    DependencyGraphV1::try_new(safe_envelope(), nodes, edges).unwrap()
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
                DependencyImpactControlDecisionV1::Cancel
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
                    DependencyImpactControlDecisionV1::DeadlineExceeded
                } else {
                    DependencyImpactControlDecisionV1::Continue
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
                DependencyImpactControlDecisionV1::Continue
            },
        )
        .unwrap();

    assert_eq!(actual, expected);
    let progress = final_progress.unwrap();
    assert_eq!(progress.completed_candidates(), 1);
    assert_eq!(progress.visited_nodes(), 2_049);
    assert_eq!(progress.visited_edges(), 2_048);
}
