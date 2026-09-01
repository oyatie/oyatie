#[derive(Clone, Copy)]
struct DependencyImpactCursorsV1 {
    root_nodes: usize,
    affected_nodes: usize,
    affected_edges: usize,
}

struct DependencyImpactSelectionsV1 {
    graph_nodes: Vec<DependencyGraphNodeV1>,
    graph_edges: Vec<DependencyGraphEdgeV1>,
    root_nodes: Vec<u32>,
    affected_nodes: Vec<u32>,
    affected_edges: Vec<u32>,
}

fn fill_dependency_impact_selections<C>(
    graph: &DependencyGraphV1,
    candidates: &[&DependencyCandidateV1],
    word_count: usize,
    memberships: &[u64],
    shape: &DependencyImpactShapeV1,
    ranges: &[DependencyImpactRangesV1],
    control: &mut DependencyImpactControlV1<C>,
) -> Result<DependencyImpactSelectionsV1, LifecycleFailureV1>
where
    C: FnMut(DependencyImpactProgressV1) -> LifecycleControlDecisionV1,
{
    let mut cursors = lifecycle_try_vec(ranges.len())?;
    for range in ranges {
        cursors.push(DependencyImpactCursorsV1 {
            root_nodes: range.root_nodes.start,
            affected_nodes: range.affected_nodes.start,
            affected_edges: range.affected_edges.start,
        });
    }
    let root_length = ranges.last().map_or(0, |range| range.root_nodes.end);
    let node_length = ranges.last().map_or(0, |range| range.affected_nodes.end);
    let edge_length = ranges.last().map_or(0, |range| range.affected_edges.end);
    let mut root_nodes = lifecycle_try_filled_vec(root_length, 0_u32)?;
    let mut affected_nodes = lifecycle_try_filled_vec(node_length, 0_u32)?;
    let mut affected_edges = lifecycle_try_filled_vec(edge_length, 0_u32)?;
    let mut graph_nodes = lifecycle_try_vec(shape.graph_node_indices.len())?;
    for node_index in &shape.graph_node_indices {
        graph_nodes.push(graph.nodes()[*node_index as usize].clone());
        control.record_work()?;
    }
    let mut graph_edges = lifecycle_try_vec(shape.graph_edge_indices.len())?;
    for edge_index in &shape.graph_edge_indices {
        graph_edges.push(graph.edges()[*edge_index as usize].clone());
        control.record_work()?;
    }
    control.checkpoint_and_reset()?;

    for (candidate_index, candidate) in candidates.iter().enumerate() {
        for root in graph.release_roots(candidate.current().identity_sha256()) {
            let cursor = &mut cursors[candidate_index].root_nodes;
            let graph_node_index = lifecycle_u32_index(root.node_index)?;
            let selected_index = shape
                .graph_node_indices
                .binary_search(&graph_node_index)
                .map_err(|_| lifecycle_internal())?;
            root_nodes[*cursor] = lifecycle_u32_index(selected_index)?;
            *cursor += 1;
            control.materialize_root_node()?;
        }
    }
    control.checkpoint_and_reset()?;
    for (selected_index, node_index) in shape.graph_node_indices.iter().enumerate() {
        let membership =
            dependency_node_membership(graph, *node_index as usize, word_count, memberships);
        for candidate in DependencyCandidateMembershipIterV1::new(membership, candidates.len()) {
            let cursor = &mut cursors[candidate].affected_nodes;
            affected_nodes[*cursor] = lifecycle_u32_index(selected_index)?;
            *cursor += 1;
            control.materialize_node()?;
        }
    }
    control.checkpoint_and_reset()?;
    for (selected_index, edge_index) in shape.graph_edge_indices.iter().enumerate() {
        let dependency = graph.edge_dependency_node(*edge_index as usize);
        let membership = dependency_node_membership(graph, dependency, word_count, memberships);
        for candidate in DependencyCandidateMembershipIterV1::new(membership, candidates.len()) {
            let cursor = &mut cursors[candidate].affected_edges;
            affected_edges[*cursor] = lifecycle_u32_index(selected_index)?;
            *cursor += 1;
            control.materialize_edge()?;
        }
    }
    control.checkpoint_and_reset()?;
    for (cursor, range) in cursors.iter().zip(ranges) {
        if cursor.root_nodes != range.root_nodes.end
            || cursor.affected_nodes != range.affected_nodes.end
            || cursor.affected_edges != range.affected_edges.end
        {
            return Err(lifecycle_internal());
        }
    }
    Ok(DependencyImpactSelectionsV1 {
        graph_nodes,
        graph_edges,
        root_nodes,
        affected_nodes,
        affected_edges,
    })
}
