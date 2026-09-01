fn analyze_dependency_impact_batch<C>(
    graph: &DependencyGraphV1,
    candidates: &[&DependencyCandidateV1],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DependencyImpactBatchV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let (word_count, memberships, membership_bytes) =
        dependency_candidate_memberships(graph, candidates, control)?;

    let shape = count_dependency_impact_selections(
        graph,
        candidates,
        word_count,
        &memberships,
        control,
    )?;
    let (ranges, selection_bytes) = dependency_impact_ranges(&shape.counts)?;
    let requested_result_bytes = dependency_impact_result_bytes(
        shape.graph_node_indices.len(),
        shape.graph_edge_indices.len(),
        selection_bytes,
        candidates.len(),
    )?;
    let working_bytes = dependency_impact_working_bytes(
        membership_bytes,
        requested_result_bytes,
        candidates.len(),
        shape.graph_node_indices.capacity(),
        shape.graph_edge_indices.capacity(),
    )?;
    if working_bytes > LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_WORKING_BYTES {
        return Err(lifecycle_bounds());
    }
    let selections = fill_dependency_impact_selections(
        graph,
        candidates,
        word_count,
        &memberships,
        &shape,
        &ranges,
        control,
    )?;
    let retained_bytes_upper_bound =
        dependency_impact_retained_bytes(&selections, candidates.len())?;
    let working_bytes = dependency_impact_working_bytes(
        membership_bytes,
        retained_bytes_upper_bound,
        candidates.len(),
        shape.graph_node_indices.capacity(),
        shape.graph_edge_indices.capacity(),
    )?;
    if working_bytes > LifecycleBoundsV1::MAX_DEPENDENCY_IMPACT_WORKING_BYTES {
        return Err(lifecycle_bounds());
    }
    let storage = std::sync::Arc::new(DependencyImpactStorageV1::new(
        graph,
        selections,
        selection_bytes,
        retained_bytes_upper_bound,
    ));
    let mut impacts = lifecycle_try_vec(candidates.len())?;
    for (candidate, ranges) in candidates.iter().zip(ranges) {
        impacts.push(DependencyImpactV1::try_from_ranges(
            std::sync::Arc::clone(&storage),
            candidate,
            ranges,
            control,
        )?);
        control.complete_candidate()?;
    }
    dependency_impact_batch(storage, impacts, control)
}
