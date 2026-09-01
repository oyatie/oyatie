fn validate_dependency_graph_memory(
    nodes: &Vec<DependencyGraphNodeV1>,
    edges: &Vec<DependencyGraphEdgeV1>,
) -> Result<usize, LifecycleFailureV1> {
    dependency_graph_memory_upper_bounds(
        nodes.len(),
        nodes.capacity(),
        edges.len(),
        edges.capacity(),
    )
    .map(|(retained, _)| retained)
}

fn dependency_graph_memory_upper_bounds(
    node_count: usize,
    node_capacity: usize,
    edge_count: usize,
    edge_capacity: usize,
) -> Result<(usize, usize), LifecycleFailureV1> {
    let offset_count = node_count.checked_add(1).ok_or_else(lifecycle_bounds)?;

    let mut retained = lifecycle_allocation_bytes::<DependencyGraphNodeV1>(node_capacity)?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<DependencyGraphEdgeV1>(edge_capacity)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<usize>(offset_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<usize>(edge_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<u32>(edge_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<DependencyReleaseRootV1>(node_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<u32>(node_count.checked_mul(2).ok_or_else(lifecycle_bounds)?)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<usize>(offset_count)?,
    )?;
    retained = lifecycle_add_bytes(
        retained,
        lifecycle_allocation_bytes::<u32>(edge_count)?,
    )?;
    retained = lifecycle_add_bytes(retained, 64 * 1024)?;
    if retained > LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_RETAINED_BYTES {
        return Err(lifecycle_bounds());
    }

    let mut working = retained;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<(DigestV1, usize)>(
            node_count.checked_mul(2).ok_or_else(lifecycle_bounds)?,
        )?,
    )?;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<(usize, usize)>(edge_count)?,
    )?;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<usize>(
            node_count.checked_mul(10).ok_or_else(lifecycle_bounds)?,
        )?,
    )?;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<u32>(
            node_count.checked_mul(2).ok_or_else(lifecycle_bounds)?,
        )?,
    )?;
    working = lifecycle_add_bytes(
        working,
        lifecycle_allocation_bytes::<(u32, u32)>(edge_count)?,
    )?;
    if working > LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_WORKING_BYTES {
        return Err(lifecycle_bounds());
    }
    Ok((retained, working))
}

#[cfg(test)]
mod dependency_graph_memory_tests {
    use super::*;

    #[test]
    fn count_limits_cannot_bypass_the_graph_byte_ceiling() {
        let failure = dependency_graph_memory_upper_bounds(
            LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_NODES,
            LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_NODES,
            LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_EDGES,
            LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_EDGES,
        )
        .unwrap_err();

        assert_eq!(failure.class(), LifecycleFailureClassV1::BoundsExceeded);
    }

    #[test]
    fn small_graph_has_explicit_retained_and_working_upper_bounds() {
        let (retained, working) = dependency_graph_memory_upper_bounds(5, 5, 4, 4).unwrap();

        assert!(retained > 0);
        assert!(working >= retained);
        assert!(working <= LifecycleBoundsV1::MAX_DEPENDENCY_GRAPH_WORKING_BYTES);
    }
}
