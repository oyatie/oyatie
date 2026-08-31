#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DependencyImpactCountsV1 {
    root_nodes: usize,
    affected_nodes: usize,
    affected_edges: usize,
}

struct DependencyImpactShapeV1 {
    counts: Vec<DependencyImpactCountsV1>,
    graph_node_indices: Vec<u32>,
    graph_edge_indices: Vec<u32>,
}

fn count_dependency_impact_selections<C>(
    graph: &DependencyGraphV1,
    candidates: &[&DependencyCandidateV1],
    word_count: usize,
    memberships: &[u64],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DependencyImpactShapeV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let mut counts =
        lifecycle_try_filled_vec(candidates.len(), DependencyImpactCountsV1::default())?;
    let mut graph_node_indices = Vec::new();
    let mut graph_edge_indices = Vec::new();
    for (candidate_index, candidate) in candidates.iter().enumerate() {
        counts[candidate_index].root_nodes = graph
            .release_roots(candidate.current().identity_sha256())
            .len();
    }
    for node_index in 0..graph.nodes().len() {
        let membership = dependency_node_membership(graph, node_index, word_count, memberships);
        control.record_work()?;
        if membership.iter().all(|word| *word == 0) {
            continue;
        }
        lifecycle_try_push(&mut graph_node_indices, lifecycle_u32_index(node_index)?)?;
        control.visit_node()?;
        for candidate in DependencyCandidateMembershipIterV1::new(membership, candidates.len()) {
            counts[candidate].affected_nodes = counts[candidate]
                .affected_nodes
                .checked_add(1)
                .ok_or_else(lifecycle_bounds)?;
            control.record_work()?;
        }
    }
    control.checkpoint_and_reset()?;
    for edge_index in 0..graph.edges().len() {
        let dependency = graph.edge_dependency_node(edge_index);
        let membership = dependency_node_membership(graph, dependency, word_count, memberships);
        control.record_work()?;
        if membership.iter().all(|word| *word == 0) {
            continue;
        }
        lifecycle_try_push(&mut graph_edge_indices, lifecycle_u32_index(edge_index)?)?;
        control.visit_edge()?;
        for candidate in DependencyCandidateMembershipIterV1::new(membership, candidates.len()) {
            counts[candidate].affected_edges = counts[candidate]
                .affected_edges
                .checked_add(1)
                .ok_or_else(lifecycle_bounds)?;
            control.record_work()?;
        }
    }
    control.checkpoint_and_reset()?;
    validate_dependency_impact_counts(&counts)?;
    Ok(DependencyImpactShapeV1 {
        counts,
        graph_node_indices,
        graph_edge_indices,
    })
}

fn dependency_impact_ranges(
    counts: &[DependencyImpactCountsV1],
) -> Result<(Vec<DependencyImpactRangesV1>, usize), LifecycleFailureV1> {
    let mut ranges = lifecycle_try_vec(counts.len())?;
    let mut root_nodes = 0_usize;
    let mut affected_nodes = 0_usize;
    let mut affected_edges = 0_usize;
    for count in counts {
        let next_root_nodes = root_nodes
            .checked_add(count.root_nodes)
            .ok_or_else(lifecycle_bounds)?;
        let next_affected_nodes = affected_nodes
            .checked_add(count.affected_nodes)
            .ok_or_else(lifecycle_bounds)?;
        let next_affected_edges = affected_edges
            .checked_add(count.affected_edges)
            .ok_or_else(lifecycle_bounds)?;
        ranges.push(DependencyImpactRangesV1 {
            root_nodes: root_nodes..next_root_nodes,
            affected_nodes: affected_nodes..next_affected_nodes,
            affected_edges: affected_edges..next_affected_edges,
        });
        root_nodes = next_root_nodes;
        affected_nodes = next_affected_nodes;
        affected_edges = next_affected_edges;
    }
    let selection_count = root_nodes
        .checked_add(affected_nodes)
        .and_then(|count| count.checked_add(affected_edges))
        .ok_or_else(lifecycle_bounds)?;
    let selection_bytes = lifecycle_allocation_bytes::<u32>(selection_count)?;
    if selection_bytes > LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_SELECTION_BYTES {
        return Err(lifecycle_bounds());
    }
    Ok((ranges, selection_bytes))
}

fn dependency_impact_result_bytes(
    graph_node_count: usize,
    graph_edge_count: usize,
    selection_bytes: usize,
    candidate_count: usize,
) -> Result<usize, LifecycleFailureV1> {
    let mut retained = selection_bytes;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<DependencyGraphNodeV1>(graph_node_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<DependencyGraphEdgeV1>(graph_edge_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<DependencyImpactV1>(candidate_count)?,
    )?;
    retained = lifecycle_add_bytes(retained, 64 * 1024)?;
    if retained > LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_RESULT_BYTES {
        return Err(lifecycle_bounds());
    }
    Ok(retained)
}

fn dependency_impact_working_bytes(
    membership_bytes: usize,
    result_bytes: usize,
    candidate_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
) -> Result<usize, LifecycleFailureV1> {
    let mut working = lifecycle_add_bytes(membership_bytes, result_bytes)?;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<u32>(
            graph_node_count
                .checked_add(graph_edge_count)
                .ok_or_else(lifecycle_bounds)?,
        )?,
    )?;
    for bytes in [
        lifecycle_allocation_bytes::<DependencyImpactCountsV1>(candidate_count)?,
        lifecycle_allocation_bytes::<DependencyImpactRangesV1>(candidate_count)?,
        lifecycle_allocation_bytes::<DependencyImpactCursorsV1>(candidate_count)?,
        lifecycle_allocation_bytes::<&DependencyCandidateV1>(candidate_count)?,
    ] {
        working = lifecycle_add_bytes(working, bytes)?;
    }
    Ok(working)
}

fn dependency_impact_retained_bytes(
    selections: &DependencyImpactSelectionsV1,
    candidate_count: usize,
) -> Result<usize, LifecycleFailureV1> {
    let selection_capacity = selections
        .root_nodes
        .capacity()
        .checked_add(selections.affected_nodes.capacity())
        .and_then(|count| count.checked_add(selections.affected_edges.capacity()))
        .ok_or_else(lifecycle_bounds)?;
    dependency_impact_result_bytes(
        selections.graph_nodes.capacity(),
        selections.graph_edges.capacity(),
        lifecycle_allocation_bytes::<u32>(selection_capacity)?,
        candidate_count,
    )
}

fn validate_dependency_impact_counts(
    counts: &[DependencyImpactCountsV1],
) -> Result<(), LifecycleFailureV1> {
    let mut total_nodes = 0_usize;
    let mut total_edges = 0_usize;
    for count in counts {
        total_nodes = total_nodes
            .checked_add(count.affected_nodes)
            .ok_or_else(lifecycle_bounds)?;
        total_edges = total_edges
            .checked_add(count.affected_edges)
            .ok_or_else(lifecycle_bounds)?;
    }
    if total_nodes > LifecycleBoundsV1::MAX_TOTAL_DEPENDENCY_IMPACT_NODES
        || total_edges > LifecycleBoundsV1::MAX_TOTAL_DEPENDENCY_IMPACT_EDGES
    {
        return Err(lifecycle_bounds());
    }
    Ok(())
}

#[cfg(test)]
mod dependency_impact_budget_tests {
    use super::*;

    #[test]
    fn selection_count_cannot_bypass_the_result_byte_ceiling() {
        let excessive_entries =
            LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_SELECTION_BYTES / std::mem::size_of::<u32>()
                + 1;
        let counts = [DependencyImpactCountsV1 {
            root_nodes: 0,
            affected_nodes: excessive_entries,
            affected_edges: 0,
        }];

        let failure = dependency_impact_ranges(&counts).unwrap_err();

        assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    }

    #[test]
    fn affected_graph_count_cannot_bypass_the_complete_result_ceiling() {
        let excessive_edges = LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_RESULT_BYTES
            / std::mem::size_of::<DependencyGraphEdgeV1>()
            + 1;

        let failure = dependency_impact_result_bytes(0, excessive_edges, 0, 0).unwrap_err();

        assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    }

    #[test]
    fn retained_budget_includes_every_candidate_result_entry() {
        let without_candidates = dependency_impact_result_bytes(0, 0, 0, 0).unwrap();
        let candidate_count = 1_024;
        let with_candidates =
            dependency_impact_result_bytes(0, 0, 0, candidate_count).unwrap();

        assert_eq!(
            with_candidates - without_candidates,
            std::mem::size_of::<DependencyImpactV1>() * candidate_count
        );
    }
}
